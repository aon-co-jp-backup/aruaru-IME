# Dev deploy: build into target-dev, then swap the engine/CLI/DLL into target\release (the registered location)
# without needing to close running apps (a loaded file can be renamed, then replaced).
# Usage: powershell -File tools\deploy-dev.ps1 [-Dll]   (-Dll also swaps the TSF DLL; apps pick it up on next launch)
param([switch]$Dll)
$ErrorActionPreference = 'Stop'
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$root = Split-Path $PSScriptRoot -Parent
$env:CARGO_TARGET_DIR = "$root\target-dev"
Push-Location $root
cargo build --release -p aruaru-ime-daemon -p aruaru-ime-cli -p aruaru-ime-tsf
if ($LASTEXITCODE -ne 0) { Pop-Location; throw "build failed" }
$rel = "$root\target\release"; $new = "$root\target-dev\release"; $stamp = Get-Date -Format HHmmss
$files = @('aruaru-ime-daemon.exe', 'aruaru-ime.exe'); if ($Dll) { $files += 'aruaru_ime_tsf.dll' }
foreach ($n in $files) {
    if (Test-Path "$rel\$n") { Rename-Item -LiteralPath "$rel\$n" -NewName "$n.old-$stamp" -ErrorAction SilentlyContinue }
    Copy-Item "$new\$n" "$rel\$n" -Force
}
Get-Process aruaru-ime-daemon -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
Start-Process "$rel\aruaru-ime-daemon.exe" -WorkingDirectory C:\Windows\System32 -WindowStyle Hidden
Start-Sleep -Seconds 5
try { (Invoke-WebRequest http://127.0.0.1:4660/health -UseBasicParsing -TimeoutSec 5).Content } catch { "daemon DOWN" }
Pop-Location
