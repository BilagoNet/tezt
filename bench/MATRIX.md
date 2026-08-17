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
| collect | tezt collect | 20 ms | 174.4x |
| collect | pytest collect | 3446 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 225 ms | 21.4x |
| full run | pytest | 4806 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6387 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 112.3x |
| collect | pytest collect | 3941 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 362 ms | 15.3x |
| full run | pytest | 5528 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7247 ms | 0.8x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 205.6x |
| collect | pytest collect | 5077 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 231 ms | 24.7x |
| full run | pytest | 5688 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6852 ms | 0.8x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 88.1x |
| collect | pytest collect | 3204 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 201 ms | 18.6x |
| full run | pytest | 3726 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5471 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.5.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 183.1x |
| collect | pytest collect | 4523 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 374 ms | 17.9x |
| full run | pytest | 6710 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8987 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 212.3x |
| collect | pytest collect | 5758 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 364 ms | 21.7x |
| full run | pytest | 7917 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12994 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 204.9x |
| collect | pytest collect | 5956 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 345 ms | 22.2x |
| full run | pytest | 7672 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12016 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 38 ms | 179.9x |
| collect | pytest collect | 6829 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 528 ms | 18.0x |
| full run | pytest | 9502 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14857 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.15`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 235.0x |
| collect | pytest collect | 6183 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 516 ms | 16.5x |
| full run | pytest | 8503 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14355 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.7`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 192.4x |
| collect | pytest collect | 6245 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 599 ms | 13.9x |
| full run | pytest | 8345 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14729 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1022-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 221.2x |
| collect | pytest collect | 6171 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 377 ms | 23.7x |
| full run | pytest | 8934 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14787 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 160.1x |
| collect | pytest collect | 5643 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 467 ms | 18.3x |
| full run | pytest | 8553 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14457 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 129.7x |
| collect | pytest collect | 5463 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 483 ms | 17.9x |
| full run | pytest | 8654 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13458 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 145.8x |
| collect | pytest collect | 5797 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 488 ms | 18.4x |
| full run | pytest | 8962 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13460 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.15`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 50 ms | 126.0x |
| collect | pytest collect | 6357 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 641 ms | 15.0x |
| full run | pytest | 9587 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14870 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 60 ms | 110.6x |
| collect | pytest collect | 6588 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 494 ms | 19.5x |
| full run | pytest | 9604 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14075 ms | 0.7x |
