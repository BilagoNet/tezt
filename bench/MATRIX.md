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
| collect | tezt collect | 53 ms | 117.5x |
| collect | pytest collect | 6211 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 307 ms | 22.6x |
| full run | pytest | 6927 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9771 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 45 ms | 118.0x |
| collect | pytest collect | 5317 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 316 ms | 19.0x |
| full run | pytest | 6005 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12627 ms | 0.5x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 319.2x |
| collect | pytest collect | 6493 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 360 ms | 20.0x |
| full run | pytest | 7188 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8721 ms | 0.8x |

## macos-latest / py3.13

- python: `3.13.15`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 17 ms | 253.5x |
| collect | pytest collect | 4308 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 294 ms | 19.6x |
| full run | pytest | 5776 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7049 ms | 0.8x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 177.1x |
| collect | pytest collect | 4928 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 324 ms | 21.9x |
| full run | pytest | 7099 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9457 ms | 0.8x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 174.2x |
| collect | pytest collect | 4651 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 285 ms | 21.7x |
| full run | pytest | 6200 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10252 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.17`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 233.8x |
| collect | pytest collect | 6677 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 382 ms | 23.1x |
| full run | pytest | 8853 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13614 ms | 0.7x |

## ubuntu-latest / py3.12

- python: `3.12.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 216.7x |
| collect | pytest collect | 5969 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 497 ms | 16.7x |
| full run | pytest | 8311 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13913 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.16`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 233.2x |
| collect | pytest collect | 5246 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 426 ms | 16.1x |
| full run | pytest | 6846 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11951 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.8`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 231.5x |
| collect | pytest collect | 6331 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 590 ms | 14.4x |
| full run | pytest | 8471 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14468 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 222.8x |
| collect | pytest collect | 5881 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 348 ms | 23.0x |
| full run | pytest | 8026 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15581 ms | 0.5x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 48 ms | 120.5x |
| collect | pytest collect | 5824 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 509 ms | 18.7x |
| full run | pytest | 9545 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14280 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 174.3x |
| collect | pytest collect | 3950 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 317 ms | 18.3x |
| full run | pytest | 5804 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8701 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 130.7x |
| collect | pytest collect | 5645 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 485 ms | 18.1x |
| full run | pytest | 8759 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13473 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 120.4x |
| collect | pytest collect | 5575 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 630 ms | 14.0x |
| full run | pytest | 8786 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14578 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 150.9x |
| collect | pytest collect | 3518 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 265 ms | 20.1x |
| full run | pytest | 5328 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8855 ms | 0.6x |
