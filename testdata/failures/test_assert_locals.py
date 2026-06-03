"""Failing assertion with interesting local variables (for failure report)."""


def test_assertion_with_locals():
    expected_total = 100
    items = [12, 30, 7]
    actual_total = sum(items)
    label = "cart-total"
    assert actual_total == expected_total, "totals diverge for %s" % label
