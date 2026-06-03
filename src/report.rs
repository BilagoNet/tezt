//! Terminal + JSON reporting.

use crate::runner::{Outcome, TestResult};
use serde::Serialize;
use std::io::Write;
use std::path::Path;

pub struct Style {
    pub color: bool,
}

impl Style {
    pub fn red(&self, s: &str) -> String {
        self.wrap(s, "31;1")
    }
    pub fn green(&self, s: &str) -> String {
        self.wrap(s, "32")
    }
    pub fn yellow(&self, s: &str) -> String {
        self.wrap(s, "33")
    }
    pub fn bold(&self, s: &str) -> String {
        self.wrap(s, "1")
    }
    pub fn dim(&self, s: &str) -> String {
        self.wrap(s, "2")
    }
    fn wrap(&self, s: &str, code: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize)]
pub struct Counts {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub xfailed: usize,
    pub xpassed: usize,
    pub errors: usize,
}

impl Counts {
    pub fn add(&mut self, o: Outcome) {
        match o {
            Outcome::Passed => self.passed += 1,
            Outcome::Failed => self.failed += 1,
            Outcome::Skipped => self.skipped += 1,
            Outcome::Xfailed => self.xfailed += 1,
            Outcome::Xpassed => self.xpassed += 1,
            Outcome::Error => self.errors += 1,
        }
    }
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped + self.xfailed + self.xpassed + self.errors
    }
    pub fn any_bad(&self) -> bool {
        self.failed > 0 || self.errors > 0
    }
}

pub struct Reporter {
    pub style: Style,
    pub verbose: bool,
    pub quiet: bool,
    pub counts: Counts,
    expected_total: usize,
    progress_at: usize,
}

impl Reporter {
    pub fn new(color: bool, verbose: bool, quiet: bool, expected_total: usize) -> Self {
        Self {
            style: Style { color },
            verbose,
            quiet,
            counts: Counts::default(),
            expected_total,
            progress_at: 0,
        }
    }

    /// Called live for every result as it streams in.
    pub fn on_result(&mut self, r: &TestResult) {
        self.counts.add(r.outcome);
        if self.verbose && !self.quiet {
            let tag = match r.outcome {
                Outcome::Passed => self.style.green("PASS"),
                Outcome::Failed => self.style.red("FAIL"),
                Outcome::Skipped => self.style.yellow("SKIP"),
                Outcome::Xfailed => self.style.yellow("XFAIL"),
                Outcome::Xpassed => self.style.yellow("XPASS"),
                Outcome::Error => self.style.red("ERROR"),
            };
            println!(
                "{tag} {} {}",
                r.id,
                self.style.dim(&format!("({:.1}ms)", r.duration_ms))
            );
        } else if !self.quiet {
            // Lightweight live progress on stderr (won't pollute stdout).
            let done = self.counts.total();
            if done >= self.progress_at + 50 || done == self.expected_total {
                let _ = write!(
                    std::io::stderr(),
                    "\r{done}/{} tests",
                    self.expected_total.max(done)
                );
                self.progress_at = done;
            }
        }
    }

    pub fn finish_progress(&self) {
        if !self.verbose && !self.quiet {
            let _ = writeln!(std::io::stderr());
        }
    }

    /// Detailed failure/error blocks, printed after the run.
    pub fn print_failures(&self, results: &[TestResult]) {
        let bad: Vec<&TestResult> = results
            .iter()
            .filter(|r| r.outcome.is_bad() || r.outcome == Outcome::Xpassed)
            .collect();
        if bad.is_empty() {
            return;
        }
        println!();
        for r in bad {
            let header = match r.outcome {
                Outcome::Failed => self.style.red(&format!("FAILED {}", r.id)),
                Outcome::Error => self.style.red(&format!("ERROR {}", r.id)),
                Outcome::Xpassed => self.style.yellow(&format!("XPASS {}", r.id)),
                _ => unreachable!(),
            };
            println!("{header}");
            if let Some(msg) = &r.message {
                for line in msg.lines() {
                    println!("  {line}");
                }
            }
            if let Some(tb) = &r.traceback {
                for line in tb.lines() {
                    println!("  {}", self.style.dim(line));
                }
            }
            if !r.stdout.is_empty() {
                println!("  {}", self.style.bold("--- captured stdout ---"));
                for line in r.stdout.lines() {
                    println!("  {line}");
                }
            }
            if !r.stderr.is_empty() {
                println!("  {}", self.style.bold("--- captured stderr ---"));
                for line in r.stderr.lines() {
                    println!("  {line}");
                }
            }
            println!();
        }
    }

