//! TSF テキストサービス本体: キー入力を `Session`(OS非依存の状態機械)へ渡し、結果を文書へ反映する。
use crate::{candwin, daemon};
use crate::engine::Engine;
use aruaru_ime_core::session::{Key, Step};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};
use windows::core::{implement, Interface, Result, GUID};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows_core::IUnknownImpl;
use windows::Win32::Foundation::{BOOL, E_FAIL, FALSE, LPARAM, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::TextServices::*;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

struct Inner {
    session: Engine,
    comp: Option<ITfComposition>,
    last_focus_post: Option<Instant>,
}

#[implement(ITfEditSession)]
struct EditSession {
    f: RefCell<Option<Box<dyn FnOnce(u32) -> Result<()>>>>,
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        match self.f.borrow_mut().take() {
            Some(f) => f(ec),
            None => Ok(()),
        }
    }
}

fn run_session(ctx: &ITfContext, tid: u32, flags: TF_CONTEXT_EDIT_CONTEXT_FLAGS, f: impl FnOnce(u32) -> Result<()> + 'static) -> Result<()> {
    let wrapped = move |ec: u32| {
        let r = f(ec);
        if let Err(e) = &r {
            crate::log(&format!("edit session error: {e}"));
        }
        r
    };
    let es: ITfEditSession = EditSession { f: RefCell::new(Some(Box::new(wrapped))) }.into();
    let hr = unsafe { ctx.RequestEditSession(tid, &es, flags) };
    match hr {
        Ok(h) => {
            if h.is_err() {
                crate::log(&format!("RequestEditSession hr={h:?}"));
            }
            h.ok()
        }
        Err(e) => {
            crate::log(&format!("RequestEditSession err={e}"));
            Err(e)
        }
    }
}

#[implement(ITfTextInputProcessorEx, ITfTextInputProcessor, ITfKeyEventSink, ITfThreadMgrEventSink, ITfCompositionSink, ITfActiveLanguageProfileNotifySink)]
pub struct TextService {
    inner: Rc<RefCell<Inner>>,
    tm: RefCell<Option<ITfThreadMgr>>,
    tid: Cell<u32>,
    sink_cookie: Cell<u32>,
    lang_cookie: Cell<u32>,
    comp_sink: RefCell<Option<ITfCompositionSink>>,
}

impl TextService {
    pub fn new() -> Self {
        let mut session = Engine::new();
        session.apply_settings(&aruaru_ime_core::settings::Settings::load());
        Self {
            inner: Rc::new(RefCell::new(Inner { session, comp: None, last_focus_post: None })),
            tm: RefCell::new(None),
            tid: Cell::new(0),
            sink_cookie: Cell::new(0),
            lang_cookie: Cell::new(0),
            comp_sink: RefCell::new(None),
        }
    }

    fn post_focus(&self, editable: bool) {
        let password = focused_is_password();
        let body = format!("{{\"editable\":{editable},\"password\":{password}}}");
        std::thread::spawn(move || {
            let _ = daemon::post("/focus", &body, 300);
        });
    }

    /// 入力欄にいる間、数秒ごとにフォーカス通知を再送する(デーモン側の TTL 維持)。
    fn keepalive(&self) {
        let mut g = self.inner.borrow_mut();
        if g.last_focus_post.map_or(true, |t| t.elapsed() > Duration::from_secs(5)) {
            g.last_focus_post = Some(Instant::now());
            drop(g);
            self.post_focus(true);
        }
    }

