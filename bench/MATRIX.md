# tezt benchmark matrix

Produced by the [Benchmarks workflow](../.github/workflows/bench.yml) across the
OS x Python matrix. Every cell runs the same generated suite under tezt and pytest,
with tezt's collection cache disabled so it's a fair parse-vs-import comparison.
Times are wall-clock medians; `speedup` is versus single-process pytest in the
same phase.

## macos-latest / py3.10

- python: `3.10.11`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 45 ms | 136.7x |
| collect | pytest collect | 6123 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 307 ms | 18.7x |
| full run | pytest | 5739 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7369 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 216.4x |
| collect | pytest collect | 4981 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 248 ms | 24.4x |
| full run | pytest | 6046 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8048 ms | 0.8x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 141.3x |
| collect | pytest collect | 4097 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 344 ms | 15.0x |
| full run | pytest | 5178 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8866 ms | 0.6x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 91.3x |
| collect | pytest collect | 3793 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 292 ms | 18.9x |
| full run | pytest | 5530 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9446 ms | 0.6x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 193.0x |
| collect | pytest collect | 3543 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 211 ms | 23.2x |
| full run | pytest | 4898 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10636 ms | 0.5x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 243.5x |
| collect | pytest collect | 5584 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 302 ms | 23.9x |
| full run | pytest | 7224 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11600 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 176.6x |
| collect | pytest collect | 5148 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 360 ms | 19.9x |
| full run | pytest | 7169 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11840 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 201.5x |
| collect | pytest collect | 4399 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 352 ms | 16.6x |
| full run | pytest | 5828 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9702 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 204.8x |
| collect | pytest collect | 5065 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 396 ms | 15.4x |
| full run | pytest | 6089 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10012 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 217.0x |
| collect | pytest collect | 3802 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 333 ms | 14.5x |
| full run | pytest | 4819 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7913 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 218.4x |
| collect | pytest collect | 4375 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 260 ms | 22.9x |
| full run | pytest | 5949 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10529 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 171.9x |
| collect | pytest collect | 6287 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 683 ms | 15.6x |
| full run | pytest | 10637 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14862 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 119.9x |
| collect | pytest collect | 5464 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 433 ms | 19.1x |
| full run | pytest | 8255 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12893 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 153.4x |
| collect | pytest collect | 5367 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 430 ms | 18.4x |
| full run | pytest | 7893 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12305 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 142.1x |
| collect | pytest collect | 3694 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 416 ms | 12.8x |
| full run | pytest | 5306 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8303 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 54 ms | 120.3x |
| collect | pytest collect | 6532 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 655 ms | 16.5x |
| full run | pytest | 10831 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14454 ms | 0.7x |
