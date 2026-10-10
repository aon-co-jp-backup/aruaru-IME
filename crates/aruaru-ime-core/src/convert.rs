//! 変換候補の生成。入力モード予測の結果に従って候補を並べ、
//! 条件が揃えば aruaru-llm(Qwen)による漢字かな交じり・英単語補完・検索用例を加える。

use crate::client::Client;
use crate::mode::{self, Context, Mode, Prediction};
use crate::romaji;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Candidate {
    pub text: String,
    pub kind: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Conversion {
    pub prediction: Prediction,
    pub candidates: Vec<Candidate>,
}

fn push(v: &mut Vec<Candidate>, text: String, kind: &'static str) {
    if !text.is_empty() && !v.iter().any(|c| c.text == text) {
        v.push(Candidate { text, kind });
    }
}

const EN_WORDS: &[&str] = &[
    "hello", "world", "windows", "github", "google", "server", "client", "error", "release", "install", "update", "version",
    "meeting", "please", "thanks", "password", "request", "response", "repository", "document", "development", "production",
];

fn prefix_words(buf: &str) -> Vec<&'static str> {
    let b = buf.to_ascii_lowercase();
    EN_WORDS.iter().copied().filter(|w| w.starts_with(&b) && *w != b).take(3).collect()
}

fn last_chars(s: &str, n: usize) -> String {
    s.chars().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()
}

fn kanji_prompt(reading: &str, ctx: &Context, web: &[String]) -> String {
    let mut p = String::from("読みを、文脈に合う自然な漢字かな交じりの表記に直す。\n例:\n文脈:今日は|読み:てんき|表記:天気\n文脈:私は|読み:がくせい|表記:学生\n文脈:会議の|読み:しりょう|表記:資料\n");
    if !web.is_empty() {
        p.push_str("参考用例:");
        p.push_str(&web.iter().take(3).map(|s| s.chars().take(60).collect::<String>()).collect::<Vec<_>>().join(" / "));
        p.push('\n');
    }
    let screen: String = ctx.screen_text.replace('\n', " ").chars().take(120).collect();
    if !screen.is_empty() {
        p.push_str(&format!("画面の文字:{}\n", screen));
    }
    p.push_str(&format!("文脈:{}|読み:{}|表記:", last_chars(&ctx.before, 30), reading));
    p
}

fn clean_llm(out: &str, reading: &str) -> Option<String> {
    let line = out.lines().next()?.split('|').next()?.trim().to_string();
    if line.is_empty() || line.chars().count() > reading.chars().count() * 2 + 2 || line.contains("読み") {
        return None;
    }
    Some(line)
}

pub fn convert(buffer: &str, ctx: &Context, client: Option<&Client>) -> Conversion {
    let pred = mode::predict(buffer, ctx);
    let mut c: Vec<Candidate> = vec![];
    let hira = romaji::to_hiragana(buffer);
    let kata = romaji::hira_to_kata(&hira);

    // LLM由来の候補(接続できなければ空)
    let mut kanji: Option<String> = None;
    if let Some(cl) = client {
        if matches!(pred.mode, Mode::Hiragana | Mode::Katakana) && romaji::is_romaji_parsable(buffer) {
            let web = if hira.chars().count() >= 2 { cl.search_snippets(&hira, 3) } else { vec![] };
            kanji = cl.generate(&kanji_prompt(&hira, ctx, &web), 12).and_then(|o| clean_llm(&o, &hira));
        }
    }

    match pred.mode {
        Mode::Alpha => {
            push(&mut c, buffer.to_string(), "alpha");
            for w in prefix_words(buffer) {
                push(&mut c, w.to_string(), "english");
            }
            push(&mut c, hira, "hiragana");
        }
        Mode::Katakana => {
            push(&mut c, kata, "katakana");
            if let Some(k) = kanji {
                push(&mut c, k, "llm");
            }
            push(&mut c, hira, "hiragana");
            push(&mut c, buffer.to_string(), "alpha");
        }
        Mode::Hiragana => {
            if let Some(k) = kanji {
                push(&mut c, k, "llm");
            }
            push(&mut c, hira, "hiragana");
            push(&mut c, kata, "katakana");
            push(&mut c, buffer.to_string(), "alpha");
        }
        Mode::Number => {
            push(&mut c, buffer.to_string(), "number");
            let full: String = buffer
                .chars()
                .map(|ch| if ch.is_ascii_digit() { char::from_u32(ch as u32 - '0' as u32 + '０' as u32).unwrap() } else { ch })
                .collect();
            push(&mut c, full, "fullwidth");
        }
        Mode::Symbol => {
            push(&mut c, buffer.to_string(), "symbol");
            push(&mut c, hira, "hiragana");
        }
    }
    Conversion { prediction: pred, candidates: c }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cx(b: &str) -> Context {
        Context { before: b.into(), ..Default::default() }
    }
    #[test]
    fn offline_japanese() {
        let r = convert("konnnichiha", &cx("彼は"), None);
        assert_eq!(r.candidates[0].text, "こんにちは");
    }
    #[test]
    fn english_completion() {
        let r = convert("hell", &cx("Say "), None);
        assert_eq!(r.prediction.mode, Mode::Alpha);
        assert!(r.candidates.iter().any(|c| c.text == "hello"));
    }
    #[test]
    fn numbers_fullwidth() {
        let r = convert("2026", &cx(""), None);
        assert!(r.candidates.iter().any(|c| c.text == "２０２６"));
    }
    #[test]
    fn clean() {
        assert_eq!(clean_llm("天気\n文脈:...", "てんき").as_deref(), Some("天気"));
        assert_eq!(clean_llm("ながいながいながいながい", "て"), None);
    }
}
