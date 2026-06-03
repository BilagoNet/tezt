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

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Resolve the Python interpreter to run workers with.
///
/// `explicit` is the `--python` flag or `$TEZT_PYTHON` (already extracted by the
/// CLI layer); when set it always wins. `rootdir` is where the `.venv` walk-up
/// starts. Returns a command string — an absolute path when an interpreter was
/// discovered, falling back to the literal `"python3"` only as a last resort.
pub fn resolve_python(explicit: Option<&str>, rootdir: &Path) -> String {
    // 1. Explicit override (--python / $TEZT_PYTHON). If it's a bare name we
    //    still try to resolve it on PATH so the worker gets an absolute path.
    if let Some(p) = explicit {
        if p.contains(std::path::MAIN_SEPARATOR) || Path::new(p).is_absolute() {
            return p.to_string();
        }
        return find_on_path(p).unwrap_or_else(|| p.to_string());
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

    // Last resort: let the OS resolve it and surface a clear spawn error later.
    "python3".to_string()
}

/// The interpreter executable inside an environment `prefix`, if it exists.
/// Unix: `<prefix>/bin/python3` then `<prefix>/bin/python`.
/// Windows: `<prefix>\Scripts\python.exe` then `<prefix>\python.exe`.
fn prefix_python(prefix: &Path) -> Option<PathBuf> {
    let candidates: [PathBuf; 2] = if cfg!(windows) {
        [
            prefix.join("Scripts").join("python.exe"),
            prefix.join("python.exe"),
        ]
    } else {
        [prefix.join("bin").join("python3"), prefix.join("bin").join("python")]
    };
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
fn find_on_path(name: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate.to_string_lossy().into_owned());
        }
        // Windows: also try the .exe-suffixed form.
        if cfg!(windows) && Path::new(name).extension().is_none() {
            let exe = dir.join(format!("{name}.exe"));
            if is_executable_file(&exe) {
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

/// Best-effort short label for the chosen interpreter, used in the run header.
/// Avoids spawning Python: shows the path, plus the env name when the
/// interpreter clearly lives inside a `.venv` (`<proj>/.venv/bin/python`).
pub fn label(python: &str) -> String {
    let p = Path::new(python);
    let in_bin = p.parent().filter(|d| d.file_name() == Some(OsStr::new("bin")));
    if let Some(env_name) = in_bin
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
        let exe = venv_bin.join(if cfg!(windows) { "python.exe" } else { "python3" });
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
        assert_eq!(label("/proj/.venv/bin/python3"), "/proj/.venv/bin/python3 (.venv)");
        // System interpreters are shown verbatim, not mislabeled "(bin)"/"(usr)".
        assert_eq!(label("/usr/bin/python3"), "/usr/bin/python3");
    }
}
