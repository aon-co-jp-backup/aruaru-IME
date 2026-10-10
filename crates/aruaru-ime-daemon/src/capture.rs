//! 画面全体のキャプチャ(PNG)。Windows は GDI、macOS は `screencapture`、Linux は grim / scrot / import。
//! OCR は 1920px 幅程度で十分なので、大きい画面は縮小して返す。

use image::{imageops::FilterType, DynamicImage, ImageFormat, RgbaImage};
use std::io::Cursor;

const MAX_WIDTH: u32 = 1920;

fn to_png(img: DynamicImage) -> Option<Vec<u8>> {
    let img = if img.width() > MAX_WIDTH {
        let h = (img.height() as f64 * MAX_WIDTH as f64 / img.width() as f64) as u32;
        img.resize_exact(MAX_WIDTH, h.max(1), FilterType::Triangle)
    } else {
        img
    };
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

#[cfg(windows)]
pub fn capture_png() -> Option<Vec<u8>> {
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe {
        let (w, h) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        if w <= 0 || h <= 0 {
            return None;
        }
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(screen);
        let bmp = CreateCompatibleBitmap(screen, w, h);
        let old = SelectObject(mem, bmp);
        let ok = BitBlt(mem, 0, 0, w, h, screen, 0, 0, SRCCOPY).is_ok();
        let mut bi = BITMAPINFO::default();
        bi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // 上から下
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            ..Default::default()
        };
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let lines = GetDIBits(mem, bmp, 0, h as u32, Some(buf.as_mut_ptr() as *mut _), &mut bi, DIB_RGB_COLORS);
        SelectObject(mem, old);
        let _ = DeleteObject(bmp);
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        if !ok || lines == 0 {
            return None;
        }
        for px in buf.chunks_exact_mut(4) {
            px.swap(0, 2); // BGRA → RGBA
            px[3] = 255;
        }
        to_png(DynamicImage::ImageRgba8(RgbaImage::from_raw(w as u32, h as u32, buf)?))
    }
}

#[cfg(not(windows))]
pub fn capture_png() -> Option<Vec<u8>> {
    use std::process::Command;
    let tmp = std::env::temp_dir().join(format!("aruaru-ime-shot-{}.png", std::process::id()));
    let tries: Vec<(&str, Vec<String>)> = if cfg!(target_os = "macos") {
        vec![("screencapture", vec!["-x".into(), tmp.display().to_string()])]
    } else {
        vec![
            ("grim", vec![tmp.display().to_string()]),
            ("scrot", vec!["-o".into(), tmp.display().to_string()]),
            ("import", vec!["-window".into(), "root".into(), tmp.display().to_string()]),
        ]
    };
    for (cmd, args) in tries {
        if Command::new(cmd).args(&args).status().map(|s| s.success()).unwrap_or(false) {
            let img = image::open(&tmp).ok();
            let _ = std::fs::remove_file(&tmp);
            return img.and_then(to_png);
        }
    }
    None
}
