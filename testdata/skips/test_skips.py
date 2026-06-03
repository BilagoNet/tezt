"""skips suite. Expected outcomes (6 tests):

- test_marked_skip            -> skipped
- test_skipif_true            -> skipped
- test_skipif_false           -> passed (condition False, runs normally)
- test_xfail_that_fails       -> xfailed (expected failure happened)
- test_xfail_that_passes      -> xpassed (unexpectedly passed)
- test_imperative_skip        -> skipped (tezt.skip() in body)

Summary: 1 passed, 3 skipped, 1 xfailed, 1 xpassed. Exit code 0
(xpassed is non-strict, does not fail the run).
"""
import tezt


@tezt.mark.skip(reason="always skipped")
def test_marked_skip():
    raise AssertionError("must never run")


@tezt.mark.skipif(True, reason="condition is true")
def test_skipif_true():
    raise AssertionError("must never run")


@tezt.mark.skipif(False, reason="condition is false; runs")
def test_skipif_false():
    assert 1 == 1


@tezt.mark.xfail(reason="known bug: off-by-one")
def test_xfail_that_fails():
    assert 1 + 1 == 3


@tezt.mark.xfail(reason="suspected bug that is actually fixed")
def test_xfail_that_passes():
    assert 1 + 1 == 2


def test_imperative_skip():
    tezt.skip("skipping from inside the body")
    raise AssertionError("unreachable")
