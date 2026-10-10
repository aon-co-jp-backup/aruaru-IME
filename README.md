# aruaru-IME

画面の文脈から「いま何を打とうとしているか」を予測して変換する AI IME。
An AI-assisted IME that predicts what you are typing (alphabet / hiragana / katakana / number / symbol / English word) from the screen and the surrounding text, then converts.

> **状態: v0.1.0 アルファ(開発中)**。Windows 版は実機のアプリ上で「ローマ字→かな→確定」まで動作を確認済みですが、
> インストーラー・macOS・Linux・Android 版は未完成です。**日常の入力にはまだ使わないでください。**
> **Status: v0.1.0 alpha.** The Windows build works end-to-end in a real app (romaji → kana → commit), but the installer and the macOS / Linux / Android builds are not finished. Do not rely on it for daily typing yet.

## 主な機能 / Features

| 機能 | 状態 |
|---|---|
| ローマ字→かな、**入力モードのAI予測**(アルファベット/ひらがな/カタカナ/数字/記号/英単語)。直前の文章・画面のOCR・入力欄の種類から判定 | 実装・単体テスト済み |
| **かな漢字変換**: Mozc OSS 辞書(BSD-3-Clause)+連接コストの最小コスト経路(Viterbi)。1回の変換はミリ秒 | 実装・実辞書で文単位まで確認 |
| **ライブ変換(既定オン)**: 漢字に変換する前提で、打つそばから最有力の変換結果(漢字かな交じり)を未確定表示。Enter で確定、Space で次候補、F6 でひらがなへ。設定 `live_conversion` で切替 | 実装・実辞書でも単体テスト済み(実機は再確認待ち) |
| **予測変換**: 途中まで打つと「ありがとう」→「ありがとうございます」。登録語・学習履歴を最優先 | 実装・単体テスト済み |
| **検索フォーム型の予測**(Google サジェスト風): 検索欄・Google/Amazon 画面で続きの検索語を出す | 実装済み(実通信は未検証) |
| **誤モード対策**: F6〜F10 で表記を強制(ひらがな/カタカナ/全角英数/半角英数)し学習、英数・かなキーは必ずオフ/オン、Shift 始まりは英字のまま、Caps Lock 中は食わない、確信が低いと英字に切り替えない、反映に失敗したら自動でオフ | 実装・単体テスト済み。実機は一部のみ確認 |
| **単語・熟語・文章の登録**: CLI / 常駐エンジンの API。`.txt` `.csv` `.xlsx` `.docx` `.pptx` `.pdf` から一括取り込み(読みは辞書の逆引きで自動生成)、ファイル単位で一括削除 | 実装・単体テスト済み(画像だけのPDFは未対応) |
| **候補窓**: 黒背景・白文字が既定。見出し右のボタンを1クリック(または F2・設定)で白背景・黒文字に切替。入力モード(あ/ア/A)を表示 | 実装済み |
| **画面キャプチャ**(入力欄にフォーカスがある間だけ数秒ごと): 一時フォルダに保存し、**60秒より古い/30枚を超えた分から自動削除**。パスワード欄では撮らない | 実装・実機確認 |
| **コンポーネント管理**: `aruaru-llm` / `open-easy-web` を重複インストールせず、無ければ取得、有れば再利用。Qwen も導入 | 実装・実機確認(open-easy-web は利用者のアカウント設定が必要) |
| **自動アップデート**(既定オン、設定 `auto_update`): **起動時の1回だけ** GitHub Releases の最新版を確認し、新しければ版ごとのフォルダへ取得して切り替え(管理者権限不要)。アプリを再起動すると新しい版になる。`.sha256` 付きならハッシュ検証、取得先は GitHub のみ。手動は `aruaru-ime update [--check]` | 実装・単体テスト済み(公開リリースがまだ無いため実取得は未検証) |
| Windows TSF 版 IME 本体 / 常駐エンジン | 実装(実機で一部確認) |
| インストーラー(Inno Setup) / macOS / Linux(IBus・fcitx5) / Android | **未完成** |

## 対応アプリ(プラグイン不要)

aruaru-IME は OS 標準の入力方式(Windows TSF)として動くので、アプリ側の拡張機能・プラグインは要りません。
Visual Studio Code・Claude デスクトップ・メモ帳・ブラウザなど、日本語入力ができるアプリならそのまま使えます(VS Code のターミナル内も、入力欄としては同じ経路)。

| アプリ | 状態 |
|---|---|
| メモ帳 / 自作テストホスト(WinForms) | 実機で確認済み(古い版のDLLが読み込まれていたため、再起動後の再確認待ち) |
| Claude デスクトップ(Electron) | 古い版を読み込んだままで未確認。再起動後に確認する |
| Visual Studio Code(Electron) | **未確認**(同じ Chromium の入力経路のため動く見込みだが、実機では試していない) |

> DLL は起動中のアプリが読み込んだままになります。**更新・再インストールの後は、アプリを一度終了して起動し直してください。**

## 構成

- `crates/aruaru-ime-core` — エンジン(OS非依存): ローマ字・モード予測・辞書変換・セッション(キー入力の状態機械)・利用者辞書・ファイル取り込み・設定・コンポーネント管理
- `crates/aruaru-ime-daemon` — 常駐エンジン(127.0.0.1:4660): 変換 API、画面キャプチャ+OCR(`aruaru-llm`)
- `crates/aruaru-ime-tsf` — Windows の IME 本体(TSF DLL)
- `crates/aruaru-ime-cli` — `aruaru-ime` コマンド(`convert` / `dict` / `build-dict` / `ensure-components`)
- `tools/` — 実機試験用のテストホストと通し試験スクリプト

## 使い方(開発用)

```
cargo test
# 辞書(Mozc OSS)を用意してコンパイル: data/mozc-src に dictionary00-09.txt・connection_single_column.txt・id.def を置く
cargo run --release -p aruaru-ime-cli -- build-dict data/mozc-src data/dict.bin
cargo run --release -p aruaru-ime-cli -- convert kyouhaiitenkidesu --before 私は --offline
cargo run --release -p aruaru-ime-cli -- dict add aon 株式会社AON
cargo run --release -p aruaru-ime-cli -- ensure-components
```

## 画面キャプチャと個人情報

- 入力欄にフォーカスがある間だけ、数秒に一度キャプチャします。画像は OS の一時フォルダの専用ディレクトリにだけ置き、新しい1枚を保存するたびに、**60秒より古いもの・30枚を超えたものを古い順に自動削除**します(終了時も全削除、強制終了で残った分は次回起動時に掃除)。
- パスワード欄ではキャプチャしません。ただし**パスワード欄の判定は、アプリ側がパスワード欄でIMEを無効にすることに依存します**(入力スコープでの判定は未実装)。
- 画像の送信先は既定で同じPCの `aruaru-llm`(`http://127.0.0.1:4600`)です。

## 正直な開示(現時点の限界)

- Qwen(0.5B)による変換は、この PC の `aruaru-llm`(CPU)で **1回約50秒かかり、候補も不正解だったため**、変換の主経路は辞書+Viterbiにしています。Qwen は再ランク・学習用の補助で、同期の変換経路には置いていません(`ARUARU_IME_QWEN=1` のときだけ追加)。
- Windows では 64bit アプリのみ(32bit 版 DLL は未作成)。
- 実機の自動打鍵試験は、同じ PC で人が操作していると他のウィンドウに混ざるため、無人の環境で行う必要があります。

ライセンス: MIT OR Apache-2.0(辞書データは Mozc の BSD-3-Clause)。
