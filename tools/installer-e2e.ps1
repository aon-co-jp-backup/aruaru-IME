# Installer end-to-end test (needs admin). Run elevated:
#   Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','tools\installer-e2e.ps1'
# Steps: install (silent) -> verify -> uninstall (silent) -> verify -> install again -> verify. Log: dist\installer-e2e.log
$ErrorActionPreference = 'Continue'
$root = Split-Path $PSScriptRoot -Parent
$setup = "$root\dist\aruaru-ime-windows-x86_64-setup.exe"
$log = "$root\dist\installer-e2e.log"
"" | Out-File $log -Encoding utf8
function L($m) { ("{0:HH:mm:ss} {1}" -f (Get-Date), $m) | Out-File $log -Append -Encoding utf8 }
function State($tag) {
    $app = "${env:ProgramFiles}\aruaru-IME"
    $r = [ordered]@{
        tag = $tag
        app_dir = (Test-Path $app)
        dll = (Test-Path "$app\aruaru_ime_tsf.dll")
        daemon_exe = (Test-Path "$app\aruaru-ime-daemon.exe")
        clsid = (Test-Path 'HKLM:\SOFTWARE\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}')
        run_key = [bool](Get-ItemProperty 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'aruaru-IME' -ErrorAction SilentlyContinue)
        dict_local = (Test-Path "$env:LOCALAPPDATA\aruaru\dict.bin")
    }
    L ($r.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ' '
}
Stop-Process -Name aruaru-ime-daemon -Force -ErrorAction SilentlyContinue
State 'before'
L 'install #1'
$p = Start-Process $setup -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/TASKS=""' -Wait -PassThru
L "install exit=$($p.ExitCode)"
Start-Sleep 5
State 'after-install'
try { L ("health: " + (Invoke-WebRequest http://127.0.0.1:4660/health -UseBasicParsing -TimeoutSec 5).Content) } catch { L "health: unreachable" }
$un = Get-ChildItem "${env:ProgramFiles}\aruaru-IME\unins*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($un) {
    L 'uninstall'
    $p = Start-Process $un.FullName -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART' -Wait -PassThru
    L "uninstall exit=$($p.ExitCode)"
    Start-Sleep 3
    State 'after-uninstall'
} else { L 'uninstaller not found' }
L 'install #2'
$p = Start-Process $setup -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/TASKS=""' -Wait -PassThru
L "install exit=$($p.ExitCode)"
Start-Sleep 5
State 'after-reinstall'
L 'done'
