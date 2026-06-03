# Changelog

Notable changes to tezt. The format loosely follows
[Keep a Changelog](https://keepachangelog.com/). tezt is pre-1.0, so behavior and
flags can still change between versions.

## Unreleased

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
