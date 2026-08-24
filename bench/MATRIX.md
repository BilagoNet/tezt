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
| collect | tezt collect | 53 ms | 117.4x |
| collect | pytest collect | 6222 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 249 ms | 22.1x |
| full run | pytest | 5517 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7365 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 200.7x |
| collect | pytest collect | 4842 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 350 ms | 20.4x |
| full run | pytest | 7126 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10774 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 237.0x |
| collect | pytest collect | 3900 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 223 ms | 23.0x |
| full run | pytest | 5125 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6981 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 214.9x |
| collect | pytest collect | 5279 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 472 ms | 14.6x |
| full run | pytest | 6892 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9349 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 265.1x |
| collect | pytest collect | 4079 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 228 ms | 22.7x |
| full run | pytest | 5188 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7742 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.21`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 206.3x |
| collect | pytest collect | 6091 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 380 ms | 21.8x |
| full run | pytest | 8275 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13722 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 179.0x |
| collect | pytest collect | 5179 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 355 ms | 20.3x |
| full run | pytest | 7208 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11974 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.14`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 230.0x |
| collect | pytest collect | 6195 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 506 ms | 16.5x |
| full run | pytest | 8327 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13907 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 206.6x |
| collect | pytest collect | 5897 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 487 ms | 15.7x |
| full run | pytest | 7637 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13212 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 233.9x |
| collect | pytest collect | 6221 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 594 ms | 14.0x |
| full run | pytest | 8341 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14383 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 211.6x |
| collect | pytest collect | 5939 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 373 ms | 23.0x |
| full run | pytest | 8562 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14415 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 160.7x |
| collect | pytest collect | 6019 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 526 ms | 18.4x |
| full run | pytest | 9671 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 17245 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 141.2x |
| collect | pytest collect | 4317 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 346 ms | 19.1x |
| full run | pytest | 6613 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10288 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 162.9x |
| collect | pytest collect | 6322 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 460 ms | 18.3x |
| full run | pytest | 8397 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14212 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 143.7x |
| collect | pytest collect | 5377 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 603 ms | 14.1x |
| full run | pytest | 8485 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13037 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 142.0x |
| collect | pytest collect | 5714 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 500 ms | 18.8x |
| full run | pytest | 9381 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13640 ms | 0.7x |
