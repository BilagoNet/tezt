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

// ---------------------------------------------------------------------------
// --durations
// ---------------------------------------------------------------------------

#[test]
fn durations_lists_slowest_tests() {
    tezt("parametrize")
        .args(["--durations", "3"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("slowest"));
}

// ---------------------------------------------------------------------------
// The collection cache is transparent: a warm run matches a cold run.
// ---------------------------------------------------------------------------

#[test]
fn collection_cache_is_transparent_across_runs() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_cached.py"),
        "def test_one():\n    assert True\n\ndef test_two():\n    assert 1 + 1 == 2\n",
    )
    .expect("write suite");

    // Two consecutive cached runs (cache enabled by default) must agree, and the
    // second one must have a cache directory to read from.
    for _ in 0..2 {
        let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
        cmd.current_dir(dir.path())
            .env("TEZT_PYTHON", test_python())
            .arg("--color")
            .arg("never")
            .arg(".");
        cmd.assert()
            .code(0)
            .stdout(predicate::str::contains("2 passed"));
    }
    assert!(
        dir.path().join(".tezt_cache").is_dir(),
        "a warm run should have populated .tezt_cache"
    );
}

// ---------------------------------------------------------------------------
// -m mark expressions (testdata/marks)
// ---------------------------------------------------------------------------

#[test]
fn marks_suite_all_pass_without_filter() {
    tezt("marks")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("4 passed"));
}

#[test]
fn m_filter_selects_by_single_mark() {
    // `slow` is on test_slow_one and test_slow_and_net.
    tezt("marks")
        .arg("-m")
        .arg("slow")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed"))
        .stdout(predicate::str::contains("2 deselected"));
}

#[test]
fn m_filter_boolean_and() {
    // Only test_slow_and_net carries both marks.
    tezt("marks")
        .arg("-m")
        .arg("slow and net")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"))
        .stdout(predicate::str::contains("3 deselected"));
}

#[test]
fn m_filter_not_excludes_marked() {
    // not slow => test_net_only + test_unmarked.
    tezt("marks")
        .arg("-m")
        .arg("not slow")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed"))
        .stdout(predicate::str::contains("2 deselected"));
}

#[test]
fn m_filter_collect_only_lists_only_selected() {
    tezt("marks")
        .arg("-m")
        .arg("net")
        .arg("--collect-only")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("test_slow_and_net"))
        .stdout(predicate::str::contains("test_net_only"))
        .stdout(predicate::str::contains("test_unmarked").not())
        .stdout(predicate::str::contains("2 deselected"));
}

// ---------------------------------------------------------------------------
// Rich assertion diffs (operator-aware) — exercised end-to-end through the
// real worker, so this also proves the Rust<->Python wiring carries the
// enriched message into the failure report.
// ---------------------------------------------------------------------------

#[test]
fn assertion_diff_shows_both_operands_and_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_diff.py"),
        "def test_list_diff():\n    assert [1, 2, 3] == [1, 2, 4]\n",
    )
    .expect("write test");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(dir.path());
    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("left"))
        .stdout(predicate::str::contains("right"))
        .stdout(predicate::str::contains("index 2"));
}

// ---------------------------------------------------------------------------
// --lf / --ff (last-failed / failed-first). These are stateful: each writes
// the failing-test record into the run's .tezt_cache, so they use an isolated
// temp working directory and run the binary twice.
// ---------------------------------------------------------------------------

#[test]
fn last_failed_reruns_only_previous_failures() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_a.py"),
        "def test_ok():\n    assert True\n\ndef test_bad():\n    assert False\n",
    )
    .expect("write a");
    std::fs::write(
        dir.path().join("test_b.py"),
        "def test_ok2():\n    assert True\n",
    )
    .expect("write b");

    // Run 1: full run records the one failure.
    let mut first = Command::cargo_bin("tezt").expect("tezt binary should build");
    first
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".");
    first
        .assert()
        .code(1)
        .stdout(predicate::str::contains("2 passed"))
        .stdout(predicate::str::contains("1 failed"));

    // Run 2: --lf collects only the previously-failed test.
    let mut second = Command::cargo_bin("tezt").expect("tezt binary should build");
    second
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--lf")
        .arg(".");
    second
        .assert()
        .code(1)
        .stdout(predicate::str::contains("collected 1 tests"))
        .stdout(predicate::str::contains("1 failed"));
}

