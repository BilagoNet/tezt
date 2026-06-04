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
    /// Mark names attached to this test (from `@pytest.mark.X` / `@tezt.mark.X`
    /// decorators plus module- and class-level `pytestmark`/`teztmark`),
    /// deduplicated. Used by `-m` selection. `parametrize` is intentionally
    /// excluded (it is structural, not a selection mark). Empty for items from
    /// files that could not be parsed statically.
    pub marks: Vec<String>,
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
    // `dunce::canonicalize` resolves symlinks like `std::fs::canonicalize` but,
    // on Windows, strips the `\\?\` verbatim prefix. Without that, the absolute
    // path handed to the worker (`\\?\D:\...`) and the rootdir it compares
    // against (`D:\...`) look like different mounts, and `os.path.relpath`
    // raises `ValueError: path is on mount ...`.
    let abs =
        dunce::canonicalize(file).with_context(|| format!("cannot resolve {}", file.display()))?;
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
                    // No AST to read marks from; `-m` can never select a file we
                    // could not parse. (`--lf`/`--ff` still match it by path.)
                    marks: Vec::new(),
                }],
                file: abs.to_path_buf(),
                rel: rel.to_string(),
                dynamic: true,
            };
        }
    };

    // Module-level `pytestmark`/`teztmark` applies to every test in the file.
    // Computed once; folded into each item's marks below.
    let module_marks = pytestmark_assignments(&body);

    let mut items = Vec::new();
    for stmt in &body {
        match stmt {
            ast::Stmt::FunctionDef(f) if f.name.as_str().starts_with("test") => {
                if is_test_name(f.name.as_str()) {
                    let marks = combine_marks(decorator_marks(&f.decorator_list), &module_marks);
                    items.push(make_item(
                        rel,
                        abs,
                        f.name.as_str(),
                        None,
                        &f.decorator_list,
                        marks,
                    ));
                }
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                if is_test_name(f.name.as_str()) {
                    let marks = combine_marks(decorator_marks(&f.decorator_list), &module_marks);
                    items.push(make_item(
                        rel,
                        abs,
                        f.name.as_str(),
                        None,
                        &f.decorator_list,
                        marks,
                    ));
                }
            }
            ast::Stmt::ClassDef(c) if c.name.as_str().starts_with("Test") => {
                if class_has_init(c) {
                    continue; // pytest skips Test* classes with __init__
                }
                // Class-level marks = class decorators + class-body
                // `pytestmark`/`teztmark` + the module-level marks. Each method
                // inherits these and adds its own decorator marks.
                let mut class_marks = decorator_marks(&c.decorator_list);
                push_unique(&mut class_marks, pytestmark_assignments(&c.body));
                push_unique(&mut class_marks, module_marks.clone());
                for sub in &c.body {
                    match sub {
                        ast::Stmt::FunctionDef(m) if is_test_name(m.name.as_str()) => {
                            let marks =
                                combine_marks(decorator_marks(&m.decorator_list), &class_marks);
                            items.push(make_item(
                                rel,
                                abs,
                                m.name.as_str(),
                                Some(c.name.as_str()),
                                &m.decorator_list,
                                marks,
                            ));
                        }
                        ast::Stmt::AsyncFunctionDef(m) if is_test_name(m.name.as_str()) => {
                            let marks =
                                combine_marks(decorator_marks(&m.decorator_list), &class_marks);
                            items.push(make_item(
                                rel,
                                abs,
                                m.name.as_str(),
                                Some(c.name.as_str()),
                                &m.decorator_list,
                                marks,
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
            marks: c.marks,
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
            marks: i.marks.clone(),
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
    marks: Vec<String>,
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
        marks,
    }
}

// --- static mark extraction (best effort) -----------------------------------
//
// pytest's `-m` selects on *marks*. We collect them statically from the AST —
// no Python import — from three sources, matching pytest's own precedence-free
// union: `@pytest.mark.X` / `@tezt.mark.X` decorators on the test, and
// `pytestmark`/`teztmark` assignments at module and class scope. `parametrize`
// is filtered out everywhere: it is a structural decorator, not a selection
// mark (pytest does not let you `-m parametrize`).

/// Extract the mark name from a decorator expression: matches
/// `<anything>.mark.NAME`, `<anything>.mark.NAME(...)`, or `mark.NAME` /
/// `mark.NAME(...)`. Returns None for non-mark decorators. `parametrize` is
/// filtered out by the caller.
fn decorator_mark_name(dec: &ast::Expr) -> Option<String> {
    // A bare `@pytest.mark.slow` is an Attribute; `@pytest.mark.slow(...)` is a
    // Call whose `func` is that Attribute. Unwrap the call to its callee first.
    let callee = if let ast::Expr::Call(c) = dec {
        c.func.as_ref()
    } else {
        dec
    };
    // We want the final `.NAME` of a `....mark.NAME` chain. So `callee` must be
    // an Attribute (`.NAME`) whose `.value` is itself the `....mark` part.
    if let ast::Expr::Attribute(a) = callee {
        // The thing being attribute-accessed must be `mark`: either the tail of
        // a dotted path (`pytest.mark` => Attribute with attr == "mark") or a
        // bare imported `mark` name (`from pytest import mark` => Name "mark").
        let is_mark = match a.value.as_ref() {
            ast::Expr::Attribute(inner) => inner.attr.as_str() == "mark",
            ast::Expr::Name(n) => n.id.as_str() == "mark",
            _ => false,
        };
        if is_mark {
            return Some(a.attr.to_string());
        }
    }
    None
}

/// All mark names on a decorator list (excludes `parametrize`).
fn decorator_marks(decorators: &[ast::Expr]) -> Vec<String> {
    decorators
        .iter()
        .filter_map(decorator_mark_name)
        .filter(|m| m != "parametrize")
        .collect()
}

/// Mark names from a `pytestmark`/`teztmark` assignment value: a single mark
/// expr (`pytest.mark.X` / `pytest.mark.X(...)`), or a list/tuple of them.
fn mark_names_from_value(expr: &ast::Expr) -> Vec<String> {
    let names: Vec<String> = match expr {
        // `pytestmark = [pytest.mark.a, pytest.mark.b]` (or a tuple).
        ast::Expr::List(l) => l.elts.iter().filter_map(decorator_mark_name).collect(),
        ast::Expr::Tuple(t) => t.elts.iter().filter_map(decorator_mark_name).collect(),
        // `pytestmark = pytest.mark.a` — a single mark.
        other => decorator_mark_name(other).into_iter().collect(),
    };
    names.into_iter().filter(|m| m != "parametrize").collect()
}

/// Scan a statement body for top-of-scope `pytestmark`/`teztmark = ...`
/// assignments and return the mark names they contribute. Handles both
/// `Assign` (targets contain a `Name`) and `AnnAssign` (target is a `Name`).
fn pytestmark_assignments(body: &[ast::Stmt]) -> Vec<String> {
    let mut marks = Vec::new();
    for stmt in body {
        match stmt {
            ast::Stmt::Assign(a) => {
                let is_pytestmark = a.targets.iter().any(is_pytestmark_name);
                if is_pytestmark {
                    push_unique(&mut marks, mark_names_from_value(&a.value));
                }
            }
            ast::Stmt::AnnAssign(a) => {
                if is_pytestmark_name(&a.target) {
                    if let Some(v) = &a.value {
                        push_unique(&mut marks, mark_names_from_value(v));
                    }
                }
            }
            _ => {}
        }
    }
    marks
}

/// Is `expr` a `Name` equal to `pytestmark` or `teztmark`?
fn is_pytestmark_name(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::Name(n) if n.id.as_str() == "pytestmark" || n.id.as_str() == "teztmark")
}

/// Append `extra` onto `marks`, skipping any name already present
/// (order-preserving dedup — first occurrence wins, so a test's own decorator
/// marks keep their position ahead of inherited ones).
fn push_unique(marks: &mut Vec<String>, extra: Vec<String>) {
    for m in extra {
        if !marks.contains(&m) {
            marks.push(m);
        }
    }
}

/// Combine a test's own (decorator) marks with inherited (class/module) marks,
/// deduplicated and preserving first-seen order.
fn combine_marks(mut own: Vec<String>, inherited: &[String]) -> Vec<String> {
    push_unique(&mut own, inherited.to_vec());
    own
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

    /// Look up a collected item's marks by qualname (test convenience).
    fn marks_of<'a>(c: &'a CollectedFile, qualname: &str) -> &'a [String] {
        &c.items
            .iter()
            .find(|i| i.qualname == qualname)
            .unwrap_or_else(|| panic!("no item {qualname}; have {:?}", c.items))
            .marks
    }

    #[test]
    fn marks_from_decorators() {
        // `@pytest.mark.X`, `@tezt.mark.X`, and the call form all contribute;
        // `parametrize` never does.
        let c = collect_src(
            r#"
import pytest, tezt

@pytest.mark.slow
@tezt.mark.network
@pytest.mark.timeout(5)
@pytest.mark.parametrize("x", [1, 2])
def test_a(x):
    assert x

def test_plain():
    assert True
"#,
        );
        assert_eq!(marks_of(&c, "test_a"), &["slow", "network", "timeout"]);
        assert!(marks_of(&c, "test_plain").is_empty());
    }

    #[test]
    fn marks_from_module_level_pytestmark() {
        // Module-level `pytestmark` (single, list, and the `teztmark` alias)
        // applies to every test; an annotated assignment works too.
        let c = collect_src(
            r#"
import pytest

pytestmark = [pytest.mark.slow, pytest.mark.network]

def test_a():
    assert True

@pytest.mark.extra
def test_b():
    assert True
"#,
        );
        assert_eq!(marks_of(&c, "test_a"), &["slow", "network"]);
        // Own decorator marks come first, then inherited module marks.
        assert_eq!(marks_of(&c, "test_b"), &["extra", "slow", "network"]);

        let single = collect_src(
            r#"
import tezt
teztmark = tezt.mark.slow
def test_x():
    assert True
"#,
        );
        assert_eq!(marks_of(&single, "test_x"), &["slow"]);

        let annotated = collect_src(
            r#"
import pytest
pytestmark: list = pytest.mark.slow
def test_y():
    assert True
"#,
        );
        assert_eq!(marks_of(&annotated, "test_y"), &["slow"]);
    }

    #[test]
    fn marks_from_class_level() {
        // Class decorators + class-body `pytestmark` + module marks all flow
        // down to methods, combined with each method's own decorator marks and
        // deduplicated (no duplicate `slow`).
        let c = collect_src(
            r#"
import pytest

pytestmark = pytest.mark.modwide

@pytest.mark.slow
class TestThing:
    pytestmark = [pytest.mark.classwide, pytest.mark.slow]

    @pytest.mark.fast
    def test_m(self):
        assert True

    def test_n(self):
        assert True
"#,
        );
        // test_m: own (fast) + class decorator (slow) + class body
        // (classwide, slow-deduped) + module (modwide).
        assert_eq!(
            marks_of(&c, "TestThing::test_m"),
            &["fast", "slow", "classwide", "modwide"]
        );
        assert_eq!(
            marks_of(&c, "TestThing::test_n"),
            &["slow", "classwide", "modwide"]
        );
    }

    #[test]
    fn dynamic_file_has_empty_marks() {
        // An unparseable file degrades to one dynamic item with no marks.
        let c = collect_src("def test_broken(:\n    pass\n");
        assert!(c.dynamic);
        assert!(c.items[0].marks.is_empty());
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
