"""classes suite: Test* classes with xunit-style setup hooks.

All 6 tests pass. Assertions about setup hooks are order-independent:
- setup_method must run before each test method (instance flag check)
- setup_class must run once before any method (class flag check)
- a class-level counter tracks setup_method invocations; each test only
  asserts the counter is >= 1 and equals the number of torn-down tests + 1,
  which holds regardless of method execution order.
"""


class TestCounter:
    setup_calls = 0
    teardown_calls = 0
    class_set_up = False

    @classmethod
    def setup_class(cls):
        cls.class_set_up = True

    def setup_method(self, method):
        TestCounter.setup_calls += 1
        self.ready = True
        self.value = 10

    def teardown_method(self, method):
        TestCounter.teardown_calls += 1

    def test_setup_ran(self):
        assert self.ready is True
        assert TestCounter.class_set_up is True

    def test_setup_fresh_state(self):
        # setup_method gives every test a fresh value.
        assert self.value == 10
        self.value = 99  # mutate; must not leak into other tests

    def test_setup_counter_consistent(self):
        # setup ran exactly once more than teardown at this point,
        # no matter which order the test methods execute in.
        assert TestCounter.setup_calls == TestCounter.teardown_calls + 1
        assert TestCounter.setup_calls >= 1


class TestSimple:
    def test_method_one(self):
        assert "abc".upper() == "ABC"

    def test_method_two(self):
        assert len([1, 2, 3]) == 3


class NotATestClass:
    """Not collected: name does not start with Test."""

    def test_ignored(self):
        raise AssertionError("must never run")


class TestWithInit:
    """Not collected: has __init__ (pytest parity)."""

    def __init__(self):
        self.x = 1

    def test_also_ignored(self):
        raise AssertionError("must never run")


def test_module_level_alongside_classes():
    assert True
