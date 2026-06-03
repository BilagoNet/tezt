"""Test that fails by raising an uncaught ValueError."""


def parse_positive(text):
    value = int(text)
    if value <= 0:
        raise ValueError("expected a positive integer, got %d" % value)
    return value


def test_raises_value_error():
    parse_positive("-3")  # raises ValueError -> failed outcome
