//! aruaru-IME の Android キーボード。
//!
//! - 変換はサーバー(easy-web.tokyo)のエンジンで行う。この crate は、その通信と、端末の中だけの定型文の学習を受け持つ。
//! - 画面(キーボード)は RPoem(`open-runo-crossui` / `open-runo-view`)で生成する HTML。Java は WebView に載せて、
//!   入力欄の操作(確定・組み立て中・削除)を中継するだけ。
//! - 定型文(メルカリ・ラクマ・ヤフオク・Yahoo!フリマ)と、確定した長い文章の自動学習は `aruaru-ime-core` の `templates` を使う。
//!   学習した文章は端末のアプリ領域にだけ置き、サーバーには送らない。

pub mod jni_glue;
pub mod ui;

use aruaru_ime_core::templates::{self, Site};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// 変換サーバーの場所。
pub const DEFAULT_UPSTREAM: &str = "https://easy-web.tokyo/ime/api";

pub struct Engine {
    root: PathBuf,
    upstream: String,
    sid: String,
    agent: ureq::Agent,
    site: Option<Site>,
    secret: bool,
    last_reading: String,
}

impl Engine {
    pub fn new(root: &Path) -> Self {
        Self::with_upstream(root, DEFAULT_UPSTREAM)
    }

    pub fn with_upstream(root: &Path, upstream: &str) -> Self {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        Self {
            root: root.to_path_buf(),
            upstream: upstream.trim_end_matches('/').to_string(),
            sid: format!("and-{:x}", nanos & 0xffff_ffff_ffff),
            agent: ureq::AgentBuilder::new().timeout_connect(Duration::from_secs(4)).timeout(Duration::from_secs(8)).build(),
            site: None,
            secret: false,
            last_reading: String::new(),
        }
    }

    /// 入力先が変わったとき。アプリ名(パッケージ名)からフリマ・オークションかを見分ける。パスワード欄では何も出さず、学習もしない。
    /// 戻り値はサイト名(違えば空)。
    pub fn set_field(&mut self, package: &str, secret: bool) -> String {
        self.secret = secret;
        self.last_reading.clear();
        self.site = if secret { None } else { templates::detect_site(package) };
        self.site.map(|s| s.name().to_string()).unwrap_or_default()
    }

    fn post(&self, path: &str, body: Value) -> Option<Value> {
        self.agent.post(&format!("{}{}", self.upstream, path)).send_json(body).ok()?.into_json().ok()
    }

    /// 画面(WebView のページ)からの呼び出し。`path` は `/key` `/session` `/template`、`body` は JSON、`before` はカーソル直前の文章。
    /// 戻り値は JSON(エンジンの応答 + 定型文 `tpl`)。通信できないときは `null`。
    pub fn api(&mut self, path: &str, body: &str, before: &str) -> String {
        let req: Value = serde_json::from_str(body).unwrap_or(Value::Null);
        match path {
            "/key" => {
                let msg = json!({"sid": self.sid, "key": req["key"], "before": before});
                self.finish("/key", msg)
            }
            "/session" => {
                let mut msg = req.clone();
                msg["sid"] = json!(self.sid);
                self.finish("/session", msg)
            }
            "/template" => {
                // 定型文を選んで確定した(入力欄への挿入は画面側)。サーバーの組み立ては取り消し、使った回数を数える。
                let text = req["text"].as_str().unwrap_or("");
                let reading = req["reading"].as_str().unwrap_or("");
                let _ = self.post("/session", json!({"sid": self.sid, "op": "cancel"}));
                self.learn(reading, text);
                self.last_reading.clear();
                "{}".to_string()
            }
            _ => "null".to_string(),
        }
    }

    fn finish(&mut self, path: &str, msg: Value) -> String {
        let Some(mut r) = self.post(path, msg) else { return "null".to_string() };
        if let Some(c) = r["commit"].as_str().map(str::to_string) {
            let reading = self.last_reading.clone();
            self.learn(&reading, &c);
        }
        let composing = r["composing"].as_bool().unwrap_or(false);
        let reading = r["reading"].as_str().unwrap_or("").to_string();
        if composing && !reading.is_empty() {
            self.last_reading = reading.clone();
        }
        if composing && !self.secret && reading.chars().count() >= 2 {
            r["tpl"] = json!(self.templates_for(&reading));
        }
        r.to_string()
    }

    /// 読みに合う定型文。サイトが分かるときは、組み込み・利用者・学習した文章をすべて。それ以外は、2回以上確定した文章だけ。
    pub fn templates_for(&self, reading: &str) -> Vec<String> {
        match self.site {
            Some(site) => templates::lookup(reading, site, &templates::load_all(&self.root), 8),
            None => templates::frequent(&self.root, reading, 3),
        }
    }