#[test]
fn last_failed_with_no_history_runs_all() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_ok.py"),
        "def test_one():\n    assert True\n\ndef test_two():\n    assert True\n",
    )
    .expect("write");

    // No prior run => no record => --lf must fall back to running everything.
    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--lf")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed"));
}

#[test]
fn failed_first_schedules_failures_before_the_rest() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Alphabetically test_afail < test_zlast, but failed-first ordering is by
    // previous failure, not name, so the failing file must run first.
    std::fs::write(
        dir.path().join("test_zlast.py"),
        "def test_z():\n    assert True\n",
    )
    .expect("write z");
    std::fs::write(
        dir.path().join("test_afail.py"),
        "def test_fails():\n    assert False\n",
    )
    .expect("write a");

    // Run 1 records the failure.
    let mut first = Command::cargo_bin("tezt").expect("tezt binary should build");
    first
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".");
    first.assert().code(1);

    // Run 2: --ff with a single worker => sequential output; the failing test
    // must appear before the passing one, and nothing is deselected.
    let out = Command::cargo_bin("tezt")
        .expect("tezt binary should build")
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("-j")
        .arg("1")
        .arg("-v")
        .arg("--ff")
        .arg(".")
        .output()
        .expect("run tezt --ff");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let fail_pos = stdout
        .find("test_fails")
        .expect("failing test in -v output");
    let pass_pos = stdout.find("test_z").expect("passing test in -v output");
    assert!(
        fail_pos < pass_pos,
        "--ff should run the previously-failed test first:\n{stdout}"
    );
    assert!(
        stdout.contains("collected 2 tests"),
        "--ff must not deselect anything:\n{stdout}"
    );
}

// ---------------------------------------------------------------------------
// Plugin hooks (conftest pytest_* hooks, run worker-side)
// ---------------------------------------------------------------------------

#[test]
fn plugin_setup_hook_can_skip_a_test() {
    let dir = tempfile::tempdir().expect("tempdir");
    // A conftest `pytest_runtest_setup` hook that skips one specific test.
    std::fs::write(
        dir.path().join("conftest.py"),
        "import tezt\n\n\ndef pytest_runtest_setup(item):\n    if item.name == \"test_skip_me\":\n        tezt.skip(\"skipped by hook\")\n",
    )
    .expect("write conftest");
    std::fs::write(
        dir.path().join("test_h.py"),
        "def test_runs():\n    assert True\n\n\ndef test_skip_me():\n    assert True\n",
    )
    .expect("write tests");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"))
        .stdout(predicate::str::contains("1 skipped"));
}

// ---------------------------------------------------------------------------
// Coverage (--cov). Skipped unless `coverage` is importable by the test
// interpreter; CI installs it so the matrix exercises this for real.
// ---------------------------------------------------------------------------

