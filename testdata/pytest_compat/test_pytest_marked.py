"""pytestmark: module-level mark skips every test in this file (2 skipped)."""
import pytest

pytestmark = pytest.mark.skip(reason="entire module skipped via pytestmark")


def test_skipped_by_module_mark_one():
    raise AssertionError("must never run")


def test_skipped_by_module_mark_two():
    raise AssertionError("must never run")
