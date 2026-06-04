# Changelog

Notable changes to tezt. The format loosely follows
[Keep a Changelog](https://keepachangelog.com/). tezt is pre-1.0, so behavior and
flags can still change between versions.

## Unreleased

### Added

- **Mark expressions** — `-m "slow and not network"` selects tests by their
  marks, read statically at collection (no import) from `@pytest.mark.*` /
  `@tezt.mark.*` decorators and module- / class-level `pytestmark`.
- **`--lf` / `--ff`** (`--last-failed` / `--failed-first`) — re-run only the
  tests that failed last run, or run them first. The failing set is recorded in
  `.tezt_cache` and merged across runs.
- **Class-scoped and async fixtures** — `@fixture(scope="class")` now has its
  own lifecycle (built once per class, torn down at the class boundary), and
  async fixtures are supported, including `async` generators with teardown.
  Async fixtures and `async def` tests share one per-worker event loop, so a
  resource created in a fixture is valid inside the test that uses it.
- **Rich operator-aware assertion diffs** — a failing bare `assert a == b`
  (and `!=`, `<`, `in`, `is`, …) now prints both operands and a type-aware
  diff: the differing index of a list, the changed key of a dict, the items
  unique to each set, a unified diff of two strings. Operands that contain a
  call fall back to the source-line-plus-locals form, so capturing a value
  never re-runs your code.
- **Coverage** — `--cov` measures coverage with `coverage.py` (each worker
  records a parallel data file; tezt combines and reports them after the run).
  `--cov-source` scopes it, `--cov-report` picks `term` / `term-missing` /
  `html` / `xml`, and `--cov-branch` adds branch coverage.
- **conftest plugin hooks** — tezt now runs the `pytest_*` hooks that fit its
  model from `conftest.py`: `pytest_configure`, `pytest_sessionstart` /
  `pytest_sessionfinish` (per worker), and `pytest_runtest_setup` /
  `pytest_runtest_teardown` (a setup hook may skip a test). Collection-level
  hooks aren't supported, since tezt collects in Rust.
- **Capture / log / warning fixtures** — `capsys` and `capfd` (read captured
  stdout/stderr via `readouterr()`), `caplog` (records / `text` / `set_level` /
  `at_level`), and `recwarn` plus a `tezt.warns(...)` context manager.
- **`tezt.approx`** — tolerant float comparison for scalars, sequences, and
  mappings (`assert 0.1 + 0.2 == tezt.approx(0.3)`).
- **Config file** — an optional `[tool.tezt]` table in `pyproject.toml` with
  `addopts` (default flags, folded in before your argv), `testpaths` (used when
  no path is given), and `markers`.
- **JUnit XML** — `--junitxml PATH` writes a `<testsuites>` report that CI test
  reporters (GitHub, GitLab, Jenkins) understand.
- **Autouse and parametrized fixtures** — `@fixture(autouse=True)` runs for every
  test in scope without being requested; `@fixture(params=[...])` expands a test
  into one case per value, with `request.param` inside the fixture. `request`
  grew up too: `.param`, `.config`, `.node`, `.getfixturevalue(name)`,
  `.addfinalizer(fn)`.
- **`--tb` styles** — `auto` (default), `long`, `short`, `line`, `no` control how
  much traceback a failure prints.
- **`--stepwise` / `--sw`** — stop at the first failure and resume from it on the
  next run (sequential, fail-fast); the resume point lives in `.tezt_cache`.
- **`--fixtures` and `--markers`** — list the fixtures available to a path (with
  scope, location, and docstring) or the built-in + registered markers, then exit.

### Fixed

Found by a new differential conformance harness (`conformance/`) that runs the
same scenarios through tezt and pytest and diffs the verdict + exit code:

- **Strict xfail** — `@mark.xfail(strict=True)` that unexpectedly passes is now
  a `failed`, not an `xpassed` (pytest parity). xfail also honors a condition.
- **Fixture teardown errors** — a `yield` fixture whose teardown raises now turns
  a passing test into an `error` instead of being silently swallowed.
- **Nested test classes** — `Test*` classes nested inside other `Test*` classes
  are now collected (previously silently skipped).

## 0.1.0 — 2026-06-04

The first working version. tezt discovers tests by parsing them in Rust (no
imports) and runs them on a warm pool of persistent Python workers.

### Added

- Discovery for `test_*.py` / `*_test.py`, `test_*` functions, and `Test*`
  classes, with static `parametrize` expansion (including stacked decorators and
  `ids=`).
- Fixtures — function / module / session scope, `yield` teardown, and
  `conftest.py` chains — plus `skip` / `skipif` / `xfail`, async tests, and the
  xunit `setup_*` / `teardown_*` hooks. Works through `@pytest.*` markers or the
  zero-dependency `import tezt` API, with the builtins `tmp_path`,
  `tmp_path_factory`, and `monkeypatch`.
- Interpreter discovery: an active virtualenv, `$CONDA_PREFIX`, a project-local
  `.venv` (up to the project root), then `PATH` — plus the `py` launcher and
  `--python X.Y` version selection on Windows.
- A persistent collection cache (`.tezt_cache`) keyed on file size and mtime, so
  unchanged files are never re-parsed.
- `--timeout` to kill and report a test that runs too long.
- `-k` expressions, `-x` / `--maxfail`, `-j`, `--durations`, `--json`,
  `--collect-only`, `--no-capture`, and pytest-compatible exit codes (0 / 1 / 5).
- Clean interruption: Ctrl-C (or a CI `kill`) tears down every worker and any
  subprocess a test spawned — process groups on unix, a Job Object on Windows.
- Failure output enriched with the failing source line and locals; tracebacks
  start at the user's test, not tezt's internals.

### Known limitations

No plugin ecosystem, assertion rewriting, class-scoped or async fixtures, or mark
expressions (`-m`) yet. A file that imports `pytest` still needs pytest installed.
See the compatibility table in the README.
