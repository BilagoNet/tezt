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
| collect | tezt collect | 15 ms | 248.4x |
| collect | pytest collect | 3682 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 225 ms | 20.8x |
| full run | pytest | 4690 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6633 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 227.1x |
| collect | pytest collect | 3362 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 182 ms | 23.5x |
| full run | pytest | 4288 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5736 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 5

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 58 ms | 52.1x |
| collect | pytest collect | 3001 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 172 ms | 20.5x |
| full run | pytest | 3528 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 4158 ms | 0.8x |

## macos-latest / py3.13

- python: `3.13.15`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 79 ms | 85.4x |
| collect | pytest collect | 6767 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 497 ms | 18.6x |
| full run | pytest | 9238 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11185 ms | 0.8x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 176.0x |
| collect | pytest collect | 6150 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 373 ms | 18.7x |
| full run | pytest | 6957 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9592 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 225.1x |
| collect | pytest collect | 5188 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 319 ms | 21.9x |
| full run | pytest | 6988 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12160 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 203.6x |
| collect | pytest collect | 5313 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 354 ms | 21.0x |
| full run | pytest | 7447 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12069 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 162.0x |
| collect | pytest collect | 5838 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 468 ms | 16.4x |
| full run | pytest | 7663 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12420 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 206.0x |
| collect | pytest collect | 5937 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 512 ms | 15.9x |
| full run | pytest | 8131 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13841 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 236.7x |
| collect | pytest collect | 6146 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 590 ms | 14.3x |
| full run | pytest | 8423 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14337 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 190.8x |
| collect | pytest collect | 5945 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 375 ms | 23.2x |
| full run | pytest | 8710 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14801 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 138.8x |
| collect | pytest collect | 6156 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 531 ms | 18.1x |
| full run | pytest | 9601 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15233 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 111.2x |
| collect | pytest collect | 4291 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 377 ms | 18.4x |
| full run | pytest | 6935 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10736 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 159.4x |
| collect | pytest collect | 6015 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 495 ms | 18.6x |
| full run | pytest | 9202 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14203 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 148.9x |
| collect | pytest collect | 5610 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 619 ms | 14.0x |
| full run | pytest | 8650 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13596 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 159.1x |
| collect | pytest collect | 4899 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 425 ms | 18.3x |
| full run | pytest | 7796 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12530 ms | 0.6x |
