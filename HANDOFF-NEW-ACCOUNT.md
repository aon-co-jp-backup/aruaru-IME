# 新しい Claude アカウントへの引き継ぎ(aruaru-IME・dream-os・作業ルール)/ Hand-off to a new Claude account

> **前提**: 旧アカウントを削除すると、Claude の「記憶」(`~/.claude/projects/F--runo/memory/`)は**失われる**。
> この文書と、各リポジトリの `CLAUDE.md`(冒頭と HANDOFF)・`PORTING.md` が**唯一の正**。秘密(キー・トークン・パスワード)の値はここに書かない(場所だけ)。
> *Assume Claude's memory is lost with the old account. This file plus each repo's `CLAUDE.md` / `PORTING.md` are the only source of truth. No secret values here — locations only.*

## 0. 最初にやること / First steps
1. このメッセージをそのまま新しい Claude に貼る。`F:\runo\CLAUDE.md` と、作業するリポジトリの `CLAUDE.md` の冒頭・HANDOFF を読ませる。
2. 全体(エコシステム)の引き継ぎは `F:\runo\PORTING.md` の「アカウント引っ越しのための引き継ぎ(2026-10-14)」。この文書は、**aruaru-IME・dream-os と、ユーザーの作業ルール**の分。
3. 推測で動かず、`git log` / `git status` / 実機で**確認してから**進める。

