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
| collect | tezt collect | 22 ms | 236.4x |
| collect | pytest collect | 5157 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 316 ms | 17.7x |
| full run | pytest | 5604 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9123 ms | 0.6x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 46 ms | 145.2x |
| collect | pytest collect | 6628 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 363 ms | 20.2x |
| full run | pytest | 7316 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10060 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.17`
- platform: `Darwin 25.6.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 32 ms | 201.8x |
| collect | pytest collect | 6530 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 646 ms | 12.5x |
| full run | pytest | 8067 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11122 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.22`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 17 ms | 227.3x |
| collect | pytest collect | 3785 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 216 ms | 22.0x |
| full run | pytest | 4749 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9247 ms | 0.5x |

## ubuntu-latest / py3.11

- python: `3.11.18`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 198.8x |
| collect | pytest collect | 3089 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 187 ms | 21.3x |
| full run | pytest | 3992 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7133 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.16`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 209.1x |
| collect | pytest collect | 4517 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 348 ms | 16.9x |
| full run | pytest | 5901 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10055 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.18`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 16 ms | 277.3x |
| collect | pytest collect | 4300 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 312 ms | 17.3x |
| full run | pytest | 5393 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9219 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.10`
- platform: `Linux 7.0.0-1019-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 22 ms | 206.6x |
| collect | pytest collect | 4647 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 411 ms | 13.9x |
| full run | pytest | 5724 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 9904 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 23 ms | 161.7x |
| collect | pytest collect | 3718 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 290 ms | 18.8x |
| full run | pytest | 5453 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 8688 ms | 0.6x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 31 ms | 142.5x |
| collect | pytest collect | 4399 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 338 ms | 18.1x |
| full run | pytest | 6128 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10034 ms | 0.6x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 41 ms | 167.4x |
| collect | pytest collect | 6895 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 629 ms | 15.3x |
| full run | pytest | 9626 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15145 ms | 0.6x |

## windows-latest / py3.13

- python: `3.13.17`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 162.7x |
| collect | pytest collect | 6089 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 611 ms | 14.2x |
| full run | pytest | 8640 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14184 ms | 0.6x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 39 ms | 143.6x |
| collect | pytest collect | 5579 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 500 ms | 18.6x |
| full run | pytest | 9291 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13625 ms | 0.7x |
