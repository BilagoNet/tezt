#!/usr/bin/env python3
"""Benchmark orchestrator: tezt vs pytest (and pytest-xdist).

Measures wall time (time.perf_counter around subprocess.run) for:

  a) collection-only       tezt <suite> --collect-only -q
                           python3 -m pytest -q --collect-only <suite>
  b) full run              tezt <suite> -j J -q
                           python3 -m pytest -q <suite>
                           python3 -m pytest -q -n J <suite>   (if xdist importable)
  c) cold-start micro      a generated 1-file/1-test suite, full run, both runners

Every measurement is repeated --runs times; we report median/min/max
seconds plus amortized ms per test (median / total_cases * 1000).

Sanity checks: both runners' stdout is parsed for collected/passed
counts; the script aborts with a clear error if the runners disagree
with each other, if a run exits non-zero, or if not every test passed.
(Generated suites contain no intentional failures, so rc must be 0.)

Results are appended to a structured JSON file (--out, default
bench/results.json) and bench/RESULTS.md is regenerated from the full
JSON on every invocation.

All paths are derived from arguments or this file's location; nothing
is hardcoded. stdlib only.
"""

import argparse
import datetime
import json
import os
import platform
import re
import shlex
import statistics
import subprocess
import sys
import tempfile
import time

BENCH_DIR = os.path.dirname(os.path.abspath(__file__))
TIMEOUT_S = 300

# ---------------------------------------------------------------- output parsing

# tezt summary:   "collected N tests"  /  "N passed"
TEZT_COLLECTED = re.compile(r"collected\s+(\d+)\s+tests?\b")
TEZT_PASSED = re.compile(r"\b(\d+)\s+passed\b")

# pytest -q --collect-only:  "N tests collected in 0.01s"
#   (older/edge: "no tests collected", "1 test collected")
PYTEST_COLLECTED = re.compile(r"(\d+)\s+tests?\s+collected\b")
# pytest -q full run: "N passed in 0.12s" (also "N passed, M warnings in ...")
PYTEST_PASSED = re.compile(r"\b(\d+)\s+passed\b")


def parse_count(runner: str, mode: str, text: str):
    """Extract collected/passed count from a runner's combined output.

    Returns int or None if no recognizable summary was found.
    """
    if runner == "tezt":
        pat = TEZT_COLLECTED if mode == "collect" else TEZT_PASSED
    else:
        pat = PYTEST_COLLECTED if mode == "collect" else PYTEST_PASSED
    m = pat.search(text)
    return int(m.group(1)) if m else None


# ---------------------------------------------------------------- measurement

class BenchError(SystemExit):
    pass


def _fail(msg: str, cmd=None, output: str = ""):
    parts = [f"benchmark error: {msg}"]
    if cmd:
        parts.append(f"  command: {' '.join(shlex.quote(c) for c in cmd)}")
    if output:
        tail = output.strip().splitlines()[-15:]
        parts.append("  --- last output lines ---")
        parts.extend("  " + ln for ln in tail)
    raise BenchError("\n".join(parts))


def timed_run(cmd, cwd=None, env=None):
    """Run cmd once, return (seconds, combined_output). Raises on rc!=0/timeout."""
    t0 = time.perf_counter()
    try:
        proc = subprocess.run(
            cmd, cwd=cwd, env=env,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, timeout=TIMEOUT_S,
        )
    except subprocess.TimeoutExpired as e:
        out = e.output or ""
        if isinstance(out, bytes):
            out = out.decode("utf-8", "replace")
        _fail(f"timed out after {TIMEOUT_S}s", cmd, out)
    except FileNotFoundError:
        _fail(f"executable not found: {cmd[0]}", cmd)
    elapsed = time.perf_counter() - t0
    if proc.returncode != 0:
        _fail(f"exit code {proc.returncode} (expected 0 -- generated suites "
              f"contain no failing tests)", cmd, proc.stdout)
    return elapsed, proc.stdout


