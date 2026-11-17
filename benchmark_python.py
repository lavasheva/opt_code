"""Запускать из папки конкретной Python-работы: python ../benchmark_python.py time|memory."""
import cProfile
import csv
import gc
import importlib
import io
from pathlib import Path
import pstats
import random
import statistics
import sys
import time
import tracemalloc

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from line_profiler import LineProfiler
from memory_profiler import memory_usage

sys.path.insert(0, str(Path.cwd()))


def reset(module):
    if hasattr(module, 'expensive_calc'):
        module.expensive_calc.cache_clear()
    random.seed(42)
    gc.collect()



def main():
    original = importlib.import_module('original')
    optimized = importlib.import_module('optimized')
    mode = sys.argv[1]
    out = Path('results')
    out.mkdir(exist_ok=True)
    sizes = [200, 400, 600, 800, 1000] if mode == 'time' else [1000, 5000, 10000, 20000, 40000]
    
    rows = []
    for n in sizes:
        row = {'N': n}
        values = []
        for label, module in [('before', original), ('after', optimized)]:
            times = []
            for _ in range(3):
                reset(module)
                start = time.perf_counter()
                result = module.run_task(n)
                times.append(time.perf_counter() - start)
            values.append(result)
            row[label + '_seconds'] = statistics.median(times)
            if mode == 'memory':
                reset(module)
                tracemalloc.start()
                module.run_task(n)
                current, peak = tracemalloc.get_traced_memory()
                snapshot = tracemalloc.take_snapshot()
                tracemalloc.stop()
                row[label + '_peak_MiB'] = peak / 2**20
                row[label + '_retained_MiB'] = current / 2**20
                if n == sizes[-1]:
                    (out / f'{label}_allocations.txt').write_text('\n'.join(map(str, snapshot.statistics('lineno')[:7])), encoding='utf-8')
        assert values[0] == values[1], (n, values)
        row['speedup'] = row['before_seconds'] / row['after_seconds']
        if mode == 'memory':
            row['memory_reduction'] = row['before_peak_MiB'] / row['after_peak_MiB']
        rows.append(row)
        print(row, flush=True)
    
    with (out / 'measurements.csv').open('w', newline='', encoding='utf-8-sig') as f:
        writer = csv.DictWriter(f, fieldnames=rows[0].keys())
        writer.writeheader()
        writer.writerows(rows)
    
    metric = 'seconds' if mode == 'time' else 'peak_MiB'
    for label in ['before', 'after']:
        plt.plot(sizes, [r[f'{label}_{metric}'] for r in rows], marker='o', label=label)
    plt.xlabel('N')
    plt.ylabel('Time, seconds' if mode == 'time' else 'Peak memory, MiB (tracemalloc)')
    plt.legend()
    plt.grid(alpha=.3)
    plt.tight_layout()
    plt.savefig(out / 'comparison.png', dpi=160)
    
    for label, module in [('before', original), ('after', optimized)]:
        reset(module)
        profiler = cProfile.Profile()
        profiler.runcall(module.run_task, sizes[-1])
        stream = io.StringIO()
        pstats.Stats(profiler, stream=stream).strip_dirs().sort_stats('cumulative').print_stats(10)
        (out / f'{label}_cprofile.txt').write_text(stream.getvalue(), encoding='utf-8')
        reset(module)
        if mode == 'time':
            line = LineProfiler(module.fib, module.deep_sum, module.build_dict)
            line.runcall(module.run_task, 200)
            stream = io.StringIO()
            line.print_stats(stream=stream)
            (out / f'{label}_line_profile.txt').write_text(stream.getvalue(), encoding='utf-8')
        else:
            # RSS включает интерпретатор и библиотеки, tracemalloc — Python-выделения.
            rss = memory_usage((module.run_task, (sizes[-1],)), interval=.01, max_iterations=1)
            (out / f'{label}_rss.txt').write_text(f'Min RSS: {min(rss):.3f} MiB\nPeak RSS: {max(rss):.3f} MiB\nSampling interval: 0.01 s\n', encoding='utf-8')


if __name__ == "__main__":
    main()
