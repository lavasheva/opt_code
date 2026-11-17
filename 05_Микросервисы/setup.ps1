$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
if (!(Get-Command docker -ErrorAction SilentlyContinue)) {
    $env:Path = "$env:ProgramFiles\Docker\Docker\resources\bin;" + $env:Path
}
if (!(Test-Path -LiteralPath '.env')) {
    $password = [Guid]::NewGuid().ToString('N')
    "POSTGRES_PASSWORD=$password" | Set-Content -Encoding ascii '.env'
}
docker compose up --build -d
if ($LASTEXITCODE -ne 0) { throw 'Docker не запустился. Проверь Docker Desktop и WSL.' }

for ($attempt = 0; $attempt -lt 45; $attempt++) {
    try {
        Invoke-RestMethod -Uri 'http://localhost:8000/health' -TimeoutSec 2 | Out-Null
        Write-Host 'Готово: API http://localhost:8000'
        return
    } catch { Start-Sleep -Seconds 1 }
}
throw 'Контейнеры созданы, но API не ответил. Посмотри docker compose logs.'