## 1. ユーザーとの付き合い方(記憶の代わり)/ Working rules (what the memory held)
- **チャットは日本語のみ**(英語を混ぜない)。ただしリポジトリの文書は日英併記(README/CLAUDE/PORTING は日本語+英語。GitHub は最大10言語程度、PC・VPS は日本語のみでよい。言語の追加・削除は都度ユーザー指示が優先)。
- **断片のメッセージには、続きを待つ**(「スマホは、」のような途切れた文)。推測で大きく動かない。確認が要るのは、ダウンロード(許可が要る。辞書・フォント・インストーラー等は、取得元・名前・大きさを示して)・VPS/リポジトリ/フォルダの新規作成(命名に選択肢があるとき)・再起動・破壊的操作。
- **「完成」と言う前に実機で試験**(CI 成功≠動作確認)。バグは自分で見つけて直す。着手したスライスは難所まで最後まで通す(「次フェーズ」に逃げない、半分繋がった足場を成果と報告しない)。UI・インストーラーは、複数解像度・縦横・インストール/アンインストールまで試験。
- **開発 → 試験 → 改善を、実用になるまで繰り返す**。終わったら README/CLAUDE/PORTING を日英で更新 → コミット → push → リリース → VPS へ反映、までが一式(「コミットして push、VPS へ UPLOAD」と言われたら全部)。リミット接近前にもドキュメントを更新して push。
- **最先端は自動で追従**: 日英検索で他社が既に対応済みと分かったら、「今は大丈夫」と報告せず実装まで進める。
- **汎用化できるロジックは RPoem(`F:\RPoem`、`open-runo-*`)の共有クレートへ**。アプリ本体は Rust+RPoem、Python を残さない。JSON は `RS-JSON` の `parse_secure_sonic` を使える所で使う。
- **秘密は `F:\` 直下の txt から読み、`scp` で VPS の `.env` へ**。値を画面・コミット・ログに出さない。チャットに貼らせない。
- **`cargo fmt --all` 禁止**(兄弟リポジトリまで整形される)。`cargo fmt -p <crate>`。他のセッションの未コミット変更があるので **`git add -A` しない**(自分のファイルだけ)。
- ユーザーが同じ PC を操作中は、SendKeys などの実機打鍵試験をしない(混線する)。ユーザーのアプリを勝手に終了しない。

## 2. 環境 / Environment
- Windows 11、作業ドライブ `F:\`(索引リポジトリ `F:\runo`、各プロジェクトは `F:\<名前>` の兄弟)。Rust は `%USERPROFILE%\.cargo\bin` を PATH の先頭に。WSL に Ubuntu あり(`wsl -u root` / `-u norukia`。`MSYS_NO_PATHCONV=1`)。Android 実機(moto g53y)が USB 接続(`adb`)。Mac は無い(macOS は CI のビルド確認のみ)。
- 本番は ConoHa VPS(`ssh conoha`、AlmaLinux、4 コア・RAM 3.6GB・GPU なし)。リポジトリは `/root/repository/`、サイトは `https://easy-web.tokyo`。
- **落とし穴**: Bash のヒアドキュメントで日本語や `\` を書くと壊れる → Write/Edit ツールでファイルに書く。日本語を含む `.ps1` は BOM 付き UTF-8、`.cmd` は英語のみ(`.gitattributes` で CRLF)。Git Bash の WSL 呼び出しは `MSYS_NO_PATHCONV=1`。`sleep` の連鎖は禁止(待つなら `until ...; do sleep` をバックグラウンドで)。

## 3. aruaru-IME(`F:\aruaru-IME`、https://github.com/aon-co-jp/aruaru-IME)
**現状**: v0.2.14 公開済み(Windows/macOS/Linux/Android、CI 全成功)。VPS のオンライン画面(https://easy-web.tokyo/ime)も 0.2.14。
- 構成: `aruaru-ime-core`(変換エンジン、Mozc 辞書+Viterbi、各言語、パック、郵便番号、駅名…)、`aruaru-ime-daemon`(常駐エンジン `127.0.0.1:4660`、サーバーモード `ARUARU_IME_SERVER=1`)、`aruaru-ime-cli`、`aruaru-ime-tsf`(Windows TSF DLL)、`aruaru-ime-ibus`(Linux IBus)、`fcitx5/`(Linux Fcitx5)、`macos/`(InputMethodKit)、`android/`(キーボード+WebView)。共有の画面は RPoem の `open-runo-crossui` が生成。
- 入力の約束(ユーザー指示): 漢字に変換することが前提(ライブ変換)。ひらがなモードでも **`, . / \ [ ] @ ^` は半角のまま**、`{` は `{` の次に `「`(2回で `「`)、`<` は `< ＜ 、`、`>` は `> ＞ 。`、`?` は `? ？ ・ ･`、`;` `:` は元のまま。無変換キーはひらがな。「よう → 用 様 要」「ようつべ → youtube」「やふー → yahoo」。
- **リリース手順**(2回つまずいた): (1) バージョンを `Cargo.toml`(6 行目)・`installer/aruaru-ime.iss`(5 行目)・`android/app/build.gradle`(versionCode/Name)で上げ、`dist/release-notes.md` に追記(`dist/` は git 管理外なので `git add dist` は失敗する)。(2) **コミットが入ったことを確認してから** `git tag vX && git push origin vX`(確認前にタグを打つと古い HEAD に付く)。(3) Windows インストーラーは `git worktree add ../aruaru-ime-rel HEAD` に `data/` をコピーして cargo build → ISCC(別作業の未コミット変更を混ぜない)。(4) `gh release create vX --target main --prerelease -F dist/release-notes.md <setup.exe> <sha256>`、CI の release ジョブが残りの成果物を足す。(5) VPS: `scp` で `/var/lib/aruaru-ime/downloads/` へ(exe・sha256・VERSION)、エンジンは `/root/repository/aruaru-IME` で `git checkout Cargo.lock; git pull; cargo build --release -p aruaru-ime-daemon` → `/opt/aruaru-ime/aruaru-ime-daemon` を差し替えて `systemctl restart aruaru-ime-server`。
- データ(辞書など)は、利用者が選んだものだけ自動で取らない(例外: 国名・都市名と、取得済みデータの更新)。お引越しは `aruaru-ime move export|import`。
- **未了**: インストーラーの実機の通し試験(最近の版)、右クリックメニュー/2列候補窓の目視確認、多言語の切替入力の IME 本体への組み込み、台湾・ギリシャ等の郵便番号、macOS/Linux の実機(Mac 無し)、Windows の通知領域メニューの目視確認。同じリポジトリを別の Claude セッションも触ることがある(`geo.rs`・`people.tsv`・`dataupdate.rs` など)。
- 詳細: `aruaru-IME/CLAUDE.md`(HANDOFF)・`PORTING.md`(「引っ越し手順」「v0.2.11〜v0.2.13」)。

## 4. dream-os(`F:\dream-os`、https://github.com/aon-co-jp/dream-os)
- **構想**: Linux × TRON のハイブリッドカーネルで各 OS のアプリを動かす(構想段階)。**単独で起動できる OS はまだ無い**。実在するのは、実機で動作確認済みの実行基盤(open-cuda の Vulkan、open-directx の DXBC→SPIR-V、open-raid-z の RAID6/Z2、通信)と、`dream-os-fonts`(IPAゴシック同梱、描画確認済み)。
- Windows への導入: `installer\windows\install.ps1`(ユーザー単位。`dream-os status|raid|dx|fonts|linux|both`)。「Windows と dream-os の同時起動」は **WSL2 との並行実行**で実現(切り替え・単独起動は不可と、ユーザーに正直に説明済み)。
- 試験: Windows(GT 730)と Ubuntu(WSL2・llvmpipe)で、dream-os・open-cuda・open-directx・open-cpu・aruaru-llm が成功。**未試験**: Ubuntu での `dream-os-raid-bridge`(`dxc` が要る。取得はユーザーの許可を取ってから)。
- ビルドには `glslc`(Vulkan SDK。Ubuntu は `apt install glslc`)が要り、`open-cuda/examples/flash_attention_vulkan_real/shaders/flash_attention.spv` は git 管理外なので `glslc --target-env=vulkan1.1 flash_attention.comp -o flash_attention.spv` で作る。詳細は `dream-os/PORTING.md`。

## 5. 次にやるとよいこと / Suggested next steps
1. `F:\runo\PORTING.md` の最優先の未完了(`open-web-server` の VPS 上の未 push 作業を GitHub へ push)を先に確認。
2. aruaru-IME: インストーラー実機の通し試験 → 次のリリース。ユーザーの新しい要望(変換の追加・キー入力の調整)に応える。
3. dream-os: `dream-os-raid-bridge` の Ubuntu 試験(許可を取って `dxc`)、構想の次の段階の相談。
