//! Command-line interface definition.

use clap::{ColorChoice, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tezt",
    version,
    about = "An extremely fast Python test runner, written in Rust.",
    long_about = None,
    color = ColorChoice::Auto
)]
pub struct Cli {
    /// Files or directories to collect tests from (default: current directory)
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,

    /// Only run tests matching the given expression
    /// (substring match, supports `and`, `or`, `not`, parentheses)
    #[arg(short = 'k', value_name = "EXPR")]
    pub keyword: Option<String>,

    /// Only run tests matching the given mark expression
    /// (e.g. `slow`, `not slow`, `slow and not network`).
    #[arg(short = 'm', value_name = "MARKEXPR")]
    pub markers: Option<String>,

    /// Re-run only the tests that failed during the last run. If no tests failed
    /// last time (or there is no record), all tests run.
    #[arg(long = "lf", visible_alias = "last-failed")]
    pub last_failed: bool,

    /// Run the tests that failed during the last run first, then everything else.
    #[arg(long = "ff", visible_alias = "failed-first")]
    pub failed_first: bool,

    /// Verbose output: one line per test
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet output: only the summary
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Stop after the first failure (alias for --maxfail=1)
    #[arg(short = 'x', long = "exitfirst")]
    pub exitfirst: bool,

    /// Stop after N failures or errors
    #[arg(long = "maxfail", value_name = "N")]
    pub maxfail: Option<usize>,

    /// Stop at the first failure and resume from it on the next run
    /// (pytest's `--stepwise`). Forces sequential, fail-fast execution
    /// (`--jobs=1`, `--maxfail=1`) so ordering and the resume point are
    /// deterministic.
    #[arg(long = "stepwise", visible_alias = "sw")]
    pub stepwise: bool,

    /// Traceback style passed to each worker: one of `auto` (the default,
    /// nothing is forwarded), `long`, `short`, `line`, or `no`.
    #[arg(long = "tb", value_name = "STYLE", default_value = "auto",
          value_parser = ["auto", "long", "short", "line", "no"])]
    pub tb: String,

    /// List the fixtures available to the collected tests and exit (does not
    /// run anything). Like `--collect-only`, this is a query mode.
    #[arg(long = "fixtures")]
    pub fixtures: bool,

    /// List the built-in and project-registered markers and exit (does not
    /// run anything). Named `markers_list` to avoid clashing with `-m`.
    #[arg(long = "markers")]
    pub markers_list: bool,

    /// Number of parallel Python workers (default: number of CPUs)
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<usize>,

    /// Collect tests without running them
    #[arg(long = "collect-only")]
    pub collect_only: bool,

    /// Write a machine-readable JSON report to the given path
    #[arg(long = "json", value_name = "PATH")]
    pub json: Option<PathBuf>,

    /// Write a JUnit XML report to the given path (for CI test reporters)
    #[arg(long = "junitxml", value_name = "PATH")]
    pub junitxml: Option<PathBuf>,

    /// Do not capture test stdout/stderr (stream to terminal via worker stderr)
    #[arg(short = 's', long = "no-capture")]
    pub no_capture: bool,

    /// Show the N slowest tests after the run
    #[arg(long = "durations", value_name = "N")]
    pub durations: Option<usize>,

    /// Kill a test (and report it as an error) if it runs longer than this many
    /// seconds. Applies per test; off by default.
    #[arg(long = "timeout", value_name = "SECONDS")]
    pub timeout: Option<f64>,

    /// Control colored output
    #[arg(long = "color", value_name = "WHEN", default_value = "auto",
          value_parser = ["auto", "always", "never"])]
    pub color: String,

    /// Python executable used to run workers (also: TEZT_PYTHON env var)
    #[arg(long = "python", value_name = "EXE")]
    pub python: Option<String>,

    /// Disable the persistent collection cache for this run
    #[arg(long = "no-cache")]
    pub no_cache: bool,

    /// Delete the collection cache (.tezt_cache) before running
    #[arg(long = "clear-cache")]
    pub clear_cache: bool,

    /// Measure code coverage during the run (requires the `coverage` package in
    /// the test interpreter). Coverage is combined and reported after the run.
    #[arg(long = "cov")]
    pub cov: bool,

    /// Limit coverage to these sources (a package name or directory; repeatable).
    /// Defaults to the rootdir. Implies `--cov`.
    #[arg(long = "cov-source", value_name = "SRC")]
    pub cov_source: Vec<String>,

    /// Coverage report format(s): `term`, `term-missing`, `html`, `xml`
    /// (repeatable). Defaults to `term-missing`. Implies `--cov`.
    #[arg(long = "cov-report", value_name = "KIND",
          value_parser = ["term", "term-missing", "html", "xml"])]
    pub cov_report: Vec<String>,

    /// Also measure branch coverage. Implies `--cov`.
    #[arg(long = "cov-branch")]
    pub cov_branch: bool,
}

