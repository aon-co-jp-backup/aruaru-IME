# aruaru-IME PORTING / 移植・再開メモ  (Porting & resume notes)

## やさしい説明 / Plain words

**日本語**: このファイルは「別のパソコンや別の人が、続きから作業する」ための手順書です。順番は、(1)道具を入れる、(2)ソースを取って動かして試す、(3)最後にある「再開用メッセージ」を Claude に貼る、です。わからない言葉は README の「はじめに」を見てください。
**English**: This file is a guide for "continuing the work on another computer or by another person". The order is: (1) install the tools, (2) get the source, run it and test it, (3) paste the "resume message" at the bottom to Claude. If a word is new to you, see "What is this?" in the README.

## 引っ越し手順 / Moving to another machine (2026-10-13)

このリポジトリだけで開発を続けられます(`F:\runo` 側の索引リポジトリは、一覧への入口にすぎません)。
*This repository is self-sufficient. The `runo` index repository only links to it.*

### 1. 必要なもの / What you need

| 用途 / For | 内容 / What |
|---|---|
| Rust | stable(`cargo`)。Windows は `%USERPROFILE%\.cargo\bin` を PATH の先頭に / put `~/.cargo/bin` first on PATH |
| Windows 版のビルド | MSVC のビルドツール。インストーラーは Inno Setup 6(`tools\build-installer.ps1`) / MSVC build tools, Inno Setup 6 for the installer |
| Linux 版 | Windows からは WSL(Ubuntu)で。`gcc`・`libdbus-1-dev`・`pkg-config` |
| macOS 版 | Mac または GitHub の macOS ランナー。Windows からの確認は、zig を C コンパイラにした `cargo check --target aarch64-apple-darwin` まで(リンクは Apple SDK が要り、ライセンス上、Apple 製ハードウェア向け) |
| 辞書のもと | Mozc OSS(`data/mozc-src`)、JMdict、中国語ピンインの辞書は、初回に利用者の許可を得て自動取得 / downloaded on first use with consent |

### 2. 取得・ビルド・テスト / Get, build, test

```
git clone https://github.com/aon-co-jp/aruaru-IME.git && cd aruaru-IME
cargo test -p aruaru-ime-core      # コアのテスト(約180件)
cargo build -p aruaru-ime-cli -p aruaru-ime-daemon
```

- 辞書(`dict.bin`)は **git に入れていない**。`cargo run --release -p aruaru-ime-cli -- build-dict data/mozc-src data/dict.bin` で作る(README の「開発」参照)。
- 安全な実機確認: 別のデータ領域と別ポートで常駐エンジンを動かし、`POST /key` で打鍵する。本物の IME(ポート 4660)とデータには触れない。
  *Safe check: run the daemon with a separate data dir and port, and drive it with `POST /key`; it never touches your real IME (port 4660) or data.*

```
$env:ARUARU_DATA_DIR = "<空のフォルダに、%LOCALAPPDATA%\aruaru をコピー>"
$env:ARUARU_IME_PORT = "4699"
target\debug\aruaru-ime-daemon.exe
```

### 3. データの場所 / Where data lives

- ユーザーのデータ(利用者辞書・学習履歴・設定・取得済みの郵便番号・辞書): データ領域 `%LOCALAPPDATA%\aruaru`(環境変数 `ARUARU_DATA_DIR` で変更可)。**git には入らない**。
- 引っ越し先へ持っていく(2026-10-14、v0.2.14 予定): **`aruaru-ime move export [フォルダ]`** で 1 ファイル(`aruaru-ime-move-<日時>.json`)を書き出し、新しい PC で **`aruaru-ime move import <ファイル> [--replace]`** で取り込む(利用者辞書・学習履歴・設定・用語辞書・語彙・取得済みの郵便番号を含む。`--replace` なしは、いまの内容に足す統合。反映は常駐エンジンの再起動か再ログイン後)。辞書本体(`dict.bin`)・用語の検索辞書・中国語の辞書は大きく、移った先で自動取得されるので入れない。実データで書き出し→取り込みし、利用者辞書と郵便番号がバイト単位で一致することを確認(約7.6MB)。従来どおり、右クリック「バックアップと復元」(Google ドライブ)や、データ領域のフォルダのコピーでも移せる。
  *English*: `aruaru-ime move export [dir]` writes one file (`aruaru-ime-move-<time>.json`: user dictionary, learning history, settings, term dictionaries, vocab, downloaded postal data); on the new PC run `aruaru-ime move import <file> [--replace]` (merge by default; restart the engine afterwards). The big downloadable dictionaries are not included. Verified with real data (≈7.6 MB, byte-identical). The tray "Backup & restore" and copying the data folder still work.
- 人名辞書の表(`crates/aruaru-ime-core/pack_tables/people.tsv`)は git に入っている。再生成は `tools/build-people.ps1`(世界版) と `tools/build-people.ps1 -Japan`(日本の人物の追加)、続けて `aruaru-ime people-pinyin <people.tsv>`(中国語の拼音付け)。

### 4. 環境固有の落とし穴 / Environment pitfalls

- PowerShell 5.1 は BOM の無い UTF-8 を読み間違える。日本語を含む `.ps1` は **BOM 付き UTF-8**。`Get-Content`/`Set-Content` は ANSI 既定なので `-Encoding utf8` を付ける。
- `perl -e` に日本語を直接書くと失敗する(黙って何もしないことがある)。編集は Edit/Write ツールで。Bash のヒアドキュメントで `\u` や `\t` を書くと、環境によって実際の文字に変換される。
- `Remove-Item` が `F:\aruaru-IME` 配下でブロックされることがある(絶対パスで、`$env:TEMP` 配下を消す)。
- ureq は HTTP 304 をエラーにせず、本文の無い成功応答として返す(`dataupdate.rs` で対処済み)。
- Wikidata は英語ラベルを `mul` に移しつつある。クエリは英語を任意にして `mul` へ代替する。
- リポジトリ作成や検索を短時間に連打すると、GitHub・検索元の二次レート制限(403)で止まる。

### 5. 未了 / Open items (priority order)

1. 右クリックメニュー(辞書・登録を3区分に整理)と2列候補窓の実機の目視確認、インストーラーの実機の通し試験。
2. 多言語の切替入力(IME 本体への組み込み)・世界130言語。中国語の人名の拼音は逆引きで付与(多音字は誤る可能性があるため、利用者が気づいたら登録で補う)。
3. 台湾・ギリシャの郵便番号(GeoNames に無い)。ブラジルは市の代表コードのみ。オランダ(`1011AB` のように数字+英字)は未対応。
4. 用語辞書の検索→辞書化の実通し試験(検索元の休止明け)。
5. macOS / Linux / Android の実機確認(Android の WebView シェルは `open-backup` 側で別途)。

## アーキテクチャ / Architecture

```
 アプリ(TSF / IBus / IMK / Android)
   └ OS別の薄い層 ── キーを送る ──▶ 常駐エンジン(aruaru-ime-daemon, 127.0.0.1:4660)
        (aruaru-ime-tsf)               ├ POST /key {sid, key, before?, raw?} → Step(未確定・候補・確定・状態)
        ・キー判定は通信なし           ├ セッション(core::session::Session)をエンジン側で保持 ← アプリの再起動なしに更新できる
          (would_eat_with)             ├ 変換 = core::convert(モード予測 + 辞書Viterbi + 予測 + 検索サジェスト + 英訳)
        ・繋がらなければ DLL 内の       ├ 画面キャプチャ → OCR(aruaru-llm) → 画面の文字(文脈)
          簡易版で続行                 ├ 通知領域のアイコン・右クリックメニュー、バックアップ、更新、用語辞書の調査
                                       └ 共有データ領域 (%LOCALAPPDATA%\aruaru ほか)
```

OS 別の層が行うこと(他OSへ移植するときのチェックリスト):
1. キー(物理キー+修飾)を `Key` に変換し、`would_eat_with(key, on, composing, last_ja)`(`Step` が持ち帰る状態)で食う/食わないを決める。
2. `POST /key` を呼び、`Step.preedit`(未確定)・`Step.commit`(確定)・`Step.cands/sel`(日本語の候補)・`Step.cands_en/sel_en/col`(英語の列)・`Step.mode` を反映する。**反映に失敗したら `/session {op:"off"}` で IME をオフにして素通しにする。**
3. 組み立て開始時に、カーソル直前の文章を読んで `before` に載せる。先頭を Shift で打ったら `raw:true`。
4. 入力欄にフォーカスが入ったら `/focus {"editable":true,"password":false}` を数秒ごとに送る。パスワード欄では `password:true`。
5. 候補窓: 左=日本語、右=英語。←→で列、↑↓で選択、数字キー 1〜9・0 で直接選択(1ページ10件)。配色・倍率は `settings.json`。

## 共有データ領域 / Shared data dir

