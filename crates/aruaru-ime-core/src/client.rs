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

    /// 検索API(Web)から語句の用例タイトル・要約を集める。
    pub fn search_snippets(&self, query: &str, max: u8) -> Vec<String> {
        let Some(v) = self.post("/v1/search/raw", json!({"source": "google", "query": query, "max_results": max, "free_only": true}))
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
}
