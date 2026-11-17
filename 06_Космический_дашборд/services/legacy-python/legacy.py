"""Замена Pascal CLI. Сохраняет имена функций, диапазоны и CSV-контракт."""
import csv
from datetime import datetime, timezone
import os
from pathlib import Path
import random
import subprocess


def GetEnvDef(name, default):
    return os.getenv(name) or default


def RandFloat(minV, maxV):
    return minV + random.random() * (maxV - minV)


def GenerateAndCopy():
    directory = Path(GetEnvDef('CSV_OUT_DIR', '/data/csv'))
    directory.mkdir(parents=True, exist_ok=True)
    now = datetime.now(timezone.utc)
    filename = 'telemetry_' + now.strftime('%Y%m%d_%H%M%S') + '.csv'
    path = directory / filename
    with path.open('w', newline='', encoding='utf-8') as stream:
        writer = csv.writer(stream)
        writer.writerow(['recorded_at', 'voltage', 'temp', 'source_file'])
        writer.writerow([now.strftime('%Y-%m-%d %H:%M:%S'), f'{RandFloat(3.2,12.6):.2f}',
                         f'{RandFloat(-50,80):.2f}', filename])
    # COPY FROM STDIN не требует shell и не подставляет путь в SQL.
    with path.open('rb') as stream:
        subprocess.run(['psql', '-v', 'ON_ERROR_STOP=1', '-c',
            'COPY telemetry_legacy(recorded_at,voltage,temp,source_file) FROM STDIN WITH (FORMAT csv, HEADER true)'],
            stdin=stream, check=True)
    print(f'Imported {filename}', flush=True)
    return path


if __name__ == '__main__':
    GenerateAndCopy()