def measure(label, runner, mode, cmd, runs, expected_cases=None, env=None):
    """Run cmd `runs` times; return a result dict.

    Verifies the parsed collected/passed count is identical across runs
    and (if expected_cases given) matches it.
    """
    times = []
    count = None
    for i in range(runs):
        secs, out = timed_run(cmd, env=env)
        c = parse_count(runner, mode, out)
        if c is None:
            _fail(f"{runner} ({label}): could not parse "
                  f"{'collected' if mode == 'collect' else 'passed'} count "
                  f"from output", cmd, out)
        if count is None:
            count = c
        elif c != count:
            _fail(f"{runner} ({label}): count changed between runs "
                  f"({count} vs {c}) -- non-deterministic suite?", cmd, out)
        times.append(secs)
    if expected_cases is not None and count != expected_cases:
        _fail(f"{runner} ({label}): reported {count} "
              f"{'collected' if mode == 'collect' else 'passed'} but the "
              f"suite nominally contains {expected_cases} test cases", cmd)
    med = statistics.median(times)
    return {
        "label": label,
        "runner": runner,
        "mode": mode,
        "command": cmd,
        "runs": runs,
        "times_s": [round(t, 6) for t in times],
        "median_s": round(med, 6),
        "min_s": round(min(times), 6),
        "max_s": round(max(times), 6),
        "count": count,
        "ms_per_test": round(med / count * 1000, 4) if count else None,
    }


# ---------------------------------------------------------------- suite info

def _param_multiplier(func_node):
    """Return the parametrize expansion factor for a function's decorators."""
    import ast
    mult = 1
    for dec in func_node.decorator_list:
        if (isinstance(dec, ast.Call)
                and isinstance(dec.func, ast.Attribute)
                and dec.func.attr == "parametrize"
                and len(dec.args) >= 2
                and isinstance(dec.args[1], (ast.List, ast.Tuple))):
            mult *= max(1, len(dec.args[1].elts))
    return mult


def count_suite_cases(suite_dir):
    """Recompute nominal case count by statically parsing the suite (ast).

    Counts `def test_*` at module level and methods inside `Test*`
    classes, multiplying by literal parametrize case counts. Matches
    pytest/tezt default collection rules for suites produced by
    gen_suite.py (and most conventional suites).
    """
    import ast
    total = 0
    for root, _dirs, files in os.walk(suite_dir):
        for name in sorted(files):
            if not (name.startswith("test_") and name.endswith(".py")):
                continue
            path = os.path.join(root, name)
            try:
                with open(path) as fh:
                    tree = ast.parse(fh.read(), filename=path)
            except SyntaxError as e:
                _fail(f"suite file {path} has a syntax error: {e}")
            for node in tree.body:
                if (isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
                        and node.name.startswith("test")):
                    total += _param_multiplier(node)
                elif (isinstance(node, ast.ClassDef)
                        and node.name.startswith("Test")):
                    for sub in node.body:
                        if (isinstance(sub, (ast.FunctionDef,
                                             ast.AsyncFunctionDef))
                                and sub.name.startswith("test")):
                            total += _param_multiplier(sub)
    return total


def xdist_available(py):
    try:
        subprocess.run([py, "-c", "import xdist"], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                       timeout=30)
        return True
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired,
            FileNotFoundError):
        return False


def gen_micro_suite(tmpdir):
    """Write a 1-file / 1-test plain suite for cold-start measurement."""
    path = os.path.join(tmpdir, "test_cold.py")
    with open(path, "w") as fh:
        fh.write("def test_cold_start():\n    assert 1 + 1 == 2\n")
    return tmpdir, 1


# ---------------------------------------------------------------- reporting

