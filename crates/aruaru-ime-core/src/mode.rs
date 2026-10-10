//! 入力モードの予測。入力途中の文字列・直前の文章・画面上の文字(OCR)から、
//! 今どの文字種で打っているかを推定する。

use crate::romaji;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Alpha,
    Hiragana,
    Katakana,
    Number,
    Symbol,
}

#[derive(Debug, Clone, Serialize)]
pub struct Prediction {
    pub mode: Mode,
    pub confidence: f32,
    /// 英単語として確定(アルファベットのまま)を勧めるか。
    pub english_word: bool,
    pub reason: String,
}

#[derive(Debug, Default, Clone)]
pub struct Context {
    /// カーソル直前の文章。
    pub before: String,
    /// カーソル直後の文章。
    pub after: String,
    /// 画面全体のOCR結果(行をつなげたもの)。
    pub screen_text: String,
    /// 入力欄の種類(password / url / email / number / search / text …)。
    pub field_kind: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Ratios {
    pub ascii_alpha: f32,
    pub kana: f32,
    pub kanji: f32,
    pub digit: f32,
    pub katakana: f32,
}

pub fn ratios(s: &str) -> Ratios {
    let mut n = 0f32;
    let (mut a, mut k, mut j, mut d, mut kt) = (0f32, 0f32, 0f32, 0f32, 0f32);
    for c in s.chars().filter(|c| !c.is_whitespace()) {
        n += 1.0;
        match c {
            'a'..='z' | 'A'..='Z' => a += 1.0,
            '0'..='9' => d += 1.0,
            'ぁ'..='ゖ' => k += 1.0,
            'ァ'..='ヺ' | 'ー' => {
                kt += 1.0;
                k += 1.0
            }
            '一'..='龥' => j += 1.0,
            _ => {}
        }
    }
    if n == 0.0 {
        return Ratios::default();
    }
    Ratios { ascii_alpha: a / n, kana: k / n, kanji: j / n, digit: d / n, katakana: kt / n }
}

/// 英単語としてよくある綴り(ローマ字として不自然なもの)。
fn looks_english(lower: &str) -> bool {
    if lower.len() < 3 {
        return false;
    }
    if !romaji::is_romaji_progress(lower) {
        return true;
    }
    ["th", "ck", "ph", "wh", "ght", "tion", "ing", "ough", "qu"].iter().any(|p| lower.contains(p))
}

const COMMON_EN: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "from", "have", "you", "are", "not", "but", "what", "your", "about", "hello", "world",
    "windows", "github", "google", "server", "client", "error", "file", "name", "test", "code", "email", "password", "http", "https",
    "www", "com", "json", "api", "rust", "python", "release", "build", "install", "update", "version", "please", "thanks", "meeting",
];

pub fn predict(buffer: &str, ctx: &Context) -> Prediction {
    let kind = ctx.field_kind.as_deref().unwrap_or("text");
    let p = |mode, confidence, english_word, reason: &str| Prediction { mode, confidence, english_word, reason: reason.to_string() };

    if matches!(kind, "password" | "url" | "email") && !buffer.is_empty() {
        return p(Mode::Alpha, 0.99, false, "入力欄の種類が英数字向け");
    }
    if kind == "number" {
        return p(Mode::Number, 0.99, false, "数値欄");
    }
    if buffer.is_empty() {
        return p(Mode::Hiragana, 0.3, false, "入力なし");
    }
    if buffer.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-' | ':' | '/')) && buffer.chars().any(|c| c.is_ascii_digit()) {
        return p(Mode::Number, 0.95, false, "数字のみ");
    }
    if buffer.chars().all(|c| !c.is_alphanumeric()) {
        return p(Mode::Symbol, 0.9, false, "記号のみ");
    }

    // 直前の文脈(最後の60文字)と画面全体の文字種の傾向
    let tail: String = ctx.before.chars().rev().take(60).collect::<Vec<_>>().into_iter().rev().collect();
    let rb = ratios(&tail);
    let rs = ratios(&ctx.screen_text);
    let lower = buffer.to_ascii_lowercase();
    let mut score_alpha = 0f32;
    let mut score_kana = 0f32;
    let mut why: Vec<&str> = vec![];

    let langs = crate::code::detect(ctx);
    let code_word = crate::code::is_code_word_for_alpha(&lower, !langs.is_empty())
        || (!langs.is_empty() && lower.chars().count() >= 3 && !crate::code::complete(&lower, &langs, 1).is_empty());
    if code_word {
        score_alpha += 1.3;
        why.push("プログラミング用語");
    }
    let known_en = COMMON_EN.contains(&lower.as_str());
    let en_like = looks_english(&lower);
    if known_en {
        score_alpha += 1.2;
        why.push("頻出英単語");
    }
    if en_like {
        score_alpha += 1.0;
        why.push("ローマ字として不自然");
    } else if romaji::is_romaji_progress(&lower) {
        score_kana += 0.8;
        why.push("ローマ字として自然");
    }
    if buffer.chars().skip(1).any(|c| c.is_ascii_uppercase()) {
        score_alpha += 0.6;
        why.push("大文字混在");
    }
    if rb.ascii_alpha > 0.6 {
        score_alpha += 0.7;
        why.push("直前は英字");
    }
    if rb.kana + rb.kanji > 0.4 {
        score_kana += 0.7;
        why.push("直前は日本語");
    }
    if rs.ascii_alpha > 0.6 {
        score_alpha += 0.25;
    } else if rs.kana + rs.kanji > 0.3 {
        score_kana += 0.25;
    }

    let total = (score_alpha + score_kana).max(0.01);
    if score_alpha > score_kana {
        return Prediction {
            mode: Mode::Alpha,
            confidence: (score_alpha / total).min(0.99),
            english_word: en_like || known_en || code_word,
            reason: why.join("・"),
        };
    }
    if rb.katakana > 0.5 {
        return Prediction { mode: Mode::Katakana, confidence: 0.7, english_word: false, reason: "直前がカタカナ主体".into() };
    }
    Prediction { mode: Mode::Hiragana, confidence: (score_kana / total).min(0.99), english_word: false, reason: why.join("・") }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(before: &str) -> Context {
        Context { before: before.into(), ..Default::default() }
    }
    #[test]
    fn digits() {
        assert_eq!(predict("2026", &ctx("")).mode, Mode::Number);
    }
    #[test]
    fn english_in_english_text() {
        assert_eq!(predict("hello", &ctx("Say ")).mode, Mode::Alpha);
    }
    #[test]
    fn romaji_in_japanese() {
        assert_eq!(predict("sakura", &ctx("春は")).mode, Mode::Hiragana);
    }
    #[test]
    fn english_word_in_japanese() {
        let r = predict("github", &ctx("私は"));
        assert_eq!(r.mode, Mode::Alpha);
        assert!(r.english_word);
    }
    #[test]
    fn password() {
        let c = Context { field_kind: Some("password".into()), ..Default::default() };
        assert_eq!(predict("sakura", &c).mode, Mode::Alpha);
    }
    #[test]
    fn katakana_context() {
        assert_eq!(predict("ra-men", &ctx("コンピュータ")).mode, Mode::Katakana);
    }
}
