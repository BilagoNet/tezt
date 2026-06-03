#!/usr/bin/env python3
"""tezt_worker.py -- persistent Python test-execution worker for the `tezt` runner.

Speaks JSON Lines over stdio with the Rust supervisor:
  in : {"cmd":"run","batch_id":N,"items":[{"id","file","qualname"}]} | {"cmd":"shutdown"}
  out: ready / result / batch_done / bye / fatal events (one JSON object per line)

Pure stdlib, Python 3.8+ compatible. Single file by design.
"""

import argparse
import asyncio
import inspect
import io
import importlib.util
import itertools
import json
import linecache
import os
import re
import shutil
import sys
import tempfile
import time
import traceback as tb_module
import types

# ============================================================================
# Constants / globals
# ============================================================================

CAPTURE_LIMIT = 64 * 1024          # truncate captured stdout/stderr fields
TRACEBACK_LINES = 50               # keep last N lines of tracebacks
DEBUG = os.environ.get("TEZT_DEBUG") == "1"

_REAL_STDOUT = None                # protocol channel (saved before any capture)
_REAL_STDERR = None
ROOTDIR = None
NO_CAPTURE = False


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
    """Metadata wrapper a @tezt.fixture decoration produces (callable passthrough)."""

    def __init__(self, func, scope, name):
        self.func = func
        self.scope = scope
        self.name = name or func.__name__
        self._tezt_fixture = True

    def __call__(self, *args, **kwargs):  # allow direct invocation in user code
        return self.func(*args, **kwargs)


def _make_tezt_module():
    mod = types.ModuleType("tezt")
    mod.__doc__ = "Virtual tezt module injected by tezt_worker."

    def fixture(func=None, *, scope="function", name=None):
        if scope not in ("function", "module", "session", "class"):
            raise ValueError("invalid fixture scope: %r" % scope)
        if func is not None:
            return FixtureDef(func, scope, name)

        def deco(f):
            return FixtureDef(f, scope, name)
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

    mod.fixture = fixture
    mod.parametrize = parametrize
    mod.mark = _MarkNamespace()
    mod.skip = skip
    mod.fail = fail
    mod.xfail = xfail
    mod.raises = raises
    mod.Skipped = Skipped
    mod.Failed = Failed
    mod.XFailed = XFailed
    return mod


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
# Fixture discovery -- recognize both tezt FixtureDef and pytest fixtures.
# ============================================================================

