# aruaru-IME 実機(Windows)の通し試験: TestHost に打鍵を送り、確定された文字列を検証する。
# 前提: aruaru-ime-daemon が 127.0.0.1:4660 で動作し、TSF DLL が登録(HKCU/HKLM)済みであること。
# 使い方: powershell -File tools\ime-e2e.ps1 [-Dll <path>]
param([string]$HostExe = "$PSScriptRoot\testhost\TestHost.exe")
Add-Type -AssemblyName Microsoft.VisualBasic, System.Windows.Forms

$out = Join-Path $env:TEMP 'testhost.txt'
Get-Process TestHost -ErrorAction SilentlyContinue | Stop-Process -Force
Remove-Item $out -ErrorAction SilentlyContinue
Start-Process $HostExe
Start-Sleep -Seconds 3

function Get-Text {
    $raw = [IO.File]::ReadAllText($out, [Text.Encoding]::UTF8)
    $i = $raw.IndexOf("`n")
    return $raw.Substring($i + 1)
}

$script:pass = 0
$script:fail = 0
function Test-Scenario($name, $keys, $expected) {
    [Microsoft.VisualBasic.Interaction]::AppActivate('aruaru-IME TestHost')
    Start-Sleep -Milliseconds 400
    $before = Get-Text
    [System.Windows.Forms.SendKeys]::SendWait($keys)
    Start-Sleep -Milliseconds 900
    $after = Get-Text
    $got = if ($after.StartsWith($before)) { $after.Substring($before.Length) } else { "<?>" + $after }
    # 区切りに改行を入れて次のシナリオと混ざらないようにする
    [System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
    Start-Sleep -Milliseconds 300
    if ($got -eq $expected) { $script:pass++; "PASS  $name : $got" }
    else { $script:fail++; "FAIL  $name : got '$got' expected '$expected'" }
}

Test-Scenario 'romaji->hiragana'        'sakura{ENTER}'                      'さくら'
Test-Scenario 'convert first candidate' 'sakura {ENTER}'               '桜'
Test-Scenario 'sentence (Viterbi)'      'kyouhaiitenkidesu {ENTER}'    '今日はいい天気です'
Test-Scenario 'Shift-start stays alpha' 'Hello{ENTER}'                       'Hello'
Test-Scenario 'F7 katakana'             'kamera{F7}{ENTER}'                  'カメラ'
Test-Scenario 'F10 half-width alpha'    'wifi{F10}{ENTER}'                   'wifi'
Test-Scenario 'F6 hiragana'             'abc{F6}{ENTER}'                     (([string][char]0x3042) + 'bc')
Test-Scenario 'backspace'               'kaki{BS}{BS}u{ENTER}'               'かう'
Test-Scenario 'escape cancels'          'sakura{ESC}x'                       'x'
Test-Scenario 'plain digits pass'       '2026'                               '2026'

"---- pass=$script:pass fail=$script:fail"
Get-Process TestHost -ErrorAction SilentlyContinue | Stop-Process -Force
if ($script:fail -gt 0) { exit 1 }
