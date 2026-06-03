# tezt benchmark results

- date: 2026-06-03 19:23:00
- python: 3.10.12
- cpu_count: 4
- platform: Linux 6.8.0-106-generic (aarch64)

Wall time measured with `time.perf_counter` around `subprocess.run`; each row is the median of N runs. `ms/test` = median seconds / collected cases x 1000 (amortized per-test overhead, includes process startup). Speedup is vs the single-process `pytest` row in the same suite+mode group.

## suite: trivial-200  (200 test cases)
_measured 2026-06-03 19:22:35; runs per row: 5; jobs: 4_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| pytest collect | 0.2058 | 0.1980 | 0.2115 | 1.00x (baseline) | 1.029 |
| pytest | 0.2431 | 0.2397 | 0.2467 | 1.00x (baseline) | 1.216 |
| pytest -n 4 (xdist) | 0.4784 | 0.4616 | 0.5390 | 0.51x | 2.392 |

## suite: cold-start (via trivial-200)  (1 test cases)
_measured 2026-06-03 19:22:35; runs per row: 5; jobs: 1_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| pytest cold (1 test) | 0.0906 | 0.0898 | 0.0965 | 1.00x (baseline) | 90.615 |

## suite: trivial-2k  (2000 test cases)
_measured 2026-06-03 19:23:00; runs per row: 5; jobs: 4_

| runner / mode | median (s) | min (s) | max (s) | speedup vs pytest | ms/test |
|---|---:|---:|---:|---:|---:|
| pytest collect | 1.2550 | 1.2457 | 1.3066 | 1.00x (baseline) | 0.627 |
| pytest | 1.6680 | 1.6629 | 1.7940 | 1.00x (baseline) | 0.834 |
| pytest -n 4 (xdist) | 1.9903 | 1.9538 | 2.0528 | 0.84x | 0.995 |

