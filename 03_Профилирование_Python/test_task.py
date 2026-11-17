import random
import pytest
from hypothesis import given, strategies as st
import original
import optimized


@pytest.mark.parametrize('n', [-50, 0, 50, 200, 400, 600, 1000])
def test_run_task(n):
    random.seed(42)
    expected = original.run_task(n)
    random.seed(42)
    assert optimized.run_task(n) == expected


@given(st.integers(min_value=-5, max_value=22))
def test_fibonacci(n):
    assert optimized.fib(n) == original.fib(n)


def test_nested_sum():
    assert optimized.deep_sum({'a': [1, {'b': 2.5}], 'c': 'skip'}) == 3.5
