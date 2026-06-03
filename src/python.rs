//! Python interpreter discovery.
//!
//! A pytest alternative is only useful if it runs against the *same* interpreter
//! the user's project uses — the one with their dependencies installed. Blindly
//! invoking `python3` off `PATH` (tezt's original behavior) silently ignores an
//! activated virtualenv, a project-local `.venv`, or a conda environment, and
//! then every test import-errors on packages that *are* installed, just not in
//! the base interpreter.
//!
//! We mirror a scaled-down version of uv's discovery order (uv-python's
//! `discovery.rs`): explicit override → `$VIRTUAL_ENV` → `$CONDA_PREFIX` →
//! a `.venv` found by walking up from the working directory → `PATH`. The result
//! is resolved to an absolute path so every worker spawns the identical
//! interpreter.
//!
//! ## Windows specifics
//!
//! Windows has no `python3` symlink convention and ships several traps that a
//! production tool must handle. We follow uv here:
//!
//! * Interpreters are `python.exe` / `python3.exe`, and venvs put them in
//!   `Scripts\` (not `bin/`). See uv `virtualenv.rs:189` (`virtualenv_python_executable`).
//! * The **Windows Store alias stub**: a 0-byte reparse-point `python.exe` placed
//!   on `PATH` under `…\Microsoft\WindowsApps\` that, when run, opens the Store
//!   instead of executing Python. uv detects and skips these
//!   (`discovery.rs:1552` `is_windows_store_shim`, originally from Rye); so do we.
//! * The **`py` launcher** (`py.exe`): the official PEP 397 selector. uv does not
//!   shell out to it — it reads the registry (PEP 514) and Store locations
//!   directly (`discovery.rs:473` `registry_pythons`, `microsoft_store.rs`). tezt
//!   stays dependency-light and instead uses `py` as a *fallback* PATH source and
//!   as the resolver for an `X.Y` version request, asking it to print the concrete
//!   `sys.executable`. See [`py_launcher_executable`].

use std::path::{Path, PathBuf};

/// Resolve the Python interpreter to run workers with.
///
/// `explicit` is the `--python` flag or `$TEZT_PYTHON` (already extracted by the
/// CLI layer); when set it always wins. `rootdir` is where the `.venv` walk-up
/// starts. Returns a command string — an absolute path when an interpreter was
/// discovered, falling back to the literal `"python3"` only as a last resort.
///
/// The returned string is always a single executable token (path or bare name),
/// because the runner spawns it with `Command::new(python).arg(..)`. We never
/// return something like `"py -3.12"`; a `py`-resolved interpreter is queried for
/// its concrete `sys.executable` first (see [`py_launcher_executable`]).
pub fn resolve_python(explicit: Option<&str>, rootdir: &Path) -> String {
    // 1. Explicit override (--python / $TEZT_PYTHON).
    if let Some(p) = explicit {
        return resolve_explicit(p);
    }

    // 2. Active virtualenv.
    if let Some(p) = env_prefix_python("VIRTUAL_ENV") {
        return p;
    }

    // 3. Active conda environment.
    if let Some(p) = env_prefix_python("CONDA_PREFIX") {
        return p;
    }

    // 4. A project-local `.venv` discovered by walking up from rootdir.
    if let Some(p) = discover_dot_venv(rootdir) {
        return p;
    }

    // 5. PATH (prefer the versioned `python3`, then `python`).
    if let Some(p) = find_on_path("python3").or_else(|| find_on_path("python")) {
        return p;
    }

    // 6. Windows only: fall back to the `py` launcher's default interpreter.
    //    On a fresh Windows install where Python was added without touching PATH,
    //    `py` is frequently the only thing that can find it.
    if cfg!(windows) {
        if let Some(p) = py_launcher_executable(None) {
            return p;
        }
    }

    // Last resort: let the OS resolve it and surface a clear spawn error later.
    "python3".to_string()
}

