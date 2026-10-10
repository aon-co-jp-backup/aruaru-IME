# Build the Windows installer: release build into target-dev, then Inno Setup.
# Usage: powershell -File tools\build-installer.ps1
$ErrorActionPreference = 'Stop'
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$root = Split-Path $PSScriptRoot -Parent
$env:CARGO_TARGET_DIR = "$root\target-dev"
Push-Location $root
cargo build --release -p aruaru-ime-daemon -p aruaru-ime-cli -p aruaru-ime-tsf
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "build failed" }
if (-not (Test-Path "$root\data\dict.bin")) { Pop-Location; throw "data\dict.bin is missing (see README: build-dict)" }
$iscc = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $iscc) { Pop-Location; throw "Inno Setup 6 (ISCC.exe) not found" }
New-Item -ItemType Directory -Force "$root\dist" | Out-Null
& $iscc "$root\installer\aruaru-ime.iss"
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "ISCC failed" }
Get-ChildItem "$root\dist\*.exe" | ForEach-Object { "{0}  {1:N1} MB" -f $_.Name, ($_.Length / 1MB) }
Pop-Location
