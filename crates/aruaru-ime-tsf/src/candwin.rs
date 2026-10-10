//! 候補ウィンドウ(フォーカスを奪わない最前面の小窓)。ホストアプリの UI スレッド上で作る。
//! 配色は既定が黒背景・白文字。設定(settings.json の theme)または F2 で白背景・黒文字に切り替えられる。

use aruaru_ime_core::settings::Settings;
use std::cell::RefCell;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

const ROW_H: i32 = 26;
const MAX_ROWS: usize = 9;
const FLASH_TIMER: usize = 1;

struct Palette {
    bg: u32,
    text: u32,
    header: u32,
    sel: u32,
}

/// COLORREF は 0x00BBGGRR。
fn palette(dark: bool) -> Palette {
    if dark {
        Palette { bg: 0x0000_0000, text: 0x00FF_FFFF, header: 0x00C8_C8C8, sel: 0x00A0_602A }
    } else {
        Palette { bg: 0x00FF_FFFF, text: 0x0020_2020, header: 0x0080_6040, sel: 0x00F0_D8A8 }
    }
}

thread_local! {
    static STATE: RefCell<(Vec<String>, Option<usize>, String)> = const { RefCell::new((Vec::new(), None, String::new())) };
    static HWND_CAND: RefCell<Option<HWND>> = const { RefCell::new(None) };
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
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
            let font = CreateFontW(-18, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w!("Yu Gothic UI"));
            let old = SelectObject(hdc, font);
            SetBkMode(hdc, TRANSPARENT);
            STATE.with(|s| {
                let (cands, sel, header) = &*s.borrow();
                // 見出し行: 予測した入力モード(間違っていたら F6〜F10 で直せる)
                SetTextColor(hdc, COLORREF(pal.header));
                let hint = if cands.is_empty() { String::new() } else { "  F6あ F7ア F9Ａ F10A F2配色".to_string() };
                let mut ht: Vec<u16> = format!(" [{header}]{hint}").encode_utf16().collect();
                let mut hr = RECT { left: 0, top: 0, right: rc.right, bottom: ROW_H };
                DrawTextW(hdc, &mut ht, &mut hr, DT_SINGLELINE | DT_VCENTER | DT_LEFT);
                // 選択位置を含む9件のページを表示
                let page = sel.map_or(0, |i| i / MAX_ROWS) * MAX_ROWS;
                for (row, (i, c)) in cands.iter().enumerate().skip(page).take(MAX_ROWS).enumerate() {
                    let r = RECT { left: 0, top: (row as i32 + 1) * ROW_H, right: rc.right, bottom: (row as i32 + 2) * ROW_H };
                    if *sel == Some(i) {
                        let hb = CreateSolidBrush(COLORREF(pal.sel));
                        FillRect(hdc, &r, hb);
                        let _ = DeleteObject(hb);
                    }
                    SetTextColor(hdc, COLORREF(pal.text));
                    let mut text: Vec<u16> = format!(" {} {}", row + 1, c).encode_utf16().collect();
                    let mut r2 = r;
                    DrawTextW(hdc, &mut text, &mut r2, DT_SINGLELINE | DT_VCENTER | DT_LEFT);
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
pub fn show(cands: &[String], sel: Option<usize>, header: &str, x: i32, y: i32) {
    let Some(hw) = ensure_window() else { return };
    if cands.is_empty() {
        hide();
        return;
    }
    STATE.with(|s| *s.borrow_mut() = (cands.to_vec(), sel, header.to_string()));
    let page = sel.map_or(0, |i| i / MAX_ROWS) * MAX_ROWS;
    let rows = cands.iter().skip(page).take(MAX_ROWS).count() as i32;
    let widest = cands.iter().skip(page).take(MAX_ROWS).map(|c| c.chars().map(|ch| if ch.is_ascii() { 10 } else { 19 }).sum::<i32>()).max().unwrap_or(60);
    unsafe {
        let _ = KillTimer(hw, FLASH_TIMER);
        let _ = SetWindowPos(hw, HWND_TOPMOST, x, y + 2, (widest + 56).max(300), (rows + 1) * ROW_H + 4, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        let _ = InvalidateRect(hw, None, true);
        let _ = UpdateWindow(hw);
    }
}

/// 入力モードの切替(あ / A など)を、キャレット付近に約1秒だけ表示する。
pub fn flash(label: &str, x: i32, y: i32) {
    let Some(hw) = ensure_window() else { return };
    STATE.with(|s| *s.borrow_mut() = (Vec::new(), None, label.to_string()));
    unsafe {
        let _ = SetWindowPos(hw, HWND_TOPMOST, x, y + 2, 90, ROW_H + 4, SWP_NOACTIVATE | SWP_SHOWWINDOW);
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
