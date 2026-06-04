//! tezt — an extremely fast Python test runner, written in Rust.

// mimalloc: collection parses many files in parallel (rayon), so allocation
// is hot and multi-threaded — exactly where a per-thread-arena allocator beats
// the system one. Same reason uv/ruff ship a custom global allocator.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod cache;
mod cli;
mod collect;
mod config;
mod kexpr;
mod python;
mod report;
mod runner;

use anyhow::{Context, Result};
use clap::Parser;
use rustc_hash::FxHashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

// Exit codes (pytest parity).
const EXIT_OK: i32 = 0;
const EXIT_TESTS_FAILED: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_NO_TESTS: i32 = 5;

fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("tezt: error: {e:#}");
            EXIT_USAGE
        }
    };
    std::process::exit(code);
}

fn run() -> Result<i32> {
    // Resolve the rootdir *before* parsing argv: the config file lives there,
    // and its `addopts` must be folded into the argument list we actually parse.
    let rootdir = std::env::current_dir()?;
    let cfg_file = config::Config::load(&rootdir);

    // addopts merge: prepend the config's default args to the user's argv, then
    // parse the combined list. The order is `[program] + addopts + user-args` so
    // that an explicitly-passed flag follows (and, for clap's last-wins value
    // options like `--color`/`--maxfail`, overrides) the config default. Boolean
    // flags can't be *un*set this way (there's no `--no-foo`), so addopts should
    // hold opt-ins (`-q`, `--tb=short`) rather than things a user must later turn
    // off — same caveat pytest's addopts carries. We splice after `argv[0]`
    // because `parse_from` expects the program name in slot 0.
    let argv: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let args = if cfg_file.addopts.is_empty() {
        cli::Cli::parse_from(&argv)
    } else {
        let prog = argv.first().cloned().unwrap_or_else(|| "tezt".into());
        let combined = std::iter::once(prog)
            .chain(cfg_file.addopts.iter().map(std::ffi::OsString::from))
            .chain(argv.into_iter().skip(1));
        cli::Cli::parse_from(combined)
    };

    let style = report::Style {
        color: args.use_color(),
    };

    // Registered markers from `[tool.tezt]`, consumed by `--markers` below (and
    // a later `--strict-markers`).
    let registered_markers: Vec<String> = cfg_file.markers;

    // Positional paths: explicit CLI args win; otherwise fall back to the
    // config's `testpaths`; otherwise the historical default of the cwd.
    let paths: Vec<PathBuf> = if !args.paths.is_empty() {
        args.paths.clone()
    } else if !cfg_file.testpaths.is_empty() {
        cfg_file.testpaths.iter().map(PathBuf::from).collect()
    } else {
        vec![PathBuf::from(".")]
    };

    // --- query modes (no test run) ------------------------------------------
    // `--markers` and `--fixtures` are introspection queries, like
    // `--collect-only`: each prints what it found and exits without running any
    // tests. They are handled here, before collection, so neither pays for it.
    if args.markers_list {
        return Ok(list_markers(&registered_markers, &style));
    }
    if args.fixtures {
        return Ok(list_fixtures(&args, &paths, &rootdir, &style));
    }

    // -k expression
    let kexpr = match &args.keyword {
        Some(expr) => Some(
            kexpr::KExpr::compile(expr)
                .map_err(|e| anyhow::anyhow!("invalid -k expression: {e}"))?,
        ),
        None => None,
    };

    // -m mark expression: same boolean engine as -k, but each term is matched
    // against an item's mark set rather than a substring of its id.
    let mexpr = match &args.markers {
        Some(e) => Some(
            kexpr::KExpr::compile(e).map_err(|e| anyhow::anyhow!("invalid -m expression: {e}"))?,
        ),
        None => None,
    };

    // --- collection ---------------------------------------------------------
    // Persistent collection cache: unchanged files skip read+parse on warm
    // runs. `--clear-cache` wipes it first (best-effort); `--no-cache` disables
    // it entirely for this run.
    if args.clear_cache {
        let _ = cache::Cache::clear(&rootdir);
    }
    let cache = if args.no_cache {
        None
    } else {
        Some(cache::Cache::new(&rootdir, true))
    };

    let collect_start = Instant::now();
    let collected_files = collect::collect(&paths, &rootdir, cache.as_ref())?;
    let mut items: Vec<collect::TestItem> =
        collected_files.into_iter().flat_map(|f| f.items).collect();

    // Baseline case count *before* any selection, so `deselected` reflects the
    // combined effect of -k, -m, and --lf (recomputed once, below).
    let before_total: usize = items.iter().map(|i| i.expected_cases()).sum();

    // Apply -k: keep an item if its base id or any statically-known case id
    // matches. (Parametrized cases are re-filtered on results below.)
    if let Some(k) = &kexpr {
        items.retain(|item| {
            item.display_ids().iter().any(|id| k.matches(id)) || k.matches(&item.id)
        });
    }

    // Apply -m: keep an item whose mark set satisfies the expression. Marks are
    // matched by exact membership (see `kexpr::KExpr::eval_with`).
    if let Some(m) = &mexpr {
        items.retain(|item| {
            let set: rustc_hash::FxHashSet<&str> = item.marks.iter().map(String::as_str).collect();
            m.eval_with(&|term: &str| set.contains(term))
        });
    }

    // Last-failed record from the previous run. Loaded once: it feeds both
    // `--lf` (retain) and `--ff` (priority ordering), and the merge-save below.
    let prev_failed = cache::load_last_failed(&rootdir);

    // --lf: re-run only what failed last time. With no history (empty record)
    // we run everything, mirroring pytest, and say so unless quiet.
    if args.last_failed {
        if prev_failed.is_empty() {
            if !args.quiet {
                println!("no previously failed tests; running all");
            }
        } else {
            items.retain(|item| item_matches_failed(item, &prev_failed));
        }
    }

    // --stepwise resume: drop everything before the recorded resume point so
    // this run picks up where the last one stopped. We operate on the already
    // -k/-m/--lf-filtered `items`, so stepwise composes with those (their
    // ordering is unchanged; stepwise only trims a prefix). If the resume id is
    // no longer present (e.g. the test was renamed or removed) we run all,
    // mirroring pytest. `--stepwise` itself forces `jobs=1`/`maxfail=1` later so
    // results arrive in this collection order and the run stops at the first
    // failure. The trimmed prefix is reflected in `deselected` below.
    let stepwise_resume: Option<String> = if args.stepwise {
        cache::load_stepwise(&rootdir)
    } else {
        None
    };
    if let Some(resume_id) = &stepwise_resume {
        if let Some(pos) = items
            .iter()
            .position(|i| item_is_resume_point(i, resume_id))
        {
            if pos > 0 {
                items.drain(..pos);
            }
            if !args.quiet {
                println!("stepwise: resuming from {resume_id}");
            }
        }
        // If not found, fall through with the full selection (run all).
    }

    // Combined deselection count across -k/-m/--lf (and any --stepwise resume
    // trim), computed once from the pre-selection baseline so the summary and
    // JSON report agree.
    let expected_total: usize = items.iter().map(|i| i.expected_cases()).sum();
    let deselected = before_total.saturating_sub(expected_total);

    // --ff: schedule files containing a previously-failed test first. Built from
    // the (already -k/-m/--lf-filtered) items so we never prioritize a file that
    // was deselected this run.
    let priority_files: FxHashSet<PathBuf> = if args.failed_first {
        items
            .iter()
            .filter(|i| item_matches_failed(i, &prev_failed))
            .map(|i| i.file.clone())
            .collect()
    } else {
        FxHashSet::default()
    };

    let collect_ms = collect_start.elapsed().as_secs_f64() * 1000.0;

    // --- collect-only mode ----------------------------------------------------
    if args.collect_only {
        if !args.quiet {
            for item in &items {
                for id in item.display_ids() {
                    if kexpr
                        .as_ref()
                        .map(|k| k.matches(&id) || k.matches(&item.id))
                        .unwrap_or(true)
                    {
                        println!("{id}");
                    }
                }
            }
            if deselected > 0 {
                println!();
                println!(
                    "collected {expected_total} tests ({deselected} deselected) in {collect_ms:.0}ms"
                );
            } else {
                println!();
                println!("collected {expected_total} tests in {collect_ms:.0}ms");
            }
        } else {
            println!("collected {expected_total} tests");
        }
        return Ok(if expected_total > 0 {
            EXIT_OK
        } else {
            EXIT_NO_TESTS
        });
    }

    if expected_total == 0 {
        println!("collected 0 tests in {collect_ms:.0}ms");
        println!("{}", style.yellow("no tests collected"));
        return Ok(EXIT_NO_TESTS);
    }

    if !args.quiet {
        println!(
            "{} {}",
            style.bold(&format!("collected {expected_total} tests")),
            style.dim(&format!("in {collect_ms:.0}ms"))
        );
    } else {
        println!("collected {expected_total} tests");
    }

    // --- execution ------------------------------------------------------------
    // `--stepwise` forces sequential, fail-fast execution: one worker so results
    // arrive in collection order (making "the first failure" well-defined and
    // the resume point stable across runs), and an effective maxfail of 1 so we
    // stop at that first failure. This overrides any `-j`/`--maxfail`/`-x` for
    // the run; without it, parallel workers could fail tests out of order and we
    // could not record a deterministic resume point.
    let jobs = if args.stepwise {
        1
    } else {
        args.jobs.unwrap_or_else(num_cpus::get).max(1)
    };
    let python = python::resolve_python(args.python_override().as_deref(), &rootdir);
    if !args.quiet {
        println!(
            "{}",
            style.dim(&format!("python: {}", python::label(&python)))
        );
    }

    // Coverage setup + pre-check. Done here, right after the interpreter is
    // resolved and before any worker spawns, so a missing `coverage` package
    // fails fast with one clear message instead of every worker erroring out.
    // The data dir is a per-run temp directory (keyed by pid like the worker
    // shim) into which each worker drops `.coverage.<pid>`; the parent combines
    // and reports them after the run and removes the dir. `cov_dir` (the Some
    // case) doubles as "coverage is on" for the post-run block below.
    let cov_dir: Option<PathBuf> = if args.cov_enabled() {
        // Fail fast if the test interpreter can't import `coverage`. We check
        // the *resolved* interpreter (the exact string handed to workers), not
        // the display label, so the error names what we'll actually run.
        let importable = std::process::Command::new(&python)
            .args(["-c", "import coverage"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !importable {
            anyhow::bail!(
                "--cov requires the 'coverage' package in the test interpreter ({python}); \
                 install it with: pip install coverage"
            );
        }
        let dir = std::env::temp_dir().join(format!("tezt-cov-{}", std::process::id()));
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create coverage data dir {}", dir.display()))?;
        Some(dir)
    } else {
        None
    };

    let cfg = runner::RunConfig {
        python,
        rootdir: rootdir.clone(),
        jobs,
        no_capture: args.no_capture,
        tb: args.tb.clone(),
        // `--stepwise` implies maxfail 1; combine with any explicit `-x` /
        // `--maxfail` by taking the smaller (most aggressive) threshold so
        // neither relaxes the other.
        maxfail: stepwise_maxfail(args.stepwise, args.effective_maxfail()),
        timeout: args.timeout.map(std::time::Duration::from_secs_f64),
        priority_files,
        cov: cov_dir.as_ref().map(|dir| runner::CovConfig {
            data_dir: dir.clone(),
            sources: args.cov_source.clone(),
            branch: args.cov_branch,
        }),
    };

    let mut reporter = report::Reporter::new(
        args.use_color(),
        args.verbose > 0,
        args.quiet,
        expected_total,
    );
    let run_out = runner::run_tests(items, &cfg, |r| {
        // Late -k filter for parametrized case ids unknown at collection.
        if let Some(k) = &kexpr {
            let base = r.id.split('[').next().unwrap_or(&r.id);
            if !(k.matches(&r.id) || k.matches(base)) {
                return;
            }
        }
        reporter.on_result(r);
    })?;
    reporter.finish_progress();

    // Re-filter the stored results the same way for accurate totals/JSON.
    let results: Vec<runner::TestResult> = run_out
        .results
        .into_iter()
        .filter(|r| {
            kexpr
                .as_ref()
                .map(|k| {
                    let base = r.id.split('[').next().unwrap_or(&r.id);
                    k.matches(&r.id) || k.matches(base)
                })
                .unwrap_or(true)
        })
        .collect();

    // Merge-update the last-failed record, pytest-style: start from the prior
    // set, drop ids that passed this run, and add ids that failed. Starting from
    // `prev_failed` (rather than rebuilding from scratch) preserves failures in
    // files we did not run this time — e.g. when a path filter, -k, -m, or --lf
    // narrowed the selection — so a later bare run still re-runs them under
    // --lf. Best-effort; never fails the run.
    let mut merged = prev_failed.clone();
    for r in &results {
        if r.outcome.is_bad() {
            merged.insert(r.id.clone());
        } else {
            merged.remove(&r.id);
        }
    }
    cache::save_last_failed(&rootdir, &merged);

    // --stepwise: record (or clear) the resume point. With `jobs=1`/`maxfail=1`
    // forced above, results arrive in collection order and the run stops at the
    // first bad result — so the first `is_bad()` id is exactly where to resume
    // next time. If nothing was bad the slice ran clean to the end, so we clear
    // the resume point (a clean suite resumes from the start). Best-effort;
    // never fails the run. A Ctrl-C exits via the signal handler before reaching
    // here, so an interrupted run never spuriously clears the resume point.
    if args.stepwise {
        let first_bad = results.iter().find(|r| r.outcome.is_bad());
        cache::save_stepwise(&rootdir, first_bad.map(|r| r.id.as_str()));
    }

    // Recompute counts from the filtered set (reporter counted live already;
    // keep them consistent).
    let mut counts = report::Counts::default();
    for r in &results {
        counts.add(r.outcome);
    }
    reporter.counts = counts;

    reporter.print_failures(&results);
    if let Some(n) = args.durations {
        reporter.print_durations(&results, n);
    }

    let exit_code = if counts.any_bad() || run_out.stopped_early {
        EXIT_TESTS_FAILED
    } else if counts.total() == 0 {
        EXIT_NO_TESTS
    } else {
        EXIT_OK
    };

    reporter.print_summary(run_out.wall_time_s, run_out.stopped_early, deselected);

    // Coverage combine + report. Runs regardless of pass/fail (coverage is
    // observability, not correctness) but only when something actually
    // executed: the `expected_total == 0` early return above already prevents
    // an empty run from reaching here, and `--collect-only` returns earlier
    // still, so neither path measures coverage. Best-effort throughout — a
    // tooling failure warns on stderr but never changes tezt's exit code.
    if let Some(dir) = &cov_dir {
        report_coverage(&cfg.python, &rootdir, dir, &args.cov_reports(), &style);
    }

    if let Some(json_path) = &args.json {
        report::write_json_report(
            json_path,
            expected_total,
            &counts,
            deselected,
            run_out.wall_time_s,
            exit_code,
            run_out.stopped_early,
            &results,
        )?;
    }

    if let Some(junit_path) = &args.junitxml {
        report::write_junit_xml(
            junit_path,
            expected_total,
            &counts,
            run_out.wall_time_s,
            &results,
        )?;
    }

    Ok(exit_code)
}

/// Combine the per-worker coverage data files and print the requested reports.
///
/// This is the parent-side tail of the coverage feature: each worker wrote a
/// `<data_dir>/.coverage.<pid>` parallel-data file during the run; here we ask
/// the test interpreter's own `coverage` module to merge them and emit reports.
/// We deliberately shell out to `<python> -m coverage` rather than reimplement
/// any of it — the data format is coverage.py's private contract, and using the
/// same interpreter guarantees a version match with whatever the workers wrote.
///
/// Two invariants make the file paths line up:
///   * `COVERAGE_FILE=<data_dir>/.coverage` — `combine` reads the siblings
///     `<data_dir>/.coverage.*` and writes the merged db to `COVERAGE_FILE`;
///     every later `report`/`html`/`xml` then reads that same merged db.
///   * `current_dir(rootdir)` — source paths in the data files are relative to
///     the rootdir (where workers ran), and `html`/`xml` outputs land in the
///     project, not in the throwaway data dir.
///
/// Everything is best-effort. Coverage is observability: if the tooling is
/// missing data, errors, or a report kind fails, we warn on stderr and move on
/// without touching tezt's exit code. Returns `()` for that reason.
fn report_coverage(
    python: &str,
    rootdir: &Path,
    data_dir: &Path,
    reports: &[String],
    style: &report::Style,
) {
    // The merged database lives inside the data dir too, so removing the dir at
    // the end cleans up both the per-worker files and the combined db. (html/xml
    // reports are written under the rootdir and so survive.)
    let coverage_file = data_dir.join(".coverage");

    // Build a `<python> -m coverage <sub-args...>` command with the env + cwd
    // that make file paths resolve. Stdout/stderr are inherited so the user sees
    // coverage's own output (the report table, html/xml progress) live.
    let coverage_cmd = |sub: &[&str]| -> std::process::Command {
        let mut cmd = std::process::Command::new(python);
        cmd.arg("-m")
            .arg("coverage")
            .args(sub)
            .env("COVERAGE_FILE", &coverage_file)
            .current_dir(rootdir);
        cmd
    };

    // `combine` merges `<data_dir>/.coverage.*` into `COVERAGE_FILE`. Passing the
    // directory lets coverage discover the parallel files itself. A non-zero exit
    // here usually means "no data to combine" — e.g. every test errored before
    // coverage started, or the workers never wrote a file — which is not a tezt
    // failure, so we note it and skip the (now pointless) reports.
    match coverage_cmd(&["combine", "-q", &data_dir.to_string_lossy()]).status() {
        Ok(status) if status.success() => {}
        Ok(_) => {
            eprintln!("tezt: coverage: no coverage data to report");
            cleanup_cov_dir(data_dir);
            return;
        }
        Err(e) => {
            eprintln!("tezt: coverage: failed to run `coverage combine`: {e}");
            cleanup_cov_dir(data_dir);
            return;
        }
    }

    // Emit each requested report kind. `term`/`term-missing` go through the same
    // `report` subcommand (the latter adds the missing-line column); `html`/`xml`
    // write files under the rootdir and we point the user at them. Unknown kinds
    // can't reach here — clap's `value_parser` rejects them at parse time.
    for kind in reports {
        let result = match kind.as_str() {
            "term" => {
                cov_term_header(style);
                coverage_cmd(&["report"]).status()
            }
            "term-missing" => {
                cov_term_header(style);
                coverage_cmd(&["report", "--show-missing"]).status()
            }
            "html" => coverage_cmd(&["html", "-d", "htmlcov"]).status(),
            "xml" => coverage_cmd(&["xml", "-o", "coverage.xml"]).status(),
            // Defensive: clap restricts the values, so this is unreachable in
            // practice. Skip rather than panic if that ever changes.
            other => {
                eprintln!("tezt: coverage: unknown report kind {other:?}");
                continue;
            }
        };
        match result {
            Ok(status) if status.success() => match kind.as_str() {
                "html" => println!("wrote HTML coverage report to htmlcov/index.html"),
                "xml" => println!("wrote XML coverage report to coverage.xml"),
                _ => {}
            },
            Ok(status) => {
                eprintln!("tezt: coverage: `coverage {kind}` exited with {status}");
            }
            Err(e) => {
                eprintln!("tezt: coverage: failed to run `coverage {kind}`: {e}");
            }
        }
    }

    cleanup_cov_dir(data_dir);
}

/// Print the blank line + bold `coverage:` header shown just above a terminal
/// coverage table. Kept tiny and separate so both `term` and `term-missing`
/// render an identical lead-in.
fn cov_term_header(style: &report::Style) {
    println!();
    println!("{}", style.bold("coverage:"));
}

/// Best-effort removal of the per-run coverage data dir (its `.coverage.*` files
/// and the merged `.coverage` db). Never errors out: a leftover temp dir is
/// harmless, and html/xml reports live under the rootdir, not here.
fn cleanup_cov_dir(data_dir: &Path) {
    let _ = std::fs::remove_dir_all(data_dir);
}

/// Does this collected item correspond to any previously-failed test id?
/// Failed ids are case ids like `f.py::test[1]`; items are pre-expansion. An
/// item matches if a failed id's base (`split('[')`) equals the item id, or a
/// failed id belongs under the item (covers class items and the `*` dynamic
/// file item whose id is the file path).
fn item_matches_failed(item: &collect::TestItem, failed: &FxHashSet<String>) -> bool {
    failed.iter().any(|fid| {
        let base = fid.split('[').next().unwrap_or(fid);
        base == item.id
            || base.starts_with(&format!("{}::", item.id))
            || fid.starts_with(&format!("{}[", item.id))
    })
}

/// Is this collected item the `--stepwise` resume point `resume_id`?
///
/// `resume_id` is a single recorded case id (e.g. `f.py::test[1]` or
/// `f.py::test`); items are pre-parametrize-expansion. We reuse the
/// [`item_matches_failed`] matching so a parametrized resume id maps back to its
/// (unexpanded) collection item — the resume point is "this item", regardless of
/// which case first failed.
fn item_is_resume_point(item: &collect::TestItem, resume_id: &str) -> bool {
    let base = resume_id.split('[').next().unwrap_or(resume_id);
    base == item.id
        || base.starts_with(&format!("{}::", item.id))
        || resume_id.starts_with(&format!("{}[", item.id))
}

/// Effective maxfail for the run, folding in `--stepwise`.
///
/// `--stepwise` implies a maxfail of 1 (stop at the first failure). When also
/// given `-x`/`--maxfail=N`, we take the smaller (more aggressive) of the two so
/// neither relaxes the other; stepwise's 1 is already the minimum, so the result
/// is `Some(1)` whenever stepwise is on. Without stepwise the explicit value
/// (if any) is returned unchanged.
fn stepwise_maxfail(stepwise: bool, explicit: Option<usize>) -> Option<usize> {
    if !stepwise {
        return explicit;
    }
    match explicit {
        Some(n) => Some(n.min(1)),
        None => Some(1),
    }
}

/// `--markers`: print the built-in markers (with descriptions) followed by the
/// project-registered markers from `[tool.tezt] markers`, then exit 0.
///
/// No worker is needed — the built-ins are fixed and the registered list comes
/// straight from config. Each registered entry is printed verbatim under its
/// own sub-header so they're visually distinct from the built-ins. (Config
/// currently keeps only the marker *name*, dropping any `: description`, so a
/// configured `slow: long-running` shows as `slow`; restoring the description
/// is a `config` change out of scope here.)
fn list_markers(registered: &[String], style: &report::Style) -> i32 {
    // (name shown in bold, description) for each built-in marker tezt honors.
    const BUILTINS: &[(&str, &str)] = &[
        ("skip", "skip the test unconditionally"),
        (
            "skipif(condition, reason=...)",
            "skip the test if the condition is true",
        ),
        (
            "xfail(reason=...)",
            "expect the test to fail (report xfailed/xpassed)",
        ),
        (
            "parametrize(argnames, argvalues)",
            "run the test once per set of argument values",
        ),
    ];

    println!("{}", style.bold("built-in markers:"));
    for (name, desc) in BUILTINS {
        println!("  {}", style.bold(name));
        println!("    {}", style.dim(desc));
    }

    if registered.is_empty() {
        println!();
        println!(
            "{}",
            style.dim("no registered markers (add them under [tool.tezt] markers)")
        );
    } else {
        println!();
        println!("{}", style.bold("registered markers:"));
        for entry in registered {
            println!("  {entry}");
        }
    }
    EXIT_OK
}

/// `--fixtures`: list the fixtures available to the collected `paths` and exit.
///
/// A query mode (no test run): resolve the interpreter exactly as the run path
/// does, ask one worker to enumerate fixtures (see
/// [`runner::list_fixtures`]), and print them grouped one per line as
/// `name [scope]`, with the docstring and defining location dimmed beneath. The
/// worker has already sorted the list. Returns [`EXIT_OK`] on success; on any
/// worker error (failed spawn, no fixtures event, or a `fatal`) it prints a
/// friendly message to stderr and returns [`EXIT_USAGE`].
fn list_fixtures(args: &cli::Cli, paths: &[PathBuf], rootdir: &Path, style: &report::Style) -> i32 {
    let python = python::resolve_python(args.python_override().as_deref(), rootdir);
    match runner::list_fixtures(&python, rootdir, paths) {
        Ok(fixtures) => {
            println!("{}", style.bold("available fixtures:"));
            for f in &fixtures {
                println!("  {} {}", f.name, style.dim(&format!("[{}]", f.scope)));
                if !f.doc.is_empty() {
                    // Show only the first line of the docstring, like pytest's
                    // one-line fixture summary; the rest is available in source.
                    if let Some(first) = f.doc.lines().next() {
                        let first = first.trim();
                        if !first.is_empty() {
                            println!("    {}", style.dim(first));
                        }
                    }
                }
                if !f.location.is_empty() {
                    println!("    {}", style.dim(&f.location));
                }
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("tezt: could not list fixtures: {e:#}");
            EXIT_USAGE
        }
    }
}
