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
| collect | tezt collect | 30 ms | 186.7x |
| collect | pytest collect | 5682 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 312 ms | 23.1x |
| full run | pytest | 7214 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9266 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 215.4x |
| collect | pytest collect | 5026 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 310 ms | 20.0x |
| full run | pytest | 6204 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8988 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 200.1x |
| collect | pytest collect | 5940 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 380 ms | 20.1x |
| full run | pytest | 7658 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9563 ms | 0.8x |

## macos-latest / py3.13

- python: `3.13.15`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 121.2x |
| collect | pytest collect | 4697 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 293 ms | 19.8x |
| full run | pytest | 5807 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8089 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 232.6x |
| collect | pytest collect | 7011 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 372 ms | 24.0x |
| full run | pytest | 8929 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12076 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 220.1x |
| collect | pytest collect | 4519 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 280 ms | 21.8x |
| full run | pytest | 6102 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10725 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 201.5x |
| collect | pytest collect | 5521 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 367 ms | 21.8x |
| full run | pytest | 7991 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12974 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 208.6x |
| collect | pytest collect | 6193 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 510 ms | 16.7x |
| full run | pytest | 8510 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14272 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 200.6x |
| collect | pytest collect | 4795 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 382 ms | 16.2x |
| full run | pytest | 6187 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10299 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 236.6x |
| collect | pytest collect | 6175 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 588 ms | 14.2x |
| full run | pytest | 8346 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14212 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 224.2x |
| collect | pytest collect | 6317 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 380 ms | 24.0x |
| full run | pytest | 9123 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15362 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 136.7x |
| collect | pytest collect | 5718 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 460 ms | 18.9x |
| full run | pytest | 8684 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14456 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 142.7x |
| collect | pytest collect | 4088 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 338 ms | 18.3x |
| full run | pytest | 6188 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9516 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 130.1x |
| collect | pytest collect | 5424 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 434 ms | 20.0x |
| full run | pytest | 8668 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12851 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 141.3x |
| collect | pytest collect | 5357 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 596 ms | 13.2x |
| full run | pytest | 7842 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12438 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 130.2x |
| collect | pytest collect | 5611 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 499 ms | 18.5x |
| full run | pytest | 9215 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14214 ms | 0.6x |
