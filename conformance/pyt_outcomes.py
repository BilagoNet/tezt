"""pytest plugin: record the authoritative per-test outcome for each nodeid.

Loaded via `-p pyt_outcomes` (this dir on PYTHONPATH). Writes
`{"outcomes": {nodeid: outcome}, "exitstatus": N}` to $PYT_OUT at sessionfinish.

Outcome vocabulary matches tezt's: passed / failed / skipped / xfailed /
xpassed / error. This is how *pytest itself* classifies each result, so it is
the oracle the conformance harness compares tezt against.
"""

import json
import os

_outcomes = {}


def pytest_runtest_logreport(report):
    nid = report.nodeid
    if report.when == "call":
        oc = report.outcome
        if hasattr(report, "wasxfail"):
            if oc == "skipped":
                _outcomes[nid] = "xfailed"
            elif oc == "passed":
                _outcomes[nid] = "xpassed"
            else:  # strict xfail that unexpectedly passed -> a failure
                _outcomes[nid] = "failed"
        else:
            _outcomes[nid] = oc  # passed | failed | skipped
    elif report.when == "setup":
        if report.outcome == "failed":
            _outcomes[nid] = "error"
        elif report.outcome == "skipped" and nid not in _outcomes:
            _outcomes[nid] = "xfailed" if hasattr(report, "wasxfail") else "skipped"
    elif report.when == "teardown":
        # a teardown failure turns an otherwise-passing test into an error
        if report.outcome == "failed" and _outcomes.get(nid) == "passed":
            _outcomes[nid] = "error"


def pytest_sessionfinish(session, exitstatus):
    path = os.environ.get("PYT_OUT")
    if path:
        with open(path, "w") as f:
            json.dump({"outcomes": _outcomes, "exitstatus": int(exitstatus)}, f)
