import csv
import importlib.util
from pathlib import Path
from unittest.mock import patch

path = Path(__file__).parents[1] / 'services' / 'legacy-python' / 'legacy.py'
spec = importlib.util.spec_from_file_location('legacy', path)
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)


def test_env(monkeypatch):
    monkeypatch.setenv('TEST_LEGACY_ENV', '')
    assert legacy.GetEnvDef('TEST_LEGACY_ENV', 'default') == 'default'
    monkeypatch.setenv('TEST_LEGACY_ENV', 'value')
    assert legacy.GetEnvDef('TEST_LEGACY_ENV', 'default') == 'value'


def test_random_bounds():
    for _ in range(100):
        assert 3.2 <= legacy.RandFloat(3.2, 12.6) < 12.6


def test_csv_and_copy_contract(tmp_path, monkeypatch):
    monkeypatch.setenv('CSV_OUT_DIR', str(tmp_path))
    with patch.object(legacy.subprocess, 'run') as run:
        output = legacy.GenerateAndCopy()
        arguments = run.call_args.args[0]
        assert arguments[:3] == ['psql', '-v', 'ON_ERROR_STOP=1']
        assert 'FROM STDIN' in arguments[-1]
        assert run.call_args.kwargs['check'] is True
        assert 'shell' not in run.call_args.kwargs
    with output.open(encoding='utf-8', newline='') as stream:
        rows = list(csv.DictReader(stream))
    assert len(rows) == 1
    assert list(rows[0]) == ['recorded_at', 'voltage', 'temp', 'source_file']
    assert 3.2 <= float(rows[0]['voltage']) <= 12.6
    assert -50 <= float(rows[0]['temp']) <= 80
    assert rows[0]['source_file'] == output.name
