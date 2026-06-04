# tezt benchmark matrix

Produced by the [Benchmarks workflow](../.github/workflows/bench.yml) across the
OS x Python matrix. Every cell runs the same generated suite under tezt and pytest,
with tezt's collection cache disabled so it's a fair parse-vs-import comparison.
Times are wall-clock medians; `speedup` is versus single-process pytest in the
same phase.

## macos-latest / py3.10

- python: `3.10.11`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 209.9x |
| collect | pytest collect | 5724 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 256 ms | 28.9x |
| full run | pytest | 7395 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9401 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 56 ms | 64.8x |
| collect | pytest collect | 3635 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 248 ms | 19.8x |
| full run | pytest | 4920 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8533 ms | 0.6x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 33 ms | 157.8x |
| collect | pytest collect | 5178 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 249 ms | 26.2x |
| full run | pytest | 6542 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7357 ms | 0.9x |

## macos-latest / py3.13

- python: `3.13.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 196.0x |
| collect | pytest collect | 4188 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 229 ms | 21.0x |
| full run | pytest | 4806 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6950 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 246.1x |
| collect | pytest collect | 6669 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 346 ms | 25.0x |
| full run | pytest | 8638 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12354 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 252.8x |
| collect | pytest collect | 5987 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 339 ms | 26.0x |
| full run | pytest | 8805 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14743 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 199.0x |
| collect | pytest collect | 5130 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 274 ms | 24.5x |
| full run | pytest | 6714 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11077 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 228.2x |
| collect | pytest collect | 5920 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 414 ms | 19.0x |
| full run | pytest | 7886 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12770 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.13`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 271.8x |
| collect | pytest collect | 5738 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 420 ms | 17.6x |
| full run | pytest | 7380 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13782 ms | 0.5x |

## ubuntu-latest / py3.14

- python: `3.14.5`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 256.6x |
| collect | pytest collect | 6046 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 517 ms | 15.8x |
| full run | pytest | 8157 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14279 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 246.7x |
| collect | pytest collect | 5969 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 324 ms | 26.7x |
| full run | pytest | 8630 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14672 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 150.5x |
| collect | pytest collect | 5953 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 398 ms | 24.1x |
| full run | pytest | 9604 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14550 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 137.7x |
| collect | pytest collect | 5363 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 389 ms | 20.9x |
| full run | pytest | 8137 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14162 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 134.5x |
| collect | pytest collect | 5351 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 503 ms | 16.0x |
| full run | pytest | 8051 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12884 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.13`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 48 ms | 129.7x |
| collect | pytest collect | 6178 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 609 ms | 16.0x |
| full run | pytest | 9755 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 17448 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 161.2x |
| collect | pytest collect | 5542 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 435 ms | 19.6x |
| full run | pytest | 8508 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13472 ms | 0.6x |
