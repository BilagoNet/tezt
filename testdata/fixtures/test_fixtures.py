"""fixtures suite: function/module/session scopes, conftest chain,
yield-fixture teardown, builtin tmp_path. All 7 tests pass.

Teardown-order assertions are self-contained: each assertion only
inspects state that is fully determined by the time it runs, never
relying on teardown of a *later* test.
"""
import os

import tezt

MODULE_FIXTURE_CALLS = {"count": 0}


@tezt.fixture(scope="module")
def module_resource():
    MODULE_FIXTURE_CALLS["count"] += 1
    return {"name": "shared", "instantiations": MODULE_FIXTURE_CALLS["count"]}


@tezt.fixture
def number():
    return 41


@tezt.fixture
def number_plus_one(number):
    # fixture depending on another fixture
    return number + 1


@tezt.fixture
def file_with_teardown(tmp_path):
    """Yield fixture that records setup/teardown order into a tmp file."""
    log = tmp_path / "order.log"
    with open(str(log), "a") as f:
        f.write("setup\n")
    yield log
    with open(str(log), "a") as f:
        f.write("teardown\n")


def test_conftest_fixture(conftest_greeting):
    assert conftest_greeting == "hello from conftest"


def test_module_fixture_value(module_resource):
    assert module_resource["name"] == "shared"
    # module scope: created at most once for this module
    assert module_resource["instantiations"] == 1


def test_module_fixture_cached(module_resource):
    # second test using it must get the cached instance, not a new one
    assert MODULE_FIXTURE_CALLS["count"] == 1
    assert module_resource["instantiations"] == 1


def test_fixture_chain(number_plus_one):
    assert number_plus_one == 42


def test_yield_fixture_setup_visible(file_with_teardown):
    # During the test body, only setup has been written; teardown not yet.
    content = open(str(file_with_teardown)).read()
    assert content == "setup\n"


def test_tmp_path_is_writable(tmp_path):
    p = tmp_path / "data.txt"
    p.write_text("payload")
    assert p.read_text() == "payload"
    assert os.path.isdir(str(tmp_path))


def test_session_log_order(event_log, logged_resource):
    # Self-contained: our own logged_resource setup appended 'setup' as the
    # most recent event. Prior tests (if any used it) contributed balanced
    # setup/teardown pairs before ours.
    assert logged_resource == "resource"
    assert event_log[-1] == "setup"
    assert event_log.count("setup") == event_log.count("teardown") + 1