/// True if `import coverage` succeeds under the test interpreter.
fn coverage_available() -> bool {
    StdCommand::new(test_python())
        .args(["-c", "import coverage"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn coverage_reports_a_term_table() {
    if !coverage_available() {
        eprintln!("skipping coverage_reports_a_term_table: coverage not importable");
        return;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("calc.py"),
        "def add(a, b):\n    return a + b\n\n\ndef unused(a, b):\n    return a - b\n",
    )
    .expect("write src");
    std::fs::write(
        dir.path().join("test_calc.py"),
        "import calc\n\n\ndef test_add():\n    assert calc.add(2, 3) == 5\n",
    )
    .expect("write test");

    // Run from inside the project dir so `import calc` resolves (rootdir is cwd).
    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--cov")
        .arg("--cov-source")
        .arg(".")
        .arg("--cov-report")
        .arg("term-missing")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"))
        // The coverage table: header + the measured source file + a total.
        .stdout(predicate::str::contains("coverage:"))
        .stdout(predicate::str::contains("calc.py"))
        .stdout(predicate::str::contains("TOTAL"));
}

#[test]
fn coverage_html_report_is_written() {
    if !coverage_available() {
        eprintln!("skipping coverage_html_report_is_written: coverage not importable");
        return;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("m.py"), "def f():\n    return 1\n").expect("write src");
    std::fs::write(
        dir.path().join("test_m.py"),
        "import m\n\n\ndef test_f():\n    assert m.f() == 1\n",
    )
    .expect("write test");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--cov-source")
        .arg(".")
        .arg("--cov-report")
        .arg("html")
        .arg(".");
    cmd.assert().code(0);
    assert!(
        dir.path().join("htmlcov").join("index.html").is_file(),
        "--cov-report html should write htmlcov/index.html"
    );
}

// ---------------------------------------------------------------------------
// pyproject.toml [tool.tezt] config: addopts + testpaths
// ---------------------------------------------------------------------------

#[test]
fn config_addopts_and_testpaths_are_applied() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join("sub")).expect("mkdir sub");
    std::fs::write(
        dir.path().join("sub").join("test_x.py"),
        "def test_in_sub():\n    assert True\n",
    )
    .expect("write sub test");
    // A test OUTSIDE testpaths must not be collected when no path is given.
    std::fs::write(
        dir.path().join("test_ignored.py"),
        "def test_ignored():\n    assert True\n",
    )
    .expect("write ignored test");
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.tezt]\naddopts = [\"-v\"]\ntestpaths = [\"sub\"]\n",
    )
    .expect("write pyproject");

    // No path and no flags on the command line: `testpaths` supplies the path,
    // `addopts = ["-v"]` supplies verbose mode.
    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never");
    cmd.assert()
        .code(0)
        // testpaths scoped collection to sub/ (test_ignored never runs).
        .stdout(predicate::str::contains("1 passed"))
        .stdout(predicate::str::contains("test_ignored").not())
        // addopts -v turned on the per-test line.
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("test_in_sub"));
}

// ---------------------------------------------------------------------------
// --junitxml
// ---------------------------------------------------------------------------

#[test]
fn junitxml_report_is_written_and_well_shaped() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_j.py"),
        "def test_ok():\n    assert True\n\n\ndef test_bad():\n    assert 1 == 2\n",
    )
    .expect("write test");

    let xml_path = dir.path().join("out.xml");
    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--junitxml")
        .arg(&xml_path)
        .arg(".");
    cmd.assert().code(1);

    let xml = std::fs::read_to_string(&xml_path).expect("junit xml should exist");
    assert!(
        xml.contains("<testsuite"),
        "has a testsuite element:\n{xml}"
    );
    assert!(xml.contains("tests=\"2\""), "two tests:\n{xml}");
    assert!(xml.contains("failures=\"1\""), "one failure:\n{xml}");
    assert!(xml.contains("<failure"), "has a failure element:\n{xml}");
}

// ---------------------------------------------------------------------------
// capsys + approx, end-to-end through the real worker
// ---------------------------------------------------------------------------

#[test]
fn capsys_and_approx_fixtures_work() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_conv.py"),
        "import tezt\n\n\ndef test_capsys(capsys):\n    print(\"hi\")\n    out, err = capsys.readouterr()\n    assert out == \"hi\\n\"\n\n\ndef test_approx():\n    assert 0.1 + 0.2 == tezt.approx(0.3)\n",
    )
    .expect("write test");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("2 passed"));
}

// ---------------------------------------------------------------------------
// autouse + parametrized fixtures, end-to-end
// ---------------------------------------------------------------------------

