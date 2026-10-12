# Re-register the dev build of the TSF DLL (needs admin). Used after installer tests that unregistered the IME.
$root = Split-Path $PSScriptRoot -Parent
$dll = "$root\target\release\aruaru_ime_tsf.dll"
$log = "$root\dist\reregister-dev.log"
$p = Start-Process "$env:SystemRoot\System32\regsvr32.exe" -ArgumentList '/s', "`"$dll`"" -Wait -PassThru
"regsvr32 exit=$($p.ExitCode)" | Out-File $log -Encoding utf8
Test-Path 'HKLM:\SOFTWARE\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}' | Out-File $log -Append -Encoding utf8
