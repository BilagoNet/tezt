#!/usr/bin/env python3
"""tezt_worker.py -- persistent Python test-execution worker for the `tezt` runner.

Speaks JSON Lines over stdio with the Rust supervisor:
  in : {"cmd":"run","batch_id":N,"items":[{"id","file","qualname"}]} | {"cmd":"shutdown"}
  out: ready / result / batch_done / bye / fatal events (one JSON object per line)

Pure stdlib, Python 3.8+ compatible. Single file by design.
"""

import argparse
import ast
import asyncio
import collections
import difflib
import inspect
import io
import importlib.util
import itertools
import json
import linecache
import logging
import math
import os
import re
import shutil
import sys
import tempfile
import time
import traceback as tb_module
import types
import warnings

# ============================================================================
# Constants / globals
# ============================================================================

CAPTURE_LIMIT = 64 * 1024          # truncate captured stdout/stderr fields
TRACEBACK_LINES = 50               # keep last N lines of tracebacks
TB_STYLE = "auto"                  # --tb: auto|long|short|line|no (set in main())
DEBUG = os.environ.get("TEZT_DEBUG") == "1"

_REAL_STDOUT = None                # protocol channel (saved before any capture)
_REAL_STDERR = None
ROOTDIR = None
NO_CAPTURE = False

# Hook lifecycle objects (built once per worker in main()); see the hook
# section below. _CONFIG/_SESSION are passed to configure/session hooks and
# carried on every HookItem.
_CONFIG = None
_SESSION = None

# coverage.py controller, created lazily in main() ONLY when --cov-data-dir is
# given. The `coverage` package is third-party, so it is imported lazily there
# (never at module top level) to keep the worker stdlib-only by default.
_COVERAGE = None


def debug(msg):
    if DEBUG:
        print("[tezt-debug] %s" % msg, file=_REAL_STDERR or sys.__stderr__)


# ============================================================================
# Internal outcome exceptions (also raised by the virtual `tezt` module).
# Class-name suffixes intentionally match pytest's ("Skipped", "Failed",
# "XFailed") so the same suffix-based classification handles both libraries.
# ============================================================================

class Skipped(Exception):
    pass


class Failed(Exception):
    pass


class XFailed(Exception):
    pass


def _classify_exception(exc):
    """Map an exception to an outcome via class-name suffix (pytest compat).

    Returns "skipped" | "xfailed" | "failed" | None (None = ordinary error).
    """
    for klass in type(exc).__mro__:
        name = klass.__name__
        if name.endswith("Skipped"):
            return "skipped"
        if name.endswith("XFailed"):
            return "xfailed"
        if name.endswith("Failed"):
            return "failed"
    return None


# ============================================================================
# Virtual `tezt` module -- injected into sys.modules so user code can
# `import tezt` and use fixtures/marks without any installed package.
# ============================================================================

class _Mark:
    """A mark applied to a test (skip/skipif/xfail/parametrize)."""
    __slots__ = ("name", "args", "kwargs")

    def __init__(self, name, args, kwargs):
        self.name = name
        self.args = args
        self.kwargs = kwargs


def _attach_mark(obj, mark):
    marks = obj.__dict__.get("teztmark") if isinstance(obj, type) else getattr(obj, "teztmark", None)
    if marks is None:
        marks = []
        try:
            obj.teztmark = marks
        except (AttributeError, TypeError):
            return obj
    marks.append(mark)
    return obj


class _MarkDecorator:
    """tezt.mark.<name>(...) -> decorator appending a _Mark to func.teztmark."""

    def __init__(self, name):
        self._name = name

    def __call__(self, *args, **kwargs):
        # Bare usage: @tezt.mark.skip applied directly to a function.
        if len(args) == 1 and not kwargs and callable(args[0]) and not isinstance(args[0], type):
            return _attach_mark(args[0], _Mark(self._name, (), {}))
        mark = _Mark(self._name, args, kwargs)
        return lambda obj: _attach_mark(obj, mark)


class _MarkNamespace:
    def __getattr__(self, name):
        return _MarkDecorator(name)


class FixtureDef:
    """Metadata wrapper a @tezt.fixture decoration produces (callable passthrough).

    `params` (optional) parametrizes the fixture: one test case per value, with
    `request.param` set to that value inside the fixture body. `ids` overrides
    the per-value id fragments (parallel to `params`). `autouse=True` makes the
    fixture run for every test in its scope without being requested by name.
    """

    def __init__(self, func, scope, name, autouse=False, params=None, ids=None):
        self.func = func
        self.scope = scope
        self.name = name or func.__name__
        self.autouse = autouse
        self.params = list(params) if params is not None else None
        self.ids = list(ids) if ids is not None else None
        self._tezt_fixture = True

    def __call__(self, *args, **kwargs):  # allow direct invocation in user code
        return self.func(*args, **kwargs)


def _make_tezt_module():
    mod = types.ModuleType("tezt")
    mod.__doc__ = "Virtual tezt module injected by tezt_worker."

    def fixture(func=None, *, scope="function", name=None, autouse=False,
                params=None, ids=None):
        if scope not in ("function", "module", "session", "class"):
            raise ValueError("invalid fixture scope: %r" % scope)
        if func is not None:
            return FixtureDef(func, scope, name, autouse, params, ids)

        def deco(f):
            return FixtureDef(f, scope, name, autouse, params, ids)
        return deco

    def parametrize(argnames, argvalues, ids=None):
        return _MarkDecorator("parametrize")(argnames, argvalues, ids=ids)

    def skip(reason=""):
        raise Skipped(reason)

    def fail(reason=""):
        raise Failed(reason)

    def xfail(reason=""):
        raise XFailed(reason)

    class raises:
        """Context manager asserting that a block raises ExcType (match=re.search)."""

        def __init__(self, exc_type, match=None):
            self.exc_type = exc_type
            self.match = match
            self.value = None

        def __enter__(self):
            return self

        def __exit__(self, etype, evalue, etb):
            if etype is None:
                raise Failed("DID NOT RAISE %r" % (self.exc_type,))
            if not issubclass(etype, self.exc_type):
                return False  # propagate unexpected exception
            self.value = evalue
            if self.match is not None and not re.search(self.match, str(evalue)):
                raise Failed("pattern %r not found in %r" % (self.match, str(evalue)))
            return True  # swallow the expected exception

    class warns:
        """Context manager asserting a block emits a matching warning (mirrors raises).

        On __enter__ it records warnings via warnings.catch_warnings(record=True);
        on a clean __exit__ it asserts at least one recorded warning is an instance
        of `expected_warning` (and, if `match` is given, that re.search(match, ...)
        matches one of them). Recorded warnings are exposed as .list and the first
        match as .matched after the block. Real exceptions propagate untouched.
        """

        def __init__(self, expected_warning, match=None):
            self.expected_warning = expected_warning
            self.match = match
            self.list = []
            self.matched = None
            self._cm = None

        def __enter__(self):
            self._cm = warnings.catch_warnings(record=True)
            self.list = self._cm.__enter__()
            warnings.simplefilter("always")
            return self

        def __exit__(self, etype, evalue, etb):
            self._cm.__exit__(etype, evalue, etb)
            if etype is not None:
                return False  # let real exceptions propagate
            matches = [w for w in self.list
                       if issubclass(w.category, self.expected_warning)]
            if not matches:
                raise Failed("DID NOT WARN %r" % (self.expected_warning,))
            if self.match is not None:
                matched = [w for w in matches
                           if re.search(self.match, str(w.message))]
                if not matched:
                    raise Failed("pattern %r not found in any of %r" % (
                        self.match, [str(w.message) for w in matches]))
                self.matched = matched[0]
            else:
                self.matched = matches[0]
            return False

    def approx(expected, rel=None, abs=None, nan_ok=False):
        # Dispatch to a type-specific comparator (scalar / sequence / mapping).
        # See ApproxBase and friends below for the comparison semantics.
        return _make_approx(expected, rel, abs, nan_ok)

    mod.fixture = fixture
    mod.parametrize = parametrize
    mod.mark = _MarkNamespace()
    mod.skip = skip
    mod.fail = fail
    mod.xfail = xfail
    mod.raises = raises
    mod.warns = warns
    mod.approx = approx
    mod.Skipped = Skipped
    mod.Failed = Failed
    mod.XFailed = XFailed
    return mod


# ============================================================================
# tezt.approx -- approximate equality for floats, sequences, and mappings.
# Defaults mirror pytest: a value is "close" when the difference is within
# max(rel * abs(expected), abs). Comparison works in both directions because
# Python falls back to the right operand's __eq__ when the left returns
# NotImplemented (and the bare object on the other side does). __ne__ negates.
# ============================================================================

# pytest's default relative / absolute tolerances.
_APPROX_REL = 1e-6
_APPROX_ABS = 1e-12


class ApproxBase:
    """Base for approx comparators: holds the expected value and tolerances."""

    def __init__(self, expected, rel, abs, nan_ok):
        self.expected = expected
        self.rel = rel
        self.abs = abs
        self.nan_ok = nan_ok

    def __ne__(self, actual):
        return not (self == actual)

    # Python tries actual.__eq__(self) first; only when that returns
    # NotImplemented does it try self.__eq__(actual). Because the other operand
    # is an ordinary value, both `actual == approx(x)` and `approx(x) == actual`
    # route through our __eq__, giving symmetric comparison.


class ApproxScalar(ApproxBase):
    """Approximate equality for a single int/float/complex."""

    def _tolerance(self):
        rel = self.rel if self.rel is not None else _APPROX_REL
        abs_ = self.abs if self.abs is not None else _APPROX_ABS
        return max(rel * abs(self.expected), abs_)

    def __eq__(self, actual):
        exp = self.expected
        # NaN never equals NaN unless explicitly allowed.
        try:
            exp_nan = isinstance(exp, float) and math.isnan(exp)
            act_nan = isinstance(actual, float) and math.isnan(actual)
        except TypeError:
            return NotImplemented
        if exp_nan or act_nan:
            return bool(self.nan_ok and exp_nan and act_nan)
        # Infinities: equal only if identical (no finite tolerance bridges inf).
        try:
            if math.isinf(exp) or (isinstance(actual, float) and math.isinf(actual)):
                return exp == actual
        except TypeError:
            pass
        try:
            return abs(actual - exp) <= self._tolerance()
        except TypeError:
            return NotImplemented

    def __repr__(self):
        try:
            tol = self._tolerance()
        except TypeError:
            return "approx(%r)" % (self.expected,)
        return "approx(%r %s %.3g)" % (self.expected, "±", tol)


class ApproxSequence(ApproxBase):
    """Approximate equality for a list/tuple, compared elementwise."""

    def __eq__(self, actual):
        if not isinstance(actual, (list, tuple)):
            return NotImplemented
        if len(actual) != len(self.expected):
            return False
        for a, e in zip(actual, self.expected):
            if not (ApproxScalar(e, self.rel, self.abs, self.nan_ok) == a):
                return False
        return True

    def __repr__(self):
        return "approx(%r)" % (self.expected,)


class ApproxMapping(ApproxBase):
    """Approximate equality for a dict, compared key-by-key."""

    def __eq__(self, actual):
        if not isinstance(actual, dict):
            return NotImplemented
        if set(actual.keys()) != set(self.expected.keys()):
            return False
        for k, e in self.expected.items():
            if not (ApproxScalar(e, self.rel, self.abs, self.nan_ok) == actual[k]):
                return False
        return True

    def __repr__(self):
        return "approx(%r)" % (self.expected,)


def _make_approx(expected, rel, abs, nan_ok):
    if isinstance(expected, dict):
        return ApproxMapping(expected, rel, abs, nan_ok)
    if isinstance(expected, (list, tuple)):
        return ApproxSequence(expected, rel, abs, nan_ok)
    return ApproxScalar(expected, rel, abs, nan_ok)


# ============================================================================
# Mark extraction -- unify tezt marks and pytest marks into _Mark objects.
# ============================================================================

