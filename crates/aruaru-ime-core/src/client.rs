//! aruaru-llm(open-cuda のQwen・Tesseract OCR・検索)へのクライアント。
//! 接続できない/モデル未導入のときは全てエラーではなく `None` / 空で返し、IMEは規則だけで動き続ける。

use base64::Engine;
use serde_json::{json, Value};
use std::time::Duration;

pub const DEFAULT_URL: &str = "http://127.0.0.1:4600";

#[derive(Clone)]
pub struct Client {
    base: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base: &str) -> Self {
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(8)).build();
        Self { base: base.trim_end_matches('/').to_string(), agent }
    }

    fn post(&self, path: &str, body: Value) -> Option<Value> {
        self.agent.post(&format!("{}{}", self.base, path)).send_json(body).ok()?.into_json().ok()
    }

    fn post_timeout(&self, path: &str, body: Value, secs: u64) -> Option<Value> {
        self.agent.post(&format!("{}{}", self.base, path)).timeout(Duration::from_secs(secs)).send_json(body).ok()?.into_json().ok()
    }

    pub fn alive(&self) -> bool {
        self.agent.get(&format!("{}/v1/runtime", self.base)).call().is_ok()
    }

    /// 画面(PNG)をOCRして行を連結した文字列を返す。
    pub fn ocr_png(&self, png: &[u8]) -> Option<String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(png);
        let v = self.post("/v1/ocr", json!({"image_base64": b64, "languages": ["jpn", "eng"], "quality": "fast"}))?;
        let lines: Vec<String> = v["pages"][0]["lines"]
            .as_array()?
            .iter()
            .filter(|l| l["conf"].as_f64().unwrap_or(0.0) >= 40.0)
            .filter_map(|l| l["text"].as_str().map(str::to_string))
            .collect();
        Some(lines.join("\n"))
    }

    /// Qwenで続きを生成。
    pub fn generate(&self, prompt: &str, max_new_tokens: usize) -> Option<String> {
        let v = self.post("/v1/generate-qwen", json!({"prompt": prompt, "max_new_tokens": max_new_tokens}))?;
        v["text"].as_str().map(str::to_string)
    }

    /// 検索(Web)の結果から、タイトルと抜粋を集める。検索は時間がかかるので、待ち時間は長め(45秒)。
    pub fn search_snippets(&self, query: &str, max: u8) -> Vec<String> {
        let Some(v) = self.post_timeout("/v1/search/raw", json!({"source": "google", "query": query, "max_results": max, "free_only": true}), 45)
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
