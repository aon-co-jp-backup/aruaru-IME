# aruaru-IME PORTING / 移植・再開メモ

## アーキテクチャ / Architecture

```
 アプリ(TSF/IBus/IMK/Android)
   └ OS別の薄い層 ── キー入力 ──▶ core::session::Session(状態機械・OS非依存)
        (aruaru-ime-tsf)               ├ Converter → 常駐エンジン(127.0.0.1:4660)または辞書なしのローカル変換
                                       ├ 学習履歴・利用者辞書(共有データ領域)
                                       └ 未確定・候補・確定(Step)を返す
 常駐エンジン(aruaru-ime-daemon)
   ├ /convert … core::convert(モード予測 + 辞書Viterbi + 予測 + 検索サジェスト)
   ├ /focus   … 入力欄のフォーカス通知(これがある間だけキャプチャ)
   ├ /dict/*  … 利用者辞書の登録・削除
   └ 画面キャプチャ → CaptureStore(一時フォルダ、60秒/30枚) → aruaru-llm /v1/ocr
```

OS 別の層が行うこと(他OSへ移植するときのチェックリスト):
1. キー(物理キー+修飾)を `Key` に変換し、`would_eat` で食う/食わないを決める(押下前の判定と実処理を分ける OS ではこの関数を使う)。
2. `Step.preedit`(未確定)・`Step.commit`(確定)・`Step.cands/sel/mode`(候補窓)を反映する。**反映に失敗したら `session.on=false` にして素通しにする。**
3. 入力欄にフォーカスが入ったら `/focus {"editable":true,"password":false}` を数秒ごとに送る。パスワード欄では `password:true`。
4. カーソル直前の文章を読めるなら `set_before` に渡す(予測精度が上がる)。
5. ライブ変換(既定オン): `Session.live`。入力のたびに最有力の変換結果を `preedit`(`sel=Some(0)`)として返す。Space で次候補、Enter で確定、F6でひらがな。
6. 配色は `settings.json` の `theme`(既定 `dark`)。F2 で切替。

## アプリ互換 / App compatibility

TSF の標準 API(`ITfContextComposition`・`ITfInsertAtSelection`・`GetTextExt`)だけを使うので、アプリ別の拡張は無い。Chromium 系(VS Code・Claude・Chrome・Edge)は TSF のテキストストア経由。
実機確認の済んだアプリは README の表を参照。未確認のアプリで不具合があれば `%TEMP%ruaru-ime-log.on` を作ってログ(`pid=` 付き)を取る。

## 自動アップデート / Auto-update

`core::update`。**起動時の1回だけ**確認する(定期確認なし)。`<データ領域>/app/versions/<タグ>/` に展開し、Windows は HKCU の COM 登録を新しい DLL に向けて(管理者権限不要)、
常駐エンジンを新しい版で入れ替える(新しいエンジンが旧エンジン終了後にポートを引き継ぐ)。使用中のファイルは上書きしないので、起動中のアプリは再起動すると新しい版になる。
リリース資産は `aruaru-ime-windows-x86_64.zip`(必須: `aruaru_ime_tsf.dll`・`aruaru-ime-daemon.exe`)+任意で `<zip名>.sha256`。

## 共有データ領域 / Shared data dir

`%LOCALAPPDATA%\aruaru` / `~/.local/share/aruaru` / `~/Library/Application Support/aruaru`(`ARUARU_DATA_DIR` で上書き)。
`components.json`(入れた部品の台帳)、`user_dict.json`(登録語)、`learned.json`(学習履歴)、`settings.json`、`<name>.env`(部品の起動設定)。

## 辞書 / Dictionary

Mozc OSS 辞書(`google/mozc` の `src/data/dictionary_oss`、BSD-3-Clause)を `aruaru-ime build-dict <src> data/dict.bin` でコンパイルする
(約64MB、122万エントリ)。バイナリ形式は `crates/aruaru-ime-core/src/kanji.rs` 冒頭。`data/` は `.gitignore`(配布物に同梱する)。

## Windows(実装済み)

- `aruaru_ime_tsf.dll`(TSF): `ITfTextInputProcessorEx`+キー/スレッドマネージャ/コンポジションのシンク。登録は `regsvr32`(HKLM、管理者権限)。
- 開発用: DLL の COM 登録を HKCU で上書きして試す(CLAUDE.md 参照)。`tools/testhost/TestHost.exe` は自プロセスだけで IME を有効化し、入力内容を `%TEMP%\testhost.txt` に出す。
- 診断ログ: `%TEMP%\aruaru-ime-log.on` を作ると `%TEMP%\aruaru-ime-tsf.log` に出力。
- 未対応: 32bit DLL、入力スコープによるパスワード欄判定、ITfLangBarItem(タスクバーのモード表示。代わりに切替時のフラッシュ表示)、候補窓のクリック操作。

## 未着手 / TODO

- macOS: InputMethodKit(Swift か Rust+FFI)、Linux: IBus / fcitx5 のエンジン、Android: `InputMethodService`。いずれも `Session` を呼ぶ薄い層にする。
- インストーラー: Windows は Inno Setup(管理者権限)。手順: ファイル配置 → `regsvr32` → 入力方式リストへの追加 → `aruaru-ime ensure-components`(aruaru-llm / open-easy-web を重複なく導入、Qwen を導入)。
  open-easy-web は固定アカウントのメールアドレスが必須なので、インストーラーのページで利用者に入力してもらい `<data>/open-easy-web.env` に書く(勝手に決めない)。
- Qwen の高速化: 現状は0.5Bで約50秒/回(CPU)。`open-cuda` の行列積(AVX2/並列)の最適化、またはバッチ化したスコアリング(`/v1/ocr/fill` と同じ `score_continuations`)で再ランクする。
- 利用者辞書の登録ダイアログ(いまは CLI と HTTP のみ)。

## 試験 / Testing

- `cargo test`(コア 54 件+TSF 1 件)。辞書がある時だけ走る実辞書の試験を含む。
- 実機: `tools/ime-e2e.ps1`(**無人で**走らせる)。OCR: 合成画像で VPS の `aruaru-llm` に 0.7 秒・高精度を確認済み(ローカルには Tesseract が無い)。
