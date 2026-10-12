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
