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
| collect | tezt collect | 48 ms | 135.0x |
| collect | pytest collect | 6540 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 363 ms | 21.4x |
| full run | pytest | 7746 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10946 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 262.6x |
| collect | pytest collect | 5744 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 425 ms | 13.7x |
| full run | pytest | 5828 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7881 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 166.9x |
| collect | pytest collect | 6347 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 337 ms | 18.5x |
| full run | pytest | 6222 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8892 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.17`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 133.3x |
| collect | pytest collect | 5907 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 376 ms | 13.6x |
| full run | pytest | 5118 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8870 ms | 0.6x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 53 ms | 125.6x |
| collect | pytest collect | 6689 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 470 ms | 18.8x |
| full run | pytest | 8815 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13462 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 223.2x |
| collect | pytest collect | 6205 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 377 ms | 22.1x |
| full run | pytest | 8331 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14391 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.18`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 226.9x |
| collect | pytest collect | 4845 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 266 ms | 21.0x |
| full run | pytest | 5591 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8705 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.16`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 208.6x |
| collect | pytest collect | 4591 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 357 ms | 16.0x |
| full run | pytest | 5723 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9680 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.18`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 205.2x |
| collect | pytest collect | 3186 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 305 ms | 13.3x |
| full run | pytest | 4043 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7135 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.10`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 195.1x |
| collect | pytest collect | 6129 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 576 ms | 14.3x |
| full run | pytest | 8267 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13786 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 155.7x |
| collect | pytest collect | 3329 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 276 ms | 17.5x |
| full run | pytest | 4835 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8199 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 136.8x |
| collect | pytest collect | 5920 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 487 ms | 17.8x |
| full run | pytest | 8662 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13792 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 135.1x |
| collect | pytest collect | 5814 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 536 ms | 14.5x |
| full run | pytest | 7770 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12700 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.17`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 45 ms | 133.3x |
| collect | pytest collect | 5970 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 598 ms | 14.7x |
| full run | pytest | 8774 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14357 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 149.2x |
| collect | pytest collect | 6543 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 504 ms | 18.7x |
| full run | pytest | 9443 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14253 ms | 0.7x |
