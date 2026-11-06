# aruaru-IME

画面の文脈から「いま何を打とうとしているか」を予測して変換する AI IME。漢字に変換することを前提に、打つそばから変換結果を出します(ライブ変換)。
An AI-assisted IME that predicts what you are typing from the screen and the surrounding text, and shows the converted result (kanji) as you type.

> **状態: v0.1.0 アルファ(Windows のみ)**。日本語入力は実機のアプリ(メモ帳・VS Code・Claude など)で動作しますが、インストーラーは**まだ実機で通し試験していません**。macOS / Linux / Android 版は未着手です。日常の入力には、まだ使わないでください。
> **Status: v0.1.0 alpha (Windows only).** Japanese input works in real apps, but the installer has **not been end-to-end tested yet**, and the macOS / Linux / Android builds are not started.

## できること / Features

| 機能 | 状態 |
|---|---|
| **かな漢字変換**: Mozc OSS 辞書(BSD-3-Clause)+連接コストの最小コスト経路(Viterbi)。1回の変換は約 2 ミリ秒 | 実機で動作 |
| **ライブ変換(既定オン)**: 打つそばから最有力の変換結果を表示。Enter で確定、Space で次の候補、数字キー 1〜9・0 で直接選択(1ページ10件) | 実機で動作 |
| **入力モードの AI 予測**(ひらがな・カタカナ・英字・数字・記号)。直前の文章・画面の OCR・入力欄の種類から判定。誤判定の対策: F6〜F10・無変換キーで表記を切替、英数キー/かなキーで必ずオフ/オン、Shift 始まりは英字、Caps Lock 中は食わない、反映に失敗したら自動でオフ | 実機で動作(一部) |
| **日本語⇔英語の相互翻訳(既定)**: 候補窓の左に日本語、右に英語の訳(JMdict)。←→ で列を切替、↑↓ で選択。英語を打つと日本語の対訳も候補に出る(例: `word` → 単語) | 実機のエンジンで確認(候補窓の2列表示は実機の画面で未確認) |
| **予測変換・学習**: 途中まで打つと続きを予測(ありがとう→ありがとうございます)。選んだ候補を学習。不自然な履歴は自動で捨てる | 実機で動作 |
| **単語・熟語・文章の登録**: 右クリックメニュー/コマンド。`.txt` `.csv` `.xlsx` `.docx` `.pptx` `.pdf` から一括取り込み(読みは辞書の逆引きで自動生成)、ファイル単位で削除 | 単体テスト済み(画面の操作は未確認) |
| **用語辞書(希望用語)**: 漫画・アニメ・ゲーム・ドラマ・映画・言語・フレームワークの名前を入れると、検索(aruaru-search)で調べて、登場人物・必殺技・アイテム名などの読みと表記を辞書化。検索元が一時的に使えないときは自動で再試行 | 検索→辞書化の部品は単体テスト済み。実際の検索での通し試験は、検索元の休止中のため未完 |
| **プログラミング言語・フレームワークの補完と綴り訂正**: Python/Rust/TypeScript/React/PHP/Ruby/Perl/COBOL/Go/Java/SQL(各DB)/HTML5/CSS3/Bootstrap/Next.js/Nuxt.js/GraphQL/WunderGraph Cosmo ほか。未知の名前は検索で語彙を自動作成 | 単体テスト済み |
| **検索フォーム型の予測**(Google サジェスト風)、辞書に無い作品名・固有名詞は Space で Web のサジェストを引く(設定でオフ可) | 実装済み |
| **候補窓**: 黒背景・白文字が既定(見出し右のボタン・F2・設定で白背景・黒文字)、既定で 1.8 倍の大きさ(設定で変更)、アラビア語などは右から左 | 実装済み |
| **旧字体(國・學・體)**: 右クリックメニューで切替 | 単体テスト済み |
| **多言語の入力エンジン**: ロシア語・ウクライナ語・アラビア語・ペルシア語・ヘブライ語・韓国語(ハングル)・ヨーロッパ言語のアクセント。右クリックで最大7言語まで選択 | 規則と単体テストのみ。IME 本体への組み込み(言語の切替)は**未了**。中国語・世界130言語は**未対応** |
| **バックアップと復元**: 辞書・学習履歴・設定・用語辞書を、Google ドライブ(デスクトップ版の同期フォルダ。例 `G:\マイドライブ`)へ自動バックアップ。復元は「足す」「置き換え」 | 実機でバックアップを確認(Google ドライブへ保存) |
| **自動アップデート**(起動時の1回)/ 右クリックの「今すぐ更新」(**アプリの再起動不要**: 変換の状態管理はエンジン側にあるため) | 実装・単体テスト済み(公開リリースからの実取得は未検証) |
| **コンポーネント管理**: aruaru-llm / open-easy-web / Qwen を、重複させず、無ければ導入・有れば再利用 | 実機で確認 |
| **郵便番号 → 国名つきの住所(世界44か国)** / World postal codes → "country + address": `100-0001` でも `1000001` でも(`-` や空白は有っても無くても同じ)。英国 `SW1A 1AA`・カナダ `K1A 0B1`・アイルランドも(地区まで)。候補は IPアドレスから推定した国 → 選択中の言語の国 → その他の順(最大8件、優先国は3件ずつ)。国名は言語に合わせて表示。データは右クリックで取得(GeoNames, CC BY 4.0) | 実機のエンジン(HTTP)で確認 |
| **人名・有名人・重要人物名の辞書(約3300人、既定で搭載)** / People-name dictionary (~3,300, installed by default): Wikidata(CC0)から生成。日本人は読み(かな表記/ヘボン式)、外国人はカタカナの読みと英語名(`isaacnewton` → Isaac Newton)・한글・fr/es/de/it/pt の表記。中国語は簡体・繁体で、ピンインは中国語辞書の逆引きで付与(3291人中3230人。例: `aisakeniudun` → 艾萨克·牛顿) | 実機のエンジンで確認 |
| **毎朝の自動データ更新** / Automatic data refresh every morning: 端末のローカル時刻の朝6〜8時台(端末ごとにずらす)に、**取得済みの**郵便番号データを条件付き取得(ETag/Last-Modified、変更が無ければダウンロードしない)で更新し、国名・主要都市名(人口100万以上)を Wikidata から利用者辞書へ入れ直す。右クリック「設定」でオフ可 | 実データで確認(2回目以降は「変更なし」で省略) |
| **長音の読み** / Long-vowel readings: 辞書の `にゅーとん` を `nyuuton` でも `nyu-ton` でも打てる(ニュートン・ニュージーランド) | 実機のエンジンで確認 |
| **画面キャプチャ**(入力欄にフォーカスがある間、数秒ごと): 一時フォルダに保存し、**60秒より古い/30枚を超えた分から自動削除**。パスワード欄では撮らない | 実機で確認 |

