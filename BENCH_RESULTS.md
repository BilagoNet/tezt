# tezt benchmark results

**Headline:** tezt runs 10,000 tests in 0.19s (0.02 ms/test) -- 71.6x faster than pytest.

- date: 2026-06-04 01:37:34
- python: 3.9.9
- cpu_count: 11
- platform: Darwin 25.6.0 (arm64)

Wall time measured with `time.perf_counter` around `subprocess.run`; each row is the median of N runs. `ms/test` = median seconds / collected cases x 1000 (amortized per-test overhead, includes process startup). Speedup is vs the single-process `pytest` row in the same suite+mode group.

## suite: trivial-10k  (10000 test cases)
_measured 2026-06-04 01:37:34; runs per row: 5; jobs: 8_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| tezt collect | 0.0186 | 0.0180 | 0.5959 | 542.45x | 0.002 |
| pytest collect | 10.1090 | 10.0199 | 10.1851 | 1.00x (baseline) | 1.011 |
| tezt -j 8 | 0.1854 | 0.1833 | 0.1923 | 71.58x | 0.018 |
| pytest | 13.2688 | 13.2069 | 13.3408 | 1.00x (baseline) | 1.327 |
| pytest -n 8 (xdist) | 19.5332 | 19.1883 | 19.7102 | 0.68x | 1.953 |

## suite: cold-start (via trivial-10k)  (1 test cases)
_measured 2026-06-04 01:37:34; runs per row: 5; jobs: 1_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| tezt cold (1 test) | 0.0613 | 0.0604 | 0.0841 | 9.38x | 61.297 |
| pytest cold (1 test) | 0.5752 | 0.5735 | 0.6812 | 1.00x (baseline) | 575.174 |

