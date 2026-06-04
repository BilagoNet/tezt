//! Project configuration from `pyproject.toml`'s `[tool.tezt]` table.
//!
//! This is tezt's equivalent of pytest's `[tool.pytest.ini_options]`: a place to
//! record per-project defaults so they don't have to be retyped on every
//! invocation. We read exactly three keys today — `addopts`, `testpaths`, and
//! `markers` — and only from `pyproject.toml` at the rootdir.
//!
//! The guiding principle is that configuration is a *convenience*, never a
//! gate: a missing file, a missing `[tool.tezt]` table, a malformed TOML
//! document, or a key with the wrong type must all degrade gracefully to "as if
//! that key weren't set" rather than abort the run. A test runner that refuses
//! to start because a comma is misplaced in `pyproject.toml` is worse than one
//! that quietly ignores the typo and tells you on stderr. So [`Config::load`]
//! never returns an error — every failure path warns (`tezt: config: ...`) and
//! falls back to a default.

use std::path::Path;

/// Project configuration read from `[tool.tezt]` in `pyproject.toml` at the
/// rootdir. All fields optional; a missing file or missing table yields an
/// empty config. Parsing is best-effort — a malformed pyproject must never
/// fail the run (warn to stderr and continue with defaults).
#[derive(Debug, Default, Clone)]
pub struct Config {
    /// Extra default CLI args, prepended before the user's argv (pytest's `addopts`).
    pub addopts: Vec<String>,
    /// Default paths to collect when none are given on the command line.
    pub testpaths: Vec<String>,
    /// Registered markers, each kept verbatim (e.g. `slow: long-running`) so
    /// `--markers` can show the description. The bare name (before the first
    /// `:`) can be derived later for a `--strict-markers` check.
    pub markers: Vec<String>,
}

impl Config {
    /// Load `<rootdir>/pyproject.toml` and read `[tool.tezt]`. Never errors.
    ///
    /// Resolution order for each failure, all yielding an empty/partial config:
    ///   * no `pyproject.toml` → silent (the common case: most projects using
    ///     tezt ad hoc have no config, and a "file not found" warning on every
    ///     run would be noise);
    ///   * unreadable or un-parseable `pyproject.toml` → one warning, default
    ///     config (we can't trust *any* of it, so we take none of it);
    ///   * no `[tool.tezt]` table → silent (the file exists for other tools);
    ///   * a present key with the wrong TOML type → one warning naming the key,
    ///     and we keep the other keys.
    pub fn load(rootdir: &Path) -> Config {
        let path = rootdir.join("pyproject.toml");
        // A missing file is the overwhelmingly common case and not worth a
        // warning. Any *other* read error (permissions, a directory in the way)
        // is unusual enough to surface, but still non-fatal.
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Config::default(),
            Err(e) => {
                eprintln!("tezt: config: cannot read {}: {e}", path.display());
                return Config::default();
            }
        };

        // A syntactically broken pyproject is a project-wide problem the user
        // will hit with every tool; we report it once and proceed with defaults
        // rather than letting a stray bracket block the test run.
        let table: toml::Table = match text.parse() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("tezt: config: ignoring malformed {}: {e}", path.display());
                return Config::default();
            }
        };

        // `[tool.tezt]` — absent is normal (the file may exist only for build
        // metadata or other tools), so a missing table is silent.
        let tezt = table
            .get("tool")
            .and_then(toml::Value::as_table)
            .and_then(|tool| tool.get("tezt"))
            .and_then(toml::Value::as_table);
        let Some(tezt) = tezt else {
            return Config::default();
        };

        Config::from_table(tezt)
    }

    /// Build a [`Config`] from an already-resolved `[tool.tezt]` table. Split
    /// out from [`load`](Config::load) so the parsing rules can be unit-tested
    /// directly and so the I/O (find + read + parse the file) is separable from
    /// the *interpretation* of the keys. Per-key type errors warn and skip just
    /// that key, leaving the rest intact.
    fn from_table(tezt: &toml::Table) -> Config {
        let mut cfg = Config::default();

        // addopts: accept either a single string (shell-style, split on
        // whitespace) or an array of already-tokenized strings. The string form
        // matches how a human writes flags in a file (`addopts = "-q --tb=short"`)
        // and is what pytest accepts; the array form is unambiguous when an
        // individual arg would otherwise be split.
        if let Some(v) = tezt.get("addopts") {
            match v {
                // NOTE: this is a *naive* whitespace split — it does NOT honor
                // shell quoting, so `addopts = "-k 'a or b'"` becomes the four
                // tokens `-k`, `'a`, `or`, `b'` rather than `-k` + `a or b`. A
                // value that needs embedded spaces should use the array form,
                // e.g. `addopts = ["-k", "a or b"]`. Documenting the limitation
                // here (and in the README later) is cheaper than shipping a
                // half-correct shell-lexer.
                toml::Value::String(s) => {
                    cfg.addopts = s.split_whitespace().map(str::to_owned).collect();
                }
                toml::Value::Array(arr) => {
                    cfg.addopts = string_array(arr, "addopts");
                }
                _ => eprintln!(
                    "tezt: config: 'addopts' must be a string or array of strings; ignoring"
                ),
            }
        }

        // testpaths: where to collect when the command line names no paths.
        // Accept a single string for the one-directory case, otherwise an array.
        if let Some(v) = tezt.get("testpaths") {
            match v {
                toml::Value::String(s) => cfg.testpaths = vec![s.clone()],
                toml::Value::Array(arr) => cfg.testpaths = string_array(arr, "testpaths"),
                _ => eprintln!(
                    "tezt: config: 'testpaths' must be a string or array of strings; ignoring"
                ),
            }
        }

        // markers: pytest stores entries as `name: human description`. We keep
        // each entry verbatim so `--markers` can print the description; the bare
        // name (everything before the first `:`) can be derived later for a
        // `--strict-markers` check.
        if let Some(v) = tezt.get("markers") {
            match v {
                toml::Value::Array(arr) => {
                    cfg.markers = string_array(arr, "markers")
                        .iter()
                        .map(|entry| entry.trim().to_owned())
                        .filter(|entry| !entry.is_empty())
                        .collect();
                }
                _ => eprintln!("tezt: config: 'markers' must be an array of strings; ignoring"),
            }
        }

        cfg
    }
}