## インストール / Install (Windows)

`aruaru-ime-windows-x86_64-setup.exe`(リリースのページ)を実行します(管理者権限が必要)。インストール時に、IME の登録・入力方式の一覧への追加・エンジンの自動起動・必要な部品(aruaru-llm・Qwen・英語辞書 JMdict)の導入を行います。インストール後は、Win+Space で「aruaru-IME」を選びます。通知領域のアイコンを**右クリック**するとメニューが出ます。

- アンインストールで、IME の登録解除とエンジンの停止を行います。辞書・学習履歴(`%LOCALAPPDATA%\aruaru`)は残します。
- **既に起動しているアプリは、古い版の IME を読み込んだままです。更新・再インストールの後は、アプリを一度終了して起動し直してください。** 以後の更新は、エンジンが入れ替わるだけなので再起動は要りません。

## 構成

- `crates/aruaru-ime-core` — エンジン(OS非依存): ローマ字・モード予測・辞書変換・セッション・利用者辞書・用語辞書・バックアップ・更新・多言語
- `crates/aruaru-ime-daemon` — 常駐エンジン(127.0.0.1:4660): 変換・セッション API、画面キャプチャ+OCR、通知領域のアイコンと右クリックメニュー
- `crates/aruaru-ime-tsf` — Windows の IME 本体(TSF DLL。キーを送って結果を表示するだけの薄い層)
- `crates/aruaru-ime-cli` — `aruaru-ime` コマンド / commands: `convert` `dict` `wish` `backup` `update` `build-dict` `ensure-components` `postal-install`(日本郵便) `geo-install`(世界の郵便番号+現在地の国) `geo <郵便番号> [言語]`(候補の確認) `data-update`(毎朝の更新をいま1回) `people-pinyin <people.tsv>`(人名辞書の拼音付け)
- `installer/` — Inno Setup のインストーラー、`tools/` — 開発用のスクリプト

## 開発 / Build

```
cargo test
# 辞書(Mozc OSS)を用意: data/mozc-src に dictionary00-09.txt・connection_single_column.txt・id.def を置いて
cargo run --release -p aruaru-ime-cli -- build-dict data/mozc-src data/dict.bin
powershell -File tools\build-installer.ps1     # インストーラーを作る(Inno Setup 6 が必要)
powershell -File tools\deploy-dev.ps1 -Dll     # 開発機で、動いているエンジン/DLLを入れ替える
```

## 画面キャプチャと個人情報

