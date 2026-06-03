//! Persistent collection cache.
//!
//! Collection's dominant cost is parsing every test file with
//! `rustpython-parser`. On a warm run almost nothing has changed, so re-parsing
//! is wasted work. This module caches each file's collection result on disk and
//! reuses it when the file is unchanged.
//!
//! Design (modeled on uv's `uv-cache` / `uv-cache-info`):
//!
//! * **Location** — `<rootdir>/.tezt_cache/` (project-local, like pytest's
//!   `.pytest_cache`). On first use we drop a `CACHEDIR.TAG` and a `.gitignore`
//!   so the directory is recognised as a cache and never committed.
//! * **One entry per file** — `collect-v1/<ab>/<digest>.json`, where `<digest>`
//!   is an FNV-1a hash of the file's canonical absolute path and `<ab>` is the
//!   first two hex chars (sharding so no single directory grows huge). The
//!   `collect-v1` segment is the on-disk-format kill switch: bump it whenever
//!   [`CachedCollection`] changes shape.
//! * **Freshness via struct equality** (uv's trick) — each entry stores a
//!   [`FileCacheKey`] of `(size, mtime_ns, tezt_version)`. On read, if the
//!   stored key equals the file's current key it is a HIT and we reconstruct
//!   the result without touching the source. Otherwise it is a MISS. `mtime` +
//!   `size` is the same freshness signal pytest uses; we deliberately do not
//!   hash file contents on the hot path.
//! * **Atomic, lock-free writes** — entries are written to a `NamedTempFile` in
//!   the shard directory and then atomically `persist`ed (same-dir rename).
//!   Because every file owns a distinct entry, the parallel collector writes
//!   without any locking. Any IO/parse error on read or write is treated as a
//!   cache miss — the cache must never fail a run.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// On-disk format version. Bump when [`CachedCollection`] changes shape.
const COLLECT_VERSION: &str = "collect-v1";

/// Standard cache-directory tag content (see <https://bford.info/cachedir/>).
const CACHEDIR_TAG: &str = "Signature: 8a477f597d28d172789f06886806bc55\n\
# This file is a cache directory tag created by tezt.\n\
# For information about cache directory tags see https://bford.info/cachedir/\n";

/// Freshness signal for a single source file.
///
/// Two keys are equal iff the file is considered unchanged. Equality (rather
/// than e.g. a timestamp comparison) is the freshness check itself, so a
/// `tezt` upgrade — which bumps `tezt_version` — invalidates every entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileCacheKey {
    /// File size in bytes.
    pub size: u64,
    /// Modification time in nanoseconds since the Unix epoch.
    pub mtime_ns: i128,
    /// `tezt` version that produced the entry (`CARGO_PKG_VERSION`).
    pub tezt_version: String,
}

/// A cached test item. Mirrors [`crate::collect::TestItem`] minus the absolute
/// file path, which is reconstructed on load (the same path that produced the
/// cache digest).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedItem {
    pub id: String,
    pub qualname: String,
    pub param_ids: Option<Vec<String>>,
}

/// A cached collection result for one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedCollection {
    /// Freshness key the entry was written with.
    pub key: FileCacheKey,
    pub items: Vec<CachedItem>,
    pub dynamic: bool,
}

