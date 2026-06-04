#!/usr/bin/env python3
"""Combine per-cell benchmark JSONs (produced by run_bench.py in CI) into a
single Markdown matrix, in milliseconds.

Usage: assemble_matrix.py <incoming-dir> <output.md>

Each *.json in <incoming-dir> is one OS x Python cell; its filename minus the
extension is the cell label (we render "__" as " / ").
"""

import glob
import json
import os
import sys


def ms(seconds):
    return f"{seconds * 1000:.0f} ms"


def speedup(baseline_s, value_s):
    if not baseline_s or not value_s:
        return "-"
    return f"{baseline_s / value_s:.1f}x"


def render_cell(cell, doc):
    env = doc.get("environment", {})
    out = [
        f"## {cell.replace('__', ' / ')}",
        "",
        f"- python: `{env.get('python', '?')}`",
        f"- platform: `{env.get('platform', '?')}`",
        f"- cpu cores: {env.get('cpu_count', '?')}",
        "",
    ]
    # Only the real suite entries; cold-start micro-benchmarks are summarized
    # separately by run_bench.py and add noise here.
    entries = [e for e in doc.get("entries", []) if not str(e.get("label", "")).startswith("cold-start")]
    for entry in entries:
        out.append(
            f"**{entry.get('test_cases', '?')} tests** "
            f"(median of {entry.get('runs', '?')} runs, jobs {entry.get('jobs', '?')}):"
        )
        out.append("")
        out.append("| phase | runner | median | speedup vs pytest |")
        out.append("|---|---|--:|--:|")
        for mode, phase in (("collect", "collect"), ("full", "full run")):
            rows = [r for r in entry.get("results", []) if r.get("mode") == mode]
            base = next(
                (r["median_s"] for r in rows if r["runner"] == "pytest" and "xdist" not in r["label"]),
                None,
            )
            for r in rows:
                if r["runner"] == "pytest" and "xdist" not in r["label"]:
                    sp = "1.0x (baseline)"
                else:
                    sp = speedup(base, r.get("median_s"))
                out.append(f"| {phase} | {r['label']} | {ms(r['median_s'])} | {sp} |")
        out.append("")
    return "\n".join(out)


def main(argv):
    if len(argv) != 3:
        print("usage: assemble_matrix.py <incoming-dir> <output.md>", file=sys.stderr)
        return 2
    incoming, output = argv[1], argv[2]
    files = sorted(glob.glob(os.path.join(incoming, "*.json")))

    parts = [
        "# tezt benchmark matrix",
        "",
        "Produced by the [Benchmarks workflow](../.github/workflows/bench.yml) across the",
        "OS x Python matrix. Every cell runs the same generated suite under tezt and pytest,",
        "with tezt's collection cache disabled so it's a fair parse-vs-import comparison.",
        "Times are wall-clock medians; `speedup` is versus single-process pytest in the",
        "same phase.",
        "",
    ]
    if not files:
        parts.append("_No results were collected._")
    for path in files:
        cell = os.path.splitext(os.path.basename(path))[0]
        try:
            with open(path) as fh:
                doc = json.load(fh)
        except (OSError, json.JSONDecodeError) as exc:
            parts.append(f"## {cell}\n\n_unreadable result: {exc}_\n")
            continue
        parts.append(render_cell(cell, doc))

    with open(output, "w") as fh:
        fh.write("\n".join(parts).rstrip() + "\n")
    print(f"wrote {output} from {len(files)} cell(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