def render_results_md(all_results, md_path):
    """Regenerate RESULTS.md from the full results.json contents."""
    lines = []
    lines.append("# tezt benchmark results")
    lines.append("")
    env = all_results.get("environment", {})
    lines.append(f"- date: {env.get('date', '?')}")
    lines.append(f"- python: {env.get('python', '?')}")
    lines.append(f"- cpu_count: {env.get('cpu_count', '?')}")
    lines.append(f"- platform: {env.get('platform', '?')}")
    lines.append("")
    lines.append("Wall time measured with `time.perf_counter` around "
                 "`subprocess.run`; each row is the median of N runs. "
                 "`ms/test` = median seconds / collected cases x 1000 "
                 "(amortized per-test overhead, includes process startup). "
                 "Speedup is vs the single-process `pytest` row in the "
                 "same suite+mode group.")
    lines.append("")

    headlines = []
    for entry in all_results.get("entries", []):
        label = entry["label"]
        cases = entry.get("test_cases", "?")
        ts = entry.get("timestamp", "")
        lines.append(f"## suite: {label}  ({cases} test cases)")
        if ts:
            lines.append(f"_measured {ts}; runs per row: "
                         f"{entry.get('runs', '?')}; jobs: "
                         f"{entry.get('jobs', '?')}_")
        lines.append("")
        lines.append("| runner / mode | median (s) | min (s) | max (s) "
                     "| speedup vs pytest | ms/test |")
        lines.append("|---|---:|---:|---:|---:|---:|")

        for mode in ("collect", "full"):
            rows = [r for r in entry["results"] if r["mode"] == mode]
            if not rows:
                continue
            baseline = next((r for r in rows
                             if r["runner"] == "pytest"
                             and "xdist" not in r["label"]), None)
            base_med = baseline["median_s"] if baseline else None
            for r in rows:
                name = r["label"]
                if base_med and r["median_s"] > 0:
                    speed = base_med / r["median_s"]
                    speed_str = f"{speed:.2f}x" if r is not baseline else "1.00x (baseline)"
                else:
                    speed_str = "-"
                mspt = (f"{r['ms_per_test']:.3f}"
                        if r.get("ms_per_test") is not None else "-")
                lines.append(
                    f"| {name} | {r['median_s']:.4f} | {r['min_s']:.4f} "
                    f"| {r['max_s']:.4f} | {speed_str} | {mspt} |")

                # headline candidate: tezt full run on the biggest real
                # suite (skip the 1-test cold-start micro entries)
                if (r["runner"] == "tezt" and mode == "full"
                        and isinstance(cases, int) and cases > 1
                        and not label.startswith("cold-start")
                        and base_med):
                    headlines.append((cases, r, base_med))
        lines.append("")

    if headlines:
        headlines.sort(key=lambda x: x[0])
        cases, r, base_med = headlines[-1]
        speed = base_med / r["median_s"] if r["median_s"] > 0 else 0
        head = (f"**Headline:** tezt runs {cases:,} tests in "
                f"{r['median_s']:.2f}s ({r['ms_per_test']:.2f} ms/test) "
                f"-- {speed:.1f}x faster than pytest.")
        lines.insert(1, "")
        lines.insert(2, head)
        print(head.replace("**", ""))

    with open(md_path, "w") as fh:
        fh.write("\n".join(lines) + "\n")


# ---------------------------------------------------------------- main

