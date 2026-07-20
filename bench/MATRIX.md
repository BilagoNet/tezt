# tezt benchmark matrix

Produced by the [Benchmarks workflow](../.github/workflows/bench.yml) across the
OS x Python matrix. Every cell runs the same generated suite under tezt and pytest,
with tezt's collection cache disabled so it's a fair parse-vs-import comparison.
Times are wall-clock medians; `speedup` is versus single-process pytest in the
same phase.

## macos-latest / py3.10

- python: `3.10.11`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 128.4x |
| collect | pytest collect | 3945 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 283 ms | 19.4x |
| full run | pytest | 5488 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9711 ms | 0.6x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 64 ms | 60.8x |
| collect | pytest collect | 3910 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 310 ms | 14.2x |
| full run | pytest | 4395 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8144 ms | 0.5x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 198.2x |
| collect | pytest collect | 5267 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 361 ms | 15.5x |
| full run | pytest | 5606 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9938 ms | 0.6x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 18 ms | 185.7x |
| collect | pytest collect | 3262 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 200 ms | 22.9x |
| full run | pytest | 4580 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6005 ms | 0.8x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 221.0x |
| collect | pytest collect | 7917 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 393 ms | 26.8x |
| full run | pytest | 10546 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12021 ms | 0.9x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 221.9x |
| collect | pytest collect | 5626 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 349 ms | 21.4x |
| full run | pytest | 7479 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14200 ms | 0.5x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 177.7x |
| collect | pytest collect | 5011 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 352 ms | 20.2x |
| full run | pytest | 7093 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11722 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 190.1x |
| collect | pytest collect | 5851 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 466 ms | 16.6x |
| full run | pytest | 7729 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12485 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 220.7x |
| collect | pytest collect | 5869 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 507 ms | 15.8x |
| full run | pytest | 8025 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13835 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 209.9x |
| collect | pytest collect | 4699 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 430 ms | 14.1x |
| full run | pytest | 6057 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9990 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 221.1x |
| collect | pytest collect | 5847 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 369 ms | 22.8x |
| full run | pytest | 8432 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14449 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 147.5x |
| collect | pytest collect | 4485 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 357 ms | 19.0x |
| full run | pytest | 6783 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10311 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 150.1x |
| collect | pytest collect | 5463 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 483 ms | 17.9x |
| full run | pytest | 8657 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13150 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 143.4x |
| collect | pytest collect | 3518 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 352 ms | 12.9x |
| full run | pytest | 4533 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7872 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 144.9x |
| collect | pytest collect | 5634 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 607 ms | 14.7x |
| full run | pytest | 8911 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13285 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 151.4x |
| collect | pytest collect | 5600 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 490 ms | 18.7x |
| full run | pytest | 9183 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13515 ms | 0.7x |
