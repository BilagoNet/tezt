# tezt benchmark matrix

Produced by the [Benchmarks workflow](../.github/workflows/bench.yml) across the
OS x Python matrix. Every cell runs the same generated suite under tezt and pytest,
with tezt's collection cache disabled so it's a fair parse-vs-import comparison.
Times are wall-clock medians; `speedup` is versus single-process pytest in the
same phase.

## macos-latest / py3.10

- python: `3.10.11`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 33 ms | 201.8x |
| collect | pytest collect | 6689 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 325 ms | 25.0x |
| full run | pytest | 8109 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10764 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 183.1x |
| collect | pytest collect | 4726 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 372 ms | 20.4x |
| full run | pytest | 7594 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8683 ms | 0.9x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 130.1x |
| collect | pytest collect | 3418 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 193 ms | 21.6x |
| full run | pytest | 4162 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6838 ms | 0.6x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 238.2x |
| collect | pytest collect | 5059 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 305 ms | 18.7x |
| full run | pytest | 5709 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10170 ms | 0.6x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 17 ms | 272.4x |
| collect | pytest collect | 4593 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 383 ms | 16.1x |
| full run | pytest | 6174 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8285 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 204.2x |
| collect | pytest collect | 5965 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 399 ms | 21.6x |
| full run | pytest | 8624 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14574 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 160.1x |
| collect | pytest collect | 5189 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 333 ms | 20.5x |
| full run | pytest | 6840 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11492 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 192.0x |
| collect | pytest collect | 6061 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 512 ms | 16.5x |
| full run | pytest | 8431 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14031 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 197.0x |
| collect | pytest collect | 6227 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 514 ms | 16.2x |
| full run | pytest | 8352 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14055 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 202.4x |
| collect | pytest collect | 6206 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 577 ms | 14.0x |
| full run | pytest | 8048 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13840 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 33 ms | 175.1x |
| collect | pytest collect | 5797 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 373 ms | 22.9x |
| full run | pytest | 8524 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14712 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 161.6x |
| collect | pytest collect | 6384 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 507 ms | 19.3x |
| full run | pytest | 9788 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14351 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 132.6x |
| collect | pytest collect | 5587 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 491 ms | 17.9x |
| full run | pytest | 8805 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13199 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 150.5x |
| collect | pytest collect | 5540 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 583 ms | 14.9x |
| full run | pytest | 8695 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13518 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 47 ms | 113.9x |
| collect | pytest collect | 5306 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 584 ms | 14.3x |
| full run | pytest | 8330 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12818 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 154.2x |
| collect | pytest collect | 5384 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 422 ms | 19.7x |
| full run | pytest | 8334 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12585 ms | 0.7x |