def _coerce_marks(raw):
    """Normalize pytestmark/teztmark values (single mark, list, MarkDecorator)."""
    if raw is None:
        return []
    if not isinstance(raw, (list, tuple)):
        raw = [raw]
    out = []
    for m in raw:
        # pytest MarkDecorator has a .mark attribute holding the Mark
        inner = getattr(m, "mark", m)
        name = getattr(inner, "name", None)
        if name is None:
            continue
        out.append(_Mark(name, tuple(getattr(inner, "args", ())),
                         dict(getattr(inner, "kwargs", {}))))
    return out


def collect_marks(func, cls, module):
    """Gather marks for a test: module-level + class-level + function-level."""
    marks = []
    marks += _coerce_marks(getattr(module, "pytestmark", None))
    if cls is not None:
        marks += _coerce_marks(getattr(cls, "pytestmark", None))
        marks += _coerce_marks(cls.__dict__.get("teztmark"))
    marks += _coerce_marks(getattr(func, "pytestmark", None))
    marks += _coerce_marks(getattr(func, "teztmark", None))
    return marks


# ============================================================================
# Plugin / hook system -- conftest-based, pytest-compatible hook NAMES.
#
# Hooks are plain functions discovered *by name* in conftest modules and in the
# test module itself; there is no plugin-registration ceremony. We support only
# the subset of pytest hooks that map onto tezt's execution model:
#
#   pytest_configure(config)               -- once per worker, before any import
#   pytest_sessionstart(session)           -- once per worker, right after configure
#   pytest_sessionfinish(session, exitstatus) -- once per worker, at shutdown
#   pytest_runtest_setup(item)             -- before each test's setup phase
#   pytest_runtest_teardown(item)          -- after each test, during teardown
#
# Intentionally NOT supported, because tezt performs *collection* in Rust (the
# worker only ever executes already-collected items): pytest_collection,
# pytest_collection_modifyitems, pytest_generate_tests, pytest_itemcollected,
# pytest_runtest_call/makereport, fixture/plugin manager hooks, etc. There is no
# place in this worker for collection-time hooks to run, so we deliberately drop
# them rather than half-implement them.
#
# Session semantics: tezt runs a *pool* of these workers, so each worker is its
# own "session". pytest_sessionstart / pytest_configure therefore fire ONCE PER
# WORKER (not once per test run), and pytest_sessionfinish fires once per worker
# at shutdown with exitstatus=0. Plugins that assume a single global session
# should treat each worker process as an independent session.
# ============================================================================

# The exact hook names we recognize. Discovery matches function names against
# this set verbatim (pytest-compatible spelling).
SUPPORTED_HOOKS = (
    "pytest_configure",
    "pytest_sessionstart",
    "pytest_sessionfinish",
    "pytest_runtest_setup",
    "pytest_runtest_teardown",
)


class HookRegistry:
    """Holds the discovered hook callables per hook name, in registration order.

    De-duplicates by function identity so the same conftest hook (e.g. a root
    conftest scanned eagerly in main() and again as part of a per-file chain)
    is never registered twice.
    """

    def __init__(self):
        # name -> list of callables (registration order preserved)
        self._hooks = {name: [] for name in SUPPORTED_HOOKS}
        # set of id() of already-registered callables (identity de-dup)
        self._seen = set()

    def register(self, name, func):
        if name not in self._hooks:
            return
        key = id(func)
        if key in self._seen:
            return
        self._seen.add(key)
        self._hooks[name].append(func)

    def get(self, name):
        return self._hooks.get(name, ())


# One registry per worker process (the worker is the session). Created in main()
# and passed implicitly via this module global so run_case/handle_run can reach
# it without threading it through every signature.
_HOOKS = HookRegistry()

# Set of id() of modules already scanned for hooks, so importing the same
# conftest/test module for many test items only scans it once.
_scanned_modules = set()


def register_hooks_from(module):
    """Scan vars(module) for callables named exactly like a supported hook and
    append them to the registry. Cheap and idempotent (id-based de-dup in both
    the per-module guard here and the registry itself)."""
    if module is None:
        return
    mid = id(module)
    if mid in _scanned_modules:
        return
    _scanned_modules.add(mid)
    members = vars(module)
    for name in SUPPORTED_HOOKS:
        func = members.get(name)
        if func is not None and callable(func):
            _HOOKS.register(name, func)


def call_hooks(name, **kwargs):
    """Call every registered hook for `name` in registration order.

    Each hook only receives the kwargs whose names appear in its signature
    (inspected via _signature_params), so a hook declared as
    `def pytest_runtest_setup(item)` and one declared with extra/fewer params
    both work. Errors are NOT caught here -- the caller decides whether an
    exception should propagate (e.g. a Skipped from a setup hook) or be
    swallowed (teardown / configure / sessionfinish). This keeps the policy at
    the call site, matching how existing teardowns swallow their own errors.
    """
    for func in _HOOKS.get(name):
        try:
            accepted = _signature_params(func)
        except (TypeError, ValueError):
            accepted = None
        if accepted is None:
            # Could not introspect (e.g. a builtin): call with all kwargs and
            # let any TypeError surface to the caller's policy.
            func(**kwargs)
            continue
        call_kwargs = {k: v for k, v in kwargs.items() if k in accepted}
        func(**call_kwargs)


class _Config:
    """Light stand-in for pytest's Config, passed to pytest_configure and held
    on session/item. Implements the handful of methods plugins commonly call so
    a typical `config.addinivalue_line("markers", ...)` does not explode."""

    def __init__(self, rootdir):
        self.rootdir = rootdir
        # `args` is the list of test paths pytest was invoked with; tezt does
        # not pass these to the worker, so expose an empty list.
        self.args = []

    def getoption(self, name, default=None):
        # tezt has no pytest-style option store; always return the default.
        return default

    def getini(self, name):
        # No ini file in the worker; return the empty-string default pytest uses
        # for unset string inis.
        return ""

    def addinivalue_line(self, name, line):
        # Marker/line registration is a no-op here (tezt does not enforce
        # registered markers), but the method must exist so plugins can call it.
        return None


class _Session:
    """Light stand-in for pytest's Session. One per worker (see module header).

    `.items` stays empty because collection happened in Rust; we never populate
    a worker-side item list.
    """

    def __init__(self, config):
        self.config = config
        # pytest exposes both startpath (pathlib) and the legacy startdir; tezt
        # uses plain strings for both, pointed at the rootdir.
        self.startpath = config.rootdir
        self.startdir = config.rootdir
        self.items = []


class HookItem:
    """The `item` object passed to pytest_runtest_setup / pytest_runtest_teardown.

    Exposes the slice of pytest's Item/Function API that runtest hooks realistically
    touch: identity (nodeid/name/originalname), location (fspath/path), the test
    callable/class/module, marker access, and back-references to config/session.
    Marker queries are answered from the marks collected by collect_marks(); a hook
    may also add_marker() best-effort (stored on an internal list, not persisted to
    the function).
    """

    def __init__(self, nodeid, name, func, cls, module, config, session):
        self.nodeid = nodeid
        self.name = name
        # pytest's Item.originalname is the unparametrized function name; tezt's
        # `name` already is the bare function name, so they coincide here.
        self.originalname = name
        fspath = getattr(module, "__file__", None) or ""
        self.fspath = fspath
        self.path = fspath
        self.function = func
        self.cls = cls
        self.module = module
        self.config = config
        self.session = session
        # keywords: mark-name -> _Mark, built from module+class+function marks.
        marks = collect_marks(func, cls, module)
        self.keywords = {}
        # Preserve the collected marks for iter_markers/get_closest_marker; when
        # a name repeats, the LAST one wins in `keywords` (mirrors a dict), but
        # iter_markers still returns every occurrence.
        self._marks = list(marks)
        for m in marks:
            self.keywords[m.name] = m

    def get_closest_marker(self, name):
        """Return the closest _Mark with `name`, or None.

        collect_marks orders marks module -> class -> function, i.e. nearest
        (function) last, so the closest match is the LAST occurrence.
        """
        found = None
        for m in self._marks:
            if m.name == name:
                found = m
        return found

    def iter_markers(self, name=None):
        """List of _Mark objects, optionally filtered by name."""
        if name is None:
            return list(self._marks)
        return [m for m in self._marks if m.name == name]

    def add_marker(self, marker):
        """Best-effort: append a marker (accepts a _MarkDecorator, _Mark, or a
        bare str name). Stored on this item only; we do not mutate the test
        function. Mirrors pytest's Item.add_marker just enough for plugins."""
        mark = None
        if isinstance(marker, _Mark):
            mark = marker
        elif isinstance(marker, _MarkDecorator):
            # tezt's decorator carries the name in ._name; no bound args yet.
            mark = _Mark(marker._name, (), {})
        elif isinstance(marker, str):
            mark = _Mark(marker, (), {})
        else:
            # Unknown shape (e.g. a real pytest MarkDecorator): try to read a
            # name off it, else ignore silently (best-effort contract).
            inner = getattr(marker, "mark", marker)
            mname = getattr(inner, "name", None) or getattr(marker, "_name", None)
            if mname is None:
                return
            mark = _Mark(mname, tuple(getattr(inner, "args", ())),
                         dict(getattr(inner, "kwargs", {})))
        self._marks.append(mark)
        self.keywords[mark.name] = mark


# ============================================================================
# Fixture discovery -- recognize both tezt FixtureDef and pytest fixtures.
# ============================================================================

# What _as_fixture returns and what fixture lookup tables store. Carrying
# autouse/params/ids on a namedtuple keeps every unpack site readable and lets
# new fields be added without reshuffling positional tuples everywhere.
FixtureInfo = collections.namedtuple(
    "FixtureInfo", ["func", "scope", "name", "autouse", "params", "ids"])
LookupEntry = collections.namedtuple(
    "LookupEntry", ["func", "scope", "module", "autouse", "params", "ids"])


def _as_fixture(obj):
    """If obj is a fixture definition (tezt or pytest), return a FixtureInfo.

    Returns None for non-fixtures. Never imports pytest itself. autouse/params/
    ids are read off the marker for pytest fixtures (defaulting False/None).
    """
    # tezt fixture
    if isinstance(obj, FixtureDef):
        return FixtureInfo(obj.func, obj.scope, obj.name,
                           obj.autouse, obj.params, obj.ids)
    # pytest >= 8.4: FixtureFunctionDefinition object wrapping the function
    if type(obj).__name__ == "FixtureFunctionDefinition":
        func = getattr(obj, "_fixture_function", None)
        marker = getattr(obj, "_fixture_function_marker", None)
        scope = getattr(marker, "scope", "function") if marker else "function"
        name = getattr(marker, "name", None) if marker else None
        autouse = bool(getattr(marker, "autouse", False)) if marker else False
        params = getattr(marker, "params", None) if marker else None
        ids = getattr(marker, "ids", None) if marker else None
        if func is not None:
            return FixtureInfo(func, _norm_scope(scope),
                               name or getattr(func, "__name__", None),
                               autouse,
                               list(params) if params is not None else None,
                               list(ids) if ids is not None else None)
        return None
    # classic pytest: decorated function carries _pytestfixturefunction marker
    marker = getattr(obj, "_pytestfixturefunction", None)
    if marker is not None:
        func = getattr(obj, "__wrapped__", obj)
        scope = getattr(marker, "scope", "function")
        name = getattr(marker, "name", None)
        autouse = bool(getattr(marker, "autouse", False))
        params = getattr(marker, "params", None)
        ids = getattr(marker, "ids", None)
        return FixtureInfo(func, _norm_scope(scope),
                           name or getattr(func, "__name__", None),
                           autouse,
                           list(params) if params is not None else None,
                           list(ids) if ids is not None else None)
    return None


def _norm_scope(scope):
    if not isinstance(scope, str):
        scope = getattr(scope, "value", str(scope))
    scope = str(scope).lower()
    if scope not in ("function", "module", "session", "class", "package"):
        scope = "function"
    if scope == "package":
        scope = "session"
    return scope


def scan_fixtures(module):
    """Map fixture-name -> LookupEntry(func, scope, module, autouse, params, ids)."""
    found = {}
    for attr_name, obj in vars(module).items():
        info = _as_fixture(obj)
        if info is not None:
            found[info.name or attr_name] = LookupEntry(
                info.func, info.scope, module, info.autouse,
                info.params, info.ids)
    return found


