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
| collect | tezt collect | 62 ms | 90.1x |
| collect | pytest collect | 5557 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 335 ms | 19.9x |
| full run | pytest | 6654 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9876 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 151.0x |
| collect | pytest collect | 4400 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 267 ms | 17.9x |
| full run | pytest | 4777 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7303 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 134.8x |
| collect | pytest collect | 4020 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 308 ms | 16.7x |
| full run | pytest | 5142 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8208 ms | 0.6x |

## macos-latest / py3.13

- python: `3.13.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 197.2x |
| collect | pytest collect | 2947 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 185 ms | 20.9x |
| full run | pytest | 3862 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5948 ms | 0.6x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 17 ms | 296.7x |
| collect | pytest collect | 4902 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 301 ms | 21.8x |
| full run | pytest | 6571 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8636 ms | 0.8x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 258.4x |
| collect | pytest collect | 6211 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 403 ms | 21.3x |
| full run | pytest | 8588 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14732 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 228.1x |
| collect | pytest collect | 5749 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 359 ms | 21.4x |
| full run | pytest | 7695 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12142 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 225.9x |
| collect | pytest collect | 5907 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 476 ms | 16.6x |
| full run | pytest | 7921 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12584 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 198.3x |
| collect | pytest collect | 5810 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 505 ms | 16.0x |
| full run | pytest | 8078 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13865 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 246.9x |
| collect | pytest collect | 4966 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 433 ms | 14.7x |
| full run | pytest | 6368 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10083 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 198.5x |
| collect | pytest collect | 6175 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 365 ms | 24.4x |
| full run | pytest | 8893 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14856 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 139.8x |
| collect | pytest collect | 6494 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 700 ms | 14.2x |
| full run | pytest | 9963 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15059 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 152.1x |
| collect | pytest collect | 5307 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 420 ms | 19.2x |
| full run | pytest | 8067 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 16112 ms | 0.5x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 126.9x |
| collect | pytest collect | 5571 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 617 ms | 13.9x |
| full run | pytest | 8595 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 16461 ms | 0.5x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 52 ms | 113.9x |
| collect | pytest collect | 5925 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 602 ms | 15.2x |
| full run | pytest | 9156 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13482 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 161.3x |
| collect | pytest collect | 5632 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 435 ms | 19.9x |
| full run | pytest | 8651 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14474 ms | 0.6x |
