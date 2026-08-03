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
| collect | tezt collect | 16 ms | 230.0x |
| collect | pytest collect | 3636 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 310 ms | 16.7x |
| full run | pytest | 5162 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8165 ms | 0.6x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 185.4x |
| collect | pytest collect | 4966 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 305 ms | 19.3x |
| full run | pytest | 5884 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8479 ms | 0.7x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 156.7x |
| collect | pytest collect | 3966 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 266 ms | 21.6x |
| full run | pytest | 5749 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7774 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 115.6x |
| collect | pytest collect | 2782 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 181 ms | 19.4x |
| full run | pytest | 3509 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6458 ms | 0.5x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 261.9x |
| collect | pytest collect | 3897 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 229 ms | 24.7x |
| full run | pytest | 5646 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7754 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 195.8x |
| collect | pytest collect | 5938 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 372 ms | 22.3x |
| full run | pytest | 8276 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13645 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 175.7x |
| collect | pytest collect | 5185 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 359 ms | 20.0x |
| full run | pytest | 7187 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11809 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 211.5x |
| collect | pytest collect | 5914 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 501 ms | 16.6x |
| full run | pytest | 8334 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13817 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 189.0x |
| collect | pytest collect | 5860 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 507 ms | 16.0x |
| full run | pytest | 8121 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13827 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 195.1x |
| collect | pytest collect | 5966 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 551 ms | 13.9x |
| full run | pytest | 7669 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12710 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 219.5x |
| collect | pytest collect | 6155 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 380 ms | 23.1x |
| full run | pytest | 8783 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14957 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 47 ms | 128.7x |
| collect | pytest collect | 6046 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 680 ms | 15.5x |
| full run | pytest | 10546 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 18751 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 138.5x |
| collect | pytest collect | 5367 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 429 ms | 18.6x |
| full run | pytest | 7993 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12341 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 139.8x |
| collect | pytest collect | 5364 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 467 ms | 18.0x |
| full run | pytest | 8387 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12885 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 129.2x |
| collect | pytest collect | 5481 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 590 ms | 13.3x |
| full run | pytest | 7844 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12259 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 158.1x |
| collect | pytest collect | 5756 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 569 ms | 15.0x |
| full run | pytest | 8563 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14579 ms | 0.6x |
