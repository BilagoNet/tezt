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
| collect | tezt collect | 63 ms | 87.4x |
| collect | pytest collect | 5502 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 352 ms | 17.0x |
| full run | pytest | 5986 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9004 ms | 0.7x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 226.5x |
| collect | pytest collect | 4584 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 331 ms | 16.2x |
| full run | pytest | 5353 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6944 ms | 0.8x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 33 ms | 188.3x |
| collect | pytest collect | 6275 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 348 ms | 23.6x |
| full run | pytest | 8217 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8113 ms | 1.0x |

## macos-latest / py3.13

- python: `3.13.17`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 132.7x |
| collect | pytest collect | 5139 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 457 ms | 15.1x |
| full run | pytest | 6895 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8346 ms | 0.8x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 334.2x |
| collect | pytest collect | 6934 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 416 ms | 22.1x |
| full run | pytest | 9202 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13731 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 7.0.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 196.7x |
| collect | pytest collect | 3024 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 192 ms | 21.0x |
| full run | pytest | 4020 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7357 ms | 0.5x |

## ubuntu-latest / py3.11

- python: `3.11.18`
- platform: `Linux 7.0.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 174.6x |
| collect | pytest collect | 5451 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 343 ms | 20.9x |
| full run | pytest | 7168 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12327 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.16`
- platform: `Linux 7.0.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 210.4x |
| collect | pytest collect | 5853 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 472 ms | 16.0x |
| full run | pytest | 7556 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13448 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.18`
- platform: `Linux 7.0.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 173.2x |
| collect | pytest collect | 6116 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 486 ms | 16.1x |
| full run | pytest | 7827 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13788 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.10`
- platform: `Linux 7.0.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 238.6x |
| collect | pytest collect | 5313 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 468 ms | 14.3x |
| full run | pytest | 6687 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11914 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 135.9x |
| collect | pytest collect | 3239 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 266 ms | 20.0x |
| full run | pytest | 5327 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8908 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 161.0x |
| collect | pytest collect | 5942 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 481 ms | 17.7x |
| full run | pytest | 8487 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13333 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 128.1x |
| collect | pytest collect | 4338 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 426 ms | 14.3x |
| full run | pytest | 6088 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10004 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.17`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 127.9x |
| collect | pytest collect | 3101 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 327 ms | 12.9x |
| full run | pytest | 4221 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7491 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 148.8x |
| collect | pytest collect | 5540 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 489 ms | 18.6x |
| full run | pytest | 9064 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13662 ms | 0.7x |
