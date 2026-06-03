"""conftest.py for the fixtures suite."""
import tezt


@tezt.fixture
def conftest_greeting():
    return "hello from conftest"


@tezt.fixture(scope="session")
def event_log():
    """Session-scoped mutable list shared by all tests in the session.

    Tests append events; assertions about ordering are self-contained
    within each test (a yield-fixture appends 'setup' before the test
    body runs, so the test can assert its own setup landed last).
    """
    return []


@tezt.fixture
def logged_resource(event_log):
    event_log.append("setup")
    yield "resource"
    event_log.append("teardown")
