//! Test execution: a pool of persistent Python worker processes fed over a
//! JSON-lines stdio protocol. Workers import each test module once, cache
//! fixture plans, and stream results back as they finish.

use crate::collect::TestItem;
use anyhow::{Context, Result};
use crossbeam_channel as channel;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The embedded Python worker, shipped inside the tezt binary.
pub const WORKER_SOURCE: &str = include_str!("../python/tezt_worker.py");

const BATCH_CHUNK: usize = 64;

// --- process cleanup machinery ----------------------------------------------
//
// Three layers of defense ensure no orphaned Python workers (or grandchildren
// they spawned) survive a Ctrl-C, kill, panic, or early return:
//   1. each worker runs in its own process group (so a kill reaches the whole
//      subtree), see `Worker::spawn`;
//   2. a `Drop` impl force-terminates the group as a backstop (panics / early
//      `?` returns in worker threads);
//   3. a process-wide Ctrl-C / SIGTERM handler kills every live group, below.

/// Registry of live worker process-group ids (== worker pid on unix). A pgid is
/// inserted on successful spawn and removed on reap. The Ctrl-C handler walks
/// this set to terminate every outstanding worker subtree.
static LIVE_WORKERS: Mutex<Option<FxHashSet<i32>>> = Mutex::new(None);

/// Process-global stop flag. The signal handler sets this; `run_tests` also
/// threads it through the per-run `Arc<AtomicBool>` used for --maxfail, so a
/// signal interrupts in-flight scheduling the same way --maxfail does.
static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

/// Ensures the Ctrl-C / SIGTERM handler is installed at most once per process.
static HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Record a freshly-spawned worker's process-group id in the global registry.
fn register_worker(pgid: i32) {
    let mut guard = LIVE_WORKERS.lock().unwrap();
    guard.get_or_insert_with(FxHashSet::default).insert(pgid);
}

/// Remove a worker's process-group id once it has been reaped.
fn unregister_worker(pgid: i32) {
    if let Some(set) = LIVE_WORKERS.lock().unwrap().as_mut() {
        set.remove(&pgid);
    }
}

/// Force-kill a worker by pid/pgid. Used by the timeout watchdog, which doesn't
/// own the `Worker`/`Child` and so can't call `force_kill_group`. On unix this
/// signals the whole process group, so a hung test's own subprocesses die too;
/// on Windows it terminates the worker process (its descendants are reaped when
/// the shared Job Object closes).
#[cfg(unix)]
fn force_kill_pid(pgid: i32) {
    // SAFETY: best-effort SIGKILL to a process group we created with
    // `process_group(0)`; harmless if the group is already gone.
    unsafe {
        libc::killpg(pgid, libc::SIGKILL);
    }
}

#[cfg(windows)]
fn force_kill_pid(pid: i32) {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    // SAFETY: open the worker by pid for termination, terminate it, close the
    // handle. All best-effort: a null handle (already exited) is skipped.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid as u32);
        if !handle.is_null() {
            let _ = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
        }
    }
}

