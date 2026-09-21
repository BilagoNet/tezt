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
| collect | tezt collect | 31 ms | 144.8x |
| collect | pytest collect | 4434 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 312 ms | 17.7x |
| full run | pytest | 5521 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8794 ms | 0.6x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 212.3x |
| collect | pytest collect | 3118 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 193 ms | 21.4x |
| full run | pytest | 4121 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5894 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 264.1x |
| collect | pytest collect | 4133 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 227 ms | 23.5x |
| full run | pytest | 5331 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7638 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.15`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 233.2x |
| collect | pytest collect | 4905 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 357 ms | 18.5x |
| full run | pytest | 6592 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9336 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 129.8x |
| collect | pytest collect | 4552 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 279 ms | 27.2x |
| full run | pytest | 7576 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9253 ms | 0.8x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 188.5x |
| collect | pytest collect | 5726 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 392 ms | 21.3x |
| full run | pytest | 8331 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14366 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 136.5x |
| collect | pytest collect | 2765 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 182 ms | 20.5x |
| full run | pytest | 3723 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6415 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 217.5x |
| collect | pytest collect | 5885 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 493 ms | 17.0x |
| full run | pytest | 8353 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13543 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 156.9x |
| collect | pytest collect | 6064 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 487 ms | 16.4x |
| full run | pytest | 8008 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13364 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 17 ms | 228.8x |
| collect | pytest collect | 3814 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 326 ms | 14.8x |
| full run | pytest | 4835 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7671 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 150.1x |
| collect | pytest collect | 6038 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 349 ms | 24.5x |
| full run | pytest | 8564 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13713 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 153.4x |
| collect | pytest collect | 6611 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 522 ms | 19.4x |
| full run | pytest | 10113 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15587 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 142.7x |
| collect | pytest collect | 4171 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 337 ms | 18.5x |
| full run | pytest | 6233 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9760 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 45 ms | 135.8x |
| collect | pytest collect | 6086 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 493 ms | 19.1x |
| full run | pytest | 9448 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14286 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 135.1x |
| collect | pytest collect | 5826 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 661 ms | 13.4x |
| full run | pytest | 8884 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15361 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 119.8x |
| collect | pytest collect | 5541 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 492 ms | 18.7x |
| full run | pytest | 9203 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13961 ms | 0.7x |
