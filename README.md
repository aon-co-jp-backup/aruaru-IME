# aruaru-IME

画面の文脈から「いま何を打とうとしているか」を予測して変換する AI IME。漢字に変換することを前提に、打つそばから変換結果を出します(ライブ変換)。
An AI-assisted IME that predicts what you are typing from the screen and the surrounding text, and shows the converted result (kanji) as you type.

> **状態: v0.2.26 公開済み(Windows・Android・Linux・macOS)**。Windows ではメモ帳・VS Code などで日本語入力が動きます。右クリックメニューの見た目、メモ帳での実入力、インストーラーの入れ直しは、人の目での確認がまだです。
> **Status: v0.2.26 is released (Windows, Android, Linux, macOS).** Japanese typing works in apps such as Notepad and VS Code on Windows. A person still needs to check how the right-click menu looks, real typing in Notepad, and a re-install with the installer.

## はじめに(やさしい説明)/ What is this? (plain words)

**日本語**: aruaru-IME は「文字を打つ道具(IME)」です。ローマ字で「nihongo」と打つと、すぐ「日本語」と漢字に変えてくれます。画面に書いてある内容も見て、「いま何を打ちたいか」を当てようとします。うまく当たらなくても、ふつうの文字が打てなくなることはありません。インターネットや AI につながらなくても動きます。
- **IME**: キーボードで日本語を打つための道具。
- **ライブ変換**: 打っている途中で、もう漢字にして見せること。
- **Release(リリース)**: 完成したアプリを、ダウンロードできるようにして公開すること。
- **SHA-256**: ファイルが壊れていない・すり替えられていないか調べるための「指紋」。
- **UAC**: Windows が「このインストールを許しますか?」と聞く画面。人が押す必要があります。
- **VPS**: インターネット上に借りている、いつもつけっぱなしのパソコン。

**English**: aruaru-IME is a tool for typing. When you type "nihongo" with letters, it turns it into Japanese characters right away. It also looks at what is on your screen and guesses what you want to write. If the guess is wrong, you can still type normally. It works even when the internet or AI is not available.
- **IME**: a tool that lets you type Japanese on a keyboard.
- **Live conversion**: showing the Japanese characters while you are still typing.
- **Release**: making a finished app ready for everyone to download.
- **SHA-256**: a "fingerprint" of a file. It helps check that the file is not broken or swapped.
- **UAC**: the Windows window that asks "Do you allow this install?" A person has to press the button.
- **VPS**: a computer we rent on the internet that stays switched on all the time.

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

**v0.2.23 (2026-11-06)**: Fix release for v0.2.22: the Linux IBus engine could not connect (zbus 6 runtime feature missing), and the Android app still said 0.2.21 and offered to update to itself. CI is green on all platforms again (the Android smoke test now retries and dismisses the update prompt).
**v0.2.23(2026-11-06)**: v0.2.22 の修正版。Linux の IBus エンジンが接続できない不具合(zbus 6 の実行基盤の機能が抜けていた)と、Android の版が 0.2.21 のままで自分自身への更新を案内する不具合を修正。CI は全 OS で成功に戻した(Android のスモーク試験は、更新案内を閉じ、描画の取りこぼしを再試行する)。

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

(AI翻訳、ネイティブ確認前 / AI translations, not yet checked by native speakers)

