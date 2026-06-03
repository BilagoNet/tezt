//! Test collection: walk the filesystem and statically parse Python test
//! files with a Rust AST parser. No Python interpreter is needed to collect.
//!
//! Parsing is collection's dominant cost, so a warm run reuses a persistent
//! per-file cache ([`crate::cache`]): if a file is unchanged we reconstruct its
//! items without reading or parsing it.

use crate::cache::{Cache, CachedCollection, CachedItem};
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use rayon::prelude::*;
use rustpython_parser::{ast, Parse};
use std::path::{Path, PathBuf};

/// One collected test item (pre-parametrize-expansion unit the worker runs).
#[derive(Debug, Clone)]
pub struct TestItem {
    /// Stable id, e.g. `testdata/basic/test_math.py::TestFoo::test_bar`.
    /// For dynamic (unparseable) files this is just the file path.
    pub id: String,
    /// Absolute path to the file.
    pub file: PathBuf,
    /// `test_func`, `TestClass::test_method`, or `*` for dynamic discovery.
    pub qualname: String,
    /// Statically predicted parametrize case ids (display/filtering only).
    /// `None` = not parametrized; `Some(vec![])` = parametrized but case ids
    /// could not be determined statically.
    pub param_ids: Option<Vec<String>>,
}

impl TestItem {
    /// Number of test cases this item is expected to expand to.
    pub fn expected_cases(&self) -> usize {
        match &self.param_ids {
            None => 1,
            Some(ids) if ids.is_empty() => 1, // unknown expansion; count base
            Some(ids) => ids.len(),
        }
    }

    /// Display ids after static parametrize expansion.
    pub fn display_ids(&self) -> Vec<String> {
        match &self.param_ids {
            None => vec![self.id.clone()],
            Some(ids) if ids.is_empty() => vec![format!("{}[...]", self.id)],
            Some(ids) => ids.iter().map(|p| format!("{}[{}]", self.id, p)).collect(),
        }
    }
}

/// Result of collecting one file.
//
// `file` and `dynamic` are part of the per-file collection record (useful for
// debugging and future reporting), even though the run path currently consumes
// only `items` — hence the targeted `allow`.
#[derive(Debug)]
#[allow(dead_code)]
pub struct CollectedFile {
    pub file: PathBuf,
    pub rel: String,
    pub items: Vec<TestItem>,
    /// File could not be parsed statically; the worker will discover tests
    /// at import time (a syntax error will surface as an `error` outcome).
    pub dynamic: bool,
}

const SKIP_DIRS: &[&str] = &[
    ".git",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    ".nox",
    "node_modules",
    ".mypy_cache",
    ".ruff_cache",
    ".pytest_cache",
    "dist",
    "build",
    ".eggs",
];

fn is_test_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    name.ends_with(".py") && (name.starts_with("test_") || name.ends_with("_test.py"))
}

fn skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name) || name.ends_with(".egg-info")
}

