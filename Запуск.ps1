$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
Write-Host '1 — Рефакторинг C# (CSV)'
Write-Host '2 — Оптимизация C# (Фибоначчи)'
Write-Host '3 — Профилирование Python'
Write-Host '4 — Оптимизация памяти Python'
Write-Host '5 — Микросервисы (способы оплаты)'
Write-Host '6 — Космический дашборд'
$choice = Read-Host 'Номер папки'
switch ($choice) {
    '1' { Set-Location '01_CSharp'; & 'C:\Program Files\dotnet\dotnet.exe' run -c Release -- --no-pause }
    '2' { Set-Location '02_CSharp'; & 'C:\Program Files\dotnet\dotnet.exe' run -c Release }
    '3' { Set-Location '03_Профилирование_Python'; & '..\.venv\Scripts\python.exe' -m pytest -q; & '..\.venv\Scripts\python.exe' '..\benchmark_python.py' time }
    '4' { Set-Location '04_Память_Python'; & '..\.venv\Scripts\python.exe' -m pytest -q; & '..\.venv\Scripts\python.exe' '..\benchmark_python.py' memory }
    '5' { & '.\05_Микросервисы\setup.ps1' }
    '6' { & '.\06_Космический_дашборд\setup.ps1' }
    default { Write-Host 'Нужно выбрать от 1 до 6.' }
}
Read-Host 'Enter — закрыть'