### 简体中文
手机和平板键盘:可以选择平假名、片假名、英文字母、符号四种模式(按左上角的按钮)。在平假名和片假名模式下,右上角一直显示「、」和「,」。模式按钮从左下移到左上,下面的「空白/变换」键变宽了。已在真实的安卓手机上确认。
### 繁體中文
手機與平板鍵盤:可選擇平假名、片假名、英文字母、符號四種模式(按左上角的按鈕)。在平假名與片假名模式下,右上角一直顯示「、」與「,」。模式按鈕從左下移到左上,下方的「空白/變換」鍵變寬了。已在實際的 Android 手機上確認。
### 한국어
휴대폰·태블릿 키보드: 히라가나, 가타카나, 알파벳, 기호 네 가지 모드를 고를 수 있습니다(왼쪽 위 버튼). 히라가나·가타카나 모드에서는 오른쪽 위에 「、」와 「,」가 항상 보입니다. 모드 버튼을 왼쪽 아래에서 왼쪽 위로 옮겨, 아래의 「공백/변환」 키가 넓어졌습니다. 실제 안드로이드 폰에서 확인했습니다.
### Deutsch
Tastatur für Handy und Tablet: Man kann zwischen vier Modi wählen – Hiragana, Katakana, Alphabet und Symbole (Taste oben links). Im Hiragana- und Katakana-Modus stehen 「、」 und 「,」 immer oben rechts. Die Modus-Taste ist von unten links nach oben links gewandert, dadurch ist die Leertaste breiter. Auf einem echten Android-Handy geprüft.
### Français
Clavier pour téléphone et tablette : on peut choisir entre quatre modes – hiragana, katakana, alphabet et symboles (bouton en haut à gauche). En hiragana et katakana, 「、」 et 「,」 sont toujours affichés en haut à droite. Le bouton de mode est passé d'en bas à gauche à en haut à gauche, la barre d'espace est donc plus large. Vérifié sur un vrai téléphone Android.
### Español
Teclado para móvil y tableta: se puede elegir entre cuatro modos – hiragana, katakana, alfabeto y símbolos (botón de arriba a la izquierda). En hiragana y katakana, 「、」 y 「,」 aparecen siempre arriba a la derecha. El botón de modo pasó de abajo a la izquierda a arriba a la izquierda, así que la barra espaciadora es más ancha. Comprobado en un teléfono Android real.
### Português
Teclado para celular e tablet: dá para escolher entre quatro modos – hiragana, katakana, alfabeto e símbolos (botão no canto superior esquerdo). Em hiragana e katakana, 「、」 e 「,」 ficam sempre no canto superior direito. O botão de modo foi do canto inferior esquerdo para o superior esquerdo, então a barra de espaço ficou mais larga. Verificado num celular Android real.
### Русский
Клавиатура для телефона и планшета: можно выбрать один из четырёх режимов — хирагана, катакана, латиница и символы (кнопка в левом верхнем углу). В режимах хираганы и катаканы в правом верхнем углу всегда видны 「、」 и 「,」. Кнопка режима переехала из левого нижнего угла в левый верхний, поэтому пробел стал шире. Проверено на настоящем Android-телефоне.

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

(AI翻訳、ネイティブ確認前 / AI translations, not yet checked by native speakers)

### 简体中文
已将手机和平板键盘的新布局作为 v0.2.27 发布。所有平台的自动测试均已通过,服务器上的下载文件也已更新为 0.2.27。
### 繁體中文
已將手機與平板鍵盤的新配置以 v0.2.27 發布。所有平台的自動測試皆已通過,伺服器上的下載檔案也已更新為 0.2.27。
### 한국어
휴대폰·태블릿 키보드의 새 배치를 v0.2.27로 공개했습니다. 모든 플랫폼의 자동 테스트가 통과했고, 서버의 다운로드 파일도 0.2.27로 맞췄습니다.
### Deutsch
Das neue Tastaturlayout für Handy und Tablet ist als v0.2.27 veröffentlicht. Alle automatischen Tests auf allen Plattformen sind bestanden, und die Download-Dateien auf dem Server sind auf 0.2.27 aktualisiert.
### Français
La nouvelle disposition du clavier pour téléphone et tablette est publiée en v0.2.27. Tous les tests automatiques sont passés sur toutes les plateformes, et les fichiers de téléchargement du serveur sont passés en 0.2.27.
### Español
La nueva distribución del teclado para móvil y tableta se publicó como v0.2.27. Todas las pruebas automáticas pasaron en todas las plataformas y los archivos de descarga del servidor ya están en 0.2.27.
### Português
O novo layout do teclado para celular e tablet foi publicado como v0.2.27. Todos os testes automáticos passaram em todas as plataformas, e os arquivos de download do servidor já estão em 0.2.27.
### Русский
Новая раскладка клавиатуры для телефона и планшета выпущена как v0.2.27. Все автоматические тесты на всех платформах пройдены, файлы для скачивания на сервере обновлены до 0.2.27.

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

(AI翻訳、ネイティブ確認前 / AI translations, not yet checked by native speakers)