impl Cli {
    /// Effective fail-fast threshold: `-x` wins, then `--maxfail`.
    pub fn effective_maxfail(&self) -> Option<usize> {
        if self.exitfirst {
            Some(1)
        } else {
            self.maxfail
        }
    }

    /// Whether to colorize output, resolving "auto" against a TTY check.
    pub fn use_color(&self) -> bool {
        match self.color.as_str() {
            "always" => true,
            "never" => false,
            _ => {
                if std::env::var_os("NO_COLOR").is_some() {
                    false
                } else {
                    use std::io::IsTerminal;
                    std::io::stdout().is_terminal()
                }
            }
        }
    }

    /// Explicit Python override from `--python` or `$TEZT_PYTHON`, if any.
    /// Full discovery (active venv, `$CONDA_PREFIX`, project `.venv`, `PATH`)
    /// happens in [`crate::python::resolve_python`] when this returns `None`.
    pub fn python_override(&self) -> Option<String> {
        if let Some(p) = &self.python {
            return Some(p.clone());
        }
        match std::env::var("TEZT_PYTHON") {
            Ok(p) if !p.is_empty() => Some(p),
            _ => None,
        }
    }

    /// Whether coverage measurement is on (any `--cov*` flag enables it).
    ///
    /// The sub-flags (`--cov-source`, `--cov-report`, `--cov-branch`) all imply
    /// `--cov` so a user never has to repeat the bare flag — passing only e.g.
    /// `--cov-branch` is enough to turn coverage on. Kept as a method (not a
    /// stored field) so the implication can't drift out of sync with the flags.
    pub fn cov_enabled(&self) -> bool {
        self.cov || !self.cov_source.is_empty() || !self.cov_report.is_empty() || self.cov_branch
    }

    /// Report kinds to produce, defaulting to `term-missing` when coverage is on.
    ///
    /// `term-missing` (the per-file table with uncovered line ranges) is the most
    /// useful default for an interactive run, matching what pytest-cov shows when
    /// you pass a bare `--cov`. An explicit `--cov-report` list is honored as-is.
    pub fn cov_reports(&self) -> Vec<String> {
        if self.cov_report.is_empty() {
            vec!["term-missing".to_string()]
        } else {
            self.cov_report.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse argv the way `main` does, but without requiring a real binary name
    /// on the front to be threaded through every call site.
    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("tezt").chain(args.iter().copied()))
    }

    #[test]
    fn cov_disabled_by_default() {
        let cli = parse(&[]);
        assert!(!cli.cov_enabled());
    }

    #[test]
    fn bare_cov_enables() {
        assert!(parse(&["--cov"]).cov_enabled());
    }

    #[test]
    fn cov_branch_alone_enables() {
        // Each sub-flag implies --cov on its own; --cov-branch is the trickiest
        // because it carries no value, so assert it explicitly.
        assert!(parse(&["--cov-branch"]).cov_enabled());
    }

    #[test]
    fn cov_source_alone_enables() {
        let cli = parse(&["--cov-source", "pkg"]);
        assert!(cli.cov_enabled());
        assert_eq!(cli.cov_source, vec!["pkg".to_string()]);
    }

    #[test]
    fn cov_report_alone_enables() {
        assert!(parse(&["--cov-report", "xml"]).cov_enabled());
    }

    #[test]
    fn default_report_is_term_missing() {
        // Coverage on but no explicit format => the interactive default.
        assert_eq!(parse(&["--cov"]).cov_reports(), vec!["term-missing"]);
    }

    #[test]
    fn explicit_reports_are_preserved_in_order() {
        let cli = parse(&["--cov-report", "term", "--cov-report", "html"]);
        assert_eq!(cli.cov_reports(), vec!["term", "html"]);
    }

    #[test]
    fn tb_defaults_to_auto() {
        // The default keeps the worker's untouched path (no `--tb` forwarded).
        assert_eq!(parse(&[]).tb, "auto");
    }

    #[test]
    fn tb_accepts_known_styles() {
        for style in ["auto", "long", "short", "line", "no"] {
            assert_eq!(parse(&["--tb", style]).tb, style);
        }
    }

    #[test]
    fn stepwise_has_sw_alias() {
        // Both spellings set the same flag; neither is on by default.
        assert!(!parse(&[]).stepwise);
        assert!(parse(&["--stepwise"]).stepwise);
        assert!(parse(&["--sw"]).stepwise);
    }

    #[test]
    fn fixtures_and_markers_are_distinct_query_flags() {
        // `--markers` populates `markers_list`, leaving the `-m` selector (the
        // `markers` field) untouched — the two must never collide.
        let cli = parse(&["--markers"]);
        assert!(cli.markers_list);
        assert!(cli.markers.is_none());
        assert!(parse(&["--fixtures"]).fixtures);
    }
}