def _as_fixture(obj):
    """If obj is a fixture definition (tezt or pytest), return (func, scope, name).

    Returns None for non-fixtures. Never imports pytest itself.
    """
    # tezt fixture
    if isinstance(obj, FixtureDef):
        return obj.func, obj.scope, obj.name
    # pytest >= 8.4: FixtureFunctionDefinition object wrapping the function
    if type(obj).__name__ == "FixtureFunctionDefinition":
        func = getattr(obj, "_fixture_function", None)
        marker = getattr(obj, "_fixture_function_marker", None)
        scope = getattr(marker, "scope", "function") if marker else "function"
        name = getattr(marker, "name", None) if marker else None
        if func is not None:
            return func, _norm_scope(scope), name or getattr(func, "__name__", None)
        return None
    # classic pytest: decorated function carries _pytestfixturefunction marker
    marker = getattr(obj, "_pytestfixturefunction", None)
    if marker is not None:
        func = getattr(obj, "__wrapped__", obj)
        scope = getattr(marker, "scope", "function")
        name = getattr(marker, "name", None)
        return func, _norm_scope(scope), name or getattr(func, "__name__", None)
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
    """Map fixture-name -> (func, scope, source_module) for one module."""
    found = {}
    for attr_name, obj in vars(module).items():
        info = _as_fixture(obj)
        if info is not None:
            func, scope, name = info
            found[name or attr_name] = (func, scope, module)
    return found


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
# Note: "class" scope is treated as module scope in v0 (documented limitation;
# a class-scoped fixture lives until its module is torn down).
# ============================================================================

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
        # per-test (function scope) state lives in a TestContext

    # -- scope lifecycle -----------------------------------------------------

    def switch_module(self, module_path):
        if self.current_module_path == module_path:
            return
        self.teardown_module_scope()
        self.current_module_path = module_path

    def teardown_module_scope(self):
        self._drain(self.module_teardowns)
        self.module_cache.clear()
        self.current_module_path = None

    def teardown_session_scope(self):
        self.teardown_module_scope()
        self._drain(self.session_teardowns)
        self.session_cache.clear()
        self.tmp_factory.cleanup()

    @staticmethod
    def _drain(teardowns):
        while teardowns:
            name, gen = teardowns.pop()
            try:
                next(gen)
            except StopIteration:
                pass
            except Exception:
                debug("teardown of fixture %r raised:\n%s" % (name, tb_module.format_exc()))

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
        if name == "request":   # tolerated minimal stub so common fixtures load
            if "request" not in ctx.cache:
                ctx.cache["request"] = types.SimpleNamespace(
                    param=None, node=None, config=None,
                    addfinalizer=lambda f: ctx.finalizers.append(f))
            return ctx.cache["request"]

        # user-defined fixture lookup
        entry = None
        for table in lookup:
            if name in table:
                entry = table[name]
                break
        if entry is None:
            raise FixtureError("fixture %r not found" % name)
        func, scope, src_module = entry

        cache = self._cache_for(scope, ctx)
        if name in cache:
            return cache[name][0]

        value, gen = self._instantiate(name, func, lookup, ctx, stack)
        cache[name] = (value, gen)
        if gen is not None:
            self._teardowns_for(scope, ctx).append((name, gen))
        return value

    def _cache_for(self, scope, ctx):
        if scope == "session":
            return self.session_cache
        if scope in ("module", "class"):
            return self.module_cache
        return ctx.cache

    def _teardowns_for(self, scope, ctx):
        if scope == "session":
            return self.session_teardowns
        if scope in ("module", "class"):
            return self.module_teardowns
        return ctx.teardowns

    def _instantiate(self, name, func, lookup, ctx, stack):
        if inspect.iscoroutinefunction(func) or inspect.isasyncgenfunction(func):
            raise FixtureError(
                "async fixture %r is not supported in tezt v0 "
                "(use a sync fixture or run async setup inside the test)" % name)
        # Resolve the fixture's own dependencies (recursive)
        kwargs = {}
        try:
            params = _signature_params(func)
        except (TypeError, ValueError):
            params = []
        new_stack = stack + (name,)
        for pname in params:
            kwargs[pname] = self.resolve(pname, lookup, ctx, new_stack)
        try:
            result = func(**kwargs)
        except FixtureError:
            raise
        except BaseException as exc:
            if isinstance(exc, (KeyboardInterrupt, SystemExit)):
                raise
            raise FixtureError(
                "fixture %r raised %s: %s\n%s" % (
                    name, type(exc).__name__, exc, _format_traceback(exc))) from exc
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
                        name, type(exc).__name__, exc, _format_traceback(exc))) from exc
            return value, result
        return result, None


class TestContext:
    """Function-scope fixture state for a single test case."""

    __slots__ = ("cache", "teardowns", "monkeypatches", "cleanup_dirs", "finalizers")

    def __init__(self):
        self.cache = {}
        self.teardowns = []
        self.monkeypatches = []
        self.cleanup_dirs = []
        self.finalizers = []

    def teardown(self):
        FixtureEngine._drain(self.teardowns)
        for f in reversed(self.finalizers):
            try:
                f()
            except Exception:
                pass
        for mp in self.monkeypatches:
            try:
                mp.undo()
            except Exception:
                pass
        for d in self.cleanup_dirs:
            shutil.rmtree(d, ignore_errors=True)


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
# Assertion enrichment -- bare `assert x == y` gets source line + locals.
# ============================================================================