/// Find candidate test files under the given paths.
pub fn find_test_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for p in paths {
        let p = if p.as_os_str().is_empty() {
            Path::new(".")
        } else {
            p.as_path()
        };
        if p.is_file() {
            // An explicitly named file is always collected (pytest behavior).
            if p.extension().and_then(|e| e.to_str()) == Some("py") {
                files.push(p.to_path_buf());
            }
            continue;
        }
        if !p.exists() {
            anyhow::bail!("path does not exist: {}", p.display());
        }
        let walker = WalkBuilder::new(p)
            .hidden(true)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(false)
            .follow_links(false)
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !(e.file_type().map(|t| t.is_dir()).unwrap_or(false) && skip_dir(&name))
            })
            .build();
        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) && is_test_file(entry.path())
            {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// Collect all test items from the given root paths, in parallel.
///
/// `cache`, when `Some`, lets unchanged files skip read+parse. `Option<&Cache>`
/// is `Sync`, so each rayon worker shares it without locking (every file owns a
/// distinct cache entry).
pub fn collect(
    paths: &[PathBuf],
    rootdir: &Path,
    cache: Option<&Cache>,
) -> Result<Vec<CollectedFile>> {
    let files = find_test_files(paths)?;
    let mut collected: Vec<CollectedFile> = files
        .par_iter()
        .map(|f| collect_file(f, rootdir, cache))
        .collect::<Result<Vec<_>>>()?;
    // Keep deterministic order (par_iter preserves order, but be explicit).
    collected.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(collected)
}

fn rel_id(file: &Path, rootdir: &Path) -> String {
    let rel = file.strip_prefix(rootdir).unwrap_or(file);
    rel.to_string_lossy().replace('\\', "/")
}

fn collect_file(file: &Path, rootdir: &Path, cache: Option<&Cache>) -> Result<CollectedFile> {
    let abs =
        fs_err::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
    let rel = rel_id(file, rootdir);

    // Freshness key from the file's metadata (size + mtime + tezt version).
    // If we can read it and the cache has a matching entry, reconstruct the
    // result without reading or parsing the source.
    let key = fs_err::metadata(&abs)
        .ok()
        .map(|m| crate::cache::file_key(&m));
    if let (Some(cache), Some(key)) = (cache, &key) {
        if let Some(hit) = cache.get(&abs, key) {
            return Ok(from_cache(hit, &abs, &rel));
        }
    }

    let source =
        fs_err::read_to_string(&abs).with_context(|| format!("cannot read {}", abs.display()))?;

    let collected = parse_file(&source, &abs, &rel);

    // Best-effort write-through: only when we have a cache and a freshness key.
    if let (Some(cache), Some(key)) = (cache, key) {
        cache.put(&abs, &to_cache(&collected, key));
    }
    Ok(collected)
}

/// Statically parse `source` into a [`CollectedFile`]. Unparseable files fall
/// back to a single dynamic item the Python worker discovers at import time.
fn parse_file(source: &str, abs: &Path, rel: &str) -> CollectedFile {
    let parsed = ast::Suite::parse(source, rel);
    let body = match parsed {
        Ok(b) => b,
        Err(e) => {
            // Unparseable: a real syntax error, or syntax newer than the parser
            // understands (e.g. PEP 701 same-quote f-strings). Fall back to
            // import-time discovery by the worker — still correct, just slower
            // and without statically-known parametrize ids. Surfaced under
            // TEZT_DEBUG so the (rare) degradation is observable.
            if std::env::var_os("TEZT_DEBUG").is_some() {
                eprintln!("tezt: {rel}: static parse failed ({e}); using import-time discovery");
            }
            return CollectedFile {
                items: vec![TestItem {
                    id: rel.to_string(),
                    file: abs.to_path_buf(),
                    qualname: "*".to_string(),
                    param_ids: None,
                }],
                file: abs.to_path_buf(),
                rel: rel.to_string(),
                dynamic: true,
            };
        }
    };

    let mut items = Vec::new();
    for stmt in &body {
        match stmt {
            ast::Stmt::FunctionDef(f) if f.name.as_str().starts_with("test") => {
                if is_test_name(f.name.as_str()) {
                    items.push(make_item(
                        rel,
                        abs,
                        f.name.as_str(),
                        None,
                        &f.decorator_list,
                    ));
                }
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                if is_test_name(f.name.as_str()) {
                    items.push(make_item(
                        rel,
                        abs,
                        f.name.as_str(),
                        None,
                        &f.decorator_list,
                    ));
                }
            }
            ast::Stmt::ClassDef(c) if c.name.as_str().starts_with("Test") => {
                if class_has_init(c) {
                    continue; // pytest skips Test* classes with __init__
                }
                for sub in &c.body {
                    match sub {
                        ast::Stmt::FunctionDef(m) if is_test_name(m.name.as_str()) => {
                            items.push(make_item(
                                rel,
                                abs,
                                m.name.as_str(),
                                Some(c.name.as_str()),
                                &m.decorator_list,
                            ));
                        }
                        ast::Stmt::AsyncFunctionDef(m) if is_test_name(m.name.as_str()) => {
                            items.push(make_item(
                                rel,
                                abs,
                                m.name.as_str(),
                                Some(c.name.as_str()),
                                &m.decorator_list,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    CollectedFile {
        file: abs.to_path_buf(),
        rel: rel.to_string(),
        items,
        dynamic: false,
    }
}

/// Reconstruct a [`CollectedFile`] from a cache hit. The absolute path and
/// `rel` id are not stored in the entry — they are re-derived from the file
/// being collected (the same path that produced the cache digest).
fn from_cache(hit: CachedCollection, abs: &Path, rel: &str) -> CollectedFile {
    let items = hit
        .items
        .into_iter()
        .map(|c| TestItem {
            id: c.id,
            file: abs.to_path_buf(),
            qualname: c.qualname,
            param_ids: c.param_ids,
        })
        .collect();
    CollectedFile {
        file: abs.to_path_buf(),
        rel: rel.to_string(),
        items,
        dynamic: hit.dynamic,
    }
}

/// Build a cache entry from a freshly collected file. The absolute path is
/// intentionally dropped (reconstructed on load).
fn to_cache(collected: &CollectedFile, key: crate::cache::FileCacheKey) -> CachedCollection {
    let items = collected
        .items
        .iter()
        .map(|i| CachedItem {
            id: i.id.clone(),
            qualname: i.qualname.clone(),
            param_ids: i.param_ids.clone(),
        })
        .collect();
    CachedCollection {
        key,
        items,
        dynamic: collected.dynamic,
    }
}

fn is_test_name(name: &str) -> bool {
    name == "test" || name.starts_with("test_")
}

fn class_has_init(c: &ast::StmtClassDef) -> bool {
    c.body.iter().any(|s| match s {
        ast::Stmt::FunctionDef(f) => f.name.as_str() == "__init__",
        _ => false,
    })
}

fn make_item(
    rel: &str,
    abs: &Path,
    func: &str,
    class: Option<&str>,
    decorators: &[ast::Expr],
) -> TestItem {
    let qualname = match class {
        Some(c) => format!("{c}::{func}"),
        None => func.to_string(),
    };
    let id = format!("{rel}::{qualname}");
    let param_ids = static_parametrize_ids(decorators);
    TestItem {
        id,
        file: abs.to_path_buf(),
        qualname,
        param_ids,
    }
}

// --- static parametrize expansion (best effort, literals only) -------------

/// Detect stacked `@<...>parametrize(argnames, argvalues, ids=...)` decorators
/// and predict the expanded case ids. Mirrors the worker's id rules:
/// str -> as-is, bool -> True/False, None -> None, numbers -> str(value),
/// other -> "p<N>"; multi-arg values joined with "-"; `ids=` overrides.
/// Stacked decorators form a cartesian product with the innermost
/// (bottom-most) decorator varying fastest.
fn static_parametrize_ids(decorators: &[ast::Expr]) -> Option<Vec<String>> {
    // Collect per-decorator case-id lists, top-to-bottom.
    let mut groups: Vec<Option<Vec<String>>> = Vec::new();
    for dec in decorators {
        if let Some(case_ids) = parametrize_case_ids(dec) {
            groups.push(case_ids);
        }
    }
    if groups.is_empty() {
        return None;
    }
    // Any group with unknown ids => give up on exact ids but keep "parametrized".
    if groups.iter().any(|g| g.is_none()) {
        return Some(Vec::new());
    }
    let groups: Vec<Vec<String>> = groups.into_iter().map(|g| g.unwrap()).collect();
    // Cartesian product. pytest composes ids as "<inner>-<outer>" with the
    // innermost (bottom) decorator varying fastest. Decorator list is
    // top-to-bottom, so the LAST group is innermost.
    let mut acc: Vec<Vec<String>> = vec![Vec::new()];
    // Iterate outermost -> innermost so inner ends up varying fastest:
    for g in &groups {
        let mut next = Vec::with_capacity(acc.len() * g.len());
        for prefix in &acc {
            for id in g {
                let mut v = prefix.clone();
                v.push(id.clone());
                next.push(v);
            }
        }
        acc = next;
    }
    // Combined id: innermost part first => reverse each combo.
    Some(
        acc.into_iter()
            .map(|mut parts| {
                parts.reverse();
                parts.join("-")
            })
            .collect(),
    )
}

/// If `dec` is a parametrize decorator call, return its case ids
/// (None inside the Option means "parametrized, ids unknown").
fn parametrize_case_ids(dec: &ast::Expr) -> Option<Option<Vec<String>>> {
    let ast::Expr::Call(call) = dec else {
        return None;
    };
    if !is_parametrize_callee(&call.func) {
        return None;
    }
    // ids= keyword override
    for kw in &call.keywords {
        if kw.arg.as_ref().map(|a| a.as_str()) == Some("ids") {
            if let Some(ids) = literal_str_list(&kw.value) {
                return Some(Some(ids));
            }
            return Some(None);
        }
    }
    if call.args.len() < 2 {
        return Some(None);
    }
    let values = &call.args[1];
    let elts = match values {
        ast::Expr::List(l) => &l.elts,
        ast::Expr::Tuple(t) => &t.elts,
        _ => return Some(None),
    };
    let mut ids = Vec::with_capacity(elts.len());
    for (i, e) in elts.iter().enumerate() {
        ids.push(value_case_id(e, i));
    }
    Some(Some(ids))
}

fn is_parametrize_callee(func: &ast::Expr) -> bool {
    // Matches `parametrize`, `tezt.parametrize`, `pytest.mark.parametrize`,
    // `tezt.mark.parametrize`, or any dotted path ending in `.parametrize`.
    match func {
        ast::Expr::Name(n) => n.id.as_str() == "parametrize",
        ast::Expr::Attribute(a) => a.attr.as_str() == "parametrize",
        _ => false,
    }
}

fn literal_str_list(expr: &ast::Expr) -> Option<Vec<String>> {
    let elts = match expr {
        ast::Expr::List(l) => &l.elts,
        ast::Expr::Tuple(t) => &t.elts,
        _ => return None,
    };
    let mut out = Vec::with_capacity(elts.len());
    for e in elts {
        match e {
            ast::Expr::Constant(c) => match &c.value {
                ast::Constant::Str(s) => out.push(s.clone()),
                _ => return None,
            },
            _ => return None,
        }
    }
    Some(out)
}

fn value_case_id(expr: &ast::Expr, index: usize) -> String {
    match expr {
        ast::Expr::Constant(c) => constant_case_id(&c.value, index),
        ast::Expr::Tuple(t) => {
            let mut parts = Vec::with_capacity(t.elts.len());
            for e in &t.elts {
                match e {
                    ast::Expr::Constant(c) => parts.push(constant_case_id(&c.value, index)),
                    _ => return format!("p{index}"),
                }
            }
            parts.join("-")
        }
        ast::Expr::UnaryOp(u) => {
            // Handle negative number literals like -1.
            if let ast::Expr::Constant(c) = u.operand.as_ref() {
                if matches!(u.op, ast::UnaryOp::USub) {
                    return format!("-{}", constant_case_id(&c.value, index));
                }
            }
            format!("p{index}")
        }
        _ => format!("p{index}"),
    }
}

fn constant_case_id(c: &ast::Constant, index: usize) -> String {
    match c {
        ast::Constant::Str(s) => s.clone(),
        ast::Constant::Bool(b) => {
            if *b {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        ast::Constant::None => "None".to_string(),
        ast::Constant::Int(i) => i.to_string(),
        ast::Constant::Float(f) => format!("{f}"),
        _ => format!("p{index}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn collect_src(src: &str) -> CollectedFile {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_sample.py");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(src.as_bytes()).unwrap();
        collect_file(&path, dir.path(), None).unwrap()
    }

    #[test]
    fn collects_functions_and_classes() {
        let c = collect_src(
            r#"
def test_a():
    assert True

async def test_b():
    assert True

def helper():
    pass

class TestFoo:
    def test_m(self):
        assert True
    def not_a_test(self):
        pass

class TestWithInit:
    def __init__(self):
        pass
    def test_skipped(self):
        pass

class Helper:
    def test_nope(self):
        pass
"#,
        );
        let ids: Vec<_> = c.items.iter().map(|i| i.qualname.clone()).collect();
        assert_eq!(ids, vec!["test_a", "test_b", "TestFoo::test_m"]);
        assert!(!c.dynamic);
    }

    #[test]
    fn syntax_error_goes_dynamic() {
        let c = collect_src("def test_broken(:\n    pass\n");
        assert!(c.dynamic);
        assert_eq!(c.items.len(), 1);
        assert_eq!(c.items[0].qualname, "*");
    }

    #[test]
    fn static_parametrize_simple() {
        let c = collect_src(
            r#"
import tezt

@tezt.parametrize("x", [1, 2, 3])
def test_p(x):
    assert x
"#,
        );
        let ids = c.items[0].param_ids.clone().unwrap();
        assert_eq!(ids, vec!["1", "2", "3"]);
        assert_eq!(c.items[0].expected_cases(), 3);
    }

    #[test]
    fn static_parametrize_tuples_and_ids() {
        let c = collect_src(
            r#"
import pytest

@pytest.mark.parametrize("a,b", [(1, 2), (3, 4)])
def test_t(a, b):
    assert a < b

@pytest.mark.parametrize("v", ["x", "y"], ids=["first", "second"])
def test_ids(v):
    assert v
"#,
        );
        assert_eq!(c.items[0].param_ids.clone().unwrap(), vec!["1-2", "3-4"]);
        assert_eq!(
            c.items[1].param_ids.clone().unwrap(),
            vec!["first", "second"]
        );
    }

    #[test]
    fn static_parametrize_cartesian() {
        let c = collect_src(
            r#"
import tezt

@tezt.parametrize("x", [0, 1])
@tezt.parametrize("y", [2, 3, 4])
def test_c(x, y):
    assert True
"#,
        );
        assert_eq!(c.items[0].expected_cases(), 6);
    }

    #[test]
    fn unknown_parametrize_values() {
        let c = collect_src(
            r#"
import tezt
DATA = [1, 2]

@tezt.parametrize("x", DATA)
def test_d(x):
    assert x
"#,
        );
        assert_eq!(c.items[0].param_ids, Some(Vec::new()));
        assert_eq!(c.items[0].expected_cases(), 1);
    }

    // The collector must keep its fast (static) path on modern Python syntax;
    // anything it can't parse silently degrades to import-time discovery, so we
    // pin the boundary with tests. (These assert the *parser's* reach and are
    // independent of the Python version actually running the tests.)

    #[test]
    fn collects_match_and_except_star() {
        // 3.10 `match` and 3.11 `except*` inside test bodies parse statically.
        let c = collect_src(concat!(
            "def test_match(v=1):\n",
            "    match v:\n",
            "        case 1:\n",
            "            assert True\n",
            "        case _:\n",
            "            assert False\n",
            "\n",
            "def test_eg():\n",
            "    try:\n",
            "        raise ValueError\n",
            "    except* ValueError:\n",
            "        assert True\n",
        ));
        assert!(!c.dynamic, "match/except* should parse statically");
        let ids: Vec<_> = c.items.iter().map(|i| i.qualname.clone()).collect();
        assert_eq!(ids, vec!["test_match", "test_eg"]);
    }

    #[test]
    fn collects_pep695_type_params_and_alias() {
        // 3.12 PEP 695 type parameters and `type` aliases parse statically.
        let c = collect_src(concat!(
            "type IntList = list[int]\n",
            "\n",
            "def test_generic[T](x=None):\n",
            "    assert True\n",
            "\n",
            "class TestBox[T]:\n",
            "    def test_inner(self):\n",
            "        assert True\n",
        ));
        assert!(!c.dynamic, "PEP 695 syntax should parse statically");
        let ids: Vec<_> = c.items.iter().map(|i| i.qualname.clone()).collect();
        assert!(ids.contains(&"test_generic".to_string()), "got {ids:?}");
        assert!(
            ids.contains(&"TestBox::test_inner".to_string()),
            "got {ids:?}"
        );
    }

    #[test]
    fn pep701_fstring_falls_back_gracefully() {
        // The lone modern-syntax gap in rustpython-parser 0.4 is PEP 701
        // same-quote nested f-strings. tezt must still collect the file (via
        // import-time discovery), never crash. Whether it parses statically or
        // falls back, the file stays collectable — that's the invariant.
        let c = collect_src("def test_fstr():\n    assert f\"{\"x\"}\" == \"x\"\n");
        if c.dynamic {
            assert_eq!(c.items.len(), 1);
            assert_eq!(c.items[0].qualname, "*");
        } else {
            assert!(c.items.iter().any(|i| i.qualname == "test_fstr"));
        }
    }
}