### 简体中文
服务器上的转换引擎也已更新为 v0.2.27。先在其他端口试运行确认无误后才替换,旧版本已保留,可随时恢复。在真实手机上也确认可以正常转换。
### 繁體中文
伺服器上的轉換引擎也已更新為 v0.2.27。先在其他連接埠試執行確認無誤後才替換,舊版已保留,可隨時還原。在實際手機上也確認可正常轉換。
### 한국어
서버의 변환 엔진도 v0.2.27로 교체했습니다. 다른 포트에서 먼저 시험 실행해 문제가 없음을 확인한 뒤 교체했고, 이전 버전은 남겨 두어 언제든 되돌릴 수 있습니다. 실제 휴대폰에서도 변환이 잘 되는 것을 확인했습니다.
### Deutsch
Auch die Umwandlungs-Engine auf dem Server läuft jetzt mit v0.2.27. Sie wurde vorher auf einem anderen Port getestet; die alte Version ist gesichert und kann jederzeit zurückgespielt werden. Auf einem echten Handy funktioniert die Umwandlung.
### Français
Le moteur de conversion du serveur est lui aussi passé en v0.2.27. Il a d'abord été testé sur un autre port ; l'ancienne version est conservée et peut être rétablie à tout moment. La conversion fonctionne sur un vrai téléphone.
### Español
El motor de conversión del servidor también se actualizó a v0.2.27. Antes se probó en otro puerto; la versión anterior se guardó y se puede restaurar en cualquier momento. La conversión funciona en un teléfono real.
### Português
O motor de conversão do servidor também foi atualizado para v0.2.27. Antes foi testado em outra porta; a versão anterior foi guardada e pode ser restaurada a qualquer momento. A conversão funciona num celular real.
### Русский
Движок преобразования на сервере тоже обновлён до v0.2.27. Сначала его проверили на другом порту; старая версия сохранена, и её можно вернуть в любой момент. Преобразование работает на настоящем телефоне.

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

(AI翻訳、ネイティブ確認前 / AI translations, not yet checked by native speakers)

### 简体中文
修正了平板和横屏时的问题:①平板或横屏时键盘有时不显示;②手机横屏时键盘太高,挡住输入框;③竖屏时最下面一行被切掉一点。现在键盘高度最多为屏幕高度的 55%,按键行会随高度伸缩。已在平板模拟器(横/竖)和真实手机(横/竖)上确认。
### 繁體中文
修正了平板與橫向時的問題:①平板或橫向時鍵盤有時不顯示;②手機橫向時鍵盤太高,擋住輸入框;③直向時最下面一列被切掉一點。現在鍵盤高度最多為螢幕高度的 55%,按鍵列會隨高度伸縮。已在平板模擬器(橫/直)與實際手機(橫/直)上確認。
### 한국어
태블릿과 가로 화면의 문제를 고쳤습니다: ① 태블릿이나 가로 화면에서 키보드가 안 나올 때가 있었음, ② 휴대폰 가로 화면에서 키보드가 너무 높아 입력란을 가렸음, ③ 세로 화면에서 맨 아래 줄이 조금 잘렸음. 이제 키보드 높이는 화면 높이의 최대 55%이고, 키 줄이 높이에 맞춰 늘고 줄어듭니다. 태블릿 에뮬레이터(가로/세로)와 실제 휴대폰(가로/세로)에서 확인했습니다.
### Deutsch
Probleme auf Tablets und im Querformat behoben: ① Auf Tablets oder im Querformat erschien die Tastatur manchmal nicht; ② auf dem Handy im Querformat war die Tastatur zu hoch und verdeckte das Eingabefeld; ③ im Hochformat war die unterste Reihe leicht abgeschnitten. Die Tastatur ist jetzt höchstens 55 % der Bildschirmhöhe hoch, und die Tastenreihen passen sich an. Geprüft im Tablet-Emulator (quer/hoch) und auf einem echten Handy (quer/hoch).
### Français
Corrections pour tablettes et mode paysage : ① sur tablette ou en paysage, le clavier n'apparaissait parfois pas ; ② sur téléphone en paysage, le clavier était trop haut et cachait le champ de saisie ; ③ en portrait, la dernière rangée était un peu coupée. Le clavier fait désormais au plus 55 % de la hauteur de l'écran et les rangées s'adaptent. Vérifié sur l'émulateur de tablette (paysage/portrait) et sur un vrai téléphone (paysage/portrait).
### Español
Arreglos para tabletas y modo horizontal: ① en tabletas o en horizontal, a veces el teclado no aparecía; ② en el móvil en horizontal, el teclado era demasiado alto y tapaba el campo de texto; ③ en vertical, la última fila se cortaba un poco. Ahora el teclado ocupa como máximo el 55 % de la altura de la pantalla y las filas se ajustan. Comprobado en el emulador de tableta (horizontal/vertical) y en un móvil real (horizontal/vertical).
### Português
Correções para tablets e modo paisagem: ① em tablets ou na horizontal, às vezes o teclado não aparecia; ② no celular na horizontal, o teclado era alto demais e cobria o campo de texto; ③ na vertical, a última fileira ficava um pouco cortada. Agora o teclado ocupa no máximo 55% da altura da tela e as fileiras se ajustam. Verificado no emulador de tablet (horizontal/vertical) e num celular real (horizontal/vertical).
### Русский
Исправлены проблемы на планшетах и в горизонтальной ориентации: ① на планшете или в горизонтальном положении клавиатура иногда не появлялась; ② на телефоне в горизонтальном положении клавиатура была слишком высокой и закрывала поле ввода; ③ в вертикальном положении нижний ряд немного обрезался. Теперь высота клавиатуры — не больше 55 % высоты экрана, а ряды клавиш подстраиваются. Проверено на эмуляторе планшета (горизонтально/вертикально) и на настоящем телефоне (горизонтально/вертикально).

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