def _enrich_assertion(exc, exc_tb, test_file):
    """For AssertionError with no message: 'assert failed: <line> | locals: ...'."""
    if not isinstance(exc, AssertionError) or exc.args:
        return None
    target = None
    tb = exc_tb
    norm_file = os.path.normcase(os.path.abspath(test_file))
    while tb is not None:           # innermost frame located in the test file
        fname = os.path.normcase(os.path.abspath(tb.tb_frame.f_code.co_filename))
        if fname == norm_file:
            target = tb
        tb = tb.tb_next
    if target is None:
        return None
    frame = target.tb_frame
    line = linecache.getline(frame.f_code.co_filename, target.tb_lineno).strip()
    parts = []
    for k, v in list(frame.f_locals.items())[:8]:
        if k.startswith("__"):
            continue
        try:
            r = repr(v)
        except Exception:
            r = "<unreprable>"
        if len(r) > 120:
            r = r[:117] + "..."
        parts.append("%s=%s" % (k, r))
    msg = "assert failed: %s" % (line or "<source unavailable>")
    if parts:
        msg += " | locals: " + ", ".join(parts)
    return msg


def _format_traceback(exc, limit_lines=TRACEBACK_LINES):
    # Hide tezt's own worker frames so the traceback starts at the user's test,
    # the way pytest hides its framework frames. We drop the leading frames whose
    # file is this worker module (run_case and the call plumbing).
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
# Collection -- turn a (file, qualname) item into concrete test "plan" dicts.
# Plans are param-expanded specs cached per (module file, qualname) so that
# signature inspection and mark evaluation happen once per test function.
# ============================================================================

_plan_cache = {}


def _evaluate_static_marks(marks, module):
    """Pre-evaluate skip/skipif/xfail marks. Returns (skip_reason, xfail_flag)."""
    skip_reason = None
    xfail = False
    for m in marks:
        if m.name == "skip":
            skip_reason = m.kwargs.get("reason") or (m.args[0] if m.args else "") or "skip"
        elif m.name == "skipif":
            cond = m.args[0] if m.args else m.kwargs.get("condition", False)
            if isinstance(cond, str):
                try:
                    cond = eval(cond, {"sys": sys, "os": os,
                                       "config": None, **vars(module)})
                except Exception:
                    cond = True
            if cond:
                skip_reason = m.kwargs.get("reason", "") or "skipif"
        elif m.name == "xfail":
            xfail = True
    return skip_reason, xfail


def build_plans(module, file_path, qualname):
    """Return list of plan dicts for one qualname (cached per module/qualname)."""
    key = (file_path, qualname)
    cached = _plan_cache.get(key)
    if cached is not None:
        return cached

    plans = []
    if "::" in qualname:
        cls_name, _, fn_name = qualname.partition("::")
        cls = getattr(module, cls_name, None)
        if cls is None or not inspect.isclass(cls):
            raise FixtureError("class %r not found in %s" % (cls_name, file_path))
        func = getattr(cls, fn_name, None)
        if func is None:
            raise FixtureError("method %r not found on %s" % (fn_name, cls_name))
        plans.extend(_plan_one(module, cls, fn_name, func, prefix=None))
    else:
        func = getattr(module, qualname, None)
        if func is None or not callable(func):
            raise FixtureError("test %r not found in %s" % (qualname, file_path))
        plans.extend(_plan_one(module, None, qualname, func, prefix=None))

    _plan_cache[key] = plans
    return plans


