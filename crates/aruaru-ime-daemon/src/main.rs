//! aruaru-ime-daemon: 常駐エンジン。
//!   GET  /health                 → "ok"
//!   POST /focus   {"editable":bool,"password":bool}   OSのIME部品が入力欄のフォーカス変化を通知
//!   POST /convert {"buffer","before","after","field_kind"} → Conversion(JSON)
//! 入力欄にフォーカスがある間だけ、数秒に一度画面をキャプチャして(一時フォルダ・60秒/30枚で自動削除)
//! aruaru-llm の OCR に渡し、得た画面上の文字を入力モード予測・変換の文脈にする。パスワード欄では撮らない。

mod capture;
#[cfg(windows)]
mod tray;

use aruaru_ime_core::capture_store::CaptureStore;
use aruaru_ime_core::client::{Client, DEFAULT_URL};
use aruaru_ime_core::kanji::Dict;
use aruaru_ime_core::session::{Converter, Key, Session};
use aruaru_ime_core::{convert, Context, Conversion};
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

/// 入力セッション用の変換器。画面の文字(OCR)・パスワード欄の状態を、そのときの最新のものから取る。
struct EngineConv {
    dict: Arc<Option<Dict>>,
    client: Client,
    state: Arc<Mutex<State>>,
    gloss: Arc<std::sync::RwLock<Option<aruaru_ime_core::gloss::Glossary>>>,
}

impl EngineConv {
    fn ctx(&self, before: &str, field_kind: Option<&str>) -> (Context, bool) {
        let (screen, pw) = {
            let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
            (s.screen_text.clone(), s.password)
        };
        let field = if pw { Some("password".to_string()) } else { field_kind.map(str::to_string) };
        let password = field.as_deref() == Some("password");
        (Context { before: before.to_string(), after: String::new(), screen_text: screen, field_kind: field, prefer_kanji: aruaru_ime_core::settings::Settings::load().prefer_kanji }, password)
    }
}

impl Converter for EngineConv {
    fn convert(&self, buffer: &str, before: &str, field_kind: Option<&str>) -> Conversion {
        let (ctx, password) = self.ctx(before, field_kind);
        if !password {
            auto_learn(&self.client, &ctx);
        }
        convert(buffer, &ctx, if password { None } else { Some(&self.client) }, self.dict.as_ref().as_ref())
    }

    fn to_english(&self, surface: &str, reading: &str) -> Vec<String> {
        self.gloss.read().ok().and_then(|g| g.as_ref().map(|g| g.lookup(surface, reading))).unwrap_or_default()
    }

    fn to_japanese(&self, word: &str) -> Vec<String> {
        self.gloss.read().ok().and_then(|g| g.as_ref().map(|g| g.to_japanese(word))).unwrap_or_default()
    }

    /// 変換を求められたときだけ、Web のサジェストから固有名詞・作品名の候補を取る(パスワード欄では何もしない)。
    fn extra(&self, buffer: &str, before: &str) -> Vec<String> {
        let (_, password) = self.ctx(before, None);
        if password || !aruaru_ime_core::settings::Settings::load().web_suggest {
            return vec![];
        }
        let reading = aruaru_ime_core::romaji::to_hiragana(buffer);
        if reading.chars().count() < 3 {
            return vec![];
        }
        let mut out: Vec<String> = vec![];
        for s in self.client.web_suggest(&reading, "ja") {
            // 検索語の「読み + 続き」ではなく、読みに対応する題名そのものだけを拾う: 読みで始まらず、長すぎないもの
            let t = s.trim().to_string();
            if t.chars().count() <= 40 && !t.starts_with(&reading) && !out.contains(&t) {
                out.push(t);
            }
            if out.len() >= 4 {
                break;
            }
        }
        out
    }
}

type Sessions = Mutex<std::collections::HashMap<String, (Session, Instant)>>;

fn sessions() -> &'static Sessions {
    static S: std::sync::OnceLock<Sessions> = std::sync::OnceLock::new();
    S.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

#[derive(Deserialize)]
struct KeyReq {
    sid: String,
    key: Key,
    /// カーソル直前の文章(組み立て開始時に DLL が読んだもの)。
    #[serde(default)]
    before: Option<String>,
    /// 先頭を Shift で打った(一時英字入力)。
    #[serde(default)]
    raw: bool,
}

