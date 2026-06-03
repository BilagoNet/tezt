//! Integration tests for the `tezt` binary, driven by the suites under
//! `testdata/`. Expected outcome counts are documented in
//! `testdata/EXPECTED.md`; every assertion here mirrors that table.
//!
//! Conventions:
//! - Summary lines contain substrings like "3 passed", "1 failed",
//!   "3 skipped", "1 xfailed", "1 xpassed", "1 error(s)". We only assert
//!   the categories that are nonzero for a given suite, and we use
//!   "1 error" (no plural suffix) so both "1 error" and "1 errors" match.
//! - `--color never` is passed everywhere so ANSI escapes can never break
//!   substring matching.
//! - Exit codes: 0 = all good, 1 = failures/errors, 5 = nothing collected.

use std::path::PathBuf;
use std::process::Command as StdCommand;

use assert_cmd::Command;
use predicates::prelude::*;

/// Absolute path to a suite directory under `testdata/`.
fn suite(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(name)
}

/// A `tezt` command pre-configured with `--color never` and the given suite.
///
/// Pins the worker interpreter to `python3` via `TEZT_PYTHON` so these tests
/// exercise tezt's run/report logic deterministically, independent of the
/// machine's interpreter-discovery surface (an ambient `$VIRTUAL_ENV`, a stray
/// project `.venv`, etc.). Discovery itself is covered by unit tests in
/// `src/python.rs`. `pytest_compat` needs the same interpreter that
/// `pytest_available()` probes — which is exactly `python3` on PATH.
fn tezt(suite_name: &str) -> Command {
    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.env("TEZT_PYTHON", test_python());
    cmd.arg("--color").arg("never").arg(suite(suite_name));
    cmd
}

/// Interpreter the worker should run under during tests. Defaults to `python3`
/// (correct on macOS/Linux dev machines). CI sets `TEZT_TEST_PYTHON` to the
/// exact `setup-python` interpreter — essential on Windows, which has no
/// `python3` executable, only `python.exe`.
fn test_python() -> String {
    std::env::var("TEZT_TEST_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

/// True if `import pytest` succeeds under the test interpreter.
fn pytest_available() -> bool {
    StdCommand::new(test_python())
        .args(["-c", "import pytest"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Per-suite outcome counts + exit codes (see testdata/EXPECTED.md)
// ---------------------------------------------------------------------------

#[test]
fn basic_suite_counts_and_exit_code() {
    tezt("basic")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("3 passed"))
        .stdout(predicate::str::contains("1 failed"));
}

#[test]
fn classes_suite_all_pass() {
    tezt("classes")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("6 passed"));
}

#[test]
fn fixtures_suite_all_pass() {
    tezt("fixtures")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("7 passed"));
}

#[test]
fn parametrize_suite_expands_to_16_cases() {
    tezt("parametrize")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("16 passed"));
}

#[test]
fn skips_suite_skip_xfail_xpass_counts() {
    tezt("skips")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"))
        .stdout(predicate::str::contains("3 skipped"))
        .stdout(predicate::str::contains("1 xfailed"))
        .stdout(predicate::str::contains("1 xpassed"));
}

#[test]
fn asyncio_suite_counts() {
    tezt("asyncio_suite")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("2 passed"))
        .stdout(predicate::str::contains("1 failed"));
}

#[test]
fn failures_suite_failures_and_collection_error() {
    tezt("failures")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("2 failed"))
        // matches both "1 error" and "1 errors"
        .stdout(predicate::str::contains("1 error"));
}

#[test]
fn kfilter_suite_all_pass() {
    tezt("kfilter")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("3 passed"));
}

// ---------------------------------------------------------------------------
// --collect-only
// ---------------------------------------------------------------------------

#[test]
fn collect_only_reports_counts() {
    // Parametrized cases expand at collection time: 5 + 3 + 6 + 2 = 16.
    tezt("parametrize")
        .arg("--collect-only")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("collected 16 tests"));

    tezt("basic")
        .arg("--collect-only")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("collected 4 tests"));
}

// ---------------------------------------------------------------------------
// -k filtering
// ---------------------------------------------------------------------------

