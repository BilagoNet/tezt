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
| collect | tezt collect | 14 ms | 203.3x |
| collect | pytest collect | 2926 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 180 ms | 21.6x |
| full run | pytest | 3893 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6259 ms | 0.6x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 161.7x |
| collect | pytest collect | 4173 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 238 ms | 20.1x |
| full run | pytest | 4776 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7062 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 150.9x |
| collect | pytest collect | 3561 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 233 ms | 18.9x |
| full run | pytest | 4405 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6590 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 19 ms | 209.5x |
| collect | pytest collect | 3985 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 208 ms | 24.6x |
| full run | pytest | 5117 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6346 ms | 0.8x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 127.5x |
| collect | pytest collect | 4641 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 310 ms | 18.6x |
| full run | pytest | 5752 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7794 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 243.2x |
| collect | pytest collect | 5863 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 384 ms | 21.9x |
| full run | pytest | 8399 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14273 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 205.0x |
| collect | pytest collect | 5157 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 326 ms | 21.0x |
| full run | pytest | 6832 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11350 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 244.1x |
| collect | pytest collect | 5848 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 489 ms | 16.5x |
| full run | pytest | 8077 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13649 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.13`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 204.1x |
| collect | pytest collect | 5834 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 498 ms | 16.3x |
| full run | pytest | 8133 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14314 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.5`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 238.4x |
| collect | pytest collect | 5889 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 548 ms | 14.0x |
| full run | pytest | 7646 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12631 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 238.8x |
| collect | pytest collect | 5893 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 360 ms | 23.5x |
| full run | pytest | 8463 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14645 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 168.8x |
| collect | pytest collect | 6479 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 525 ms | 20.0x |
| full run | pytest | 10529 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15357 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 151.6x |
| collect | pytest collect | 4232 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 344 ms | 18.2x |
| full run | pytest | 6274 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9719 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 49 ms | 122.2x |
| collect | pytest collect | 6030 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 586 ms | 15.2x |
| full run | pytest | 8900 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12928 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.13`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 41 ms | 132.0x |
| collect | pytest collect | 5463 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 1244 ms | 7.0x |
| full run | pytest | 8703 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12146 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 126.6x |
| collect | pytest collect | 5592 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 473 ms | 19.2x |
| full run | pytest | 9070 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13726 ms | 0.7x |
