# aruaru-IME

画面の文脈から「今どの文字種で打っているか」を予測して変換する AI IME。
An AI-assisted IME that predicts the input mode (alphabet / hiragana / katakana / number / symbol / English word) from the screen context.

対象: Windows / macOS / Linux / Android(開発中)。

## 現在の状態(v0.1.0・開発初期)

| 部品 | 状態 |
|---|---|
| `aruaru-ime-core`: ローマ字→かな、入力モード予測、候補生成 | 実装・単体テスト済み |
| `aruaru-llm` 連携(OCR `/v1/ocr`、Qwen `/v1/generate-qwen`、検索 `/v1/search/raw`) | 実装済み(未接続でも規則だけで動作)。実サーバーでの通し試験は未了 |
| 画面キャプチャの一時保管(`capture_store`) | 実装・テスト済み |
| OS 組み込み(Windows TSF / macOS InputMethodKit / Linux IBus・fcitx5 / Android IME)・画面キャプチャ常駐・インストーラー | **未着手** |

## 画面キャプチャの扱い(プライバシー)

- 文字入力欄にカーソルがあるときだけ、数秒に一度キャプチャする(予定)。
- 画像は OS の一時フォルダ内の専用ディレクトリにだけ保存し、**新しい1枚を保存するたびに、60秒より古いもの・30枚を超えた古いものから自動削除**。終了時も全削除。
- パスワード欄ではキャプチャしない(予定)。送信先は既定で同じ PC の `aruaru-llm`(`http://127.0.0.1:4600`)。

## 試す

```
cargo test
cargo run -p aruaru-ime-cli -- convert konnnichiha --before 彼は
cargo run -p aruaru-ime-cli -- convert hello --before "Say " --offline
```

ライセンス: MIT OR Apache-2.0