# Scope nesting rank: higher scopes set up FIRST (session before module before
# class before function). Used to order autouse fixtures and unknown scopes
# sort last.
_SCOPE_RANK = {"session": 0, "module": 1, "class": 2, "function": 3}


def autouse_fixtures(lookup):
    """Ordered, de-duplicated list of autouse fixture names across `lookup`.

    `lookup` is the ordered list of {name: LookupEntry} tables for a file (test
    module first, then conftests nearest-first). We collect every entry whose
    `autouse` is true, keep the FIRST table's entry for a repeated name (matching
    normal fixture-override precedence: nearest definition wins), then sort by
    scope rank so higher scopes (session/module) are resolved before lower ones.
    """
    seen = {}     # name -> scope (first occurrence wins)
    for table in lookup:
        for name, entry in table.items():
            if entry.autouse and name not in seen:
                seen[name] = entry.scope
    return sorted(seen, key=lambda n: (_SCOPE_RANK.get(seen[n], 99), n))


# ============================================================================
# Module importing -- file path -> module, with conftest chain support.
# ============================================================================

_module_cache = {}        # abs file path -> module object (or Exception)
_conftest_chain_cache = {}  # dir path -> ordered list of conftest modules


def module_name_for(path):
    try:
        rel = os.path.relpath(path, ROOTDIR)
    except ValueError:
        # Windows: `path` and ROOTDIR are on different drives/mounts (tests on
        # another drive than the working dir). relpath can't express that, so
        # derive the name from the drive-stripped absolute path instead.
        rel = os.path.splitdrive(os.path.abspath(path))[1]
    if rel.endswith(".py"):
        rel = rel[:-3]
    name = rel.replace(os.sep, ".").replace("/", ".")
    # Make path-derived names safe-ish module identifiers.
    return name.strip(".")


def import_module_from_path(path):
    """Import (and cache) a module by absolute file path. Raises on failure."""
    path = os.path.abspath(path)
    cached = _module_cache.get(path)
    if cached is not None:
        if isinstance(cached, BaseException):
            raise cached
        return cached
    name = module_name_for(path)
    try:
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise ImportError("cannot create import spec for %s" % path)
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
    except BaseException as exc:
        sys.modules.pop(name, None)
        _module_cache[path] = exc
        raise
    _module_cache[path] = module
    return module


def conftest_chain(test_file):
    """Ordered conftest modules from rootdir down to the test file's directory.

    Nearest directory comes LAST so later entries override earlier ones.
    Import errors propagate (treated as setup errors for the file's tests).
    """
    d = os.path.dirname(os.path.abspath(test_file))
    cached = _conftest_chain_cache.get(d)
    if cached is not None:
        if isinstance(cached, BaseException):
            raise cached
        return cached
    # Build the directory list rootdir -> test dir
    dirs = []
    cur = d
    root = os.path.abspath(ROOTDIR)
    while True:
        dirs.append(cur)
        if os.path.normcase(cur) == os.path.normcase(root):
            break
        parent = os.path.dirname(cur)
        if parent == cur:          # filesystem root; test file outside rootdir
            break
        cur = parent
    dirs.reverse()
    chain = []
    try:
        for dd in dirs:
            cf = os.path.join(dd, "conftest.py")
            if os.path.isfile(cf):
                chain.append(import_module_from_path(cf))
    except BaseException as exc:
        _conftest_chain_cache[d] = exc
        raise
    _conftest_chain_cache[d] = chain
    return chain


# ============================================================================
# Parametrize expansion
# ============================================================================

def _split_argnames(argnames):
    if isinstance(argnames, (list, tuple)):
        return [str(a).strip() for a in argnames]
    return [a.strip() for a in str(argnames).split(",") if a.strip()]


def _param_id(value):
    if isinstance(value, str):
        return value
    if isinstance(value, bool):
        return "True" if value else "False"
    if value is None:
        return "None"
    if isinstance(value, (int, float)):
        return str(value)
    return None  # placeholder -> p<N>


def expand_parametrize(marks):
    """Return list of (param_id, {argname: value}) combos; [(None, {})] if none.

    Multiple parametrize marks combine as a cartesian product with the
    innermost (closest-to-function) decorator varying fastest. Decorators
    apply bottom-up, so the innermost mark is FIRST in the collected list.
    """
    param_marks = [m for m in marks if m.name == "parametrize"]
    if not param_marks:
        return [(None, {})]

    layers = []  # each: list of (id_fragment, {name: value})
    for mark in param_marks:
        argnames = _split_argnames(mark.args[0])
        argvalues = list(mark.args[1])
        ids = mark.kwargs.get("ids")
        cases = []
        for i, val in enumerate(argvalues):
            if len(argnames) == 1:
                values = (val,)
            else:
                values = tuple(val)
                if len(values) != len(argnames):
                    raise ValueError(
                        "parametrize: %d values for %d argnames" % (len(values), len(argnames)))
            if ids is not None and i < len(ids):
                frag = str(ids[i])
            else:
                parts = []
                for v in values:
                    p = _param_id(v)
                    parts.append(p if p is not None else "p%d" % i)
                frag = "-".join(parts)
            cases.append((frag, dict(zip(argnames, values))))
        layers.append(cases)

    # Innermost decorator (first layer) varies fastest -> make it the LAST
    # factor of itertools.product. Id fragments join innermost-first to match
    # pytest (e.g. inner "p"/outer 10 -> "p-10").
    combos = []
    for combo in itertools.product(*reversed(layers)):
        # combo is ordered outermost..innermost
        frag = "-".join(c[0] for c in reversed(combo))
        params = {}
        for c in combo:
            params.update(c[1])
        combos.append((frag, params))
    return combos


def _fixture_param_id(value, idx, ids):
    """Id fragment for one parametrized-fixture value (mirrors @parametrize ids)."""
    if ids is not None and idx < len(ids) and ids[idx] is not None:
        return str(ids[idx])
    p = _param_id(value)
    return p if p is not None else "p%d" % idx


def collect_param_fixtures(fixture_names, lookup):
    """Ordered list of (name, params, ids) for parametrized fixtures in the
    dependency closure of `fixture_names`.

    Post-order DFS: a fixture's own (possibly parametrized) dependencies are
    visited before the fixture itself, so a parametrized dependency is set up
    before the fixture that consumes it. The traversal order is deterministic
    (signature order of requested fixtures, then each fixture's own signature
    order), giving stable ids. Builtins and unknown names are skipped; cycles
    are guarded by a visiting set.
    """
    out = []
    seen = set()        # names already emitted (closure de-dup)
    visiting = set()    # cycle guard for the DFS

    def entry_for(name):
        for table in lookup:
            if name in table:
                return table[name]
        return None

    def visit(name):
        if name in seen or name in visiting:
            return
        entry = entry_for(name)
        if entry is None:               # builtin / unknown: nothing to expand
            return
        visiting.add(name)
        try:
            deps = _signature_params(entry.func)
        except (TypeError, ValueError):
            deps = []
        for dep in deps:
            visit(dep)
        visiting.discard(name)
        seen.add(name)
        if entry.params is not None:
            out.append((name, list(entry.params), entry.ids))

    for fname in fixture_names:
        visit(fname)
    return out


# ============================================================================
# Built-in fixtures: tmp_path, tmp_path_factory, monkeypatch
# ============================================================================

class TmpPathFactory:
    """Session-scoped factory; .mktemp(name) returns a fresh pathlib.Path."""

    def __init__(self):
        self._base = None
        self._counts = {}

    def _basedir(self):
        if self._base is None:
            self._base = tempfile.mkdtemp(prefix="tezt-")
        return self._base

    def mktemp(self, name):
        import pathlib
        n = self._counts.get(name, 0)
        self._counts[name] = n + 1
        p = os.path.join(self._basedir(), "%s%d" % (name, n))
        os.makedirs(p, exist_ok=True)
        return pathlib.Path(p)

    def cleanup(self):
        if self._base and os.path.isdir(self._base):
            shutil.rmtree(self._base, ignore_errors=True)
        self._base = None


_NOTSET = object()


class MonkeyPatch:
    """Minimal pytest-style monkeypatch with automatic undo."""

    def __init__(self):
        self._setattr = []   # (obj, name, old)
        self._setitem = []   # (mapping, key, old)

    def setattr(self, target, name, value=_NOTSET):
        if value is _NOTSET:
            # string form: "module.path.attr"
            if not isinstance(target, str):
                raise TypeError("setattr: 2-arg form requires a dotted string target")
            path, _, attr = target.rpartition(".")
            obj = _import_dotted(path)
            value = name
            name = attr
            target = obj
        old = getattr(target, name, _NOTSET)
        self._setattr.append((target, name, old))
        setattr(target, name, value)

    def delattr(self, target, name=_NOTSET):
        if name is _NOTSET:
            if not isinstance(target, str):
                raise TypeError("delattr: 1-arg form requires a dotted string target")
            path, _, attr = target.rpartition(".")
            target = _import_dotted(path)
            name = attr
        old = getattr(target, name, _NOTSET)
        self._setattr.append((target, name, old))
        if old is not _NOTSET:
            delattr(target, name)

    def setitem(self, mapping, key, value):
        old = mapping.get(key, _NOTSET) if hasattr(mapping, "get") else (
            mapping[key] if key in mapping else _NOTSET)
        self._setitem.append((mapping, key, old))
        mapping[key] = value

    def delitem(self, mapping, key):
        old = mapping[key] if key in mapping else _NOTSET
        self._setitem.append((mapping, key, old))
        if key in mapping:
            del mapping[key]

    def setenv(self, name, value, prepend=None):
        value = str(value)
        if prepend and name in os.environ:
            value = value + prepend + os.environ[name]
        self.setitem(os.environ, name, value)

    def delenv(self, name, raising=True):
        if name not in os.environ:
            if raising:
                raise KeyError(name)
            return
        self.delitem(os.environ, name)

    def undo(self):
        for obj, name, old in reversed(self._setattr):
            if old is _NOTSET:
                try:
                    delattr(obj, name)
                except AttributeError:
                    pass
            else:
                setattr(obj, name, old)
        for mapping, key, old in reversed(self._setitem):
            if old is _NOTSET:
                mapping.pop(key, None)
            else:
                mapping[key] = old
        self._setattr = []
        self._setitem = []


def _import_dotted(path):
    import importlib
    parts = path.split(".")
    for i in range(len(parts), 0, -1):
        modname = ".".join(parts[:i])
        try:
            obj = importlib.import_module(modname)
        except ImportError:
            continue
        for attr in parts[i:]:
            obj = getattr(obj, attr)
        return obj
    raise ImportError("cannot import %r" % path)


# ============================================================================
# Fixture engine -- resolution, scope caches, generator teardowns.
# Scope nesting is function < class < module < session: tearing down an outer
# scope must dispose every inner scope first. The engine keeps one cache and
# one LIFO teardown list per non-function scope (function scope lives in a
# per-test TestContext). Async fixtures and async tests share ONE event loop
# (see .loop()) so async resources built in a fixture are valid in the test.
# ============================================================================

class _RequestConfig:
    """Fallback `request.config` when no worker _Config exists (e.g. self-tests
    that resolve fixtures without the full main() lifecycle). Mirrors the tiny
    slice of pytest's Config that fixtures commonly touch."""

    def __init__(self):
        self.rootdir = ROOTDIR
        self.args = []

    def getoption(self, name, default=None):
        return default

    def getini(self, name):
        return ""

    def addinivalue_line(self, name, line):
        return None


