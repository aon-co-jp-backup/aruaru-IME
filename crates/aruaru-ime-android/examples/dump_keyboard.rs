//! キーボードの画面(HTML)を標準出力へ書き出す。ブラウザで見た目を確かめるための道具。
//! 使い方: `cargo run -p aruaru-ime-android --example dump_keyboard > keyboard.html`
fn main() {
    print!("{}", aruaru_ime_android::ui::keyboard_html());
}
