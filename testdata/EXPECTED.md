# Expected outcomes per testdata suite

Counts are per full-suite run (`tezt testdata/<suite>`), deterministic and
order-independent. A blank/0 cell means the summary line must not report that
category. "collected" is the number of expanded test cases (`--collect-only`).

| suite         | collected | passed | failed | skipped | xfailed | xpassed | error | exit code |
|---------------|-----------|--------|--------|---------|---------|---------|-------|-----------|
| basic         | 4         | 3      | 1      | 0       | 0       | 0       | 0     | 1         |
| classes       | 6         | 6      | 0      | 0       | 0       | 0       | 0     | 0         |
| fixtures      | 7         | 7      | 0      | 0       | 0       | 0       | 0     | 0         |
| parametrize   | 16        | 16     | 0      | 0       | 0       | 0       | 0     | 0         |
| skips         | 6         | 1      | 0      | 3       | 1       | 1       | 0     | 0         |
| asyncio_suite | 3         | 2      | 1      | 0       | 0       | 0       | 0     | 1         |
| failures      | 2 (+1 collection error) | 0 | 2 | 0  | 0       | 0       | 1     | 1         |
| kfilter       | 3         | 3      | 0      | 0       | 0       | 0       | 0     | 0         |
| marks         | 4         | 4      | 0      | 0       | 0       | 0       | 0     | 0         |
| pytest_compat | 17        | 10     | 1      | 4       | 1       | 1       | 0     | 1         |
| empty         | 0         | 0      | 0      | 0       | 0       | 0       | 0     | 5         |

## Per-suite notes

### basic/ (test_math.py)
3 passing arithmetic tests; `test_wrong_addition` fails on `assert 2 + 2 == 5`.

### classes/ (test_classes.py)
- `TestCounter` (3 tests): asserts `setup_class`/`setup_method`/`teardown_method`
  hooks ran, with order-independent counter invariants.
- `TestSimple` (2 tests), plus 1 module-level test = 6 collected.
- `NotATestClass` (bad name) and `TestWithInit` (has `__init__`) must NOT be
  collected; each contains a test that raises if ever run.

### fixtures/ (conftest.py + test_fixtures.py)
7 passing tests covering: conftest fixture, module-scope caching (asserted via
an instantiation counter — valid for both shared-process and per-process
workers), fixture-depends-on-fixture, yield fixture writing setup/teardown to a
file under `tmp_path` (the test asserts only "setup" is present during the
body), builtin `tmp_path`, and a session-scope list fixture whose ordering
assertions are self-contained (`setup count == teardown count + 1` while the
test's own resource is live).

### parametrize/ (test_param.py)
16 cases total, all passing:
- `test_square`: 5 (single arg)
- `test_add`: 3 (multi-arg tuples `"a,b,expected"`)
- `test_concat`: 6 (stacked decorators, 2 prefixes x 3 suffixes)
- `test_is_even`: 2 with `ids=["two", "four"]` → ids `test_is_even[two]`,
  `test_is_even[four]`

### skips/ (test_skips.py)
6 tests: `mark.skip` → skipped; `mark.skipif(True)` → skipped;
`mark.skipif(False)` → passed; `mark.xfail` + failing body → xfailed;
`mark.xfail` + passing body → xpassed (non-strict, does not affect exit code);
`tezt.skip()` in body → skipped. Net: 1 passed, 3 skipped, 1 xfailed,
1 xpassed, exit 0.

### asyncio_suite/ (test_async.py)
3 bare `async def` tests run via asyncio: 2 pass, `test_async_failing` fails.
(Stock pytest without a plugin cannot run these; tezt runs them natively.)

### failures/
- `test_assert_locals.py`: 1 failed (assertion with several locals in scope).
- `test_raises_error.py`: 1 failed (uncaught `ValueError`).
- `test_syntax_error.py`: INTENTIONAL SyntaxError → 1 collection/import
  **error** outcome. Do not fix this file; `py_compile` rejects it by design.
- `test_empty.py`: compiles fine, contributes 0 tests.
Net: 0 passed, 2 failed, 1 error, exit 1. With `-x` the run stops at the first
failure/error, so strictly fewer than (2 failed + 1 error) are reported.

### kfilter/ (test_names.py)
`test_alpha`, `test_beta`, `TestGamma::test_delta` — all pass.
`-k alpha` → 1; `-k "alpha or beta"` → 2; `-k "not alpha"` → 2; `-k delta` → 1.

### marks/ (test_marks.py)
4 tests carrying `@tezt.mark` decorators (so the suite runs without pytest):
`test_slow_one` (slow), `test_slow_and_net` (slow + net), `test_net_only`
(net), `test_unmarked` (none). Marks are read statically at collection, so
`-m` selection needs no Python import. `-m slow` → 2; `-m net` → 2;
`-m "slow and net"` → 1; `-m "not slow"` → 2. All selected tests pass.

### pytest_compat/ (conftest.py + test_pytest_style.py + test_pytest_marked.py)
Pure `import pytest` style; only meaningful when pytest is importable.
Verified against real pytest 9.0.3:
`1 failed, 10 passed, 4 skipped, 1 xfailed, 1 xpassed` (17 collected), exit 1.
Breakdown: conftest fixture (1 pass), yield fixture (1 pass), parametrize 4
cases (pass), parametrize with ids 2 cases (pass), `pytest.raises(..., match=)`
(1 pass), `skipif(False)` (1 pass), `test_xfail_passing` (xpassed),
`test_xfail_failing` (xfailed), `mark.skip` + imperative `pytest.skip()` +
2 tests under module-level `pytestmark = pytest.mark.skip` (4 skipped),
`test_compat_failing` (1 failed).

### empty/
Contains only `.gitkeep`; zero collectable test files → "no tests collected",
exit code 5 (pytest parity).
