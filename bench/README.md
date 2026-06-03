# tezt benchmarks

Reproducible wall-time comparison of **tezt** vs **pytest** (and
pytest-xdist): collection time, full-run time, amortized ms/test, and
cold-start latency. Python 3 stdlib only; `pip install pytest
pytest-xdist` is the only external dependency (for the pytest side).

## Reproduce (3 commands)

```sh
# 1. generate a 10,000-case suite (identical plain files run under BOTH runners)
python3 bench/gen_suite.py --out /tmp/suite_trivial_10k --files 500 --tests-per-file 20 --style trivial --flavor plain

# 2. benchmark both runners (5 runs each, 4 jobs), append to results.json
python3 bench/run_bench.py --tezt-bin target/release/tezt --suite /tmp/suite_trivial_10k --label trivial-10k --runs 5 --jobs 4

# 3. read the report
cat bench/RESULTS.md
```

## What gets measured

| phase | tezt | pytest |
|---|---|---|
| collection | `tezt <dir> --collect-only -q` | `python3 -m pytest -q --collect-only <dir>` |
| full run | `tezt <dir> -j J -q` | `python3 -m pytest -q <dir>` and `-n J` (xdist, if installed) |
| cold start | 1-file/1-test temp suite, full run | same |

Each measurement is the **median of N runs** (`--runs`, default 5) of
`time.perf_counter` around `subprocess.run` — i.e. true end-to-end wall
time including process startup. `ms/test` = median / collected cases ×
1000. Both runners' summaries are parsed for collected/passed counts
and the harness **aborts if the runners disagree**, if any process
exits non-zero, or if a run exceeds 300 s. `.pytest_cache` and `.pyc`
writing are disabled so repeated runs are comparable and the suite dir
stays pristine.

## Suite styles

- `--style trivial --flavor plain` (default): pure arithmetic asserts,
  **zero imports** — the exact same files run under both runners. Use
  this for headline numbers.
- `--style mixed --flavor pytest|tezt`: 70 % trivial / 15 % parametrized
  (5 cases each) / 10 % module-scoped-fixture / 5 % class-based.
  Because parametrize/fixture decorators come from the runner's own
  module, generate it **twice** (same `--files/--tests-per-file`, one
  per flavor) and pass each runner its own copy — the generator is
  deterministic, so the test logic is byte-for-byte identical apart
  from the `import pytest` / `import tezt` line:

  ```sh
  python3 bench/gen_suite.py --out /tmp/mixed_pt --files 200 --tests-per-file 20 --style mixed --flavor pytest
  python3 bench/gen_suite.py --out /tmp/mixed_tz --files 200 --tests-per-file 20 --style mixed --flavor tezt
  python3 bench/run_bench.py --only pytest --suite /tmp/mixed_pt --label mixed-4k --runs 5 --jobs 4
  python3 bench/run_bench.py --only tezt  --tezt-bin target/release/tezt --suite /tmp/mixed_tz --label mixed-4k-tezt --runs 5 --jobs 4
  ```

## Useful flags

- `--only tezt|pytest|all` — benchmark one side only (e.g. pytest
  baselines before the tezt binary exists).
- `--runs N`, `--jobs J` — repetitions and parallelism.
- `--label STR` — results for the same label are replaced, not
  duplicated, so re-running updates in place.
- `--skip-cold` — omit the cold-start micro benchmark.
- `--python PATH` — interpreter used for pytest (defaults to the one
  running the script).

Outputs: `bench/results.json` (structured, accumulates across suites)
and `bench/RESULTS.md` (regenerated from the JSON on every run, with an
environment header and a headline line).

## Fairness notes

- Identical files for both runners in trivial/plain mode; identical
  logic in mixed mode.
- `PYTHONDONTWRITEBYTECODE=1` and `-p no:cacheprovider` for pytest, so
  neither runner benefits from on-disk caches between runs.
- All generated tests pass; exit code 0 is asserted, so a runner can't
  "win" by skipping work.
- Run on an idle machine; CPU count, Python version, platform and date
  are recorded in the report header.
