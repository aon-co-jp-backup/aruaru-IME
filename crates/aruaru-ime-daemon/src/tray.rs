//! 通知領域(タスクトレイ)のアイコンと右クリックメニュー(Windows)。
//! 現在対応している言語・プログラミング言語・フレームワークの一覧、単語・熟語の登録・一覧・ファイルからの取り込み、
//! 未知の言語・フレームワークの追加、旧字体・ライブ変換・自動訂正・配色・候補窓の大きさ・自動アップデートの切替を出す。

use aruaru_ime_core::settings::{self, Settings};
use aruaru_ime_core::userdict::UserDict;
use aruaru_ime_core::{code, translit};
use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::Dialogs::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const WM_TRAY: u32 = WM_APP + 1;
const WM_NOTICE: u32 = WM_APP + 2;

static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
static NOTICE: Mutex<Option<(String, String)>> = Mutex::new(None);

fn wz(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// バックグラウンドの処理の結果を、通知領域の吹き出しで知らせる(どのスレッドからでも呼べる)。
pub fn notify(title: &str, msg: &str) {
    *NOTICE.lock().unwrap_or_else(|e| e.into_inner()) = Some((title.to_string(), msg.to_string()));
    let h = TRAY_HWND.load(Ordering::SeqCst);
    if h != 0 {
        unsafe {
            let _ = PostMessageW(HWND(h as *mut c_void), WM_NOTICE, WPARAM(0), LPARAM(0));
        }
    }
}

#[derive(Clone)]
enum Action {
    AddVocab,
    RegisterWord,
    ListWords,
    ImportFile,
    RemoveSource(String),
    ToggleLang(translit::Lang),
    ToggleOldForms,
    ToggleLive,
    ToggleAutocorrect,
    ToggleTheme,
    ToggleAutoUpdate,
    Scale(f32),
    Exit,
}

struct Menu {
    items: Vec<(usize, Action)>,
}

impl Menu {
    fn push(&mut self, a: Action) -> usize {
        let id = 1000 + self.items.len();
        self.items.push((id, a));
        id
    }
}

unsafe fn add(h: HMENU, flags: MENU_ITEM_FLAGS, id: usize, text: &str) {
    let t = wz(text);
    let _ = AppendMenuW(h, flags, id, PCWSTR(t.as_ptr()));
}

unsafe fn add_sub(h: HMENU, sub: HMENU, text: &str) {
    let t = wz(text);
    let _ = AppendMenuW(h, MF_POPUP, sub.0 as usize, PCWSTR(t.as_ptr()));
}

fn check(on: bool) -> MENU_ITEM_FLAGS {
    if on { MF_STRING | MF_CHECKED } else { MF_STRING }
}

unsafe fn build_menu() -> (HMENU, Menu) {
    let s = Settings::load();
    let mut m = Menu { items: vec![] };
    let root = CreatePopupMenu().unwrap();
    add(root, MF_STRING | MF_GRAYED, 1, &format!("aruaru-IME v{}", env!("CARGO_PKG_VERSION")));
    let _ = AppendMenuW(root, MF_SEPARATOR, 0, PCWSTR::null());

    // 入力言語(最大7つ選べる。日本語・英語は固定。選択中は ✓)
    let langs = CreatePopupMenu().unwrap();
    let chosen = s.selected_languages();
    add(langs, MF_STRING | MF_GRAYED, 2, &format!("選べる言語は最大{}つ / Up to {} languages", settings::MAX_LANGUAGES, settings::MAX_LANGUAGES));
    let _ = AppendMenuW(langs, MF_SEPARATOR, 0, PCWSTR::null());
    for l in translit::ALL {
        let on = chosen.contains(l);
        let fixed = matches!(l, translit::Lang::Ja | translit::Lang::En);
        let rtl = if l.rtl() { "  (RTL 右から左)" } else { "" };
        let name = if fixed || *l == translit::Lang::En { format!("{} / {}", l.native_name(), l.english_name()) } else { format!("{} / {}", l.native_name(), l.english_name()) };
        let id = m.push(Action::ToggleLang(*l));
        add(langs, if on { MF_STRING | MF_CHECKED } else { MF_STRING }, id, &format!("{name}{rtl}"));
    }
    let _ = AppendMenuW(langs, MF_SEPARATOR, 0, PCWSTR::null());
    add(langs, MF_STRING | MF_GRAYED, 3, "切替 / Switch: Win+Space");
    add_sub(root, langs, &format!("入力言語 / Input languages ({}/{})", chosen.len(), settings::MAX_LANGUAGES));

    // 対応しているプログラミング言語・フレームワーク(★ = 追加・自動学習したもの)
    let code_menu = CreatePopupMenu().unwrap();
    let list = code::list();
    for (name, n, user) in &list {
        add(code_menu, MF_STRING | MF_GRAYED, 4, &format!("{}{}  ({} words / {}語)", if *user { "★ " } else { "" }, name, n, n));
    }
    add_sub(root, code_menu, &format!("対応しているプログラミング言語・フレームワーク / Programming languages & frameworks ({})", list.len()));
    let id = m.push(Action::AddVocab);
    add(root, MF_STRING, id, "言語・フレームワークを追加… / Add a language or framework (auto-support)");
    let _ = AppendMenuW(root, MF_SEPARATOR, 0, PCWSTR::null());

    // 単語・熟語
    let id = m.push(Action::RegisterWord);
    add(root, MF_STRING, id, "単語・熟語を登録… / Register a word or phrase");
    let id = m.push(Action::ListWords);
    let count = UserDict::load(&UserDict::default_path()).entries().len();
    add(root, MF_STRING, id, &format!("登録した単語・熟語の一覧… / Registered words & phrases ({count})"));
    let id = m.push(Action::ImportFile);
    add(root, MF_STRING, id, "ファイルから登録… / Register from a file (txt / xlsx / docx / pptx / pdf)");
    let sources = UserDict::load(&UserDict::default_path()).sources();
    if !sources.is_empty() {
        let sub = CreatePopupMenu().unwrap();
        for (name, n) in sources {
            let id = m.push(Action::RemoveSource(name.clone()));
            add(sub, MF_STRING, id, &format!("{name} ({n}) を削除 / Remove"));
        }
        add_sub(root, sub, "ファイル単位で登録を削除 / Remove registrations by file");
    }
    let _ = AppendMenuW(root, MF_SEPARATOR, 0, PCWSTR::null());

    // 表示・動作
    let id = m.push(Action::ToggleOldForms);
    add(root, check(s.old_forms), id, "旧字体で表示 / Old character forms (國・學・體)");
    let id = m.push(Action::ToggleLive);
    add(root, check(s.live_conversion), id, "ライブ変換 / Live conversion (kanji as you type)");
    let id = m.push(Action::ToggleAutocorrect);
    add(root, check(s.autocorrect), id, "プログラミング用語の自動訂正 / Auto-correct code words");
    let id = m.push(Action::ToggleTheme);
    add(root, MF_STRING, id, if s.dark() { "候補窓を白背景・黒文字に / Candidate window: white bg, black text" } else { "候補窓を黒背景・白文字に / Candidate window: black bg, white text" });
    let size = CreatePopupMenu().unwrap();
    for (label, v) in [("標準 / Normal (1.0x)", 1.0f32), ("大きい / Large (1.8x, default)", 1.8), ("とても大きい / Very large (2.5x)", 2.5), ("特大 / Huge (3.2x)", 3.2)] {
        let id = m.push(Action::Scale(v));
        add(size, check((s.scale() - v).abs() < 0.05), id, label);
    }
    add_sub(root, size, "候補窓の大きさ / Candidate window size");
    let id = m.push(Action::ToggleAutoUpdate);
    add(root, check(s.auto_update), id, "自動アップデート / Auto-update (checked at startup)");
    let _ = AppendMenuW(root, MF_SEPARATOR, 0, PCWSTR::null());
    let id = m.push(Action::Exit);
    add(root, MF_STRING, id, "終了 / Exit");
    (root, m)
}

// ---------------------------------------------------------------- 入力ダイアログ

thread_local! {
    static DONE: Cell<bool> = const { Cell::new(false) };
    static RESULT: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    static EDITS: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
    static LIST: RefCell<Option<HWND>> = const { RefCell::new(None) };
    static DELETER: RefCell<Option<Box<dyn FnMut(usize) -> bool>>> = RefCell::new(None);
}

fn font_all(h: HWND) {
    unsafe {
        let f = GetStockObject(DEFAULT_GUI_FONT);
        SendMessageW(h, WM_SETFONT, WPARAM(f.0 as usize), LPARAM(1));
    }
}

fn text_of(h: HWND) -> String {
    unsafe {
        let n = GetWindowTextLengthW(h) as usize;
        let mut buf = vec![0u16; n + 1];
        let got = GetWindowTextW(h, &mut buf) as usize;
        String::from_utf16_lossy(&buf[..got])
    }
}

unsafe extern "system" fn dlg_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let id = (wp.0 & 0xFFFF) as i32;
            match id {
                1 => {
                    let vals: Vec<String> = EDITS.with(|e| e.borrow().iter().map(|h| text_of(*h)).collect());
                    RESULT.with(|r| *r.borrow_mut() = Some(vals));
                    DONE.with(|d| d.set(true));
                    let _ = DestroyWindow(hwnd);
                }
                2 => {
                    DONE.with(|d| d.set(true));
                    let _ = DestroyWindow(hwnd);
                }
                3 => {
                    // 一覧の選択項目を削除
                    if let Some(l) = LIST.with(|l| *l.borrow()) {
                        let sel = SendMessageW(l, 0x0188 /* LB_GETCURSEL */, WPARAM(0), LPARAM(0)).0;
                        if sel >= 0 {
                            let ok = DELETER.with(|d| d.borrow_mut().as_mut().map(|f| f(sel as usize)).unwrap_or(false));
                            if ok {
                                SendMessageW(l, 0x0182 /* LB_DELETESTRING */, WPARAM(sel as usize), LPARAM(0));
                            }
                        }
                    }
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            DONE.with(|d| d.set(true));
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn ensure_dlg_class() {
    unsafe {
        let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let wc = WNDCLASSW {
            lpfnWndProc: Some(dlg_proc),
            hInstance: hinst.into(),
            lpszClassName: w!("AruaruImeDialog"),
            hbrBackground: HBRUSH((5 + 1) as *mut c_void), // COLOR_WINDOW + 1
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
            ..Default::default()
        };
        RegisterClassW(&wc);
    }
}

unsafe fn child(parent: HWND, class: PCWSTR, text: &str, style: u32, x: i32, y: i32, w: i32, h: i32, id: i32) -> HWND {
    let t = wz(text);
    let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
    let hw = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class,
        PCWSTR(t.as_ptr()),
        WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style),
        x,
        y,
        w,
        h,
        parent,
        HMENU(id as isize as *mut c_void),
        hinst,
        None,
    )
    .unwrap_or_default();
    font_all(hw);
    hw
}

fn run_modal(dlg: HWND) {
    unsafe {
        let _ = ShowWindow(dlg, SW_SHOW);
        let _ = SetForegroundWindow(dlg);
        let mut msg = MSG::default();
        DONE.with(|d| d.set(false));
        while !DONE.with(|d| d.get()) && GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if !IsDialogMessageW(dlg, &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

fn create_dialog(title: &str, width: i32, height: i32) -> HWND {
    ensure_dlg_class();
    unsafe {
        let t = wz(title);
        let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_DLGMODALFRAME,
            w!("AruaruImeDialog"),
            PCWSTR(t.as_ptr()),
            WS_CAPTION | WS_SYSMENU,
            (sw - width) / 2,
            (sh - height) / 3,
            width,
            height,
            None,
            None,
            hinst,
            None,
        )
        .unwrap_or_default()
    }
}

/// 複数の入力欄を持つ小さなダイアログ。OK なら各欄の文字列、キャンセルなら None。
fn input_dialog(title: &str, labels: &[&str], hint: &str) -> Option<Vec<String>> {
    let height = 90 + labels.len() as i32 * 58 + if hint.is_empty() { 0 } else { 40 };
    let dlg = create_dialog(title, 520, height);
    EDITS.with(|e| e.borrow_mut().clear());
    RESULT.with(|r| *r.borrow_mut() = None);
    unsafe {
        let mut y = 12;
        for l in labels {
            child(dlg, w!("STATIC"), l, 0, 14, y, 480, 20, 0);
            let e = child(dlg, w!("EDIT"), "", WS_BORDER.0 | WS_TABSTOP.0 | 0x80 /* ES_AUTOHSCROLL */, 14, y + 22, 480, 26, 100 + y);
            EDITS.with(|v| v.borrow_mut().push(e));
            y += 58;
        }
        if !hint.is_empty() {
            child(dlg, w!("STATIC"), hint, 0, 14, y, 480, 36, 0);
            y += 40;
        }
        child(dlg, w!("BUTTON"), "OK", 1 /* BS_DEFPUSHBUTTON */ | WS_TABSTOP.0, 290, y + 6, 100, 30, 1);
        child(dlg, w!("BUTTON"), "キャンセル", WS_TABSTOP.0, 396, y + 6, 100, 30, 2);
        if let Some(first) = EDITS.with(|e| e.borrow().first().copied()) {
            let _ = windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(first);
        }
    }
    run_modal(dlg);
    RESULT.with(|r| r.borrow_mut().take())
}

/// 一覧ダイアログ(選択して「削除」できる)。`delete` は項目の位置を受け取り、削除できたら true を返す。
fn list_dialog(title: &str, items: &[String], delete: Option<Box<dyn FnMut(usize) -> bool>>) {
    let dlg = create_dialog(title, 760, 520);
    unsafe {
        let list = child(dlg, w!("LISTBOX"), "", WS_BORDER.0 | WS_VSCROLL.0 | WS_TABSTOP.0 | 1 /* LBS_NOTIFY */, 14, 12, 720, 420, 200);
        for it in items {
            let t = wz(it);
            SendMessageW(list, 0x0180 /* LB_ADDSTRING */, WPARAM(0), LPARAM(t.as_ptr() as isize));
        }
        LIST.with(|l| *l.borrow_mut() = Some(list));
        let has_delete = delete.is_some();
        DELETER.with(|d| *d.borrow_mut() = delete);
        if has_delete {
            child(dlg, w!("BUTTON"), "選択した項目を削除", WS_TABSTOP.0, 14, 440, 200, 30, 3);
        }
        child(dlg, w!("BUTTON"), "閉じる", 1 | WS_TABSTOP.0, 634, 440, 100, 30, 2);
    }
    run_modal(dlg);
    LIST.with(|l| *l.borrow_mut() = None);
    DELETER.with(|d| *d.borrow_mut() = None);
}

fn pick_file() -> Option<String> {
    unsafe {
        let mut buf = vec![0u16; 1024];
        let filter: Vec<u16> = "単語・熟語の一覧(txt csv xlsx docx pptx pdf)\0*.txt;*.csv;*.tsv;*.xlsx;*.docx;*.pptx;*.pdf\0すべてのファイル\0*.*\0\0".encode_utf16().collect();
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            lpstrFilter: PCWSTR(filter.as_ptr()),
            lpstrFile: PWSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER,
            ..Default::default()
        };
        if GetOpenFileNameW(&mut ofn).as_bool() {
            let n = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
            return Some(String::from_utf16_lossy(&buf[..n]));
        }
        None
    }
}

// ---------------------------------------------------------------- 動作

fn with_settings(f: impl FnOnce(&mut Settings)) {
    let mut s = Settings::load();
    f(&mut s);
    let _ = s.save();
}

fn handle(a: Action, client: &aruaru_ime_core::client::Client) -> bool {
    match a {
        Action::Exit => return false,
        Action::ToggleLang(l) => {
            let mut s = Settings::load();
            match s.toggle_language(l) {
                Ok(()) => {
                    let _ = s.save();
                }
                Err(e) => notify("aruaru-IME", &e),
            }
        }
        Action::ToggleOldForms => with_settings(|s| s.old_forms = !s.old_forms),
        Action::ToggleLive => with_settings(|s| s.live_conversion = !s.live_conversion),
        Action::ToggleAutocorrect => with_settings(|s| s.autocorrect = !s.autocorrect),
        Action::ToggleAutoUpdate => with_settings(|s| s.auto_update = !s.auto_update),
        Action::ToggleTheme => with_settings(|s| {
            s.toggle_theme();
        }),
        Action::Scale(v) => with_settings(|s| s.candidate_scale = v),
        Action::AddVocab => {
            if let Some(v) = input_dialog(
                "対応する言語・フレームワークを追加",
                &["言語・プログラミング言語・フレームワークの名前(例: Svelte / Kotlin / Laravel)"],
                "aruaru-llm の検索で資料を集め、補完・訂正に使う語彙を自動で作ります(数十秒かかります)。",
            ) {
                let name = v[0].trim().to_string();
                if !name.is_empty() {
                    notify("aruaru-IME", &format!("{name} の語彙を作っています…"));
                    crate::learn_in_background(client, name);
                }
            }
        }
        Action::RegisterWord => {
            if let Some(v) = input_dialog("単語・熟語を登録", &["読み(ひらがな。ローマ字でも可)", "表記(単語・熟語・文章)"], "例:  読み「おせわになっております」 → 表記「お世話になっております。」") {
                let mut u = UserDict::load(&UserDict::default_path());
                let msg = match u.add(&v[0], &v[1]) {
                    Ok(true) => format!("登録しました: {} → {}", v[0].trim(), v[1].trim()),
                    Ok(false) => "すでに登録されています".to_string(),
                    Err(e) => format!("登録できませんでした: {e}"),
                };
                notify("単語・熟語", &msg);
            }
        }
        Action::ListWords => {
            let path = UserDict::default_path();
            let entries: Vec<_> = UserDict::load(&path).entries().to_vec();
            let lines: Vec<String> = entries
                .iter()
                .map(|e| format!("{}  →  {}{}", e.reading, e.surface, e.source.as_deref().map(|s| format!("    [{s}]")).unwrap_or_default()))
                .collect();
            let shown = entries.clone();
            let mut removed = 0usize;
            list_dialog(
                &format!("登録した単語・熟語 ({}件)", lines.len()),
                &lines,
                Some(Box::new(move |i| {
                    let Some(e) = shown.get(i.saturating_sub(0)) else { return false };
                    let ok = UserDict::load(&path).remove(&e.reading, Some(&e.surface)).is_ok();
                    removed += usize::from(ok);
                    ok
                })),
            );
        }
        Action::ImportFile => {
            if let Some(file) = pick_file() {
                notify("単語・熟語", "ファイルを読み込んでいます…");
                std::thread::spawn(move || {
                    let dict = aruaru_ime_core::kanji::Dict::load(&crate::dict_path()).ok();
                    let mut u = UserDict::load(&UserDict::default_path());
                    let msg = match aruaru_ime_core::docimport::import_file(&mut u, std::path::Path::new(&file), dict.as_ref()) {
                        Ok(r) => format!("{} 件登録(既存 {} 件・読みを作れず除外 {} 件)", r.added, r.existing, r.no_reading),
                        Err(e) => format!("取り込めませんでした: {e}"),
                    };
                    notify("単語・熟語", &msg);
                });
            }
        }
        Action::RemoveSource(name) => {
            let n = UserDict::load(&UserDict::default_path()).remove_source(&name).unwrap_or(0);
            notify("単語・熟語", &format!("{name} 由来の登録を {n} 件削除しました"));
        }
    }
    true
}

unsafe extern "system" fn main_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_TRAY => {
            let ev = (lp.0 & 0xFFFF) as u32;
            if ev == WM_RBUTTONUP || ev == WM_LBUTTONUP || ev == WM_CONTEXTMENU {
                let (menu, items) = build_menu();
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);
                let _ = SetForegroundWindow(hwnd);
                let cmd = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, None::<*const RECT>);
                let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
                let _ = DestroyMenu(menu);
                if cmd.0 > 0 {
                    if let Some((_, action)) = items.items.iter().find(|(id, _)| *id == cmd.0 as usize) {
                        let keep = CLIENT.with(|c| c.borrow().as_ref().map(|c| handle(action.clone(), c)).unwrap_or(true));
                        if !keep {
                            let _ = DestroyWindow(hwnd);
                        }
                    }
                }
            }
            LRESULT(0)
        }
        WM_NOTICE => {
            if let Some((t, m)) = NOTICE.lock().unwrap_or_else(|e| e.into_inner()).take() {
                balloon(hwnd, &t, &m);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let mut nid = NOTIFYICONDATAW { cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32, hWnd: hwnd, uID: 1, ..Default::default() };
            let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

thread_local! {
    static CLIENT: RefCell<Option<aruaru_ime_core::client::Client>> = const { RefCell::new(None) };
}

fn fill(dst: &mut [u16], s: &str) {
    let v: Vec<u16> = s.encode_utf16().take(dst.len() - 1).collect();
    dst[..v.len()].copy_from_slice(&v);
    dst[v.len()] = 0;
}

unsafe fn balloon(hwnd: HWND, title: &str, msg: &str) {
    let mut nid = NOTIFYICONDATAW { cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32, hWnd: hwnd, uID: 1, uFlags: NIF_INFO, dwInfoFlags: NIIF_INFO, ..Default::default() };
    fill(&mut nid.szInfoTitle, title);
    fill(&mut nid.szInfo, msg);
    let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
}

/// 通知領域のアイコンを作り、メッセージループを回す(終了メニューが選ばれるまで戻らない)。
pub fn run(client: aruaru_ime_core::client::Client) {
    CLIENT.with(|c| *c.borrow_mut() = Some(client));
    unsafe {
        let hinst = GetModuleHandleW(PCWSTR::null()).unwrap_or_default();
        let wc = WNDCLASSW { lpfnWndProc: Some(main_proc), hInstance: hinst.into(), lpszClassName: w!("AruaruImeTray"), ..Default::default() };
        RegisterClassW(&wc);
        let hwnd = match CreateWindowExW(WINDOW_EX_STYLE(0), w!("AruaruImeTray"), w!("aruaru-IME"), WS_OVERLAPPED, 0, 0, 0, 0, None, None, hinst, None) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("tray window failed: {e}");
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600)); // アイコン無しでも変換APIは動かし続ける
                }
            }
        };
        TRAY_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAY,
            hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
            ..Default::default()
        };
        fill(&mut nid.szTip, "aruaru-IME(右クリックでメニュー)");
        let ok = Shell_NotifyIconW(NIM_ADD, &mut nid).as_bool();
        eprintln!("tray icon added: {ok}");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    std::process::exit(0);
}