#[derive(Deserialize)]
struct SessReq {
    sid: String,
    op: String,
    #[serde(default)]
    lang: Option<String>,
}

/// セッションを取り出して `f` を実行する。無ければ作る。15分使われていないセッションは捨てる。
fn with_session<T>(sid: &str, conv: impl FnOnce() -> EngineConv, f: impl FnOnce(&mut Session) -> T) -> T {
    let mut g = sessions().lock().unwrap_or_else(|e| e.into_inner());
    g.retain(|_, (_, t)| t.elapsed() < Duration::from_secs(900));
    let entry = g.entry(sid.to_string()).or_insert_with(|| {
        let mut s = Session::new(Box::new(conv()));
        let root = aruaru_ime_core::components::data_dir();
        s.enable_learning(root.join("learned.json"));
        s.enable_user_dict(&root.join("user_dict.json"));
        s.apply_settings(&aruaru_ime_core::settings::Settings::load());
        (s, Instant::now())
    });
    entry.1 = Instant::now();
    f(&mut entry.0)
}

#[derive(Deserialize)]
struct VocabReq {
    name: String,
}

/// 未知の言語・フレームワークの語彙を、バックグラウンドで aruaru-llm の検索から作る(同時に1件、同じ名前は1回だけ)。
pub(crate) fn learn_in_background(client: &Client, name: String) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUSY: AtomicBool = AtomicBool::new(false);
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    let client = client.clone();
    std::thread::spawn(move || {
        match aruaru_ime_core::uservocab::learn(&client, &name) {
            Ok((p, n)) => {
                eprintln!("learned {name}: {n} words -> {}", p.display());
                #[cfg(windows)]
                tray::notify("aruaru-IME", &format!("{name} に対応しました({n}語)"));
            }
            Err(e) => {
                eprintln!("learn {name} failed: {e}");
                #[cfg(windows)]
                tray::notify("aruaru-IME", &format!("{name} に対応できませんでした: {e}"));
            }
        }
        BUSY.store(false, Ordering::SeqCst);
    });
}

/// 画面・直前の文章に `npm install svelte` のような記述があり、まだ知らない名前なら、自動で学習を始める(1時間に最大5件)。
fn auto_learn(client: &Client, ctx: &Context) {
    use std::collections::HashSet;
    use std::sync::Mutex;
    static TRIED: Mutex<Option<(HashSet<String>, Vec<Instant>)>> = Mutex::new(None);
    let text = format!("{}\n{}", ctx.before, ctx.screen_text);
    let mut g = TRIED.lock().unwrap_or_else(|e| e.into_inner());
    let (tried, times) = g.get_or_insert_with(|| (HashSet::new(), Vec::new()));
    times.retain(|t| t.elapsed() < Duration::from_secs(3600));
    for name in aruaru_ime_core::uservocab::install_targets(&text) {
        if times.len() >= 5 || tried.contains(&name) || aruaru_ime_core::code::knows(&name) {
            continue;
        }
        tried.insert(name.clone());
        times.push(Instant::now());
        learn_in_background(client, name);
        break;
    }
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

/// 辞書(dict.bin)を探す候補を、優先順に返す。作業フォルダに依存しない場所を先にする(IMEが自動起動すると作業フォルダが違うため)。
pub(crate) fn dict_candidates() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = vec![];
    if let Ok(p) = std::env::var("ARUARU_IME_DICT") {
        v.push(p.into());
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        v.push(dir.join("dict.bin"));
        v.push(dir.join("data").join("dict.bin"));
        // 開発時: <リポジトリ>/target*/release/ の2つ上の data/
        if let Some(root) = dir.parent().and_then(|p| p.parent()) {
            v.push(root.join("data").join("dict.bin"));
        }
    }
    v.push(aruaru_ime_core::components::data_dir().join("dict.bin"));
    v.push(PathBuf::from("data/dict.bin"));
    v
}

pub(crate) fn dict_path() -> PathBuf {
    let c = dict_candidates();
    c.iter().find(|p| p.is_file()).cloned().unwrap_or_else(|| c.into_iter().next().unwrap_or_else(|| PathBuf::from("dict.bin")))
}

