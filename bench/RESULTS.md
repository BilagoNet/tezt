# tezt benchmark results

**Headline:** tezt runs 10,000 tests in 0.19s (0.02 ms/test) -- 71.0x faster than pytest.

- date: 2026-06-04 02:58:21
- python: 3.9.9
- cpu_count: 11
- platform: Darwin 25.6.0 (arm64)

Wall time measured with `time.perf_counter` around `subprocess.run`; each row is the median of N runs. `ms/test` = median seconds / collected cases x 1000 (amortized per-test overhead, includes process startup). Speedup is vs the single-process `pytest` row in the same suite+mode group.

## suite: trivial-10k  (10000 test cases)
_measured 2026-06-04 02:58:21; runs per row: 5; jobs: 8_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| tezt collect | 0.0186 | 0.0177 | 0.5964 | 562.73x | 0.002 |
| pytest collect | 10.4646 | 10.3957 | 10.7738 | 1.00x (baseline) | 1.046 |
| tezt -j 8 | 0.1915 | 0.1896 | 0.2180 | 71.04x | 0.019 |
| pytest | 13.6078 | 13.4581 | 13.6571 | 1.00x (baseline) | 1.361 |
| pytest -n 8 (xdist) | 18.2916 | 18.1564 | 20.3923 | 0.74x | 1.829 |

## suite: cold-start (via trivial-10k)  (1 test cases)
_measured 2026-06-04 02:58:21; runs per row: 5; jobs: 1_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| tezt cold (1 test) | 0.0566 | 0.0566 | 0.0789 | 9.26x | 56.620 |
| pytest cold (1 test) | 0.5242 | 0.5188 | 0.6420 | 1.00x (baseline) | 524.181 |

