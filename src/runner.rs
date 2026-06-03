//! Test execution: a pool of persistent Python worker processes fed over a
//! JSON-lines stdio protocol. Workers import each test module once, cache
//! fixture plans, and stream results back as they finish.

use crate::collect::TestItem;
use anyhow::{Context, Result};
use crossbeam_channel as channel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// The embedded Python worker, shipped inside the tezt binary.
pub const WORKER_SOURCE: &str = include_str!("../python/tezt_worker.py");

const BATCH_CHUNK: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Passed,
    Failed,
    Skipped,
    Xfailed,
    Xpassed,
    Error,
}

impl Outcome {
    pub fn is_bad(self) -> bool {
        matches!(self, Outcome::Failed | Outcome::Error)
    }
    pub fn label(self) -> &'static str {
        match self {
            Outcome::Passed => "passed",
            Outcome::Failed => "failed",
            Outcome::Skipped => "skipped",
            Outcome::Xfailed => "xfailed",
            Outcome::Xpassed => "xpassed",
            Outcome::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub id: String,
    pub outcome: Outcome,
    pub duration_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceback: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub stdout: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub stderr: String,
}

// --- wire types -------------------------------------------------------------

#[derive(Serialize)]
struct WireItem<'a> {
    id: &'a str,
    file: &'a str,
    qualname: &'a str,
}

#[derive(Serialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
enum WireCmd<'a> {
    Run {
        batch_id: u64,
        items: Vec<WireItem<'a>>,
    },
    Shutdown,
}

#[derive(Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum WireEvent {
    Ready {
        #[allow(dead_code)]
        pid: i64,
        #[allow(dead_code)]
        python: String,
    },
    Result {
        #[allow(dead_code)]
        batch_id: u64,
        id: String,
        outcome: Outcome,
        duration_ms: f64,
        message: Option<String>,
        traceback: Option<String>,
        #[serde(default)]
        stdout: String,
        #[serde(default)]
        stderr: String,
    },
    BatchDone {
        #[allow(dead_code)]
        batch_id: u64,
    },
    Bye,
    Fatal {
        message: String,
        traceback: Option<String>,
    },
}

// --- batches ----------------------------------------------------------------

#[derive(Debug)]
struct Batch {
    id: u64,
    items: Vec<TestItem>,
}

/// Group items by file and chunk into batches. File-grouping preserves
/// module/fixture locality inside a single worker.
fn make_batches(items: Vec<TestItem>) -> Vec<Batch> {
    let mut by_file: Vec<(PathBuf, Vec<TestItem>)> = Vec::new();
    for item in items {
        match by_file.last_mut() {
            Some((f, v)) if *f == item.file => v.push(item),
            _ => by_file.push((item.file.clone(), vec![item])),
        }
    }
    // Schedule larger files first: better tail latency on uneven suites.
    by_file.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    let mut batches = Vec::new();
    let mut next_id = 0u64;
    for (_, file_items) in by_file {
        for chunk in file_items.chunks(BATCH_CHUNK) {
            batches.push(Batch {
                id: next_id,
                items: chunk.to_vec(),
            });
            next_id += 1;
        }
    }
    batches
}

// --- worker process ---------------------------------------------------------

struct Worker {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl Worker {
    fn spawn(python: &str, shim: &Path, rootdir: &Path, no_capture: bool) -> Result<Self> {
        let mut cmd = Command::new(python);
        cmd.arg("-u")
            .arg(shim)
            .arg("--rootdir")
            .arg(rootdir)
            .current_dir(rootdir)
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if no_capture {
            cmd.arg("--no-capture");
        }
        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to start Python worker (`{python}`)"))?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let mut w = Worker {
            child,
            stdin,
            stdout,
        };
        // Handshake.
        match w.read_event()? {
            Some(WireEvent::Ready { .. }) => Ok(w),
            Some(WireEvent::Fatal { message, .. }) => {
                anyhow::bail!("worker failed to start: {message}")
            }
            _ => anyhow::bail!("worker did not send ready handshake"),
        }
    }

    fn read_event(&mut self) -> Result<Option<WireEvent>> {
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line)?;
            if n == 0 {
                return Ok(None); // EOF: worker died
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<WireEvent>(trimmed) {
                Ok(ev) => return Ok(Some(ev)),
                Err(_) => {
                    // Stray output on protocol stream; ignore defensively.
                    continue;
                }
            }
        }
    }

    fn send(&mut self, cmd: &WireCmd) -> Result<()> {
        let mut buf = serde_json::to_string(cmd)?;
        buf.push('\n');
        self.stdin.write_all(buf.as_bytes())?;
        self.stdin.flush()?;
        Ok(())
    }

    fn shutdown(mut self) {
        let _ = self.send(&WireCmd::Shutdown);
        // Give the worker a moment to run session teardowns, then reap.
        let _ = self.read_until_bye();
        let _ = self.child.wait();
    }

    fn read_until_bye(&mut self) -> Result<()> {
        for _ in 0..1000 {
            match self.read_event()? {
                Some(WireEvent::Bye) | None => return Ok(()),
                _ => continue,
            }
        }
        Ok(())
    }

    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// --- pool -------------------------------------------------------------------

pub struct RunConfig {
    pub python: String,
    pub rootdir: PathBuf,
    pub jobs: usize,
    pub no_capture: bool,
    pub maxfail: Option<usize>,
}

pub struct RunOutput {
    pub results: Vec<TestResult>,
    /// True if the run stopped early because of --maxfail / -x.
    pub stopped_early: bool,
    pub wall_time_s: f64,
}

/// Write the embedded worker source to a private temp file, once per run.
fn materialize_worker_shim() -> Result<PathBuf> {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("tezt_worker_{}.py", std::process::id()));
    std::fs::write(&path, WORKER_SOURCE)?;
    Ok(path)
}

