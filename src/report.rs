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

// --- JUnit XML report --------------------------------------------------------

/// Escape a string for safe inclusion in XML, in either attribute or text
/// position (we use the same routine for both — over-escaping `"`/`'` in text
/// is harmless and keeps the helper to one code path).
///
/// Two things happen here:
///   * the five XML metacharacters (`&`, `<`, `>`, `"`, `'`) become entity
///     references — `&` first, so we never double-escape an entity we just
///     emitted;
///   * characters that XML 1.0 forbids *entirely* (most C0 control codes — a
///     raw NUL or ESC makes the document unparseable, and there is no entity
///     for them) are dropped, except the three that are legal: tab, newline,
///     carriage return. Python tracebacks and captured output routinely carry
///     such bytes (e.g. ANSI color escapes), so silently stripping them is what
///     keeps the XML well-formed for the CI reporter that ingests it.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(c),
            // Strip the remaining C0 controls (and the lone DEL) that XML 1.0
            // disallows; keep everything else (all printable + higher Unicode).
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {}
            c => out.push(c),
        }
    }
    out
}

/// Split a test id into a JUnit `(classname, name)` pair.
///
/// JUnit's model is "a class containing methods"; pytest maps a test's *file*
/// to `classname` and the *rest* of the id to `name`, and CI dashboards group
/// by `classname`, so it pays to derive a stable, dotted module-like value. We
/// take the file portion (everything before the first `::`), strip a trailing
/// `.py`, and turn path separators into dots (`tests/test_api.py` →
/// `tests.test_api`). The remainder after `::` becomes the `name`
/// (`TestC::test_x` → `TestC::test_x`); for a file-level id with no `::` we fall
/// back to the dotted file as the name too, so the pair is never empty.
fn junit_classname_name(id: &str) -> (String, String) {
    match id.split_once("::") {
        Some((file, rest)) => (dotted_module(file), rest.to_string()),
        // No `::` (e.g. a whole-file dynamic item): use the dotted file for both
        // halves so neither attribute is blank and the case stays identifiable.
        None => {
            let module = dotted_module(id);
            (module.clone(), module)
        }
    }
}

/// Turn a file path into a dotted module name: drop a trailing `.py`, then
/// replace both `/` and `\` with `.` so the value is stable across platforms.
fn dotted_module(file: &str) -> String {
    // Normalize a leading `./` (or `.\`), which appears when tests are collected
    // from `.`, so the classname is `test_conv`, not `..test_conv`. Trailing/
    // leading dots are trimmed after separator replacement for the same reason.
    let file = file
        .strip_prefix("./")
        .or_else(|| file.strip_prefix(".\\"))
        .unwrap_or(file);
    let stem = file.strip_suffix(".py").unwrap_or(file);
    stem.replace(['/', '\\'], ".").trim_matches('.').to_string()
}

