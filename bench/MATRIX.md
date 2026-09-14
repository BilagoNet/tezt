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
| collect | tezt collect | 22 ms | 208.9x |
| collect | pytest collect | 4500 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 296 ms | 19.9x |
| full run | pytest | 5889 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8545 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 177.4x |
| collect | pytest collect | 3278 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 197 ms | 25.1x |
| full run | pytest | 4936 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6230 ms | 0.8x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 5

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 10 ms | 259.4x |
| collect | pytest collect | 2681 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 132 ms | 26.4x |
| full run | pytest | 3474 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 4022 ms | 0.9x |

## macos-latest / py3.13

- python: `3.13.15`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 178.9x |
| collect | pytest collect | 3554 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 266 ms | 17.6x |
| full run | pytest | 4665 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9913 ms | 0.5x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 69 ms | 66.2x |
| collect | pytest collect | 4570 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 474 ms | 13.2x |
| full run | pytest | 6251 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12036 ms | 0.5x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 233.4x |
| collect | pytest collect | 5106 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 294 ms | 23.6x |
| full run | pytest | 6940 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11172 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 190.0x |
| collect | pytest collect | 5554 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 367 ms | 21.5x |
| full run | pytest | 7903 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12661 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 215.7x |
| collect | pytest collect | 5910 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 498 ms | 16.6x |
| full run | pytest | 8286 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14033 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 169.4x |
| collect | pytest collect | 5974 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 478 ms | 16.0x |
| full run | pytest | 7652 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12612 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 169.5x |
| collect | pytest collect | 6381 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 592 ms | 13.9x |
| full run | pytest | 8208 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14204 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 198.3x |
| collect | pytest collect | 5867 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 370 ms | 23.0x |
| full run | pytest | 8512 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14402 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 159.7x |
| collect | pytest collect | 5990 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 518 ms | 18.2x |
| full run | pytest | 9452 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15719 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 160.2x |
| collect | pytest collect | 5846 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 463 ms | 18.9x |
| full run | pytest | 8724 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14206 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 147.3x |
| collect | pytest collect | 4381 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 350 ms | 18.0x |
| full run | pytest | 6294 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10095 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 138.7x |
| collect | pytest collect | 5597 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 730 ms | 13.1x |
| full run | pytest | 9550 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13468 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 41 ms | 141.9x |
| collect | pytest collect | 5774 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 513 ms | 18.7x |
| full run | pytest | 9582 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14243 ms | 0.7x |
