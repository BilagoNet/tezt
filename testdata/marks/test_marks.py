"""Suite for `-m` mark-expression selection.

Uses `import tezt` (the worker's virtual module) so the suite runs without
pytest installed. Marks are read statically from the decorators at collection
time, so selection works with no Python import at all.
"""

import tezt


@tezt.mark.slow
def test_slow_one():
    assert True


@tezt.mark.slow
@tezt.mark.net
def test_slow_and_net():
    assert True


@tezt.mark.net
def test_net_only():
    assert True


def test_unmarked():
    assert True
