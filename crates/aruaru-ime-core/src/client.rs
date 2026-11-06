//! aruaru-llm(open-cuda のQwen・Tesseract OCR・検索)へのクライアント。
//! 接続できない/モデル未導入のときは全てエラーではなく `None` / 空で返し、IMEは規則だけで動き続ける。

use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;

pub const DEFAULT_URL: &str = "http://127.0.0.1:4600";

#[derive(Clone)]
pub struct Client {
    base: String,
    agent: crate::http::Agent,
}

impl Client {
    pub fn new(base: &str) -> Self {
        let agent = crate::http::AgentBuilder::new().timeout(Duration::from_secs(8)).build();
        Self { base: base.trim_end_matches('/').to_string(), agent }
    }

    fn post(&self, path: &str, body: Value) -> Option<Value> {
        self.agent.post(&format!("{}{}", self.base, path)).send_json(body).ok()?.into_json().ok()
    }

    fn post_timeout(&self, path: &str, body: Value, secs: u64) -> Option<Value> {
        self.agent.post(&format!("{}{}", self.base, path)).timeout(Duration::from_secs(secs)).send_json(body).ok()?.into_json().ok()
    }

    /// aruaru-llm の知識ベース(DATABASE)を全件取得する(IMEの補足表示用に手元へ取り込む)。
    pub fn knowledge_all(&self) -> Option<Vec<Value>> {
        // Connect の単純呼び出し(旧 GET /v1/knowledge/search)。q 無しで全件。
        let v = self.post("/aruaru_llm.v1.Llm/KnowledgeSearch", json!({}))?;
        v["items"].as_array().cloned()
    }

    /// 公開サイト(maid-cafe.tokyo)のDBから、語(消費税・核武装など)の固定回答を取る。無ければ None。
    pub fn public_answer(term: &str) -> Option<String> {
        let agent = crate::http::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
        let v: Value = agent
            .post("https://maid-cafe.tokyo/openenglish.v1.ChatProviders/CompletePriority")
            .send_json(json!({"prompt": format!("Student: {term}について")}))
            .ok()?
            .into_json()
            .ok()?;
        if v["reply"]["provider"].as_str() != Some("knowledge") {
            return None;
        }
        v["reply"]["text"].as_str().map(str::to_string)
    }

    /// 公開サイトのカスタムQ&A(DATABASE。核武装など)を全件取得する。(キーワード群, 日本語の回答)。
    pub fn public_custom_qa() -> Vec<(Vec<String>, String)> {
        let agent = crate::http::AgentBuilder::new().timeout(Duration::from_secs(10)).build();
        let Some(v) = agent
            .post("https://maid-cafe.tokyo/openenglish.v1.CustomQa/CustomQaGet")
            .send_json(json!({}))
            .ok()
            .and_then(|r| r.into_json::<Value>().ok())
        else {
            return vec![];
        };
        v["pairs"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|p| {
                        let kw: Vec<String> = p["keywords"].as_array()?.iter().filter_map(|k| k.as_str().map(str::to_string)).collect();
                        let ans = p["answer_ja"].as_str().or_else(|| p["answer"].as_str())?.to_string();
                        (!kw.is_empty() && !ans.is_empty()).then_some((kw, ans))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn alive(&self) -> bool {
        self.agent.post(&format!("{}/aruaru_llm.v1.Llm/Runtime", self.base)).send_json(json!({})).is_ok()
    }

    /// 画面(PNG)をOCRして行を連結した文字列を返す。
    pub fn ocr_png(&self, png: &[u8]) -> Option<String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(png);
        let v = self.post("/aruaru_llm.v1.Llm/Ocr", json!({"image_base64": b64, "languages": ["jpn", "eng"], "quality": "fast"}))?;
        let lines: Vec<String> = v["pages"][0]["lines"]
            .as_array()?
            .iter()
            .filter(|l| l["conf"].as_f64().unwrap_or(0.0) >= 40.0)
            .filter_map(|l| l["text"].as_str().map(str::to_string))
            .collect();
        Some(lines.join("\n"))
    }

    /// aruaru-llm が、この PC の中(127.0.0.1 / localhost)にあるか。利用者の文章を送ってよいかの判定に使う。
    pub fn is_local(&self) -> bool {
        let b = self.base.as_str();
        ["http://127.0.0.1", "http://localhost", "http://[::1]"].iter().any(|p| b.starts_with(p))
    }

    /// 時間のかかる生成(定型文の自動調査など)。待ち時間を長くとる。
    pub fn generate_long(&self, prompt: &str, max_new_tokens: usize, secs: u64) -> Option<String> {
        let v = self.post_timeout("/aruaru_llm.v1.Llm/GenerateQwen", json!({"prompt": prompt, "max_new_tokens": max_new_tokens}), secs)?;
        v["text"].as_str().map(str::to_string)
    }

    /// Qwenで続きを生成。
    pub fn generate(&self, prompt: &str, max_new_tokens: usize) -> Option<String> {
        let v = self.post("/aruaru_llm.v1.Llm/GenerateQwen", json!({"prompt": prompt, "max_new_tokens": max_new_tokens}))?;
        v["text"].as_str().map(str::to_string)
    }

    /// 検索(Web)の結果から、タイトルと抜粋を集める。検索は時間がかかるので、待ち時間は長め(45秒)。
    pub fn search_snippets(&self, query: &str, max: u8) -> Vec<String> {
        let Some(v) = self.post_timeout("/aruaru_llm.v1.Llm/SearchRaw", json!({"source": "google", "query": query, "max_results": max, "free_only": true}), 45)
        else {
            return vec![];
        };
        let arr = v.as_array().cloned().or_else(|| v["results"].as_array().cloned()).unwrap_or_default();
        arr.iter()
            .filter_map(|r| {
                let t = r["title"].as_str().unwrap_or("");
                let s = r["snippet"].as_str().or_else(|| r["description"].as_str()).unwrap_or("");
                let line = format!("{} {}", t, s).trim().to_string();
                (!line.is_empty()).then_some(line)
            })
            .collect()
    }

    /// 検索フォーム型のサジェスト(Google の公開サジェスト)。失敗時は空。
    pub fn web_suggest(&self, query: &str, lang: &str) -> Vec<String> {
        let q: String = query.chars().take(64).collect();
        let Ok(resp) = self
            .agent
            .get("https://suggestqueries.google.com/complete/search")
            .query("client", "firefox")
            .query("hl", lang)
            .query("q", &q)
            .timeout(Duration::from_secs(3))
            .call()
        else {
            return vec![];
        };
        resp.into_json::<Value>().map(|v| parse_suggest(&v)).unwrap_or_default()
    }
}

/// `["q",["a","b",...]]` 形式を取り出す。
pub fn parse_suggest(v: &Value) -> Vec<String> {
    v.get(1)
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).take(8).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_suggest() {
        let v = json!(["ねこ", ["猫 飼い方", "猫 画像"]]);
        assert_eq!(parse_suggest(&v), vec!["猫 飼い方", "猫 画像"]);
        assert!(parse_suggest(&json!({})).is_empty());
    }
}