/// Resolve the explicit `--python` / `$TEZT_PYTHON` value.
///
/// Order of interpretation, mirroring uv's `PythonRequest::parse`
/// (`discovery.rs:1773`) but pared down to what tezt needs:
///
/// 1. A bare `X.Y` (or `X.Y.Z`) version → resolve to a matching interpreter.
/// 2. A path (contains a separator or is absolute) → used verbatim, with a
///    Windows `.exe` fix-up so `--python C:\py\python` finds `python.exe`.
/// 3. A bare executable name → looked up on `PATH` so the worker gets an
///    absolute path; falls back to the literal name if not found.
fn resolve_explicit(p: &str) -> String {
    // (1) Bare version request, e.g. `--python 3.12`.
    if let Some(req) = VersionRequest::parse(p) {
        if let Some(found) = resolve_version_request(&req) {
            return found;
        }
        // No interpreter matched the requested version. Fall through: the value
        // can't be a real path (it parsed as a version), so returning it verbatim
        // would only produce a confusing "no such file" spawn error. `python3` at
        // least yields a recognizable failure. We intentionally do *not* silently
        // pick an arbitrary version — that would defeat the point of the request.
        return "python3".to_string();
    }

    // (2) Explicit path.
    if p.contains(std::path::MAIN_SEPARATOR) || Path::new(p).is_absolute() {
        // On Windows, `--python C:\tools\python` should find `python.exe`.
        #[cfg(windows)]
        {
            let path = Path::new(p);
            if path.extension().is_none() && !path.is_file() {
                let with_exe = path.with_extension("exe");
                if with_exe.is_file() {
                    return with_exe.to_string_lossy().into_owned();
                }
            }
        }
        return p.to_string();
    }

    // (3) Bare executable name — resolve on PATH for an absolute path.
    find_on_path(p).unwrap_or_else(|| p.to_string())
}

/// The interpreter executable inside an environment `prefix`, if it exists.
/// Unix: `<prefix>/bin/python3` then `<prefix>/bin/python`.
/// Windows: `<prefix>\Scripts\python.exe`, then the msys2/conda fallbacks
/// `<prefix>\bin\python.exe` and `<prefix>\python.exe`
/// (cf. uv `virtualenv.rs:189`, which checks these same locations).
fn prefix_python(prefix: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    let candidates: [PathBuf; 3] = [
        prefix.join("Scripts").join("python.exe"),
        // Python installed via msys2 can produce a POSIX-like layout.
        prefix.join("bin").join("python.exe"),
        // Conda environments place the interpreter at the prefix root.
        prefix.join("python.exe"),
    ];
    #[cfg(not(windows))]
    let candidates: [PathBuf; 2] = [
        prefix.join("bin").join("python3"),
        prefix.join("bin").join("python"),
    ];

    candidates.into_iter().find(|p| p.is_file())
}

/// Resolve an interpreter from an environment variable naming an env prefix.
fn env_prefix_python(var: &str) -> Option<String> {
    let val = std::env::var_os(var)?;
    if val.is_empty() {
        return None;
    }
    let exe = prefix_python(Path::new(&val))?;
    Some(exe.to_string_lossy().into_owned())
}

/// Walk up from `start` looking for a `.venv` directory containing an
/// interpreter. Stops once it reaches (and checks) a project root, so we never
/// escape the project into `$HOME` and grab a stray `~/.venv` that belongs to
/// something else.
fn discover_dot_venv(start: &Path) -> Option<String> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    for dir in start.ancestors() {
        if let Some(exe) = prefix_python(&dir.join(".venv")) {
            return Some(exe.to_string_lossy().into_owned());
        }
        if is_project_root(dir) {
            break;
        }
    }
    None
}

/// A directory that looks like the top of a project — the walk-up for `.venv`
/// stops here (after checking this level) rather than climbing into `$HOME`.
fn is_project_root(dir: &Path) -> bool {
    const MARKERS: &[&str] = &[
        "pyproject.toml",
        "setup.py",
        "setup.cfg",
        "tox.ini",
        "tezt.toml",
        ".git",
    ];
    MARKERS.iter().any(|m| dir.join(m).exists())
}