/// Collect a TOML array into `Vec<String>`, skipping (with a per-element
/// warning) any entry that isn't a string. Used by every key that takes a list,
/// so a single stray non-string never discards the whole list — we keep what we
/// can and name what we dropped.
fn string_array(arr: &[toml::Value], key: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(arr.len());
    for v in arr {
        match v.as_str() {
            Some(s) => out.push(s.to_owned()),
            None => eprintln!("tezt: config: '{key}' entry {v} is not a string; skipping"),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    /// Write `body` to a fresh tempdir's `pyproject.toml`, load it, and hand
    /// back both the parsed config and the dir guard (kept alive by the caller
    /// so the file isn't reaped before `load` reads it).
    fn load_from(body: &str) -> (Config, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join("pyproject.toml")).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        f.flush().unwrap();
        let cfg = Config::load(dir.path());
        (cfg, dir)
    }

    #[test]
    fn addopts_as_string_is_whitespace_split() {
        let (cfg, _d) = load_from("[tool.tezt]\naddopts = \"-q --tb=short -v\"\n");
        assert_eq!(cfg.addopts, vec!["-q", "--tb=short", "-v"]);
    }

    #[test]
    fn addopts_as_array_is_taken_verbatim() {
        // The array form preserves a token with an embedded space, which the
        // string form could not.
        let (cfg, _d) = load_from("[tool.tezt]\naddopts = [\"-k\", \"a or b\"]\n");
        assert_eq!(cfg.addopts, vec!["-k", "a or b"]);
    }

    #[test]
    fn testpaths_array_and_single_string() {
        let (cfg, _d) = load_from("[tool.tezt]\ntestpaths = [\"tests\", \"integration\"]\n");
        assert_eq!(cfg.testpaths, vec!["tests", "integration"]);

        let (cfg, _d) = load_from("[tool.tezt]\ntestpaths = \"tests\"\n");
        assert_eq!(cfg.testpaths, vec!["tests"]);
    }

    #[test]
    fn markers_are_kept_verbatim() {
        let (cfg, _d) = load_from(
            "[tool.tezt]\nmarkers = [\"slow: marks tests as slow\", \"network\", \"db: needs a database\"]\n",
        );
        assert_eq!(
            cfg.markers,
            vec![
                "slow: marks tests as slow",
                "network",
                "db: needs a database"
            ]
        );
    }

    #[test]
    fn missing_file_yields_default() {
        // A directory with no pyproject.toml at all => empty config, no panic.
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config::load(dir.path());
        assert!(cfg.addopts.is_empty());
        assert!(cfg.testpaths.is_empty());
        assert!(cfg.markers.is_empty());
    }

    #[test]
    fn missing_tezt_table_yields_default() {
        // pyproject exists for some *other* tool; we read nothing and don't warn.
        let (cfg, _d) = load_from("[tool.black]\nline-length = 88\n");
        assert!(cfg.addopts.is_empty());
        assert!(cfg.testpaths.is_empty());
        assert!(cfg.markers.is_empty());
    }

    #[test]
    fn malformed_toml_yields_default_without_panicking() {
        // A broken document must not abort: load returns defaults.
        let (cfg, _d) = load_from("[tool.tezt\naddopts = ");
        assert!(cfg.addopts.is_empty());
    }

    #[test]
    fn wrong_type_skips_only_that_key() {
        // `addopts` is the wrong type (a number) but `testpaths` is valid: we
        // drop the bad key and keep the good one.
        let (cfg, _d) = load_from("[tool.tezt]\naddopts = 42\ntestpaths = [\"tests\"]\n");
        assert!(cfg.addopts.is_empty());
        assert_eq!(cfg.testpaths, vec!["tests"]);
    }
}
