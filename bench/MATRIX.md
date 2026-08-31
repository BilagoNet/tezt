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
| collect | tezt collect | 52 ms | 110.8x |
| collect | pytest collect | 5757 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 429 ms | 17.8x |
| full run | pytest | 7617 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10064 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 244.0x |
| collect | pytest collect | 5754 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 412 ms | 22.1x |
| full run | pytest | 9087 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10904 ms | 0.8x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 17 ms | 391.7x |
| collect | pytest collect | 6638 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 454 ms | 18.7x |
| full run | pytest | 8482 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8369 ms | 1.0x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 166.4x |
| collect | pytest collect | 4954 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 424 ms | 15.6x |
| full run | pytest | 6604 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8632 ms | 0.8x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 185.8x |
| collect | pytest collect | 5917 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 303 ms | 26.3x |
| full run | pytest | 7979 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10982 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 184.1x |
| collect | pytest collect | 5926 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 368 ms | 21.7x |
| full run | pytest | 7996 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13426 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 170.2x |
| collect | pytest collect | 4029 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 254 ms | 20.3x |
| full run | pytest | 5157 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8787 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 208.7x |
| collect | pytest collect | 6281 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 473 ms | 16.6x |
| full run | pytest | 7873 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12998 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 221.7x |
| collect | pytest collect | 5248 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 450 ms | 15.4x |
| full run | pytest | 6925 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12976 ms | 0.5x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 232.5x |
| collect | pytest collect | 6101 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 581 ms | 14.2x |
| full run | pytest | 8264 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14082 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 189.1x |
| collect | pytest collect | 4350 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 259 ms | 22.7x |
| full run | pytest | 5877 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10360 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 160.0x |
| collect | pytest collect | 5736 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 471 ms | 18.4x |
| full run | pytest | 8671 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13730 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 142.4x |
| collect | pytest collect | 5650 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 499 ms | 17.8x |
| full run | pytest | 8884 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13685 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 153.2x |
| collect | pytest collect | 5533 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 580 ms | 15.1x |
| full run | pytest | 8779 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13843 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 134.6x |
| collect | pytest collect | 5424 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 611 ms | 14.3x |
| full run | pytest | 8736 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13546 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 163.5x |
| collect | pytest collect | 4802 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 417 ms | 17.8x |
| full run | pytest | 7420 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11769 ms | 0.6x |
