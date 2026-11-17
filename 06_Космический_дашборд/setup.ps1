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
if ($LASTEXITCODE -ne 0) { throw 'Проверь Docker Desktop и WSL.' }
