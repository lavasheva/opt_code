import sys, io
from pathlib import Path
from memory_profiler import profile
sys.path.insert(0,str(Path.cwd()))
import original,optimized
for label,module in [('before',original),('after',optimized)]:
 module.expensive_calc.cache_clear()
 stream=io.StringIO()
 profile(module.run_with_growing_cache,stream=stream)(1000)
 Path('results',label+'_memory_lines.txt').write_text(stream.getvalue(),encoding='utf-8')
