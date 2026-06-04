# Conformance harness — tezt vs pytest

A differential test: the same little suites are run through **both** pytest (the
oracle) and tezt, and we compare the per-test **verdict**
(passed / failed / skipped / xfailed / xpassed / error) and the process **exit
code**. Message wording is deliberately *not* compared — only "given this
situation, what result does the runner produce", which is the part that has to
match for tezt to be a drop-in.

Each scenario lives inline in [`run.py`](run.py); add one with `scn(name, src)`.
Tag a scenario with `note=` when a difference is intentional/architectural — it
is then reported as `DIVERGE*` rather than counted as a failure.

## Running

You need pytest in a venv (kept out of the repo):

```sh
python3 -m venv /tmp/conf-venv
/tmp/conf-venv/bin/pip install pytest
cargo build --release

TEZT_BIN=target/release/tezt VENV_PY=/tmp/conf-venv/bin/python \
    python3 conformance/run.py          # add -v to print matches too
```

Exit 0 means tezt matches pytest on every scenario except the tagged
divergences.

## Known, intentional divergences

- **`async def` tests** — tezt runs them natively on a worker event loop; plain
  pytest (no asyncio plugin) does not, so the verdicts differ. This is a tezt
  feature, not a bug.
- **Syntax error in a test file** — pytest reports a *collection* error and exits
  `2`; tezt reports the file as one `error` item and exits `1`, so the rest of
  the suite still runs. A deliberate design choice.

The behaviors the harness pins (and that CI guards via `tests/integration.rs`,
without needing pytest) include: pass/fail/error/skip/xfail/xpass classification,
strict xfail, fixture-teardown errors, nested classes, parametrize, fixtures
(autouse / parametrized / yield), `approx`, `raises`/`warns`, `capsys`/`caplog`,
and exit codes.
