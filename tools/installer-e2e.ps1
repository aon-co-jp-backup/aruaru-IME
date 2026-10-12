# Installer end-to-end test (needs admin). Run elevated, with a step as argument:
#   Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','tools\installer-e2e.ps1','-Step','install'
# Steps: install | uninstall | status.  An IME DLL that is loaded by running apps can only be replaced/removed after a reboot,
# so run the steps across reboots: install -> (use it) -> reboot -> uninstall -> reboot -> install -> status.
# Log: dist\installer-e2e.log (appended)
param([ValidateSet('install','uninstall','status')][string]$Step = 'status')
$ErrorActionPreference = 'Continue'
$root = Split-Path $PSScriptRoot -Parent
$setup = "$root\dist\aruaru-ime-windows-x86_64-setup.exe"
$log = "$root\dist\installer-e2e.log"
function L($m) { ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m) | Out-File $log -Append -Encoding utf8 }
function State($tag) {
    $app = "${env:ProgramFiles}\aruaru-IME"
    $items = @(
        "tag=$tag",
        "app_dir=$(Test-Path $app)",
        "dll=$(Test-Path "$app\aruaru_ime_tsf.dll")",
        "daemon_exe=$(Test-Path "$app\aruaru-ime-daemon.exe")",
        "uninstaller=$(@(Get-ChildItem "$app\unins*.exe" -ErrorAction SilentlyContinue).Count -gt 0)",
        "clsid=$(Test-Path 'HKLM:\SOFTWARE\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}')",
        "clsid_dll=$((Get-ItemProperty 'HKLM:\SOFTWARE\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}\InprocServer32' -ErrorAction SilentlyContinue).'(default)')",
        "run_key=$([bool](Get-ItemProperty 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'aruaru-IME' -ErrorAction SilentlyContinue))",
        "dict_local=$(Test-Path "$env:LOCALAPPDATA\aruaru\dict.bin")"
    )
    L ($items -join ' ')
}
switch ($Step) {
    'install' {
        State 'before-install'
        $p = Start-Process $setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/TASKS=""' -Wait -PassThru
        L "install exit=$($p.ExitCode)"
        Start-Sleep 5
        State 'after-install'
        try { L ("health: " + (Invoke-WebRequest http://127.0.0.1:4660/health -UseBasicParsing -TimeoutSec 5).Content) } catch { L 'health: unreachable' }
    }
    'uninstall' {
        State 'before-uninstall'
        $un = Get-ChildItem "${env:ProgramFiles}\aruaru-IME\unins*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($un) {
            $p = Start-Process $un.FullName -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait -PassThru
            L "uninstall exit=$($p.ExitCode)"
        } else { L 'uninstaller not found' }
        Start-Sleep 3
        State 'after-uninstall'
    }
    'status' { State 'status' }
}
L 'done'