/// Find an executable by name on `$PATH`, returning its absolute path.
///
/// On Windows we also try the `.exe`-suffixed form and skip the Windows Store
/// alias stub (see [`is_windows_store_shim`]).
fn find_on_path(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        // Candidate names to try in this directory, most-specific first.
        let candidate = dir.join(name);
        if is_executable_file(&candidate) && !is_windows_store_shim(&candidate) {
            return Some(candidate.to_string_lossy().into_owned());
        }
        // Windows: also try the .exe-suffixed form.
        if cfg!(windows) && Path::new(name).extension().is_none() {
            let exe = dir.join(format!("{name}.exe"));
            if is_executable_file(&exe) && !is_windows_store_shim(&exe) {
                return Some(exe.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg(unix)]
fn is_executable_file(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(p) {
        Ok(m) => m.is_file() && (m.permissions().mode() & 0o111 != 0),
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable_file(p: &Path) -> bool {
    p.is_file()
}

// ---------------------------------------------------------------------------
// Windows Store alias stub detection
// ---------------------------------------------------------------------------

/// Detect the Windows Store "App execution alias" stub for Python.
///
/// When Python is *not* installed from the Store but the alias is left enabled
/// (Settings → Apps → Advanced app settings → App execution aliases), a 0-byte
/// reparse-point `python.exe` / `python3.exe` sits on `PATH` under
/// `…\Local\Microsoft\WindowsApps\`. Spawning it pops open the Microsoft Store
/// installer instead of running Python — which for tezt would mean every worker
/// hangs or "succeeds" without running a single test.
///
/// uv skips these (`discovery.rs:650` filters `is_windows_store_shim`, ported
/// from Rye). It parses the reparse point and looks for the redirector marker
/// `\AppInstallerPythonRedirector.exe`. We use the same approach. Reading the
/// reparse point requires the `windows` crate; tezt deliberately avoids that
/// dependency, so we use a robust **path + zero-length heuristic** instead:
///
/// * the executable lives under a `WindowsApps` directory whose parent is
///   `Microsoft`, **and**
/// * the file is zero bytes (the alias stub has no real content; a genuine
///   `python.exe` is megabytes).
///
/// This is strictly a pre-filter to avoid *picking* a stub during discovery; a
/// real interpreter found elsewhere on `PATH` (or via `py`) is used instead. If
/// no real interpreter exists we fall through to the launcher, which is exactly
/// the behavior a user wants. The heuristic cannot misfire on a normal install
/// because real interpreters are never zero bytes.
#[cfg(windows)]
fn is_windows_store_shim(path: &Path) -> bool {
    // Match `…\Microsoft\WindowsApps\python*.exe`.
    let mut components = path.components().rev();

    // Filename must be `python*.exe`.
    let is_python_exe = components
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .is_some_and(|name| {
            let lower = name.to_ascii_lowercase();
            lower.starts_with("python") && lower.ends_with(".exe")
        });
    if !is_python_exe {
        return false;
    }

    // Parent dir must be `WindowsApps`, grandparent `Microsoft`.
    let under_windows_apps = components
        .next()
        .is_some_and(|c| c.as_os_str().eq_ignore_ascii_case("WindowsApps"));
    if !under_windows_apps {
        return false;
    }
    let under_microsoft = components
        .next()
        .is_some_and(|c| c.as_os_str().eq_ignore_ascii_case("Microsoft"));
    if !under_microsoft {
        return false;
    }

    // The alias stub is zero bytes; a real interpreter is not. `symlink_metadata`
    // avoids following the reparse point (which could trigger the redirector).
    match std::fs::symlink_metadata(path) {
        Ok(md) => md.len() == 0,
        // If we can't stat it, be conservative and treat it as a stub so we don't
        // hand the runner something that opens the Store.
        Err(_) => true,
    }
}

/// On Unix there are no Windows Store shims.
#[cfg(not(windows))]
fn is_windows_store_shim(_path: &Path) -> bool {
    false
}

// ---------------------------------------------------------------------------
// `py` launcher (Windows)
// ---------------------------------------------------------------------------

/// Resolve an interpreter via the Windows `py` launcher, returning its concrete
/// `sys.executable`.
///
/// `selector` is an optional launcher selector such as `Some("-3.12")` or
/// `Some("-3")`; `None` asks for the launcher's default interpreter. We run
/// `py [selector] -c "import sys; print(sys.executable)"` and take the printed
/// path. Resolving to a concrete path (rather than spawning `py` per worker)
/// keeps `resolve_python`'s single-token contract, lets the run header show the
/// real interpreter, and means all workers share one interpreter even if the
/// launcher's default were to change mid-run.
///
/// Returns `None` on any failure (no launcher, non-zero exit, unparseable
/// output, or a path that doesn't exist) so callers can fall through.
#[cfg(windows)]
fn py_launcher_executable(selector: Option<&str>) -> Option<String> {
    use std::process::Command;

    let mut cmd = Command::new("py");
    if let Some(sel) = selector {
        cmd.arg(sel);
    }
    cmd.args(["-c", "import sys; print(sys.executable)"]);

    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return None;
    }
    // Trust, but verify: the launcher should print an existing file.
    if Path::new(&path).is_file() {
        Some(path)
    } else {
        None
    }
}

/// On Unix there is no `py` launcher.
#[cfg(not(windows))]
fn py_launcher_executable(_selector: Option<&str>) -> Option<String> {
    None
}

// ---------------------------------------------------------------------------
// Version selection (`--python 3.12`)
// ---------------------------------------------------------------------------

/// A bare interpreter version request, e.g. `3`, `3.12`, or `3.12.1`.
///
/// This is intentionally a tiny subset of uv's `VersionRequest`
/// (`discovery.rs:191`): tezt only needs to turn a literal `X[.Y[.Z]]` from
/// `--python` into a matching interpreter. Ranges (`>=3.11`), implementations
/// (`pypy`), pre-releases, and free-threaded variants are out of scope; if the
/// user needs those they can pass an explicit path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VersionRequest {
    major: u8,
    minor: Option<u8>,
    patch: Option<u8>,
}

impl VersionRequest {
    /// Parse a *bare* version like `3`, `3.12`, or `3.12.1`. Returns `None` if
    /// the string is anything else (a path, a name, a range), so the caller can
    /// fall back to path/name handling — matching how uv only treats unambiguous
    /// version-looking values as `Version` requests (`discovery.rs:1790`).
    ///
    /// We require a leading digit and at most three dot-separated numeric
    /// segments. A single bare segment like `3` is accepted (major only). We do
    /// *not* accept things like `312` (wheel-tag form) to avoid ever mistaking a
    /// stray executable name for a version; `--python 3.12` is the documented
    /// form.
    fn parse(s: &str) -> Option<Self> {
        // Fast reject: must start with an ASCII digit and contain only digits/dots.
        if !s.starts_with(|c: char| c.is_ascii_digit()) {
            return None;
        }
        if !s.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return None;
        }

        let mut parts = s.split('.');
        let major: u8 = parts.next()?.parse().ok()?;
        let minor = match parts.next() {
            Some(seg) => Some(seg.parse().ok()?),
            None => None,
        };
        let patch = match parts.next() {
            Some(seg) => Some(seg.parse().ok()?),
            None => None,
        };
        // Reject extra segments like `3.12.1.2`.
        if parts.next().is_some() {
            return None;
        }
        // A lone `0` major (or similar) is not a Python we support; keep it simple
        // and require Python 3+ like uv does (`check_supported`, discovery.rs:2818).
        if major < 3 {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    /// The versioned executable basename for this request, e.g. `python3.12`.
    /// On Windows this gains an `.exe` suffix at call sites via [`find_on_path`].
    /// Mirrors uv's `ExecutableName` rendering (`discovery.rs:2612`): only the
    /// major and major.minor forms exist as real on-disk names (`python3`,
    /// `python3.12`); there is no `python3.12.1` binary, so a patch request
    /// degrades to the major.minor name and is verified by querying.
    fn executable_name(&self) -> String {
        match self.minor {
            Some(minor) => format!("python{}.{}", self.major, minor),
            None => format!("python{}", self.major),
        }
    }

    /// The `py` launcher selector for this request, e.g. `-3.12` or `-3`.
    fn launcher_selector(&self) -> String {
        match self.minor {
            Some(minor) => format!("-{}.{}", self.major, minor),
            None => format!("-{}", self.major),
        }
    }
}

/// Resolve a bare `X.Y` version request to a concrete interpreter path.
///
/// Search order, deliberately narrow and side-effect-free:
///
/// 1. **Active / discovered environment**, but only if it actually matches the
///    requested version. A user who asked for `3.12` while a `3.11` venv is
///    active wants `3.12`, not the venv — so we *verify* the version rather than
///    blindly preferring the env (this is why we query the interpreter).
/// 2. **`PATH`** for the versioned name (`python3.12` / `python3.12.exe`).
/// 3. **Windows `py` launcher** with the matching selector (`py -3.12`).
///
/// A requested patch (`3.12.1`) has no dedicated executable name; we resolve the
/// `python3.12` / `py -3.12` candidate and then confirm the full `X.Y.Z` by
/// querying it, so `--python 3.12.1` won't silently accept `3.12.4`.
fn resolve_version_request(req: &VersionRequest) -> Option<String> {
    // (1) An active/discovered environment that happens to match the request.
    for var in ["VIRTUAL_ENV", "CONDA_PREFIX"] {
        if let Some(p) = env_prefix_python(var) {
            if interpreter_matches(&p, req) {
                return Some(p);
            }
        }
    }

    // (2) PATH: the versioned name. `find_on_path` already handles `.exe` and
    //     skips Store stubs on Windows.
    let name = req.executable_name();
    if let Some(p) = find_on_path(&name) {
        // For a major-only or major.minor request the name already encodes the
        // match; for a patch request we still confirm Z.
        if req.patch.is_none() || interpreter_matches(&p, req) {
            return Some(p);
        }
    }

    // (3) Windows `py` launcher with the matching selector.
    if cfg!(windows) {
        if let Some(p) = py_launcher_executable(Some(&req.launcher_selector())) {
            if req.patch.is_none() || interpreter_matches(&p, req) {
                return Some(p);
            }
        }
    }

    None
}

/// Query an interpreter for its `major.minor.patch` and check it satisfies `req`.
///
/// Used only when we cannot prove the match from the executable name alone
/// (an active env, or a patch-level request). Spawning the interpreter once here
/// is cheap relative to a whole test run and mirrors uv, which ultimately queries
/// every candidate before trusting it (`discovery.rs:778` `matches_interpreter`).
///
/// Returns `false` on any failure to query, so a broken candidate is skipped
/// rather than mistakenly accepted.
fn interpreter_matches(python: &str, req: &VersionRequest) -> bool {
    use std::process::Command;

    // Print "major minor patch" on one line; trivial to parse, no JSON dep.
    let output = Command::new(python)
        .args(["-c", "import sys;print(*sys.version_info[:3])"])
        .output();
    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut nums = stdout
        .split_whitespace()
        .filter_map(|s| s.parse::<u8>().ok());
    let (Some(major), Some(minor), Some(patch)) = (nums.next(), nums.next(), nums.next()) else {
        return false;
    };

    if major != req.major {
        return false;
    }
    if let Some(req_minor) = req.minor {
        if minor != req_minor {
            return false;
        }
    }
    if let Some(req_patch) = req.patch {
        if patch != req_patch {
            return false;
        }
    }
    true
}

/// Best-effort short label for the chosen interpreter, used in the run header.
/// Avoids spawning Python: shows the path, plus the env name when the
/// interpreter clearly lives inside a `.venv` (`<proj>/.venv/bin/python` on
/// Unix, `<proj>\.venv\Scripts\python.exe` on Windows).
pub fn label(python: &str) -> String {
    let p = Path::new(python);
    // The directory that holds the executable: `bin` on Unix, `Scripts` on
    // Windows venvs (with a `bin`/root fallback for msys2/conda layouts).
    let exe_dir_names: &[&str] = if cfg!(windows) {
        &["Scripts", "bin"]
    } else {
        &["bin"]
    };
    let in_exe_dir = p.parent().filter(|d| {
        d.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| exe_dir_names.contains(&n))
    });
    if let Some(env_name) = in_exe_dir
        .and_then(|d| d.parent())
        .and_then(|d| d.file_name())
        .and_then(|n| n.to_str())
        .filter(|n| *n == ".venv" || n.ends_with("venv") || n.ends_with("-env"))
    {
        return format!("{python} ({env_name})");
    }
    python.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn explicit_absolute_path_wins() {
        let got = resolve_python(Some("/opt/py/bin/python3"), Path::new("/tmp"));
        assert_eq!(got, "/opt/py/bin/python3");
    }

    #[test]
    fn discovers_dot_venv_by_walking_up() {
        // Only meaningful when no ambient venv/conda would take precedence.
        if std::env::var_os("VIRTUAL_ENV").is_some() || std::env::var_os("CONDA_PREFIX").is_some() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let venv_bin = dir
            .path()
            .join(".venv")
            .join(if cfg!(windows) { "Scripts" } else { "bin" });
        fs::create_dir_all(&venv_bin).unwrap();
        let exe = venv_bin.join(if cfg!(windows) {
            "python.exe"
        } else {
            "python3"
        });
        fs::write(&exe, b"#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
        }
        // Start from a nested subdirectory so the walk-up must climb to find it.
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        let got = resolve_python(None, &nested);
        // Compare canonicalized forms: discovery canonicalizes its start dir, so
        // `got` may differ from `exe` only by symlinks (e.g. macOS /var vs
        // /private/var) — the underlying file must be the same.
        assert_eq!(
            fs::canonicalize(&got).unwrap(),
            fs::canonicalize(&exe).unwrap()
        );
    }

    #[test]
    fn label_annotates_only_venvs() {
        assert_eq!(
            label("/proj/.venv/bin/python3"),
            "/proj/.venv/bin/python3 (.venv)"
        );
        // System interpreters are shown verbatim, not mislabeled "(bin)"/"(usr)".
        assert_eq!(label("/usr/bin/python3"), "/usr/bin/python3");
    }

    #[test]
    fn label_annotates_windows_scripts_venvs() {
        // Use backslashes regardless of host so the test is meaningful on CI.
        // `label` matches on the `Scripts` / `bin` component name, which is
        // platform-independent string logic.
        #[cfg(windows)]
        {
            assert_eq!(
                label(r"C:\proj\.venv\Scripts\python.exe"),
                r"C:\proj\.venv\Scripts\python.exe (.venv)"
            );
        }
    }

    // --- version request parsing ------------------------------------------

    #[test]
    fn version_request_parses_bare_versions() {
        assert_eq!(
            VersionRequest::parse("3"),
            Some(VersionRequest {
                major: 3,
                minor: None,
                patch: None
            })
        );
        assert_eq!(
            VersionRequest::parse("3.12"),
            Some(VersionRequest {
                major: 3,
                minor: Some(12),
                patch: None
            })
        );
        assert_eq!(
            VersionRequest::parse("3.12.1"),
            Some(VersionRequest {
                major: 3,
                minor: Some(12),
                patch: Some(1)
            })
        );
    }

    #[test]
    fn version_request_rejects_non_versions() {
        // Paths, names, ranges, wheel-tag form, junk — none are bare versions.
        for s in [
            "python3",
            "python3.12",
            "/usr/bin/python3",
            "./python",
            ">=3.11",
            "3.12rc1",
            "312",      // wheel-tag form is intentionally not accepted
            "3.12.1.2", // too many segments
            "2.7",      // Python <3 unsupported
            "",
            "3.x",
        ] {
            assert_eq!(VersionRequest::parse(s), None, "should reject {s:?}");
        }
    }

    #[test]
    fn version_request_executable_and_selector_names() {
        let mm = VersionRequest::parse("3.12").unwrap();
        assert_eq!(mm.executable_name(), "python3.12");
        assert_eq!(mm.launcher_selector(), "-3.12");

        let major_only = VersionRequest::parse("3").unwrap();
        assert_eq!(major_only.executable_name(), "python3");
        assert_eq!(major_only.launcher_selector(), "-3");
    }

    #[test]
    fn explicit_path_with_separator_is_used_verbatim_on_unix() {
        // A value containing a separator that isn't a bare version is treated as a
        // path. On Unix there is no `.exe` fix-up, so it round-trips unchanged.
        #[cfg(unix)]
        {
            let got = resolve_explicit("/opt/python/bin/python3");
            assert_eq!(got, "/opt/python/bin/python3");
        }
    }

    #[test]
    fn windows_store_shim_detection_is_path_scoped() {
        // On non-Windows this is always false (no stubs exist); assert the
        // platform contract so the cfg wiring can't silently break.
        #[cfg(not(windows))]
        {
            assert!(!is_windows_store_shim(Path::new(
                r"C:\Users\x\AppData\Local\Microsoft\WindowsApps\python.exe"
            )));
        }
        // On Windows, a zero-byte file under Microsoft\WindowsApps is a stub;
        // a path that doesn't match the layout is never treated as one.
        #[cfg(windows)]
        {
            let dir = tempfile::tempdir().unwrap();
            let unrelated = dir.path().join("python.exe");
            fs::write(&unrelated, b"not empty").unwrap();
            assert!(!is_windows_store_shim(&unrelated));

            let store = dir
                .path()
                .join("Microsoft")
                .join("WindowsApps")
                .join("python.exe");
            fs::create_dir_all(store.parent().unwrap()).unwrap();
            fs::write(&store, b"").unwrap(); // zero bytes => stub
            assert!(is_windows_store_shim(&store));
        }
    }
}