    fn read_before(&self, ctx: &ITfContext) {
        let inner = self.inner.clone();
        let tid = self.tid.get();
        let c2 = ctx.clone();
        let _ = run_session(ctx, tid, TF_ES_SYNC | TF_ES_READ, move |ec| {
            let mut sel = [TF_SELECTION::default()];
            let mut fetched = 0u32;
            unsafe { c2.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)? };
            let Some(range) = sel[0].range.take() else { return Err(E_FAIL.into()) };
            let r = unsafe { range.Clone()? };
            unsafe {
                r.Collapse(ec, TF_ANCHOR_START)?;
                let mut shifted = 0i32;
                r.ShiftStart(ec, -80, &mut shifted, std::ptr::null())?;
                let mut buf = [0u16; 128];
                let mut n = 0u32;
                r.GetText(ec, 0, &mut buf, &mut n)?;
                inner.borrow_mut().session.set_before(&String::from_utf16_lossy(&buf[..n as usize]));
            }
            Ok(())
        });
    }

    fn apply(&self, ctx: &ITfContext, step: Step) {
        let inner = self.inner.clone();
        let tid = self.tid.get();
        let c2 = ctx.clone();
        let sink: ITfCompositionSink = self.comp_sink.borrow().clone().expect("activated");
        let inner_fail = self.inner.clone();
        let r = run_session(ctx, tid, TF_ES_SYNC | TF_ES_READWRITE, move |ec| unsafe {
            let set_caret_end = |range: &ITfRange| -> Result<()> {
                let r = range.Clone()?;
                r.Collapse(ec, TF_ANCHOR_END)?;
                let sel = TF_SELECTION { range: std::mem::ManuallyDrop::new(Some(r)), style: TF_SELECTIONSTYLE { ase: TF_AE_NONE, fInterimChar: FALSE } };
                c2.SetSelection(ec, &[sel])
            };
            let mut g = inner.borrow_mut();

            if let Some(text) = &step.commit {
                if let Some(comp) = g.comp.take() {
                    let range = comp.GetRange()?;
                    range.SetText(ec, 0, &wide(text))?;
                    comp.EndComposition(ec)?;
                    set_caret_end(&range)?;
                } else {
                    let ins: ITfInsertAtSelection = c2.cast()?;
                    let range = ins.InsertTextAtSelection(ec, INSERT_TEXT_AT_SELECTION_FLAGS(0), &wide(text))?;
                    set_caret_end(&range)?;
                }
            }

            if !step.preedit.is_empty() {
                if g.comp.is_none() {
                    let mut sel = [TF_SELECTION::default()];
                    let mut fetched = 0u32;
                    c2.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
                    let range = sel[0].range.take().ok_or(E_FAIL)?;
                    let cc: ITfContextComposition = c2.cast()?;
                    let comp = cc.StartComposition(ec, &range, &sink)?;
                    g.comp = Some(comp);
                }
                let comp = g.comp.clone().ok_or(E_FAIL)?;
                let range = comp.GetRange()?;
                range.SetText(ec, 0, &wide(&step.preedit))?;
                set_caret_end(&range)?;
                if !step.cands.is_empty() {
                    if let Ok(view) = c2.GetActiveView() {
                        let mut rc = RECT::default();
                        let mut clipped = BOOL(0);
                        if view.GetTextExt(ec, &range, &mut rc, &mut clipped).is_ok() {
                            candwin::show(&step.cands, step.sel, &step.cands_en, step.col, step.sel_en, &step.mode, rc.left, rc.bottom);
                        }
                    }
                } else {
                    candwin::hide();
                }
            } else {
                if let Some(comp) = g.comp.take() {
                    comp.GetRange()?.SetText(ec, 0, &[])?;
                    comp.EndComposition(ec)?;
                }
                candwin::hide();
            }
            Ok(())
        });
        if let Err(e) = r {
            // 文書へ反映できなかった: 通常の文字が打てなくなるのを防ぐため、IME を自動でオフにして素通しにする
            crate::log(&format!("apply failed ({e}); auto-disabling IME"));
            let mut g = inner_fail.borrow_mut();
            g.comp = None;
            g.session.force_off();
            drop(g);
            candwin::hide();
            self.flash_mode(ctx, "A (自動オフ)");
        }
    }

    /// 入力モードの切替をキャレット付近に一瞬表示する。
    fn flash_mode(&self, ctx: &ITfContext, label: &'static str) {
        let tid = self.tid.get();
        let c2 = ctx.clone();
        let _ = run_session(ctx, tid, TF_ES_SYNC | TF_ES_READ, move |ec| unsafe {
            let mut sel = [TF_SELECTION::default()];
            let mut fetched = 0u32;
            c2.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?;
            let range = sel[0].range.take().ok_or(E_FAIL)?;
            let view = c2.GetActiveView()?;
            let mut rc = RECT::default();
            let mut clipped = BOOL(0);
            view.GetTextExt(ec, &range, &mut rc, &mut clipped)?;
            candwin::flash(label, rc.left, rc.bottom);
            Ok(())
        });
    }
}

