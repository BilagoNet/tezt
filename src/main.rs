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
            kexpr::KExpr::compile(expr).map_err(|e| anyhow::anyhow!("invalid -k expression: {e}"))?,
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
    let mut items: Vec<collect::TestItem> = collected_files
        .into_iter()
        .flat_map(|f| f.items)
        .collect();

    // Apply -k: keep an item if its base id or any statically-known case id
    // matches. (Parametrized cases are re-filtered on results below.)
    let mut deselected = 0usize;
    if let Some(k) = &kexpr {
        let before: usize = items.iter().map(|i| i.expected_cases()).sum();
        items.retain(|item| item.display_ids().iter().any(|id| k.matches(id)) || k.matches(&item.id));
        let after: usize = items.iter().map(|i| i.expected_cases()).sum();
        deselected = before.saturating_sub(after);
    }

    let expected_total: usize = items.iter().map(|i| i.expected_cases()).sum();
    let collect_ms = collect_start.elapsed().as_secs_f64() * 1000.0;

    // --- collect-only mode ----------------------------------------------------
    if args.collect_only {
        if !args.quiet {
            for item in &items {
                for id in item.display_ids() {
                    if kexpr.as_ref().map(|k| k.matches(&id) || k.matches(&item.id)).unwrap_or(true)
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
        return Ok(if expected_total > 0 { EXIT_OK } else { EXIT_NO_TESTS });
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
        println!("{}", style.dim(&format!("python: {}", python::label(&python))));
    }
    let cfg = runner::RunConfig {
        python,
        rootdir: rootdir.clone(),
        jobs,
        no_capture: args.no_capture,
        maxfail: args.effective_maxfail(),
    };

    let mut reporter =
        report::Reporter::new(args.use_color(), args.verbose > 0, args.quiet, expected_total);
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
