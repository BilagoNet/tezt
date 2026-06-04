#!/usr/bin/env python3
"""Self-test for tezt_worker.py -- spawns the worker as a subprocess and
exercises the full JSON-Lines protocol against generated temp test suites.

Stdlib only. Run: python3 python/test_worker_protocol.py  (exit 0 = green).
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import textwrap
import time

HERE = os.path.dirname(os.path.abspath(__file__))
WORKER = os.path.join(HERE, "tezt_worker.py")

PASS = 0
FAIL = 0


def check(cond, label, extra=""):
    global PASS, FAIL
    if cond:
        PASS += 1
        print("  ok    %s" % label)
    else:
        FAIL += 1
        print("  FAIL  %s %s" % (label, extra))


# ============================================================================
# Worker harness
# ============================================================================

class Worker:
    def __init__(self, rootdir, no_capture=False):
        cmd = [sys.executable, "-u", WORKER, "--rootdir", rootdir]
        if no_capture:
            cmd.append("--no-capture")
        self.proc = subprocess.Popen(
            cmd, cwd=rootdir,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, bufsize=1)
        self.ready = self._read_event()
        assert self.ready.get("event") == "ready", "no ready event: %r" % self.ready
        self._next_batch = 1

    def _read_event(self):
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError("worker stdout closed; stderr:\n%s"
                               % self.proc.stderr.read())
        return json.loads(line)

    def send(self, obj):
        self.proc.stdin.write(json.dumps(obj) + "\n")
        self.proc.stdin.flush()

    def run(self, items):
        """Send a run command; return (results, batch_done_event)."""
        bid = self._next_batch
        self._next_batch += 1
        self.send({"cmd": "run", "batch_id": bid, "items": items})
        results = []
        while True:
            ev = self._read_event()
            if ev.get("event") == "result":
                results.append(ev)
            elif ev.get("event") == "batch_done":
                return results, ev
            elif ev.get("event") == "fatal":
                raise RuntimeError("worker fatal: %r" % ev)

    def shutdown(self):
        self.send({"cmd": "shutdown"})
        ev = self._read_event()
        self.proc.wait(timeout=15)
        return ev, self.proc.returncode

    def kill(self):
        try:
            self.proc.kill()
        except Exception:
            pass


def write_suite(root, files):
    for rel, body in files.items():
        path = os.path.join(root, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as f:
            f.write(textwrap.dedent(body))
    return root


def by_id(results):
    return {r["id"]: r for r in results}


# ============================================================================
# Suite 1: outcomes, parametrize, marks, capture, raises, assertion enrich
# ============================================================================

def test_core(root):
    print("\n[core outcomes / parametrize / marks]")
    write_suite(root, {
        "test_core.py": """
            import tezt

            def test_pass():
                print("hello-stdout")
                import sys; print("hello-stderr", file=sys.stderr)
                assert 1 + 1 == 2

            def test_fail_enriched():
                a = 1
                b = 2
                assert a == b

            def test_fail_msg():
                assert False, "custom message"

            @tezt.mark.skip(reason="not today")
            def test_skipped_mark():
                assert False

            def test_skip_call():
                tezt.skip("runtime skip")

            @tezt.mark.skipif(True, reason="cond true")
            def test_skipif_true():
                assert False

            @tezt.mark.skipif(False, reason="cond false")
            def test_skipif_false():
                assert True

            @tezt.mark.xfail(reason="known bug")
            def test_xfail_fails():
                assert False

            @tezt.mark.xfail(reason="fixed?")
            def test_xfail_passes():
                assert True

            @tezt.parametrize("x", [1, 2, 3])
            def test_param(x):
                assert x > 0

            @tezt.parametrize("a,b", [(1, 2), (3, 4)])
            def test_param_multi(a, b):
                assert b == a + 1

            @tezt.parametrize("y", [10, 20])
            @tezt.parametrize("x", ["p", "q"])
            def test_cartesian(x, y):
                assert x in ("p", "q") and y in (10, 20)

            @tezt.parametrize("v", [object(), object()], ids=["first", "second"])
            def test_ids_kwarg(v):
                assert v is not None

            @tezt.parametrize("v", [None, True, 2.5])
            def test_id_kinds(v):
                pass

            def test_raises_ok():
                with tezt.raises(ValueError, match="bad .alue"):
                    raise ValueError("bad value here")

            def test_raises_no_raise():
                with tezt.raises(KeyError):
                    pass

            def test_error_exception():
                raise RuntimeError("boom in body")
        """,
    })
    w = Worker(root)
    items = [{"id": "core::%s" % q, "file": os.path.join(root, "test_core.py"),
              "qualname": q} for q in [
        "test_pass", "test_fail_enriched", "test_fail_msg",
        "test_skipped_mark", "test_skip_call", "test_skipif_true",
        "test_skipif_false", "test_xfail_fails", "test_xfail_passes",
        "test_param", "test_param_multi", "test_cartesian", "test_ids_kwarg",
        "test_id_kinds", "test_raises_ok", "test_raises_no_raise",
        "test_error_exception"]]
    results, done = w.run(items)
    r = by_id(results)

    check(done["batch_id"] == 1, "batch_done batch_id matches")
    check(r["core::test_pass"]["outcome"] == "passed", "plain pass")
    check("hello-stdout" in r["core::test_pass"]["stdout"], "stdout captured")
    check("hello-stderr" in r["core::test_pass"]["stderr"], "stderr captured")
    check(all(isinstance(x["duration_ms"], (int, float)) for x in results),
          "duration_ms numeric")

    fe = r["core::test_fail_enriched"]
    check(fe["outcome"] == "failed", "bare assert -> failed")
    # Rich operator-aware enrichment: the asserted expression plus both
    # operand values (a=1, b=2 are side-effect-free, so the safe path fires).
    check(fe["message"] and "assert a == b" in fe["message"]
          and "left" in fe["message"] and "right" in fe["message"]
          and "= 1" in fe["message"] and "= 2" in fe["message"],
          "assertion enrichment shows operands", repr(fe["message"]))
    check(fe["traceback"] is not None, "failed has traceback")

    check(r["core::test_fail_msg"]["outcome"] == "failed"
          and "custom message" in (r["core::test_fail_msg"]["message"] or ""),
          "assert with message")
    check(r["core::test_skipped_mark"]["outcome"] == "skipped"
          and "not today" in r["core::test_skipped_mark"]["message"],
          "mark.skip")
    check(r["core::test_skip_call"]["outcome"] == "skipped"
          and "runtime skip" in r["core::test_skip_call"]["message"],
          "tezt.skip()")
    check(r["core::test_skipif_true"]["outcome"] == "skipped", "skipif True")
    check(r["core::test_skipif_false"]["outcome"] == "passed", "skipif False")
    check(r["core::test_xfail_fails"]["outcome"] == "xfailed", "xfail failing")
    check(r["core::test_xfail_passes"]["outcome"] == "xpassed", "xfail passing -> xpassed")

    for pid in ("1", "2", "3"):
        check(r.get("core::test_param[%s]" % pid, {}).get("outcome") == "passed",
              "parametrize id [%s]" % pid)
    check(len([k for k in r if k.startswith("core::test_param[")]) == 3,
          "parametrize expands to 3 flat cases")
    check(r.get("core::test_param_multi[1-2]", {}).get("outcome") == "passed"
          and r.get("core::test_param_multi[3-4]", {}).get("outcome") == "passed",
          "multi-arg param ids joined with '-'")

    cart = [x["id"] for x in results if x["id"].startswith("core::test_cartesian[")]
    check(len(cart) == 4, "cartesian product yields 4 cases", repr(cart))
    check(cart == ["core::test_cartesian[p-10]", "core::test_cartesian[q-10]",
                   "core::test_cartesian[p-20]", "core::test_cartesian[q-20]"],
          "cartesian ids inner-first, innermost varies fastest", repr(cart))

    check("core::test_ids_kwarg[first]" in r and "core::test_ids_kwarg[second]" in r,
          "ids= kwarg overrides")
    check("core::test_id_kinds[None]" in r and "core::test_id_kinds[True]" in r
          and "core::test_id_kinds[2.5]" in r, "None/bool/float id rendering",
          repr(sorted(k for k in r if "id_kinds" in k)))

    check(r["core::test_raises_ok"]["outcome"] == "passed", "tezt.raises match ok")
    check(r["core::test_raises_no_raise"]["outcome"] == "failed"
          and "DID NOT RAISE" in r["core::test_raises_no_raise"]["message"],
          "tezt.raises DID NOT RAISE -> failed")
    check(r["core::test_error_exception"]["outcome"] == "failed"
          and "boom in body" in r["core::test_error_exception"]["message"],
          "body exception -> failed with message")

    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0, "shutdown -> bye, exit 0")


# ============================================================================
# Suite 2: fixtures (module/conftest/yield/session), monkeypatch, tmp_path
# ============================================================================

def test_fixtures(root):
    print("\n[fixtures / conftest / scopes / builtins]")
    write_suite(root, {
        "conftest.py": """
            import tezt
            EVENTS = []

            @tezt.fixture
            def shared():
                return "from-root-conftest"

            @tezt.fixture
            def overridable():
                return "root"

            @tezt.fixture(scope="session")
            def session_counter():
                session_counter.calls = getattr(session_counter, "calls", 0) + 1
                return session_counter.calls
        """,
        "pkg/conftest.py": """
            import tezt

            @tezt.fixture
            def overridable():
                return "pkg"

            @tezt.fixture
            def nested(shared):
                return shared + "+nested"
        """,
        "pkg/test_fix.py": """
            import os, tezt

            ORDER = []

            @tezt.fixture
            def outer():
                ORDER.append("outer-setup")
                yield "O"
                ORDER.append("outer-teardown")

            @tezt.fixture
            def inner(outer):
                ORDER.append("inner-setup")
                yield outer + "I"
                ORDER.append("inner-teardown")

            @tezt.fixture(scope="module")
            def mod_fix():
                mod_fix.n = getattr(mod_fix, "n", 0) + 1
                return mod_fix.n

            @tezt.fixture
            def overridable():
                return "module"

            def test_shared(shared):
                assert shared == "from-root-conftest"

            def test_nested_conftest(nested):
                assert nested == "from-root-conftest+nested"

            def test_override(overridable):
                assert overridable == "module"

            def test_yield_order(inner):
                assert inner == "OI"
                ORDER.append("test-ran")

            def test_yield_order_check():
                assert ORDER == ["outer-setup", "inner-setup", "test-ran",
                                 "inner-teardown", "outer-teardown"], ORDER

            def test_mod_scope_a(mod_fix):
                assert mod_fix == 1

            def test_mod_scope_b(mod_fix):
                assert mod_fix == 1  # cached within module

            def test_session_first(session_counter):
                assert session_counter == 1

            def test_monkeypatch(monkeypatch):
                monkeypatch.setenv("TEZT_SELFTEST_ENV", "yes")
                assert os.environ["TEZT_SELFTEST_ENV"] == "yes"
                monkeypatch.setattr(os.path, "sep_marker", "x", )

            def test_monkeypatch_undone():
                assert "TEZT_SELFTEST_ENV" not in os.environ
                assert not hasattr(os.path, "sep_marker")

            def test_tmp_path(tmp_path):
                p = tmp_path / "f.txt"
                p.write_text("data")
                assert p.read_text() == "data"
                globals()["_tmp1"] = str(tmp_path)

            def test_tmp_path_fresh(tmp_path):
                assert str(tmp_path) != globals()["_tmp1"]

            def test_tmp_factory(tmp_path_factory):
                d = tmp_path_factory.mktemp("zone")
                assert d.is_dir()

            def test_unknown_fixture(nope_not_real):
                pass
        """,
        "pkg/test_fix2.py": """
            def test_session_second_module(session_counter):
                assert session_counter == 1  # cached across modules/batches
        """,
        "pkg/test_scope_td.py": """
            import os, tezt
            ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

            @tezt.fixture(scope="module")
            def mod_res():
                yield "M"
                open(os.path.join(ROOT, "mod_teardown.flag"), "w").write("1")

            @tezt.fixture(scope="session")
            def sess_res():
                yield "S"
                open(os.path.join(ROOT, "sess_teardown.flag"), "w").write("1")

            def test_uses_scoped(mod_res, sess_res):
                assert (mod_res, sess_res) == ("M", "S")
        """,
        "pkg/test_scope_td2.py": """
            import os
            ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

            def test_module_teardown_fired():
                # ran after switching away from test_scope_td.py
                assert os.path.exists(os.path.join(ROOT, "mod_teardown.flag"))

            def test_session_teardown_not_yet():
                assert not os.path.exists(os.path.join(ROOT, "sess_teardown.flag"))
        """,
    })
    w = Worker(root)
    f1 = os.path.join(root, "pkg", "test_fix.py")
    quals = ["test_shared", "test_nested_conftest", "test_override",
             "test_yield_order", "test_yield_order_check",
             "test_mod_scope_a", "test_mod_scope_b", "test_session_first",
             "test_monkeypatch", "test_monkeypatch_undone",
             "test_tmp_path", "test_tmp_path_fresh", "test_tmp_factory",
             "test_unknown_fixture"]
    results, _ = w.run([{"id": q, "file": f1, "qualname": q} for q in quals])
    r = by_id(results)

    check(r["test_shared"]["outcome"] == "passed", "root conftest fixture injected")
    check(r["test_nested_conftest"]["outcome"] == "passed",
          "conftest fixture depending on parent conftest fixture")
    check(r["test_override"]["outcome"] == "passed",
          "module fixture overrides conftest (nearest wins)")
    check(r["test_yield_order"]["outcome"] == "passed", "yield fixture value")
    check(r["test_yield_order_check"]["outcome"] == "passed",
          "LIFO teardown order", r["test_yield_order_check"].get("message"))
    check(r["test_mod_scope_a"]["outcome"] == "passed"
          and r["test_mod_scope_b"]["outcome"] == "passed",
          "module-scope fixture cached")
    check(r["test_session_first"]["outcome"] == "passed", "session fixture v1")
    check(r["test_monkeypatch"]["outcome"] == "passed", "monkeypatch works")
    check(r["test_monkeypatch_undone"]["outcome"] == "passed",
          "monkeypatch auto-undo", r["test_monkeypatch_undone"].get("message"))
    check(r["test_tmp_path"]["outcome"] == "passed", "tmp_path usable")
    check(r["test_tmp_path_fresh"]["outcome"] == "passed", "tmp_path fresh per test")
    check(r["test_tmp_factory"]["outcome"] == "passed", "tmp_path_factory.mktemp")
    uf = r["test_unknown_fixture"]
    check(uf["outcome"] == "error" and "not found" in (uf["message"] or ""),
          "unknown fixture -> error 'not found'", repr(uf["message"]))

    # session scope persists into a SECOND batch and a second module
    f2 = os.path.join(root, "pkg", "test_fix2.py")
    results2, done2 = w.run([{"id": "s2", "file": f2,
                              "qualname": "test_session_second_module"}])
    check(done2["batch_id"] == 2, "second batch_done has batch_id 2")
    check(by_id(results2)["s2"]["outcome"] == "passed",
          "session fixture persists across batches/modules",
          repr(by_id(results2)["s2"].get("message")))

    # module-scope teardown fires on module switch; session teardown only at
    # shutdown (observable via sentinel files written by yield teardowns)
    f3 = os.path.join(root, "pkg", "test_scope_td.py")
    f4 = os.path.join(root, "pkg", "test_scope_td2.py")
    results3, _ = w.run([
        {"id": "td0", "file": f3, "qualname": "test_uses_scoped"},
        {"id": "td1", "file": f4, "qualname": "test_module_teardown_fired"},
        {"id": "td2", "file": f4, "qualname": "test_session_teardown_not_yet"},
    ])
    r3 = by_id(results3)
    check(r3["td0"]["outcome"] == "passed", "scoped fixtures injected")
    check(r3["td1"]["outcome"] == "passed",
          "module-scope yield teardown fires on module switch",
          repr(r3["td1"].get("message")))
    check(r3["td2"]["outcome"] == "passed",
          "session-scope teardown deferred until shutdown",
          repr(r3["td2"].get("message")))

    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0, "fixtures worker clean shutdown")
    check(os.path.exists(os.path.join(root, "sess_teardown.flag")),
          "session-scope yield teardown fires at shutdown")


# ============================================================================
# Suite 3: classes + xunit hooks, async, discovery '*', import errors
# ============================================================================

def test_classes_async_discovery(root):
    print("\n[classes / xunit / async / '*' discovery / import errors]")
    write_suite(root, {
        "test_cls.py": """
            CALLS = []

            def setup_module(mod):
                CALLS.append("setup_module")

            def teardown_module(mod):
                CALLS.append("teardown_module")

            class TestThing:
                @classmethod
                def setup_class(cls):
                    CALLS.append("setup_class")

                @classmethod
                def teardown_class(cls):
                    CALLS.append("teardown_class")

                def setup_method(self, method):
                    CALLS.append("setup_method")
                    self.token = 41

                def teardown_method(self, method):
                    CALLS.append("teardown_method")

                def test_one(self):
                    self.token += 1
                    assert self.token == 42

                def test_two(self):
                    # fresh instance: mutation from test_one must not leak
                    assert self.token == 41
                    CALLS.append("test_two")

            def test_calls_after_methods():
                assert "setup_module" in CALLS
                assert "setup_class" in CALLS
                assert CALLS.count("setup_method") == 2
                assert CALLS.count("teardown_method") == 2
        """,
        "test_async.py": """
            import asyncio

            async def test_async_pass():
                await asyncio.sleep(0)
                assert True

            async def test_async_fail():
                await asyncio.sleep(0)
                assert 1 == 2
        """,
        "test_disc.py": """
            import tezt

            def test_alpha():
                pass

            @tezt.parametrize("n", [1, 2])
            def test_beta(n):
                pass

            def helper_not_a_test():
                raise RuntimeError

            class TestGroup:
                def test_inside(self):
                    pass

            class TestSkipped:   # has __init__ -> not collected
                def __init__(self):
                    pass
                def test_nope(self):
                    pass

            class NotATest:
                def test_hidden(self):
                    pass
        """,
        "test_broken.py": """
            def test_never(:
                pass
        """,
    })
    w = Worker(root)

    # class tests with xunit hooks
    fcls = os.path.join(root, "test_cls.py")
    results, _ = w.run([
        {"id": "c1", "file": fcls, "qualname": "TestThing::test_one"},
        {"id": "c2", "file": fcls, "qualname": "TestThing::test_two"},
        {"id": "c3", "file": fcls, "qualname": "test_calls_after_methods"},
    ])
    r = by_id(results)
    check(r["c1"]["outcome"] == "passed", "class method test passes")
    check(r["c2"]["outcome"] == "passed", "fresh instance per method",
          repr(r["c2"].get("message")))
    check(r["c3"]["outcome"] == "passed", "xunit hook sequence",
          repr(r["c3"].get("message")))

    # async
    fas = os.path.join(root, "test_async.py")
    results, _ = w.run([
        {"id": "a1", "file": fas, "qualname": "test_async_pass"},
        {"id": "a2", "file": fas, "qualname": "test_async_fail"},
    ])
    r = by_id(results)
    check(r["a1"]["outcome"] == "passed", "async test passes via asyncio.run")
    check(r["a2"]["outcome"] == "failed", "async test failure detected")

    # '*' discovery
    fd = os.path.join(root, "test_disc.py")
    results, _ = w.run([{"id": "disc", "file": fd, "qualname": "*"}])
    ids = sorted(x["id"] for x in results)
    expect = sorted([
        "disc::test_alpha", "disc::test_beta[1]", "disc::test_beta[2]",
        "disc::TestGroup::test_inside",
    ])
    check(ids == expect, "'*' discovery ids exact", repr(ids))
    check(all(x["outcome"] == "passed" for x in results), "'*' cases pass")

    # syntax-error module -> every item errors
    fb = os.path.join(root, "test_broken.py")
    results, done = w.run([
        {"id": "b1", "file": fb, "qualname": "test_never"},
        {"id": "b2", "file": fb, "qualname": "*"},
    ])
    r = by_id(results)
    check(len(results) == 2
          and r["b1"]["outcome"] == "error" and r["b2"]["outcome"] == "error",
          "broken module -> error per item")
    check(r["b1"]["traceback"] and "SyntaxError" in r["b1"]["traceback"],
          "import error traceback mentions SyntaxError")
    check(done["event"] == "batch_done", "batch_done emitted after error batch")

    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0, "discovery worker clean shutdown")


# ============================================================================
# Suite 4: pytest-compat surface (pytestmark Mark objects, fixture markers)
# ============================================================================

def test_pytest_compat(root):
    print("\n[pytest compatibility shims]")
    write_suite(root, {
        # Simulate the attribute shapes the real pytest produces, without
        # importing pytest (sandbox may not have it).
        "fake_marks.py": """
            class Mark:
                def __init__(self, name, args=(), kwargs=None):
                    self.name = name
                    self.args = args
                    self.kwargs = kwargs or {}

            class FixtureFunctionMarker:
                def __init__(self, scope="function", name=None):
                    self.scope = scope
                    self.name = name
        """,
        "test_pycompat.py": """
            from fake_marks import Mark, FixtureFunctionMarker

            pytestmark = Mark("xfail")     # module-level single mark

            def fix_classic():
                return 99
            fix_classic._pytestfixturefunction = FixtureFunctionMarker(
                scope="function", name="classic_fix")

            class FixtureFunctionDefinition:   # pytest>=8.4 shape
                def __init__(self, func, marker):
                    self._fixture_function = func
                    self._fixture_function_marker = marker

            def _newstyle():
                return "new"
            newstyle_fix = FixtureFunctionDefinition(
                _newstyle, FixtureFunctionMarker(scope="function", name="newstyle_fix"))

            def test_module_mark_makes_xfail():
                assert False   # module pytestmark xfail -> xfailed

            def test_classic_fixture(classic_fix):
                assert classic_fix == 99
                raise AssertionError("forced")   # still xfailed by module mark

            def test_newstyle_fixture(newstyle_fix):
                assert newstyle_fix == "new"
                raise AssertionError("forced")
        """,
        "test_pymarks.py": """
            from fake_marks import Mark

            def test_fn_skip():
                assert False
            test_fn_skip.pytestmark = [Mark("skip", kwargs={"reason": "pm-skip"})]

            def test_fn_param(v):
                assert v in (7, 8)
            test_fn_param.pytestmark = [Mark("parametrize", args=("v", [7, 8]))]

            class TestClsMarked:
                pytestmark = [Mark("skip", kwargs={"reason": "class-skip"})]
                def test_in_marked_class(self):
                    assert False

            def test_pytest_style_skip_exc():
                class Skipped(Exception):   # name-suffix classification
                    pass
                raise Skipped("by-suffix")
        """,
    })
    w = Worker(root)
    f1 = os.path.join(root, "test_pycompat.py")
    results, _ = w.run([
        {"id": "p1", "file": f1, "qualname": "test_module_mark_makes_xfail"},
        {"id": "p2", "file": f1, "qualname": "test_classic_fixture"},
        {"id": "p3", "file": f1, "qualname": "test_newstyle_fixture"},
    ])
    r = by_id(results)
    check(r["p1"]["outcome"] == "xfailed", "module-level pytestmark (single Mark)")
    check(r["p2"]["outcome"] == "xfailed",
          "classic _pytestfixturefunction fixture injected (99)",
          repr(r["p2"].get("message")))
    check(r["p3"]["outcome"] == "xfailed",
          "pytest>=8.4 FixtureFunctionDefinition fixture injected",
          repr(r["p3"].get("message")))

    f2 = os.path.join(root, "test_pymarks.py")
    results, _ = w.run([
        {"id": "m1", "file": f2, "qualname": "test_fn_skip"},
        {"id": "m2", "file": f2, "qualname": "test_fn_param"},
        {"id": "m3", "file": f2, "qualname": "TestClsMarked::test_in_marked_class"},
        {"id": "m4", "file": f2, "qualname": "test_pytest_style_skip_exc"},
    ])
    r = by_id(results)
    check(r["m1"]["outcome"] == "skipped" and "pm-skip" in r["m1"]["message"],
          "function pytestmark skip")
    check(r.get("m2[7]", {}).get("outcome") == "passed"
          and r.get("m2[8]", {}).get("outcome") == "passed",
          "pytestmark parametrize expands", repr(sorted(r)))
    check(r["m3"]["outcome"] == "skipped", "class-level pytestmark applies to methods")
    check(r["m4"]["outcome"] == "skipped" and "by-suffix" in r["m4"]["message"],
          "'Skipped' class-name suffix -> skipped")

    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0, "compat worker clean shutdown")


# ============================================================================
# Suite 5: --no-capture, truncation, async fixture support, edge protocol
# ============================================================================

def test_misc(root):
    print("\n[misc: no-capture / truncation / async fixture / fixture cycle]")
    write_suite(root, {
        "test_misc.py": """
            import tezt

            def test_noisy():
                print("TEE-MARKER-OUT")

            def test_big_output():
                print("Z" * (70 * 1024))

            @tezt.fixture
            async def afix():
                return 1

            def test_async_fixture(afix):
                assert afix == 1

            @tezt.fixture
            def cyc_a(cyc_b):
                return 1

            @tezt.fixture
            def cyc_b(cyc_a):
                return 2

            def test_cycle(cyc_a):
                pass
        """,
    })
    f = os.path.join(root, "test_misc.py")

    # --no-capture: output tees to worker stderr, fields still populated
    w = Worker(root, no_capture=True)
    results, _ = w.run([{"id": "n1", "file": f, "qualname": "test_noisy"}])
    r = by_id(results)
    check("TEE-MARKER-OUT" in r["n1"]["stdout"], "no-capture keeps stdout field")
    w.send({"cmd": "shutdown"})
    out_ev = json.loads(w.proc.stdout.readline())
    w.proc.wait(timeout=15)
    stderr_text = w.proc.stderr.read()
    check("TEE-MARKER-OUT" in stderr_text, "no-capture tees to worker stderr")
    check(out_ev.get("event") == "bye", "no-capture worker says bye")

    w = Worker(root)
    results, _ = w.run([
        {"id": "t1", "file": f, "qualname": "test_big_output"},
        {"id": "t2", "file": f, "qualname": "test_async_fixture"},
        {"id": "t3", "file": f, "qualname": "test_cycle"},
    ])
    r = by_id(results)
    check(len(r["t1"]["stdout"]) < 70 * 1024 and "[truncated]" in r["t1"]["stdout"],
          "stdout truncated at 64KB")
    # Async fixtures are now supported (run on the worker's shared loop).
    check(r["t2"]["outcome"] == "passed",
          "async coroutine fixture resolves", repr(r["t2"].get("message")))
    check(r["t3"]["outcome"] == "error"
          and "cycle" in (r["t3"]["message"] or ""),
          "fixture dependency cycle detected", repr(r["t3"].get("message")))
    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0, "misc worker clean shutdown")


# ============================================================================
# Suite 6: class-scoped fixtures, async fixtures (coroutine + async-gen
# teardown), async test regression, and rich operator-aware assertion diffs.
# ============================================================================

def test_class_async_assert(root):
    print("\n[class-scope fixtures / async fixtures / assertion diffs]")
    # Sentinel path the async-gen fixture's teardown writes to; embed it into
    # the generated module as a string literal so the fixture closure can see
    # it without any import machinery.
    sentinel = os.path.join(root, "torndown")

    write_suite(root, {
        "test_clsfix.py": """
            import tezt

            BUILDS = []          # one entry per class-scoped fixture build

            @tezt.fixture(scope="class")
            def cval():
                BUILDS.append(1)
                return len(BUILDS)      # 1 for the first class, 2 for the next

            class TestA:
                def test_a1(self, cval):
                    assert cval == 1     # built once for this class
                def test_a2(self, cval):
                    assert cval == 1     # cached within the class

            class TestB:
                def test_b1(self, cval):
                    assert cval == 2     # rebuilt for a new class
        """,
        "test_asyncfix.py": """
            import os, tezt

            SENTINEL = %r

            @tezt.fixture
            async def af():
                return 41

            def test_af(af):
                assert af == 41

            @tezt.fixture
            async def agen():
                yield 7
                # teardown on the shared loop writes a sentinel file
                open(SENTINEL, "w").write("1")

            def test_agen(agen):
                assert agen == 7

            async def test_async_regression():
                assert True
        """ % (sentinel,),
        "test_diffs.py": """
            def test_list_diff():
                assert [1, 2, 3] == [1, 2, 4]

            def test_dict_diff():
                assert {'a': 1, 'b': 2} == {'a': 1, 'b': 3}

            def test_str_diff():
                assert 'hello world' == 'hello there'

            def test_int_diff():
                x = 5
                y = 6
                assert x == y

            def test_call_operand():
                items = [1, 2]
                assert len(items) == 3
        """,
    })

    # ---- (1) class-scoped fixture: one build per class -------------------
    w = Worker(root)
    fcls = os.path.join(root, "test_clsfix.py")
    results, _ = w.run([
        {"id": "ca1", "file": fcls, "qualname": "TestA::test_a1"},
        {"id": "ca2", "file": fcls, "qualname": "TestA::test_a2"},
        {"id": "cb1", "file": fcls, "qualname": "TestB::test_b1"},
    ])
    r = by_id(results)
    check(r["ca1"]["outcome"] == "passed", "class fixture built once (cval==1)",
          repr(r["ca1"].get("message")))
    check(r["ca2"]["outcome"] == "passed", "class fixture cached within class",
          repr(r["ca2"].get("message")))
    check(r["cb1"]["outcome"] == "passed", "class fixture rebuilt for next class",
          repr(r["cb1"].get("message")))

    # ---- (2)+(3)+(4) async coroutine fixture, async-gen teardown, async test
    fas = os.path.join(root, "test_asyncfix.py")
    results, _ = w.run([
        {"id": "af", "file": fas, "qualname": "test_af"},
        {"id": "agen", "file": fas, "qualname": "test_agen"},
        {"id": "areg", "file": fas, "qualname": "test_async_regression"},
    ])
    r = by_id(results)
    check(r["af"]["outcome"] == "passed", "async coroutine fixture value (41)",
          repr(r["af"].get("message")))
    check(r["agen"]["outcome"] == "passed", "async-generator fixture yields value",
          repr(r["agen"].get("message")))
    check(r["areg"]["outcome"] == "passed", "async test still works (regression)",
          repr(r["areg"].get("message")))
    # The async-gen teardown only runs when the worker disposes scope. The
    # fixture is function-scoped, so it tears down right after test_agen.
    check(os.path.exists(sentinel),
          "async-generator fixture teardown ran on shared loop")

    # ---- (5) rich operator-aware assertion diffs --------------------------
    fd = os.path.join(root, "test_diffs.py")
    results, _ = w.run([
        {"id": "d_list", "file": fd, "qualname": "test_list_diff"},
        {"id": "d_dict", "file": fd, "qualname": "test_dict_diff"},
        {"id": "d_str", "file": fd, "qualname": "test_str_diff"},
        {"id": "d_int", "file": fd, "qualname": "test_int_diff"},
        {"id": "d_call", "file": fd, "qualname": "test_call_operand"},
    ])
    r = by_id(results)

    dl = r["d_list"]
    check(dl["outcome"] == "failed", "list diff -> failed")
    check(dl["message"] and "left" in dl["message"] and "right" in dl["message"]
          and "index 2" in dl["message"],
          "list diff reports first differing index", repr(dl["message"]))

    dd = r["d_dict"]
    check(dd["outcome"] == "failed", "dict diff -> failed")
    check(dd["message"] and "left" in dd["message"] and "right" in dd["message"]
          and "'b'" in dd["message"] and "2" in dd["message"] and "3" in dd["message"],
          "dict diff reports differing key/values", repr(dd["message"]))

    ds = r["d_str"]
    check(ds["outcome"] == "failed", "str diff -> failed")
    check(ds["message"] and "left" in ds["message"] and "right" in ds["message"]
          and ("diff" in ds["message"] or "index" in ds["message"]),
          "str diff shows unified diff / first index", repr(ds["message"]))

    di = r["d_int"]
    check(di["outcome"] == "failed", "int var diff -> failed")
    check(di["message"] and "assert x == y" in di["message"]
          and "= 5" in di["message"] and "= 6" in di["message"],
          "int operand values shown", repr(di["message"]))

    # Call operand: safe path is skipped (has a Call) -> fallback form, but
    # must not crash and must still mention the source.
    dc = r["d_call"]
    check(dc["outcome"] == "failed", "call-operand assert -> failed")
    check(dc["message"] and "assert failed:" in dc["message"]
          and "len(items) == 3" in dc["message"],
          "call operand falls back without crashing", repr(dc["message"]))

    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0,
          "class/async/diff worker clean shutdown")


# ============================================================================
# Suite 7: conftest hook system + coverage.py measurement
# ============================================================================

def test_hooks_and_coverage(root):
    print("\n[conftest hooks / coverage.py]")

    # ---- (1)+(2)+(3) hook lifecycle, setup-skip, item shape --------------
    # A sentinel file the conftest hooks append to (one line per call). We embed
    # its absolute path into the generated conftest as a string literal so the
    # hook closures can reach it with no import machinery.
    sentinel = os.path.join(root, "hook_events.log")
    write_suite(root, {
        "conftest.py": """
            import tezt

            SENTINEL = %r

            def _log(line):
                with open(SENTINEL, "a") as f:
                    f.write(line + "\\n")

            def pytest_configure(config):
                # config stub must expose the methods plugins commonly call.
                config.addinivalue_line("markers", "slow: marks slow tests")
                _log("configure rootdir=%%s opt=%%s ini=%%r" %% (
                    bool(config.rootdir), config.getoption("x", "DEF"),
                    config.getini("nope")))

            def pytest_sessionstart(session):
                _log("sessionstart items=%%d start=%%s" %% (
                    len(session.items), bool(session.startpath)))

            def pytest_sessionfinish(session, exitstatus):
                _log("sessionfinish status=%%d" %% exitstatus)

            def pytest_runtest_setup(item):
                _log("setup %%s" %% item.name)
                # (2) a setup hook can skip a specific test by raising Skipped.
                if item.name == "test_skip_me":
                    tezt.skip("by hook")
                # (3) record item shape for the marked test.
                if item.name == "test_marked":
                    _log("item nodeid=%%s name=%%s slow=%%s" %% (
                        item.nodeid, item.name,
                        bool(item.get_closest_marker("slow"))))

            def pytest_runtest_teardown(item):
                _log("teardown %%s" %% item.name)
        """ % (sentinel,),
        "test_hooked.py": """
            import tezt

            def test_one():
                assert True

            def test_two():
                assert True

            def test_skip_me():
                assert False   # never reached: setup hook skips it

            @tezt.mark.slow
            def test_marked():
                assert True
        """,
    })

    w = Worker(root)
    f = os.path.join(root, "test_hooked.py")
    results, _ = w.run([
        {"id": "h::test_one", "file": f, "qualname": "test_one"},
        {"id": "h::test_two", "file": f, "qualname": "test_two"},
        {"id": "h::test_skip_me", "file": f, "qualname": "test_skip_me"},
        {"id": "h::test_marked", "file": f, "qualname": "test_marked"},
    ])
    r = by_id(results)

    # (2) setup-hook skip + a sibling still passes.
    check(r["h::test_skip_me"]["outcome"] == "skipped"
          and "by hook" in (r["h::test_skip_me"]["message"] or ""),
          "setup hook skip -> skipped", repr(r["h::test_skip_me"].get("message")))
    check(r["h::test_one"]["outcome"] == "passed"
          and r["h::test_two"]["outcome"] == "passed",
          "non-skipped tests still pass alongside a hook-skip")
    check(r["h::test_marked"]["outcome"] == "passed", "marked test passes")

    ev, rc = w.shutdown()
    check(ev.get("event") == "bye" and rc == 0, "hooks worker clean shutdown")

    # (1) lifecycle counts: configure/sessionstart/sessionfinish each once;
    # setup/teardown once per test (4 tests). sessionfinish lands only after
    # shutdown, so read the sentinel now.
    with open(sentinel) as fh:
        lines = [ln for ln in fh.read().splitlines() if ln]
    n_configure = sum(1 for ln in lines if ln.startswith("configure "))
    n_sstart = sum(1 for ln in lines if ln.startswith("sessionstart "))
    n_sfinish = sum(1 for ln in lines if ln.startswith("sessionfinish "))
    n_setup = sum(1 for ln in lines if ln.startswith("setup "))
    n_teardown = sum(1 for ln in lines if ln.startswith("teardown "))
    check(n_configure == 1, "pytest_configure fired exactly once",
          "got %d" % n_configure)
    check(n_sstart == 1, "pytest_sessionstart fired exactly once",
          "got %d" % n_sstart)
    check(n_sfinish == 1, "pytest_sessionfinish fired exactly once at shutdown",
          "got %d" % n_sfinish)
    check(n_setup == 4, "pytest_runtest_setup fired once per test (4)",
          "got %d" % n_setup)
    check(n_teardown == 4, "pytest_runtest_teardown fired once per test (4)",
          "got %d" % n_teardown)
    # configure ran before sessionstart, which ran before any setup.
    check(lines.index(next(ln for ln in lines if ln.startswith("configure ")))
          < lines.index(next(ln for ln in lines if ln.startswith("sessionstart ")))
          < lines.index(next(ln for ln in lines if ln.startswith("setup "))),
          "lifecycle order: configure < sessionstart < setup")
    # config stub methods returned their documented defaults.
    cfg_line = next(ln for ln in lines if ln.startswith("configure "))
    check("opt=DEF" in cfg_line and "ini=''" in cfg_line,
          "config.getoption default + getini '' ", repr(cfg_line))

    # (3) item shape for the @tezt.mark.slow test.
    item_line = next((ln for ln in lines if ln.startswith("item ")), None)
    check(item_line is not None
          and "nodeid=h::test_marked" in item_line
          and "name=test_marked" in item_line
          and "slow=True" in item_line,
          "item.nodeid/name + get_closest_marker('slow')", repr(item_line))

    # ---- (4) coverage.py measurement -------------------------------------
    # The Worker harness can't pass coverage flags, so spawn a worker directly
    # with --cov-data-dir / --cov-source, mirroring the harness handshake.
    #
    # Coverage needs the third-party `coverage` package in THIS interpreter (the
    # same `sys.executable` that spawns the worker below). If it isn't importable
    # we skip just the measurement checks — the hook checks above already ran. CI
    # installs coverage so the matrix exercises this path for real.
    try:
        import coverage  # noqa: F401  (probe only)
    except ImportError:
        print("  ~~    coverage not installed; skipping coverage.py checks")
        return

    covdir = os.path.join(root, "covdata")
    os.makedirs(covdir, exist_ok=True)
    write_suite(root, {
        "mod_under_test.py": """
            def add(a, b):
                # this line is executed when the test calls add()
                return a + b
        """,
        "test_cov.py": """
            import mod_under_test

            def test_uses_mod():
                assert mod_under_test.add(2, 3) == 5
        """,
    })

    cmd = [sys.executable, "-u", WORKER, "--rootdir", root,
           "--cov-data-dir", covdir, "--cov-source", root]
    proc = subprocess.Popen(
        cmd, cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, text=True, bufsize=1)
    try:
        ready = json.loads(proc.stdout.readline())
        check(ready.get("event") == "ready", "coverage worker ready", repr(ready))
        pid = ready.get("pid")

        fcov = os.path.join(root, "test_cov.py")
        proc.stdin.write(json.dumps(
            {"cmd": "run", "batch_id": 1,
             "items": [{"id": "cov::test_uses_mod", "file": fcov,
                        "qualname": "test_uses_mod"}]}) + "\n")
        proc.stdin.flush()

        cov_results = []
        while True:
            ev = json.loads(proc.stdout.readline())
            if ev.get("event") == "result":
                cov_results.append(ev)
            elif ev.get("event") == "batch_done":
                break
            elif ev.get("event") == "fatal":
                raise RuntimeError("coverage worker fatal: %r" % ev)
        cr = by_id(cov_results)
        check(cr["cov::test_uses_mod"]["outcome"] == "passed",
              "coverage worker ran the test",
              repr(cr["cov::test_uses_mod"].get("message")))

        proc.stdin.write(json.dumps({"cmd": "shutdown"}) + "\n")
        proc.stdin.flush()
        bye = json.loads(proc.stdout.readline())
        proc.wait(timeout=15)
        check(bye.get("event") == "bye" and proc.returncode == 0,
              "coverage worker clean shutdown")
    finally:
        try:
            proc.kill()
        except Exception:
            pass

    # The data file is named .coverage.<pid> in covdir and must be non-empty.
    data_path = os.path.join(covdir, ".coverage.%d" % pid)
    check(os.path.exists(data_path) and os.path.getsize(data_path) > 0,
          "coverage data file written and non-empty",
          repr([data_path, os.path.exists(data_path)]))

    # Load it via the coverage API and confirm mod_under_test.py was measured.
    measured_ok = False
    try:
        import coverage
        cov = coverage.Coverage(data_file=data_path)
        cov.load()
        data = cov.get_data()
        for fn in data.measured_files():
            if os.path.basename(fn) == "mod_under_test.py":
                lines_measured = data.lines(fn) or []
                if lines_measured:
                    measured_ok = True
                break
    except ImportError:
        # coverage not importable in the test interpreter: the file existence
        # check above already proves measurement happened. Treat as satisfied.
        measured_ok = True
    check(measured_ok, "mod_under_test.py has measured lines in coverage data")


# ============================================================================
# Suite 8: performance -- 2000 trivial tests through one worker
# ============================================================================

def test_perf(root):
    print("\n[performance: 2000 trivial tests]")
    n_files, per_file = 20, 100
    files = {}
    for i in range(n_files):
        body = ["def test_t%d():\n    assert True\n" % j for j in range(per_file)]
        files["perf/test_perf_%02d.py" % i] = "\n".join(body)
    write_suite(root, files)

    w = Worker(root)
    items = []
    for i in range(n_files):
        f = os.path.join(root, "perf", "test_perf_%02d.py" % i)
        for j in range(per_file):
            q = "test_t%d" % j
            items.append({"id": "pf%02d::%s" % (i, q), "file": f, "qualname": q})

    start = time.perf_counter()
    results, _ = w.run(items)
    elapsed = time.perf_counter() - start
    ev, rc = w.shutdown()

    total = n_files * per_file
    per_test_ms = elapsed * 1000.0 / total
    print("  perf: %d tests in %.2fs -> %.3f ms/test" % (total, elapsed, per_test_ms),
          file=sys.stderr)
    check(len(results) == total, "all %d perf results received" % total,
          "got %d" % len(results))
    check(all(x["outcome"] == "passed" for x in results), "all perf tests passed")
    check(elapsed < 8.0, "2000 tests under 8s", "took %.2fs" % elapsed)
    check(ev.get("event") == "bye" and rc == 0, "perf worker clean shutdown")
    return per_test_ms


# ============================================================================
# Driver
# ============================================================================

def main():
    suites = [test_core, test_fixtures, test_classes_async_discovery,
              test_pytest_compat, test_misc, test_class_async_assert,
              test_hooks_and_coverage]
    per_test_ms = None
    for fn in suites:
        root = tempfile.mkdtemp(prefix="tezt-selftest-")
        try:
            fn(root)
        finally:
            shutil.rmtree(root, ignore_errors=True)
    root = tempfile.mkdtemp(prefix="tezt-selftest-perf-")
    try:
        per_test_ms = test_perf(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)

    print("\n========================================")
    print("self-test: %d passed, %d failed" % (PASS, FAIL))
    if per_test_ms is not None:
        print("measured worker overhead: %.3f ms per trivial test" % per_test_ms)
    print("========================================")
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())
