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
| collect | tezt collect | 24 ms | 207.2x |
| collect | pytest collect | 5064 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 350 ms | 18.9x |
| full run | pytest | 6614 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9140 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 240.1x |
| collect | pytest collect | 3631 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 202 ms | 22.6x |
| full run | pytest | 4565 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6452 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.17`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 234.5x |
| collect | pytest collect | 5305 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 352 ms | 16.7x |
| full run | pytest | 5892 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8568 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 149.0x |
| collect | pytest collect | 6191 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 305 ms | 26.2x |
| full run | pytest | 8000 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11416 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 204.2x |
| collect | pytest collect | 3672 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 212 ms | 21.5x |
| full run | pytest | 4541 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8070 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.17`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 213.7x |
| collect | pytest collect | 3776 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 219 ms | 22.1x |
| full run | pytest | 4842 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8092 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.15`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 204.2x |
| collect | pytest collect | 4903 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 366 ms | 16.3x |
| full run | pytest | 5978 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10430 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.17`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 231.5x |
| collect | pytest collect | 3409 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 287 ms | 14.9x |
| full run | pytest | 4269 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7860 ms | 0.5x |

## ubuntu-latest / py3.14

- python: `3.14.9`
- platform: `Linux 7.0.0-1017-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 225.1x |
| collect | pytest collect | 4001 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 340 ms | 14.6x |
| full run | pytest | 4970 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7865 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 173.4x |
| collect | pytest collect | 7541 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 540 ms | 19.6x |
| full run | pytest | 10561 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15776 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 47 ms | 118.5x |
| collect | pytest collect | 5621 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 486 ms | 17.8x |
| full run | pytest | 8670 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13919 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 142.0x |
| collect | pytest collect | 4441 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 434 ms | 13.9x |
| full run | pytest | 6043 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9929 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.17`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 159.6x |
| collect | pytest collect | 3564 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 355 ms | 12.9x |
| full run | pytest | 4569 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8236 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 48 ms | 123.8x |
| collect | pytest collect | 5927 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 479 ms | 18.1x |
| full run | pytest | 8666 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14067 ms | 0.6x |
