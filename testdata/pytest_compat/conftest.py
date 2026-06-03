"""conftest.py for the pytest_compat suite (pytest-style APIs)."""
import pytest


@pytest.fixture
def compat_greeting():
    return "hello from pytest conftest"


@pytest.fixture
def tracked_resource():
    state = {"open": True}
    yield state
    state["open"] = False