class Request:
    """The `request` fixture object handed to fixtures/tests that ask for it.

    Exposes the slice of pytest's FixtureRequest that real-world fixtures use:
      .param            -- the current parametrized-fixture value (settable;
                           save/restored by the engine around each fixture call).
      .config           -- the worker config (_Config) or a minimal stub.
      .node             -- namespace with .name and .nodeid (the result id).
      .scope            -- "function" (single, per-test request object).
      .fixturename      -- name of the fixture currently being set up, or None.
      .addfinalizer(fn) -- register a teardown callback (runs at test teardown).
      .getfixturevalue(name) -- resolve another fixture on demand.

    One instance per test, cached in ctx.cache["request"]; the engine/lookup/
    result-id needed by getfixturevalue are read off the TestContext.
    """

    __slots__ = ("param", "config", "node", "scope", "fixturename", "_ctx")

    def __init__(self, ctx):
        self._ctx = ctx
        self.param = None
        self.config = _CONFIG if _CONFIG is not None else _RequestConfig()
        self.node = types.SimpleNamespace(
            name=getattr(ctx, "result_id", None),
            nodeid=getattr(ctx, "result_id", None))
        self.scope = "function"
        self.fixturename = None

    def addfinalizer(self, fn):
        self._ctx.finalizers.append(fn)

    def getfixturevalue(self, name):
        engine = self._ctx.engine
        if engine is None:
            raise FixtureError(
                "request.getfixturevalue(%r): no engine bound to request" % name)
        return engine.resolve(name, self._ctx.lookup, self._ctx, ())


class FixtureError(Exception):
    """Setup-side fixture failure (maps to outcome 'error')."""


class FixtureEngine:
    def __init__(self):
        self.tmp_factory = TmpPathFactory()
        # scope caches: name -> (value, generator_or_None)
        self.session_cache = {}
        self.session_teardowns = []   # LIFO (name, generator)
        self.module_cache = {}
        self.module_teardowns = []
        self.current_module_path = None
        # class scope sits between module and function: a class-scoped fixture
        # lives only while we are running methods of `current_class`.
        self.class_cache = {}
        self.class_teardowns = []
        self.current_class = None
        # Lazily-created event loop shared by every async fixture/test; created
        # on first use, closed once at session teardown. See .loop().
        self._loop = None
        # per-test (function scope) state lives in a TestContext

    # -- shared event loop ---------------------------------------------------

    def loop(self):
        """Lazily-created event loop shared by all async fixtures and async
        tests in this worker, so fixture-created async resources are valid
        inside the test that uses them."""
        if self._loop is None:
            self._loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self._loop)
        return self._loop

    # -- scope lifecycle -----------------------------------------------------

    def switch_module(self, module_path):
        if self.current_module_path == module_path:
            return
        self.teardown_module_scope()
        self.current_module_path = module_path

    def switch_class(self, cls):
        # cls may be None (a module-level function): that still tears down any
        # live class scope, since we have left the previous class.
        if self.current_class is cls:
            return
        self.teardown_class_scope()
        self.current_class = cls

    def teardown_class_scope(self):
        self._drain(self.class_teardowns)
        self.class_cache.clear()
        self.current_class = None

    def teardown_module_scope(self):
        # Class scope is nested inside module scope, so dispose it first.
        self.teardown_class_scope()
        self._drain(self.module_teardowns)
        self.module_cache.clear()
        self.current_module_path = None

    def teardown_session_scope(self):
        # Ordering: function (already drained per-test) < class < module <
        # session, then close the shared loop last so async teardowns above can
        # still run on it.
        self.teardown_module_scope()
        self._drain(self.session_teardowns)
        self.session_cache.clear()
        self.tmp_factory.cleanup()
        if self._loop is not None:
            try:
                self._loop.run_until_complete(self._loop.shutdown_asyncgens())
            except Exception:
                pass
            self._loop.close()
            self._loop = None
            asyncio.set_event_loop(None)

    def _drain(self, teardowns):
        # Drain both sync generator (next) and async generator (__anext__)
        # fixture teardowns; async ones resume on the shared loop.
        while teardowns:
            name, gen = teardowns.pop()
            try:
                if hasattr(gen, "__anext__"):           # async generator fixture
                    self.loop().run_until_complete(gen.__anext__())
                else:
                    next(gen)
            except (StopIteration, StopAsyncIteration):
                pass
            except Exception:
                debug("teardown of fixture %r raised:\n%s" % (name, tb_module.format_exc()))

    def _drain_capture(self, teardowns):
        """Like `_drain`, but return the FIRST exception a teardown raised (or
        None), draining all of them either way. Used for function-scope
        teardown, where pytest turns a fixture-teardown failure into an `error`
        on the test that used the fixture (higher scopes stay swallowed —
        their boundary isn't tied to a single test)."""
        first = None
        while teardowns:
            name, gen = teardowns.pop()
            try:
                if hasattr(gen, "__anext__"):
                    self.loop().run_until_complete(gen.__anext__())
                else:
                    next(gen)
            except (StopIteration, StopAsyncIteration):
                pass
            except Exception as exc:
                if first is None:
                    first = exc
                debug("teardown of fixture %r raised:\n%s" % (name, tb_module.format_exc()))
        return first

    # -- resolution ------------------------------------------------------------

    def resolve(self, name, lookup, ctx, stack):
        """Resolve fixture `name` to a value.

        lookup : ordered list of {name: (func, scope, module)} dicts to search
                 (test module first, then conftests nearest-first).
        ctx    : TestContext holding function-scope cache/teardowns.
        stack  : tuple of names currently being resolved (cycle detection).
        """
        if name in stack:
            raise FixtureError("fixture dependency cycle: %s" % " -> ".join(stack + (name,)))

        # builtins
        if name == "tmp_path":
            if "tmp_path" not in ctx.cache:
                import pathlib
                d = tempfile.mkdtemp(prefix="tezt-tmp-")
                ctx.cache["tmp_path"] = pathlib.Path(d)
                ctx.cleanup_dirs.append(d)
            return ctx.cache["tmp_path"]
        if name == "tmp_path_factory":
            return self.tmp_factory
        if name == "monkeypatch":
            if "monkeypatch" not in ctx.cache:
                mp = MonkeyPatch()
                ctx.cache["monkeypatch"] = mp
                ctx.monkeypatches.append(mp)
            return ctx.cache["monkeypatch"]
        if name == "request":
            if "request" not in ctx.cache:
                ctx.cache["request"] = Request(ctx)
            return ctx.cache["request"]
        if name == "capsys":
            if "capsys" not in ctx.cache:
                cap = ctx.capture
                ctx.cache["capsys"] = CaptureFixture(cap.out, cap.err)
            return ctx.cache["capsys"]
        if name == "capfd":
            # tezt captures at the Python level (sys.stdout/err swap), so capfd
            # shares that same capture -- true fd-level/C-extension capture is
            # not separated. This is a documented approximation (no os.dup2).
            if "capfd" not in ctx.cache:
                cap = ctx.capture
                ctx.cache["capfd"] = CaptureFixture(cap.out, cap.err)
            return ctx.cache["capfd"]
        if name == "caplog":
            if "caplog" not in ctx.cache:
                cl = LogCaptureFixture()
                ctx.cache["caplog"] = cl
                ctx.finalizers.append(cl.remove)
            return ctx.cache["caplog"]
        if name == "recwarn":
            if "recwarn" not in ctx.cache:
                rw = WarningsRecorder()
                ctx.cache["recwarn"] = rw
                ctx.finalizers.append(rw.finish)
            return ctx.cache["recwarn"]

        # user-defined fixture lookup
        entry = None
        for table in lookup:
            if name in table:
                entry = table[name]
                break
        if entry is None:
            raise FixtureError("fixture %r not found" % name)
        func, scope = entry.func, entry.scope

        cache = self._cache_for(scope, ctx)
        if name in cache:
            return cache[name][0]

        value, gen = self._instantiate(name, func, lookup, ctx, stack,
                                       params=entry.params)
        cache[name] = (value, gen)
        if gen is not None:
            self._teardowns_for(scope, ctx).append((name, gen))
        return value

    def _cache_for(self, scope, ctx):
        if scope == "session":
            return self.session_cache
        if scope == "module":
            return self.module_cache
        if scope == "class":
            return self.class_cache
        return ctx.cache

    def _teardowns_for(self, scope, ctx):
        if scope == "session":
            return self.session_teardowns
        if scope == "module":
            return self.module_teardowns
        if scope == "class":
            return self.class_teardowns
        return ctx.teardowns

    def _instantiate(self, name, func, lookup, ctx, stack, params=None):
        # If this fixture is parametrized and the current case picked a value for
        # it, expose that value as request.param for the duration of THIS
        # fixture's setup. We build/cache the per-test `request` first and
        # save/restore its .param + .fixturename so nested parametrized fixtures
        # don't clobber each other.
        req = None
        prev_param = None
        prev_fixturename = None
        if params is not None and name in ctx.fixture_params:
            req = self.resolve("request", lookup, ctx, stack)
            prev_param = req.param
            prev_fixturename = req.fixturename
            req.param = ctx.fixture_params[name]
            req.fixturename = name
        try:
            return self._instantiate_inner(name, func, lookup, ctx, stack)
        finally:
            if req is not None:
                req.param = prev_param
                req.fixturename = prev_fixturename

    def _instantiate_inner(self, name, func, lookup, ctx, stack):
        # Resolve the fixture's own dependencies (recursive)
        kwargs = {}
        try:
            params = _signature_params(func)
        except (TypeError, ValueError):
            params = []
        new_stack = stack + (name,)
        for pname in params:
            kwargs[pname] = self.resolve(pname, lookup, ctx, new_stack)

        # Async fixtures run on the worker's shared loop so resources they
        # create (connections, tasks, etc.) stay valid inside the test, which
        # runs on the same loop.
        if inspect.iscoroutinefunction(func):
            try:
                value = self.loop().run_until_complete(func(**kwargs))
            except FixtureError:
                raise
            except BaseException as exc:
                if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                    raise
                raise FixtureError("fixture %r raised %s: %s\n%s" % (
                    name, type(exc).__name__, exc,
                    _format_traceback(exc, style="auto"))) from exc
            return value, None
        if inspect.isasyncgenfunction(func):
            agen = func(**kwargs)
            try:
                value = self.loop().run_until_complete(agen.__anext__())
            except StopAsyncIteration:
                raise FixtureError("async generator fixture %r yielded no value" % name)
            except BaseException as exc:
                if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                    raise
                raise FixtureError("fixture %r raised %s: %s\n%s" % (
                    name, type(exc).__name__, exc,
                    _format_traceback(exc, style="auto"))) from exc
            return value, agen      # teardown drains via __anext__ (handled in _drain)

        try:
            result = func(**kwargs)
        except FixtureError:
            raise
        except BaseException as exc:
            if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                raise
            raise FixtureError(
                "fixture %r raised %s: %s\n%s" % (
                    name, type(exc).__name__, exc,
                    _format_traceback(exc, style="auto"))) from exc
        if inspect.isgenerator(result):
            try:
                value = next(result)
            except StopIteration:
                raise FixtureError("generator fixture %r yielded no value" % name)
            except BaseException as exc:
                if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                    raise
                raise FixtureError(
                    "fixture %r raised %s: %s\n%s" % (
                        name, type(exc).__name__, exc,
                    _format_traceback(exc, style="auto"))) from exc
            return value, result
        return result, None


class TestContext:
    """Function-scope fixture state for a single test case."""

    __slots__ = ("cache", "teardowns", "monkeypatches", "cleanup_dirs",
                 "finalizers", "capture", "fixture_params", "lookup",
                 "result_id", "engine")

    def __init__(self):
        self.cache = {}
        self.teardowns = []
        self.monkeypatches = []
        self.cleanup_dirs = []
        self.finalizers = []
        # The active Capture for this test, set by run_case before fixtures
        # resolve, so capsys/capfd can read its _Tee buffers. None until then.
        self.capture = None
        # {fixture_name: param_value} for parametrized fixtures this case picked;
        # consulted by _instantiate to set request.param. Set in run_case.
        self.fixture_params = {}
        # Resolution context for the `request` object's getfixturevalue: the
        # file's fixture lookup tables, this test's result id, and the engine.
        # All set in run_case before fixtures resolve.
        self.lookup = []
        self.result_id = None
        self.engine = None

    def teardown(self, engine):
        # Drain via the engine so async-generator function-scope fixtures resume
        # on the shared loop. Return the first teardown/finalizer exception (or
        # None) so the caller can turn a teardown failure into an `error`, like
        # pytest. monkeypatch.undo and tmp cleanup are internal and stay quiet.
        err = engine._drain_capture(self.teardowns)
        for f in reversed(self.finalizers):
            try:
                f()
            except Exception as exc:
                if err is None:
                    err = exc
        for mp in self.monkeypatches:
            try:
                mp.undo()
            except Exception:
                pass
        for d in self.cleanup_dirs:
            shutil.rmtree(d, ignore_errors=True)
        return err


