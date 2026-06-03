"""kfilter suite: 3 passing tests with distinctive names for -k filtering.

Test ids:
- test_names.py::test_alpha
- test_names.py::test_beta
- test_names.py::TestGamma::test_delta

-k examples:
  -k alpha            -> 1 test
  -k "alpha or beta"  -> 2 tests
  -k "not alpha"      -> 2 tests
  -k Gamma            -> 1 test (class name matches)
  -k delta            -> 1 test
"""


def test_alpha():
    assert "a" in "alpha"


def test_beta():
    assert "b" in "beta"


class TestGamma:
    def test_delta(self):
        assert "d" in "delta"
