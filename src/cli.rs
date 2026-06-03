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

    /// Number of parallel Python workers (default: number of CPUs)
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    pub jobs: Option<usize>,

    /// Collect tests without running them
    #[arg(long = "collect-only")]
    pub collect_only: bool,

    /// Write a machine-readable JSON report to the given path
    #[arg(long = "json", value_name = "PATH")]
    pub json: Option<PathBuf>,

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
}