/// Install the process-wide interrupt handler exactly once. On SIGINT/SIGTERM
/// (or a Windows Ctrl-C/Ctrl-Break event) it flips the global stop flag, kills
/// every outstanding worker (its process group on unix; the shared Job Object on
/// Windows), and exits with the conventional interrupted code (130 on unix,
/// `STATUS_CONTROL_C_EXIT` on Windows).
fn install_signal_handler() {
    if HANDLER_INSTALLED.swap(true, Ordering::SeqCst) {
        return; // already installed this process
    }
    // `ctrlc` catches SIGINT/SIGTERM on unix and Ctrl-C/Ctrl-Break on Windows.
    // Ignore an "already set" error so this stays safe if some other code also
    // registered a handler.
    let _ = ctrlc::set_handler(move || {
        SHOULD_STOP.store(true, Ordering::SeqCst);

        // unix: signal every worker process group we registered. SIGTERM takes
        // down the worker plus any grandchildren sharing its group.
        #[cfg(unix)]
        {
            let pgids: Vec<i32> = LIVE_WORKERS
                .lock()
                .map(|g| {
                    g.as_ref()
                        .map(|s| s.iter().copied().collect())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            for pgid in pgids {
                // SAFETY: killpg only signals process groups we created (one per
                // worker via `process_group(0)`). Best-effort; the group may
                // already be gone.
                unsafe {
                    libc::killpg(pgid, libc::SIGTERM);
                }
            }
            // 128 + SIGINT: the conventional code for a Ctrl-C'd program.
            std::process::exit(130);
        }

        // Windows: a single TerminateJobObject kills every worker and every
        // grandchild assigned to the shared job — no per-worker loop needed.
        #[cfg(windows)]
        {
            if let Some(job) = winjob::shared() {
                job.terminate();
            }
            // STATUS_CONTROL_C_EXIT, what cmd.exe reports for a Ctrl-C'd console app.
            std::process::exit(0xC000_013A_u32 as i32);
        }
    });
}

// --- Windows: Job Object cleanup (the analogue of unix process groups) ------
//
// Windows has no process group we can signal. Instead we create ONE Job Object
// for the whole run, configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, and
// assign every worker to it. When the job's last handle is closed — on clean
// exit, when the OS closes our handles as the process dies, or when the Ctrl-C
// handler calls `TerminateJobObject` — Windows terminates every process still in
// the job: all workers AND any grandchildren a test spawned. Mirrors uv
// (crates/uv-windows/src/job.rs).
#[cfg(windows)]
mod winjob {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::sync::OnceLock;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    /// Owns a Job Object handle. Because the job sets
    /// `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, closing its last handle terminates
    /// every process still assigned to it.
    pub struct WorkerJob {
        handle: HANDLE,
    }

    // SAFETY: a Job Object handle is a kernel object whose APIs are thread-safe;
    // we only read it after creation, so sharing it across threads (it lives in a
    // `static`) is sound.
    unsafe impl Send for WorkerJob {}
    unsafe impl Sync for WorkerJob {}

    impl WorkerJob {
        fn create() -> Option<Self> {
            // SAFETY: null name + null security attributes is the documented form
            // for an anonymous job object; it returns NULL on failure.
            let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if handle.is_null() {
                return None;
            }
            let job = WorkerJob { handle };

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            // SAFETY: `handle` is the job we just created; `info` is a
            // fully-initialized struct for the extended-limit info class and we
            // pass its real byte length. Returns 0 on failure.
            let ok = unsafe {
                SetInformationJobObject(
                    job.handle,
                    JobObjectExtendedLimitInformation,
                    std::ptr::addr_of!(info).cast(),
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };
            if ok == 0 {
                return None; // `job` drops -> CloseHandle; harmless on a limit-less job
            }
            Some(job)
        }

        /// Assign a freshly-spawned child (and its future descendants) to the job.
        pub fn assign(&self, child: &Child) {
            let raw = child.as_raw_handle();
            // SAFETY: `self.handle` is a valid job; `raw` is the live process
            // handle owned by `child`. AssignProcessToJobObject only borrows it.
            unsafe {
                let _ = AssignProcessToJobObject(self.handle, raw as HANDLE);
            }
        }

        /// Synchronously terminate every process in the job. Used by the Ctrl-C
        /// handler, which exits the process immediately afterward.
        pub fn terminate(&self) {
            // SAFETY: valid job handle; the exit code for the killed processes is
            // arbitrary. Best-effort.
            unsafe {
                let _ = TerminateJobObject(self.handle, 1);
            }
        }
    }

    impl Drop for WorkerJob {
        fn drop(&mut self) {
            // SAFETY: we own `self.handle`. Closing the last handle to a
            // KILL_ON_JOB_CLOSE job terminates any processes still in it.
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }

    /// The single Job Object for the process, created on first worker spawn.
    /// `None` if creation failed (we then fall back to the per-worker kill).
    static JOB: OnceLock<Option<WorkerJob>> = OnceLock::new();

    pub fn shared() -> Option<&'static WorkerJob> {
        JOB.get_or_init(WorkerJob::create).as_ref()
    }
}

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
    /// Process-group id of the worker. On unix this equals the worker's pid
    /// (we put each worker in its own group via `process_group(0)`), and a
    /// `killpg` on it reaches the worker plus any grandchildren it spawned.
    /// On non-unix this is unused (we kill the child directly).
    pgid: i32,
    /// Set once the worker has been reaped through `shutdown`/`kill` so the
    /// `Drop` backstop knows not to signal/wait a second time.
    reaped: bool,
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
        // Layer 1: put the worker in its own process group so a single kill
        // (killpg) reaches the worker *and* any grandchildren a test spawned.
        // `process_group(0)` makes the child's pgid equal its own pid. This is
        // the clean std API (no hand-rolled pre_exec). No-op on non-unix.
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd
            .spawn()
            .with_context(|| format!("failed to start Python worker (`{python}`)"))?;
        // On unix the pgid equals the pid thanks to `process_group(0)`.
        let pgid = child.id() as i32;
        // Windows has no process groups; instead assign the worker to a shared
        // Job Object so it — and anything it spawns — is terminated when the job
        // closes (on exit/panic/Ctrl-C). Best-effort: the kill-on-drop backstop
        // still reaps the direct child if assignment fails.
        #[cfg(windows)]
        if let Some(job) = winjob::shared() {
            job.assign(&child);
        }
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let mut w = Worker {
            child,
            stdin,
            stdout,
            pgid,
            reaped: false,
        };
        // Layer 3 bookkeeping: make this group visible to the signal handler.
        register_worker(pgid);
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
        // Graceful path: ask the worker to run session teardowns, wait for its
        // `bye`, then reap. Marking `reaped` first means the `Drop` backstop
        // (which runs when `self` falls out of scope here) is a no-op — normal
        // completion never force-kills.
        let _ = self.send(&WireCmd::Shutdown);
        let _ = self.read_until_bye();
        let _ = self.child.wait();
        unregister_worker(self.pgid);
        self.reaped = true;
        // `self` drops here; Drop sees `reaped == true` and does nothing.
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
        // Explicit force-kill path (worker died mid-batch / --maxfail). Reap the
        // whole process group, then mark `reaped` so Drop doesn't repeat it.
        self.force_kill_group();
        unregister_worker(self.pgid);
        self.reaped = true;
    }

    /// Force-terminate the worker's entire process group (worker + any
    /// grandchildren). Shared by `kill` and the `Drop` backstop.
    fn force_kill_group(&mut self) {
        #[cfg(unix)]
        unsafe {
            // SAFETY: `killpg` only signals our own worker's process group
            // (pgid == the worker pid we spawned with `process_group(0)`).
            // SIGTERM first for a chance at orderly shutdown, a brief grace,
            // then SIGKILL to guarantee nothing in the subtree survives. Both
            // calls are best-effort: the group may already be gone.
            libc::killpg(self.pgid, libc::SIGTERM);
            std::thread::sleep(std::time::Duration::from_millis(150));
            libc::killpg(self.pgid, libc::SIGKILL);
            // Reap the worker itself so it doesn't linger as a zombie.
            let _ = self.child.wait();
        }
        #[cfg(not(unix))]
        {
            // No process groups here; kill the direct child and reap it.
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl Drop for Worker {
    /// Layer 2: kill-on-drop backstop. If a worker is dropped without having
    /// been reaped through `shutdown`/`kill` — e.g. a panic or an early `?`
    /// return unwinds a worker thread — force-terminate its process group so no
    /// Python worker (or grandchild) is orphaned. After a clean shutdown
    /// `reaped` is already true, so this is a no-op and never double-kills.
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        self.force_kill_group();
        unregister_worker(self.pgid);
        self.reaped = true;
    }
}

// --- pool -------------------------------------------------------------------

pub struct RunConfig {
    pub python: String,
    pub rootdir: PathBuf,
    pub jobs: usize,
    pub no_capture: bool,
    pub maxfail: Option<usize>,
    /// Per-test wall-clock budget. When set, a watchdog kills any worker that
    /// produces no event within this long and the test is reported as an error.
    pub timeout: Option<Duration>,
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
    // Layer 3: install the process-wide Ctrl-C / SIGTERM handler once. It kills
    // every registered worker process group and exits 130 on interrupt.
    install_signal_handler();

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

    // --- per-test timeout watchdog (opt-in) ---------------------------------
    // With --timeout set, a watchdog thread kills any worker that produces no
    // event within the budget; that worker's thread then sees EOF and reports
    // its in-flight items as timed-out errors. With no timeout the watchdog is
    // never spawned and the execution hot path is unchanged.
    let timeout = cfg.timeout;
    let activity: Option<Arc<Mutex<FxHashMap<i32, Instant>>>> =
        timeout.map(|_| Arc::new(Mutex::new(FxHashMap::default())));
    let timed_out: Arc<Mutex<FxHashSet<i32>>> = Arc::new(Mutex::new(FxHashSet::default()));
    let watchdog_done = Arc::new(AtomicBool::new(false));
    let watchdog: Option<std::thread::JoinHandle<()>> = match (timeout, &activity) {
        (Some(budget), Some(act)) => {
            let activity = Arc::clone(act);
            let timed_out = Arc::clone(&timed_out);
            let watchdog_done = Arc::clone(&watchdog_done);
            // Poll a few times per budget, bounded so we neither miss the
            // deadline by much nor spin.
            let tick = budget
                .checked_div(4)
                .unwrap_or(budget)
                .clamp(Duration::from_millis(20), Duration::from_millis(250));
            Some(std::thread::spawn(move || {
                while !watchdog_done.load(Ordering::SeqCst) {
                    std::thread::sleep(tick);
                    let now = Instant::now();
                    let stale: Vec<i32> = {
                        let map = activity.lock().unwrap();
                        map.iter()
                            .filter(|(_, &last)| now.duration_since(last) > budget)
                            .map(|(&pgid, _)| pgid)
                            .collect()
                    };
                    for pgid in stale {
                        timed_out.lock().unwrap().insert(pgid);
                        force_kill_pid(pgid);
                        // Drop it so we don't kill twice; the worker thread will
                        // see EOF and report the test as timed out.
                        activity.lock().unwrap().remove(&pgid);
                    }
                }
            }))
        }
        _ => None,
    };

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
        let activity = activity.clone();
        let timed_out = Arc::clone(&timed_out);

        handles.push(std::thread::spawn(move || -> Result<()> {
            let mut worker: Option<Worker> = None;
            while let Ok(batch) = batch_rx.recv() {
                if should_stop(&stop) {
                    break;
                }
                if worker.is_none() {
                    worker = Some(Worker::spawn(&python, &shim, &rootdir, no_capture)?);
                }
                let w = worker.as_mut().unwrap();
                let pgid = w.pgid;
                // Mark this worker active so the watchdog measures the new test
                // from now (also covers the first test's module import).
                if let Some(act) = &activity {
                    act.lock().unwrap().insert(pgid, Instant::now());
                }

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

                let mut seen: FxHashSet<String> = FxHashSet::default();
                let mut worker_died = false;
                loop {
                    let ev = w.read_event()?;
                    // Any event = the worker made progress; reset its deadline.
                    if ev.is_some() {
                        if let Some(act) = &activity {
                            act.lock().unwrap().insert(pgid, Instant::now());
                        }
                    }
                    match ev {
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
                            // EOF: the worker exited, crashed, or was killed by
                            // the timeout watchdog. Name the timeout case so the
                            // failure is self-explanatory; either way the batch's
                            // unfinished items are reported as errors.
                            let msg = if timed_out.lock().unwrap().contains(&pgid) {
                                match timeout {
                                    Some(b) => {
                                        format!(
                                            "test timed out after {:.0}s; worker killed",
                                            b.as_secs_f64()
                                        )
                                    }
                                    None => "test timed out; worker killed".to_string(),
                                }
                            } else {
                                "worker process exited unexpectedly".to_string()
                            };
                            report_missing(&batch, &seen, &res_tx, &msg, None);
                            worker_died = true;
                            break;
                        }
                    }
                }
                if worker_died {
                    if let Some(w) = worker.take() {
                        w.kill();
                    }
                    if let Some(act) = &activity {
                        act.lock().unwrap().remove(&pgid);
                    }
                }
                if should_stop(&stop) {
                    break;
                }
            }
            if let Some(w) = worker.take() {
                // Take this worker off the watchdog's radar before its graceful
                // shutdown, which legitimately takes a moment.
                if let Some(act) = &activity {
                    act.lock().unwrap().remove(&w.pgid);
                }
                if should_stop(&stop) {
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
    // All workers are done; stop and reap the watchdog.
    watchdog_done.store(true, Ordering::SeqCst);
    if let Some(handle) = watchdog {
        let _ = handle.join();
    }
    let _ = std::fs::remove_file(&shim);

    Ok(RunOutput {
        stopped_early: stop.load(Ordering::SeqCst),
        results,
        wall_time_s: started.elapsed().as_secs_f64(),
    })
}

/// True if scheduling should halt: either this run hit --maxfail (`stop`) or a
/// signal flipped the process-global stop flag.
fn should_stop(stop: &AtomicBool) -> bool {
    stop.load(Ordering::SeqCst) || SHOULD_STOP.load(Ordering::SeqCst)
}

/// Emit `error` results for items of a batch that never produced a result
/// (worker crash mid-batch).
fn report_missing(
    batch: &Batch,
    seen: &FxHashSet<String>,
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
