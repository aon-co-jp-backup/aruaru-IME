//! 候補ウィンドウ(フォーカスを奪わない最前面の小窓)。ホストアプリの UI スレッド上で作る。
//! 左に「日本語の候補」、右に「英語の訳」の2列。左右キーで列を切り替え、上下キー(または数字キー 1〜9・0)で選ぶ。
//! 配色は既定が黒背景・白文字。設定(settings.json の theme)・見出し右のボタン・F2 で白背景・黒文字に切り替えられる。
//! アラビア語・ペルシア語・ヘブライ語などの右から左の文字は、行を右揃え・右から左の読み順で描く。

use aruaru_ime_core::settings::Settings;
use aruaru_ime_core::translit::is_rtl_text;
use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

const BASE_ROW_H: i32 = 26;
const MAX_ROWS: usize = 10;
const FLASH_TIMER: usize = 1;
/// 見出し行右端の配色切替ボタンの幅。
const BASE_BUTTON_W: i32 = 110;

fn scale() -> f32 {
    Settings::load().scale()
}

fn row_h() -> i32 {
    (BASE_ROW_H as f32 * scale()) as i32
}

fn button_w() -> i32 {
    (BASE_BUTTON_W as f32 * scale()) as i32
}

struct Palette {
    bg: u32,
    text: u32,
    header: u32,
    sel: u32,
    sel_inactive: u32,
}

/// COLORREF は 0x00BBGGRR。
fn palette(dark: bool) -> Palette {
    if dark {
        Palette { bg: 0x0000_0000, text: 0x00FF_FFFF, header: 0x00C8_C8C8, sel: 0x00A0_602A, sel_inactive: 0x0050_3018 }
    } else {
        Palette { bg: 0x00FF_FFFF, text: 0x0020_2020, header: 0x0080_6040, sel: 0x00F0_D8A8, sel_inactive: 0x00F8_ECD8 }
    }
}

#[derive(Default, Clone)]
struct View {
    cands: Vec<String>,
    sel: Option<usize>,
    en: Vec<String>,
    en_sel: Option<usize>,
    col: u8,
    header: String,
}

thread_local! {
    static STATE: RefCell<View> = RefCell::new(View::default());
    static HWND_CAND: RefCell<Option<HWND>> = const { RefCell::new(None) };
}

fn char_w(ch: char) -> i32 {
    if ch.is_ascii() { 10 } else { 19 }
}

fn text_w(s: &str) -> i32 {
    (s.chars().map(char_w).sum::<i32>() as f32 * scale()) as i32
}

fn page_items(list: &[String], sel: Option<usize>) -> (usize, Vec<(usize, &String)>) {
    let page = sel.map_or(0, |i| i / MAX_ROWS) * MAX_ROWS;
    (page, list.iter().enumerate().skip(page).take(MAX_ROWS).collect())
}

fn col_width(list: &[String], sel: Option<usize>, min: i32) -> i32 {
    let (_, items) = page_items(list, sel);
    let widest = items.iter().map(|(_, c)| text_w(c)).max().unwrap_or(0);
    (widest + (64.0 * scale()) as i32).max(min)
}

