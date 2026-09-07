# tezt benchmark matrix

Produced by the [Benchmarks workflow](../.github/workflows/bench.yml) across the
OS x Python matrix. Every cell runs the same generated suite under tezt and pytest,
with tezt's collection cache disabled so it's a fair parse-vs-import comparison.
Times are wall-clock medians; `speedup` is versus single-process pytest in the
same phase.

## macos-latest / py3.10

- python: `3.10.11`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 262.4x |
| collect | pytest collect | 5862 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 338 ms | 23.8x |
| full run | pytest | 8059 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10582 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 170.1x |
| collect | pytest collect | 5442 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 294 ms | 22.7x |
| full run | pytest | 6680 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9033 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 194.5x |
| collect | pytest collect | 3545 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 209 ms | 21.3x |
| full run | pytest | 4447 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6577 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.15`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 166.2x |
| collect | pytest collect | 3467 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 263 ms | 16.8x |
| full run | pytest | 4433 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5913 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 304.9x |
| collect | pytest collect | 6318 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 354 ms | 25.2x |
| full run | pytest | 8900 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10750 ms | 0.8x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 202.1x |
| collect | pytest collect | 4539 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 285 ms | 21.7x |
| full run | pytest | 6198 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10214 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 199.8x |
| collect | pytest collect | 5305 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 357 ms | 20.9x |
| full run | pytest | 7452 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12218 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 217.4x |
| collect | pytest collect | 6090 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 504 ms | 16.6x |
| full run | pytest | 8361 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13746 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 206.8x |
| collect | pytest collect | 5803 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 508 ms | 15.8x |
| full run | pytest | 8022 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13750 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 194.8x |
| collect | pytest collect | 6526 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 594 ms | 14.7x |
| full run | pytest | 8718 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14855 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 186.8x |
| collect | pytest collect | 5824 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 340 ms | 23.5x |
| full run | pytest | 7995 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13068 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 166.4x |
| collect | pytest collect | 5783 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 670 ms | 13.8x |
| full run | pytest | 9267 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14958 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 143.8x |
| collect | pytest collect | 5609 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 490 ms | 17.9x |
| full run | pytest | 8791 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13498 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 154.8x |
| collect | pytest collect | 5501 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 440 ms | 18.4x |
| full run | pytest | 8102 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14146 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 153.3x |
| collect | pytest collect | 5803 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 617 ms | 15.5x |
| full run | pytest | 9573 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 16795 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 156.9x |
| collect | pytest collect | 6066 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 661 ms | 15.1x |
| full run | pytest | 9988 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13933 ms | 0.7x |
