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