(AI翻訳、ネイティブ確認前 / AI translations, not yet checked by native speakers)

### 简体中文
修正了"まさひろ→正浩""ひろし→浩"无法转换的问题。原因是手机键盘只显示前12个候选词,而引擎只取前30个。现在显示全部候选词,每按一次"变换"就依次选下一个。已在真实手机和服务器上确认。
### 繁體中文
修正了「まさひろ→正浩」「ひろし→浩」無法轉換的問題。原因是手機鍵盤只顯示前12個候選字,而引擎只取前30個。現在顯示全部候選字,每按一次「變換」就依序選下一個。已在實際手機與伺服器上確認。
### 한국어
"まさひろ→正浩", "ひろし→浩" 변환이 안 되던 문제를 고쳤습니다. 휴대폰 키보드가 후보를 12개만 보여 주고, 엔진이 30개에서 잘랐기 때문입니다. 이제 모든 후보를 보여 주고, "변환"을 누를 때마다 다음 후보를 차례로 고릅니다. 실제 휴대폰과 서버에서 확인했습니다.
### Deutsch
Behoben: 「まさひろ→正浩」 und 「ひろし→浩」 ließen sich nicht umwandeln. Die Handy-Tastatur zeigte nur 12 Vorschläge, und die Engine schnitt nach 30 ab. Jetzt werden alle Vorschläge gezeigt, und jedes Drücken von „Umwandeln“ wählt den nächsten. Auf einem echten Handy und auf dem Server geprüft.
### Français
Corrigé : 「まさひろ→正浩」 et 「ひろし→浩」 ne se convertissaient pas. Le clavier du téléphone n'affichait que 12 suggestions et le moteur s'arrêtait à 30. Désormais toutes les suggestions s'affichent, et chaque appui sur « Convertir » choisit la suivante. Vérifié sur un vrai téléphone et sur le serveur.
### Español
Arreglado: 「まさひろ→正浩」 y 「ひろし→浩」 no se convertían. El teclado del móvil mostraba solo 12 candidatos y el motor cortaba en 30. Ahora se muestran todos, y cada pulsación de «Convertir» elige el siguiente. Comprobado en un móvil real y en el servidor.
### Português
Corrigido: 「まさひろ→正浩」 e 「ひろし→浩」 não eram convertidos. O teclado do celular mostrava só 12 candidatos e o motor parava em 30. Agora todos aparecem, e cada toque em «Converter» escolhe o próximo. Verificado num celular real e no servidor.
### Русский
Исправлено: 「まさひろ→正浩」 и 「ひろし→浩」 не преобразовывались. Клавиатура телефона показывала только 12 вариантов, а движок обрезал список на 30. Теперь показываются все варианты, и каждое нажатие «Преобразовать» выбирает следующий. Проверено на настоящем телефоне и на сервере.

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
