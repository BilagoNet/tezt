# Contributing to tezt

Thanks for your interest! tezt is alpha software, so issues, bug reports, and small focused PRs are all welcome.

## Dev setup

You'll need:

- Rust 1.85+ (`rustup` recommended)
- Python 3.8+

Build:

```sh
cargo build
```

## Running the tests

Run the Python worker protocol self-test:

```sh
python3 python/test_worker_protocol.py
```

Run the Rust test suite:

```sh
cargo test
```

Run benchmarks (optional, used for the README numbers):

```sh
cd bench
./run.sh            # generates suites under bench/.suites/ and writes bench/results.json
```

A change is good to go when `cargo test` and the worker protocol self-test both pass.

## Code layout

| Path | What it is |
| --- | --- |
| `src/main.rs` | Entry point; wires CLI → collect → run → report |
| `src/cli.rs` | Argument parsing and flag definitions |
| `src/collect.rs` | Rust AST-based test discovery (no Python imports) |
| `src/kexpr.rs` | `-k` expression parser and matcher |
| `src/runner.rs` | Scheduler + persistent Python worker pool (JSON-lines protocol) |
| `src/report.rs` | Terminal output, failure rendering, JSON report |
| `python/tezt_worker.py` | The Python side of the worker: receives test ids, runs them, streams results |
| `testdata/` | Small Python suites used as fixtures by the Rust tests |
| `tests/` | Rust integration tests |
| `bench/` | Benchmark harness |

## PR guidelines

- Keep PRs small and single-purpose; split unrelated changes.
- Add or update a test for any behavior change — `testdata/` suites for runner behavior, Rust unit tests for parsing/matching logic.
- Run `cargo fmt` and `cargo clippy` before pushing.
- pytest compatibility questions are decided by "what does pytest do?" — when in doubt, match pytest's observable behavior and link to its docs in the PR description.
- For larger features (anything on the [roadmap](README.md#roadmap)), open an issue first so we can agree on the approach.
