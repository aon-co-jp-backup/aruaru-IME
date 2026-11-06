# POLICY(API 方針)

正本: RUNO の API-POLICY.md。REST 全廃・Connect・gRPC。

## REST 全廃・Connect 移行の状況 (2026-11-06)

方針の正本は RUNO の `API-POLICY.md`。常駐エンジン(`aruaru-ime-daemon`、127.0.0.1:4660)の局所API(14本)と aruaru-llm 呼び出し(1本)の状況:

- **移した**: エンジン側 `POST /aruaru_ime.v1.Ime/<Method>`(Key / Templates / Session / VocabLearn / VocabRemove / VocabList / DictAdd / DictRemove / DictList / Focus / Convert / Stats / Version の13本)。POST 以外・未知のメソッドは404。呼び出し元のうちこのリポジトリ内のもの(TSF DLL・ibus・fcitx5・macOS・`tools/e2e-smoke.sh`)を同時に新経路へ。aruaru-llm の `GET /v1/knowledge/search`(1本)は、現行Connect形 `POST /aruaru_llm.v1.Llm/KnowledgeSearch`(本文 `{}`)へ。
- **期限つき互換で残した(旧経路 /key /session /templates /vocab/* /dict/* /focus /convert /stats /version)**: すでに配布済みのクライアント(インストール済みのTSF DLL、easy-web.tokyo のオンライン画面、RPoem `open-runo-crossui` が生成するページ、Android の upstream `easy-web.tokyo/ime/api`)が使うため。**期限 2026-12-06**。期限後に `crates/aruaru-ime-daemon/src/main.rs` の旧 match 腕と、Android(`crates/aruaru-ime-android`)の経路名を削除する。
- **例外**: `GET /health`(ヘルスチェック)、`GET /ui`(画面のHTML)。
- **未了/注意**: Android は upstream(easy-web.tokyo 側)の経路が変わるまで旧経路のまま。サーバー側(open-easy-web の /ime/api)の移行は別担当。受信JSONは既存どおり `sonic_rs`(上限なしのローカル専用ループバック)。