unsafe fn draw_column(hdc: HDC, list: &[String], sel: Option<usize>, active: bool, x0: i32, x1: i32, pal: &Palette) {
    let (page, items) = page_items(list, sel);
    for (row, (i, c)) in items.into_iter().enumerate() {
        let r = RECT { left: x0, top: (row as i32 + 1) * row_h(), right: x1, bottom: (row as i32 + 2) * row_h() };
        if sel == Some(i) {
            let hb = CreateSolidBrush(COLORREF(if active { pal.sel } else { pal.sel_inactive }));
            FillRect(hdc, &r, hb);
            let _ = DeleteObject(hb);
        }
        SetTextColor(hdc, COLORREF(pal.text));
        let num = (i - page + 1) % 10;
        let rtl = is_rtl_text(c);
        let mut text: Vec<u16> = if rtl { format!("{c} {num} ") } else { format!(" {num} {c}") }.encode_utf16().collect();
        let mut r2 = r;
        let flags = if rtl { DT_SINGLELINE | DT_VCENTER | DT_RIGHT | DT_RTLREADING } else { DT_SINGLELINE | DT_VCENTER | DT_LEFT };
        DrawTextW(hdc, &mut text, &mut r2, flags);
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_LBUTTONDOWN => {
            // 見出し行の右側のボタンをワンクリックで、黒背景・白文字 ⇔ 白背景・黒文字 を切り替える
            let y = ((lp.0 >> 16) & 0xFFFF) as i16 as i32;
            let x = (lp.0 & 0xFFFF) as i16 as i32;
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            if y < row_h() && x > rc.right - button_w() {
                Settings::load().toggle_theme();
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            let _ = KillTimer(hwnd, FLASH_TIMER);
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        WM_PAINT => {
            let pal = palette(Settings::load().dark());
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rc = RECT::default();
            let _ = GetClientRect(hwnd, &mut rc);
            let bg = CreateSolidBrush(COLORREF(pal.bg));
            FillRect(hdc, &rc, bg);
            let _ = DeleteObject(bg);
            let font = CreateFontW(-((18.0 * scale()) as i32), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w!("Yu Gothic UI"));
            let old = SelectObject(hdc, font);
            SetBkMode(hdc, TRANSPARENT);
            STATE.with(|s| {
                let v = s.borrow();
                // 見出し行: 予測した入力モードと、操作の案内
                SetTextColor(hdc, COLORREF(pal.header));
                let hint = if v.cands.is_empty() {
                    String::new()
                } else if v.en.is_empty() {
                    "  F6あ F7ア F9Ａ F10A".to_string()
                } else {
                    "  ←日本語 英語→  ↑↓".to_string()
                };
                let mut ht: Vec<u16> = format!(" [{}]{hint}", v.header).encode_utf16().collect();
                let mut hr = RECT { left: 0, top: 0, right: rc.right, bottom: row_h() };
                DrawTextW(hdc, &mut ht, &mut hr, DT_SINGLELINE | DT_VCENTER | DT_LEFT);
                if !v.cands.is_empty() {
                    // 配色切替ボタン(押すと反対の配色になる)
                    let to_light = Settings::load().dark();
                    let br = RECT { left: rc.right - button_w(), top: 2, right: rc.right - 4, bottom: row_h() - 2 };
                    let bb = CreateSolidBrush(COLORREF(if to_light { 0x00F0_F0F0 } else { 0x0030_3030 }));
                    FillRect(hdc, &br, bb);
                    let _ = DeleteObject(bb);
                    SetTextColor(hdc, COLORREF(if to_light { 0x0020_2020 } else { 0x00FF_FFFF }));
                    let mut bt: Vec<u16> = (if to_light { "白背景・黒文字" } else { "黒背景・白文字" }).encode_utf16().collect();
                    let mut br2 = br;
                    DrawTextW(hdc, &mut bt, &mut br2, DT_SINGLELINE | DT_VCENTER | DT_CENTER);
                }
                // 2列: 左=日本語の候補、右=英語の訳(あるときだけ)
                let left_w = col_width(&v.cands, v.sel, (300.0 * scale()) as i32 / 2);
                if v.en.is_empty() {
                    draw_column(hdc, &v.cands, v.sel, true, 0, rc.right, &pal);
                } else {
                    draw_column(hdc, &v.cands, v.sel, v.col == 0, 0, left_w, &pal);
                    // 列の区切り線
                    let hp = CreatePen(PS_SOLID, 1, COLORREF(pal.header));
                    let oldp = SelectObject(hdc, hp);
                    let _ = MoveToEx(hdc, left_w, row_h(), None);
                    let _ = LineTo(hdc, left_w, rc.bottom);
                    SelectObject(hdc, oldp);
                    let _ = DeleteObject(hp);
                    draw_column(hdc, &v.en, v.en_sel, v.col == 1, left_w + 1, rc.right, &pal);
                }
            });
            SelectObject(hdc, old);
            let _ = DeleteObject(font);
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

fn ensure_window() -> Option<HWND> {
    HWND_CAND.with(|h| {
        if let Some(hw) = *h.borrow() {
            return Some(hw);
        }
        unsafe {
            let hinst = GetModuleHandleW(PCWSTR::null()).ok()?;
            let class = w!("AruaruImeCandidate");
            let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: hinst.into(), lpszClassName: class, hCursor: LoadCursorW(None, IDC_ARROW).ok()?, ..Default::default() };
            RegisterClassW(&wc); // 既登録なら失敗するが無視
            let hw = CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE, class, w!(""), WS_POPUP | WS_BORDER, 0, 0, 10, 10, None, None, hinst, None).ok()?;
            *h.borrow_mut() = Some(hw);
            Some(hw)
        }
    })
}

/// 候補を表示(`cands` が空なら隠す)。`x,y` は画面座標(組み立て中の文字列の左下)。
pub fn show(cands: &[String], sel: Option<usize>, en: &[String], col: u8, en_sel: Option<usize>, header: &str, x: i32, y: i32) {
    let Some(hw) = ensure_window() else { return };
    if cands.is_empty() {
        hide();
        return;
    }
    let v = View { cands: cands.to_vec(), sel, en: en.to_vec(), en_sel, col, header: header.to_string() };
    let left_w = col_width(&v.cands, v.sel, (300.0 * scale()) as i32 / 2);
    let right_w = if v.en.is_empty() { 0 } else { col_width(&v.en, v.en_sel, (200.0 * scale()) as i32 / 2) };
    let rows = page_items(&v.cands, v.sel).1.len().max(page_items(&v.en, v.en_sel).1.len()) as i32;
    let min_w = (380.0 * scale()) as i32;
    let width = (left_w + right_w).max(min_w);
    STATE.with(|s| *s.borrow_mut() = v);
    unsafe {
        let _ = KillTimer(hw, FLASH_TIMER);
        let _ = SetWindowPos(hw, HWND_TOPMOST, x, y + 2, width, (rows + 1) * row_h() + 4, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        let _ = InvalidateRect(hw, None, true);
        let _ = UpdateWindow(hw);
    }
}

/// 入力モードの切替(あ / A など)を、キャレット付近に約1秒だけ表示する。
pub fn flash(label: &str, x: i32, y: i32) {
    let Some(hw) = ensure_window() else { return };
    STATE.with(|s| *s.borrow_mut() = View { header: label.to_string(), ..Default::default() });
    unsafe {
        let _ = SetWindowPos(hw, HWND_TOPMOST, x, y + 2, (90.0 * scale()) as i32, row_h() + 4, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        let _ = InvalidateRect(hw, None, true);
        let _ = UpdateWindow(hw);
        SetTimer(hw, FLASH_TIMER, 1000, None);
    }
}

/// 配色を切り替えた直後に、表示中の窓を描き直す。
pub fn redraw() {
    HWND_CAND.with(|h| {
        if let Some(hw) = *h.borrow() {
            unsafe {
                let _ = InvalidateRect(hw, None, true);
                let _ = UpdateWindow(hw);
            }
        }
    });
}

pub fn hide() {
    HWND_CAND.with(|h| {
        if let Some(hw) = *h.borrow() {
            unsafe {
                let _ = KillTimer(hw, FLASH_TIMER);
                let _ = ShowWindow(hw, SW_HIDE);
            }
        }
    });
}

pub fn destroy() {
    HWND_CAND.with(|h| {
        if let Some(hw) = h.borrow_mut().take() {
            unsafe {
                let _ = DestroyWindow(hw);
            }
        }
    });
}