`%LOCALAPPDATA%\aruaru` / `~/.local/share/aruaru` / `~/Library/Application Support/aruaru`(`ARUARU_DATA_DIR` で上書き)。
`dict.bin`(かな漢字辞書)、`glossary.tsv`(JMdict の英訳)、`user_dict.json`(登録語)、`learned.json`(学習履歴)、`settings.json`、`wishlist.json`(希望用語)、`vocab/*.json`(言語・フレームワークの語彙)、
`words/<言語>.tsv`(多言語の語)、`components.json`(導入した部品の台帳)、`<部品>.env`(部品の起動設定)、`app/versions/<版>/`(自動アップデートの展開先)。

## 辞書とデータの出どころ / Data sources

- かな漢字: Mozc OSS(`google/mozc` の `src/data/dictionary_oss`、BSD-3-Clause)を `aruaru-ime build-dict <src> data/dict.bin` でコンパイル(約64MB、122万エントリ)。形式は `kanji.rs` 冒頭。
- 英訳: JMdict(`scriptin/jmdict-simplified` の `jmdict-eng-*.json.zip`、CC BY-SA 4.0)。エンジンの初回起動(とインストール時)に自動で取得し、`glossary.tsv` に変換。出典表示は `JMdict-LICENSE.txt`。
- 用語辞書・語彙: aruaru-llm の `/v1/search/raw`(→ aruaru-search)の抜粋から、「題名(よみ)」の組・繰り返し出るカタカナ名・API らしい識別子を抽出。

## 自動アップデートと「今すぐ更新」 / Update

`core::update`。起動時の1回だけ確認(定期確認なし)。右クリックの「今すぐ更新」で手動実行。
`<データ領域>/app/versions/<タグ>/` に展開し、新しいエンジンで入れ替える(旧エンジン終了後にポートを引き継ぐ)。**変換の状態はエンジン側にあるので、アプリの再起動は要らない。**
(DLL を変えるリリースのときだけ、起動中のアプリは再起動まで古い DLL を使う。プロトコルが互換な間は動く。)
リリース資産: `aruaru-ime-windows-x86_64.zip`(必須: `aruaru_ime_tsf.dll`・`aruaru-ime-daemon.exe`)+任意で `<zip名>.sha256`、およびインストーラー `aruaru-ime-windows-x86_64-setup.exe`。

## Windows(実装済み)

- `aruaru_ime_tsf.dll`(TSF): `ITfTextInputProcessorEx`+キー/スレッドマネージャ/コンポジションのシンク。登録は `regsvr32`(HKLM、管理者権限)=インストーラーが行う。
- 候補窓(`candwin.rs`): 2列、黒/白テーマ、倍率、右から左の描画。通知領域のメニューは `tray.rs`。
- 診断: `%TEMP%\aruaru-ime-log.on` を作ると `%TEMP%\aruaru-ime-tsf.log` に出力(`pid=` つき)。`GET /health` は `ok dict=<件数> v=<版> proto=1`。
- 未対応: 32bit DLL、入力スコープによるパスワード欄判定、タスクバーの言語表示(ITfLangBarItemButton)の右クリック、候補窓のクリック選択、言語プロファイルの追加登録(言語ごとの入力)。

## 未着手 / TODO

- **多言語の入力の組み込み**: `translit.rs` の規則(ロシア語・ウクライナ語・アラビア語・ペルシア語・ヘブライ語・韓国語・ヨーロッパ言語)を `Session` に載せ、言語ごとの TSF プロファイル(LANGID・GUID は `Lang::langid()/profile_guid()`)を登録して Windows の言語切替で選べるようにする。世界130言語の台帳と、右クリックでの最大7言語の選択(`Settings::toggle_language` は実装済み)。
- 中国語: ピンイン→漢字の辞書(ダウンロード許可が必要。簡体字・繁体字)。
- GitHub への選択式バックアップ: 利用者が用意するトークンで、公開/非公開のリポジトリを選ぶ(公開では学習履歴を含めない)。
- macOS: InputMethodKit、Linux: IBus / fcitx5、Android: `InputMethodService`。いずれも `POST /key` を呼ぶ薄い層にする(エンジンの HTTP API はそのまま使える)。
- aruaru-search の同梱導入(「無制限の検索」を手元で完結させる)。Qwen の高速化(open-cpu/open-cuda 側)と、`score_continuations` を使った再ランク。

## 試験 / Testing

- `cargo test`: コア 127 件+TSF 1 件。辞書(`data/dict.bin`)・JMdict(`data/jmdict-src`)がある時だけ走る実データの試験を含む。
- エンジンの API を直接: `POST /key` に `{"sid":"x","key":{"Char":"k"}}` を順に送り、`preedit`/`cands`/`cands_en` を見る(画面に触れない)。
- 実機の打鍵: `tools/ime-e2e.ps1`(**無人で**)。バックアップ: `aruaru-ime backup`(Google ドライブが見つかれば `G:\マイドライブ\aruaru-IME-backup`)。
- 検索の通し試験: `ssh -L 4601:127.0.0.1:4600 conoha` のトンネルを張り、`ARUARU_LLM_URL=http://127.0.0.1:4601 aruaru-ime wish add manga キメツ`(検索元が休止中だと失敗するので、連続して試さない)。

---

## English

**Architecture.** The Windows text service (TSF DLL) is a thin client: it maps keys to `Key`, decides locally whether to eat a key from the state carried back in each `Step` (`would_eat_with`), and calls the resident engine (`aruaru-ime-daemon`, `127.0.0.1:4660`, `POST /key`). The engine owns the `Session` state machine, so an engine swap updates behaviour **without restarting apps**; if the engine is unreachable the DLL falls back to a dictionary-less local session so typing never stops.

**Porting checklist.** (1) map keys → `Key`; decide eaten with `would_eat_with`; (2) call `/key`, apply `preedit`/`commit`/`cands`/`cands_en`(+`col`,`sel_en`)/`mode`; on any failure turn the IME off (`/session {op:"off"}`); (3) send the text before the caret as `before` when a composition starts, `raw:true` for Shift-started words; (4) post `/focus` every few seconds in editable fields (`password:true` in password fields); (5) candidate window: Japanese left, English right, ←/→ switch column, ↑/↓ select, digits 1–9/0 pick (10 per page).

**Data.** Kana-kanji: Mozc OSS (BSD-3-Clause) compiled to `dict.bin`. English glosses: JMdict via jmdict-simplified (CC BY-SA 4.0), downloaded automatically on first start/install into `glossary.tsv` with `JMdict-LICENSE.txt`. Term dictionaries and framework vocabularies are extracted from search snippets (aruaru-search through aruaru-llm).

**Update.** Checked once at startup, or on demand from the tray menu; staged in `<data>/app/versions/<tag>/`; the new engine takes over the port. Release assets: `aruaru-ime-windows-x86_64.zip` (+ optional `.sha256`) and the installer.

**TODO.** In-IME multilingual input (rules exist in `translit.rs`; TSF profiles per language), Chinese (needs a pinyin dictionary download approval), 130-language registry, GitHub backup (user-supplied token; public repos must exclude learning history), taskbar-indicator right-click (ITfLangBarItemButton), macOS/Linux/Android front ends, bundling aruaru-search, Qwen speed-up.

## 中国語ピンイン入力 / Chinese pinyin input (2026-10-11)
- 日本語: 簡体字・繁体字のピンイン入力に対応。辞書 `zh.tsv`(jieba頻度・pinyin-data読み・OpenCC繁体化、MIT/Apache-2.0、`ZH-LICENSES.txt`)を初回起動時に自動取得して作成。候補は数字キー/Space/Enterで確定、入力の一部だけ使う候補は残りを続けて変換(部分確定)。
- English: Simplified/Traditional pinyin input. `zh.tsv` is built automatically on first run from jieba, pinyin-data and OpenCC. Digits/Space/Enter commit; partial-length candidates keep the rest of the input.

## 世界130言語の一覧 / 132-language registry (2026-10-11)
- `core/src/languages.rs`: 132言語の一覧(出典 realdata.pro)。入力エンジンがある15言語は「対応」、他は「準備中」。`aruaru-ime languages all` で表示。残: 準備中言語のエンジン、トレイの全言語一覧UI。
- 132 languages listed; 15 have input engines, the rest are "planned". Remaining: engines for the rest and a full-list tray UI.

## 季節用語辞書とGitHubバックアップ / Seasonal dictionary & GitHub backup (2026-10-11)
- `core/src/seasonal.rs`: 季節用語(季語・二十四節気・行事・花)と言い回し(花が咲く/雪が降る/季節+には…)を組み込み、Viterbiに1語として注入(`kanji.rs build_extra`)。「春には菜の花が咲き」「秋には紅葉が色づき」を正しく変換。
- `core/src/ghbackup.rs`: GitHub(Contents API)へバックアップ/復元。トークンは利用者が `%LOCALAPPDATA%\aruaru\github-token.txt` に置く(設定・ログに出さない)。公開リポジトリは履歴(learned.json)と設定を自動で除外。トレイ「バックアップと復元」から設定/実行。
- Seasonal terms and phrases are injected into the Viterbi lattice; GitHub backup via Contents API with a user-supplied token file, auto-excluding history/settings for public repos.

