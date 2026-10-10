//! 入力エンジンの窓口。変換の状態管理(Session)は常駐エンジン側にあり、この DLL は「キーを送って結果を受け取る」だけ。
//! これにより、エンジンを更新・入れ替えても、起動中のアプリを再起動せずに最新の機能になる。
//! 常駐エンジンに繋がらないときは、DLL 内の簡易版(辞書なし)に切り替わり、入力は止まらない。

use crate::daemon;
use aruaru_ime_core::session::{would_eat_lang, Key, LocalConverter, Session, Step};
use aruaru_ime_core::translit::Lang;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static NEXT: AtomicU64 = AtomicU64::new(1);

pub struct Engine {
    sid: String,
    /// 通信できないとき用の簡易版(辞書なし)。
    local: Session,
    /// 直前の結果から持ち帰った状態(通信なしの「このキーを食うか」判定に使う)。
    on: bool,
    composing: bool,
    last_ja: bool,
    lang: Lang,
    /// 組み立て開始時に読んだ、カーソル直前の文章(次のキー送信に載せる)。
    pending_before: Option<String>,
    pending_raw: bool,
    /// 通信に失敗した時刻。しばらくは簡易版を使う。
    down_since: Option<Instant>,
    last_preedit: String,
}

impl Engine {
    pub fn new() -> Self {
        let mut local = Session::new(Box::new(LocalConverter { dict: None }));
        local.apply_settings(&aruaru_ime_core::settings::Settings::load());
        Self {
            sid: format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)),
            local,
            on: true,
            composing: false,
            last_ja: false,
            lang: Lang::Ja,
            pending_before: None,
            pending_raw: false,
            down_since: None,
            last_preedit: String::new(),
        }
    }

    fn remote_usable(&self) -> bool {
        self.down_since.is_none_or(|t| t.elapsed() > Duration::from_secs(3))
    }

    fn remember(&mut self, st: &Step) {
        self.on = st.on;
        self.composing = st.composing;
        self.last_ja = st.last_ja;
        if let Some(l) = Lang::from_code(&st.lang) {
            self.lang = l;
        }
        self.last_preedit = st.preedit.clone();
    }

    pub fn composing(&self) -> bool {
        self.composing
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// このキーを IME が処理するか(通信なし)。
    pub fn would_eat(&self, key: &Key) -> bool {
        would_eat_lang(key, self.on, self.composing, self.last_ja, self.lang)
    }

    pub fn set_before(&mut self, text: &str) {
        self.pending_before = Some(text.to_string());
        self.local.set_before(text);
    }

    /// 入力する言語を切り替える(Windows の言語切替で、言語ごとの入力方式が選ばれたとき)。
    pub fn set_lang(&mut self, lang: Lang) {
        if self.lang == lang {
            return;
        }
        self.lang = lang;
        self.composing = false;
        self.last_preedit.clear();
        self.local.set_lang(lang);
        let body = serde_json::json!({"sid": self.sid, "op": "lang", "lang": lang.code()}).to_string();
        let _ = daemon::post("/session", &body, 400);
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn begin_raw(&mut self) {
        self.pending_raw = true;
        self.local.begin_raw();
    }

    pub fn apply_settings(&mut self, s: &aruaru_ime_core::settings::Settings) {
        self.local.apply_settings(s);
        let body = serde_json::json!({"sid": self.sid, "op": "settings"}).to_string();
        let _ = daemon::post("/session", &body, 300);
    }

    pub fn cancel(&mut self) {
        self.local.cancel();
        self.composing = false;
        self.last_preedit.clear();
        if self.remote_usable() {
            let body = serde_json::json!({"sid": self.sid, "op": "cancel"}).to_string();
            let _ = daemon::post("/session", &body, 300);
        }
    }

    /// IME を自動でオフにする(文書へ反映できなかったとき、通常の文字が打てなくなるのを防ぐ)。
    pub fn force_off(&mut self) {
        self.local.cancel();
        self.local.on = false;
        self.on = false;
        self.composing = false;
        let body = serde_json::json!({"sid": self.sid, "op": "off"}).to_string();
        let _ = daemon::post("/session", &body, 300);
    }

    pub fn key(&mut self, key: Key) -> Step {
        if self.remote_usable() {
            let body = serde_json::json!({
                "sid": self.sid,
                "key": key,
                "before": self.pending_before,
                "raw": self.pending_raw,
            })
            .to_string();
            if let Some(resp) = daemon::post("/key", &body, 1500) {
                if let Ok(st) = serde_json::from_str::<Step>(&resp) {
                    self.pending_before = None;
                    self.pending_raw = false;
                    self.down_since = None;
                    self.remember(&st);
                    return st;
                }
            }
            // 通信に失敗: 組み立て中の文字は確定して失わないようにし、簡易版へ切り替える
            self.down_since = Some(Instant::now());
            daemon::revive();
            let saved = std::mem::take(&mut self.last_preedit);
            self.local.cancel();
            self.local.on = self.on;
            if self.composing && !saved.is_empty() {
                self.composing = false;
                return Step { eaten: true, commit: Some(saved), on: self.on, ..Default::default() };
            }
        }
        let st = self.local.key(key);
        self.remember(&st);
        st
    }
}
