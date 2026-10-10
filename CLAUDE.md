# aruaru-IME 開発メモ(CLAUDE.md)/ Development notes

作業ドライブは `F:\aruaru-IME`(`F:\runo` 配下ではなく兄弟ディレクトリ)。共通ルールは
[`open-raid-z/CLAUDE.md`](https://github.com/aon-co-jp/open-raid-z) を参照(ここへは複製しない)。

## 方針 / Policy

- 日本語入力の主経路は**辞書+連接コストのViterbi**(ミリ秒)。LLM(`aruaru-llm`/`open-cuda` の Qwen)は、現状の速度(0.5Bで約50秒)では
  キー入力ごとの変換に使えないため、同期経路に置かない。LLM が使えるのは OCR(画面の文字)・検索・非同期の再ランクまで。
  *The main path is dictionary + Viterbi (ms). The Qwen path is ~50 s per call on this PC, so it is kept off the keystroke path.*
- LLM・OCR・Web に繋がらなくても、IME は規則と辞書だけで動き続ける(失敗はエラーにせず空/None)。
- 「通常の文字が打てなくなる」事態を作らない: 反映に失敗したら自動でオフ、英数キーは必ずオフ、Caps Lock 中は食わない。
- 画面キャプチャは一時フォルダに60秒/30枚まで。パスワード欄では撮らない。送信先は既定でローカルの `aruaru-llm`。
- 秘密・個人設定(open-easy-web の固定アカウント等)は勝手に決めず、インストーラーで利用者に入力してもらう。
- 汎用化できるロジックは RPoem へ切り出す方針(未実施)。

## 注意(環境固有)

- `cargo` は PATH に無い。`%USERPROFILE%\.cargo\bin` を前置きする。
- TSF DLL はアプリに読み込まれるとロックされる。開発ビルドは `CARGO_TARGET_DIR=F:\aruaru-IME\target-dev` に出し、
  管理者権限なしで試すには `HKCU:\Software\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}\InprocServer32` を開発 DLL に向ける
  (**試験後は削除すること。** 本番は HKLM を regsvr32 で登録)。
- Bash ツールのヒアドキュメントは、内容によって `unexpected EOF` で壊れることがある。長い編集は Write/Edit か、スクリプトファイル経由で行う。
- PowerShell 5.1 の `Get-Content`/`Set-Content` は既定が ANSI で日本語を壊す。`-Encoding UTF8` を付けるか Python で編集する。
  日本語を含む `.ps1` は BOM 付き UTF-8 で保存する。
- 実機の打鍵試験(`tools/ime-e2e.ps1`)は `SendKeys` を使うため、**人が同じ PC を操作していない時だけ**走らせる(他のウィンドウへ誤入力するため)。
- ユーザーのメモ帳には既存の未保存文書が開いていることがある。試験は専用の `tools/testhost/TestHost.exe` で行う。

## HANDOFF

- **2026-10-10 新設・初日**: 中核エンジン(54テスト)、辞書変換(Mozc OSS を `data/dict.bin` にコンパイル、1225528 エントリ)、予測変換、
  誤モード対策、利用者辞書+ファイル取り込み、設定(配色)、コンポーネント管理、常駐エンジン、Windows TSF DLL まで実装。
  実機(`TestHost`)で `sakura`→`さくら`→Enter 確定、F6/F7/F10、Shift 始まり、BS を確認(6/10 PASS。残りは打鍵混線による失敗で、IME の不具合とは断定していない)。
  **未了**: (1) 実機の通し試験の再実施(無人環境で)、(2) インストーラー(Inno Setup: IME 登録・`ensure-components`・Qwen 導入・open-easy-web のアカウント入力ページ)、
  (3) macOS(InputMethodKit)/Linux(IBus・fcitx5)/Android の IME 本体、(4) パスワード欄の入力スコープ判定、(5) 32bit DLL、
  (6) 利用者辞書の登録 UI(いまは CLI/HTTP API のみ)、(7) Qwen の高速化(open-cpu/open-cuda 側)と再ランク、(8) GitHub リリース。
  ユーザーのバグ報告「かんじにヘンカンするぜんていになってません」(=漢字に変換する前提になっていない)を受け、**ライブ変換**(打つそばから漢字かな交じりを未確定表示、既定オン、設定 `live_conversion`)を実装(単体テスト済み、実機での再確認は未了)。
  それ以外の「デフォルトで、」「このIMEは、」で途切れたメッセージの続きは未確認。確認すること。
  開発用に HKCU の COM 登録が `target-dev` の DLL を指している(**試験後に削除**: `Remove-Item HKCU:\Software\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11} -Recurse`)。