## パスワード欄検知と知識ベース / Password-field detection & knowledge base (2026-10-11)
- TSF DLL: フォーカス中のEdit欄が `ES_PASSWORD` なら `password:true` をデーモンへ通知(取り込み・学習を止める)。Win32 Edit のみ。ブラウザ等の独自描画欄は未対応。
- `core/src/knowledge.rs`: open-english と同形式の `knowledge.json`。`aruaru-ime knowledge add|search|import`。候補ウィンドウへの補足表示、aruaru-llm連携、自動学習は未実装。
- 残り(未実装): 準備中言語エンジン/全言語選択UI、タスクバー右クリック、32bit DLL、macOS/Linux/Android、インストーラー実機試験(UAC要)。

## 共有フロントエンドの自動生成 / Shared front end auto-generation (2026-10-11)
- RPoemに新クレート `open-runo-crossui`(Rust)を追加。フロントエンドを1回(`open-runo-view`のVNode+エンジンとの橋渡し)書けば、Windows/macOS/Linux/Android/Web向けに共通ページ(SSR)と各OSの起動・配布記述を一括生成する。
- aruaru-IME: 常駐エンジンの `GET /ui` が同じ画面を配り、`aruaru-ime frontends [出力先]` が全OS分を `dist/frontends` に生成。`.github/workflows/build.yml` が3OSでエンジン/CLIをビルドし、フロントエンド一式を成果物にする(RPoemを兄弟ディレクトリにcloneして使う)。
- 残: macOS/Linuxの実機検証、AndroidのWebViewシェル(アプリ化)、OS固有のIME統合(macOS InputMethodKit・Linux IBus/Fcitx・Android InputMethodService)はまだ無く、現状は常駐エンジン+Web画面経由で使う。
- RPoem `open-runo-crossui` generates the shared page and per-OS launchers from one definition; aruaru-IME serves it at `/ui` and emits all targets with `aruaru-ime frontends`. OS-native IME integration is still to do.

## open-easy-web からのオンライン利用 / Online use from open-easy-web (2026-10-11)
- `https://easy-web.tokyo/ime`(open-easy-web サーバーの `GET /ime`)が、RPoem `open-runo-crossui` の同じ画面を配る。入力は利用者PCの aruaru-IME エンジン(127.0.0.1:4660)へ直接つなぐ(入力内容はサーバーを通らない)。
- エンジンは Origin が `https://easy-web.tokyo`(と自分自身)のときだけ CORS + Private Network Access を許可。それ以外は403。
- 未了: VPS(open-easy-web)への反映は未実施。ブラウザ(Chrome/Edge/Firefox)からのloopback接続の実機確認も未了。
- `/ime` on open-easy-web serves the shared page; keystrokes go straight to the user's local engine, which allows only easy-web.tokyo via CORS/PNA. VPS deploy pending.

## 起動時のオンライン画面 / Online screen at startup (2026-10-11)
- 設定 `open_online_ui`(既定オン、トレイで切替): エンジン起動の20秒後、easy-web.tokyo に届くときだけ `https://easy-web.tokyo/ime` を既定ブラウザで最小化して開く(Windows)。ブラウザによっては最小化を無視する。スマホ/タブレットは対応アプリが無く未対応。
- Opens the online screen minimized ~20 s after engine start when easy-web.tokyo is reachable (Windows; some browsers ignore minimize). Phones/tablets: not supported yet.

## 「〜療法」「気功」など健康用語 / Health & therapy terms (2026-10-11)
- `seasonal.rs` に `THERAPY_PREFIX`(気功・音楽・睡眠・アロマ…+療法)と `TERMS` を追加。辞書に無い語(気功)も名詞として扱い、「きこうりょうほう」→気功療法、「すいみんりょうほう」→睡眠療法 に変換。

## 最後に選んだIMEの自動選択 / Re-select the last IME after reboot (2026-10-11)
- TSF DLL が、選ばれた入力方式を `%LOCALAPPDATA%\aruaru\last-ime.txt` に記録(`aruaru <プロファイルGUID>` / `other`)。常駐エンジンは起動8秒後、最後が aruaru-IME なら `ActivateProfile`(現在のセッション)と `Set-WinDefaultInputMethodOverride`(次回サインイン以降の既定)で選び直す。ほかのIMEが最後なら何もしない。実機で既定の入力方式が設定されることを確認。
- The DLL records the last selected input method; at startup the engine re-activates aruaru-IME and sets it as the default override. If another IME was last, it does nothing.

## 再起動後の自動選択のBUG修正ほか / Reboot auto-select fix, etc. (2026-10-12)
- 原因: エンジンがサインイン時に自動起動しておらず(開発機にRunキー無し)、再選択が走らなかった。エンジン起動時にHKLM/HKCUのRunキーを確認し、無ければHKCUに自己登録(`ensure_autostart`)。再選択後に `restore-ime.log` へ「aruaru-IME active afterwards」を記録。手動の再選択で active=true を確認。2026-10-12 PC再起動後に aruaru-IME が既定になることをユーザーが実機で確認済み。
- 無変換キー: 最初の1回は必ずひらがな(変換中の漢字も読みに戻す)、以後 カタカナ→全角英数→半角英数→ひらがな。
- 「あきのこうよう」→「秋の紅葉」: 季節+の+季節語を言い回しに追加、辞書に無い季節語のコストを下げた。

## 候補の優先順位・記号・開発スタック (2026-10-12)
- `prefer.rs`: 読みごとの優先順位(組み込み + 利用者の `%LOCALAPPDATA%\aruaru\prefer.tsv`、`読み<TAB>表記,表記`)。学習履歴より優先。きゅう→急,休暇,級,休息,休息時間,急速,球速 / たいおう→対応,太王四神記,大王,滞欧 / でない→出ない。
- 記号: 最初に `-` `+` を打つと、半角・全角・ダッシュ・マイナス・長音・半角カナの長音(ｰ)/プラス・カタカナ「プラス」などを候補に並べる(直前が英数字のとき、数字が続くときは半角のまま)。「はいふん」「ぷらす」読みでも同じ一覧。
- 「そのような→その様な」(こ/そ/あ/どの+様な/様に…)、「じゅんい」は junni で 順に に加えて 順位 も2番目に出す、「9足」など数字入りの言い換えは後ろへ。
- 右クリック: 人名の登録(姓・名・フルネーム)、開発で使う言語・フレームワークの選択(`dev_stack`。選択にないものは入力フォーム→検索→学習し自動で選択に追加)。
- オンライン画面(/ime): ダウンロード(GitHub Releases)と「このPCの画面で開く」のリンク、接続できないときの案内とエラー表示を追加。

## オンライン画面のサーバー変換 / Server-side conversion for the online screen (2026-10-12)
- ブラウザがローカルネットワークへの通信を止める問題を避けるため、`https://easy-web.tokyo/ime` は VPS のエンジンで変換する(入力はサーバーを通る。保存・学習はしない。画面にも明記)。
- VPS: `/root/repository/aruaru-IME`(clone・`cargo build --release -p aruaru-ime-daemon`)→ `/opt/aruaru-ime/aruaru-ime-daemon` + `/var/lib/aruaru-ime/dict.bin`、`aruaru-ime-server.service`(`ARUARU_IME_SERVER=1`: 学習・利用者辞書・自動更新・バックアップを止める。127.0.0.1:4660のみ)。更新: git pull → build → バイナリをコピー → `systemctl restart aruaru-ime-server`。
- open-easy-web: `/ime/api/{health,key,session}` だけをエンジンへ中継(本文4KBまで)。Chrome 等の実ブラウザでの確認は未了(内蔵ブラウザでは候補表示を確認)。
- 利用者PC側のエンジンへ直接つなぐ従来の経路は、このページでは使わない(PCのエンジンは `http://127.0.0.1:4660/ui`)。

## v0.2.0 公開とインストーラー試験 / v0.2.0 release & installer test (2026-10-12)
- v0.2.0 を GitHub Releases に公開(プレリリース)。同じファイルを `https://easy-web.tokyo/ime/download/aruaru-ime-windows-x86_64-setup.exe`(VPS `/var/lib/aruaru-ime/downloads/`)からも直接配布(オンライン画面の「ダウンロードはこちら」)。新版を出したら、このフォルダへ scp して入れ替える。
- インストーラー実機試験(`tools/installer-e2e.ps1`、管理者): 無人インストール exit=0 → 無人アンインストール exit=0 → 直後の再インストールは exit=8(アンインストールで DLL の削除が再起動待ちになるため。Windows の仕様)。**再起動後の再インストールの確認は未了**。試験後は `tools/reregister-dev.ps1`(管理者)で開発版を再登録。
- Installer test: silent install OK, silent uninstall OK; reinstall right after uninstall needs a reboot (pending DLL delete). Post-reboot reinstall check still pending.
- 右クリック: 知識ベース(追加・一覧)、世界132言語の一覧(対応/準備中)を追加。
- 未了: 準備中言語(約117)の入力エンジン、知識ベースの候補窓への表示、macOS/Linux/Android のIME本体。

