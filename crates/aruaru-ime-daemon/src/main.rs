//! aruaru-ime-daemon: 常駐エンジン。
//!   GET  /health                 → "ok"
//!   POST /focus   {"editable":bool,"password":bool}   OSのIME部品が入力欄のフォーカス変化を通知
//!   POST /convert {"buffer","before","after","field_kind"} → Conversion(JSON)
//! 入力欄にフォーカスがある間だけ、数秒に一度画面をキャプチャして(一時フォルダ・60秒/30枚で自動削除)
//! aruaru-llm の OCR に渡し、得た画面上の文字を入力モード予測・変換の文脈にする。パスワード欄では撮らない。

mod capture;

use aruaru_ime_core::capture_store::CaptureStore;
use aruaru_ime_core::client::{Client, DEFAULT_URL};
use aruaru_ime_core::kanji::Dict;
use aruaru_ime_core::{convert, Context};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CAPTURE_INTERVAL: Duration = Duration::from_secs(3);
/// フォーカス通知がこれより古ければ「もう入力していない」とみなして撮らない。
const FOCUS_TTL: Duration = Duration::from_secs(15);
pub const PORT: u16 = 4660;

#[derive(Default)]
struct State {
    editable: bool,
    password: bool,
    focus_at: Option<Instant>,
    screen_text: String,
    captures: u64,
    last_capture_ms: u64,
}

#[derive(Deserialize)]
struct FocusReq {
    editable: bool,
    #[serde(default)]
    password: bool,
}

#[derive(Deserialize)]
struct DictReq {
    reading: String,
    #[serde(default)]
    surface: String,
}

#[derive(Deserialize)]
struct ConvReq {
    buffer: String,
    #[serde(default)]
    before: String,
    #[serde(default)]
    after: String,
    #[serde(default)]
    field_kind: Option<String>,
}

fn dict_path() -> PathBuf {
    if let Ok(p) = std::env::var("ARUARU_IME_DICT") {
        return p.into();
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    let beside = exe_dir.join("dict.bin");
    if beside.is_file() {
        return beside;
    }
    PathBuf::from("data/dict.bin")
}

fn main() {
    let llm_url = std::env::var("ARUARU_LLM_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let client = Client::new(&llm_url);
    let dict = Dict::load(&dict_path()).map_err(|e| eprintln!("dictionary not loaded ({}): {e}", dict_path().display())).ok();
    let dict = Arc::new(dict);
    let state = Arc::new(Mutex::new(State::default()));

    // 画面キャプチャ + OCR スレッド
    {
        let (state, client) = (Arc::clone(&state), client.clone());
        std::thread::spawn(move || {
            let mut store = match CaptureStore::new() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("capture store unavailable: {e}");
                    return;
                }
            };
            loop {
                std::thread::sleep(CAPTURE_INTERVAL);
                let active = {
                    let s = state.lock().unwrap();
                    s.editable && !s.password && s.focus_at.is_some_and(|t| t.elapsed() < FOCUS_TTL)
                };
                if !active {
                    continue;
                }
                let t = Instant::now();
                let Some(png) = capture::capture_png() else { continue };
                let _ = store.save(&png);
                let text = client.ocr_png(&png);
                let mut s = state.lock().unwrap();
                // OCR待ちの間にパスワード欄へ移っていたら捨てる
                if s.password {
                    continue;
                }
                s.captures += 1;
                s.last_capture_ms = t.elapsed().as_millis() as u64;
                if let Some(text) = text {
                    s.screen_text = text;
                }
            }
        });
    }

    let server = tiny_http::Server::http(format!("127.0.0.1:{PORT}")).expect("bind 127.0.0.1:4660");
    println!("aruaru-ime-daemon listening on 127.0.0.1:{PORT} (dict: {} entries)", dict.as_ref().as_ref().map_or(0, |d| d.len()));
    for mut req in server.incoming_requests() {
        let mut body = String::new();
        let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
        let path = req.url().to_string();
        let (code, out) = match (req.method().as_str(), path.as_str()) {
            ("GET", "/health") => (200, "ok".to_string()),
            ("GET", "/stats") => {
                let s = state.lock().unwrap();
                (200, serde_json::json!({"captures": s.captures, "last_capture_ms": s.last_capture_ms, "screen_chars": s.screen_text.chars().count(), "editable": s.editable, "password": s.password}).to_string())
            }
            ("GET", "/dict/list") => {
                let u = aruaru_ime_core::userdict::UserDict::load(&aruaru_ime_core::userdict::UserDict::default_path());
                (200, serde_json::to_string(u.entries()).unwrap_or_default())
            }
            ("POST", "/dict/add") => match serde_json::from_str::<DictReq>(&body) {
                Ok(d) => {
                    let mut u = aruaru_ime_core::userdict::UserDict::load(&aruaru_ime_core::userdict::UserDict::default_path());
                    match u.add(&d.reading, &d.surface) {
                        Ok(added) => (200, serde_json::json!({"added": added}).to_string()),
                        Err(e) => (400, e.to_string()),
                    }
                }
                Err(e) => (400, e.to_string()),
            },
            ("POST", "/dict/remove") => match serde_json::from_str::<DictReq>(&body) {
                Ok(d) => {
                    let mut u = aruaru_ime_core::userdict::UserDict::load(&aruaru_ime_core::userdict::UserDict::default_path());
                    let n = u.remove(&d.reading, if d.surface.is_empty() { None } else { Some(&d.surface) }).unwrap_or(0);
                    (200, serde_json::json!({"removed": n}).to_string())
                }
                Err(e) => (400, e.to_string()),
            },
            ("POST", "/focus") => match serde_json::from_str::<FocusReq>(&body) {
                Ok(f) => {
                    let mut s = state.lock().unwrap();
                    s.editable = f.editable;
                    s.password = f.password;
                    s.focus_at = Some(Instant::now());
                    if f.password || !f.editable {
                        s.screen_text.clear(); // 入力欄を離れたら画面の文字は持ち越さない
                    }
                    (200, "ok".into())
                }
                Err(e) => (400, e.to_string()),
            },
            ("POST", "/convert") => match serde_json::from_str::<ConvReq>(&body) {
                Ok(r) => {
                    let (screen, pw) = {
                        let s = state.lock().unwrap();
                        (s.screen_text.clone(), s.password)
                    };
                    let ctx = Context { before: r.before, after: r.after, screen_text: screen, field_kind: if pw { Some("password".into()) } else { r.field_kind } };
                    let alive = if ctx.field_kind.as_deref() == Some("password") { None } else { Some(&client) };
                    let res = convert(&r.buffer, &ctx, alive, dict.as_ref().as_ref());
                    (200, serde_json::to_string(&res).unwrap_or_default())
                }
                Err(e) => (400, e.to_string()),
            },
            _ => (404, "not found".into()),
        };
        let mut resp = tiny_http::Response::from_string(out).with_status_code(code);
        resp.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap());
        let _ = req.respond(resp);
    }
}