- 入力欄にフォーカスがある間だけ、数秒に一度キャプチャします。画像は OS の一時フォルダの専用ディレクトリにだけ置き、新しい1枚を保存するたびに、**60秒より古いもの・30枚を超えたものを古い順に自動削除**します(終了時も全削除、強制終了で残った分は次回起動時に掃除)。
- パスワード欄ではキャプチャしません。ただし**パスワード欄の判定は、アプリ側がパスワード欄でIMEを無効にすることに依存します**(入力スコープでの判定は未実装)。
- 画像の送信先は既定で同じPCの `aruaru-llm`(`http://127.0.0.1:4600`)です。
- **Web のサジェスト**(Google)は、Space で変換を求めたとき、辞書に無い語の読みだけを送ります(パスワード欄では送りません)。不要なら右クリックのメニューでオフにできます。
- バックアップには、変換の学習履歴(あなたが入力した言葉)が含まれます。保存先は Google ドライブの**あなたのフォルダ**です。

## 正直な開示(現時点の限界)

- **郵便番号・人名・毎朝の更新(2026-10-13)の限界 / Limits**: 台湾・ギリシャの郵便番号は GeoNames に無く、ブラジルは市の代表コードのみ、オランダ(`1011AB`)のように数字+英字は未対応。英国・カナダ・アイルランドは地区までで、番地まではわかりません。中国語の人名のピンインは、多音字で誤ることがあります。毎朝の更新は、実機の長期運用(数日)では未確認で、1回の実行と条件付き取得の動作を確認した段階です。右クリックメニュー(辞書・登録を3区分に整理)の見た目は、実機の目視確認が未了です。
  *Taiwan and Greece have no GeoNames postal data, Brazil only city-level codes, and codes such as Dutch `1011AB` (digits + letters) are not supported yet. UK/Canada/Ireland resolve only to the postcode area. Pinyin for Chinese names can be wrong for polyphonic characters. The morning refresh has been verified for a single run and for conditional requests, not yet over days of real use. The regrouped right-click menu has not been checked visually on a real screen.*
- インストーラーは、ビルドできたことまでの確認で、**実機のインストール/アンインストールの通し試験は未了**です。
- 候補窓の2列表示・右クリックメニューの各画面は、コードとエンジン側の確認までで、**実機の画面での目視確認は未了**です。
- Qwen(0.5B)は、この PC では 1 回約 50 秒かかり、候補も不正解だったため、変換の主経路は辞書+Viterbi です。Qwen は補助で、同期の変換経路には置いていません。
- 右クリックで言語を切り替えて入力する機能(ロシア語など)、中国語、世界130言語、GitHub へのバックアップ、タスクバーの「日本」表示の右クリック、macOS / Linux / Android 版は**未実装**です。
- Windows では 64bit アプリのみです。
- 「無制限の検索」は aruaru-search(VPS 上)を使い、検索元(Bing・Brave 等)に拒否されると数十分休みます。短時間に続けて調べないでください。

ライセンス: MIT OR Apache-2.0。辞書データは Mozc の BSD-3-Clause、英語訳は JMdict の CC BY-SA 4.0(`installer/LICENSES.txt`)。

---

## English summary

aruaru-IME is a Windows IME (TSF) that predicts what you are typing from the screen and surrounding text and converts as you type ("live conversion").
It uses the Mozc OSS dictionary (BSD-3-Clause) with a Viterbi lattice (~2 ms per conversion), shows Japanese candidates on the left and English glosses (JMdict, CC BY-SA 4.0, downloaded automatically) on the right (←/→ switch columns, ↑/↓ select, digits 1–9/0 pick), adds Japanese equivalents when you type English, supports user dictionaries (txt/csv/xlsx/docx/pptx/pdf import), term dictionaries built by web search (manga/anime/game/drama/movie/frameworks), programming-term completion and typo correction, auto backup/restore to Google Drive (desktop sync folder), and engine-side sessions so updates need no app restart.

**Alpha, Windows only.** Not yet done: end-to-end tests of the installer and of the on-screen menus, in-IME language switching for Russian/Arabic/Persian/Hebrew/Korean/European languages (engines exist and are unit-tested), Chinese, 130 languages, GitHub backup, taskbar-indicator right-click, macOS/Linux/Android.
Install with `aruaru-ime-windows-x86_64-setup.exe` (admin); right-click the tray icon for the menu. **Restart already-running apps once after installing or updating** (they keep the old thin DLL loaded); later updates only swap the engine.

**New since v0.2 (2026-10-13)**: world postal codes → "country + address" (44 countries, with or without `-`/space; UK/Canada/Ireland down to the postcode area; ordered by your IP's country → the selected language's country → others, max 8), a built-in people-name dictionary (~3,300 people from Wikidata, CC0), long-vowel readings (`nyuuton` → ニュートン), a much faster first start (the dictionary packs are saved once instead of per entry: 246 s → 8 s), and **an automatic morning refresh** of the data you already downloaded (conditional requests, so nothing is re-downloaded if unchanged) plus country / major-city names from Wikidata. The refresh runs at 06:00–08:00 local time (a per-machine offset spreads the load) and can be turned off in the tray menu → Settings. **Nothing is downloaded that you did not choose**; the only automatic additions are country/city names.