## v0.2.1 (2026-10-12): 用語辞書パック・記号候補・言語の近似対応
- 組み込みの用語辞書(パック): 鬼滅の刃・ワンピース・セーラームーン・ドラゴンボール・UFOロボ グレンダイザー(フランス語版 Goldorak・ゴルドラック含む)・マジンガーZ・グレートマジンガー・大空魔竜ガイキング・鋼鉄ジーグ(`packs.rs`)。日本語の読み・表記に、英語・ヨーロッパ各国での呼び名(Goldorak/Goldrake/Actarus 等)をローマ字の読みで併記。**網羅ではなく主要な語**。既定で自動インストール(一度入れた/外したものは二度と自動では入れない `packs_seen`)。右クリック「用語辞書を選んで入れる」で入れる/外す。
- 記号: 最初(または日本語・記号の直後)に `/ < > ? ! [ ] ( ) ~ * = など` を打つと種類を候補に並べ、先頭を表示(`/` → ・ ／ / ･ …)。「なかてん」「かっこ」「まる」「さんかく」「しかく」「ほし」「やじるし」読みでも一覧。
- 言語: キリル文字系(ブルガリア語等)=ロシア語、アラビア文字系(ウルドゥー語等)=アラビア語、ラテン文字系(約75言語)=アクセント記号つき入力で**近似**対応。右クリック「すべての言語」で選べる(最大7)。インド系・タイ・ジョージア・アルメニア・ギリシャ等(約27言語)は準備中。