#[test]
fn parametrized_fixture_expands_into_cases() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_pf.py"),
        "import tezt\n\n\n@tezt.fixture(params=[1, 2, 3])\ndef n(request):\n    return request.param\n\n\ndef test_n(n):\n    assert n in (1, 2, 3)\n",
    )
    .expect("write test");

    // One collected test function, three param values => three cases run.
    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("3 passed"));
}

#[test]
fn autouse_fixture_runs_without_being_requested() {
    let dir = tempfile::tempdir().expect("tempdir");
    // An autouse fixture sets an env var the (non-requesting) test then reads.
    std::fs::write(
        dir.path().join("conftest.py"),
        "import os\nimport tezt\n\n\n@tezt.fixture(autouse=True)\ndef _setenv():\n    os.environ[\"TEZT_AUTOUSE\"] = \"1\"\n",
    )
    .expect("write conftest");
    std::fs::write(
        dir.path().join("test_au.py"),
        "import os\n\n\ndef test_sees_autouse():\n    assert os.environ.get(\"TEZT_AUTOUSE\") == \"1\"\n",
    )
    .expect("write test");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("1 passed"));
}

// ---------------------------------------------------------------------------
// --markers / --fixtures (query modes, no test run)
// ---------------------------------------------------------------------------

#[test]
fn markers_lists_builtins_and_registered() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.tezt]\nmarkers = [\"slow: long-running tests\"]\n",
    )
    .expect("write pyproject");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--markers");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("skip"))
        .stdout(predicate::str::contains("xfail"))
        // registered marker shown verbatim, with its description
        .stdout(predicate::str::contains("slow: long-running tests"));
}

#[test]
fn fixtures_lists_conftest_and_builtins() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("conftest.py"),
        "import tezt\n\n\n@tezt.fixture\ndef my_fixture():\n    \"a demo fixture\"\n    return 1\n",
    )
    .expect("write conftest");
    std::fs::write(
        dir.path().join("test_f.py"),
        "def test_uses(my_fixture):\n    assert my_fixture == 1\n",
    )
    .expect("write test");

    let mut cmd = Command::cargo_bin("tezt").expect("tezt binary should build");
    cmd.current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--fixtures")
        .arg(".");
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("my_fixture"))
        // builtins are always listed
        .stdout(predicate::str::contains("tmp_path"))
        .stdout(predicate::str::contains("capsys"));
}

// ---------------------------------------------------------------------------
// --tb styles
// ---------------------------------------------------------------------------

#[test]
fn tb_no_omits_the_traceback() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_tb.py"),
        "def test_boom():\n    raise ValueError(\"nope\")\n",
    )
    .expect("write test");

    // Default keeps a full traceback; --tb=no drops it (the failure header and
    // message still print, but not the "Traceback (most recent call last)" dump).
    Command::cargo_bin("tezt")
        .expect("tezt binary should build")
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg(".")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Traceback"));

    Command::cargo_bin("tezt")
        .expect("tezt binary should build")
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--tb")
        .arg("no")
        .arg(".")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Traceback").not())
        .stdout(predicate::str::contains("test_boom"));
}

// ---------------------------------------------------------------------------
// --stepwise: stop at the first failure, resume there next run
// ---------------------------------------------------------------------------

#[test]
fn stepwise_stops_then_resumes() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("test_sw.py"),
        "def test_a():\n    assert True\n\n\ndef test_b():\n    assert False\n\n\ndef test_c():\n    assert True\n",
    )
    .expect("write test");

    // First --stepwise run: a passes, b fails, run stops there (maxfail=1).
    Command::cargo_bin("tezt")
        .expect("tezt binary should build")
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--stepwise")
        .arg(".")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("1 failed"));

    // Second --stepwise run resumes from the failing test_b.
    Command::cargo_bin("tezt")
        .expect("tezt binary should build")
        .current_dir(dir.path())
        .env("TEZT_PYTHON", test_python())
        .arg("--color")
        .arg("never")
        .arg("--stepwise")
        .arg(".")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("resuming from"));
}