/// Write a JUnit XML report to `path`.
///
/// The shape is the widely-supported subset that GitLab, Jenkins, and the
/// GitHub test-reporter actions all understand: a `<testsuites>` root wrapping a
/// single `<testsuite name="tezt">` whose attributes carry the run totals, then
/// one `<testcase>` per result. Outcomes map to JUnit's vocabulary as follows:
///   * `Passed` / `Xpassed` → a bare `<testcase>` (Xpassed is a *pass* as far as
///     totals go — see [`Counts::any_bad`] — so we report it as one here to keep
///     the suite's `tests`/`failures` arithmetic consistent with `Counts`);
///   * `Failed` → `<testcase><failure message="...">traceback</failure>`;
///   * `Error`  → `<testcase><error   message="...">traceback</error></...>`;
///   * `Skipped` / `Xfailed` → `<testcase><skipped message="..."/></testcase>`.
///
/// `time` is seconds (JUnit's unit), i.e. `duration_ms / 1000`. Every attribute
/// value and every text node is run through [`xml_escape`]. `expected_total` is
/// accepted for signature parity with the JSON writer and to document intent;
/// the authoritative `tests=` value is `counts.total()` so it can never drift
/// from the per-case rows we actually emit.
pub fn write_junit_xml(
    path: &Path,
    expected_total: usize,
    counts: &Counts,
    wall_time_s: f64,
    results: &[TestResult],
) -> anyhow::Result<()> {
    let _ = expected_total;
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");

    // `errors` and `failures` are distinct JUnit columns: a `failure` is an
    // assertion that didn't hold, an `error` is an unexpected exception/crash.
    // We mirror tezt's own Failed vs Error split onto them. `skipped` folds in
    // xfailed (an expected, deselected-style skip).
    let total = counts.total();
    let failures = counts.failed;
    let errors = counts.errors;
    let skipped = counts.skipped + counts.xfailed;

    out.push_str("<testsuites>\n");
    out.push_str(&format!(
        "  <testsuite name=\"tezt\" tests=\"{total}\" failures=\"{failures}\" errors=\"{errors}\" skipped=\"{skipped}\" time=\"{wall_time_s:.3}\">\n"
    ));

    for r in results {
        let (classname, name) = junit_classname_name(&r.id);
        let time_s = r.duration_ms / 1000.0;
        let open = format!(
            "    <testcase classname=\"{}\" name=\"{}\" time=\"{time_s:.3}\"",
            xml_escape(&classname),
            xml_escape(&name),
        );

        match r.outcome {
            Outcome::Passed | Outcome::Xpassed => {
                // Self-closing: a clean pass carries no child element.
                out.push_str(&open);
                out.push_str("/>\n");
            }
            Outcome::Failed | Outcome::Error => {
                let tag = if r.outcome == Outcome::Failed {
                    "failure"
                } else {
                    "error"
                };
                let msg = r.message.as_deref().unwrap_or("");
                out.push_str(&open);
                out.push_str(">\n");
                out.push_str(&format!("      <{tag} message=\"{}\">", xml_escape(msg)));
                // The traceback is the human-readable body; include it as text
                // so a reporter can show the full context on click-through.
                if let Some(tb) = &r.traceback {
                    out.push_str(&xml_escape(tb));
                }
                out.push_str(&format!("</{tag}>\n"));
                out.push_str("    </testcase>\n");
            }
            Outcome::Skipped | Outcome::Xfailed => {
                let msg = r.message.as_deref().unwrap_or("");
                out.push_str(&open);
                out.push_str(">\n");
                out.push_str(&format!(
                    "      <skipped message=\"{}\"/>\n",
                    xml_escape(msg)
                ));
                out.push_str("    </testcase>\n");
            }
        }
    }

    out.push_str("  </testsuite>\n");
    out.push_str("</testsuites>\n");

    let mut f = std::fs::File::create(path)?;
    f.write_all(out.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(id: &str, outcome: Outcome, message: Option<&str>) -> TestResult {
        TestResult {
            id: id.to_string(),
            outcome,
            duration_ms: 12.0,
            message: message.map(str::to_string),
            traceback: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    #[test]
    fn junit_xml_shape_counts_and_escaping() {
        // One pass, one failure (with a `<` in its message that must be escaped),
        // and one skip — exercises every branch plus the suite-level arithmetic.
        let results = vec![
            result("tests/test_api.py::test_ok", Outcome::Passed, None),
            result(
                "tests/test_api.py::test_bad",
                Outcome::Failed,
                Some("assert 1 < 2 failed"),
            ),
            result(
                "tests/test_api.py::test_skip",
                Outcome::Skipped,
                Some("no db"),
            ),
        ];
        let mut counts = Counts::default();
        for r in &results {
            counts.add(r.outcome);
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("junit.xml");
        write_junit_xml(&path, results.len(), &counts, 0.5, &results).unwrap();
        let xml = std::fs::read_to_string(&path).unwrap();

        // Suite element with the right rolled-up totals.
        assert!(xml.contains("<testsuite name=\"tezt\""));
        assert!(xml.contains("tests=\"3\""));
        assert!(xml.contains("failures=\"1\""));
        assert!(xml.contains("errors=\"0\""));
        assert!(xml.contains("skipped=\"1\""));

        // A failure element is emitted for the failing case.
        assert!(xml.contains("<failure message="));
        // The `<` inside the message was escaped, and no raw `<` leaked into
        // the attribute text.
        assert!(xml.contains("assert 1 &lt; 2 failed"));
        assert!(!xml.contains("assert 1 < 2 failed"));

        // classname derivation: file → dotted module, `.py` stripped.
        assert!(xml.contains("classname=\"tests.test_api\""));
        assert!(xml.contains("name=\"test_ok\""));

        // The skip becomes a <skipped> child.
        assert!(xml.contains("<skipped message=\"no db\""));
    }

    #[test]
    fn xml_escape_strips_invalid_control_chars_but_keeps_ws() {
        // NUL and ESC are illegal in XML 1.0 and must vanish; tab/newline stay.
        let escaped = xml_escape("a\x00b\x1b\tc\nd");
        assert_eq!(escaped, "ab\tc\nd");
    }
}
