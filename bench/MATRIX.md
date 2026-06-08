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
| collect | tezt collect | 22 ms | 223.8x |
| collect | pytest collect | 4913 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 312 ms | 22.4x |
| full run | pytest | 6983 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10856 ms | 0.6x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 198.7x |
| collect | pytest collect | 4793 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 280 ms | 25.0x |
| full run | pytest | 7004 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8474 ms | 0.8x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 185.5x |
| collect | pytest collect | 4695 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 293 ms | 21.9x |
| full run | pytest | 6402 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7893 ms | 0.8x |

## macos-latest / py3.13

- python: `3.13.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 211.6x |
| collect | pytest collect | 3192 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 216 ms | 18.4x |
| full run | pytest | 3988 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6143 ms | 0.6x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 24.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 34 ms | 128.7x |
| collect | pytest collect | 4367 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 354 ms | 16.5x |
| full run | pytest | 5840 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9600 ms | 0.6x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 231.2x |
| collect | pytest collect | 5928 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 364 ms | 22.3x |
| full run | pytest | 8129 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13540 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 184.6x |
| collect | pytest collect | 5180 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 336 ms | 21.4x |
| full run | pytest | 7209 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12075 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 231.0x |
| collect | pytest collect | 5910 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 449 ms | 17.7x |
| full run | pytest | 7964 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12881 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.13`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 249.2x |
| collect | pytest collect | 5817 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 480 ms | 16.5x |
| full run | pytest | 7924 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13603 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.5`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 247.7x |
| collect | pytest collect | 6091 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 569 ms | 14.5x |
| full run | pytest | 8243 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14368 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1015-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 25 ms | 240.3x |
| collect | pytest collect | 5959 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 358 ms | 24.0x |
| full run | pytest | 8580 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14381 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 47 ms | 140.8x |
| collect | pytest collect | 6638 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 456 ms | 20.0x |
| full run | pytest | 9112 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14399 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 44 ms | 124.8x |
| collect | pytest collect | 5539 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 419 ms | 19.6x |
| full run | pytest | 8207 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12958 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 174.6x |
| collect | pytest collect | 6769 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 612 ms | 16.1x |
| full run | pytest | 9868 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14193 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.13`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 35 ms | 148.0x |
| collect | pytest collect | 5111 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 529 ms | 14.4x |
| full run | pytest | 7646 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11880 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 122.8x |
| collect | pytest collect | 5608 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 488 ms | 19.1x |
| full run | pytest | 9345 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14143 ms | 0.7x |
