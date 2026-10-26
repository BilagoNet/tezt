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
| collect | tezt collect | 16 ms | 239.5x |
| collect | pytest collect | 3924 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 342 ms | 14.7x |
| full run | pytest | 5029 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10862 ms | 0.5x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 116.0x |
| collect | pytest collect | 5393 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 318 ms | 19.2x |
| full run | pytest | 6098 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8600 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 194.0x |
| collect | pytest collect | 6564 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 403 ms | 18.3x |
| full run | pytest | 7357 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10064 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.16`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 124.6x |
| collect | pytest collect | 5451 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 343 ms | 18.3x |
| full run | pytest | 6272 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8407 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 193.5x |
| collect | pytest collect | 6603 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 402 ms | 20.7x |
| full run | pytest | 8301 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12664 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 209.2x |
| collect | pytest collect | 6021 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 395 ms | 21.9x |
| full run | pytest | 8644 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14847 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.17`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 202.6x |
| collect | pytest collect | 5800 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 376 ms | 20.9x |
| full run | pytest | 7854 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12840 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.15`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 169.2x |
| collect | pytest collect | 6011 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 511 ms | 15.8x |
| full run | pytest | 8095 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13967 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.16`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 238.8x |
| collect | pytest collect | 4938 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 404 ms | 15.8x |
| full run | pytest | 6383 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11064 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.8`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 231.8x |
| collect | pytest collect | 5241 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 444 ms | 14.5x |
| full run | pytest | 6433 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10605 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 45 ms | 129.6x |
| collect | pytest collect | 5809 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 515 ms | 18.0x |
| full run | pytest | 9287 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14205 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 147.4x |
| collect | pytest collect | 3573 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 273 ms | 19.0x |
| full run | pytest | 5174 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8872 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 122.2x |
| collect | pytest collect | 5615 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 632 ms | 13.9x |
| full run | pytest | 8771 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13457 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.16`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 109.9x |
| collect | pytest collect | 4001 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 432 ms | 13.9x |
| full run | pytest | 5991 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9566 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 143.9x |
| collect | pytest collect | 4185 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 341 ms | 19.4x |
| full run | pytest | 6591 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10253 ms | 0.6x |