/// Build the freshness key for a file from its metadata.
pub fn file_key(meta: &std::fs::Metadata) -> FileCacheKey {
    // A file with no readable mtime is treated as "epoch"; combined with size
    // this still changes whenever the file is rewritten with different content,
    // and a stale-but-equal key only ever costs us a needless reparse on miss.
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i128)
        .unwrap_or(0);
    FileCacheKey {
        size: meta.len(),
        mtime_ns,
        tezt_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Tiny FNV-1a 64-bit hash, formatted as 16 lowercase hex chars.
///
/// Used only to derive a stable on-disk filename from the canonical absolute
/// path — it is not security sensitive, just needs to be deterministic across
/// runs (which `DefaultHasher` is not guaranteed to be).
fn fnv1a_hex(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

/// The persistent collection cache.
///
/// Construct with [`Cache::new`] (enabled, project-local) or [`Cache::disabled`]
/// (every `get` misses, every `put` is a no-op). `&Cache` is `Sync`, so it can
/// be shared across the rayon collection workers.
#[derive(Debug)]
pub struct Cache {
    /// Root of the versioned entry tree: `<rootdir>/.tezt_cache/collect-v1`.
    /// `None` when the cache is disabled.
    collect_root: Option<PathBuf>,
}

impl Cache {
    /// Create a cache rooted at `<rootdir>/.tezt_cache`.
    ///
    /// When `enabled`, the cache directory, its `CACHEDIR.TAG`, and an internal
    /// `.gitignore` are created best-effort. If that setup fails (e.g. a
    /// read-only filesystem) the cache silently degrades to disabled rather
    /// than failing the run. `enabled == false` yields a disabled cache.
    pub fn new(rootdir: &Path, enabled: bool) -> Self {
        if !enabled {
            return Self::disabled();
        }
        let cache_dir = rootdir.join(".tezt_cache");
        match init_cache_dir(&cache_dir) {
            Ok(()) => Self {
                collect_root: Some(cache_dir.join(COLLECT_VERSION)),
            },
            Err(_) => Self::disabled(),
        }
    }

    /// A no-op cache: `get` always misses, `put` never writes.
    pub fn disabled() -> Self {
        Self { collect_root: None }
    }

    /// Absolute path of the on-disk entry for `abs` (canonical source path).
    fn entry_path(&self, abs: &Path) -> Option<PathBuf> {
        let root = self.collect_root.as_ref()?;
        let digest = fnv1a_hex(abs.to_string_lossy().as_bytes());
        let shard = &digest[..2];
        Some(root.join(shard).join(format!("{digest}.json")))
    }

    /// Look up a fresh cache entry for `abs`.
    ///
    /// Returns `Some` only on a HIT: the entry exists, deserializes, and its
    /// stored key equals `key`. Missing, corrupt, or stale entries return
    /// `None` (a cache miss never surfaces an error).
    pub fn get(&self, abs: &Path, key: &FileCacheKey) -> Option<CachedCollection> {
        let path = self.entry_path(abs)?;
        let bytes = fs_err::read(&path).ok()?;
        let entry: CachedCollection = serde_json::from_slice(&bytes).ok()?;
        if &entry.key == key {
            Some(entry)
        } else {
            None
        }
    }

    /// Write `entry` for `abs`, best-effort.
    ///
    /// Creates the shard directory if needed, then writes atomically via a
    /// same-directory tempfile + rename so concurrent workers (each writing a
    /// distinct file) need no locking. All errors are ignored.
    pub fn put(&self, abs: &Path, entry: &CachedCollection) {
        let Some(path) = self.entry_path(abs) else {
            return;
        };
        let _ = write_entry(&path, entry);
    }

    /// Remove the entire `<rootdir>/.tezt_cache` directory.
    ///
    /// Returns `Ok(())` if the directory is gone afterwards (including when it
    /// never existed).
    pub fn clear(rootdir: &Path) -> std::io::Result<()> {
        let cache_dir = rootdir.join(".tezt_cache");
        match fs_err::remove_dir_all(&cache_dir) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// Create the cache directory and drop the `CACHEDIR.TAG` + `.gitignore`
/// marker files (best-effort: only the directory itself is required).
fn init_cache_dir(cache_dir: &Path) -> std::io::Result<()> {
    fs_err::create_dir_all(cache_dir)?;
    // Marker files are nice-to-have; don't fail setup if they can't be written.
    let tag = cache_dir.join("CACHEDIR.TAG");
    if !tag.exists() {
        let _ = fs_err::write(&tag, CACHEDIR_TAG);
    }
    let gitignore = cache_dir.join(".gitignore");
    if !gitignore.exists() {
        let _ = fs_err::write(&gitignore, "*\n");
    }
    Ok(())
}

/// Serialize `entry` and write it atomically to `path` (tempfile + rename).
fn write_entry(path: &Path, entry: &CachedCollection) -> std::io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "entry has no parent"))?;
    fs_err::create_dir_all(dir)?;
    let bytes = serde_json::to_vec(entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    // Same-directory tempfile so `persist` is an atomic rename, not a copy.
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    {
        use std::io::Write as _;
        tmp.write_all(&bytes)?;
        tmp.flush()?;
    }
    tmp.persist(path)
        .map_err(|e| e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    /// Write `src` to `dir/name` and return (path, freshness key).
    fn write_file(dir: &Path, name: &str, src: &str) -> (PathBuf, FileCacheKey) {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(src.as_bytes()).unwrap();
        f.flush().unwrap();
        let meta = std::fs::metadata(&path).unwrap();
        (path, file_key(&meta))
    }

    fn sample_entry(key: FileCacheKey) -> CachedCollection {
        CachedCollection {
            key,
            items: vec![
                CachedItem {
                    id: "test_a.py::test_one".into(),
                    qualname: "test_one".into(),
                    param_ids: None,
                },
                CachedItem {
                    id: "test_a.py::TestC::test_two".into(),
                    qualname: "TestC::test_two".into(),
                    param_ids: Some(vec!["1".into(), "2".into()]),
                },
            ],
            dynamic: false,
        }
    }

    #[test]
    fn cold_miss_then_warm_hit() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let (path, key) = write_file(dir.path(), "test_a.py", "def test_one(): pass\n");

        // Cold: nothing cached yet.
        assert!(cache.get(&path, &key).is_none());

        // Populate, then a warm get with the same key is a hit with identical
        // contents.
        let entry = sample_entry(key.clone());
        cache.put(&path, &entry);
        let got = cache.get(&path, &key).expect("warm hit");
        assert_eq!(got, entry);
        assert_eq!(got.items.len(), 2);
        assert!(!got.dynamic);
    }

    #[test]
    fn size_change_invalidates() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let (path, key) = write_file(dir.path(), "test_b.py", "def test_x(): pass\n");
        cache.put(&path, &sample_entry(key.clone()));

        // Same path, different size => miss.
        let changed = FileCacheKey {
            size: key.size + 1,
            ..key.clone()
        };
        assert!(cache.get(&path, &changed).is_none());
        // Original key still hits.
        assert!(cache.get(&path, &key).is_some());
    }

    #[test]
    fn mtime_change_invalidates() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let (path, key) = write_file(dir.path(), "test_c.py", "def test_y(): pass\n");
        cache.put(&path, &sample_entry(key.clone()));

        let changed = FileCacheKey {
            mtime_ns: key.mtime_ns + 1,
            ..key.clone()
        };
        assert!(cache.get(&path, &changed).is_none());
    }

    #[test]
    fn version_change_invalidates() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let (path, key) = write_file(dir.path(), "test_v.py", "def test_z(): pass\n");
        cache.put(&path, &sample_entry(key.clone()));

        let changed = FileCacheKey {
            tezt_version: "0.0.0-other".into(),
            ..key.clone()
        };
        assert!(cache.get(&path, &changed).is_none());
    }

    #[test]
    fn disabled_cache_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::disabled();
        let (path, key) = write_file(dir.path(), "test_d.py", "def test_w(): pass\n");

        cache.put(&path, &sample_entry(key.clone()));
        assert!(cache.get(&path, &key).is_none());
        // A disabled cache writes nothing: no .tezt_cache directory appears.
        assert!(!dir.path().join(".tezt_cache").exists());
    }

    #[test]
    fn new_creates_marker_files() {
        let dir = tempfile::tempdir().unwrap();
        let _ = Cache::new(dir.path(), true);
        let cache_dir = dir.path().join(".tezt_cache");
        assert!(cache_dir.join("CACHEDIR.TAG").exists());
        assert!(cache_dir.join(".gitignore").exists());
        let tag = std::fs::read_to_string(cache_dir.join("CACHEDIR.TAG")).unwrap();
        assert!(tag.starts_with("Signature: 8a477f597d28d172789f06886806bc55"));
        let gi = std::fs::read_to_string(cache_dir.join(".gitignore")).unwrap();
        assert_eq!(gi, "*\n");
    }

    #[test]
    fn clear_removes_cache_dir() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let (path, key) = write_file(dir.path(), "test_e.py", "def test_q(): pass\n");
        cache.put(&path, &sample_entry(key));
        assert!(dir.path().join(".tezt_cache").exists());

        Cache::clear(dir.path()).unwrap();
        assert!(!dir.path().join(".tezt_cache").exists());
        // Clearing a non-existent cache is fine.
        Cache::clear(dir.path()).unwrap();
    }

    #[test]
    fn corrupt_entry_is_a_miss() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let (path, key) = write_file(dir.path(), "test_f.py", "def test_r(): pass\n");
        // Write garbage where the entry would live.
        let entry_path = cache.entry_path(&path).unwrap();
        fs_err::create_dir_all(entry_path.parent().unwrap()).unwrap();
        fs_err::write(&entry_path, b"not json at all").unwrap();
        assert!(cache.get(&path, &key).is_none());
    }

    #[test]
    fn entry_path_is_sharded_and_stable() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path(), true);
        let abs = dir.path().join("some/test_g.py");
        let p1 = cache.entry_path(&abs).unwrap();
        let p2 = cache.entry_path(&abs).unwrap();
        assert_eq!(p1, p2, "digest must be stable for the same path");

        let file_name = p1.file_name().unwrap().to_str().unwrap();
        assert!(file_name.ends_with(".json"));
        let digest = file_name.trim_end_matches(".json");
        assert_eq!(digest.len(), 16, "FNV-1a 64-bit => 16 hex chars");
        // Shard dir is the first two hex chars of the digest.
        let shard = p1.parent().unwrap().file_name().unwrap().to_str().unwrap();
        assert_eq!(shard, &digest[..2]);
        // Lives under the versioned root.
        assert!(p1.to_string_lossy().contains(COLLECT_VERSION));
    }

    #[test]
    fn cached_item_round_trips_via_json() {
        // Round-trip the CachedItem <-> TestItem mapping shape through serde.
        let item = CachedItem {
            id: "testdata/test_h.py::TestK::test_s".into(),
            qualname: "TestK::test_s".into(),
            param_ids: Some(vec!["a".into(), "b-c".into()]),
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: CachedItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, back);
    }
}