#[allow(dead_code)]
fn dict_path_old() -> PathBuf {
    if let Ok(p) = std::env::var("ARUARU_IME_DICT") {
        return p.into();
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_default();
    let beside = exe_dir.join("dict.bin");
    if beside.is_file() {
        return beside;
    }
    let shared = aruaru_ime_core::components::data_dir().join("dict.bin");
    if shared.is_file() {
        return shared;
    }
    PathBuf::from("data/dict.bin")
}

/// 更新で入れ替わるとき、前のエンジンが終了するまで待ってポートを取り直す。
fn bind_with_retry() -> tiny_http::Server {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match tiny_http::Server::http(format!("127.0.0.1:{PORT}")) {
            Ok(s) => return s,
            Err(e) if Instant::now() < deadline => {
                eprintln!("port busy ({e}); retrying");
                std::thread::sleep(Duration::from_millis(400));
            }
            Err(e) => {
                // ポートを取れないなら、プロセスごと終了する(アイコンだけ残って応答しない状態を作らない。IME側が起動し直す)
                eprintln!("bind 127.0.0.1:{PORT} failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// 英語訳の辞書(JMdict)を読み込む。無ければ、起動時に自動で取得して辞書化する(取得できなくても IME は訳語なしで動く)。
fn spawn_glossary(slot: Arc<std::sync::RwLock<Option<aruaru_ime_core::gloss::Glossary>>>) {
    std::thread::spawn(move || {
        let root = aruaru_ime_core::components::data_dir();
        let path = aruaru_ime_core::gloss::glossary_path(&root);
        if !path.is_file() {
            eprintln!("glossary: {:?}", aruaru_ime_core::gloss::ensure(&root, true));
        }
        if let Some(g) = aruaru_ime_core::gloss::Glossary::load(&path) {
            eprintln!("glossary loaded: {} entries", g.len());
            if let Ok(mut w) = slot.write() {
                *w = Some(g);
            }
        }
    });
}

/// エンジンの起動と同時に、オンライン画面を最小化したブラウザで開く(設定 `open_online_ui`、Windows)。
/// デスクトップが整うまで待ち、easy-web.tokyo に届かないとき(オフライン)は開かない。
fn spawn_online_ui() {
    #[cfg(windows)]
    std::thread::spawn(|| {
        use std::net::ToSocketAddrs;
        std::thread::sleep(std::time::Duration::from_secs(20));
        if !aruaru_ime_core::settings::Settings::load().open_online_ui {
            return;
        }
        let reachable = ("easy-web.tokyo", 443u16).to_socket_addrs().ok().and_then(|mut a| a.next()).is_some_and(|a| std::net::TcpStream::connect_timeout(&a, std::time::Duration::from_secs(3)).is_ok());
        if !reachable {
            return;
        }
        use windows::core::{w, PCWSTR};
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWMINNOACTIVE;
        unsafe {
            ShellExecuteW(None, w!("open"), w!("https://easy-web.tokyo/ime"), PCWSTR::null(), PCWSTR::null(), SW_SHOWMINNOACTIVE);
        }
    });
}

/// 中国語のピンイン辞書を読み込む。無ければ、起動時に自動で取得して辞書化する。
fn spawn_zh() {
    std::thread::spawn(|| {
        let root = aruaru_ime_core::components::data_dir();
        let path = aruaru_ime_core::zh::zh_path(&root);
        if !path.is_file() {
            eprintln!("zh: {:?}", aruaru_ime_core::zh::ensure(&root, true));
        }
        if let Some(d) = aruaru_ime_core::zh::ZhDict::load(&path) {
            eprintln!("zh loaded: {} keys", d.len());
            aruaru_ime_core::zh::set_global(d);
        }
    });
}

/// 希望用語を、バックグラウンドで検索して辞書にする(同時に1件。結果は通知領域の吹き出しで知らせる)。
pub(crate) fn run_wish(client: &Client, term: String, cat: aruaru_ime_core::wish::Category) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUSY: AtomicBool = AtomicBool::new(false);
    if BUSY.swap(true, Ordering::SeqCst) {
        #[cfg(windows)]
        tray::notify("aruaru-IME", "別の用語を調べている最中です。終わってからもう一度お試しください(一覧に記録は残っています)");
        return;
    }
    let client = client.clone();
    std::thread::spawn(move || {
        let root = aruaru_ime_core::components::data_dir();
        let index = Dict::load(&dict_path()).ok().map(|d| d.surface_index());
        let r = aruaru_ime_core::wish::research(&client, &root, &term, cat, index.as_ref());
        let _msg = match &r {
            Ok(n) => format!("{term}: 辞書を作りました({n}語)"),
            Err(e) => format!("{term}: 作れませんでした — {e}"),
        };
        #[cfg(windows)]
        tray::notify("用語辞書", &_msg);
        BUSY.store(false, Ordering::SeqCst);
    });
}

/// 更新で入れ替わった新しい版のエンジンを起動し、自分は終了する(新しいエンジンがポートを引き継ぐ)。
pub(crate) fn swap_engine(dir: &std::path::Path) {
    let exe = dir.join("aruaru-ime-daemon.exe");
    let me = std::env::current_exe().ok();
    if exe.is_file() && me.as_deref() != Some(exe.as_path()) {
        let mut c = std::process::Command::new(&exe);
        c.current_dir(dir);
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut c, 0x0800_0000);
        if c.spawn().is_ok() {
            std::thread::sleep(Duration::from_millis(800));
            std::process::exit(0);
        }
    }
}

/// 自動バックアップ: 起動1分後と、その後6時間おき。内容が前回から変わっていたときだけ書く(設定 auto_backup)。
fn spawn_autobackup() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(60));
        loop {
            let s = aruaru_ime_core::settings::Settings::load();
            if s.auto_backup {
                let root = aruaru_ime_core::components::data_dir();
                let (dir, in_drive) = aruaru_ime_core::backup::backup_dir(s.backup_dir.as_deref());
                match aruaru_ime_core::backup::write_backup(&root, &dir, false) {
                    Ok(Some(p)) => {
                        eprintln!("backup: {}", p.display());
                        #[cfg(windows)]
                        if !in_drive {
                            tray::notify("バックアップ", "Google Drive のデスクトップ版が見つからないため、ドキュメントの aruaru-IME-backup に保存しました");
                        }
                        let _ = in_drive;
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("backup failed: {e}"),
                }
            }
            std::thread::sleep(Duration::from_secs(6 * 3600));
        }
    });
}