#[test]
fn k_filter_single_substring() {
    tezt("kfilter")
        .args(["-k", "alpha"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"))
        .stdout(predicate::str::contains("3 passed").not());
}

#[test]
fn k_filter_boolean_expressions_and_class_name() {
    // "alpha or beta" keeps two of the three tests
    tezt("kfilter")
        .args(["-k", "alpha or beta"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed"));

    // "not alpha" keeps test_beta and TestGamma::test_delta
    tezt("kfilter")
        .args(["-k", "not alpha"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed"));

    // class-name substring matches the method inside TestGamma
    tezt("kfilter")
        .args(["-k", "Gamma"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"));
}

// ---------------------------------------------------------------------------
// -x / --maxfail early stop
// ---------------------------------------------------------------------------

#[test]
fn x_flag_stops_before_reporting_all_failures() {
    // Full run reports 2 failed + 1 error (asserted elsewhere). With -x the
    // run halts at the first failure/error, so "2 failed" must never appear.
    tezt("failures")
        .arg("-x")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("2 failed").not());
}

// ---------------------------------------------------------------------------
// --json report
// ---------------------------------------------------------------------------

#[test]
fn json_report_is_valid_and_well_shaped() {
    let json_path = std::env::temp_dir().join("tezt_it_basic_report.json");
    let _ = std::fs::remove_file(&json_path);

    tezt("basic").arg("--json").arg(&json_path).assert().code(1);

    let raw = std::fs::read_to_string(&json_path).expect("json report file should exist");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("report should be valid JSON");

    assert!(
        value.get("summary").map(|s| s.is_object()).unwrap_or(false),
        "top-level \"summary\" must be a JSON object, got: {value}"
    );
    let tests = value
        .get("tests")
        .and_then(|t| t.as_array())
        .expect("top-level \"tests\" must be a JSON array");
    assert!(
        !tests.is_empty(),
        "\"tests\" array should not be empty for basic suite"
    );

    let _ = std::fs::remove_file(&json_path);
}

// ---------------------------------------------------------------------------
// -j parallelism: identical counts regardless of worker count
// ---------------------------------------------------------------------------

#[test]
fn jobs_1_and_4_produce_identical_counts() {
    for jobs in ["1", "4"] {
        tezt("parametrize")
            .args(["-j", jobs])
            .assert()
            .code(0)
            .stdout(predicate::str::contains("16 passed"));

        tezt("skips")
            .args(["-j", jobs])
            .assert()
            .code(0)
            .stdout(predicate::str::contains("1 passed"))
            .stdout(predicate::str::contains("3 skipped"))
            .stdout(predicate::str::contains("1 xfailed"))
            .stdout(predicate::str::contains("1 xpassed"));
    }
}

// ---------------------------------------------------------------------------
// Exit code 5: nothing collected
// ---------------------------------------------------------------------------

#[test]
fn empty_dir_exits_5() {
    tezt("empty").assert().code(5);
}

#[test]
fn k_filter_matching_nothing_exits_5() {
    tezt("kfilter")
        .args(["-k", "zzz_no_such_test"])
        .assert()
        .code(5);
}

// ---------------------------------------------------------------------------
// pytest-style compatibility suite (only when pytest is importable)
// ---------------------------------------------------------------------------

#[test]
fn pytest_compat_suite_counts() {
    if !pytest_available() {
        eprintln!("skipping pytest_compat_suite_counts: pytest not importable via python3");
        return;
    }
    tezt("pytest_compat")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("10 passed"))
        .stdout(predicate::str::contains("1 failed"))
        .stdout(predicate::str::contains("4 skipped"))
        .stdout(predicate::str::contains("1 xfailed"))
        .stdout(predicate::str::contains("1 xpassed"));
}

#[test]
fn pytest_compat_collects_17_tests() {
    if !pytest_available() {
        eprintln!("skipping pytest_compat_collects_17_tests: pytest not importable via python3");
        return;
    }
    tezt("pytest_compat")
        .arg("--collect-only")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("collected 17 tests"));
}

// ---------------------------------------------------------------------------
// --timeout: a hung test is killed and reported, and the run doesn't hang
// ---------------------------------------------------------------------------

#[test]
fn timeout_kills_a_hung_test() {
    // A test that would otherwise block for 30s is killed at the 2s budget. The
    // run finishes promptly (well under the 30s sleep), reports the test as a
    // timeout error, and exits 1. This also exercises the cross-platform kill
    // path — process group on unix, TerminateProcess on Windows — in CI.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_hang.py"),
        "import time\n\n\ndef test_sleeps_forever():\n    time.sleep(30)\n",
    )
    .expect("write hung test");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--no-cache")
        .arg("--timeout")
        .arg("2")
        .arg(dir.path());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("timed out"));
}