# signature-parameter cache (perf): func -> list of positional/keyword names
_sig_cache = {}


def _signature_params(func):
    cached = _sig_cache.get(func)
    if cached is not None:
        return cached
    sig = inspect.signature(func)
    names = [
        p.name for p in sig.parameters.values()
        if p.kind in (p.POSITIONAL_OR_KEYWORD, p.KEYWORD_ONLY)
        and p.name not in ("self", "cls")
    ]
    _sig_cache[func] = names
    return names


# ============================================================================
# Assertion enrichment -- a bare `assert` (no explicit message) is rewritten
# into a rich diagnostic: the source of the asserted expression plus, for a
# side-effect-free comparison, both operand values and a type-aware diff.
# Everything here is best-effort: any parse/eval/diff failure falls back to a
# simpler form, and the function NEVER raises.
# ============================================================================

# Caches keyed by filename so we parse each test file at most once. A value of
# None means "tried and failed" (unreadable / unparseable) -- still cached so
# we don't retry on every assertion in that file.
_assert_src_cache = {}    # filename -> source text or None
_assert_ast_cache = {}    # filename -> parsed ast.Module or None

# ast comparison operator -> display symbol.
_CMP_SYMBOLS = {
    ast.Eq: "==", ast.NotEq: "!=", ast.Lt: "<", ast.LtE: "<=",
    ast.Gt: ">", ast.GtE: ">=", ast.In: "in", ast.NotIn: "not in",
    ast.Is: "is", ast.IsNot: "is not",
}

_REPR_CAP = 200           # per-operand repr cap in the rich message
_DIFF_CAP = 20            # max diff lines to show in the str diff


def _capped_repr(value, cap=_REPR_CAP):
    """repr() that never raises and is truncated with a trailing '...'."""
    try:
        r = repr(value)
    except Exception:
        r = "<unreprable>"
    if len(r) > cap:
        r = r[:cap] + "..."
    return r


def _source_for(filename):
    """Read+cache a file's source text (None on any error)."""
    if filename in _assert_src_cache:
        return _assert_src_cache[filename]
    src = None
    try:
        with open(filename, "r") as f:
            src = f.read()
    except Exception:
        src = None
    _assert_src_cache[filename] = src
    return src


def _ast_for(filename, source):
    """Parse+cache a file's AST (None on any error)."""
    if filename in _assert_ast_cache:
        return _assert_ast_cache[filename]
    tree = None
    if source is not None:
        try:
            tree = ast.parse(source, filename)
        except Exception:
            tree = None
    _assert_ast_cache[filename] = tree
    return tree


def _is_side_effect_free(node):
    """True if `node` contains no call/await (safe to re-evaluate)."""
    for n in ast.walk(node):
        if isinstance(n, (ast.Call, ast.Await)):
            return False
    return True


def _str_diff_lines(left, right):
    """Type-aware diff body for two strings (list of message lines)."""
    out = []
    # Single-line strings: pinpoint the first differing character index, which
    # a line-based diff can't show.
    if "\n" not in left and "\n" not in right:
        n = min(len(left), len(right))
        idx = None
        for i in range(n):
            if left[i] != right[i]:
                idx = i
                break
        if idx is None and len(left) != len(right):
            idx = n
        if idx is not None:
            out.append("  first difference at index %d" % idx)
    diff = list(difflib.unified_diff(
        left.splitlines(), right.splitlines(),
        fromfile="left", tofile="right", lineterm=""))
    if diff:
        out.append("  diff (-left +right):")
        for line in diff[:_DIFF_CAP]:
            out.append("    " + line)
        if len(diff) > _DIFF_CAP:
            out.append("    ...")
    return out


def _seq_diff_lines(left, right):
    """Type-aware diff body for two lists/tuples."""
    out = []
    if len(left) != len(right):
        out.append("  length %d != %d" % (len(left), len(right)))
    for i in range(min(len(left), len(right))):
        if left[i] != right[i]:
            out.append("  index %d: %s != %s" % (
                i, _capped_repr(left[i]), _capped_repr(right[i])))
            break
    return out


def _dict_diff_lines(left, right):
    """Type-aware diff body for two dicts (capped to a few entries each)."""
    out = []
    cap = 5
    only_left = [k for k in left if k not in right]
    only_right = [k for k in right if k not in left]
    if only_left:
        out.append("  keys only in left: %s" % ", ".join(
            _capped_repr(k, 60) for k in only_left[:cap]))
    if only_right:
        out.append("  keys only in right: %s" % ", ".join(
            _capped_repr(k, 60) for k in only_right[:cap]))
    shown = 0
    for k in left:
        if k in right and left[k] != right[k]:
            out.append("  key %s: %s != %s" % (
                _capped_repr(k, 60), _capped_repr(left[k]), _capped_repr(right[k])))
            shown += 1
            if shown >= cap:
                break
    return out


def _set_diff_lines(left, right):
    """Type-aware diff body for two sets/frozensets."""
    out = []
    only_left = left - right
    only_right = right - left
    if only_left:
        out.append("  items only in left: %s" % _capped_repr(only_left))
    if only_right:
        out.append("  items only in right: %s" % _capped_repr(only_right))
    return out


def _type_aware_diff(op, left, right):
    """Best-effort diff lines for == / != operands; [] if none/unsupported."""
    if op not in ("==", "!="):
        return []
    try:
        if isinstance(left, str) and isinstance(right, str):
            return _str_diff_lines(left, right)
        if (isinstance(left, (list, tuple)) and isinstance(right, (list, tuple))
                and type(left) == type(right)):
            return _seq_diff_lines(left, right)
        if isinstance(left, dict) and isinstance(right, dict):
            return _dict_diff_lines(left, right)
        if (isinstance(left, (set, frozenset))
                and isinstance(right, (set, frozenset))):
            return _set_diff_lines(left, right)
    except Exception:
        return []
    return []


def _fallback_enrichment(test_src, frame):
    """Improved version of the original behavior: header + locals summary."""
    parts = []
    for k, v in list(frame.f_locals.items())[:8]:
        if k.startswith("__"):
            continue
        r = _capped_repr(v, 120)
        parts.append("%s=%s" % (k, r))
    msg = "assert failed: %s" % (test_src or "<source unavailable>")
    if parts:
        msg += " | locals: " + ", ".join(parts)
    return msg


def _enrich_assertion(exc, exc_tb, test_file):
    """Build a rich message for a bare `assert` (AssertionError, no message).

    Returns the message string, or None when enrichment does not apply (an
    explicit-message AssertionError, or no usable frame). Best-effort: every
    parse/eval/diff step is guarded so this function never raises.
    """
    try:
        # Only enrich bare asserts; an explicit message is already informative.
        if not isinstance(exc, AssertionError) or exc.args:
            return None

        # The deepest traceback frame is where the assert physically raised.
        # We prefer it over `test_file` so an assert inside a helper still maps
        # to the right source line.
        tb = exc_tb if exc_tb is not None else exc.__traceback__
        if tb is None:
            return None
        while tb.tb_next is not None:
            tb = tb.tb_next
        frame = tb.tb_frame
        filename = frame.f_code.co_filename
        lineno = tb.tb_lineno

        source = _source_for(filename)
        tree = _ast_for(filename, source)

        # Locate the innermost ast.Assert whose line range contains `lineno`:
        # the one with the largest starting line <= lineno.
        node = None
        if tree is not None:
            for n in ast.walk(tree):
                if isinstance(n, ast.Assert):
                    end = getattr(n, "end_lineno", None) or n.lineno
                    if n.lineno <= lineno <= end:
                        if node is None or n.lineno > node.lineno:
                            node = n

        # Source of the asserted expression (fallback: the raw source line).
        test_src = None
        if node is not None and source is not None:
            try:
                test_src = ast.get_source_segment(source, node.test)
            except Exception:
                test_src = None
        if not test_src:
            test_src = linecache.getline(filename, lineno).strip() or None

        # Rich path: a single-operator comparison with side-effect-free operands.
        if (node is not None and isinstance(node.test, ast.Compare)
                and len(node.test.ops) == 1 and len(node.test.comparators) == 1):
            left_node = node.test.left
            right_node = node.test.comparators[0]
            if _is_side_effect_free(left_node) and _is_side_effect_free(right_node):
                try:
                    left_val = eval(
                        compile(ast.Expression(left_node), filename, "eval"),
                        frame.f_globals, frame.f_locals)
                    right_val = eval(
                        compile(ast.Expression(right_node), filename, "eval"),
                        frame.f_globals, frame.f_locals)
                except BaseException:
                    return _fallback_enrichment(test_src, frame)
                op = _CMP_SYMBOLS.get(type(node.test.ops[0]), "?")
                lines = [
                    "assert %s" % (test_src or "<expr>"),
                    "  left  = %s" % _capped_repr(left_val),
                    "  right = %s" % _capped_repr(right_val),
                ]
                lines.extend(_type_aware_diff(op, left_val, right_val))
                return "\n".join(lines)

        # Fall back for anything not a safe single comparison.
        return _fallback_enrichment(test_src, frame)
    except Exception:
        # Absolutely never let enrichment break result reporting.
        return None


def _strip_worker_frames(exc):
    """Return the traceback of `exc` with tezt's own leading worker frames
    dropped, so the trace starts at the user's test (pytest hides its frames the
    same way)."""
    tb = exc.__traceback__
    worker_file = globals().get("__file__")
    if worker_file:
        worker_file = os.path.normcase(os.path.abspath(worker_file))
        while tb is not None:
            frame_file = os.path.normcase(os.path.abspath(tb.tb_frame.f_code.co_filename))
            if frame_file == worker_file:
                tb = tb.tb_next
            else:
                break
    return tb


def _deepest_user_frame(tb):
    """Walk to the last frame of `tb` (where the exception was actually raised).
    Returns (filename, lineno) or (None, None) if there is no traceback."""
    if tb is None:
        return None, None
    last = tb
    while last.tb_next is not None:
        last = last.tb_next
    return last.tb_frame.f_code.co_filename, last.tb_lineno


def _format_traceback(exc, limit_lines=TRACEBACK_LINES, style=None):
    """Render `exc`'s traceback honoring the --tb style.

      no    -> None (no traceback at all)
      line  -> a single line "<path>:<lineno>: <ExcType>: <msg>" for the
               deepest user frame; no frame dump.
      short -> each frame's `File "...", line N, in func` location plus the
               final exception line, with the echoed source-context lines dropped.
      auto/long -> the full standard traceback (unchanged historical behavior).

    `style` defaults to the module global TB_STYLE.
    """
    if style is None:
        style = TB_STYLE

    tb = _strip_worker_frames(exc)

    if style == "no":
        return None

    if style == "line":
        filename, lineno = _deepest_user_frame(tb)
        exc_line = "".join(
            tb_module.format_exception_only(type(exc), exc)).strip()
        if filename is None:
            return exc_line
        return "%s:%s: %s" % (filename, lineno, exc_line)

    if style == "short":
        rows = []
        # One location line per frame (no source-context echo), then the
        # exception-only lines (which include any chained-exception note).
        for frame, flineno in tb_module.walk_tb(tb):
            code = frame.f_code
            rows.append('  File "%s", line %d, in %s'
                        % (code.co_filename, flineno, code.co_name))
        rows.extend("".join(
            tb_module.format_exception_only(type(exc), exc)).splitlines())
        if len(rows) > limit_lines:
            rows = rows[-limit_lines:]
        return "\n".join(rows)

    # auto / long: full standard traceback.
    lines = tb_module.format_exception(type(exc), exc, tb)
    text = "".join(lines)
    rows = text.splitlines()
    if len(rows) > limit_lines:
        rows = rows[-limit_lines:]
    return "\n".join(rows)


