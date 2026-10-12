# aruaru-IME PORTING / 移植・再開メモ  (Porting & resume notes)

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
- 原因: エンジンがサインイン時に自動起動しておらず(開発機にRunキー無し)、再選択が走らなかった。エンジン起動時にHKLM/HKCUのRunキーを確認し、無ければHKCUに自己登録(`ensure_autostart`)。再選択後に `restore-ime.log` へ「aruaru-IME active afterwards」を記録。手動の再選択で active=true を確認。再起動での再確認は未了。
- 無変換キー: 最初の1回は必ずひらがな(変換中の漢字も読みに戻す)、以後 カタカナ→全角英数→半角英数→ひらがな。
- 「あきのこうよう」→「秋の紅葉」: 季節+の+季節語を言い回しに追加、辞書に無い季節語のコストを下げた。
