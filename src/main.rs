//! tezt — an extremely fast Python test runner, written in Rust.

// mimalloc: collection parses many files in parallel (rayon), so allocation
// is hot and multi-threaded — exactly where a per-thread-arena allocator beats
// the system one. Same reason uv/ruff ship a custom global allocator.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod cache;
mod cli;
mod collect;
mod kexpr;
mod python;
mod report;
mod runner;

use anyhow::Result;
use clap::Parser;
use rustc_hash::FxHashSet;
use std::path::PathBuf;
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
    let args = cli::Cli::parse();
    let style = report::Style {
        color: args.use_color(),
    };

    let rootdir = std::env::current_dir()?;
    let paths: Vec<PathBuf> = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

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

    // Combined deselection count across -k/-m/--lf, computed once from the
    // pre-selection baseline so the summary and JSON report agree.
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
    let jobs = args.jobs.unwrap_or_else(num_cpus::get).max(1);
    let python = python::resolve_python(args.python_override().as_deref(), &rootdir);
    if !args.quiet {
        println!(
            "{}",
            style.dim(&format!("python: {}", python::label(&python)))
        );
    }
    let cfg = runner::RunConfig {
        python,
        rootdir: rootdir.clone(),
        jobs,
        no_capture: args.no_capture,
        maxfail: args.effective_maxfail(),
        timeout: args.timeout.map(std::time::Duration::from_secs_f64),
        priority_files,
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

    Ok(exit_code)
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
