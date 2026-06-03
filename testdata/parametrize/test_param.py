"""parametrize suite. Case count:

- test_square:     5 cases (single arg)
- test_add:        3 cases (multi-arg tuples)
- test_concat:     6 cases (cartesian: 2 x 3 via stacked decorators)
- test_is_even:    2 cases (ids= override -> test_is_even[two], [four])

Total collected: 16, all passing. Exit code 0.
"""
import tezt


@tezt.parametrize("n", [0, 1, 2, 3, 10])
def test_square(n):
    assert n * n == n ** 2


@tezt.parametrize("a,b,expected", [(1, 2, 3), (2, 3, 5), (10, -4, 6)])
def test_add(a, b, expected):
    assert a + b == expected


@tezt.parametrize("prefix", ["x", "y"])
@tezt.parametrize("suffix", ["1", "2", "3"])
def test_concat(prefix, suffix):
    combined = prefix + suffix
    assert combined.startswith(prefix)
    assert combined.endswith(suffix)
    assert len(combined) == 2


@tezt.parametrize("value", [2, 4], ids=["two", "four"])
def test_is_even(value):
    assert value % 2 == 0
