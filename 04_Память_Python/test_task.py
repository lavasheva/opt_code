import pytest
from hypothesis import given, strategies as st
import original
import optimized


@pytest.mark.parametrize('n', [-1, 0, 1, 19, 20, 21, 1000, 10_000])
def test_run_task(n):
    assert optimized.run_task(n) == original.run_task(n)


@given(st.integers(min_value=0, max_value=2000))
def test_same_result(n):
    assert optimized.run_task(n) == original.run_task(n)


def test_cache_is_bounded():
    optimized.expensive_calc.cache_clear()
    optimized.run_task(10_000)
    assert optimized.expensive_calc.cache_info().currsize <= 500
