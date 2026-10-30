; aruaru-IME Windows installer (Inno Setup 6)
; Build: powershell -File tools\build-installer.ps1
#define MyAppName "aruaru-IME"
#ifndef MyAppVersion
  #define MyAppVersion "0.2.17"
#endif
#define BuildDir "..\target-dev\release"

[Setup]
AppId={{6A1F2D7E-4C5B-4E8A-9B31-AA0C1E5D7F42}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=aon-co-jp
AppPublisherURL=https://github.com/aon-co-jp/aruaru-IME
DefaultDirName={autopf}\aruaru-IME
DefaultGroupName=aruaru-IME
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
OutputDir=..\dist
OutputBaseFilename=aruaru-ime-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
LicenseFile=LICENSES.txt
UninstallDisplayIcon={app}\aruaru-ime-daemon.exe
CloseApplications=no
RestartApplications=no

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "components"; Description: "必要な部品(aruaru-llm・Qwen・英語辞書)を自動で導入する / Install required components automatically"; GroupDescription: "追加の部品 / Components"

[Files]
Source: "{#BuildDir}\aruaru_ime_tsf.dll"; DestDir: "{app}"; Flags: ignoreversion restartreplace uninsrestartdelete
Source: "{#BuildDir}\aruaru-ime-daemon.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BuildDir}\aruaru-ime.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\data\dict.bin"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\data\mozc-src\LICENSE"; DestDir: "{app}"; DestName: "Mozc-LICENSE.txt"; Flags: ignoreversion

[Registry]
; ログイン時に常駐エンジン(通知領域のアイコン)を起動する
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "aruaru-IME"; ValueData: """{app}\aruaru-ime-daemon.exe"""; Flags: uninsdeletevalue

[Run]
; 1) IME を登録する(管理者権限)
Filename: "{sys}\regsvr32.exe"; Parameters: "/s ""{app}\aruaru_ime_tsf.dll"""; StatusMsg: "IME を登録しています… / Registering the IME…"; Flags: runhidden waituntilterminated
; 2) いまのユーザーの日本語の入力方式の一覧に追加する(元のユーザー権限で)
Filename: "powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""$l=Get-WinUserLanguageList; $t='0411:{{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}{{C1D3E5F7-2A4B-4C6D-8E90-1F2A3B4C5D6E}'; $j=$l | Where-Object LanguageTag -eq 'ja'; if(-not $j){{$l.Add('ja'); $j=$l | Where-Object LanguageTag -eq 'ja'}}; if($j.InputMethodTips -notcontains $t){{$j.InputMethodTips.Add($t)}}; Set-WinUserLanguageList $l -Force"""; StatusMsg: "入力方式の一覧に追加しています… / Adding to the input method list…"; Flags: runhidden waituntilterminated runasoriginaluser
; 2b) 辞書(dict.bin)を、ユーザーのデータ領域にも置く(自動アップデートで新しい版のフォルダに切り替わっても、辞書を見つけられるように)
Filename: "{sys}\cmd.exe"; Parameters: "/c mkdir ""%LOCALAPPDATA%\aruaru"" 2>nul & copy /y ""{app}\dict.bin"" ""%LOCALAPPDATA%\aruaru\dict.bin"""; Flags: runhidden waituntilterminated runasoriginaluser
; 3) 常駐エンジンを起動する(元のユーザー権限で)
Filename: "{app}\aruaru-ime-daemon.exe"; WorkingDir: "{app}"; Flags: nowait runasoriginaluser skipifsilent; StatusMsg: "エンジンを起動しています… / Starting the engine…"
; 4) 必要な部品を確認・導入する(aruaru-llm・open-easy-web・Qwen。失敗しても IME 自体は動く)
Filename: "{app}\aruaru-ime.exe"; Parameters: "ensure-components"; StatusMsg: "必要な部品を確認・導入しています(数分かかることがあります)… / Checking and installing components…"; Flags: runhidden waituntilterminated runasoriginaluser; Tasks: components

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /T /IM aruaru-ime-daemon.exe"; Flags: runhidden skipifdoesntexist; RunOnceId: "KillDaemon"
Filename: "{sys}\regsvr32.exe"; Parameters: "/u /s ""{app}\aruaru_ime_tsf.dll"""; Flags: runhidden; RunOnceId: "UnregIme"

[Code]
var
  AccountPage: TInputQueryWizardPage;

procedure InitializeWizard;
begin
  AccountPage := CreateInputQueryPage(wpSelectTasks,
    'open-easy-web のアカウント設定 / open-easy-web account',
    '任意です。空のままなら open-easy-web は起動しません。 / Optional. Leave empty to skip.',
    'open-easy-web を使う場合の、ご自身のアカウント用メールアドレスを入力してください(勝手に決めたり、送信したりはしません)。' + #13#10 +
    'Enter the e-mail address for your own open-easy-web account (it is only stored on this PC).');
  AccountPage.Add('メールアドレス / E-mail:', False);
end;

// 上書きインストールのとき、動いている常駐エンジンが exe を握っていると中断する(無人時は exit=5)ため、先に止める
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /T /IM aruaru-ime-daemon.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Sleep(800);
  Result := '';
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Dir, Email: String;
begin
  if CurStep = ssPostInstall then
  begin
    Email := Trim(AccountPage.Values[0]);
    if Email <> '' then
    begin
      Dir := ExpandConstant('{localappdata}\aruaru');
      ForceDirectories(Dir);
      SaveStringToFile(Dir + '\open-easy-web.env', 'OPEN_EASYWEB_FIXED_ACCOUNT_EMAIL=' + Email + #13#10, False);
    end;
  end;
end;
