# (admin) Register a one-time logon task that runs the installer test step after the next reboot, then removes itself.
$root = Split-Path $PSScriptRoot -Parent
$script = "$root\tools\installer-e2e.ps1"
$cmd = "-NoProfile -ExecutionPolicy Bypass -Command `"Start-Sleep 60; & '$script' -Step install; Start-Sleep 5; & '$script' -Step status; schtasks /delete /tn aruaru-ime-e2e /f`""
$action = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument $cmd
$trigger = New-ScheduledTaskTrigger -AtLogOn
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -RunLevel Highest
Register-ScheduledTask -TaskName 'aruaru-ime-e2e' -Action $action -Trigger $trigger -Principal $principal -Force | Out-Null
"registered" | Out-File "$root\dist\schedule.log" -Encoding utf8
