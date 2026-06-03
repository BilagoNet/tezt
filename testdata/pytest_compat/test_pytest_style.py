"""pytest_compat suite, main file (pytest-style APIs).

This file: 10 passed, 2 skipped, 1 xfailed, 1 xpassed, 1 failed.
(See test_pytest_marked.py for 2 more skipped via pytestmark.)
"""
import pytest


def test_conftest_fixture(compat_greeting):
    assert compat_greeting == "hello from pytest conftest"


def test_yield_fixture(tracked_resource):
    # During the test the resource is open; teardown flips it afterwards.
    assert tracked_resource["open"] is True


@pytest.mark.parametrize("n,squared", [(1, 1), (2, 4), (3, 9), (4, 16)])
def test_squares(n, squared):
    assert n * n == squared


@pytest.mark.parametrize("word", ["ab", "cd"], ids=["first", "second"])
def test_word_length(word):
    assert len(word) == 2


def test_raises_with_match():
    with pytest.raises(ValueError, match="invalid literal"):
        int("not-a-number")


@pytest.mark.skip(reason="always skipped")
def test_marked_skip():
    raise AssertionError("must never run")


@pytest.mark.skipif(False, reason="condition false; runs")
def test_skipif_false():
    assert True


@pytest.mark.xfail(reason="known broken")
def test_xfail_failing():
    assert 0 == 1


@pytest.mark.xfail(reason="actually works now")
def test_xfail_passing():
    assert 1 == 1


def test_imperative_skip():
    pytest.skip("skipping imperatively")
    raise AssertionError("unreachable")


def test_compat_failing():
    # Intentional failure so the suite exercises a nonzero exit code.
    assert "left" == "right"