fn is_mode_key(vk: u32) -> bool {
    matches!(vk, 0x19 | 0xF3 | 0xF4 | 0x16 | 0x1A | 0xF0 | 0xF2 | 0x1C | 0x1D)
}

fn map_key(vk: u32) -> Option<Key> {
    let down = |k: VIRTUAL_KEY| unsafe { GetKeyState(k.0 as i32) } < 0;
    match vk {
        0x20 => return Some(Key::Space),
        0x0D => return Some(Key::Enter),
        0x1B => return Some(Key::Esc),
        0x08 => return Some(Key::Backspace),
        0x26 => return Some(Key::Up),
        0x28 => return Some(Key::Down),
        0x09 => return Some(Key::Tab),
        0x71 => return Some(Key::Fn(2)), // F2: 候補窓の配色切替
        0x75..=0x79 => return Some(Key::Fn((vk - 0x75 + 6) as u8)),
        0x16 | 0xF2 => return Some(Key::On),  // IME オン / かなキー(押すたびに必ずオン)
        0x1A | 0xF0 => return Some(Key::Off), // IME オフ / 英数キー(押すたびに必ずオフ)
        0x25 => return Some(Key::Left),       // ←: 候補窓の日本語の列へ
        0x27 => return Some(Key::Right),      // →: 候補窓の英語の列へ
        0x1C => return Some(Key::Convert),    // 変換キー
        0x1D => return Some(Key::NonConvert), // 無変換キー
        0x19 | 0xF3 | 0xF4 => return Some(Key::Toggle), // 半角/全角
        _ => {}
    }
    if down(VK_CONTROL) || down(VK_MENU) || down(VK_LWIN) || down(VK_RWIN) {
        return None;
    }
    // Caps Lock オン中は大文字の直接入力のつもりなので、IME は文字を食わない
    if (0x41..=0x5A).contains(&vk) && (unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1) != 0 {
        return None;
    }
    let mut state = [0u8; 256];
    unsafe {
        GetKeyboardState(&mut state).ok()?;
        let mut out = [0u16; 4];
        let n = ToUnicode(vk, 0, Some(&state), &mut out, 0x4);
        if n == 1 {
            let c = char::from_u32(out[0] as u32)?;
            if !c.is_control() {
                return Some(Key::Char(c));
            }
        }
    }
    None
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Option<&ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }
    fn Deactivate(&self) -> Result<()> {
        unsafe {
            if let Some(tm) = self.tm.borrow_mut().take() {
                if let Ok(km) = tm.cast::<ITfKeystrokeMgr>() {
                    let _ = km.UnadviseKeyEventSink(self.tid.get());
                }
                if let Ok(src) = tm.cast::<ITfSource>() {
                    if self.sink_cookie.get() != 0 {
                        let _ = src.UnadviseSink(self.sink_cookie.get());
                    }
                    if self.lang_cookie.get() != 0 {
                        let _ = src.UnadviseSink(self.lang_cookie.get());
                    }
                }
            }
        }
        candwin::destroy();
        *self.comp_sink.borrow_mut() = None;
        self.inner.borrow_mut().comp = None;
        self.post_focus(false);
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Option<&ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        crate::log("ActivateEx");
        let tm = ptim.ok_or(E_FAIL)?.clone();
        self.tid.set(tid);
        unsafe {
            let km: ITfKeystrokeMgr = tm.cast()?;
            let key_sink: ITfKeyEventSink = self.as_interface::<ITfKeyEventSink>().to_owned();
            km.AdviseKeyEventSink(tid, &key_sink, TRUE_)?;
            let src: ITfSource = tm.cast()?;
            let tm_sink: ITfThreadMgrEventSink = self.as_interface::<ITfThreadMgrEventSink>().to_owned();
            let cookie = src.AdviseSink(&ITfThreadMgrEventSink::IID, &tm_sink)?;
            self.sink_cookie.set(cookie);
            // 言語ごとの入力方式が選ばれたことを知る(Win+Space で言語を切り替えたとき)
            let lang_sink: ITfActiveLanguageProfileNotifySink = self.as_interface::<ITfActiveLanguageProfileNotifySink>().to_owned();
            if let Ok(c) = src.AdviseSink(&ITfActiveLanguageProfileNotifySink::IID, &lang_sink) {
                self.lang_cookie.set(c);
            }
            // 起動時に選ばれている言語
            if let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER) {
                let mut p = TF_INPUTPROCESSORPROFILE::default();
                if mgr.GetActiveProfile(&GUID_TFCAT_TIP_KEYBOARD, &mut p).is_ok() {
                    if p.clsid == crate::CLSID_ARUARU_IME {
                        aruaru_ime_core::lastime::write(&aruaru_ime_core::components::data_dir(), aruaru_ime_core::lastime::Last::Aruaru(p.guidProfile.to_u128()));
                        if let Some(l) = aruaru_ime_core::translit::Lang::from_profile_guid(p.guidProfile.to_u128()) {
                            self.inner.borrow_mut().session.set_lang(l);
                        }
                    }
                }
            }
            *self.comp_sink.borrow_mut() = Some(self.as_interface::<ITfCompositionSink>().to_owned());
        }
        *self.tm.borrow_mut() = Some(tm);
        daemon::ensure_running(&crate::dll_dir());
        Ok(())
    }
}

