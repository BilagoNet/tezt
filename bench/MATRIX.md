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
| collect | tezt collect | 24 ms | 241.4x |
| collect | pytest collect | 5807 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 448 ms | 20.2x |
| full run | pytest | 9040 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11264 ms | 0.8x |

## macos-latest / py3.11

- python: `3.11.9`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 37 ms | 123.0x |
| collect | pytest collect | 4511 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 242 ms | 26.3x |
| full run | pytest | 6364 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10674 ms | 0.6x |

## macos-latest / py3.12

- python: `3.12.10`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 15 ms | 237.5x |
| collect | pytest collect | 3679 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 217 ms | 21.6x |
| full run | pytest | 4687 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 6565 ms | 0.7x |

## macos-latest / py3.13

- python: `3.13.14`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 30 ms | 140.1x |
| collect | pytest collect | 4267 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 294 ms | 25.4x |
| full run | pytest | 7490 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7702 ms | 1.0x |

## macos-latest / py3.9

- python: `3.9.13`
- platform: `Darwin 25.4.0 (arm64)`
- cpu cores: 3

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 14 ms | 293.8x |
| collect | pytest collect | 4138 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 216 ms | 23.9x |
| full run | pytest | 5161 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 7387 ms | 0.7x |

## ubuntu-latest / py3.10

- python: `3.10.20`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 24 ms | 227.8x |
| collect | pytest collect | 5419 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 327 ms | 22.2x |
| full run | pytest | 7260 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12729 ms | 0.6x |

## ubuntu-latest / py3.11

- python: `3.11.15`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 28 ms | 188.2x |
| collect | pytest collect | 5214 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 357 ms | 20.4x |
| full run | pytest | 7287 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 11974 ms | 0.6x |

## ubuntu-latest / py3.12

- python: `3.12.13`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 21 ms | 219.4x |
| collect | pytest collect | 4621 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 381 ms | 16.0x |
| full run | pytest | 6087 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 10273 ms | 0.6x |

## ubuntu-latest / py3.13

- python: `3.13.14`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 26 ms | 218.9x |
| collect | pytest collect | 5782 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 502 ms | 16.1x |
| full run | pytest | 8065 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13674 ms | 0.6x |

## ubuntu-latest / py3.14

- python: `3.14.6`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 27 ms | 225.6x |
| collect | pytest collect | 6118 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 587 ms | 14.1x |
| full run | pytest | 8266 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 14540 ms | 0.6x |

## ubuntu-latest / py3.9

- python: `3.9.25`
- platform: `Linux 6.17.0-1020-azure (x86_64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 33 ms | 178.7x |
| collect | pytest collect | 5915 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 346 ms | 23.6x |
| full run | pytest | 8159 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13290 ms | 0.6x |

## windows-latest / py3.10

- python: `3.10.11`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 43 ms | 150.5x |
| collect | pytest collect | 6489 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 557 ms | 18.4x |
| full run | pytest | 10225 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 15162 ms | 0.7x |

## windows-latest / py3.11

- python: `3.11.9`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 149.4x |
| collect | pytest collect | 6331 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 471 ms | 18.3x |
| full run | pytest | 8625 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12964 ms | 0.7x |

## windows-latest / py3.12

- python: `3.12.10`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 41 ms | 150.7x |
| collect | pytest collect | 6207 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 611 ms | 15.0x |
| full run | pytest | 9154 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13907 ms | 0.7x |

## windows-latest / py3.13

- python: `3.13.14`
- platform: `Windows 2025Server (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 42 ms | 129.2x |
| collect | pytest collect | 5386 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 585 ms | 14.2x |
| full run | pytest | 8323 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 12676 ms | 0.7x |

## windows-latest / py3.9

- python: `3.9.13`
- platform: `Windows 10 (AMD64)`
- cpu cores: 4

**4000 tests** (median of 5 runs, jobs 4):

| phase | runner | median | speedup vs pytest |
|---|---|--:|--:|
| collect | tezt collect | 36 ms | 153.3x |
| collect | pytest collect | 5504 ms | 1.0x (baseline) |
| full run | tezt -j 4 | 483 ms | 19.2x |
| full run | pytest | 9260 ms | 1.0x (baseline) |
| full run | pytest -n 4 (xdist) | 13787 ms | 0.7x |
