"""basic suite: exactly 3 passing tests and 1 failing test."""


def test_addition():
    assert 1 + 1 == 2


def test_subtraction():
    assert 5 - 3 == 2


def test_multiplication():
    assert 3 * 4 == 12


def test_wrong_addition():
    # Intentionally failing: 2 + 2 is not 5.
    assert 2 + 2 == 5
