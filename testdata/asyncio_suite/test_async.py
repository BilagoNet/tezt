"""asyncio suite: 2 async passing, 1 async failing. Exit code 1."""
import asyncio


async def test_async_sleep_returns():
    result = await asyncio.sleep(0, result="done")
    assert result == "done"


async def test_async_gather():
    async def double(x):
        return x * 2

    values = await asyncio.gather(double(1), double(2), double(3))
    assert values == [2, 4, 6]


async def test_async_failing():
    value = await asyncio.sleep(0, result=10)
    assert value == 11  # intentional failure