    fn learn(&self, reading: &str, text: &str) {
        if !self.secret && !reading.is_empty() {
            templates::learn(&self.root, reading, text, self.site);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    /// 変換サーバーのふりをする小さな HTTP サーバー。応答を順に返す。
    fn fake_server(replies: Vec<&'static str>) -> (String, std::thread::JoinHandle<Vec<String>>) {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let url = format!("http://127.0.0.1:{}", server.server_addr().to_ip().unwrap().port());
        let h = std::thread::spawn(move || {
            let mut seen = vec![];
            for r in replies {
                let mut req = server.recv().unwrap();
                let mut b = String::new();
                req.as_reader().read_to_string(&mut b).unwrap();
                seen.push(format!("{} {}", req.url(), b));
                let resp = tiny_http::Response::from_string(r).with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap());
                req.respond(resp).unwrap();
            }
            seen
        });
        (url, h)
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-android-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn marketplace_templates_are_added_to_the_server_reply() {
        let dir = tmp("tpl");
        let (url, h) = fake_server(vec![r#"{"eaten":true,"preedit":"はっそう","commit":null,"cands":["発送"],"composing":true,"reading":"はっそう"}"#]);
        let mut e = Engine::with_upstream(&dir, &url);
        assert_eq!(e.set_field("com.kouzoh.mercari", false), "メルカリ");
        let r: Value = serde_json::from_str(&e.api("/key", r#"{"key":{"Char":"u"}}"#, "前の文")).unwrap();
        assert_eq!(r["cands"][0], "発送");
        assert!(r["tpl"].as_array().unwrap().iter().any(|t| t.as_str().unwrap().contains("発送いたしました")), "{r}");
        let seen = h.join().unwrap();
        assert!(seen[0].starts_with("/key ") && seen[0].contains("\"before\":\"前の文\"") && seen[0].contains("\"sid\":\"and-"), "{seen:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn other_apps_get_no_builtin_templates_and_secret_fields_nothing() {
        let dir = tmp("other");
        let (url, h) = fake_server(vec![
            r#"{"eaten":true,"preedit":"はっそう","commit":null,"cands":[],"composing":true,"reading":"はっそう"}"#,
            r#"{"eaten":true,"preedit":"はっそう","commit":null,"cands":[],"composing":true,"reading":"はっそう"}"#,
        ]);
        let mut e = Engine::with_upstream(&dir, &url);
        assert_eq!(e.set_field("com.example.notes", false), "");
        let r: Value = serde_json::from_str(&e.api("/key", r#"{"key":{"Char":"u"}}"#, "")).unwrap();
        assert!(r["tpl"].as_array().unwrap().is_empty(), "{r}");
        e.set_field("com.kouzoh.mercari", true);
        let r: Value = serde_json::from_str(&e.api("/key", r#"{"key":{"Char":"u"}}"#, "")).unwrap();
        assert!(r.get("tpl").is_none(), "パスワード欄では出さない: {r}");
        h.join().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn long_commits_are_learned_on_the_device_and_offered_after_two_uses() {
        let dir = tmp("learn");
        let text = "会議の日程が決まりましたので、ご連絡いたします。";
        let typing = r#"{"eaten":true,"preedit":"かいぎ","commit":null,"cands":[],"composing":true,"reading":"かいぎのにってい"}"#;
        let done = format!(r#"{{"eaten":true,"preedit":"","commit":"{text}","cands":[],"composing":false}}"#);
        let done: &'static str = Box::leak(done.into_boxed_str());
        let (url, h) = fake_server(vec![typing, done, typing, done, typing]);
        let mut e = Engine::with_upstream(&dir, &url);
        e.set_field("com.example.notes", false);
        for _ in 0..2 {
            e.api("/key", r#"{"key":{"Char":"a"}}"#, "");
            e.api("/key", r#"{"key":"Enter"}"#, "");
        }
        // 3回目の入力中は、2回使った文章が定型文として出る(サーバーへ文章は送っていない)
        let r: Value = serde_json::from_str(&e.api("/key", r#"{"key":{"Char":"a"}}"#, "")).unwrap();
        assert_eq!(r["tpl"][0], text, "{r}");
        h.join().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn offline_returns_null() {
        let dir = tmp("off");
        let mut e = Engine::with_upstream(&dir, "http://127.0.0.1:9");
        assert_eq!(e.api("/key", r#"{"key":"Space"}"#, ""), "null");
    }
}