/// Run all items on a pool of persistent Python workers, streaming each
/// result to `on_result` as it arrives.
pub fn run_tests<F>(items: Vec<TestItem>, cfg: &RunConfig, mut on_result: F) -> Result<RunOutput>
where
    F: FnMut(&TestResult),
{
    let started = Instant::now();
    let batches = make_batches(items);
    let n_batches = batches.len();
    if n_batches == 0 {
        return Ok(RunOutput {
            results: Vec::new(),
            stopped_early: false,
            wall_time_s: started.elapsed().as_secs_f64(),
        });
    }

    let shim = materialize_worker_shim()?;
    let workers = cfg.jobs.max(1).min(n_batches);

    let (batch_tx, batch_rx) = channel::unbounded::<Batch>();
    for b in batches {
        batch_tx.send(b).expect("queue send");
    }
    drop(batch_tx);

    let (res_tx, res_rx) = channel::unbounded::<TestResult>();
    let stop = Arc::new(AtomicBool::new(false));
    let bad_count = Arc::new(AtomicUsize::new(0));
    let maxfail = cfg.maxfail;

    let mut handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let batch_rx = batch_rx.clone();
        let res_tx = res_tx.clone();
        let stop = Arc::clone(&stop);
        let bad_count = Arc::clone(&bad_count);
        let python = cfg.python.clone();
        let shim = shim.clone();
        let rootdir = cfg.rootdir.clone();
        let no_capture = cfg.no_capture;

        handles.push(std::thread::spawn(move || -> Result<()> {
            let mut worker: Option<Worker> = None;
            while let Ok(batch) = batch_rx.recv() {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                if worker.is_none() {
                    worker = Some(Worker::spawn(&python, &shim, &rootdir, no_capture)?);
                }
                let w = worker.as_mut().unwrap();

                let wire_items: Vec<WireItem> = batch
                    .items
                    .iter()
                    .map(|i| WireItem {
                        id: &i.id,
                        file: i.file.to_str().unwrap_or_default(),
                        qualname: &i.qualname,
                    })
                    .collect();
                w.send(&WireCmd::Run {
                    batch_id: batch.id,
                    items: wire_items,
                })?;

                let mut seen: HashSet<String> = HashSet::new();
                let mut worker_died = false;
                loop {
                    match w.read_event()? {
                        Some(WireEvent::Result {
                            id,
                            outcome,
                            duration_ms,
                            message,
                            traceback,
                            stdout,
                            stderr,
                            ..
                        }) => {
                            // Track base ids ("foo[param]" -> "foo").
                            let base = id.split('[').next().unwrap_or(&id).to_string();
                            seen.insert(base);
                            if outcome.is_bad() {
                                bad_count.fetch_add(1, Ordering::SeqCst);
                            }
                            let _ = res_tx.send(TestResult {
                                id,
                                outcome,
                                duration_ms,
                                message,
                                traceback,
                                stdout,
                                stderr,
                            });
                            if let Some(mf) = maxfail {
                                if bad_count.load(Ordering::SeqCst) >= mf {
                                    stop.store(true, Ordering::SeqCst);
                                }
                            }
                        }
                        Some(WireEvent::BatchDone { .. }) => break,
                        Some(WireEvent::Fatal { message, traceback }) => {
                            report_missing(
                                &batch,
                                &seen,
                                &res_tx,
                                &format!("worker fatal error: {message}"),
                                traceback,
                            );
                            worker_died = true;
                            break;
                        }
                        Some(_) => continue,
                        None => {
                            report_missing(
                                &batch,
                                &seen,
                                &res_tx,
                                "worker process exited unexpectedly",
                                None,
                            );
                            worker_died = true;
                            break;
                        }
                    }
                }
                if worker_died {
                    if let Some(w) = worker.take() {
                        w.kill();
                    }
                }
                if stop.load(Ordering::SeqCst) {
                    break;
                }
            }
            if let Some(w) = worker.take() {
                if stop.load(Ordering::SeqCst) {
                    w.kill();
                } else {
                    w.shutdown();
                }
            }
            Ok(())
        }));
    }
    drop(res_tx);

    // Aggregate results on this thread, streaming to the reporter. When
    // --maxfail is hit we stop consuming entirely so the summary reflects
    // "stopped at failure N"; late in-flight results are dropped.
    let mut results = Vec::new();
    let mut bad_seen = 0usize;
    for r in res_rx.iter() {
        if r.outcome.is_bad() {
            bad_seen += 1;
        }
        on_result(&r);
        results.push(r);
        if let Some(mf) = maxfail {
            if bad_seen >= mf {
                stop.store(true, Ordering::SeqCst);
                break;
            }
        }
    }
    drop(res_rx);

    for h in handles {
        match h.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("tezt: worker thread error: {e}"),
            Err(_) => eprintln!("tezt: worker thread panicked"),
        }
    }
    let _ = std::fs::remove_file(&shim);

    Ok(RunOutput {
        stopped_early: stop.load(Ordering::SeqCst),
        results,
        wall_time_s: started.elapsed().as_secs_f64(),
    })
}

/// Emit `error` results for items of a batch that never produced a result
/// (worker crash mid-batch).
fn report_missing(
    batch: &Batch,
    seen: &HashSet<String>,
    res_tx: &channel::Sender<TestResult>,
    message: &str,
    traceback: Option<String>,
) {
    for item in &batch.items {
        if !seen.contains(&item.id) {
            let _ = res_tx.send(TestResult {
                id: item.id.clone(),
                outcome: Outcome::Error,
                duration_ms: 0.0,
                message: Some(message.to_string()),
                traceback: traceback.clone(),
                stdout: String::new(),
                stderr: String::new(),
            });
        }
    }
}