const TRUE_: BOOL = BOOL(1);

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        Ok(())
    }
    fn OnTestKeyDown(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let eat = map_key(wparam.0 as u32).is_some_and(|k| self.inner.borrow().session.would_eat(&k));
        crate::log(&format!("TestKeyDown vk={:#x} eat={eat}", wparam.0));
        Ok(BOOL(eat as i32))
    }
    fn OnTestKeyUp(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        // 食ったキーダウンに対応するキーアップはアプリへ渡さない
        let eat = is_mode_key(wparam.0 as u32);
        Ok(BOOL(eat as i32))
    }
    fn OnKeyDown(&self, pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        let (Some(ctx), Some(key)) = (pic, map_key(wparam.0 as u32)) else { return Ok(BOOL(0)) };
        if !self.inner.borrow().session.would_eat(&key) {
            // 食わないキーは何もしない
            return Ok(BOOL(0));
        }
        self.keepalive();
        if !self.inner.borrow().session.composing() {
            self.read_before(ctx);
            // 先頭を Shift で打った語は、かなにせず英字のまま(一時英字入力)
            if let Key::Char(c) = &key {
                if c.is_ascii_uppercase() && unsafe { GetKeyState(VK_SHIFT.0 as i32) } < 0 {
                    self.inner.borrow_mut().session.begin_raw();
                }
            }
        }
        if key == Key::Fn(2) {
            let mut st = aruaru_ime_core::settings::Settings::load();
            let t = st.toggle_theme().to_string();
            crate::log(&format!("theme -> {t}"));
        }
        let is_mode = matches!(key, Key::On | Key::Off | Key::Toggle | Key::Convert | Key::NonConvert);
        let step = self.inner.borrow_mut().session.key(key.clone());
        crate::log(&format!("KeyDown -> {:?}", step));
        let eaten = step.eaten;
        if step.commit.is_some() || !step.preedit.is_empty() || self.inner.borrow().comp.is_some() {
            self.apply(ctx, step);
        }
        if key == Key::Fn(2) {
            candwin::redraw();
        }
        if is_mode {
            let on = self.inner.borrow().session.is_on();
            self.flash_mode(ctx, if on { "あ" } else { "A" });
        }
        Ok(BOOL(eaten as i32))
    }
    fn OnKeyUp(&self, _pic: Option<&ITfContext>, wparam: WPARAM, _lparam: LPARAM) -> Result<BOOL> {
        Ok(BOOL(is_mode_key(wparam.0 as u32) as i32))
    }
    fn OnPreservedKey(&self, _pic: Option<&ITfContext>, _rguid: *const GUID) -> Result<BOOL> {
        Ok(BOOL(0))
    }
}