def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--tezt-bin", default=None,
                   help="path to the tezt binary (required unless --only pytest)")
    p.add_argument("--suite", required=True, help="path to a generated suite dir")
    p.add_argument("--runs", type=int, default=5, help="repetitions per measurement")
    p.add_argument("--jobs", type=int, default=4, help="parallel jobs (-j / -n)")
    p.add_argument("--label", default=None,
                   help="label for this suite in results (default: suite dirname)")
    p.add_argument("--out", default=os.path.join(BENCH_DIR, "results.json"),
                   help="results JSON path (appended)")
    p.add_argument("--only", choices=("tezt", "pytest", "all"), default="all")
    p.add_argument("--python", default=sys.executable,
                   help="python interpreter used to invoke pytest")
    p.add_argument("--skip-cold", action="store_true",
                   help="skip the 1-file/1-test cold-start micro benchmark")
    args = p.parse_args(argv)

    suite = os.path.abspath(args.suite)
    if not os.path.isdir(suite):
        _fail(f"suite directory does not exist: {suite}")
    label = args.label or os.path.basename(suite.rstrip(os.sep))

    run_tezt = args.only in ("tezt", "all")
    run_pytest = args.only in ("pytest", "all")
    tezt_bin = None
    if run_tezt:
        if not args.tezt_bin:
            _fail("--tezt-bin is required unless --only pytest")
        tezt_bin = os.path.abspath(args.tezt_bin)
        if not (os.path.isfile(tezt_bin) and os.access(tezt_bin, os.X_OK)):
            _fail(f"tezt binary not found or not executable: {tezt_bin}")

    py = args.python
    have_xdist = run_pytest and xdist_available(py)

    expected = count_suite_cases(suite)
    if expected == 0:
        _fail(f"no test cases found in suite {suite}")
    print(f"suite: {suite}  (nominal {expected} test cases)")
    print(f"runs per measurement: {args.runs}; jobs: {args.jobs}; "
          f"xdist: {'yes' if have_xdist else 'no'}")

    # Keep pytest from writing .pytest_cache into the suite (skews repeats less,
    # and -p no:cacheprovider keeps the suite dir pristine for tezt runs).
    env = dict(os.environ)
    env.setdefault("PYTHONDONTWRITEBYTECODE", "1")  # fair: tezt doesn't cache either
    pytest_base = [py, "-m", "pytest", "-q", "-p", "no:cacheprovider"]

    results = []

    def add(*a, **kw):
        r = measure(*a, runs=args.runs, env=env, **kw)
        results.append(r)
        print(f"  {r['label']:<28} median {r['median_s']:.4f}s  "
              f"min {r['min_s']:.4f}  max {r['max_s']:.4f}  "
              f"count {r['count']}")

    # ---- a) collection ----
    print("\n[collection]")
    if run_tezt:
        add("tezt collect", "tezt", "collect",
            [tezt_bin, suite, "--collect-only", "-q"],
            expected_cases=expected)
    if run_pytest:
        add("pytest collect", "pytest", "collect",
            pytest_base + ["--collect-only", suite],
            expected_cases=expected)

    # ---- cross-runner sanity on collection ----
    coll_counts = {r["runner"]: r["count"] for r in results if r["mode"] == "collect"}
    if len(set(coll_counts.values())) > 1:
        _fail(f"runners disagree on collected count: {coll_counts}")

    # ---- b) full run ----
    print("\n[full run]")
    if run_tezt:
        add(f"tezt -j {args.jobs}", "tezt", "full",
            [tezt_bin, suite, "-j", str(args.jobs), "-q"],
            expected_cases=expected)
    if run_pytest:
        add("pytest", "pytest", "full",
            pytest_base + [suite], expected_cases=expected)
        if have_xdist:
            add(f"pytest -n {args.jobs} (xdist)", "pytest", "full",
                pytest_base + ["-n", str(args.jobs), suite],
                expected_cases=expected)

    full_counts = {r["label"]: r["count"] for r in results if r["mode"] == "full"}
    if len(set(full_counts.values())) > 1:
        _fail(f"runners disagree on passed count: {full_counts}")

    # ---- c) cold start micro ----
    cold_results = []
    if not args.skip_cold:
        print("\n[cold start: 1 file / 1 test, full run]")
        with tempfile.TemporaryDirectory(prefix="tezt_cold_") as tmp:
            micro, micro_cases = gen_micro_suite(tmp)
            if run_tezt:
                r = measure("tezt cold (1 test)", "tezt", "full",
                            [tezt_bin, micro, "-j", "1", "-q"],
                            runs=args.runs, expected_cases=micro_cases, env=env)
                cold_results.append(r)
                print(f"  {r['label']:<28} median {r['median_s']:.4f}s")
            if run_pytest:
                r = measure("pytest cold (1 test)", "pytest", "full",
                            pytest_base + [micro],
                            runs=args.runs, expected_cases=micro_cases, env=env)
                cold_results.append(r)
                print(f"  {r['label']:<28} median {r['median_s']:.4f}s")

    # ---- persist ----
    out_path = os.path.abspath(args.out)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    if os.path.isfile(out_path):
        try:
            with open(out_path) as fh:
                doc = json.load(fh)
        except (json.JSONDecodeError, OSError) as e:
            _fail(f"existing results file {out_path} is unreadable: {e}")
    else:
        doc = {"entries": []}

    doc["environment"] = {
        "date": datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        "python": platform.python_version(),
        "cpu_count": os.cpu_count(),
        "platform": f"{platform.system()} {platform.release()} ({platform.machine()})",
    }
    entry = {
        "label": label,
        "suite": suite,
        "timestamp": doc["environment"]["date"],
        "test_cases": expected,
        "runs": args.runs,
        "jobs": args.jobs,
        "results": results,
    }
    doc["entries"] = [e for e in doc.get("entries", []) if e["label"] != label]
    doc["entries"].append(entry)
    if cold_results:
        cold_entry = {
            "label": f"cold-start (via {label})",
            "suite": "<temp 1 file / 1 test>",
            "timestamp": doc["environment"]["date"],
            "test_cases": 1,
            "runs": args.runs,
            "jobs": 1,
            "results": cold_results,
        }
        doc["entries"] = [e for e in doc["entries"]
                          if e["label"] != cold_entry["label"]]
        doc["entries"].append(cold_entry)

    with open(out_path, "w") as fh:
        json.dump(doc, fh, indent=2)
        fh.write("\n")

    md_path = os.path.join(os.path.dirname(out_path), "RESULTS.md")
    print()
    render_results_md(doc, md_path)
    print(f"\nwrote {out_path}")
    print(f"wrote {md_path}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except BenchError as e:
        print(e, file=sys.stderr)
        sys.exit(1)