# ============================================================================
# Output capture
# ============================================================================

class _Tee(io.TextIOBase):
    """Captures writes; optionally mirrors them to a real stream (--no-capture)."""

    def __init__(self, mirror=None):
        self.buf = io.StringIO()
        self.mirror = mirror

    def write(self, s):
        self.buf.write(s)
        if self.mirror is not None:
            self.mirror.write(s)
        return len(s)

    def flush(self):
        if self.mirror is not None:
            self.mirror.flush()

    def writable(self):
        return True

    def value(self):
        v = self.buf.getvalue()
        if len(v) > CAPTURE_LIMIT:
            v = v[:CAPTURE_LIMIT] + "\n...[truncated]"
        return v


class Capture:
    """Swap sys.stdout/sys.stderr for the duration of one test."""

    def __init__(self):
        self.out = _Tee(_REAL_STDERR if NO_CAPTURE else None)
        self.err = _Tee(_REAL_STDERR if NO_CAPTURE else None)

    def __enter__(self):
        self._old_out, self._old_err = sys.stdout, sys.stderr
        sys.stdout, sys.stderr = self.out, self.err
        return self

    def __exit__(self, *a):
        sys.stdout, sys.stderr = self._old_out, self._old_err
        return False


# ============================================================================
# Test-writing fixtures built on top of the capture machinery:
#   capsys / capfd -> CaptureFixture (reads the active _Tee buffers)
#   caplog         -> LogCaptureFixture (root-logger handler)
#   recwarn        -> WarningsRecorder (warnings.catch_warnings(record=True))
# These are resolved by name in FixtureEngine.resolve (alongside tmp_path).
# ============================================================================

CaptureResult = collections.namedtuple("CaptureResult", ["out", "err"])


class CaptureFixture:
    """pytest-style capsys/capfd: read text captured by the active Capture.

    tezt already swaps sys.stdout/sys.stderr to _Tee objects for the duration of
    each test (see Capture). This fixture reads those buffers; readouterr()
    returns the text written SINCE THE LAST readouterr() (per-stream offsets into
    the _Tee buffer's full value). The captured text still flows to the normal
    stdout/stderr result fields -- this only offers a programmatic view.
    """

    def __init__(self, out_tee, err_tee):
        self._out = out_tee
        self._err = err_tee
        self._out_pos = 0
        self._err_pos = 0

    def readouterr(self):
        out_all = self._out.buf.getvalue()
        err_all = self._err.buf.getvalue()
        out = out_all[self._out_pos:]
        err = err_all[self._err_pos:]
        self._out_pos = len(out_all)
        self._err_pos = len(err_all)
        return CaptureResult(out, err)

    def disabled(self):
        # Best-effort: temporarily restore the real streams so writes inside the
        # block pass through instead of being captured. tezt captures at the
        # Python level, so this swaps sys.stdout/err to the _Tee mirrors (the
        # real streams under --no-capture) or the saved real streams, and back.
        fixture = self

        class _Disabled:
            def __enter__(self_inner):
                self_inner._old = (sys.stdout, sys.stderr)
                sys.stdout = fixture._out.mirror or _REAL_STDOUT or sys.__stdout__
                sys.stderr = fixture._err.mirror or _REAL_STDERR or sys.__stderr__
                return fixture

            def __exit__(self_inner, *a):
                sys.stdout, sys.stderr = self_inner._old
                return False

        return _Disabled()


class LogCaptureFixture:
    """pytest-style caplog: capture logging records via a root-logger handler.

    On construction a handler is attached to the root logger that appends every
    LogRecord to .records. set_level/at_level adjust levels (remembering the
    originals); remove() detaches the handler and restores any changed levels and
    is registered as a ctx finalizer so it runs during test teardown.
    """

    def __init__(self):
        records = []

        class _ListHandler(logging.Handler):
            def emit(self_inner, record):
                records.append(record)

        self.records = records
        self.handler = _ListHandler()
        self._formatter = logging.Formatter()
        self._root = logging.getLogger()
        # Remember the root logger's level so set_level can restore it, and so
        # the handler sees records even if the root level was higher.
        self._initial_handler_level = self.handler.level
        # (logger, original_level) pairs to restore on teardown, newest last.
        self._level_restores = []
        self._root.addHandler(self.handler)

    # -- introspection --------------------------------------------------------

    @property
    def text(self):
        return "\n".join(self._formatter.format(r) for r in self.records)

    @property
    def messages(self):
        return [r.getMessage() for r in self.records]

    @property
    def record_tuples(self):
        return [(r.name, r.levelno, r.getMessage()) for r in self.records]

    # -- level control --------------------------------------------------------

    def _logger(self, logger):
        if logger is None:
            return self._root
        if isinstance(logger, str):
            return logging.getLogger(logger)
        return logger

    def set_level(self, level, logger=None):
        lg = self._logger(logger)
        self._level_restores.append((lg, lg.level))
        lg.setLevel(level)
        self.handler.setLevel(level)

    def at_level(self, level, logger=None):
        fixture = self
        lg = self._logger(logger)

        class _AtLevel:
            def __enter__(self_inner):
                self_inner._old_logger = lg.level
                self_inner._old_handler = fixture.handler.level
                lg.setLevel(level)
                fixture.handler.setLevel(level)
                return fixture

            def __exit__(self_inner, *a):
                lg.setLevel(self_inner._old_logger)
                fixture.handler.setLevel(self_inner._old_handler)
                return False

        return _AtLevel()

    def clear(self):
        del self.records[:]

    # -- teardown -------------------------------------------------------------

    def remove(self):
        self._root.removeHandler(self.handler)
        for lg, old in reversed(self._level_restores):
            lg.setLevel(old)
        self._level_restores = []
        self.handler.setLevel(self._initial_handler_level)


class WarningsRecorder:
    """pytest-style recwarn: records warnings for the duration of a test.

    Wraps warnings.catch_warnings(record=True): __enter__ gives the recorded
    list and simplefilter('always') ensures every warning is captured. finish()
    closes the context and is registered as a ctx finalizer. Exposes list-like
    access plus pop()/clear() over the recorded warnings.
    """

    def __init__(self):
        self._cm = warnings.catch_warnings(record=True)
        self.list = self._cm.__enter__()
        warnings.simplefilter("always")

    def __len__(self):
        return len(self.list)

    def __getitem__(self, i):
        return self.list[i]

    def __iter__(self):
        return iter(self.list)

    def pop(self, cls=Warning):
        """Return and remove the first recorded warning matching `cls`."""
        for i, w in enumerate(self.list):
            if issubclass(w.category, cls):
                return self.list.pop(i)
        raise AssertionError("%r not found in warning list" % cls)

    def clear(self):
        del self.list[:]

    def finish(self):
        self._cm.__exit__(None, None, None)


# ============================================================================
# Collection -- turn a (file, qualname) item into concrete test "plan" dicts.
# Plans are param-expanded specs cached per (module file, qualname) so that
# signature inspection and mark evaluation happen once per test function.
# ============================================================================

_plan_cache = {}


def _evaluate_static_marks(marks, module):
    """Pre-evaluate skip/skipif/xfail marks.

    Returns (skip_reason, xfail_flag, xfail_strict). `xfail` honors an optional
    condition (first positional arg or `condition=`) and `strict=`: pytest treats
    a strict xfail that unexpectedly passes as a failure, not an xpass.
    """
    skip_reason = None
    xfail = False
    xfail_strict = False

    def _eval_cond(cond):
        if isinstance(cond, str):
            try:
                return bool(eval(cond, {"sys": sys, "os": os, "config": None, **vars(module)}))
            except Exception:
                return True
        return bool(cond)

    for m in marks:
        if m.name == "skip":
            skip_reason = m.kwargs.get("reason") or (m.args[0] if m.args else "") or "skip"
        elif m.name == "skipif":
            raw = m.args[0] if m.args else m.kwargs.get("condition", False)
            if _eval_cond(raw):
                skip_reason = m.kwargs.get("reason", "") or "skipif"
        elif m.name == "xfail":
            # xfail with no condition always applies; with one, only when truthy.
            if m.args:
                applies = _eval_cond(m.args[0])
            elif "condition" in m.kwargs:
                applies = _eval_cond(m.kwargs["condition"])
            else:
                applies = True
            if applies:
                xfail = True
                if m.kwargs.get("strict"):
                    xfail_strict = True
    return skip_reason, xfail, xfail_strict


def build_plans(module, file_path, qualname, lookup):
    """Return list of plan dicts for one qualname (cached per module/qualname).

    `lookup` (the file's fixture tables) lets plan-building discover which
    requested fixtures are parametrized; it is stable per file, so the cache key
    stays (file_path, qualname) with nothing extra.
    """
    key = (file_path, qualname)
    cached = _plan_cache.get(key)
    if cached is not None:
        return cached

    plans = []
    if "::" in qualname:
        # Walk the class chain so nested classes resolve too:
        # `TestOuter::TestInner::test_deep` -> module.TestOuter.TestInner, then
        # the method. pytest instantiates the INNERMOST class (nesting is just
        # namespacing), so `cls` is the last class in the chain.
        parts = qualname.split("::")
        fn_name = parts[-1]
        obj = module
        for cls_name in parts[:-1]:
            obj = getattr(obj, cls_name, None)
            if obj is None or not inspect.isclass(obj):
                raise FixtureError("class %r not found in %s" % (cls_name, file_path))
        cls = obj
        func = getattr(cls, fn_name, None)
        if func is None:
            raise FixtureError(
                "method %r not found on %s" % (fn_name, "::".join(parts[:-1])))
        plans.extend(_plan_one(module, cls, fn_name, func, lookup, prefix=None))
    else:
        func = getattr(module, qualname, None)
        if func is None or not callable(func):
            raise FixtureError("test %r not found in %s" % (qualname, file_path))
        plans.extend(_plan_one(module, None, qualname, func, lookup, prefix=None))

    _plan_cache[key] = plans
    return plans


def discover_all(module, file_path, id_prefix, lookup):
    """qualname '*': all top-level test_* funcs + Test* classes' test_* methods."""
    key = (file_path, "*", id_prefix)
    cached = _plan_cache.get(key)
    if cached is not None:
        return cached
    plans = []
    for name in sorted(vars(module)):
        obj = vars(module)[name]
        if name.startswith("test_") and inspect.isfunction(obj):
            plans.extend(_plan_one(module, None, name, obj, lookup,
                                   prefix="%s::%s" % (id_prefix, name)))
        elif (name.startswith("Test") and inspect.isclass(obj)
              and obj.__module__ == module.__name__
              and "__init__" not in obj.__dict__):
            for mname in sorted(vars(obj)):
                mobj = vars(obj)[mname]
                if mname.startswith("test_") and callable(mobj):
                    plans.extend(_plan_one(
                        module, obj, mname, mobj, lookup,
                        prefix="%s::%s::%s" % (id_prefix, name, mname)))
    _plan_cache[key] = plans
    return plans


