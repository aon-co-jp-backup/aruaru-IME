# aruaru-IME 開発メモ(CLAUDE.md)/ Development notes

作業ドライブは `F:\aruaru-IME`(`F:\runo` 配下ではなく兄弟ディレクトリ)。共通ルールは
[`open-raid-z/CLAUDE.md`](https://github.com/aon-co-jp/open-raid-z) を参照(ここへは複製しない)。
*Working dir is `F:\aruaru-IME` (a sibling of `F:\runo`). Shared rules live in open-raid-z/CLAUDE.md.*

## 方針 / Policy

- **漢字に変換することが前提**。日本語入力の主経路は辞書+連接コストの Viterbi(約 2 ミリ秒)。打つそばから最有力の変換結果を出す(ライブ変換、既定オン)。
  *Kanji conversion is the premise: dictionary + Viterbi (~2 ms), live conversion by default.*
- LLM(`aruaru-llm`/`open-cuda` の Qwen)は、この PC では 0.5B で約 50 秒/回(CPU)かつ精度不足のため、**キー入力の経路に置かない**。OCR・検索・補助に限る。
- LLM・OCR・Web に繋がらなくても、IME は規則と辞書だけで動き続ける(失敗はエラーにせず空/None)。
- **変換の状態管理(Session)は常駐エンジン側**。DLL は「キーを送って結果を表示するだけ」の薄い層で、エンジンを入れ替えてもアプリの再起動は要らない。エンジンに繋がらないときは DLL 内の簡易版(辞書なし)に切り替わり、入力は止まらない。
- 「通常の文字が打てなくなる」事態を作らない: 反映に失敗したら自動でオフ、英数キーは必ずオフ、Caps Lock 中は食わない。
- 画面キャプチャは一時フォルダに60秒/30枚まで。パスワード欄では撮らない。送信先は既定でローカルの `aruaru-llm`。
- 秘密・個人設定(open-easy-web の固定アカウント等)は勝手に決めず、インストーラーで利用者に入力してもらう。GitHub 等のアカウントは作らない(利用者自身が用意するトークンを使う設計)。
- ダウンロードする辞書データは利用者の許可を得てから(Mozc OSS: BSD-3-Clause、JMdict: CC BY-SA 4.0 — 許可済み)。
- 汎用化できるロジックは RPoem へ切り出す方針(未実施)。

## 重要な実測と教訓 / Lessons learned

- **学習履歴・強制表記の学習が、不具合のあいだに「汚れる」**。「に→入力」「たんご→タンゴ」「ai は常にカタカナ」のような履歴が、先頭の候補を乗っ取った。対策: 完全一致のみ先頭・予測は末尾、`plausible_pair`/`sanitize` で不自然な履歴を捨てる、強制表記は辞書に載っていない英字だけ学習。
- **カタカナ判定を「直前60文字の比率」でやると、カタカナ書きの多い文章で常にカタカナモードになり、漢字に変換されなくなる**。直前の語がカタカナのときだけにした。
- `nn` + 母音は二通りに読める(こんにちは / サザンアイズ)。両方を変換し、登録語・辞書に読み全体が載っている読みを優先(`Conversion.exact_dict`)。
- 常駐エンジンを IME が自動起動すると**作業フォルダが違う**ので、辞書は作業フォルダに依存しない場所(exe の隣・データ領域)から探す。
- HKCU の COM 登録の上書きは、システムの起動経路では HKLM が優先されることがあり**当てにならない**。使用中のファイルは名前を変えて退避できるので、`tools/deploy-dev.ps1` で同じ場所に最新版を置く。
- 検索(aruaru-search)は、短時間に続けると Bing/Brave に拒否されて約14〜20分休む。検索語の間に3秒空け、使えないときは「未了」にして20分ごとに再試行。

## 注意(環境固有)
- **2026-10-13 に足した CLI コマンド / CLI commands added**: `aruaru-ime geo-install`(世界の郵便番号44か国+現在地の国)、`aruaru-ime geo <郵便番号> [言語]`(候補の確認)、`aruaru-ime data-update`(毎朝の更新をいま1回)、`aruaru-ime people-pinyin <people.tsv>`(人名辞書の拼音付け)。
  *`geo-install` (44 countries + your country), `geo <zip> [lang]` (inspect candidates), `data-update` (run the morning refresh once), `people-pinyin <people.tsv>` (add pinyin to the people table).*
- **データ領域を分けた安全な確認 / Safe checks with a separate data dir**: `ARUARU_DATA_DIR`(データ領域)と `ARUARU_IME_PORT`(ポート。本物は 4660)を別にして常駐エンジンを動かし、`POST /key` で打鍵する。**本物の IME のデーモン(4660)を止めない**: 止めるのは自分が起動した別ポートのプロセスだけ。
  *Run the daemon with a separate `ARUARU_DATA_DIR` and `ARUARU_IME_PORT` (the real one is 4660) and drive it with `POST /key`. Never stop the real IME daemon (4660): only stop the process you started on the other port.*
- **ダウンロードの原則 / Download rule**: 利用者が選んだデータだけを取得する。自動で入るのは、国名・主要都市名と、**すでに取得済み**のデータの更新だけ(`dataupdate.rs`)。新しいデータを足すときもこの原則を守る。
  *Download only what the user chose. The only automatic additions are country/city names and refreshes of data already downloaded (`dataupdate.rs`); keep to this rule for any new data.*
- **データ生成の順番 / Generating the people table**: `tools/build-people.ps1` → `tools/build-people.ps1 -Japan` → `aruaru-ime people-pinyin crates/aruaru-ime-core/pack_tables/people.tsv`。`.ps1` は BOM 付き UTF-8。生成後はパックの版(`PACKS_REV`、いま r6)を上げて、既存の利用者にも反映する。
- Linux の確認は WSL の Ubuntu(`CARGO_TARGET_DIR=$HOME/aime-target cargo test -p aruaru-ime-core`)、macOS は zig を C コンパイラにした `cargo check --target aarch64-apple-darwin` まで。

- `cargo` は PATH に無い。`%USERPROFILE%\.cargo\bin` を前置きする。開発ビルドは `CARGO_TARGET_DIR=F:\aruaru-IME\target-dev`。
- 開発機では `tools\deploy-dev.ps1 [-Dll]` で、ビルドして `target\release`(登録済みの場所)へ入れ替える。DLL は起動中のアプリが握っているので、**DLL を更新したらアプリを一度起動し直す**。
- Bash ツールのヒアドキュメントは、内容によって `unexpected EOF` で壊れることがある。長い編集は Write ツールでスクリプトを書いて実行する。
- PowerShell 5.1 の `Get-Content`/`Set-Content` は既定が ANSI で日本語を壊す。Python で UTF-8 のまま編集する。日本語を含む `.ps1` は BOM 付き UTF-8。`Remove-Item` は `F:\aruaru-IME` 配下でブロックされることがある(`reg delete` や `[IO.File]::Delete` を使う)。
- 実機の打鍵試験(`tools/ime-e2e.ps1`)は `SendKeys` を使うため、**人が同じ PC を操作していない時だけ**走らせる(他のウィンドウへ誤入力する)。エンジンの API(`POST /key`)を直接叩く確認は、いつでもできる。
- ユーザーのメモ帳には既存の未保存文書が開いていることがある。試験は専用の `tools/testhost/TestHost.exe` で行う。

## HANDOFF

- **2026-10-13 以降は、aruaru-IME のこのリポジトリの中だけで開発を進める**(ユーザー指示)。別の PC・環境へ移るときは、`PORTING.md` の「引っ越し手順 / Moving to another machine」を先に読む。
  *From 2026-10-13 development continues inside this repository only (user instruction). Read "Moving to another machine" in PORTING.md first when you move to another machine.*
- **2026-10-13 到達点 / State**: 郵便番号 → 国名つき住所(世界44か国。`-` 有無どちらも。英国・カナダ・アイルランドは地区まで。IPの国 → 言語の国 → その他、最大8件)、人名・有名人・重要人物名の辞書(約3300人、既定搭載、Wikidata CC0)、
  辞書の長音の別読み(nyuuton)、パックの一括保存で初回起動を 246秒 → 8秒、辞書メニューを3区分に整理、**毎朝の自動データ更新**(取得済みの郵便番号を条件付き取得で更新+国名・都市名を Wikidata から。朝6〜8時台、端末ごとにずらす。設定 `data_update` で停止)。
  コアのテスト約180件が通る。Linux は WSL でビルド・テスト成功、macOS は `cargo check`(zig cc)まで。
  *World postal codes → "country + address" (44 countries; with/without `-`; UK/CA/IE to the area; IP country → language country → others, max 8), a built-in people-name dictionary (~3,300, Wikidata CC0), long-vowel alternate readings, 246 s → 8 s first start, a regrouped dictionary menu, and an **automatic morning data refresh** (conditional refresh of already-downloaded postal data + country/city names from Wikidata; 06:00–08:00 local with a per-machine offset; stop with the `data_update` setting). ~180 core tests pass. Linux builds and tests in WSL; macOS is checked with `cargo check` (zig cc) only.*
- **ユーザーの方針(2026-10-13) / User policies**: ダウンロードは利用者が選んだものだけ(自動で追加しない)。例外は、国名・主要都市名(毎朝)と、すでに取得済みのデータの更新。GitHub は非公開、アカウントは作らない(利用者のトークン)。画面・入力・個人データを外へ出さない(Web のサジェストだけは設定でオフ可)。
- **未了(2026-10-13) / Open**: `PORTING.md` 「未了」の一覧を参照(右クリックメニューと2列候補窓の実機の目視確認、インストーラーの実機の通し試験、多言語の切替入力、台湾・ギリシャ・オランダの郵便番号、macOS/Android の実機確認ほか)。

- **2026-10-10〜11 新設・開発1〜2日目**: 中核エンジン(コア単体テスト 127 件+TSF 1 件)、辞書変換(Mozc OSS、1225528 エントリ)、ライブ変換、予測変換、誤モード対策、利用者辞書+ファイル取り込み、
  用語辞書(希望用語)、プログラミング語彙(自動学習)、JMdict の英語訳(自動取得)・日英の相互翻訳、2列の候補窓、バックアップ/復元(Google ドライブ)、自動アップデート、
  常駐エンジン(セッション API)+薄い TSF DLL、通知領域の右クリックメニュー、Inno Setup のインストーラーまで実装。**実機のメモ帳・VS Code・Claude アプリでの入力と、`G:\マイドライブ` へのバックアップを確認**。
- **未了(優先順)**: (1) インストーラーの実機の通し試験(インストール/アンインストール)、(2) 右クリックメニュー・2列の候補窓の実機の目視確認、(3) 用語辞書の検索→辞書化の実通し試験(aruaru-search の休止明け。VPS 経由は SSH トンネル `-L 4601:127.0.0.1:4600 conoha`)、
  (4) 多言語(ロシア語・アラビア語…)を IME 本体で切り替えて入力する機能(`translit.rs` に規則はある)・世界130言語・中国語(ピンイン辞書のダウンロード許可が必要)、(5) タスクバーの「日本」表示の右クリック(ITfLangBarItemButton)、
  (6) GitHub への選択式バックアップ(公開/非公開。トークンは利用者が用意)、(7) パスワード欄の入力スコープ判定、(8) 32bit DLL、(9) macOS / Linux / Android、(10) aruaru-search の同梱導入(`components.rs`)、(11) Qwen の高速化と再ランク。
- 開発用に HKCU の COM 登録を上書きした名残は削除済み。登録は HKLM(`target\release\aruaru_ime_tsf.dll`)。退避した古いファイルは `*.old-*`(`.gitignore` 済み)。
