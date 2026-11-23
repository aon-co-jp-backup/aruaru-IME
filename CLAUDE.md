# aruaru-IME 開発メモ(CLAUDE.md)/ Development notes

作業ドライブは `F:\aruaru-IME`(`F:\runo` 配下ではなく兄弟ディレクトリ)。共通ルールは
[`open-raid-z/CLAUDE.md`](https://github.com/aon-co-jp/open-raid-z) を参照(ここへは複製しない)。
*Working dir is `F:\aruaru-IME` (a sibling of `F:\runo`). Shared rules live in open-raid-z/CLAUDE.md.*

## やさしい説明 / Plain words

**日本語**: このファイルは、開発する人(と Claude)のためのメモです。むずかしい言葉は README の「はじめに」に説明があります。決まりごとは4つだけ覚えれば大丈夫です。(1)文字が打てなくなる事故を起こさない。(2)ネットや AI につながらなくても動かす。(3)パスワードなどの秘密は画面やコミットに出さない。(4)「できた」と言う前に、本物のパソコンで試す。
**English**: This file is a memo for people (and Claude) who build the app. Hard words are explained in the "What is this?" part of the README. You only need to remember four rules. (1) Never make a bug that stops people from typing. (2) Keep working even when the internet or AI is not available. (3) Never show secrets such as passwords on screen or in commits. (4) Test on a real computer before saying "it is done".

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
- **2026-10-16 v0.2.15**: 入力言語の切り替えを IME 本体に組み込んだ(**Ctrl+Shift+L**、`Key::NextLang`。TSF・IBus・Fcitx5・オンライン画面・コア)。10/16 の 109 キー記号(MS-IME 準拠)・sonic-rs 化と合わせて公開。**未了**: Windows 実機での言語名表示の目視、通知領域メニューの目視確認、macOS 対応。
  *English*: v0.2.15 adds in-IME language switching (**Ctrl+Shift+L**) and ships the 10/16 changes (MS-IME style 109-key symbols, sonic-rs). Not yet visually verified on a real Windows machine.
- **2026-10-14 お引越し機能**: `aruaru-ime move export|import`(`backup.rs` の `create_move`/`write_move`、CLI)。利用者の指示「お引越し出来るようにして」。次のリリース(v0.2.14)に含める。詳細は `PORTING.md`「データの場所」。
  *English*: added `aruaru-ime move export|import` for moving user data to another PC (next release).
- **2026-10-16 109キーの記号をMS-IME準拠に戻した(ユーザー指摘「通常の日本語109キーボードとして打てない」)**: ひらがなモードで `,`→、 `.`→。 `/`→・ `[ ]`→「 」 `-`→ー `!`→！ `?`→？ `( )`→（ ）。直前が英数字(1,000 / 3.14 / well-known)は半角のまま。`@ ^ \` のみ半角(エンジンは食わない)。下の v0.2.11 の「半角のまま」方針は本項で置き換え。実機のインストーラー版での目視確認と新バージョンの公開は未了。
- **2026-10-14 v0.2.11〜v0.2.13 公開**: Linux の Fcitx5 アドオン(`fcitx5/`、`tools/fcitx5-test.sh`)、ひらがなモードで `, . / \ [ ] @ ^` を半角のまま(ユーザー指示、以前の `,`→`、` は取りやめ)・`{` 2回で `「`・`< > ?` の候補順・JIS109(オンライン画面)、オンライン画面の Space 不具合の修正(RPoem `open-runo-crossui` の keydown を同期的に止める)、ウェブサービス名の辞書(ようつべ→youtube)とサーバーモードの共通辞書(`shared_user_dict.json`、読み取り専用)。macOS は実機が無く CI のビルド確認のみ。詳細・リリース手順の教訓は `PORTING.md` の「v0.2.11〜v0.2.13」節。
  *English*: v0.2.11–0.2.13 shipped: Fcitx5 addon, half-width `, . / \ [ ] @ ^`, `{` twice → `「`, candidate orders, JIS 109 on the online screen, the Space fix (RPoem `open-runo-crossui`), the web-service dictionary and the read-only shared dictionary for server mode. macOS is CI-build-only (no Mac). See PORTING.md for details and release lessons.

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

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**


## aruaru-llm の Connect 移行への追従(2026-11-05)
aruaru-llm の REST 全廃(ブランチ `connect-migration`)に合わせ、呼び出しを `POST /aruaru_llm.v1.Llm/<Method>`(JSON)へ移した: `Ocr`・`GenerateQwen`・`SearchRaw`・`Runtime`(ヘルス。`is_running` は `.v1.` を含む URL だけ POST)・`QwenSelect`・`QwenInstall`。WSL の実サーバーで `Runtime`/`Ocr` を確認、`cargo test -p aruaru-ime-core --lib` 195 件成功。aruaru-llm の main マージ・VPS 反映より前にこの版を配ると旧サーバーに届かない(順序に注意)。インストーラー配布物の再ビルドは未実施。`F:ruaru-ime-rel`(v0.2.15系)へは未適用。


## 語彙の拡充(2026-11-05)
- **案A(辞書の元データの追加)**: `aruaru-ime build-extra data/mozc-src <jmdict-eng.json> <jmnedict-all.json>` が、Mozc 辞書に無い「読み+表記」を JMdict(名詞・サ変名詞)と JMnedict(姓・名・人名・地名・駅名・組織名・作品名など)から `data/mozc-src/extra00.txt`(Mozc 形式)に作る。`build-dict` は `extra*.txt` を自動で読み込む。実測: 追加 532,127 件(Mozc 約 122 万 → 約 176 万、`dict.bin` 約 64MB → 約 83MB)。コストは Mozc の平均より少し大きく(=優先度は低く)付けるので、既存の変換結果は崩れない。元データは jmdict-simplified の JSON(CC BY-SA 4.0、`installer/LICENSES.txt` に JMnedict も明記)。
- **案C(毎朝の自動追加)**: `dataupdate` が Wikidata の「かな表記(P1814)」つきの項目から、作品名(アニメ・マンガ・ゲーム・映画・TV)・人名(日本語記事 3 件以上)・企業団体名・地名駅名(駅・山・名所)を利用者辞書へ出所つきで入れる(`自動更新:作品名` ほか。出所ごとに削除可)。読みが確かなものだけ。実測 約 3.2 万件。結果が少なすぎるときは辞書を消さない。
- 辞書の作り直しは `build-extra` → `build-dict`。`data/` は git 管理外。インストーラー・VPS の `dict.bin` への反映は別途必要。

## リリース時の決まり(2026-11-06・ユーザー指示)
リリース(タグ)を切るたびに **Windows の .exe インストーラーも作る**: `powershell -File tools/build-installer.ps1`(版は Cargo.toml から自動)→ `dist/` の exe と `.sha256` を `gh release upload` で添付 → VPS の `/var/lib/aruaru-ime/downloads/`(旧版は `.bak-日付` に退避、`VERSION` も揃える)へ。CI はまだ exe を作らない。

## REST 全廃・Connect 移行の状況 (2026-11-06)

方針の正本は RUNO の `API-POLICY.md`。常駐エンジン(`aruaru-ime-daemon`、127.0.0.1:4660)の局所API(14本)と aruaru-llm 呼び出し(1本)の状況:

- **移した**: エンジン側 `POST /aruaru_ime.v1.Ime/<Method>`(Key / Templates / Session / VocabLearn / VocabRemove / VocabList / DictAdd / DictRemove / DictList / Focus / Convert / Stats / Version の13本)。POST 以外・未知のメソッドは404。呼び出し元のうちこのリポジトリ内のもの(TSF DLL・ibus・fcitx5・macOS・`tools/e2e-smoke.sh`)を同時に新経路へ。aruaru-llm の `GET /v1/knowledge/search`(1本)は、現行Connect形 `POST /aruaru_llm.v1.Llm/KnowledgeSearch`(本文 `{}`)へ。
- **期限つき互換で残した(旧経路 /key /session /templates /vocab/* /dict/* /focus /convert /stats /version)**: すでに配布済みのクライアント(インストール済みのTSF DLL、easy-web.tokyo のオンライン画面、RPoem `open-runo-crossui` が生成するページ、Android の upstream `easy-web.tokyo/ime/api`)が使うため。**期限 2026-12-06**。期限後に `crates/aruaru-ime-daemon/src/main.rs` の旧 match 腕と、Android(`crates/aruaru-ime-android`)の経路名を削除する。
- **例外**: `GET /health`(ヘルスチェック)、`GET /ui`(画面のHTML)。
- **未了/注意**: Android は upstream(easy-web.tokyo 側)の経路が変わるまで旧経路のまま。サーバー側(open-easy-web の /ime/api)の移行は別担当。受信JSONは既存どおり `sonic_rs`(上限なしのローカル専用ループバック)。

## 落ちたら自動で起動し直す(自動復帰)方針と状況 / Auto-restart policy and status (2026-11-07)

### 日本語
- **方針**: 常駐するサービス・アプリは、`kill`(SIGTERM)・強制終了(SIGKILL)・異常終了のどれで止まっても、数秒以内に自動で起動し直す。`systemctl stop` のような**意図した停止**だけは止まったままにする。REST 全廃と同時に、全リポジトリで 調査 → 確認 → TEST → DEBUG → 開発 → TEST → DEBUG を行う。
- **VPS(実施済み、2026-11-07)**: `Restart=on-failure` だったと確認した自前のサービス30個と、`postgresql`・`php-fpm`(`Restart=no`)に、`/etc/systemd/system/<unit>.d/10-autorestart.conf`(`Restart=always`、`RestartSec=3`、`StartLimitIntervalSec=0`)を追加。元のユニットファイルは変更していない。`on-failure` は SIGTERM/`kill` を正常終了とみなして再起動しないため。
- **実機TEST**: `karu-tokyo-server`・`runo-tokyo`・`rs-ops` を SIGTERM、`rs-guard` を SIGKILL で止め、6秒後に全て `active`(新しいPID)で復帰。
- **PC(Windows)の IME**: 常駐エンジン `aruaru-ime-daemon.exe` を強制終了して約3秒で `/health` が復帰することを確認(2026-11-07)。
- **未了**: 他の PC/スマホ向けアプリ(open-backup 等)の常駐プロセスの自動復帰の確認、macOS(`KeepAlive`)・Linux 版の実機確認、各リポジトリへの個別追記(担当が REST 移行する際に実施)。
- **ユニットを追加するとき**: 新しいサービスは最初から `Restart=always`、`RestartSec=3`、`StartLimitIntervalSec=0` にする。

### English
- **Policy**: resident services and apps must restart automatically within a few seconds, whether stopped by `kill` (SIGTERM), a forced kill (SIGKILL) or a crash. Only an **intentional stop** such as `systemctl stop` stays stopped. Alongside the REST abolition, every repository gets survey → verify → TEST → DEBUG → develop → TEST → DEBUG.
- **VPS (done, 2026-11-07)**: for the 30 own services found with `Restart=on-failure`, plus `postgresql` and `php-fpm` (`Restart=no`), a drop-in `/etc/systemd/system/<unit>.d/10-autorestart.conf` (`Restart=always`, `RestartSec=3`, `StartLimitIntervalSec=0`) was added. The original unit files are unchanged. Reason: `on-failure` treats SIGTERM/`kill` as a clean exit and does not restart.
- **Live TEST**: `karu-tokyo-server`, `runo-tokyo`, `rs-ops` were stopped with SIGTERM and `rs-guard` with SIGKILL; after 6 s all were `active` with new PIDs.
- **PC (Windows) IME**: force-killing the resident engine `aruaru-ime-daemon.exe` brought `/health` back in about 3 s (2026-11-07).
- **Open**: confirming automatic recovery of other PC/phone apps (open-backup etc.), real-device checks for macOS (`KeepAlive`) and Linux, and per-repository notes (done by each assignee when they migrate REST).
- **When adding a unit**: start with `Restart=always`, `RestartSec=3`, `StartLimitIntervalSec=0`.

### 追加の実機TEST(2026-11-07)/ Additional live tests
- **日本語**: Linux(WSL Ubuntu・systemd): インストーラーと同じユニット(`Restart=always`)で `aruaru-ime-daemon` を動かし、SIGTERM・SIGKILL のどちらでも約10秒以内に別PIDで復帰、`systemctl stop` では止まったままを確認。Android 実機(moto g53y・Android 14): IME のプロセスを強制終了すると OS が再生成(PID 24034→24172)。端末の既定キーボードは試験後に Gboard へ戻した。端末に入っている版は 0.2.21 でリリースは 0.2.23(未更新)。macOS の `KeepAlive` は実機が無く、設定の静的確認のみ。
- **REST 移行の副作用で見つけた不具合を修正**: VPS の `maid-news-archive` が旧 `GET /v1/internal/news/export`(移行で削除済み)を呼んで404で失敗していたため、`open-english` の `scripts/archive-articles-to-github.sh` を `POST /openenglish.v1.Internal/NewsExport` に直して反映(成功を確認)。`daily-news-collect` は `scripts/daily-news-collect.sh` の実行権限が無く失敗していたので VPS で `chmod +x`(git 上のモードは 644 のままで、`aruaru-llm` 側で直す必要あり)。
- **English**: Linux (WSL Ubuntu, systemd): the installer's unit (`Restart=always`) brought `aruaru-ime-daemon` back with a new PID within ~10 s after both SIGTERM and SIGKILL, and `systemctl stop` stayed stopped. Android device (moto g53y, Android 14): force-killing the IME process made the OS recreate it (PID 24034→24172); the default keyboard was restored to Gboard afterwards. The installed build is 0.2.21 vs release 0.2.23 (not updated). macOS `KeepAlive` was only checked statically (no device).
- **Bugs found as side effects of the REST migration, fixed**: `maid-news-archive` on the VPS called the removed `GET /v1/internal/news/export` (404); `open-english`'s `scripts/archive-articles-to-github.sh` now uses `POST /openenglish.v1.Internal/NewsExport` and is deployed (verified). `daily-news-collect` failed because `scripts/daily-news-collect.sh` lost its exec bit; fixed with `chmod +x` on the VPS (git still records mode 644; `aruaru-llm` needs the fix).

### 2026-11-07 追記: VPS のオンライン画面エンジンを新版へ / VPS online-engine updated
- **日本語**: VPS の `aruaru-ime-server`(`/opt/aruaru-ime/aruaru-ime-daemon`)を 0.2.23 に入れ替えた(バックアップ `/root/aruaru-ime-daemon.bak`、WSLでビルド・glibc 2.39)。新経路 `POST /aruaru_ime.v1.Ime/*` と旧経路の両方が通ることを確認。`easy-web.tokyo` の `/ime/api` 中継(open-easy-web)と RPoem `open-runo-crossui` の呼び出し先は、別担当が作業中(未コミットの変更あり)のため触っていない。両者が新経路になれば旧経路を削除できる(期限 2026-12-06)。
- **English**: The VPS `aruaru-ime-server` (`/opt/aruaru-ime/aruaru-ime-daemon`) was replaced with 0.2.23 (backup `/root/aruaru-ime-daemon.bak`, built in WSL, glibc 2.39); both the new `POST /aruaru_ime.v1.Ime/*` and the old routes were verified. The `easy-web.tokyo` `/ime/api` relay (open-easy-web) and the RPoem `open-runo-crossui` callers are being changed by another session (uncommitted changes), so they were left alone. Once both use the new routes the old ones can be removed (deadline 2026-12-06).

### 2026-11-08 追記: オンライン画面・Android upstream を Connect へ / Online screen & Android upstream moved to Connect
- **日本語**: RPoem `open-runo-crossui`(橋渡しJSが `/aruaru_ime.v1.Ime/Key|Session` を呼ぶ、`f88f4d7`)と open-easy-web の `/ime/api` 中継(Connect パスを通す。旧 `/ime/api/key|session` は Android 旧版・古いページ向けに上流の Connect へ写す期限つき互換、`bf2677b`)を移行し、`easy-web.tokyo` へ反映した。実測: 公開URLで新 Key/Session 200・旧 `/key` 200(互換)、ブラウザで `/ime` に「konnichiwa」を打つと `…Ime/Key` が全て200で「今日わ」等の候補が出た。Android(`crates/aruaru-ime-android`)は上流への呼び出しを Connect パスへ(cargo test 5件成功、実機は未確認)。v0.2.24 としてリリース。**残り**: 期限 2026-12-06 に、daemon の旧 match 腕・easy-web の旧 `/ime/api/key|session`・Android の旧経路を削除する。
- **English**: RPoem `open-runo-crossui` now calls `/aruaru_ime.v1.Ime/Key|Session` and the open-easy-web `/ime/api` relay passes the Connect paths (old `/ime/api/key|session` kept as a deadline-bound compat for old Android builds and cached pages, mapped to Connect upstream); deployed to easy-web.tokyo. Verified on the public URL (new 200, legacy 200) and in a browser (typing "konnichiwa" at `/ime` sent only `…Ime/Key` 200s). Android upstream calls use Connect paths (5 cargo tests pass; not tried on a device). Released as v0.2.24. **Remaining**: on 2026-12-06 delete the daemon's old match arms, easy-web's old `/ime/api/key|session`, and Android's old paths.

### 2026-11-09 追記: Android 実機でConnect経路を確認 / Android device check of the Connect path
- **日本語**: moto g53y(Android 14)にリリース v0.2.24 の APK(sha256 検証済み)を `adb install -r` で入れ、IME を有効化して `easy-web.tokyo/ime` の入力欄で「konnichiwa」+空白/変換を実機のタップで入力 → 「今日わ」「今1話」「こんにちわ」等の候補が出た。VPS の open-web-server ログで同時刻の `POST /ime/api/aruaru_ime.v1.Ime/Key` が連続して全て status=200(旧 `/ime/api/key` は出ていない)=**Android の上流呼び出しが Connect 経路**で動くことを確認。試験後、既定キーボードは Gboard に戻した。注意: 同ページの表示は `v=0.2.23`(VPS の変換エンジンの版。Android アプリは 0.2.24)。旧経路の削除は期限 2026-12-06 のまま。
- **English**: Installed the v0.2.24 APK (sha256 verified) on a moto g53y (Android 14) with `adb install -r`, enabled the IME, and typed "konnichiwa" + space/convert by tapping on `easy-web.tokyo/ime`: candidates such as 今日わ / こんにちわ appeared. The VPS open-web-server log shows consecutive `POST /ime/api/aruaru_ime.v1.Ime/Key` all status=200 at that time (no legacy `/ime/api/key`), confirming the Android upstream calls use the Connect path. The default keyboard was restored to Gboard. Note: the page shows `v=0.2.23` (the VPS conversion engine; the Android app is 0.2.24). Legacy-route removal stays due 2026-12-06.

### 2026-11-09 旧REST経路の削除 / Removal of the legacy routes
- **日本語**: 常駐エンジン(`aruaru-ime-daemon`)の旧経路(`/key` `/session` `/templates` `/vocab/*` `/dict/*` `/focus` `/convert` `/stats` `/version`)を削除し、`POST /aruaru_ime.v1.Ime/*`・`GET /health`・`GET /ui` だけにした(ユーザー指示で期限2026-12-06を前倒し)。TSF/macOS/fcitx5/ibus/Android(上流)は全て新経路を使うことをコード検索で確認。ローカル実測: 旧経路すべて404、新経路200、`/health`・`/ui` 200。**Release と VPS(エンジン・`/var/lib/aruaru-ime/downloads/VERSION`)の版揃えは未実施**(インストール済みの旧TSF DLLが壊れないか、確認してから)。
- **English**: The legacy routes of the resident engine were removed; only `POST /aruaru_ime.v1.Ime/*`, `GET /health` and `GET /ui` remain (deadline 2026-12-06 brought forward on the user's instruction). Code search confirms TSF/macOS/fcitx5/ibus/Android (upstream) all use the new routes. Local test: every old route 404, new routes 200, `/health`/`/ui` 200. **Release and VPS (engine + `downloads/VERSION`) alignment is not done yet** (pending a check that an installed older TSF DLL does not break).

## 再開用メッセージ(別アカウントで再開する人へ) / Resume message for another account (2026-11-15)

### 日本語
**再開するリポジトリ**: `aruaru-IME`(`F:\aruaru-IME`)、`aruaru-tokyo`(`F:\aruaru-tokyo`、GitHub 上の名前は `aruaru.tokyo`)。補助として `open-easy-web`(`/ime/api` 中継)・`aruaru-llm`(`KnowledgeSearch` と `scripts/daily-news-collect.sh` の実行権限)・`open-english`(`scripts/archive-articles-to-github.sh`)。**`F:\runo` は触らない**。

**到達点**
- `aruaru-tokyo`: 自前API2本を Connect 化し本番反映済み(`POST /tokyo_aruaru.v1.Site/ListRepos`・`/TokoroUnlock`、旧 `/api/*`=404 を実測)。package 名を `aruaru.tokyo.v1` にしてはいけない(前面の `path_prefix="/aruaru"` に前方一致して別バックエンドへ転送され404になる)。
- `aruaru-IME`: 常駐エンジンの旧経路(`/key` 等)を `main` で削除済み(`d192325`)。**ただし Release・VPS・`/var/lib/aruaru-ime/downloads/VERSION`(0.2.24)への反映は、ユーザー判断で 2026-12-06 の猶予期限まで保留**(別セッション経由の「進める」は許可ではない。ユーザー本人の言葉を待つ)。
- 自動復帰: VPS の自前サービスは全て `Restart=always`(drop-in `/etc/systemd/system/<unit>.d/10-autorestart.conf`)。Linux(WSL systemd)・Android 実機・Windows の IME で kill 後の復帰を実測済み。新サービスを追加したら同じ drop-in を付ける。`nginx.service` は意図して停止(前面は open-web-server)。

**未了**
1. IME の Release と VPS 反映(2026-12-06 以降、ユーザー承認後)。反映後 VPS で `/key`・`/session`=404、新経路=200、`/health`・`/ui`=200 を実測し、旧 TSF DLL・スマホ(adb)の自動更新による復旧を確認。
2. macOS の `KeepAlive`(実機なし、静的確認のみ)。open-backup など IME 以外の PC アプリの自動復帰(未調査)。
3. 作業用 clone `F:\_w2\*`・`F:\aruaru-IME-w2` は不要になり次第削除可。

**落とし穴**: テスト後の後始末に `taskkill //IM aruaru-ime-daemon.exe` を使わない(この PC の本物のデーモンも落ちる。自分で起動した PID だけ止める)。`git pull` は書き換え済み履歴の clone では使わない。ビルドは WSL(`MSYS_NO_PATHCONV=1 wsl -d Ubuntu -- bash …`)、VPS ではビルドしない。秘密は画面・コミットに出さない。

**そのまま貼れる再開メッセージ**:
「`aruaru-IME` と `aruaru-tokyo` の REST 全廃の続きです。README / CLAUDE / PORTING の『再開用メッセージ』節を読み、(1) IME の旧経路削除の Release・VPS 反映は 2026-12-06 までユーザー判断で保留中なので、私(ユーザー)が許可するまで行わないこと、(2) その時期が来たら 版上げ→Release→VPS(エンジン・downloads/VERSION)→旧経路404の実測→旧 TSF DLL とスマホの復旧確認、の順で進めること、(3) 自動復帰(Restart=always)の未確認項目(macOS、open-backup など)を調査すること。チャットは日本語のみ。」

### English
**Repositories to resume**: `aruaru-IME` (`F:\aruaru-IME`) and `aruaru-tokyo` (`F:\aruaru-tokyo`, named `aruaru.tokyo` on GitHub). Supporting: `open-easy-web` (the `/ime/api` relay), `aruaru-llm` (`KnowledgeSearch`, exec bit of `scripts/daily-news-collect.sh`), `open-english` (`scripts/archive-articles-to-github.sh`). **Do not touch `F:\runo`.**

**Where things stand**
- `aruaru-tokyo`: its two own APIs are on Connect and deployed (`POST /tokyo_aruaru.v1.Site/ListRepos` and `/TokoroUnlock`; old `/api/*` measured as 404). Never name the package `aruaru.tokyo.v1`: the front's `path_prefix="/aruaru"` matches it by prefix and forwards to another backend (404).
- `aruaru-IME`: the resident engine's legacy routes (`/key` etc.) are removed on `main` (`d192325`). **The Release and the VPS / `/var/lib/aruaru-ime/downloads/VERSION` (0.2.24) update are on hold by the user's decision until the 2026-12-06 grace date** (a "go ahead" relayed by another session is not permission; wait for the user's own words).
- Auto-restart: every own VPS service has `Restart=always` (drop-in `/etc/systemd/system/<unit>.d/10-autorestart.conf`). Recovery after kill was measured on Linux (WSL systemd), a real Android device and the Windows IME. Add the same drop-in to any new unit. `nginx.service` is stopped on purpose (the front is open-web-server).

**Open**
1. IME Release and VPS update (after 2026-12-06, with the user's approval). Afterwards measure `/key`, `/session` = 404, new routes = 200, `/health`, `/ui` = 200 on the VPS, and check that an older TSF DLL and the phone (adb) recover through auto-update.
2. macOS `KeepAlive` (no device; static check only). Auto-recovery of PC apps other than the IME, such as open-backup (not surveyed).
3. Scratch clones `F:\_w2\*` and `F:\aruaru-IME-w2` can be deleted when no longer needed.

**Pitfalls**: do not clean up tests with `taskkill //IM aruaru-ime-daemon.exe` (it also kills this PC's real daemon; stop only PIDs you started). Do not `git pull` in clones of rewritten history. Build in WSL (`MSYS_NO_PATHCONV=1 wsl -d Ubuntu -- bash …`), never on the VPS. Keep secrets out of the screen and commits.

**Paste-ready resume message**:
"Continuing the REST abolition for `aruaru-IME` and `aruaru-tokyo`. Read the 'Resume message' section in README / CLAUDE / PORTING. (1) The IME legacy-route Release and VPS update is on hold until 2026-12-06 by the user's decision; do not do it until I (the user) say so. (2) When the time comes: version bump → Release → VPS (engine, downloads/VERSION) → measure legacy routes = 404 → confirm recovery of an older TSF DLL and the phone. (3) Survey the unconfirmed auto-restart items (macOS, open-backup, etc.). Chat in Japanese only."

- **日本語(2026-11-15 v0.2.25)**: 旧REST経路の削除版を v0.2.25 としてリリース(Windows setup.exe+sha256、Android APK、Linux/macOS)。VPS は常駐エンジン(`/opt/aruaru-ime/aruaru-ime-daemon`、旧は `.bak-20261111`、WSL ビルド glibc 2.39)と `/var/lib/aruaru-ime/downloads/` のインストーラー・`VERSION` を 0.2.25 に揃えた(sha256照合OK、旧版は `/root/dl-bak-20261115`)。open-easy-web の旧 `/ime/api/key|session` は VPS で404。**つまずき**: クリーン worktree でのインストーラーには `data/dict.bin` に加えて `data/mozc-src/LICENSE` が要る(ISCC が参照)。CI の install-test が旧 `/key` を叩いていたため Connect パス(`/aruaru_ime.v1.Ime/Key|Session`)へ直した。Android の `ui.rs`/`lib.rs` の `/key` `/session` は WebView→JNI の内部呼び名でネットワーク経路ではない。**未確認**: 新CI全通過、Android実機の更新後の動作、旧TSF DLLがv0.2.25エンジンで動くか。
- **English (2026-11-15, v0.2.25)**: Released the build with the old REST routes removed (Windows setup.exe+sha256, Android APK, Linux/macOS). On the VPS the engine (old binary kept as `.bak-20261111`, WSL build, glibc 2.39) and the downloads installer + `VERSION` are now 0.2.25 (sha256 verified, old files in `/root/dl-bak-20261115`). The legacy `/ime/api/key|session` in open-easy-web returns 404 on the VPS. Pitfall: a clean-worktree installer build also needs `data/mozc-src/LICENSE`. CI install-test called the old `/key`; it now uses the Connect paths. Android's `/key`/`/session` are internal WebView→JNI names, not network routes. Unverified: the full new CI run, an Android device after update, whether an old TSF DLL works with the 0.2.25 engine.

## 2026-11-17 別アカウントでの再開用メモ(依存更新の完了と未確認項目) / Resume memo for another account (dependency updates done, unverified items)

### 日本語
- **再開するリポジトリ**: `aruaru-IME`(`F:\aruaru-IME`)、補助として `runo`(`F:\runo`、索引・チェックポイント)、`open-web-server`(`F:\open-web-server`、`origin/main` と差分なし)、`open-cuda`(`F:\open-cuda`、`crates/open-cuda-whisper` がある)。
- **完了(確認済み)**: 依存の更新は全て済み。`ureq` 3・`zbus` 6・`windows`/`windows-core` 0.62・`jni` 0.22(コミット 889c514)。2026-11-17 に `main` を最新へ追従(`git pull --ff-only`、`bdf626e` 時点)し、`cargo test --workspace` が全成功(core 204件・失敗0・無視3件)、`cargo check -p aruaru-ime-android` も成功。
- **未確認(実機・環境が要る。完了とは言わないこと)**: (1) 実際の `ibus-daemon` との接続、(2) `regsvr32` でのTSF DLL登録と実入力(管理者権限)、(3) Android 実機・エミュレーターでの jni 0.22 版の動作、(4) 実ネットワーク越しの通し試験(ureq 3)。
- **未特定**: 「whisper.cpp」の状態確認は、`F:\whisper.cpp` がこのPCに無く未実施。場所をユーザーに聞くか、`open-cuda-whisper` のことかを確認する。
- **警告**: `zvariant::Value`→`zbus::Variant` などの非推奨警告、`aruaru-ime-core` の未使用変数警告3件が残る(動作に影響なし)。
- **注意**: 旧経路削除のRelease・VPS反映は2026-12-06までユーザー判断で保留(上の節を参照)。`git pull` は書き換え済み履歴のcloneでは使わない。`taskkill //IM aruaru-ime-daemon.exe` は使わない。チャットは日本語のみ。
- **そのまま貼れる再開メッセージ**: 「`aruaru-IME` の続きです。README / CLAUDE / PORTING 末尾の『2026-11-17 別アカウントでの再開用メモ』を読み、(1) 未確認4項目(ibus-daemon・regsvr32・Android実機・実ネットワーク)のうち、このPCでできるものから実測して結果を記録、(2) whisper.cpp の場所を私に確認、(3) 2026-12-06まで保留のReleaseは私の指示まで行わない、の順で進めてください。チャットは日本語のみ。」

### English
- **Repositories to resume**: `aruaru-IME` (`F:\aruaru-IME`); supporting: `runo` (`F:\runo`, index and checkpoints), `open-web-server` (`F:\open-web-server`, no diff from `origin/main`), `open-cuda` (`F:\open-cuda`, contains `crates/open-cuda-whisper`).
- **Done (verified)**: all dependency updates are in: `ureq` 3, `zbus` 6, `windows`/`windows-core` 0.62, `jni` 0.22 (commit 889c514). On 2026-11-17 `main` was fast-forwarded (`bdf626e`); `cargo test --workspace` passes (core 204 tests, 0 failed, 3 ignored) and `cargo check -p aruaru-ime-android` succeeds.
- **Unverified (need a device or environment; do not call them done)**: (1) a real `ibus-daemon` connection, (2) TSF DLL registration with `regsvr32` and real typing (needs admin), (3) the jni 0.22 build on a real Android device or emulator, (4) an end-to-end test over a real network (ureq 3).
- **Unidentified**: the "whisper.cpp" status check was not done because `F:\whisper.cpp` does not exist on this PC; ask the user where it is, or whether `open-cuda-whisper` is meant.
- **Warnings**: deprecation warnings (`zvariant::Value` → `zbus::Variant`, etc.) and 3 unused-variable warnings in `aruaru-ime-core` remain (no behaviour impact).
- **Notes**: the legacy-route Release and VPS update stay on hold until 2026-12-06 by the user's decision (see the section above). Never `git pull` in clones of rewritten history. Do not use `taskkill //IM aruaru-ime-daemon.exe`. Chat in Japanese only.
- **Paste-ready resume message**: "Continuing `aruaru-IME`. Read the section '2026-11-17 別アカウントでの再開用メモ' at the end of README / CLAUDE / PORTING. (1) Of the four unverified items (ibus-daemon, regsvr32, Android device, real network), measure what this PC can do and record the results; (2) ask me where whisper.cpp is; (3) do not perform the Release held until 2026-12-06 until I say so. Chat in Japanese only."


## 2026-11-18 別アカウントでの再開用メモ / Resume memo
**日本語**: aruaru-llm呼び出しはConnect化済み、語彙拡張(JMdict/JMnedict・Wikidata)で175万件超、版揃えテスト(`crates/aruaru-ime-cli/tests/version_sync.rs`)あり。リリースごとにWindows exeを作りReleaseとVPS`downloads/VERSION`を揃える。未了=VPSエンジンの入れ替え、`webpage/index.html`の版表記、PCインストーラー再実行、メモ帳実入力。**スマホ(Android実機)の試験は、ユーザー指示で中止中**。詳細は`PORTING.md`末尾。
**English**: All aruaru-llm calls are on Connect; vocabulary expanded to 1.75M+ entries; version-sync tests exist. Build the Windows exe per release and align Release and VPS `downloads/VERSION`. Open: swap the VPS engine, version text in `webpage/index.html`, re-run the PC installer, real typing test. **Phone (Android device) testing is stopped by the user.** Details at the end of `PORTING.md`.


## 2026-11-18 追記(VPSエンジン v0.2.26 反映・紹介ページ更新・生成の確認)
- **VPSエンジンを v0.2.26 へ差し替え済み**: WSL(Ubuntu-24.04、glibc 2.39)でビルドし、`/opt/aruaru-ime/aruaru-ime-daemon` を入れ替え(旧は `.bak-20261118`)。`/health` は `ok dict=1757640 v=0.2.26 proto=1`、サービス active。`Key`(`sid`+`{"Char":"k"}`等)で候補が返ることを確認(拡張語彙の候補を含む)。
- **紹介ページ**: `webpage/index.html` のダウンロード欄を v0.2.26 の実ハッシュ・サイズへ更新(`c424819`)、VPSの `/root/easy-web.tokyo/aruaru-IME` も追従し本番表示を確認。
- **生成の正常系**: VPS(メモリ3.6GB・空き約0.9GB・4コアが高負荷)へのQwen導入は本番を巻き込むため**見送り**。手元の `F:\aruaru-llm\models\qwen2.5-0.5b-instruct`(導入済み)でローカル確認する方針で、WSLで aruaru-llm をビルド中(未確認)。
- **スマホ(Android実機)の試験は中止中**(ユーザー指示)。
- **未了**: PCインストーラーの再実行(アプリを閉じて管理者で。UACは本人)、メモ帳での実入力、生成の正常系のローカル確認、pdf-r2l-rsの他OS成果物(macOSなし)。

- **生成の正常系(手元WSLで確認済み、2026-11-18)**: aruaru-llm(`F:\aruaru-llm`、WSL release、127.0.0.1:4691)に `QwenSelect {"id":"qwen2.5-0.5b-instruct"}` → `GenerateQwen` が成功し、続きの文章が返った(engine=`qwen2-gqa-rope-rmsnorm-swiglu-greedy-decode-v0-open-cuda-llm-cpu`)。`OcrFill` も200で応答。注意: `Generate` はGPT-2経路で、Qwenを使うのは `GenerateQwen` とOCR補完系。VPSには導入していない(資源不足のため)。確認後、手元のサーバーは停止済み。

- **OcrFillの補完を手元WSLで確認(2026-11-19)**: Qwen2.5-0.5Bを選択した状態で、`prefix`「今日は天気がよいので公園へ」・`suspect`「散歩に行きま寸」・`suffix`「。」・`extra`(候補)・`conf:30` を渡すと、`散歩に行きます` を採用(`changed:true`、`margin`≈25、対数尤度 -9.2 対 原文 -34.2)。注意: `extra` や `lm_candidates` で候補を渡さないと `top` が空で変更されない(`items` の項目は `prefix`/`suspect`/`suffix`/`gap`/`extra`/`conf`)。確認後に手元サーバーは停止済み。VPSにはQwen未導入のまま。

## 2026-11-19 v0.2.26 の公開とVPSの反映 / v0.2.26 released and the VPS is in sync

### 日本語(やさしい説明つき)
- **何をしたか**: aruaru-IME の新しい版 **v0.2.26** を、みんなが入手できる場所(GitHub の Release)に公開しました。Windows・Android・Linux・macOS(2種類)の全部で、自動のビルドとテストが成功しています。
- **VPS(サーバー)**: ダウンロード用の5ファイルと `VERSION` を 0.2.26 にそろえ、元のファイルと同じ中身かを SHA-256(ファイルの指紋)で確かめました。サーバー側で動く変換エンジンも、2026-11-18 から v0.2.26 で動いています(`/health` が `v=0.2.26` を返すことを確認)。
- **前の予定との関係**: 以前の「旧経路の削除の公開は 2026-12-06 まで保留」は、2026-11-18 にユーザーが「公開する」と決めたため、**もう保留ではありません**。旧経路は v0.2.25 から消えていて、v0.2.26 はそれを引き継いでいます。
- **まだ確かめていないこと(正直に)**: (1) タスクバー右下のアイコンを右クリックしたときの「ジャンル登録」メニューの見た目、(2) メモ帳などでの実際の入力(変換)、(3) インストーラーの入れ直しの通し試験(管理者の許可=UAC が必要で、AI は承認できません)。この3つは本人が操作して確かめてください。
- **用語のミニ辞典**: Release=公開された完成品の置き場。SHA-256=ファイルが壊れていないか見分ける指紋。UAC=「このアプリを許可しますか」の確認画面。
- **決まり**: 新しい REST は作らず Connect だけを使う。パスワードやトークンは画面にも記録にも出さない。実機で確かめてから「完了」と言う。

### English (plain words, for everyone)
- **What was done**: the new version of aruaru-IME, **v0.2.26**, is now published as a GitHub Release so anyone can download it. The automatic build and tests passed on every platform: Windows, Android, Linux and macOS (two kinds).
- **The VPS (the server)**: the five download files and `VERSION` now say 0.2.26, and each file was checked against its SHA-256 (a fingerprint that shows a file is not damaged). The conversion engine that runs on the server has also been v0.2.26 since 2026-11-18 (`/health` answers `v=0.2.26`).
- **About the earlier plan**: the old note "hold the release that removes the legacy routes until 2026-12-06" **no longer applies**, because the user decided on 2026-11-18 to publish. The legacy routes were already gone in v0.2.25, and v0.2.26 keeps that.
- **What is still unchecked (honestly)**: (1) how the "register by genre" menu looks when you right-click the tray icon at the bottom right, (2) real typing and conversion in Notepad and similar apps, (3) a full re-install test of the installer (it needs administrator approval, UAC, which an AI cannot give). Please check these three yourself.
- **Mini glossary**: Release = the place where finished versions are published. SHA-256 = a fingerprint that tells whether a file is intact. UAC = the "Do you allow this app?" confirmation window.
- **Rules**: do not build new REST (use Connect only), never show or record passwords or tokens, and say "done" only after checking on a real system.

### そのまま貼れる再開メッセージ / Paste-ready resume message
「`aruaru-IME` の続きです。README / CLAUDE / PORTING 末尾の『2026-11-19 v0.2.26 の公開とVPSの反映』を読み、(1) 右クリックメニュー・メモ帳の実入力・インストーラーの入れ直しは私が確認するので結果を待つ、(2) 他に未了があれば自動で進める(確認の質問は不要)、の順で進めてください。チャットは日本語のみ。」
"Continuing `aruaru-IME`. Read the section '2026-11-19 v0.2.26 の公開とVPSの反映' at the end of README / CLAUDE / PORTING. (1) I will check the right-click menu, real typing in Notepad and the installer re-install, so wait for my results; (2) proceed automatically with anything else that is open (no confirmation questions). Chat in Japanese only."

## 2026-11-20 スマホ・タブレット版キーボードの配置変更 / Phone and tablet keyboard layout

**日本語(やさしい説明)**
- **入力モードを4つから選べます**: ひらがな・カタカナ・アルファベット・記号。左上のボタン(あ/カ/A/記)を押すと、候補欄に4つ並ぶので、使いたいものを押します。
- **「、」と「,」を右上に常に表示**: ひらがな・カタカナのときは、候補欄の右上にいつも出ています。入力中の文字があれば、それを確定してから入れます。
- **モードのボタンを左下から左上へ移動**: そのぶん、一番下の「空白 / 変換」を広くしました。
- **カタカナ**: 打った読みをそのままカタカナで出し、⏎で確定します。漢字にしたいときは「空白 / 変換」を押します。
- **記号**: 英字の3行が記号(！？「」()・ー…〜 @#$%&* など)に変わり、押すとそのまま入ります。
- **確認したこと**: 単体試験6件成功。ブラウザでスマホ幅(375px)とタブレット幅(1024px)の見た目と動き。実機スマホ(A401OP、Android)にAPKを入れて、「か、」「コカ,」「！」が正しく入ることを確認。
- **まだのこと**: タブレット実機と横向きの実機確認(試した設定アプリは縦固定)、新しい版としてのリリース(→ v0.2.27 で公開済み)。
- 場所: `crates/aruaru-ime-android/src/ui.rs`。見た目を確かめる道具は `cargo run -p aruaru-ime-android --example dump_keyboard > keyboard.html`。

**English (plain words)**
- **Pick one of four input modes**: hiragana, katakana, alphabet, symbols. Tap the top-left button (あ/カ/A/記); the four choices appear in the candidate bar.
- **「、」 and 「,」 are always at the top right** in hiragana and katakana mode. If you are typing a word, it is committed first.
- **The mode button moved from the bottom left to the top left**, so the bottom "space / convert" key is wider.
- **Katakana**: what you type is shown and committed as katakana with ⏎; press "space / convert" if you want kanji.
- **Symbols**: the three letter rows turn into symbol keys that type directly.
- **Checked**: 6 unit tests pass; look and behaviour in a browser at phone width (375 px) and tablet width (1024 px); on a real Android phone (A401OP) 「か、」「コカ,」「！」 were typed correctly.
- **Not yet**: a real tablet and real landscape check (the test app was portrait-only), and a tagged release (done: v0.2.27).

## 2026-11-20 v0.2.27 の公開 / v0.2.27 released

**日本語(やさしい説明)**
- **公開しました**: スマホ・タブレット版キーボードの新しい配置(4モード選択・右上の「、」「,」・左上のモードボタン)を、**v0.2.27** として GitHub の Release に公開しました。
- **自動の試験**: Windows・Android(スマホとタブレットのエミュレーター)・Linux・macOS(2種類)の作成と試験が全部成功しました。
- **Windows の exe**: 手元で Inno Setup により作り、版が 0.2.27 であることを確かめて Release に添付しました(SHA-256 は `dd2f84fa…6766`)。
- **VPS**: ダウンロード用の5ファイルと `VERSION` を 0.2.27 にそろえ、SHA-256 で同じ中身か確かめました(元のファイルは `/root/dl-bak-20261120`)。`easy-web.tokyo/ime/download` は v0.2.27 と表示し、配っている exe の SHA-256 は Release と同じです。サーバーの変換エンジンも同じ日に v0.2.27 へ差し替えました(下の「VPS エンジンの v0.2.27 反映」)。
- **まだのこと(正直に)**: タブレット実機と横向きの実機確認、Windows インストーラーの入れ直しの通し試験(管理者の許可=UAC が必要で、AI は承認できません)。

**English (plain words)**
- **Released**: the new phone/tablet keyboard layout (four selectable modes, 「、」 and 「,」 at the top right, the mode button at the top left) is published on GitHub as **v0.2.27**.
- **Automatic tests**: builds and tests passed on Windows, Android (phone and tablet emulators), Linux and macOS (two kinds).
- **Windows exe**: built locally with Inno Setup, checked to be version 0.2.27, and attached to the Release (SHA-256 `dd2f84fa…6766`).
- **VPS**: the five download files and `VERSION` now say 0.2.27, each checked by SHA-256 (old files in `/root/dl-bak-20261120`). `easy-web.tokyo/ime/download` shows v0.2.27 and serves the same exe as the Release. The server's conversion engine was also swapped to v0.2.27 on the same day (see "VPS engine updated to v0.2.27" below).
- **Not yet (honestly)**: a real tablet and real landscape check, and a full re-install test of the Windows installer (it needs administrator approval, UAC, which an AI cannot give).

## 2026-11-20 VPS エンジンの v0.2.27 反映 / VPS engine updated to v0.2.27

**日本語(やさしい説明)**
- **やったこと**: サーバー(VPS)で動く変換エンジン(`aruaru-ime-server.service`、`/opt/aruaru-ime/aruaru-ime-daemon`)を v0.2.27 に入れ替えました。VPS の作業用コピー `/root/repository/aruaru-IME` も最新にしました(`git pull --ff-only`、未コミットの変更なし)。
- **作り方**: VPS と同じ glibc 2.39 の WSL(`Ubuntu-24.04`)でビルドしました(普段の `Ubuntu` は glibc 2.43 で、VPS では動かない恐れがあるため使いません)。
- **安全のための手順**: 本番の前に、別のポート(4669)と使い捨てのデータフォルダーで試しに起動し、「konnnitiha」が「今日は」に変換されることを確かめてから入れ替えました。元のエンジンは `aruaru-ime-daemon.bak-20261120` に残し、起動に失敗したら自動で戻す仕組みにしました(今回は戻す必要なし)。
- **確かめたこと**: `/health` が `v=0.2.27`、サービスは動作中でエラーのログは0件。公開URL `easy-web.tokyo/ime/api/aruaru_ime.v1.Ime/Key` が全部200で正しく変換、古い `/ime/api/key` は404のまま。実機スマホ(A401OP)のキーボードからも「今日は」に変換されました。

**English (plain words)**
- **What was done**: the conversion engine on the VPS (`aruaru-ime-server.service`, `/opt/aruaru-ime/aruaru-ime-daemon`) now runs v0.2.27. The VPS working copy `/root/repository/aruaru-IME` was also brought up to date (`git pull --ff-only`; no local changes).
- **How it was built**: in the WSL distro `Ubuntu-24.04`, which has the same glibc 2.39 as the VPS (the usual `Ubuntu` has glibc 2.43, whose builds may not run on the VPS).
- **Safety steps**: before the swap, the new engine was test-started on another port (4669) with a throwaway data folder, and "konnnitiha" converted to 「今日は」. The old engine is kept as `aruaru-ime-daemon.bak-20261120`, with an automatic rollback if the start failed (not needed).
- **Checked**: `/health` says `v=0.2.27`, the service is active with zero error logs, the public `easy-web.tokyo/ime/api/aruaru_ime.v1.Ime/Key` answers 200 with correct conversions, the old `/ime/api/key` still returns 404, and the keyboard on a real phone (A401OP) converted to 「今日は」.

## 2026-11-20 低VRAM・VRAM無し端末(VPS・PC・タブレット・スマホ)でのAI/LLMのノウハウ / Low-VRAM AI know-how
日本語: AI・LLMを使う機能(推論・要約・補完・OCR等)は、[aruaru-llm/docs/LOW-VRAM-KNOWHOW.md](https://github.com/aon-co-jp/aruaru-llm/blob/main/docs/LOW-VRAM-KNOWHOW.md) に従う: 短い文脈・上限・タイムアウト、VPSで重い推論を直接回さない、メモリが足りなければ小さいモデルへ自動で下げる。広い桁(FP64以上)は本番で使わず、試験の正解にだけ使う。
English: Features that use AI/LLMs follow the file above: short context, caps and timeouts, no heavy inference directly on the VPS, auto-downgrade to a smaller model when memory is short. Wide floats (FP64 and above) are for test oracles only, not production.

## 2026-11-20 タブレット・横向きの不具合修正 / Tablet and landscape fixes

**日本語(やさしい説明)**
- **見つけた不具合(エミュレーターと実機で確認)**:
  1. **タブレットや横向きで、キーボードが出ないことがあった**。Android の標準の動き(横向きは「全画面入力」、物理キーボードがあると画面のキーボードを隠す)のせいで、表示の要求が断られていた。Gboard では出るのに aruaru-IME だけ出なかった。
  2. **スマホの横向きで、キーボードが高すぎて入力欄が隠れた**(縦向きのときの高さのまま)。
  3. **縦向きでも一番下の列が少し切れていた**(中身の合計が 300dp を超えていた)。
- **直したこと**: 全画面入力をやめ、物理キーボードがあっても画面のキーボードを出す(aruaru-IME は物理キーからの日本語変換を持たないため)。高さは「300dp まで、ただし画面の高さの 55% まで」とし、出すたびに計算し直す。キーの列は高さに合わせて伸び縮みする(低い画面では文字と余白を小さく、説明の行は隠す)。
- **確かめたこと**: タブレットのエミュレーター(2560×1600)の横・縦で表示と入力(「か、」「カ,」)。実機スマホ(A401OP)の横向きで入力欄が見え、「こんにちは」→「今日は」の変換、縦向きに戻しても正しい。ブラウザで 393×300・860×216 の見た目。単体試験6件成功。
- **場所**: `android/app/src/main/java/jp/aon/aruaruime/AruaruImeService.java`(`onEvaluateFullscreenMode`・`onEvaluateInputViewShown`・`onShowInputRequested`・`keyboardHeightPx`・`fitHeight`)、`crates/aruaru-ime-android/src/ui.rs`(CSS)。
- **メモ**: このPCの Android 37 タブレット像はとても重く、Chrome やシステム画面が「応答なし」になりやすい(「待つ」で進む)。エミュレーターの `hw.keyboard=yes` だと物理キーボードありの扱いになる。
- **まだ**: 新しい版(タグ)としての公開。本物のタブレット実機での確認。

**English (plain words)**
- **Bugs found (emulator and real phone)**:
  1. **On tablets and in landscape the keyboard sometimes did not appear.** Android's defaults (fullscreen input in landscape, hiding the on-screen keyboard when a hardware keyboard is present) refused the show request. Gboard appeared; aruaru-IME did not.
  2. **On a phone in landscape the keyboard was too tall and hid the text field** (it kept the portrait height).
  3. **In portrait the bottom row was slightly cut off** (the content was taller than 300 dp).
- **Fixes**: no fullscreen mode; the on-screen keyboard is shown even with a hardware keyboard (aruaru-IME has no Japanese conversion for hardware keys). Height is "up to 300 dp, but at most 55% of the screen height", recomputed every time it is shown. The key rows stretch to fit (on short screens: smaller text and padding, and the status line is hidden).
- **Checked**: tablet emulator (2560x1600) in landscape and portrait (typing 「か、」 and 「カ,」); real phone (A401OP) in landscape with the field visible and 「こんにちは」 → 「今日は」, and back in portrait; the browser at 393x300 and 860x216; 6 unit tests pass.
- **Where**: `AruaruImeService.java` (`onEvaluateFullscreenMode`, `onEvaluateInputViewShown`, `onShowInputRequested`, `keyboardHeightPx`, `fitHeight`) and the CSS in `crates/aruaru-ime-android/src/ui.rs`.
- **Notes**: the Android 37 tablet image is very heavy on this PC; Chrome and System UI often show "not responding" (tap "Wait"). An emulator with `hw.keyboard=yes` counts as having a hardware keyboard.
- **Not yet**: a new tagged release; a check on a real tablet.

## 2026-11-20 変換候補が少なすぎた不具合(まさひろ→正浩、ひろし→浩) / Too few conversion candidates (masahiro → 正浩, hiroshi → 浩)

**日本語(やさしい説明)**
- **不具合**: 「まさひろ」を「正浩」に、「ひろし」を「浩」に変換できなかった。辞書には入っていたが、(1)スマホ・タブレットのキーボードが候補を**先頭12個しか並べていなかった**、(2)エンジンが辞書から出す候補を**30個で打ち切っていた**(まさひろは約130語ある)ため、15番目の「正浩」「浩」が画面に出なかった。
- **直したこと**:
  - キーボードは候補を**すべて**横に並べ、「空白 / 変換」を押すたびに**次々と次の候補**を選び、選んだ候補まで自動でスクロールする(`crates/aruaru-ime-android/src/ui.rs`)。
  - エンジンが辞書から出す候補の上限を 30 → **200** に増やした(`convert.rs` の `MAX_DICT_CANDIDATES`)。まさひろは約200個、ひろしは約210個の候補になる。
  - 試験 `names_with_many_homophones_are_all_reachable` を追加: 正浩・浩が候補にあり、変換を押し続けると着いて確定できること、まさひろの候補が100個以上あること。エンジンの試験205件すべて成功。
- **確かめたこと**: 実機スマホ(A401OP)で「masahiro」→ 変換を押し続けて「正浩」を確定。VPS のエンジンを差し替え(別ポートで試してから、元は `aruaru-ime-daemon.bak-20261120b`)、公開 API で「ひろし」の候補が211個・「浩」を含むことを確認、エラーのログ0件。Windows 版は候補窓がページ送りなので、もともと全部たどれる(上限の引き上げは次のリリースで入る)。
- **これからの予定**: 新しい版(v0.2.28)として公開(スマホのアプリはこの公開で届く)。よく使う人名(正浩・浩など)を上の方に出す順位づけの改善。タブレット実機での確認。Windows インストーラーの入れ直し試験(UAC はご本人の操作)。

**English (plain words)**
- **Bug**: 「まさひろ」 could not become 「正浩」 and 「ひろし」 could not become 「浩」. Both are in the dictionary, but (1) the phone/tablet keyboard **showed only the first 12 candidates**, and (2) the engine **cut the dictionary candidates at 30** (まさひろ has about 130 spellings), so 正浩 and 浩, both 15th, never appeared.
- **Fixes**:
  - The keyboard now lists **all** candidates in a scrolling row; each press of "space / convert" selects **the next candidate, one after another**, and the row scrolls to it (`crates/aruaru-ime-android/src/ui.rs`).
  - The engine's cap on dictionary candidates went from 30 to **200** (`MAX_DICT_CANDIDATES` in `convert.rs`): about 200 candidates for まさひろ and 210 for ひろし.
  - New test `names_with_many_homophones_are_all_reachable`: 正浩 and 浩 are candidates, pressing convert repeatedly reaches and commits them, and まさひろ has 100+ candidates. All 205 engine tests pass.
- **Checked**: on a real phone (A401OP), "masahiro" → pressing convert repeatedly → 「正浩」 committed. The VPS engine was swapped (tried on another port first; the old one is `aruaru-ime-daemon.bak-20261120b`); the public API returns 211 candidates for ひろし including 浩, with no error logs. The Windows candidate window already pages through every candidate (the higher cap arrives with the next release).
- **Planned**: publish as a new version (v0.2.28), which is how the phone app gets the fix; rank common given names (正浩, 浩, ...) higher; check on a real tablet; the Windows installer re-install test (the owner presses "Yes" on UAC).

## 2026-11-20 v0.2.28 の公開と現状・予定 / v0.2.28 released: status and plans

### 日本語
**v0.2.28 で直したこと**: 「まさひろ→正浩」「ひろし→浩」のように、同じ読みの語が多い言葉も、「変換」を押し続ければ全部の候補をたどれるようになりました(スマホのキーボードは前は12個までしか出さず、エンジンは30個で打ち切っていました)。タブレットや横向きでキーボードが出なかったり、入力欄が隠れたりする不具合も直しました。
**今できること**: Windows・macOS・Linux・Android(スマホ・タブレット)で、ローマ字から日本語へのライブ変換、ひらがな・カタカナ・英字・記号の切り替え、学習、定型文。変換エンジンはサーバー(easy-web.tokyo)でも動いています。
**これからの予定**: よく使う人名を候補の上の方に出す順位づけ、本物のタブレットでの確認、Windows インストーラーの入れ直し試験(管理者の許可「はい」はご本人が押します)。

### English
**Fixed in v0.2.28**: words with many spellings for one reading, such as 「まさひろ」→「正浩」 and 「ひろし」→「浩」, can now be reached by pressing convert repeatedly (the phone keyboard used to show only 12 candidates, and the engine stopped at 30). The keyboard also appears reliably on tablets and in landscape, without hiding the text field.
**What works now**: live romaji-to-Japanese conversion on Windows, macOS, Linux and Android (phones and tablets), switching between hiragana, katakana, Latin letters and symbols, learning, and templates. The engine also runs on the server (easy-web.tokyo).
**Planned**: ranking common given names higher, a check on a real tablet, and a full Windows installer re-install test (the owner presses "Yes" on the administrator prompt).

### 简体中文
**v0.2.28 的修复**:像"まさひろ→正浩""ひろし→浩"这样同一读音有很多写法的词,现在只要连续按"变换",就能依次看到所有候选词(以前手机键盘最多只显示12个,引擎在30个就停止)。平板和横屏时键盘不出现、挡住输入框的问题也已修好。
**现在能做的**:在 Windows、macOS、Linux、Android(手机和平板)上把罗马字实时转换成日语,切换平假名、片假名、英文字母和符号,学习用词和常用语。转换引擎也在服务器(easy-web.tokyo)上运行。
**接下来的计划**:把常用人名排得更靠前、在真实平板上确认、完整测试 Windows 安装程序的重新安装(管理员确认需要本人点击"是")。

### 繁體中文
**v0.2.28 的修正**:像「まさひろ→正浩」「ひろし→浩」這種同一讀音有許多寫法的詞,現在只要連續按「變換」,就能依序看到所有候選字(以前手機鍵盤最多只顯示12個,引擎到30個就停止)。平板與橫向時鍵盤不出現、擋住輸入框的問題也已修好。
**現在能做的**:在 Windows、macOS、Linux、Android(手機與平板)上將羅馬字即時轉換成日文,切換平假名、片假名、英文字母與符號,學習用詞與常用語。轉換引擎也在伺服器(easy-web.tokyo)上執行。
**接下來的計畫**:將常用人名排得更前面、在實際平板上確認、完整測試 Windows 安裝程式的重新安裝(管理員確認需由本人按「是」)。

### 한국어
**v0.2.28에서 고친 점**: "まさひろ→正浩", "ひろし→浩"처럼 읽는 법은 같고 표기가 많은 단어도, 이제 "변환"을 계속 누르면 모든 후보를 차례로 볼 수 있습니다(예전에는 휴대폰 키보드가 12개까지만 보여 주고, 엔진은 30개에서 멈췄습니다). 태블릿이나 가로 화면에서 키보드가 안 나오거나 입력란을 가리던 문제도 고쳤습니다.
**지금 할 수 있는 것**: Windows, macOS, Linux, Android(휴대폰·태블릿)에서 로마자를 일본어로 실시간 변환하고, 히라가나·가타카나·영문자·기호를 전환하며, 학습과 상용구를 씁니다. 변환 엔진은 서버(easy-web.tokyo)에서도 동작합니다.
**앞으로의 계획**: 자주 쓰는 사람 이름을 위쪽에 보여 주는 순위 개선, 실제 태블릿에서 확인, Windows 설치 프로그램 재설치 전체 시험(관리자 확인의 "예"는 본인이 누릅니다).

### Deutsch
**Behoben in v0.2.28**: Wörter mit vielen Schreibweisen für eine Lesung, etwa 「まさひろ」→「正浩」 und 「ひろし」→「浩」, lassen sich jetzt erreichen, indem man „Umwandeln“ mehrmals drückt (die Handy-Tastatur zeigte bisher nur 12 Vorschläge, und die Engine hörte bei 30 auf). Auf Tablets und im Querformat erscheint die Tastatur jetzt zuverlässig und verdeckt das Eingabefeld nicht mehr.
**Was jetzt funktioniert**: Live-Umwandlung von Romaji in Japanisch unter Windows, macOS, Linux und Android (Handy und Tablet), Wechsel zwischen Hiragana, Katakana, lateinischen Buchstaben und Symbolen, Lernen und Textbausteine. Die Engine läuft auch auf dem Server (easy-web.tokyo).
**Geplant**: häufige Vornamen weiter oben anzeigen, Prüfung auf einem echten Tablet, ein vollständiger Neuinstallations-Test des Windows-Installers (das „Ja“ bei der Administrator-Abfrage drückt der Besitzer selbst).

### Français
**Corrigé dans la v0.2.28** : les mots ayant de nombreuses graphies pour une même lecture, comme 「まさひろ」→「正浩」 et 「ひろし」→「浩」, sont désormais accessibles en appuyant plusieurs fois sur « Convertir » (le clavier du téléphone n'affichait que 12 suggestions et le moteur s'arrêtait à 30). Le clavier apparaît aussi correctement sur tablette et en mode paysage, sans cacher le champ de saisie.
**Ce qui fonctionne aujourd'hui** : conversion en direct du romaji vers le japonais sur Windows, macOS, Linux et Android (téléphones et tablettes), passage entre hiragana, katakana, lettres latines et symboles, apprentissage et modèles de texte. Le moteur tourne aussi sur le serveur (easy-web.tokyo).
**Prévu** : mieux classer les prénoms courants, vérifier sur une vraie tablette, et un test complet de réinstallation du programme d'installation Windows (c'est le propriétaire qui clique sur « Oui » à la demande d'administrateur).

### Español
**Corregido en v0.2.28**: las palabras con muchas escrituras para una misma lectura, como 「まさひろ」→「正浩」 y 「ひろし」→「浩」, ahora se alcanzan pulsando «Convertir» varias veces (antes el teclado del móvil mostraba solo 12 candidatos y el motor se detenía en 30). El teclado también aparece bien en tabletas y en horizontal, sin tapar el campo de texto.
**Lo que funciona ahora**: conversión en vivo de romaji a japonés en Windows, macOS, Linux y Android (móviles y tabletas), cambio entre hiragana, katakana, letras latinas y símbolos, aprendizaje y plantillas. El motor también funciona en el servidor (easy-web.tokyo).
**Previsto**: mostrar más arriba los nombres de pila habituales, comprobarlo en una tableta real y una prueba completa de reinstalación del instalador de Windows (el «Sí» del aviso de administrador lo pulsa el propietario).

### Português
**Corrigido na v0.2.28**: palavras com muitas grafias para a mesma leitura, como 「まさひろ」→「正浩」 e 「ひろし」→「浩」, agora são alcançadas tocando em «Converter» várias vezes (antes o teclado do celular mostrava só 12 candidatos e o motor parava em 30). O teclado também aparece corretamente em tablets e na horizontal, sem cobrir o campo de texto.
**O que funciona agora**: conversão ao vivo de romaji para japonês no Windows, macOS, Linux e Android (celulares e tablets), troca entre hiragana, katakana, letras latinas e símbolos, aprendizado e modelos de texto. O motor também roda no servidor (easy-web.tokyo).
**Previsto**: mostrar nomes próprios comuns mais acima, verificar num tablet real e um teste completo de reinstalação do instalador do Windows (o «Sim» do aviso de administrador é tocado pelo próprio dono).

### Русский
**Исправлено в v0.2.28**: слова с множеством написаний для одного чтения, например 「まさひろ」→「正浩」 и 「ひろし」→「浩」, теперь можно найти, нажимая «Преобразовать» несколько раз (раньше клавиатура телефона показывала только 12 вариантов, а движок останавливался на 30). На планшетах и в горизонтальном положении клавиатура теперь появляется надёжно и не закрывает поле ввода.
**Что работает сейчас**: преобразование ромадзи в японский текст на лету в Windows, macOS, Linux и Android (телефоны и планшеты), переключение между хираганой, катаканой, латиницей и символами, обучение и шаблоны текста. Движок также работает на сервере (easy-web.tokyo).
**В планах**: поднимать частые личные имена выше в списке, проверка на настоящем планшете и полный тест переустановки Windows-установщика (кнопку «Да» в запросе администратора нажимает сам владелец).

(日本語・英語以外はAI翻訳で、ネイティブの確認前です / Languages other than Japanese and English are AI translations, not yet checked by native speakers)

## 2026-11-20 人名の候補の改善(v0.2.29) / Better given-name candidates (v0.2.29)
(日本語・英語以外はAI翻訳で、ネイティブの確認前です / Languages other than Japanese and English are AI translations, not yet checked by native speakers)

### 日本語
- **よく使う名前を上の方に**: 「まさひろ→正浩」「ひろし→浩」のような、よく使う名前の表記を、いちばん上の候補のすぐ次(上位3つ)に出します。前は15番目あたりでした(`crates/aruaru-ime-core/src/names.rs`)。
- **辞書に無い名前も必ず出す**: 辞書に無い組み合わせの名前も、名前によく使う読みの部品(まさ+ひろ、たか+のぶ)をつないで表記を作り、候補に必ず入れます。例: 「たかのぶ→隆信」。
- **確かめたこと**: 試験207件すべて成功。このPCに v0.2.29 のインストーラーで入れ直し(UACはご本人が許可)、正浩2番目・浩3番目・隆信2番目を確認。VPS のエンジンも差し替え(元は `aruaru-ime-daemon.bak-20261120c`)、公開 API で同じ結果、エラー0件。スマホのキーボードは VPS で変換するので、もう届いています。
- **インストール試験で分かったこと**: アンインストールは成功。その直後の再インストールは、Windows が「再起動が必要」と判断して止まりました(使用中の IME の DLL は再起動しないと消えないため)。次のサインインで自動で入れる予約をしました。
- **これから**: 名前の一覧を増やす、Windows の再起動後の自動インストールの確認、v0.2.29 の公開(Release)。

### English
- **Common names come first**: common spellings of given names, such as 「まさひろ」→「正浩」 and 「ひろし」→「浩」, now appear right after the top candidate (top 3). Before, they were around 15th (`crates/aruaru-ime-core/src/names.rs`).
- **Names not in the dictionary always appear**: for a name the dictionary lacks, the engine joins common name parts (まさ+ひろ, たか+のぶ) and always adds the result. Example: 「たかのぶ」→「隆信」.
- **Checked**: all 207 tests pass. Re-installed with the v0.2.29 installer on this PC (the owner approved UAC): 正浩 2nd, 浩 3rd, 隆信 2nd. The VPS engine was swapped too (old one kept as `aruaru-ime-daemon.bak-20261120c`); the public API gives the same result with no errors. The phone keyboard converts on the VPS, so it already has the fix.
- **Installer test finding**: uninstall works. A re-install right after it stopped because Windows wants a restart (an IME DLL in use can only be removed after a restart). The install is scheduled to run at the next sign-in.
- **Next**: a longer name list, checking the after-restart install, and publishing v0.2.29 as a Release.

### 简体中文
常用人名的写法(まさひろ→正浩、ひろし→浩)现在排在第一个候选之后的前三位。词典里没有的名字,也会用常见的名字部件(まさ+ひろ、たか+のぶ)组合出来,一定出现在候选里(例:たかのぶ→隆信)。207项测试全部通过,已在本机安装程序和服务器上确认。卸载后立即重新安装需要重启 Windows,已预约在下次登录时自动安装。
### 繁體中文
常用人名的寫法(まさひろ→正浩、ひろし→浩)現在排在第一個候選之後的前三位。字典裡沒有的名字,也會用常見的名字部件(まさ+ひろ、たか+のぶ)組合出來,一定出現在候選中(例:たかのぶ→隆信)。207項測試全部通過,已在本機安裝程式與伺服器上確認。解除安裝後立即重新安裝需要重新啟動 Windows,已預約在下次登入時自動安裝。
### 한국어
자주 쓰는 이름 표기(まさひろ→正浩, ひろし→浩)가 이제 첫 후보 바로 다음(상위 3개)에 나옵니다. 사전에 없는 이름도 자주 쓰는 이름 부품(まさ+ひろ, たか+のぶ)을 이어 표기를 만들어 반드시 후보에 넣습니다(예: たかのぶ→隆信). 시험 207건 모두 통과, 이 PC의 설치 프로그램과 서버에서 확인했습니다. 제거 직후 재설치는 Windows 재시작이 필요해, 다음 로그인 때 자동 설치하도록 예약했습니다.
### Deutsch
Häufige Schreibweisen von Vornamen (まさひろ→正浩, ひろし→浩) stehen jetzt direkt nach dem ersten Vorschlag (unter den ersten drei). Namen, die nicht im Wörterbuch stehen, werden aus üblichen Namensteilen (まさ+ひろ, たか+のぶ) gebildet und erscheinen immer (z. B. たかのぶ→隆信). Alle 207 Tests bestehen; mit dem Installer auf diesem PC und auf dem Server geprüft. Eine Neuinstallation direkt nach der Deinstallation braucht einen Windows-Neustart; sie ist für die nächste Anmeldung geplant.
### Français
Les graphies courantes des prénoms (まさひろ→正浩, ひろし→浩) apparaissent maintenant juste après la première suggestion (parmi les trois premières). Les prénoms absents du dictionnaire sont formés à partir d'éléments de prénom courants (まさ+ひろ, たか+のぶ) et apparaissent toujours (ex. たかのぶ→隆信). Les 207 tests réussissent ; vérifié avec l'installateur sur ce PC et sur le serveur. Une réinstallation juste après la désinstallation demande un redémarrage de Windows ; elle est programmée à la prochaine connexion.
### Español
Las escrituras habituales de nombres de pila (まさひろ→正浩, ひろし→浩) aparecen ahora justo después del primer candidato (entre los tres primeros). Los nombres que no están en el diccionario se forman con partes de nombre habituales (まさ+ひろ, たか+のぶ) y siempre aparecen (p. ej. たかのぶ→隆信). Pasan las 207 pruebas; comprobado con el instalador en este PC y en el servidor. Reinstalar justo después de desinstalar requiere reiniciar Windows; quedó programado para el próximo inicio de sesión.
### Português
As grafias comuns de nomes próprios (まさひろ→正浩, ひろし→浩) agora aparecem logo após o primeiro candidato (entre os três primeiros). Nomes que não estão no dicionário são formados com partes comuns de nomes (まさ+ひろ, たか+のぶ) e sempre aparecem (ex.: たかのぶ→隆信). Os 207 testes passam; verificado com o instalador neste PC e no servidor. Reinstalar logo após desinstalar exige reiniciar o Windows; ficou agendado para o próximo login.
### Русский
Частые написания личных имён (まさひろ→正浩, ひろし→浩) теперь стоят сразу после первого варианта (в первой тройке). Имена, которых нет в словаре, собираются из частых частей имён (まさ+ひろ, たか+のぶ) и всегда появляются (например, たかのぶ→隆信). Все 207 тестов проходят; проверено установщиком на этом ПК и на сервере. Переустановка сразу после удаления требует перезагрузки Windows; она запланирована на следующий вход в систему.

## 2026-11-20 追記: 人名の一覧を拡充 / More given names
(日本語・英語以外はAI翻訳、ネイティブ確認前 / non-JA/EN parts are AI translations, not native-checked)
- **日本語**: よく使う名前の一覧を約60件から約110件に増やしました(あきひろ・みのる・ひろみ・たけし など)。よく使う名前の読みでは、長い地名の予測(ひろし→広島県)を名前の前に割り込ませないようにし、「ひろし→浩」は2番目になりました。試験207件成功、VPS のエンジンを差し替え(元は `aruaru-ime-daemon.bak-20261120d`)、公開 API で確認、エラー0件。Windows・スマホのアプリへの反映は次のリリースで入ります(スマホのキーボードは VPS で変換するので、すでに反映済み)。
- **English**: the common-name list grew from about 60 to about 110 readings (あきひろ, みのる, ひろみ, たけし, ...). For common name readings, long place-name predictions (ひろし→広島県) no longer jump ahead of the names, so 浩 is now 2nd for ひろし. 207 tests pass; the VPS engine was swapped (old one kept as `aruaru-ime-daemon.bak-20261120d`) and checked through the public API with no errors. The phone keyboard converts on the VPS, so it already has this; the Windows app gets it with the next release.
- **简体中文**: 常用人名列表从约60个增加到约110个;常用人名的读音不再被长地名预测(ひろし→広島県)插队,"ひろし→浩"现在排第2。
- **繁體中文**: 常用人名清單從約60個增加到約110個;常用人名的讀音不再被長地名預測(ひろし→広島県)插隊,「ひろし→浩」現在排第2。
- **한국어**: 자주 쓰는 이름 목록을 약 60개에서 약 110개로 늘렸습니다. 이름 읽기에서 긴 지명 예측(ひろし→広島県)이 끼어들지 않아 "ひろし→浩"가 2번째가 되었습니다.
- **Deutsch**: Die Liste häufiger Vornamen wuchs von etwa 60 auf etwa 110 Lesungen; lange Ortsnamen-Vorschläge (ひろし→広島県) drängen sich nicht mehr vor die Namen, 浩 steht jetzt an 2. Stelle.
- **Français**: la liste des prénoms courants passe d'environ 60 à environ 110 lectures ; les longues prédictions de noms de lieux (ひろし→広島県) ne passent plus devant les prénoms, 浩 est désormais 2ᵉ.
- **Español**: la lista de nombres habituales pasó de unos 60 a unas 110 lecturas; las predicciones largas de lugares (ひろし→広島県) ya no se adelantan a los nombres, y 浩 queda 2.º.
- **Português**: a lista de nomes comuns passou de cerca de 60 para cerca de 110 leituras; previsões longas de lugares (ひろし→広島県) não passam mais à frente dos nomes, e 浩 agora é o 2.º.
- **Русский**: список частых имён вырос примерно с 60 до 110 чтений; длинные подсказки географических названий (ひろし→広島県) больше не встают перед именами, 浩 теперь на 2-м месте.

## 2026-11-21 v0.2.30 公開 / v0.2.30 released
- **日本語**: よく使う人名の一覧(約110件)と「ひろし→浩」で地名が割り込まない修正を、Windows・macOS・Linux・Androidの配布物に入れました。Release・VPSの配布ファイル・紹介ページはv0.2.30です。
- **English**: The expanded given-name list (about 110 readings) and the fix that stops place names cutting in (e.g. ひろし→浩) now ship in the Windows, macOS, Linux and Android builds. The Release, VPS downloads and intro page are at v0.2.30.
- **简体中文**: 常用人名列表(约110个)以及"ひろし→浩"不再被地名插队的修复,已加入 Windows、macOS、Linux、Android 安装包。
- **繁體中文**: 常用人名清單(約110個)以及「ひろし→浩」不再被地名插隊的修正,已加入 Windows、macOS、Linux、Android 安裝檔。
- **한국어**: 자주 쓰는 이름 목록(약 110개)과 "ひろし→浩"에서 지명이 끼어들지 않게 한 수정을 Windows·macOS·Linux·Android 배포판에 넣었습니다.
- **Deutsch**: Die erweiterte Vornamenliste (etwa 110 Lesungen) und die Korrektur, dass sich Ortsnamen nicht mehr vordrängen (z. B. ひろし→浩), sind jetzt in den Builds für Windows, macOS, Linux und Android.
- **Français**: La liste élargie de prénoms (environ 110 lectures) et la correction empêchant les noms de lieux de s'intercaler (ex. ひろし→浩) sont désormais livrées pour Windows, macOS, Linux et Android.
- **Español**: La lista ampliada de nombres de pila (unas 110 lecturas) y la corrección para que los topónimos no se cuelen (p. ej. ひろし→浩) ya vienen en las versiones de Windows, macOS, Linux y Android.
- **Português**: A lista ampliada de nomes próprios (cerca de 110 leituras) e a correção para que nomes de lugares não se intrometam (ex.: ひろし→浩) já vêm nas versões para Windows, macOS, Linux e Android.
- **Русский**: Расширенный список личных имён (около 110 чтений) и исправление, при котором названия мест больше не вклиниваются (например, ひろし→浩), теперь входят в сборки для Windows, macOS, Linux и Android.
(日本語・英語以外はAI翻訳 / non-JA/EN lines are AI translations)


## 2026-11-23 追記: url 2.5.9 へ更新 / Update: url 2.5.9
**日本語**: RUSTSEC-2026-0592(`Url::origin()` のスタックあふれ)対応で `url` を 2.5.8→2.5.9 に更新(207件成功(cargo test --workspace))。VPS のクローンも追従済み。稼働バイナリは未入れ替え。再開時は未コミットの作業中変更の有無を `git status` で確認。
**English**: Updated `url` 2.5.8 to 2.5.9 for RUSTSEC-2026-0592 (stack overflow in `Url::origin()`); 207 tests pass (cargo test --workspace). The VPS clone is fast-forwarded; running binaries were not swapped. On resume, check `git status` for uncommitted work.
