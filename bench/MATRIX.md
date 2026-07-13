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
| collect | tezt collect | 27 ms | 138.8x |
| collect | pytest collect | 3693 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 343 ms | 13.5x |
| full run | pytest | 4635 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8934 ms | 0.5x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 177.5x |
| collect | pytest collect | 2801 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 185 ms | 19.6x |
| full run | pytest | 3631 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 5748 ms | 0.6x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 151.6x |
| collect | pytest collect | 3275 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 248 ms | 17.0x |
| full run | pytest | 4210 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6390 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 66 ms | 91.7x |
| collect | pytest collect | 6011 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 373 ms | 19.2x |
| full run | pytest | 7148 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10349 ms | 0.7x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 14 ms | 366.1x |
| collect | pytest collect | 5286 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 234 ms | 37.0x |
| full run | pytest | 8649 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9467 ms | 0.9x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 20 ms | 218.0x |
| collect | pytest collect | 4420 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 271 ms | 21.9x |
| full run | pytest | 5952 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10484 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 165.3x |
| collect | pytest collect | 5222 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 335 ms | 21.9x |
| full run | pytest | 7349 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12052 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 29 ms | 215.5x |
| collect | pytest collect | 6281 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 477 ms | 17.1x |
| full run | pytest | 8162 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13377 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 168.9x |
| collect | pytest collect | 5854 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 506 ms | 16.1x |
| full run | pytest | 8153 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14391 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 247.0x |
| collect | pytest collect | 5612 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 531 ms | 13.6x |
| full run | pytest | 7196 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13556 ms | 0.5x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1018-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 33 ms | 180.3x |
| collect | pytest collect | 5868 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 346 ms | 23.4x |
| full run | pytest | 8076 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13174 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 40 ms | 152.7x |
| collect | pytest collect | 6079 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 511 ms | 19.4x |
| full run | pytest | 9926 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15017 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 148.3x |
| collect | pytest collect | 5730 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 481 ms | 19.2x |
| full run | pytest | 9248 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 18142 ms | 0.5x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 161.1x |
| collect | pytest collect | 4336 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 520 ms | 12.2x |
| full run | pytest | 6355 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10567 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 149.6x |
| collect | pytest collect | 5412 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 585 ms | 13.6x |
| full run | pytest | 7939 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13501 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 41 ms | 139.0x |
| collect | pytest collect | 5688 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 490 ms | 18.7x |
| full run | pytest | 9137 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13776 ms | 0.7x |