/// 途中で終わった・検索が使えなかった希望用語(未了)を、起動40秒後と、その後20分ごとに1件ずつやり直す。
fn resume_wishes(client: Client) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(40));
        loop {
            let root = aruaru_ime_core::components::data_dir();
            if let Some(w) = aruaru_ime_core::wish::load(&root).into_iter().find(|w| w.status == aruaru_ime_core::wish::Status::Pending) {
                run_wish(&client, w.term, w.category);
            }
            std::thread::sleep(Duration::from_secs(20 * 60));
        }
    });
}

/// 自動アップデート: **起動時の1回だけ**最新版を確認し、新しければ入れ替える(設定 auto_update)。定期的な確認はしない。
fn spawn_updater() {
    std::thread::spawn(|| {
        // 起動処理(辞書の読み込み・最初の入力)を邪魔しないよう、少し待ってから1回だけ確認する
        std::thread::sleep(Duration::from_secs(15));
        {
            if aruaru_ime_core::settings::Settings::load().auto_update {
                let root = aruaru_ime_core::components::data_dir().join("app");
                match aruaru_ime_core::update::run_once(aruaru_ime_core::update::REPO, &root) {
                    Ok(Some((tag, dir))) => {
                        eprintln!("updated to {tag}: {}", dir.display());
                        let exe = dir.join("aruaru-ime-daemon.exe");
                        let me = std::env::current_exe().ok();
                        if exe.is_file() && me.as_deref() != Some(exe.as_path()) {
                            let mut c = std::process::Command::new(&exe);
                            c.current_dir(&dir);
                            #[cfg(windows)]
                            std::os::windows::process::CommandExt::creation_flags(&mut c, 0x0800_0000);
                            if c.spawn().is_ok() {
                                std::thread::sleep(Duration::from_millis(800));
                                std::process::exit(0); // 新しいエンジンがポートを引き継ぐ
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("update check failed: {e}"),
                }
            }
        }
    });
}

fn main() {
    let llm_url = std::env::var("ARUARU_LLM_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let client = Client::new(&llm_url);
    let dict = Dict::load(&dict_path()).map_err(|e| eprintln!("dictionary NOT loaded ({}): {e}\n  searched: {:?}", dict_path().display(), dict_candidates())).ok();
    eprintln!("dictionary: {}", if dict.is_some() { dict_path().display().to_string() } else { "none".into() });
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

    // 変換 API(HTTP)は別スレッド、メインスレッドは Windows では通知領域アイコン(右クリックメニュー)を担当する
    let gloss: Arc<std::sync::RwLock<Option<aruaru_ime_core::gloss::Glossary>>> = Arc::new(std::sync::RwLock::new(None));
    spawn_glossary(Arc::clone(&gloss));
    spawn_zh();
    spawn_online_ui();
    spawn_autobackup();
    resume_wishes(client.clone());
    let (c2, d2, s2, g2) = (client.clone(), Arc::clone(&dict), Arc::clone(&state), Arc::clone(&gloss));
    #[cfg(windows)]
    {
        std::thread::spawn(move || {
            // HTTP が何らかの理由で止まったら、プロセスごと終了して IME 側に起動し直してもらう
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| serve(c2, d2, s2, g2)));
            eprintln!("http server stopped: {r:?}");
            std::process::exit(1);
        });
        tray::run(client);
    }
    #[cfg(not(windows))]
    {
        let _ = &client;
        serve(c2, d2, s2, g2);
    }
}

fn serve(client: Client, dict: Arc<Option<Dict>>, state: Arc<Mutex<State>>, gloss: Arc<std::sync::RwLock<Option<aruaru_ime_core::gloss::Glossary>>>) {
    let server = bind_with_retry();
    spawn_updater();
    println!("aruaru-ime-daemon listening on 127.0.0.1:{PORT} (dict: {} entries)", dict.as_ref().as_ref().map_or(0, |d| d.len()));
    for mut req in server.incoming_requests() {
        let mut body = String::new();
        let _ = std::io::Read::read_to_string(req.as_reader(), &mut body);
        let path = req.url().to_string();
        // open-easy-web(https://easy-web.tokyo)のオンライン画面からの接続だけを許可する(CORS + Private Network Access)。
        let origin = req.headers().iter().find(|h| h.field.equiv("Origin")).map(|h| h.value.as_str().to_string());
        let allowed = origin.as_deref().filter(|o| origin_allowed(o)).map(String::from);
        if req.method().as_str() == "OPTIONS" {
            let mut r = tiny_http::Response::from_string("").with_status_code(if allowed.is_some() { 204 } else { 403 });
            add_cors(&mut r, allowed.as_deref());
            let _ = req.respond(r);
            continue;
        }
        let (code, out) = match (req.method().as_str(), path.as_str()) {
            ("GET", "/ui") => (200, open_runo_crossui::page_html(&open_runo_crossui::ime_app("", env!("CARGO_PKG_VERSION")), "")),
            ("GET", "/health") => (200, format!("ok dict={} v={} proto=1", dict.as_ref().as_ref().map_or(0, |d| d.len()), env!("CARGO_PKG_VERSION"))),
            ("GET", "/stats") => {
                let s = state.lock().unwrap();
                (200, serde_json::json!({"captures": s.captures, "last_capture_ms": s.last_capture_ms, "screen_chars": s.screen_text.chars().count(), "editable": s.editable, "password": s.password}).to_string())
            }
            ("POST", "/key") => match serde_json::from_str::<KeyReq>(&body) {
                Ok(r) => {
                    let mk = || EngineConv { dict: Arc::clone(&dict), client: client.clone(), state: Arc::clone(&state), gloss: Arc::clone(&gloss) };
                    let step = with_session(&r.sid, mk, |s| {
                        if let Some(b) = &r.before {
                            s.set_before(b);
                        }
                        if r.raw {
                            s.begin_raw();
                        }
                        s.key(r.key)
                    });
                    (200, serde_json::to_string(&step).unwrap_or_default())
                }
                Err(e) => (400, e.to_string()),
            },
            ("POST", "/session") => match serde_json::from_str::<SessReq>(&body) {
                Ok(r) => {
                    let mk = || EngineConv { dict: Arc::clone(&dict), client: client.clone(), state: Arc::clone(&state), gloss: Arc::clone(&gloss) };
                    if r.op == "drop" {
                        sessions().lock().unwrap_or_else(|e| e.into_inner()).remove(&r.sid);
                        (200, "{}".to_string())
                    } else {
                        let step = with_session(&r.sid, mk, |s| {
                            match r.op.as_str() {
                                "cancel" => s.cancel(),
                                "settings" => s.apply_settings(&aruaru_ime_core::settings::Settings::load()),
                                "lang" => {
                                    if let Some(l) = r.lang.as_deref().and_then(aruaru_ime_core::translit::Lang::from_code) {
                                        s.set_lang(l);
                                    }
                                }
                                "off" => {
                                    s.cancel();
                                    s.on = false;
                                }
                                _ => {}
                            }
                            s.snapshot()
                        });
                        (200, serde_json::to_string(&step).unwrap_or_default())
                    }
                }
                Err(e) => (400, e.to_string()),
            },
            ("GET", "/version") => (200, serde_json::json!({"version": env!("CARGO_PKG_VERSION"), "proto": 1}).to_string()),
            ("GET", "/vocab/list") => {
                let v: Vec<serde_json::Value> = aruaru_ime_core::code::list().into_iter().map(|(n, c, u)| serde_json::json!({"name": n, "words": c, "user": u})).collect();
                (200, serde_json::to_string(&v).unwrap_or_default())
            }
            ("POST", "/vocab/learn") => match serde_json::from_str::<VocabReq>(&body) {
                Ok(v) => {
                    learn_in_background(&client, v.name.clone());
                    (202, serde_json::json!({"accepted": v.name}).to_string())
                }
                Err(e) => (400, e.to_string()),
            },
            ("POST", "/vocab/remove") => match serde_json::from_str::<VocabReq>(&body) {
                Ok(v) => (200, serde_json::json!({"removed": aruaru_ime_core::uservocab::remove(&v.name).unwrap_or(false)}).to_string()),
                Err(e) => (400, e.to_string()),
            },
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
                    let ctx = Context { before: r.before, after: r.after, screen_text: screen, field_kind: if pw { Some("password".into()) } else { r.field_kind }, prefer_kanji: aruaru_ime_core::settings::Settings::load().prefer_kanji };
                    if ctx.field_kind.as_deref() != Some("password") {
                        auto_learn(&client, &ctx);
                    }
                    let alive = if ctx.field_kind.as_deref() == Some("password") { None } else { Some(&client) };
                    let res = convert(&r.buffer, &ctx, alive, dict.as_ref().as_ref());
                    (200, serde_json::to_string(&res).unwrap_or_default())
                }
                Err(e) => (400, e.to_string()),
            },
            _ => (404, "not found".into()),
        };
        let mut resp = tiny_http::Response::from_string(out).with_status_code(code);
        let ctype: &[u8] = if path == "/ui" { b"text/html; charset=utf-8" } else { b"application/json; charset=utf-8" };
        resp.add_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], ctype).unwrap());
        add_cors(&mut resp, allowed.as_deref());
        let _ = req.respond(resp);
    }
}

/// 許可する接続元: easy-web.tokyo(HTTPS)と、このPC自身。
fn origin_allowed(o: &str) -> bool {
    matches!(o, "https://easy-web.tokyo" | "https://www.easy-web.tokyo" | "http://127.0.0.1:4660" | "http://localhost:4660")
}

fn add_cors<R: std::io::Read>(r: &mut tiny_http::Response<R>, origin: Option<&str>) {
    let Some(o) = origin else { return };
    for (k, v) in [("Access-Control-Allow-Origin", o), ("Vary", "Origin"), ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"), ("Access-Control-Allow-Headers", "content-type"), ("Access-Control-Allow-Private-Network", "true")] {
        r.add_header(tiny_http::Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap());
    }
}

#[cfg(test)]
mod cors_tests {
    use super::origin_allowed;

    #[test]
    fn only_easy_web_and_self_are_allowed() {
        assert!(origin_allowed("https://easy-web.tokyo") && origin_allowed("http://127.0.0.1:4660"));
        for bad in ["https://evil.example", "http://easy-web.tokyo", "https://easy-web.tokyo.evil.com", "null", ""] {
            assert!(!origin_allowed(bad), "{bad}");
        }
    }
}
