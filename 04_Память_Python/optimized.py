"""Вариант 6. Храним 500 результатов вместо истории и неограниченного кэша."""
from functools import lru_cache


@lru_cache(maxsize=500)
def expensive_calc(x: int) -> float:
    # Число слагаемых зависит только от остатка. Порядок сложения тот же.
    return sum(i * 0.001 for i in range(x % 500 + 100))


def run_with_growing_cache(n: int) -> tuple[int, float]:
    total = 0.0
    for i in range(n):
        key = i % (n // 2 + 10)
        total += expensive_calc(key % 500)
    # Исходник добавлял по одному hist_i и по одной записи на каждый уникальный key.
    # Словарь снаружи недоступен, поэтому его размер можно посчитать без хранения.
    count = n + min(n, n // 2 + 10) if n > 0 else 0
    return count, total


def run_task(n: int):
    return run_with_growing_cache(n)


if __name__ == '__main__':
    print(run_task(10_000))