def _plan_one(module, cls, fn_name, func, lookup, prefix=None):
    """Expand one function into cases. Marks evaluated ONCE here (perf).

    Two independent expansions multiply together: the test's @parametrize combos
    (expand_parametrize) and, for every parametrized fixture in the requested
    fixtures' dependency closure, that fixture's params. The resulting case
    carries both the @parametrize `params` dict and a `fixture_params` dict
    {fixture_name: value}; its id fragment is the @parametrize fragment followed
    by each fixture-param fragment, joined with '-' in closure (post-order).
    """
    raw = inspect.unwrap(func) if hasattr(func, "__wrapped__") else func
    marks = collect_marks(func, cls, module)
    skip_reason, xfail, xfail_strict = _evaluate_static_marks(marks, module)
    combos = expand_parametrize(marks)
    try:
        all_params = _signature_params(raw)
    except (TypeError, ValueError):
        all_params = []
    is_async = inspect.iscoroutinefunction(raw)

    plans = []
    for frag, params in combos:
        fixture_names = [p for p in all_params if p not in params]
        # Parametrized fixtures in this test's dependency closure (deterministic
        # order). Each contributes a cartesian factor of its param values. We
        # iterate over INDEX tuples (not value tuples) so duplicate values get
        # correct positional ids.
        param_fixtures = collect_param_fixtures(fixture_names, lookup or [])
        if not param_fixtures:
            fx_cases = [("", {})]       # single empty fixture-param combo
        else:
            index_ranges = [range(len(pf[1])) for pf in param_fixtures]
            fx_cases = []
            for index_combo in itertools.product(*index_ranges):
                frags = []
                fparams = {}
                for (fname, vals, fids), pos in zip(param_fixtures, index_combo):
                    val = vals[pos]
                    frags.append(_fixture_param_id(val, pos, fids))
                    fparams[fname] = val
                fx_cases.append(("-".join(frags), fparams))

        for fx_frag, fx_params in fx_cases:
            # Compose the id fragment: @parametrize fragment then fixture frags.
            pieces = [p for p in (frag, fx_frag) if p]
            combined = "-".join(pieces) if pieces else None
            plans.append({
                "id_suffix": ("[%s]" % combined) if combined is not None else "",
                "prefix": prefix,           # None when item id is used directly
                "func": raw, "cls": cls, "module": module,
                "params": params, "fixture_names": fixture_names,
                "fixture_params": fx_params,
                "skip_reason": skip_reason, "xfail": xfail,
                "xfail_strict": xfail_strict, "is_async": is_async,
            })
    return plans


# ============================================================================
# xunit-style hooks: module/function/class/method setup & teardown
# ============================================================================

class XUnitState:
    """Tracks which xunit setups ran so teardowns fire at scope boundaries."""

    def __init__(self):
        self.module = None        # module whose setup_module ran
        self.cls = None           # class whose setup_class ran

    def enter_module(self, module):
        if self.module is module:
            return
        self.leave_class()
        self.leave_module()
        setup = getattr(module, "setup_module", None)
        if setup:
            _call_hook(setup, module)
        self.module = module

    def leave_module(self):
        if self.module is not None:
            td = getattr(self.module, "teardown_module", None)
            if td:
                try:
                    _call_hook(td, self.module)
                except Exception:
                    debug("teardown_module raised:\n%s" % tb_module.format_exc())
            self.module = None

    def enter_class(self, cls):
        if self.cls is cls:
            return
        self.leave_class()
        if cls is not None:
            setup = getattr(cls, "setup_class", None)
            if setup:
                setup()      # classmethod-style
        self.cls = cls

    def leave_class(self):
        if self.cls is not None:
            td = getattr(self.cls, "teardown_class", None)
            if td:
                try:
                    td()
                except Exception:
                    debug("teardown_class raised:\n%s" % tb_module.format_exc())
            self.cls = None


def _call_hook(hook, arg):
    """Call setup_module/teardown_module style hooks with 0 or 1 args."""
    try:
        n = len(_signature_params(hook))
    except (TypeError, ValueError):
        n = 0
    if n >= 1:
        hook(arg)
    else:
        hook()


# ============================================================================
# Execution of a single test case
# ============================================================================

def run_case(plan, result_id, engine, xunit, lookup, emit, batch_id):
    """Execute one expanded test case; emit a result event."""
    func, cls, module = plan["func"], plan["cls"], plan["module"]
    captured = Capture()
    ctx = TestContext()
    # Expose the active capture to fixture resolution (capsys/capfd) BEFORE the
    # `with captured:` block swaps the streams and before fixtures resolve.
    ctx.capture = captured
    # Resolution context for parametrized fixtures and request.getfixturevalue.
    ctx.fixture_params = plan.get("fixture_params", {})
    ctx.lookup = lookup
    ctx.result_id = result_id
    ctx.engine = engine
    outcome, message, trace = "passed", None, None
    duration_ms = 0.0

    # static skip mark -> no setup at all
    if plan["skip_reason"] is not None:
        emit_result(emit, batch_id, result_id, "skipped", 0.0,
                    plan["skip_reason"], None, "", "")
        return

    with captured:
        instance = None
        setup_fn_td = None      # pending teardown_function/teardown_method
        item = None             # HookItem, built lazily so teardown can reuse it
        try:
            # ---- setup phase (errors -> outcome "error") -------------------
            # Build the hook `item` first so pytest_runtest_setup runs before
            # any fixture/xunit setup (mirrors pytest's ordering) and so the
            # same object is available for pytest_runtest_teardown below. A
            # Skipped raised here propagates to the setup-phase handler (which
            # maps it to "skipped"); any other exception becomes "error".
            item = HookItem(result_id, getattr(func, "__name__", result_id),
                            func, cls, module, _CONFIG, _SESSION)
            call_hooks("pytest_runtest_setup", item=item)

            xunit.enter_module(module)
            xunit.enter_class(cls)
            # Switch the fixture engine's class scope in lockstep with the
            # xunit class hooks. cls may be None for a module-level function,
            # which tears down any live class scope (we left the prior class).
            engine.switch_class(cls)
            # Autouse fixtures run for every test in their scope without being
            # requested by name. Resolve them BEFORE the explicitly-requested
            # fixtures, ordered higher-scope-first; caching makes module/session
            # autouse run once and their teardown fire at the right scope
            # boundary. An autouse fixture that raises is a setup error.
            for name in autouse_fixtures(lookup):
                engine.resolve(name, lookup, ctx, ())
            kwargs = dict(plan["params"])
            for name in plan["fixture_names"]:
                kwargs[name] = engine.resolve(name, lookup, ctx, ())

            if cls is not None:
                instance = cls()                       # fresh instance per test
                sm = getattr(instance, "setup_method", None)
                if sm:
                    _call_hook(sm, func)
                tdm = getattr(instance, "teardown_method", None)
                if tdm:
                    setup_fn_td = lambda: _call_hook(tdm, func)
                call = func.__get__(instance, cls)
            else:
                sf = getattr(module, "setup_function", None)
                if sf:
                    _call_hook(sf, func)
                tdf = getattr(module, "teardown_function", None)
                if tdf:
                    setup_fn_td = lambda: _call_hook(tdf, func)
                call = func

            # ---- call phase (timed) ---------------------------------------
            try:
                start = time.perf_counter()
                if plan["is_async"]:
                    # Reuse the worker's shared loop (not asyncio.run, which
                    # creates and closes a fresh loop each call) so async
                    # fixtures and this test live on the same loop.
                    engine.loop().run_until_complete(call(**kwargs))
                else:
                    call(**kwargs)
                duration_ms = (time.perf_counter() - start) * 1000.0
                if plan["xfail"]:
                    if plan.get("xfail_strict"):
                        # pytest: a strict xfail that unexpectedly passes is a
                        # failure, not an xpass.
                        outcome = "failed"
                        message = "[XPASS(strict)] unexpectedly passed"
                    else:
                        outcome, message = "xpassed", "unexpectedly passed"
            except BaseException as exc:
                duration_ms = (time.perf_counter() - start) * 1000.0
                if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                    raise
                kind = _classify_exception(exc)
                if kind == "skipped":
                    outcome, message = "skipped", str(exc)
                elif kind == "xfailed":
                    outcome, message = "xfailed", str(exc)
                elif plan["xfail"]:
                    outcome, message = "xfailed", str(exc) or type(exc).__name__
                else:
                    outcome = "failed"
                    enriched = _enrich_assertion(
                        exc, exc.__traceback__, module.__file__ or "")
                    message = enriched or ("%s: %s" % (type(exc).__name__, exc)
                                           if str(exc) else type(exc).__name__)
                    trace = _format_traceback(exc)

        except BaseException as exc:
            # setup-phase failure (fixture/hook/instantiation) -> "error";
            # a skip raised inside a fixture still counts as skipped.
            if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                raise
            kind = _classify_exception(exc)
            if kind == "skipped":
                outcome, message = "skipped", str(exc)
            else:
                outcome = "error"
                message = str(exc) or type(exc).__name__
                trace = _format_traceback(exc)
        finally:
            # ---- teardown phase (failures noted but never mask outcome) ----
            # pytest_runtest_teardown runs first (pytest order: teardown hooks
            # before fixture/xunit teardown). A hook error here is swallowed and
            # logged -- a misbehaving teardown hook must never crash the worker
            # or change a test's recorded outcome (matching how the existing
            # xunit teardowns swallow their errors). `item` may be None if the
            # setup hook itself failed before assignment completed.
            if item is not None:
                try:
                    call_hooks("pytest_runtest_teardown", item=item)
                except Exception:
                    debug("pytest_runtest_teardown hook raised:\n%s"
                          % tb_module.format_exc())
            try:
                if setup_fn_td is not None:
                    setup_fn_td()
            except Exception:
                if outcome == "passed":
                    outcome = "error"
                    message = "teardown failed"
                    trace = tb_module.format_exc()
            # Fixture (yield) teardown: a failure here turns a passing test into
            # an error, matching pytest. A test that already failed/errored keeps
            # its outcome (the teardown error is secondary).
            td_err = ctx.teardown(engine)
            if td_err is not None and outcome == "passed":
                outcome = "error"
                message = "fixture teardown failed: %s: %s" % (
                    type(td_err).__name__, td_err)
                trace = _format_traceback(td_err)

    emit_result(emit, batch_id, result_id, outcome, duration_ms,
                message, trace, captured.out.value(), captured.err.value())


# ============================================================================
# Protocol I/O
# ============================================================================

def emit_event(obj):
    _REAL_STDOUT.write(json.dumps(obj, ensure_ascii=False, default=str) + "\n")
    _REAL_STDOUT.flush()


def emit_result(emit, batch_id, rid, outcome, duration_ms, message, trace, out, err):
    emit({
        "event": "result", "batch_id": batch_id, "id": rid,
        "outcome": outcome, "duration_ms": round(duration_ms, 3),
        "message": message, "traceback": trace,
        "stdout": out, "stderr": err,
    })


def handle_run(cmd, engine, xunit):
    batch_id = cmd.get("batch_id", 0)
    items = cmd.get("items", [])

    # Group consecutive items by file so module-scope teardown fires correctly.
    for item in items:
        item_id = item.get("id", "")
        file_path = os.path.abspath(item.get("file", ""))
        qualname = item.get("qualname", "*")

        # ---- import module + conftests (failure -> error result(s)) --------
        try:
            chain = conftest_chain(file_path)
            module = import_module_from_path(file_path)
        except BaseException as exc:
            if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                raise
            trace = _format_traceback(exc)
            msg = "%s: %s" % (type(exc).__name__, exc)
            emit_result(emit_event, batch_id, item_id, "error", 0.0,
                        msg, trace, "", "")
            continue

        # ---- discover hooks from this file's conftest chain + test module --
        # The root conftest's configure/sessionstart hooks already fired in
        # main(); here we additionally pick up pytest_runtest_setup/teardown
        # (and any further configure/sessionfinish) defined in NEARER conftests
        # or in the test module itself. register_hooks_from is id-guarded so a
        # conftest shared by many files is scanned only once.
        for conftest_mod in chain:
            register_hooks_from(conftest_mod)
        register_hooks_from(module)

        # module-scope fixture boundary
        if engine.current_module_path != file_path:
            xunit.leave_class()
            xunit.leave_module()
            engine.switch_module(file_path)

        # fixture lookup order: test module, then conftests nearest dir first
        lookup = _lookup_cache.get(file_path)
        if lookup is None:
            lookup = [scan_fixtures(module)]
            for cf in reversed(chain):          # nearest conftest first
                lookup.append(scan_fixtures(cf))
            _lookup_cache[file_path] = lookup

        # ---- collect plans ---------------------------------------------------
        try:
            if qualname == "*":
                plans = discover_all(module, file_path, item_id, lookup)
            else:
                plans = build_plans(module, file_path, qualname, lookup)
        except BaseException as exc:
            if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                raise
            emit_result(emit_event, batch_id, item_id, "error", 0.0,
                        str(exc) or type(exc).__name__,
                        _format_traceback(exc), "", "")
            continue

        # ---- run each expanded case ------------------------------------------
        for plan in plans:
            if plan["prefix"] is not None:        # '*' discovery: prefixed ids
                rid = plan["prefix"] + plan["id_suffix"]
            else:
                rid = item_id + plan["id_suffix"]
            run_case(plan, rid, engine, xunit, lookup, emit_event, batch_id)

    emit_event({"event": "batch_done", "batch_id": batch_id})


