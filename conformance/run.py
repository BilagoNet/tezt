#!/usr/bin/env python3
"""Differential conformance harness: tezt vs pytest.

For each scenario we write a tiny suite to a temp dir, run it through BOTH
pytest (the oracle) and tezt, and compare the per-test OUTCOME and the process
EXIT CODE. Message *wording* is intentionally not compared — only the verdict
(passed/failed/skipped/xfailed/xpassed/error) and the exit status, i.e. "given
this situation, what result does the runner produce".

Usage:
    TEZT_BIN=target/release/tezt VENV_PY=/tmp/conf-venv/bin/python \
        python3 conformance/run.py [--verbose]

Exit 0 if tezt matches pytest on every scenario (modulo scenarios tagged as a
known/intentional divergence), else 1.
"""

import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
TEZT = os.environ.get("TEZT_BIN", os.path.join(REPO, "target/release/tezt"))
VENV_PY = os.environ.get("VENV_PY", "/tmp/conf-venv/bin/python")
VERBOSE = "--verbose" in sys.argv or "-v" in sys.argv

# A scenario is (name, files, note). `files` maps filename -> source; the entry
# point is always "test_case.py". `note` (optional) marks an intentional
# divergence we don't count as a failure, with the reason.
S = []


def scn(name, src, note=None, files=None):
    f = {"test_case.py": src}
    if files:
        f.update(files)
    S.append((name, f, note))


scn("pass_and_fail", """
def test_ok():
    assert 1 + 1 == 2
def test_bad():
    assert 1 + 1 == 3
""")

scn("assert_with_message", """
def test_msg():
    assert False, "boom"
""")

scn("uncaught_exception_is_failed", """
def test_raises():
    raise RuntimeError("kaboom")
""")

scn("skip_mark", """
import pytest
@pytest.mark.skip(reason="nope")
def test_s():
    assert False
""")

scn("skip_call", """
import pytest
def test_s():
    pytest.skip("later")
""")

scn("skipif_true_false", """
import pytest
@pytest.mark.skipif(True, reason="x")
def test_a():
    assert False
@pytest.mark.skipif(False, reason="x")
def test_b():
    assert True
""")

scn("xfail_fail", """
import pytest
@pytest.mark.xfail(reason="known")
def test_x():
    assert False
""")

scn("xfail_pass_nonstrict", """
import pytest
@pytest.mark.xfail(reason="maybe")
def test_x():
    assert True
""")

scn("xfail_strict_pass_is_failed", """
import pytest
@pytest.mark.xfail(strict=True, reason="must fail")
def test_x():
    assert True
""")

scn("xfail_strict_fail", """
import pytest
@pytest.mark.xfail(strict=True, reason="must fail")
def test_x():
    assert False
""")

scn("fixture_simple", """
import pytest
@pytest.fixture
def val():
    return 42
def test_v(val):
    assert val == 42
""")

scn("fixture_yield_teardown", """
import pytest
@pytest.fixture
def res():
    yield "r"
def test_r(res):
    assert res == "r"
""")

scn("fixture_error_is_error", """
import pytest
@pytest.fixture
def broken():
    raise ValueError("setup failed")
def test_uses(broken):
    assert True
""")

scn("autouse_fixture", """
import pytest
_log = []
@pytest.fixture(autouse=True)
def auto():
    _log.append(1)
def test_sees_auto():
    assert _log == [1]
""")

scn("parametrized_fixture", """
import pytest
@pytest.fixture(params=[1, 2, 3])
def n(request):
    return request.param
def test_n(n):
    assert n > 0
""")

scn("parametrize_simple", """
import pytest
@pytest.mark.parametrize("x", [1, 2, 3])
def test_x(x):
    assert x > 0
""")

scn("parametrize_ids", """
import pytest
@pytest.mark.parametrize("v", [10, 20], ids=["ten", "twenty"])
def test_v(v):
    assert v >= 10
""")

scn("parametrize_stacked", """
import pytest
@pytest.mark.parametrize("a", [1, 2])
@pytest.mark.parametrize("b", [3, 4])
def test_ab(a, b):
    assert a < b
""")

scn("parametrize_one_failing_case", """
import pytest
@pytest.mark.parametrize("x", [1, 2, 0])
def test_pos(x):
    assert x > 0
""")

scn("class_basic", """
class TestThing:
    def test_a(self):
        assert True
    def test_b(self):
        assert False
""")

scn("class_setup_method", """
class TestC:
    def setup_method(self, method):
        self.x = 5
    def test_x(self):
        assert self.x == 5
""")

scn("class_with_init_not_collected", """
class TestWithInit:
    def __init__(self):
        pass
    def test_never(self):
        assert False
def test_real():
    assert True
""", note="pytest warns + does not collect a Test* class with __init__; tezt skips it too")

scn("setup_module_error", """
def setup_module(module):
    raise RuntimeError("module setup boom")
def test_a():
    assert True
def test_b():
    assert True
""")

scn("teardown_error_after_pass", """
import pytest
@pytest.fixture
def res():
    yield 1
    raise RuntimeError("teardown boom")
def test_uses(res):
    assert res == 1
""")

scn("raises_ok_and_missing", """
import pytest
def test_ok():
    with pytest.raises(ValueError):
        raise ValueError("x")
def test_missing():
    with pytest.raises(KeyError):
        pass
""")