    pub fn print_durations(&self, results: &[TestResult], n: usize) {
        if n == 0 || results.is_empty() {
            return;
        }
        let mut sorted: Vec<&TestResult> = results.iter().collect();
        sorted.sort_by(|a, b| b.duration_ms.partial_cmp(&a.duration_ms).unwrap());
        println!("{}", self.style.bold(&format!("slowest {n} tests:")));
        for r in sorted.into_iter().take(n) {
            println!("  {:>9.2}ms  {}", r.duration_ms, r.id);
        }
        println!();
    }

    /// Final one-line summary, pytest-flavored.
    pub fn print_summary(&self, wall_s: f64, stopped_early: bool, deselected: usize) {
        let c = &self.counts;
        let mut parts: Vec<String> = Vec::new();
        parts.push(plural_part(c.passed, "passed", true));
        if c.failed > 0 {
            parts.push(plural_part(c.failed, "failed", false));
        }
        if c.errors > 0 {
            parts.push(format!(
                "{} error{}",
                c.errors,
                if c.errors == 1 { "" } else { "s" }
            ));
        }
        if c.skipped > 0 {
            parts.push(plural_part(c.skipped, "skipped", false));
        }
        if c.xfailed > 0 {
            parts.push(plural_part(c.xfailed, "xfailed", false));
        }
        if c.xpassed > 0 {
            parts.push(plural_part(c.xpassed, "xpassed", false));
        }
        if deselected > 0 {
            parts.push(format!("{deselected} deselected"));
        }
        let body = parts.join(", ");
        let line = format!("{body} in {wall_s:.2}s");
        let line = if c.any_bad() {
            self.style.red(&line)
        } else if c.total() > 0 {
            self.style.green(&line)
        } else {
            self.style.yellow(&line)
        };
        if stopped_early {
            println!(
                "{line} {}",
                self.style.yellow("(stopped early: --maxfail reached)")
            );
        } else {
            println!("{line}");
        }
    }
}

fn plural_part(n: usize, word: &str, always: bool) -> String {
    debug_assert!(always || n > 0);
    format!("{n} {word}")
}

// --- JSON report ------------------------------------------------------------

#[derive(Serialize)]
struct JsonSummary<'a> {
    collected: usize,
    #[serde(flatten)]
    counts: &'a Counts,
    deselected: usize,
    duration_s: f64,
    exit_code: i32,
    stopped_early: bool,
}

#[derive(Serialize)]
struct JsonReport<'a> {
    tezt_version: &'static str,
    summary: JsonSummary<'a>,
    tests: &'a [TestResult],
}

#[allow(clippy::too_many_arguments)]
pub fn write_json_report(
    path: &Path,
    collected: usize,
    counts: &Counts,
    deselected: usize,
    duration_s: f64,
    exit_code: i32,
    stopped_early: bool,
    results: &[TestResult],
) -> anyhow::Result<()> {
    let report = JsonReport {
        tezt_version: env!("CARGO_PKG_VERSION"),
        summary: JsonSummary {
            collected,
            counts,
            deselected,
            duration_s,
            exit_code,
            stopped_early,
        },
        tests: results,
    };
    let mut f = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(&mut f, &report)?;
    f.write_all(b"\n")?;
    Ok(())
}