_lookup_cache = {}   # file path -> fixture lookup tables


# ============================================================================
# --list-fixtures one-shot mode
# ============================================================================

# Built-in fixtures the worker provides without any user definition. Listed so
# `--list-fixtures` reports them alongside user/conftest fixtures.
_BUILTIN_FIXTURES = [
    ("tmp_path", "function", "Temporary directory unique to each test."),
    ("tmp_path_factory", "session", "Session-scoped temporary-directory factory."),
    ("monkeypatch", "function", "Safely patch/undo attrs, items, and env vars."),
    ("capsys", "function", "Capture text written to sys.stdout/sys.stderr."),
    ("capfd", "function", "Capture output at the file-descriptor level."),
    ("caplog", "function", "Capture log records emitted during the test."),
    ("recwarn", "function", "Record warnings emitted during the test."),
    ("request", "function", "Info about the requesting test and fixtures."),
]


def _fixture_location(func):
    """'<file>:<lineno>' for a fixture function, or '' if undeterminable."""
    try:
        src = inspect.getsourcefile(func) or inspect.getfile(func)
    except (TypeError, OSError):
        src = None
    if not src:
        return ""
    try:
        _, lineno = inspect.getsourcelines(func)
    except (OSError, TypeError):
        lineno = 0
    return "%s:%d" % (src, lineno)


def _fixture_doc(func):
    """First line of a fixture func's docstring, or ''."""
    doc = inspect.getdoc(func)
    if not doc:
        return ""
    return doc.strip().splitlines()[0].strip()


def _iter_list_paths(paths):
    """Yield .py files to scan for fixtures from the given path args.

    A path may be a file (used directly) or a directory (walked for
    test_*.py / *_test.py, mirroring the Rust collector). Order is
    deterministic. Non-.py files and missing paths are skipped.
    """
    seen = set()

    def add(p):
        ap = os.path.abspath(p)
        if ap not in seen and os.path.isfile(ap) and ap.endswith(".py"):
            seen.add(ap)
            return [ap]
        return []

    out = []
    for p in paths:
        ap = os.path.abspath(p)
        if os.path.isdir(ap):
            for dirpath, dirnames, filenames in os.walk(ap):
                dirnames.sort()
                for fn in sorted(filenames):
                    if (fn.startswith("test_") and fn.endswith(".py")) \
                            or fn.endswith("_test.py"):
                        out += add(os.path.join(dirpath, fn))
        else:
            out += add(ap)
    return out


def list_fixtures(paths):
    """Import the root conftest and the given paths (+ their conftest chains),
    scan every fixture, and emit ONE {"event":"fixtures","fixtures":[...]} event.

    Each fixture entry is {"name","scope","location","doc"}; builtins use
    location "builtin". The list is de-duplicated by name (first definition wins,
    matching lookup precedence) and sorted by name. Import errors emit
    {"event":"fatal", ...} and the function returns 1; success returns 0.
    """
    found = {}   # name -> {"name","scope","location","doc"}

    def add(name, scope, location, doc):
        if name not in found:
            found[name] = {"name": name, "scope": scope,
                           "location": location, "doc": doc}

    # Builtins first (so a user fixture of the same name would override only if
    # added earlier; user fixtures are added after, so builtins win on conflict,
    # mirroring resolve() which checks builtins before user tables).
    for name, scope, doc in _BUILTIN_FIXTURES:
        add(name, scope, "builtin", doc)

    try:
        # Root conftest (if any) -- import so its fixtures are visible.
        root_conftest = os.path.join(ROOTDIR, "conftest.py")
        modules_to_scan = []
        if os.path.isfile(root_conftest):
            modules_to_scan.append(import_module_from_path(root_conftest))
        for fpath in _iter_list_paths(paths):
            for cf in conftest_chain(fpath):
                modules_to_scan.append(cf)
            modules_to_scan.append(import_module_from_path(fpath))
    except BaseException as exc:
        if isinstance(exc, (KeyboardInterrupt, SystemExit)):
            raise
        emit_event({"event": "fatal",
                    "message": "%s: %s" % (type(exc).__name__, exc),
                    "traceback": _format_traceback(exc, style="auto")})
        return 1

    # Scan each module's fixtures (id-dedup so a shared conftest scans once).
    scanned = set()
    for mod in modules_to_scan:
        if mod is None or id(mod) in scanned:
            continue
        scanned.add(id(mod))
        for name, entry in scan_fixtures(mod).items():
            add(name, entry.scope, _fixture_location(entry.func),
                _fixture_doc(entry.func))

    fixtures = [found[k] for k in sorted(found)]
    emit_event({"event": "fixtures", "fixtures": fixtures})
    return 0


# ============================================================================
# Main loop
# ============================================================================

def main():
    global _REAL_STDOUT, _REAL_STDERR, ROOTDIR, NO_CAPTURE, TB_STYLE, \
        _CONFIG, _SESSION, _COVERAGE

    parser = argparse.ArgumentParser()
    parser.add_argument("--rootdir", required=True)
    parser.add_argument("--no-capture", action="store_true")
    # Traceback style for failing tests (the Rust side passes --tb <style>).
    parser.add_argument("--tb", choices=["auto", "long", "short", "line", "no"],
                        default="auto")
    # One-shot fixture listing: when set, main() does NOT enter the run loop;
    # it imports the given paths, scans their fixtures, emits a single
    # {"event":"fixtures","fixtures":[...]} event, and returns. `paths` are the
    # positional test files/dirs the Rust side passes alongside --rootdir.
    parser.add_argument("--list-fixtures", action="store_true")
    parser.add_argument("paths", nargs="*")
    # ---- coverage.py options (presence of --cov-data-dir enables coverage) ---
    # The Rust supervisor passes exactly these three flags when coverage is on:
    #   --cov-data-dir DIR   target directory for per-worker .coverage data files
    #   --cov-source SRC     repeatable; restricts measurement to these sources
    #   --cov-branch         enable branch (not just line) coverage
    parser.add_argument("--cov-data-dir")
    parser.add_argument("--cov-source", action="append", default=[])
    parser.add_argument("--cov-branch", action="store_true")
    args = parser.parse_args()

    ROOTDIR = os.path.abspath(args.rootdir)
    NO_CAPTURE = args.no_capture
    TB_STYLE = args.tb
    _REAL_STDOUT = sys.stdout
    _REAL_STDERR = sys.stderr

    # ---- start coverage BEFORE any user import -----------------------------
    # Coverage must begin before we import the virtual `tezt` module and, more
    # importantly, before any conftest/test module is imported, so that
    # import-time (module top-level) lines are measured too. `coverage` is a
    # third-party package imported lazily here only when measurement is asked
    # for, keeping the default worker stdlib-only. It writes solely to its data
    # file, never to stdout, so the JSON-Lines protocol is unaffected.
    if args.cov_data_dir:
        try:
            import coverage
        except ImportError as exc:
            # The Rust side pre-checks for coverage, but be defensive: emit a
            # clear fatal and bail rather than crash mid-protocol.
            emit_event({
                "event": "fatal",
                "message": ("coverage measurement requested but the 'coverage'"
                            " package is not importable: %s" % exc),
            })
            return 1
        data_file = os.path.join(args.cov_data_dir, ".coverage.%d" % os.getpid())
        _COVERAGE = coverage.Coverage(
            data_file=data_file,
            source=(args.cov_source or None),
            branch=bool(args.cov_branch),
        )
        _COVERAGE.start()

    sys.path.insert(0, ROOTDIR)
    sys.modules["tezt"] = _make_tezt_module()

    # ---- one-shot --list-fixtures mode: list and exit (no run loop) --------
    # The Rust side spawns the worker with `--list-fixtures --rootdir <dir>
    # <paths...>`. We import the targets, scan their fixtures, emit a single
    # `fixtures` event, and return without emitting `ready` or reading stdin.
    if args.list_fixtures:
        return list_fixtures(args.paths)

    # ---- hook lifecycle: configure + sessionstart (once per worker) --------
    # Build the per-worker config/session, then eagerly discover the ROOT
    # conftest's hooks so pytest_configure/pytest_sessionstart see them. Doing
    # this here (rather than lazily on first run) gives a deterministic, single
    # configure/sessionstart per worker. Per-file conftests and test modules are
    # scanned later (in handle_run) for runtest_setup/teardown hooks. If there
    # is no root conftest we still fire configure/sessionstart with no extra
    # hooks so the lifecycle is consistent across workers.
    _CONFIG = _Config(ROOTDIR)
    _SESSION = _Session(_CONFIG)
    root_conftest = os.path.join(ROOTDIR, "conftest.py")
    if os.path.isfile(root_conftest):
        try:
            register_hooks_from(import_module_from_path(root_conftest))
        except BaseException as exc:
            # A broken root conftest is fatal: tests in that tree cannot run
            # meaningfully without it. Report and bail (coverage, if started, is
            # abandoned -- the process is exiting anyway).
            if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                raise
            emit_event({"event": "fatal",
                        "message": "root conftest import failed: %s" % exc,
                        "traceback": _format_traceback(exc)})
            return 1
    # configure runs before sessionstart (pytest order). Hook errors here are
    # swallowed+logged: a misbehaving configure/sessionstart must not crash the
    # worker before it can serve any test.
    try:
        call_hooks("pytest_configure", config=_CONFIG)
    except Exception:
        debug("pytest_configure hook raised:\n%s" % tb_module.format_exc())
    try:
        call_hooks("pytest_sessionstart", session=_SESSION)
    except Exception:
        debug("pytest_sessionstart hook raised:\n%s" % tb_module.format_exc())

    pyver = "%d.%d.%d" % sys.version_info[:3]
    emit_event({"event": "ready", "pid": os.getpid(), "python": pyver})

    engine = FixtureEngine()
    xunit = XUnitState()
    stdin = sys.stdin

    def shutdown_worker():
        # Single shutdown sequence used by BOTH the explicit `shutdown` command
        # and the stdin-closed path. Order matters:
        #   1. xunit + fixture session teardown (user teardowns run first),
        #   2. pytest_sessionfinish (after teardown, exitstatus always 0 since
        #      tezt has no worker-level pass/fail aggregate -- per-worker session),
        #   3. stop + save coverage (last, so coverage captures every teardown
        #      and sessionfinish line too).
        # Steps 2 and 3 are guarded so a bad hook / coverage error can never
        # prevent the worker from saying `bye` and exiting cleanly.
        xunit.leave_class()
        xunit.leave_module()
        engine.teardown_session_scope()
        try:
            call_hooks("pytest_sessionfinish", session=_SESSION, exitstatus=0)
        except Exception:
            debug("pytest_sessionfinish hook raised:\n%s" % tb_module.format_exc())
        if _COVERAGE is not None:
            try:
                _COVERAGE.stop()
                _COVERAGE.save()
            except Exception:
                debug("coverage stop/save failed:\n%s" % tb_module.format_exc())
        emit_event({"event": "bye"})

    try:
        for line in stdin:
            line = line.strip()
            if not line:
                continue
            try:
                cmd = json.loads(line)
            except ValueError:
                debug("ignoring malformed protocol line: %r" % line[:200])
                continue
            kind = cmd.get("cmd")
            if kind == "run":
                handle_run(cmd, engine, xunit)
            elif kind == "shutdown":
                shutdown_worker()
                return 0
            else:
                debug("unknown cmd: %r" % kind)
        # stdin closed without shutdown: tidy up quietly
        shutdown_worker()
        return 0
    except Exception as exc:
        emit_event({"event": "fatal", "message": str(exc) or type(exc).__name__,
                    "traceback": _format_traceback(exc)})
        return 1


if __name__ == "__main__":
    sys.exit(main())
