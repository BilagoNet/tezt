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
| collect | tezt collect | 23 ms | 199.5x |
| collect | pytest collect | 4672 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 306 ms | 23.4x |
| full run | pytest | 7157 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8910 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 182.9x |
| collect | pytest collect | 2793 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 183 ms | 19.8x |
| full run | pytest | 3640 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5708 ms | 0.6x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 133.2x |
| collect | pytest collect | 3740 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 273 ms | 16.1x |
| full run | pytest | 4408 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6170 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 19 ms | 146.5x |
| collect | pytest collect | 2811 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 177 ms | 21.0x |
| full run | pytest | 3715 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5459 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 220.0x |
| collect | pytest collect | 5159 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 397 ms | 18.5x |
| full run | pytest | 7353 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10458 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 257.8x |
| collect | pytest collect | 5947 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 385 ms | 22.1x |
| full run | pytest | 8504 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14369 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 239.0x |
| collect | pytest collect | 6074 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 368 ms | 22.9x |
| full run | pytest | 8420 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13229 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 246.3x |
| collect | pytest collect | 5803 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 496 ms | 16.5x |
| full run | pytest | 8173 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14031 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 241.2x |
| collect | pytest collect | 6047 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 492 ms | 15.7x |
| full run | pytest | 7709 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12623 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 212.0x |
| collect | pytest collect | 6672 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 569 ms | 14.6x |
| full run | pytest | 8297 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13782 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 241.3x |
| collect | pytest collect | 5937 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 362 ms | 23.6x |
| full run | pytest | 8542 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14701 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 41 ms | 146.7x |
| collect | pytest collect | 5948 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 506 ms | 19.0x |
| full run | pytest | 9595 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14927 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 157.8x |
| collect | pytest collect | 5409 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 430 ms | 18.7x |
| full run | pytest | 8021 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13188 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 131.2x |
| collect | pytest collect | 5515 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 635 ms | 13.5x |
| full run | pytest | 8594 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13681 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 48 ms | 111.0x |
| collect | pytest collect | 5374 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 596 ms | 14.1x |
| full run | pytest | 8411 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12805 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 47 ms | 118.5x |
| collect | pytest collect | 5550 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 491 ms | 18.5x |
| full run | pytest | 9100 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13707 ms | 0.7x |
