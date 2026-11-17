"""Вариант 6. Убираем повторный расчёт Фибоначчи, сохраняя порядок суммирования."""
from original import build_dict, deep_sum


def fib(n: int) -> int:
    if n <= 1:
        return n
    previous, current = 0, 1
    for _ in range(2, n + 1):
        previous, current = current, previous + current
    return current


def run_task(n: int):
    fib_value = fib(min(n // 50 + 20, 32))
    nested = build_dict(min(4, n // 200 + 2), min(5, n // 100 + 2))
    return fib_value, round(deep_sum(nested), 5)


if __name__ == '__main__':
    import random
    random.seed(42)
    print(run_task(600))