**v0.2.11–0.2.13 (2026-10-14)**: Fcitx5 support on Linux (alongside IBus); `, . / \ [ ] @ ^` are typed as half-width in hiragana mode (`{` twice gives `「`; candidate orders for `< > ?`; JIS 109 keyboards on the online screen); the online screen's Space key fix; a web-service name dictionary (ようつべ → youtube, やふー → yahoo); the server mode shares the default dictionaries read-only. macOS is only build-tested in CI (no Mac available).
**v0.2.11〜0.2.13(2026-10-14)**: Linux の Fcitx5 に対応(IBus と併用)。ひらがなモードでも `, . / \ [ ] @ ^` は半角のまま(`{` を2回で `「`、`< > ?` の候補順、オンライン画面で日本語109キーボード)。オンライン画面の Space が効かない不具合を修正。ウェブサービス名の辞書(ようつべ → youtube、やふー → yahoo)。サーバーモードでも共通の用語辞書を読み取り専用で使用。macOS は実機が無く、CI のビルド確認のみ。

**v0.2.15 (2026-10-16)**: Ctrl+Shift+L switches the input language inside the IME (Windows, IBus, Fcitx5, online screen); hiragana mode follows MS-IME for `, . / [ ]` (as Japanese 109 keyboards do).
**v0.2.15(2026-10-16)**: Ctrl+Shift+L で、IME の中で入力言語を切り替え(Windows・IBus・Fcitx5・オンライン画面)。ひらがなモードの `, . / [ ]` は、日本語109キーボードのとおり MS-IME 準拠。

**v0.2.22 (2026-11-06)**: Right-click the notification-area icon → Dictionaries & registration → **Register by genre**: word, phrase, person's name, thing, manga, anime, game, TV drama, movie. Each entry is saved to the user dictionary with the source `登録:<genre>`, so it can be removed per genre from the same menu.
**v0.2.22(2026-11-06)**: 通知領域のアイコンを右クリック → 辞書・登録 → **ジャンルを選んで登録**: 単語・熟語・人名・物・漫画・アニメ・ゲーム・ドラマ・映画。登録した語は出所「登録:ジャンル名」で利用者辞書に入り、同じメニューのファイル単位の削除からジャンルごとにまとめて消せます。

---

## 引っ越し / Moving to another machine

`aruaru-IME` は、このリポジトリだけで開発を続けられます。別の PC・別の作業環境へ移るときは [`PORTING.md`](PORTING.md) の「引っ越し手順 / Moving to another machine」(必要なツール・ビルド・テスト・データの置き場所・環境固有の落とし穴・未了の一覧)と、[`CLAUDE.md`](CLAUDE.md) の HANDOFF を読んでください。
`aruaru-IME` can be developed from this repository alone. To continue on another machine, read "Moving to another machine" in [`PORTING.md`](PORTING.md) (tools, build, tests, data locations, environment pitfalls, open items) and the HANDOFF in [`CLAUDE.md`](CLAUDE.md).

```
git clone https://github.com/aon-co-jp/aruaru-IME.git
cd aruaru-IME
cargo test            # 動作確認(コアのテスト約180件) / sanity check (~180 core tests)
```

ユーザーのデータ(辞書・学習履歴・設定・取得済みの郵便番号)は、リポジトリではなく **データ領域**(Windows: `%LOCALAPPDATA%\aruaru`)にあります。引っ越し先へ持っていくなら、右クリックの「バックアップと復元」(Google ドライブ)か、このフォルダのコピーを使います。
Your data (dictionaries, learning history, settings, downloaded postal data) lives in the **data directory** (Windows: `%LOCALAPPDATA%\aruaru`), not in the repository. Use the tray menu "Backup & restore" (Google Drive) or copy that folder to bring it along.

**お引越し / Moving your data**: `aruaru-ime move export [フォルダ]` で 1 ファイルに書き出し、新しい PC で `aruaru-ime move import <ファイル>` で取り込みます(辞書・学習履歴・設定・用語辞書・取得済みの郵便番号。大きい辞書本体は移った先で自動取得)。 / Export your data to one file with `aruaru-ime move export [dir]` and import it on the new PC with `aruaru-ime move import <file>` (big dictionaries are re-downloaded automatically).

---

作者 / Author: 石 塚 正 浩（MASAHIRO ISHIZUKA） (https://github.com/aon-co-jp)

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