scn("approx_scalar_and_seq", """
import pytest
def test_scalar():
    assert 0.1 + 0.2 == pytest.approx(0.3)
def test_seq():
    assert [0.1 + 0.2, 1.0] == pytest.approx([0.3, 1.0])
def test_bad():
    assert 0.3 == pytest.approx(0.31)
""")

scn("capsys_fixture", """
def test_capsys(capsys):
    print("hi")
    out, err = capsys.readouterr()
    assert out == "hi\\n"
""")

scn("caplog_fixture", """
import logging
def test_caplog(caplog):
    caplog.set_level(logging.INFO)
    logging.getLogger("x").info("hello")
    assert "hello" in caplog.text
""")

scn("recwarn_and_warns", """
import warnings
import pytest
def test_recwarn(recwarn):
    warnings.warn("d", DeprecationWarning)
    assert len(recwarn) >= 1
def test_warns():
    with pytest.warns(UserWarning):
        warnings.warn("u", UserWarning)
""")

scn("tmp_path_and_monkeypatch", """
def test_tmp(tmp_path):
    p = tmp_path / "f.txt"
    p.write_text("hi")
    assert p.read_text() == "hi"
def test_mp(monkeypatch):
    import os
    monkeypatch.setenv("TEZT_CONF_X", "1")
    assert os.environ["TEZT_CONF_X"] == "1"
""")

scn("conftest_fixture", """
def test_uses(shared):
    assert shared == "from-conftest"
""", files={"conftest.py": """
import pytest
@pytest.fixture
def shared():
    return "from-conftest"
"""})

scn("empty_file_no_tests", """
# no tests here
x = 1
""", note="exit code 5 (no tests collected)")

scn("syntax_error_file", """
def test_broken(:
    pass
""", note="pytest: collection error -> exit 2, no test outcome; tezt: 1 error item -> exit 1 (architectural difference)")

scn("async_test", """
async def test_async():
    assert True
""", note="tezt runs async tests natively; plain pytest (no asyncio plugin) skips/warns — intentional divergence")

scn("nested_class", """
class TestOuter:
    class TestInner:
        def test_deep(self):
            assert True
    def test_shallow(self):
        assert True
""")


def run_pytest(workdir, outpath):
    env = dict(os.environ)
    env["PYT_OUT"] = outpath
    env["PYTHONPATH"] = HERE + os.pathsep + env.get("PYTHONPATH", "")
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    proc = subprocess.run(
        [VENV_PY, "-m", "pytest", "-q", "-p", "no:cacheprovider",
         "-p", "pyt_outcomes", "-o", "addopts=", "-W", "ignore", "test_case.py"],
        cwd=workdir, env=env, capture_output=True, text=True)
    try:
        with open(outpath) as f:
            data = json.load(f)
        return data["outcomes"], data["exitstatus"], proc.stdout + proc.stderr
    except Exception:
        return {}, proc.returncode, proc.stdout + proc.stderr


def run_tezt(workdir, outpath):
    env = dict(os.environ)
    env["TEZT_PYTHON"] = VENV_PY
    proc = subprocess.run(
        [TEZT, "--color", "never", "--no-cache", "--json", outpath, "test_case.py"],
        cwd=workdir, env=env, capture_output=True, text=True)
    outcomes = {}
    try:
        with open(outpath) as f:
            data = json.load(f)
        for t in data.get("tests", []):
            outcomes[norm(t["id"])] = t["outcome"]
    except Exception:
        pass
    return outcomes, proc.returncode, proc.stdout + proc.stderr


def norm(nid):
    # Normalize ids so pytest and tezt line up: drop a leading "./".
    if nid.startswith("./"):
        nid = nid[2:]
    return nid


def main():
    matches, mismatches, diverge = 0, [], 0
    for name, files, note in S:
        with tempfile.TemporaryDirectory(prefix="tezt-conf-") as d:
            for fn, src in files.items():
                with open(os.path.join(d, fn), "w") as f:
                    f.write(src)
            p_out, p_code, p_log = run_pytest(d, os.path.join(d, "p.json"))
            t_out, t_code, t_log = run_tezt(d, os.path.join(d, "t.json"))

        p_norm = {norm(k): v for k, v in p_out.items()}
        ok_outcomes = p_norm == t_out
        ok_code = p_code == t_code
        ok = ok_outcomes and ok_code

        if ok:
            matches += 1
            tag = "MATCH"
        elif note:
            diverge += 1
            tag = "DIVERGE*"
        else:
            mismatches.append(name)
            tag = "MISMATCH"

        if not ok or VERBOSE:
            print("[%s] %s" % (tag, name))
            if note:
                print("    note: %s" % note)
            if not ok_code:
                print("    exit: pytest=%s tezt=%s" % (p_code, t_code))
            if not ok_outcomes:
                allk = sorted(set(p_norm) | set(t_out))
                for k in allk:
                    pv, tv = p_norm.get(k, "—"), t_out.get(k, "—")
                    flag = "" if pv == tv else "  <<<"
                    print("    %-45s pytest=%-9s tezt=%-9s%s" % (k, pv, tv, flag))

    print("\n" + "=" * 60)
    print("conformance: %d match, %d mismatch, %d known-divergence (of %d)"
          % (matches, len(mismatches), diverge, len(S)))
    if mismatches:
        print("MISMATCHES: " + ", ".join(mismatches))
    print("=" * 60)
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main())