## v0.2.2 (2026-10-12): 他OSのインストーラー・Android・二重起動の修正・オンライン画面の入力
- **macOS / Linux**: `installer/unix/install.sh`(ユーザー権限。エンジンとCLIを `~/.local/share/aruaru-ime` に置き、辞書(約64MB)を `https://easy-web.tokyo/ime/download/dict.bin` から取得、macOS=LaunchAgent / Linux=systemd --user(なければ XDG autostart)で自動起動、`http://127.0.0.1:4660/ui` を開く)と `uninstall.sh`。CIが tar.gz を作り、`v*` タグで GitHub Releases へ添付。**OS全体のIME(macOS InputMethodKit・Linux IBus/Fcitx)は未実装**で、`/ui` の画面で入力する。
- **Android**: `android/`(WebViewのシェル、`https://easy-web.tokyo/ime` を開く。CIが `gradle assembleDebug` で APK を作る)。OS全体のキーボード(InputMethodService)は未実装。
- **試験**: CI(`.github/workflows/build.yml`)が Windows / macOS(arm64, intel) / Linux でコアのテスト+エンジンのスモーク試験(`tools/e2e-smoke.sh`)を実行(いずれも成功)。手元では WSL の Ubuntu で CI 成果物のスモーク試験・install.sh・uninstall.sh を通した(エンジン起動・辞書取得・systemd --user 自動起動・削除)。macOS / Android の実機/実端末での試験は未実施。
- **二重起動**: 同じ版のエンジンが動いているとき、後から起動したものは何も表示せず終了(`port busy ... retrying` の連続表示を解消)。常駐エンジンはコンソール画面を出さない(Windows)。
- **オンライン画面**: OSのIMEが有効なとき(`Process` / 229)と、スマホの画面キーボード(beforeinput / compositionend)でも入力できるようにした。トップページ(https://easy-web.tokyo/)に紹介とダウンロードを追加、`/ime` に OS 別インストーラー一覧(利用中のOSを強調)。

## v0.2.3 Android: キーボード(IME)・実機試験 (2026-10-12)
- `android/` に **InputMethodService のキーボード**(`AruaruImeService.java`: ローマ字キー → サーバー `https://easy-web.tokyo/ime/api` で変換 → `setComposingText` / `commitText`、候補ストリップのタップで確定(`/session op=choose`)、あ/A 切替、`, .` は 、。、`/ +` など記号の候補)。通信できないときは打った文字をそのまま入力。入力はサーバーに送られるが保存・学習しない。
- 固定の署名鍵 `android/aruaru-alpha.keystore`(**アルファ版の検証用の鍵。公開リポジトリに置いている**。正式版では非公開の鍵へ)。CIが毎回別の鍵で署名していて上書き更新できなかったため。
- **実機(moto g53y 5G)**: APKを入れ → `adb shell ime enable/set jp.aon.aruaruime/.AruaruImeService` → キーボードが出る → `taiou` で候補 対応/太王四神記/大王/滞欧 → 候補タップで確定、まで確認。試験後は端末の既定キーボードを元(Gboard)に戻した。通常は、設定の「キーボード」でオンにし、キーボード切替(画面右下のアイコン)で選ぶ。
- CI(`android-test`)は、エミュレーター(スマホ=pixel_6、タブレット=pixel_c)でAPKを入れて起動し、画面の文字を確認(縦横)。タブレットは黒画面になるため、ソフトウェア描画(`-gpu swiftshader_indirect`)で再試験中。

## 言語エンジン15言語を追加 / 15 more language engines (2026-10-12)
- `indic.rs`: インド系9文字(ヒンディー・マラーティー・ネパール=デーヴァナーガリー、ベンガル、アッサム、パンジャブ、グジャラート、オリヤー、タミル、テルグ、カンナダ、マラヤーラム)のITRANS風ローマ字入力(Unicodeの配置が共通なので1つの変換器)と、ギリシャ・ジョージア・アルメニアの表。`translit.rs` の `Lang` に15言語を追加(TSFのプロファイルも各言語に登録される)。準備中は th lo km my si am ti bo dz dv の10言語。
- 再起動後の再インストール試験: `tools/schedule-install-after-reboot.ps1` で「ログオン時に installer-e2e.ps1 -Step install を実行」するタスク `aruaru-ime-e2e` を登録済み。再起動後、`dist/installer-e2e.log` に結果が出る(exit=0 なら成功、8なら再起動待ちが残っている)。
- 未了: 知識ベースの候補窓表示、macOS/Linux の OS 全体の IME、th lo km my si am ti bo dz dv の入力エンジン。

## 再起動をまたぐ再インストール試験 (2026-10-13)
- アンインストール → **再起動**(シャットダウンではなく。高速スタートアップでは保留中のDLL削除が消えず `exit=8` のまま) → インストール、で **exit=0**、`C:\Program Files\aruaru-IME` に導入・HKLM Runキー・エンジン起動(v0.2.3)を確認。`tools/installer-e2e.ps1 -Step install|uninstall|status` と `tools/schedule-install-after-reboot.ps1`。
- 用語辞書パックに 般若心経・ハンターハンター を追加(既定インストール)。グレンダイザー・マジンガー・ガイキング・ジーグに欧州版の呼び名(Goldorak/Goldrake/Mazinga/Jeeg Robot など)を追加。**主要な語であり網羅ではない**(利用者は `prefer.tsv`・用語辞書の追加で足せる)。

## 用語辞書パックの多言語版 (2026-10-13)
- `packs_i18n.rs`: 各パック(鬼滅の刃・ワンピース・セーラームーン・ドラゴンボール・グレンダイザー・マジンガーZ・グレートマジンガー・ガイキング・ジーグ・ハンターハンター・3X3 EYES・般若心経)に、**中国語(簡体・繁体=台湾)・韓国語・ヨーロッパ言語(es fr de it pt)**の版を追加。パックを入れると `zh-extra/<パック>.tsv`(ピンイン→簡体/繁体)・`words/ko.<パック>.tsv`(2ボル式キー→ハングル、キーはハングルから自動生成=`hangul_to_keys`)・`words/<言語>.<パック>.tsv` が書かれ、各言語の入力でも候補に出る(`longzhu`→龙珠/龍珠、`dnjsvltm`→원피스、`goldrake`→Goldrake、`coeur`→Sûtra du Cœur)。
- **主要な語・公式の訳が定まっている語だけ**。網羅ではない(グレンダイザー・ジーグ等の中国語名は未収録)。版(`PACKS_REV`)が上がると、入れたままのパックだけ入れ直す(外したものは戻さない)。
- 試験用に `ARUARU_IME_PORT` で待受ポートを変えられる(IME本体は4660)。管理者権限で起動したエンジンは通常権限から止められない点に注意(再ログオンで通常権限の起動に戻る)。

## 残り10言語のエンジン・知識ベースの候補窓表示 (2026-10-13)
- `abugida.rs`: タイ・ラオス・クメール・ミャンマー・シンハラ・チベット/ゾンカ・ターナ(ディベヒ、右から左)・エチオピア文字(アムハラ・ティグリニャ)のローマ字入力(子音+母音の簡易な表方式。チベット文字の積み重ね・タイの声調記号などは扱わない=**近似**。ネイティブ検証は未了)。`Lang` が全132言語中の130言語に専用エンジンを持つ。
- 知識ベース(`knowledge.json`)に、選んでいる候補の語を含む知識があれば、候補ウィンドウのモード表示の横に `※ 〜` と1件を出す(`Step.note`、`knowledge::note_for`)。Windowsの候補窓の目視確認は未了(コードとエンジン側の確認まで)。

## 花・国・都市・駅の辞書、郵便番号 → 住所 (2026-10-13)
- `data/packs/{flowers,countries,cities,stations}.tsv`(`よみ 日本語 English 簡体 繁体 ピンイン 한글 fr es de it pt`)から `packs_table.rs` が用語辞書パックを作る(既定でインストール。日本語の読み・表記 + 英語(小文字のつづり→大文字小文字つき)・中国語(簡体・繁体)・韓国語・ヨーロッパ言語の版)。花45・国68・都市65・駅約130(駅名は「〜駅」も)。**主要なものだけ**で、全駅(約9,000)・全都市の網羅ではない。全駅は、公開データ(駅データ.jp 等)の取得を許可してもらえれば取り込める。
- **数字の入力**(Windows含む全エンジン): 数字は未確定になり(英数字の直後 abc1・3.14 を除く)、Enter でそのまま確定、Space で 全角・漢数字・(7桁/123-4567なら)住所と〒書式 の候補。英字が来たら数字を確定してから続ける。
- **郵便番号 → 住所** (`postal.rs`): 日本郵便の公開データ(`utf_ken_all.zip`)から `postal.tsv` を作って引く。**自動では取得しない**(利用者が右クリックの「郵便番号の住所辞書を取得」か `aruaru-ime postal-install` を選んだときだけ)。取得前は住所の候補が出ない(〒書式の候補は出る)。

## 世界の郵便番号 → 国名つき住所(2026-10-13)

- `aruaru-ime-core/src/geo.rs`。数字で始まる郵便番号を打つと、「国名 住所」の候補が出る。`-`・空白は有っても無くても同じ(正規化して照合)。
- 国の並び順: IPアドレスから推定した国 → 選択中の言語の国(ja→JP、en→US/GB/CA/AU/NZ/IE、fr→FR/BE/CH/CA …) → その他。国名は言語に合わせて表示(日・英・中・韓)。
- 日本は日本郵便の漢字データ(取得済みなら)を優先、無ければ GeoNames(ローマ字)。
- データは、右クリック「世界の郵便番号データを取得」または `aruaru-ime geo-install` のときだけ取得(GeoNames, CC BY 4.0、44か国、約90秒)。
  同時に、IPから国を返す公開の窓口(Cloudflare `/cdn-cgi/trace`)へ**1回だけ**問い合わせて国を保存する(入力中の通信は無い)。当初案のGeoIP DBではなく、同じ露出で通信量が少ないこの方法にした。
- 動作確認: `aruaru-ime geo <郵便番号> [言語]`。実データで 100-0001/1000001/10001(ja/en)/75001(fr)/110-0001(ko) を確認。
- 制限: 英字を含む郵便番号(英国 SW1A 1AA、カナダ K1A 0B1)は、組み立て中が数字だけの入力にしか反応しないため未対応。GeoNames に無い国(台湾・ギリシャ)は無し。
  ブラジルは GeoNames が市の代表コードのみ(01310-100 は出ない)。IME のキー入力経由(Session)での通し確認は未実施(候補関数の単体と実データのみ)。
- 出典表示: GeoNames (https://www.geonames.org/) CC BY 4.0。

## 取り込んだ公開データ (2026-10-13)
- **郵便番号**: 取得先は `https://www.post.japanpost.jp/service/search/zipcode/download/utf/zip/utf_ken_all.zip`(旧 `/zipcode/dl/utf/zip/` は404)。この PC に 120,720 件を取り込み、`1000001`→東京都千代田区千代田 を確認。
- **全国の駅名**: OpenStreetMap(Overpass API、ODbL)から 8,086 駅を取り込み(読みが無い駅は英語名のヘボン式から作る=長音で誤ることがある)。右クリックの「全国の駅名を取り込む」/ `aruaru-ime stations-install`。出典の表示が必要(`installer/LICENSES.txt`に記載)。

## 自動アップデートの設定を更新 (2026-10-13)
- **不具合の修正**: 自動アップデートは `/releases/latest`(プレリリースを返さない)を使い、プレリリースを選ばない作りだったため、**アルファ版(全リリースがプレリリース)は一度も自動更新の対象になっていなかった**。一覧(`/releases?per_page=15`)から、現在より新しい最大の版を選ぶ(`pick_newest`)ように変更。
- **更新チャンネル** `update_channel`(既定 `alpha`=プレリリースも対象 / `stable`=正式版だけ)。右クリックの「アルファ版も自動更新の対象にする」で切替。
- 仕組みは従来どおり: 起動時の1回だけ確認 → `aruaru-ime-windows-x86_64.zip` を取得(`.sha256` で検証、取得先は GitHub のみ) → 版ごとのフォルダに展開してエンジンを入れ替え(アプリの再起動不要)。macOS / Linux は tar.gz のため自動更新の対象外(`install.sh` を再実行)。

## 人名・有名人名・重要人物名の辞書(2026-10-13、既定で搭載)

- `crates/aruaru-ime-core/pack_tables/people.tsv`(約3300人)を、既存の「表から作る用語辞書」(`packs_table.rs`)の `people` として登録。他の組み込み辞書と同じく既定でインストールされる。
- データの出所: Wikidata(CC0)。`tools/build-people.ps1` で再生成できる(職業ごとに、リンクされている言語版の数が多い人物 → 世界版 `sl>=90`、日本版 `-Japan`: 国籍または出生地が日本で `sl>=25`、職業を問わない枠を含む)。
- 日本語の読み: 表記がかな/カタカナだけならそこから、漢字を含む場合は Wikidata のかな表記(P1814)、それが無い日本人は英語表記(ヘボン式)から作る(表の読みが `=` で始まる行。IME側の `romaji` で変換、変換しきれないものは入れない=145人)。
  ラテン文字を含む表記(ジョージ・W・ブッシュ など)は読みが定まらないため除く。
- 英語名は、小文字のつづり(スペースなし)で打つと大文字小文字つきの名前になる(`isaacnewton` → Isaac Newton)。韓国語表記(한글)・fr/es/de/it/pt の表記も入る。簡体・繁体は、ピンインが無いので入れていない。
- 確認: 織田信長・夏目漱石・徳川家康・宮崎駿・黒澤明・イチロー・スティーブ・ジョブズ・アインシュタイン・ニュートン・ナポレオンは入った。北野武・モーツァルトは、日本語表記が別の形のため入っていない。
- 注意: Wikidata は英語ラベルを `mul`(多言語)に移しつつあるので、クエリは英語を任意にして `mul` へも代替している。PowerShell 5.1 は BOM の無い UTF-8 のスクリプトを文字化けするので、`build-people.ps1` は BOM つき。
- 出典表示: Wikidata (https://www.wikidata.org/) CC0。

## 起動の高速化(2026-10-13)

- 人名辞書(約3300件)の追加で、初回起動(パックの入れ直し)が **約4分** かかっていた(246秒)。原因は、パックの項目を1件ずつ `add_from` で入れるたびに、利用者辞書(約1.4MB)全体を書き直していたこと(O(n²))。
- `packs::install` を `UserDict::add_many`(保存は最後に1回)に変更。読みの決め方は `reading_for` に共通化して、1件ずつの登録と同じ結果。**246秒 → 8秒**(デバッグビルド、別ポート4699・データのコピーで測定)。
- 通し確認(HTTP `/key` で打鍵): `100-0001`・`1000001` → 日本の住所が先頭(IPの国=日本)、`10001` → 米国、`75001` → フランスも候補、`odanobunaga`+Space → 織田信長、`natsumesouseki`+Space → 夏目漱石。

## 長音の読み(2026-10-13)

- 表の辞書(人名・国名・都市名など)の読みに長音「ー」があるとき(にゅーとん)、母音に直した読み(にゅうとん。お段は「う」)も同じ表記で登録する(`packs_table::long_vowel_variant`)。
  ローマ字で `nyuuton` と打つ人向け(`nyu-ton` はそのまま)。確認: `aizakkunyuuton` → アイザック・ニュートン、`nyuujiirando` → ニュージーランド(先頭)。
- 既存の利用者にも反映するため、パックの版(`PACKS_REV`)を r4 → r5 に上げた(入れたままのパックだけ入れ直す)。

## Linux の IBus エンジン・macOS の入力メソッド・メニュー整理 (2026-10-13, v0.2.10)
- **Linux IBus** (`crates/aruaru-ime-ibus`、Rust + zbus 4): IBus の `org.freedesktop.IBus.Factory/Engine` を実装し、常駐エンジン(HTTP)にキーを送って、確定(`CommitText`)・未確定(`UpdatePreeditText`)・候補(`UpdateLookupTable`、`IBusLookupTable`=`(sa{sv}uubbiavav)`)を IBus に返す。`tools/ibus-test.sh`(ibus-daemon + python gi の入力コンテキストで `konnichiha`→今日は→Enterで確定)を、**WSL の Ubuntu と CI(ubuntu)で実行して成功**。`install.sh` が `~/.local/share/ibus/component/aruaru-ime.xml` を書いて `ibus restart`。Fcitx は未対応。落とし穴: `ibus-daemon` は `--daemonize` で起動しないと親の終了で止まる、`IBusLookupTable` は labels(`av`)まで必要(足りないと ibus-daemon が落ちる)。
- **macOS** (`macos/main.swift` + `Info.plist` + `build.sh`、InputMethodKit): `IMKInputController` がキーをエンジンへ送り、`insertText` / `setMarkedText` / `IMKCandidates` で反映。**CI(macos-latest / macos-15-intel)でビルドは成功、実機での入力試験は未実施**(`install.sh` が `~/Library/Input Methods` へ入れる。ログアウト・ログイン後に入力ソースへ追加)。落とし穴: `IMKInputController.server()` と名前が衝突するのでグローバルは `imkServer`。署名は ad-hoc(Gatekeeper の警告が出ることがある)。
- **メニュー整理**: 通知領域の右クリックを 開発 / 辞書・登録 / 設定 / バックアップ にまとめた。初回起動(設定ファイルが無い)で使い方を案内。**目視確認は未了**(この PC のエンジンが管理者権限で動いており入れ替えられないため)。
- CI の `android-test`(エミュレーター)は起動が不安定なため `continue-on-error`(失敗してもリリースは止めない)。直近2回は成果物なしで失敗しており、原因は未調査。

## 毎朝の自動データ更新 / Automatic morning data refresh (2026-10-13)

- `aruaru-ime-core/src/dataupdate.rs`、常駐エンジンの `spawn_data_update`。設定 `data_update`(既定オン。右クリック「設定」でオフ可)。手動実行は `aruaru-ime data-update`。
  *`dataupdate.rs` + `spawn_data_update` in the daemon; setting `data_update` (on by default; toggle under tray → Settings); run once by hand with `aruaru-ime data-update`.*
- **時刻 / When**: 端末のローカル時刻の朝6時から、データの置き場所とユーザー名から決まる 0〜120 分のずれを足した時刻以降。その日にまだ実行していなければ、起動が遅れてもその日のうちに1回。30分ごとに確認。世界中の端末が同じ瞬間にサーバーへ集中しないための工夫。
  *From 06:00 local time plus a per-machine 0–120 min offset, once per local day (a late start still runs it that day); checked every 30 min. The offset keeps clients from hitting the servers at the same moment.*
- **何を更新するか / What**: (1) **取得済みの**世界の郵便番号(GeoNames)と日本郵便のデータだけ。`ETag` / `Last-Modified` の条件付き取得で、変更が無ければダウンロードしない(実測: 2回目以降は44か国すべて「変更なし」)。(2) 現在地の国(世界の郵便番号を取得済みの人だけ。移動に追従)。(3) 国名(約170)・人口100万以上の都市名(約200)を Wikidata から取り、利用者辞書へ出所つき(`自動更新:国名`、`自動更新:都市名`)で入れ直す(読みが分かる=カタカナ・ひらがな表記のものだけ。結果が少なすぎるときは更新しない)。
  *(1) Only the postal data you already downloaded (GeoNames, Japan Post), by conditional GET — nothing is re-downloaded if unchanged (measured: all 44 countries "not modified" on later runs). (2) Your current country (only if you fetched world postal data). (3) ~170 country names and ~200 cities with 1M+ people from Wikidata, stored in your user dictionary with a source tag; only labels with a known reading (kana) are added, and the update is skipped if the result is suspiciously small.*
- **自動で追加しないもの / Never added automatically**: 選んでいない郵便番号の国・辞書。(国名・都市名だけが、ユーザー指示により自動で入る。)
- エラーがあれば、その日の分を完了としない(次の30分後に再試行)。取得に失敗しても、既存のデータは壊さない(解析に成功してから書く)。
- 試験: 単体 3 件(更新時刻・読みの作り方・SPARQL の読み取り)+実データ(1回目 165秒で全更新、2回目以降は条件付き取得)。

## 中国語の人名のピンイン / Pinyin for Chinese person names (2026-10-13)

- 人名辞書の表(`people.tsv`)に、Wikidata の簡体・繁体の表記を入れ、**ピンインは中国語辞書(`zh.tsv`)の逆引きで付ける**: `zh::PinyinRev`(漢字 → 頻度が最も高いピンイン、最長一致、中黒は読み飛ばし、辞書に無い字を含むと付けない)。`aruaru-ime people-pinyin <people.tsv>`。
  3291人のうち 3230人に付いた(例: 爱因斯坦 → `aiyinsitan`、艾萨克·牛顿 → `aisakeniudun`、孔子 → `kongzi`)。
  *Simplified/Traditional labels come from Wikidata; pinyin is added by a reverse lookup over the Chinese dictionary (most frequent reading, longest match, middle dots skipped, no pinyin if a character is unknown). 3,230 of 3,291 people got pinyin.*
- 制限 / Limits: 多音字は、頻度の高い読みになる(誤る場合がある)。人名辞書の再生成の順番: `build-people.ps1` → `build-people.ps1 -Japan` → `aruaru-ime people-pinyin`。パックの版(`PACKS_REV`)は r6。

## 英字を含む郵便番号と候補の件数 / Alphanumeric postal codes & candidate limits (2026-10-13)

- 英国(`SW1A 1AA` → 外側コード `SW1A`)・カナダ(`K1A 0B1` → 先頭3文字 `K1A`)・アイルランド(`D02 AF30` → `D02`)は、GeoNames が地区までしか持たないので、地区で引く。`-`・空白は有っても無くても同じ。実機のエンジン(HTTP)で確認: `SW1A1AA` → イギリス England Greater London Westminster Abbey、`K1A0B1` → カナダ Ontario Ottawa。
- 候補は、IPの国・選択中の言語の国が3件まで、その他の国は1件ずつ、全体で8件まで(候補窓があふれない)。オランダ(`1011AB`)のように数字で始まって英字を含むものは未対応。
- 右クリックの「辞書・登録」は、「登録する」「一覧・削除」「辞書・データを入れる」の3区分に整理した(実機の目視確認は未了)。

## 2026-10-13 の作業一覧 / Work log of 2026-10-13

| コミット / Commit | 内容 / What |
|---|---|
| `479d5e8` | 世界の郵便番号 → 国名つき住所(GeoNames 44か国、`-` 有無どちらも、IPの国 → 言語の国、日本は漢字データ優先) / world postal codes → "country + address" |
| `aff2f39` | 人名・有名人・重要人物名の辞書(約3300人、Wikidata CC0、既定で搭載)と生成スクリプト `tools/build-people.ps1` / people-name dictionary |
| `f9a9771` | パックの追加を1回の保存にまとめ、初回起動を 246秒 → 8秒に / first start 246 s → 8 s |
| `df5ac9f` | 辞書の長音を母音に直した読みでも打てる(`nyuuton` → ニュートン)、パック版 r5 / long-vowel alternate readings |
| `24394ae` | 候補の件数制限(優先国3件・他1件・計8件)、英字を含む郵便番号(英国・カナダ・アイルランド)、辞書メニューを3区分に整理 / candidate caps, UK/CA/IE postcodes, regrouped menu |
| `5fb4b0c` | 毎朝の自動データ更新、中国語の人名のピンイン(`aruaru-ime people-pinyin`)、パック版 r6、README/CLAUDE/PORTING の日英更新(引っ越し手順) / morning refresh, pinyin for Chinese names, docs |

- 確認したこと / Verified: コアのテスト 177 件(Windows・WSL Ubuntu)、HTTP の `POST /key` での打鍵(郵便番号・織田信長・ニュートン・英国/カナダの郵便番号)、実データでの `data-update`(1回目は全更新、2回目以降は条件付きで「変更なし」)。
- 確認していないこと / Not verified: 右クリックメニューと2列候補窓の実機の目視、インストーラーの実機の通し試験、毎朝の更新の長期運用、macOS の実行。

## v0.2.11〜v0.2.13 (2026-10-14): Fcitx5・キー入力・ウェブサービス名辞書・サーバーの共通辞書 / Fcitx5, key input, web-service names, shared server dictionary

**Fcitx5(Linux)**: `fcitx5/` に C++ のアドオン(`InputMethodEngineV2`)。キーを常駐エンジン(`127.0.0.1:4660`)の `/key` へ送り、`commit` / `preedit` / `cands` を Fcitx5 に反映する(IBus エンジンと同じ橋渡し。最小の JSON 解析と素の POSIX ソケットだけで依存を増やさない)。C++20 が必要(Fcitx5 のヘッダーが `std::span` などを使う)。
試験: `tools/fcitx5-test.sh`(DBus の `org.fcitx.Fcitx.InputContext1` から「konnichiha」→「今日は」→ Enter で確定を確認)。**落とし穴**: ヘッドレスでは `unset DISPLAY WAYLAND_DISPLAY` と専用の `XDG_RUNTIME_DIR` を設定し、`--disable=` で wayland / xcb / classicui などを止める(WSLg の Wayland に繋ぎに行って落ちる)。後始末で `pkill -f` を使うと自分自身を殺す(PID で kill する)。
`install.sh` は設定ファイルをユーザー側に置き、`.so` はシステムの fcitx5 フォルダへ(sudo が要る。無ければ実行すべきコマンドを表示)。
English: a C++ Fcitx5 addon bridges keys to the resident engine like the IBus engine does; needs C++20; tested headless via DBus (`tools/fcitx5-test.sh`). Install puts the config in the user folder and the `.so` in the system fcitx5 folder (sudo).

**キー入力の変更(ひらがなモード)**: `, . / \ [ ] @ ^` は半角のまま入力(エンジンは食わない。以前の `,`→`、` は取りやめ。`; :` は元のまま)。`{` は候補が `{ 「 ｛ 『 【`(2回続けて打つと `「`)、`<` は `< ＜ 、 …`、`>` は `> ＞ 。 …`、`?` は `? ？ ・ ･ …`。オンライン画面(RPoem の `open-runo-crossui`)は、keydown の既定動作を同期的に止め(`await` の後では間に合わず、生の文字が入って Space が効かなかった)、キーを順番に処理、IME 有効時の物理キー判定に日本語109(JIS)配列(IntlYen/IntlRo を見たら切替)を追加。最初の Space で、追加候補(「様です」など)を先頭から見せる。
English: `, . / \ [ ] @ ^` now pass through as half-width (the engine does not eat them; the earlier `,`→`、` was withdrawn); `{` double-press gives `「`; candidate orders for `< > ?`. The online screen stops the default action synchronously (it was too late after `await`, which let raw characters in and broke Space), serializes keys, and understands the JIS 109 layout. The first Space shows added candidates (e.g. 様です) first.

**ウェブサービス名の辞書(v0.2.13)**: `pack_tables/web_services.tsv`(ようつべ→youtube、ゆーちゅーぶ→YouTube、やふー→yahoo、Twitter・Google・Amazon・Facebook・Instagram・GitHub・LINE・ニコニコ動画)。既定でインストール。
**サーバーモードの共通辞書**: オンライン画面(`ARUARU_IME_SERVER=1`)は、利用者辞書を持たないため、辞書パックが効かなかった。`shared_user_dict.json`(データフォルダ)にパックだけを入れ、全セッションが**読み取り専用**で使う(利用者の入力は保存・学習しない。登録用の `/dict/*` は共通ファイルとは別の `user_dict.json` を触り、`/ime/api/` の中継には出していない)。
English: new web-service dictionary pack (ようつべ → youtube, やふー → yahoo, …). The server mode now loads the default packs into a shared, read-only `shared_user_dict.json`; user input is still not stored or learned.

**Android エミュレーターの黒画面**: エミュレーター(GPU が仮想)では WebView をソフトウェア描画にする(実機は変更なし)。pixel_6 / pixel_c の CI 試験は通る(スマホ側は時々不安定、失敗しても公開は止めない設定)。
**CI**: macOS Intel の `capture_store` の期限切れ試験は、待ち時間が短くて不安定だったので長くした。スモーク試験(`tools/e2e-smoke.sh`)は `/` が半角のまま通ることに合わせた。`uninstall.sh` は末尾に `exit 0`(Fcitx5 の後始末が `set -e` で失敗扱いになっていた)。

**リリース手順の教訓**(v0.2.11 で一度つまずいた): (1) `dist/` は `.gitignore` なので `git add dist/...` が失敗し、後ろの `git tag` が古い HEAD に付く → **コミットが入ったことを確認してからタグ**。(2) Windows インストーラーは、別の作業の未コミット変更が混ざらないよう `git worktree add ../aruaru-ime-rel HEAD` + `data/` をコピーして ISCC でビルド。(3) `gh release create` に Windows のインストーラーと `.sha256` を付けると、CI の release ジョブが残りを `--clobber` で追加する。(4) VPS は `/var/lib/aruaru-ime/downloads/` にインストーラーと `VERSION`、エンジンは `/opt/aruaru-ime/aruaru-ime-daemon`(`cargo build --release` 後に差し替えて `systemctl restart aruaru-ime-server`)。`Cargo.lock` の差分で `git pull` が止まったら `git checkout Cargo.lock`。
Release lessons: verify the commit landed before tagging (`dist/` is git-ignored); build the Windows installer from a clean `git worktree`; the CI release job uploads the other assets; VPS = installer + `VERSION` + engine binary swap + `systemctl restart`.

## 入力言語の切り替えを IME 本体に(2026-10-16, v0.2.15) / In-IME language switching
**Ctrl+Shift+L** で、設定で選んである入力言語(最大 7、右クリック「入力言語」)を順に巡る(`Key::NextLang`、`Session::next_lang`)。OS の入力方式(Win+Space)を切り替えなくても、IME の中で切り替えられる。入力中は、いまの入力を確定してから切り替える。選んである言語が 1 つ以下なら、日本語⇄英語。
配線: コア(`session.rs`、`would_eat_lang` も対応)、Windows の TSF(`map_key` で Ctrl+Shift+L、切り替え後に言語名を表示)、IBus・Fcitx5(Ctrl+Shift+L)、オンライン画面(RPoem の `open-runo-crossui`)、通知領域メニューの案内文。試験: コアの単体試験(日→英→露→日と巡る・入力中は確定してから・`would_eat`)。
**未確認**: Windows の実機での目視(言語名の表示)、macOS の入力メソッド(未対応)。
English: **Ctrl+Shift+L** cycles through the chosen input languages (up to 7) inside the IME (`Key::NextLang`); pending input is committed first; with fewer than two chosen languages it toggles Japanese and English. Wired into the core, Windows TSF, IBus, Fcitx5, the online screen and the tray hint. Tested in the core; not yet visually verified on a real Windows machine; macOS not supported yet.

## v0.2.15 のリリース内容
10/16 に別の作業が入れた変更(109 キー記号を MS-IME 準拠に: `,`→、 `.`→。 `/`→・ `[ ]`→「 」 …、直前が英数字は半角。JSON の型付き読み込みを `sonic-rs` に)に、上の言語切り替えを加えて公開。

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


## 依存更新(2026-11-06、version-control 通知への対応)
- **更新済み**(`cargo test -p aruaru-ime-core` 196件成功): base64 0.23・sha2 0.11・quick-xml 0.42(名前・本文が`&str`化、`decode()`廃止、`docimport.rs`を追従)・lopdf 0.47・sonic-rs 0.5(tsf/daemon)。
- **未更新(個別の作業が要る)**: 
- *Updated and tested: base64, sha2, quick-xml (str-based API), lopdf, sonic-rs. Deferred: windows 0.62 (the `implement` feature is gone; TSF COM code needs rewriting and on-device IME tests), ureq 3, jni 0.22, zbus 6.*

### ureq 3 へ更新済み(2026-11-06)
`crates/aruaru-ime-core/src/http.rs` に ureq 2 風の薄い互換層(`AgentBuilder`/`set`/`query`/`into_json` 等)を置き、約60箇所の呼び出しはほぼそのまま ureq 3 に載せた(`Error::Status(n,_)` は `Error::StatusCode(n)`)。ibus は直接 ureq 3 の API に移行。`cargo test -p aruaru-ime-core` 207件、WSL で ibus の `cargo check`、android の `cargo check` 成功。本文の読み込み上限は JSON/文字列が 256 MiB、バイト列は従来どおり呼び出し側の `take()`。実ネットワーク越しの通し試験は未実施。
*ureq 3 done via a thin compat layer (`http.rs`); real-network end-to-end not run.*

### zbus 6 へ更新済み(2026-11-06)
`SignalContext`→`SignalEmitter`、`#[zbus(signal_context)]`→`#[zbus(signal_emitter)]`、機能は `service`(`async-io` 機能は無くなった)、`StructureBuilder::build()` が `Result` に、`Type::signature()`→`SIGNATURE`、`Builder` の `address`/`name`/`serve_at` は `?` 不要に。WSL(Ubuntu)で `cargo test -p aruaru-ime-ibus` 成功(IBusText/LookupTableの署名が不変であることを確認)。**実際の ibus-daemon との接続試験は未実施**(Linux実機が要る)。非推奨警告(`zvariant::Value`→`zbus::Variant`等)は残している。
*zbus 6 done; builds and signature tests pass under WSL; not tested against a live ibus-daemon.*

### windows / windows-core 0.62 へ更新済み(2026-11-06)
`implement` 機能は不要に。追従した点: `BOOL` は `windows::core::BOOL`、COMの入力引数は `Option<&T>` ではなく `Ref<'_, T>`(`.as_ref()` で取り出す)、GDIの `DeleteObject`/`SelectObject` は `.into()`(HGDIOBJ)、`HWND`/`HINSTANCE`/`HDC`/`WPARAM`/`LPARAM` の一部が `Option`、`CreateFontW` の文字セット等が型付き、`AdviseKeyEventSink` の前面指定は `bool`。`cargo test`(core 204件・tsf・daemon)と tsf のリリースビルド成功。**regsvr32 での登録と実際の入力試験は未実施**(管理者権限が要る。リリース前に実機で必ず確認すること)。
*windows 0.62 done; builds and tests pass; DLL registration / typing on a real machine not tested (needs admin).*

### jni 0.22 へ更新済み(2026-11-06)
`crates/aruaru-ime-android/src/jni_glue.rs` を新API(`EnvUnowned`+`with_env`+`ThrowRuntimeExAndDefault`、`JString::try_to_string`/`from_str`、`jboolean`→`bool`)へ書き換え。パニックは JVM へ巻き戻さず例外になる。JNI の公開関数名・引数は変更なし(Java側は無変更)。`cargo test -p aruaru-ime-android`(5件)、`cargo ndk -t arm64-v8a build --release` 成功、`.so` に `Java_jp_aon_aruaruime_Native_{init,setField,api,page}` を確認。**実機/エミュレータでの起動は未実施**。
**ureq 3 移行の回帰を修正**: `send_json` が整形(インデント付き)で送っていた(androidのテストで発覚)。互換層 `http.rs` で詰めた JSON を送る。
*jni 0.22 done (glue rewritten, .so exports verified, device run not tested). Also fixed a ureq 3 regression: send_json sent pretty-printed JSON.*

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

## 2026-11-18 別アカウントでの再開用メモ(aruaru-llm Connect化・語彙拡張・版揃え) / Resume memo

**日本語**: (1) aruaru-llm呼び出しは全てConnect化済み(`/aruaru_llm.v1.Llm/<Method>`、`/v1/runtime`→`Runtime`)。RPoemの`open-runo-pdf-ocr`は`3b644db`でmain入り、pdf-r2l-rs v1.3.1公開済み(exe同梱)。(2) 語彙拡張(JMdict/JMnedict→`extra00.txt`、`aruaru-ime build-extra`、Wikidata P1814の自動更新)で辞書175万件超、VPS辞書・PCへ反映済み。(3) 版揃えテスト`crates/aruaru-ime-cli/tests/version_sync.rs`追加(afd98b8)。リリースごとにWindows exeを作りReleaseとVPS`downloads/VERSION`を揃える。
**未了**: VPSのエンジンが古く(Androidヘッダが`v=0.2.23`)、新版(`F:\_work\mig\aruaru-ime-daemon.v0225`はglibc2.39)への差し替えと`/ime`・Android画面の確認。`webpage/index.html`のDL欄が0.2.23のまま。PCインストーラーの再実行(アプリを閉じて管理者で)・メモ帳実入力・生成の正常系(VPSにQwen無し)・pdf-r2l-rsの他OS成果物。
**English**: (1) All aruaru-llm calls are on Connect; RPoem `open-runo-pdf-ocr` merged as `3b644db`; pdf-r2l-rs v1.3.1 released with exe. (2) Vocabulary expansion (JMdict/JMnedict via `build-extra`, Wikidata P1814 auto-update) gives 1.75M+ entries, deployed to VPS dict and the PC. (3) Version-sync tests added (afd98b8); always build the Windows exe per release and align Release and VPS `downloads/VERSION`.
**Open**: the VPS engine is older (Android header shows `v=0.2.23`) - swap in the new build and verify `/ime` and the Android screen; `webpage/index.html` still says 0.2.23; re-run the PC installer with apps closed; real typing test; generation success path (no Qwen on VPS); other-OS pdf-r2l-rs artifacts.
**Resume message**: "Continuing aruaru-IME (aon-co-jp/aruaru-IME, also pdf-r2l-rs, RPoem, aruaru-llm, F:\runo). Read PORTING.md section '2026-11-18 別アカウントでの再開用メモ' and finish the open items."


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

## 2026-11-23 追記: v0.2.30 インストーラーの確認(日本語)
- **ハッシュ**: `dist/aruaru-ime-windows-x86_64-setup.exe` のSHA256は同梱の`.sha256`と一致(`5d929eb1…aa5c`)。
- **インストール済みの状態(確認済み)**: `C:\Program Files\aruaru-IME` に本体・常駐エンジン・TSF DLL・辞書・`unins000.exe` があり、`aruaru-ime-daemon` が起動中。`GET http://127.0.0.1:4660/health` は `ok dict=1225528 v=0.2.30 proto=1`、`Ime/Version` は `0.2.30`。2026-11-21 15:12 の自動インストール(`installer-e2e.log`、exit=0、CLSID・Runキー登録あり)の結果と一致する。
- **今回やっていないこと**: 再インストール・アンインストールは管理者昇格(UAC)が要り、この環境では承認できないため実行していない(迂回もしていない)。メモ帳等での実入力(変換)の目視確認も未実施(画面操作の許可を取っていない)。システム設定は変更していない。
- **所有者の操作(昇格が必要)**: 管理者のPowerShellで `Start-Process F:\aruaru-IME\dist\aruaru-ime-windows-x86_64-setup.exe -ArgumentList '/VERYSILENT' -Wait` で入れ直し→メモ帳でaruaru-IMEを選んで入力→`C:\Program Files\aruaru-IME\unins000.exe /VERYSILENT` でアンインストール、の順に確認する。アンインストール後は `aruaru_ime_tsf.dll` が残らないか(前回は残った記録あり)も見る。
- **未確認**: アンインストールの通し(DLLの残り)、メモ帳での実入力、アンインストール情報の登録名(HKLM/HKCU/WOW6432では`aruaru-IME`の表示名を見つけられなかった。`unins000.exe`は存在)。


## 2026-11-23 追記: url 2.5.9 へ更新 / Update: url 2.5.9
**日本語**: RUSTSEC-2026-0592(`Url::origin()` のスタックあふれ)対応で `url` を 2.5.8→2.5.9 に更新(207件成功(cargo test --workspace))。VPS のクローンも追従済み。稼働バイナリは未入れ替え。再開時は未コミットの作業中変更の有無を `git status` で確認。
**English**: Updated `url` 2.5.8 to 2.5.9 for RUSTSEC-2026-0592 (stack overflow in `Url::origin()`); 207 tests pass (cargo test --workspace). The VPS clone is fast-forwarded; running binaries were not swapped. On resume, check `git status` for uncommitted work.

## Versionless API の方針 / Versionless API policy (2026-11-25)
**日本語**: このリポジトリのAPIは **Versionless**(URL・パッケージ名に `v1` 等を付けず、互換性を壊さず足すだけで進化させる)。欄は足すだけ・足す欄は省略可能・知らない欄は無視・名前変更は新名を足して旧を非推奨に(期限は既定で6か月以上、使用0を確認してから削除)・enumは未知の値を許容・安全上の緊急修正は例外。既存の `.v1` は呼び出し側を壊さないよう当面残し、版なしの別名を追加して段階的に畳む。詳細・出典は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md) 末尾。適用はこれから順に行う。
**English**: This repository's APIs are **Versionless**: no `v1` in URLs or package names; evolve by adding only, without breaking compatibility. Only add fields, new fields are optional, ignore unknown fields, rename by adding the new name and deprecating the old (default deadline 6 months or more, remove only after confirming zero use), tolerate unknown enum values, emergency security fixes are the exception. Existing `.v1` names stay for now so callers do not break; a versionless alias is added and `.v1` is retired in stages. Details and sources: the end of [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md). Rollout is step by step.

## 完成まで作り切る方針 / Finish everything policy (2026-11-27)
**日本語**: 全リポジトリで、**未確認は自動で確認し、未実施は実施し、未了は完成させる**(確認を求めず進める)。使用頻度が低いと判断される機能や、**1から作り直す大規模開発になる内容でも、やめずに最後まで開発・TEST・DEBUGする**(別の方式で済ませない、後回しにしない)。進め方: 調査(英語・日本語の検索とGitHub) → 設計 → 実装 → 実機/実データで確認 → 失敗は原因を直して再試験 → README・CLAUDE・PORTING・POLICY を日英で更新 → push。「できた」と書くのは実測した範囲だけ、確認できなかった所は「未確認」と正直に書く。例外(確認を取る): 元に戻せない削除、秘密の公開、所有者の認証・決済・実機操作が要るもの、利用者のシステム設定の変更。
**English**: In every repository, **verify what is unverified, do what is not yet done, and finish what is unfinished** automatically, without asking. Even features that look rarely used, or work that means **a large rewrite from scratch, are developed, tested and debugged to the end** (no substituting another approach, no postponing). Method: research (English and Japanese search plus GitHub) -> design -> implement -> verify on real devices or data -> fix causes and retest on failure -> update README, CLAUDE, PORTING and POLICY in Japanese and English -> push. Write "done" only for what was measured, and say "unverified" honestly for the rest. Exceptions that still need confirmation: irreversible deletions, publishing secrets, anything needing the owner's authentication, payment or real-device actions, and changes to the user's system settings.