impl ITfThreadMgrEventSink_Impl for TextService_Impl {
    fn OnInitDocumentMgr(&self, _p: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }
    fn OnUninitDocumentMgr(&self, _p: Option<&ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }
    fn OnSetFocus(&self, focus: Option<&ITfDocumentMgr>, _prev: Option<&ITfDocumentMgr>) -> Result<()> {
        {
            let mut g = self.inner.borrow_mut();
            g.session.apply_settings(&aruaru_ime_core::settings::Settings::load()); // 右クリックメニューでの変更を反映
            g.session.cancel();
            g.comp = None;
            g.last_focus_post = Some(Instant::now());
        }
        candwin::hide();
        self.post_focus(focus.is_some());
        Ok(())
    }
    fn OnPushContext(&self, _p: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }
    fn OnPopContext(&self, _p: Option<&ITfContext>) -> Result<()> {
        Ok(())
    }
}

impl ITfActiveLanguageProfileNotifySink_Impl for TextService_Impl {
    fn OnActivated(&self, rclsid: *const GUID, guidprofile: *const GUID, factivated: BOOL) -> Result<()> {
        unsafe {
            if factivated.as_bool() && !rclsid.is_null() && !guidprofile.is_null() {
                // 最後に選ばれた入力方式を記録(再起動後に自動で選び直すため)
                let root = aruaru_ime_core::components::data_dir();
                if *rclsid == crate::CLSID_ARUARU_IME {
                    aruaru_ime_core::lastime::write(&root, aruaru_ime_core::lastime::Last::Aruaru((*guidprofile).to_u128()));
                } else {
                    aruaru_ime_core::lastime::write(&root, aruaru_ime_core::lastime::Last::Other);
                }
            }
            if factivated.as_bool() && !rclsid.is_null() && !guidprofile.is_null() && *rclsid == crate::CLSID_ARUARU_IME {
                if let Some(l) = aruaru_ime_core::translit::Lang::from_profile_guid((*guidprofile).to_u128()) {
                    crate::log(&format!("language profile -> {}", l.code()));
                    self.inner.borrow_mut().session.set_lang(l);
                    candwin::hide();
                }
            }
        }
        Ok(())
    }
}

impl ITfCompositionSink_Impl for TextService_Impl {
    fn OnCompositionTerminated(&self, _ec: u32, _comp: Option<&ITfComposition>) -> Result<()> {
        let mut g = self.inner.borrow_mut();
        g.comp = None;
        g.session.cancel();
        candwin::hide();
        Ok(())
    }
}

/// フォーカス中のウィンドウが、パスワード欄(Win32 の Edit で ES_PASSWORD)か。画面取り込みと学習を止めるために使う。
fn focused_is_password() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetGUIThreadInfo, GetWindowLongPtrW, GUITHREADINFO, GWL_STYLE};
    const ES_PASSWORD: isize = 0x20;
    unsafe {
        let mut gi = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        if GetGUIThreadInfo(0, &mut gi).is_err() || gi.hwndFocus.0.is_null() {
            return false;
        }
        GetWindowLongPtrW(gi.hwndFocus, GWL_STYLE) & ES_PASSWORD != 0
    }
}
