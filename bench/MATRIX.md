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
| collect | tezt collect | 15 ms | 235.4x |
| collect | pytest collect | 3528 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 181 ms | 24.0x |
| full run | pytest | 4332 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6165 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 5

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 14 ms | 239.0x |
| collect | pytest collect | 3335 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 154 ms | 26.6x |
| full run | pytest | 4102 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 4053 ms | 1.0x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 173.2x |
| collect | pytest collect | 6666 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 334 ms | 26.1x |
| full run | pytest | 8700 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10160 ms | 0.9x |

## macos-latest / py3.13

- python: `3.13.16`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 197.4x |
| collect | pytest collect | 4854 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 399 ms | 16.9x |
| full run | pytest | 6735 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7749 ms | 0.9x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 227.3x |
| collect | pytest collect | 6219 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 315 ms | 37.0x |
| full run | pytest | 11632 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13570 ms | 0.9x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 178.3x |
| collect | pytest collect | 3185 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 199 ms | 21.3x |
| full run | pytest | 4234 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7246 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.17`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 221.4x |
| collect | pytest collect | 5588 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 362 ms | 20.7x |
| full run | pytest | 7484 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12562 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 239.3x |
| collect | pytest collect | 3566 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 302 ms | 16.8x |
| full run | pytest | 5061 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8539 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 185.0x |
| collect | pytest collect | 6327 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 530 ms | 16.5x |
| full run | pytest | 8770 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14767 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.8`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 232.2x |
| collect | pytest collect | 6805 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 598 ms | 15.1x |
| full run | pytest | 9031 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14836 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 225.7x |
| collect | pytest collect | 4554 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 277 ms | 22.4x |
| full run | pytest | 6217 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10806 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 166.9x |
| collect | pytest collect | 5703 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 470 ms | 18.6x |
| full run | pytest | 8720 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13720 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 150.4x |
| collect | pytest collect | 4287 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 350 ms | 18.2x |
| full run | pytest | 6378 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10133 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 161.3x |
| collect | pytest collect | 6154 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 492 ms | 19.0x |
| full run | pytest | 9356 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14232 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.16`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 143.8x |
| collect | pytest collect | 5468 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 583 ms | 14.6x |
| full run | pytest | 8511 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13227 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 157.8x |
| collect | pytest collect | 6076 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 522 ms | 18.8x |
| full run | pytest | 9808 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14683 ms | 0.7x |
