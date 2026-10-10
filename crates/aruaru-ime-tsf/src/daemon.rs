//! 常駐エンジン(aruaru-ime-daemon, 127.0.0.1:4660)との通信。標準ライブラリだけの最小 HTTP クライアント。
//! 繋がらないときは辞書なしのローカル変換にフォールバックし、IME は止まらない。

use aruaru_ime_core::convert::{Candidate, Conversion};
use aruaru_ime_core::mode::{Mode, Prediction};
use aruaru_ime_core::session::{Converter, LocalConverter};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const ADDR: &str = "127.0.0.1:4660";

pub fn post(path: &str, body: &str, timeout_ms: u64) -> Option<String> {
    let addr: SocketAddr = ADDR.parse().ok()?;
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(120)).ok()?;
    s.set_read_timeout(Some(Duration::from_millis(timeout_ms))).ok()?;
    s.set_write_timeout(Some(Duration::from_millis(timeout_ms))).ok()?;
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    text.split_once("\r\n\r\n").map(|(_, b)| b.to_string())
}

pub fn alive() -> bool {
    let Ok(addr) = ADDR.parse::<SocketAddr>() else { return false };
    TcpStream::connect_timeout(&addr, Duration::from_millis(120)).is_ok()
}

/// デーモンが居なければ、DLL と同じフォルダの aruaru-ime-daemon.exe を非表示で起動する。
pub fn ensure_running(dll_dir: &std::path::Path) {
    if alive() {
        return;
    }
    let exe = dll_dir.join("aruaru-ime-daemon.exe");
    if exe.is_file() {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new(exe).current_dir(dll_dir).creation_flags(0x0800_0000 | 0x0000_0008).spawn(); // CREATE_NO_WINDOW | DETACHED_PROCESS
    }
}

fn kind_static(k: &str) -> &'static str {
    match k {
        "alpha" => "alpha",
        "english" => "english",
        "hiragana" => "hiragana",
        "katakana" => "katakana",
        "number" => "number",
        "fullwidth" => "fullwidth",
        "symbol" => "symbol",
        "predict" => "predict",
        "llm" => "llm",
        _ => "dict",
    }
}

pub fn parse_conversion(v: &serde_json::Value) -> Option<Conversion> {
    let mode: Mode = serde_json::from_value(v["prediction"]["mode"].clone()).ok()?;
    let prediction = Prediction {
        mode,
        confidence: v["prediction"]["confidence"].as_f64().unwrap_or(0.0) as f32,
        english_word: v["prediction"]["english_word"].as_bool().unwrap_or(false),
        reason: v["prediction"]["reason"].as_str().unwrap_or("").to_string(),
    };
    let candidates = v["candidates"]
        .as_array()?
        .iter()
        .filter_map(|c| Some(Candidate { text: c["text"].as_str()?.to_string(), kind: kind_static(c["kind"].as_str().unwrap_or("")) }))
        .collect();
    let suggestions = v["suggestions"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
    Some(Conversion { prediction, candidates, suggestions })
}

pub struct DaemonConverter {
    fallback: LocalConverter,
}

impl DaemonConverter {
    pub fn new() -> Self {
        Self { fallback: LocalConverter { dict: None } }
    }
}

/// 常駐エンジンが落ちていたら起動し直す(10秒に1回まで)。
pub fn revive() {
    use std::sync::Mutex;
    static LAST: Mutex<Option<std::time::Instant>> = Mutex::new(None);
    let mut g = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if g.is_some_and(|t| t.elapsed() < Duration::from_secs(10)) {
        return;
    }
    *g = Some(std::time::Instant::now());
    drop(g);
    ensure_running(&crate::dll_dir());
}

impl Converter for DaemonConverter {
    fn convert(&self, buffer: &str, before: &str, field_kind: Option<&str>) -> Conversion {
        let body = serde_json::json!({"buffer": buffer, "before": before, "field_kind": field_kind}).to_string();
        if let Some(resp) = post("/convert", &body, 700) {
            if let Some(c) = serde_json::from_str::<serde_json::Value>(&resp).ok().and_then(|v| parse_conversion(&v)) {
                return c;
            }
        }
        revive();
        self.fallback.convert(buffer, before, field_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_daemon_json() {
        let v = serde_json::json!({
            "prediction": {"mode": "Hiragana", "confidence": 0.9, "english_word": false, "reason": "x"},
            "candidates": [{"text": "今日", "kind": "dict"}, {"text": "きょう", "kind": "hiragana"}],
            "suggestions": ["今日の天気"]
        });
        let c = parse_conversion(&v).unwrap();
        assert_eq!(c.candidates[0].text, "今日");
        assert_eq!(c.suggestions, vec!["今日の天気"]);
    }
}