def discover_all(module, file_path, id_prefix):
    """qualname '*': all top-level test_* funcs + Test* classes' test_* methods."""
    key = (file_path, "*", id_prefix)
    cached = _plan_cache.get(key)
    if cached is not None:
        return cached
    plans = []
    for name in sorted(vars(module)):
        obj = vars(module)[name]
        if name.startswith("test_") and inspect.isfunction(obj):
            plans.extend(_plan_one(module, None, name, obj,
                                   prefix="%s::%s" % (id_prefix, name)))
        elif (name.startswith("Test") and inspect.isclass(obj)
              and obj.__module__ == module.__name__
              and "__init__" not in obj.__dict__):
            for mname in sorted(vars(obj)):
                mobj = vars(obj)[mname]
                if mname.startswith("test_") and callable(mobj):
                    plans.extend(_plan_one(
                        module, obj, mname, mobj,
                        prefix="%s::%s::%s" % (id_prefix, name, mname)))
    _plan_cache[key] = plans
    return plans


def _plan_one(module, cls, fn_name, func, prefix):
    """Expand one function into param cases. Marks evaluated ONCE here (perf)."""
    raw = inspect.unwrap(func) if hasattr(func, "__wrapped__") else func
    marks = collect_marks(func, cls, module)
    skip_reason, xfail = _evaluate_static_marks(marks, module)
    combos = expand_parametrize(marks)
    try:
        all_params = _signature_params(raw)
    except (TypeError, ValueError):
        all_params = []
    is_async = inspect.iscoroutinefunction(raw)
    plans = []
    for frag, params in combos:
        fixture_names = [p for p in all_params if p not in params]
        plans.append({
            "id_suffix": ("[%s]" % frag) if frag is not None else "",
            "prefix": prefix,           # None when item id is used directly
            "func": raw, "cls": cls, "module": module,
            "params": params, "fixture_names": fixture_names,
            "skip_reason": skip_reason, "xfail": xfail, "is_async": is_async,
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
        try:
            # ---- setup phase (errors -> outcome "error") -------------------
            xunit.enter_module(module)
            xunit.enter_class(cls)
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
                    asyncio.run(call(**kwargs))
                else:
                    call(**kwargs)
                duration_ms = (time.perf_counter() - start) * 1000.0
                if plan["xfail"]:
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
            try:
                if setup_fn_td is not None:
                    setup_fn_td()
            except Exception:
                if outcome == "passed":
                    outcome = "error"
                    message = "teardown failed"
                    trace = tb_module.format_exc()
            ctx.teardown()

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
                plans = discover_all(module, file_path, item_id)
            else:
                plans = build_plans(module, file_path, qualname)
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
# Main loop
# ============================================================================

def main():
    global _REAL_STDOUT, _REAL_STDERR, ROOTDIR, NO_CAPTURE

    parser = argparse.ArgumentParser()
    parser.add_argument("--rootdir", required=True)
    parser.add_argument("--no-capture", action="store_true")
    args = parser.parse_args()

    ROOTDIR = os.path.abspath(args.rootdir)
    NO_CAPTURE = args.no_capture
    _REAL_STDOUT = sys.stdout
    _REAL_STDERR = sys.stderr

    sys.path.insert(0, ROOTDIR)
    sys.modules["tezt"] = _make_tezt_module()

    pyver = "%d.%d.%d" % sys.version_info[:3]
    emit_event({"event": "ready", "pid": os.getpid(), "python": pyver})

    engine = FixtureEngine()
    xunit = XUnitState()
    stdin = sys.stdin

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
                xunit.leave_class()
                xunit.leave_module()
                engine.teardown_session_scope()
                emit_event({"event": "bye"})
                return 0
            else:
                debug("unknown cmd: %r" % kind)
        # stdin closed without shutdown: tidy up quietly
        xunit.leave_class()
        xunit.leave_module()
        engine.teardown_session_scope()
        emit_event({"event": "bye"})
        return 0
    except Exception as exc:
        emit_event({"event": "fatal", "message": str(exc) or type(exc).__name__,
                    "traceback": _format_traceback(exc)})
        return 1


if __name__ == "__main__":
    sys.exit(main())
