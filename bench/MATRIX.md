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
| collect | tezt collect | 16 ms | 253.7x |
| collect | pytest collect | 4063 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 220 ms | 24.7x |
| full run | pytest | 5422 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9133 ms | 0.6x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 245.5x |
| collect | pytest collect | 3722 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 199 ms | 23.7x |
| full run | pytest | 4728 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6693 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 97.2x |
| collect | pytest collect | 4172 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 195 ms | 26.6x |
| full run | pytest | 5196 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8206 ms | 0.6x |

## macos-latest / py3.13

- python: `3.13.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 206.7x |
| collect | pytest collect | 3306 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 182 ms | 22.8x |
| full run | pytest | 4140 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6277 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 173.2x |
| collect | pytest collect | 6370 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 328 ms | 26.1x |
| full run | pytest | 8568 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12571 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 228.7x |
| collect | pytest collect | 6865 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 349 ms | 27.5x |
| full run | pytest | 9597 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15813 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 221.7x |
| collect | pytest collect | 5127 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 292 ms | 24.6x |
| full run | pytest | 7171 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11963 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 254.6x |
| collect | pytest collect | 6395 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 453 ms | 19.4x |
| full run | pytest | 8785 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14825 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.13`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 244.2x |
| collect | pytest collect | 6159 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 453 ms | 18.6x |
| full run | pytest | 8406 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14462 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.5`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 238.5x |
| collect | pytest collect | 5993 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 496 ms | 15.8x |
| full run | pytest | 7840 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13011 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 240.3x |
| collect | pytest collect | 6245 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 328 ms | 27.6x |
| full run | pytest | 9027 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15372 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 162.9x |
| collect | pytest collect | 5968 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 414 ms | 22.8x |
| full run | pytest | 9439 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14652 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 157.9x |
| collect | pytest collect | 5791 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 461 ms | 18.6x |
| full run | pytest | 8596 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12785 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 183.1x |
| collect | pytest collect | 6355 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 710 ms | 11.9x |
| full run | pytest | 8480 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13211 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.13`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 178.5x |
| collect | pytest collect | 6380 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 524 ms | 17.0x |
| full run | pytest | 8926 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 17357 ms | 0.5x |
