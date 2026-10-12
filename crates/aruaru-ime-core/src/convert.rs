//! 変換候補の生成。入力モード予測の結果に従って候補を並べ、
//! 条件が揃えば aruaru-llm(Qwen)による漢字かな交じり・英単語補完・検索用例を加える。

use crate::client::Client;
use crate::kanji::Dict;
use crate::code;
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
    /// 検索フォーム型の予測(続きを含む検索語)。検索欄・ブラウザ等で使う。
    pub suggestions: Vec<String>,
    /// 辞書に、読みの全体と一致する項目(1語として載っている)があるか。`nn` の二通りの読みのどちらが正しいかの判定に使う。
    #[serde(default)]
    pub exact_dict: bool,
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

/// 検索フォーム(Google・Amazon商品検索など)らしい場面か。
pub fn wants_suggest(ctx: &Context) -> bool {
    if ctx.field_kind.as_deref() == Some("search") {
        return true;
    }
    let s = ctx.screen_text.to_lowercase();
    ["google", "amazon", "検索", "カートに入れる", "yahoo"].iter().any(|k| s.contains(k))
}

pub fn convert(buffer: &str, ctx: &Context, client: Option<&Client>, dict: Option<&Dict>) -> Conversion {
    let pred = mode::predict(buffer, ctx);
    let mut c: Vec<Candidate> = vec![];
    if buffer.chars().count() == 1 {
        if let Some(list) = buffer.chars().next().and_then(crate::prefer::lead_symbol) {
            // 記号の種類を並べる(- + / < > ? ! かっこ、など)
            for h in list.split(' ') {
                push(&mut c, h.to_string(), "symbol");
            }
            return Conversion { prediction: pred, candidates: c, suggestions: vec![], exact_dict: false };
        }
    }
    let hira = romaji::to_hiragana(buffer);
    let hira_for_exact = hira.clone();
    let kata = romaji::hira_to_kata(&hira);

    // 辞書+Viterbi(ミリ秒)。Qwen は現状の速度では同期経路に置けないため、ARUARU_IME_QWEN=1 のときだけ追加する。
    let mut dict_cands: Vec<String> = vec![];
    if matches!(pred.mode, Mode::Hiragana | Mode::Katakana) && romaji::is_romaji_parsable(buffer) {
        if let Some(d) = dict {
            dict_cands = d.candidates_pref(&hira, 30, ctx.prefer_kanji);
            // 読みをそのままカタカナにしただけの候補(タンゴ)が先頭で、上位に漢字の候補(単語)があるときは、漢字を先頭にする。
            // 日本語入力では、漢字に変換することが前提のため。漢字の対応が無いカタカナ語(コンピュータ)は、そのまま先頭。
            if !ctx.prefer_kanji && dict_cands.first().is_some_and(|f| *f == kata) {
                let kanji_at = dict_cands.iter().take(6).position(|c| c.chars().any(|ch| ('一'..='龥').contains(&ch)));
                if let Some(i) = kanji_at {
                    let k = dict_cands.remove(i);
                    dict_cands.insert(0, k);
                }
            }
        }
    }
    // `nn` の直後が母音のとき、「ん」+「な行」と読んだ変換(junni → 順に)のほかに、「ん」+母音と読んだ変換(→ 順位)も2番目に出す
    if let (Some(d), Some(alt)) = (dict, romaji::to_hiragana_alt(buffer)) {
        if matches!(pred.mode, Mode::Hiragana | Mode::Katakana) && !dict_cands.is_empty() {
            if let Some(a) = d.candidates_pref(&alt, 3, ctx.prefer_kanji).into_iter().find(|x| *x != alt && !dict_cands.contains(x)) {
                dict_cands.insert(1.min(dict_cands.len()), a);
            }
        }
    }
    let mut predicted: Vec<String> = vec![];
    // 1文字だけでは候補が多すぎて役に立たないので、読みが2文字以上になってから予測する
    if matches!(pred.mode, Mode::Hiragana | Mode::Katakana) && romaji::to_hiragana_partial(buffer).0.chars().count() >= 2 {
        if let Some(d) = dict {
            // 打ちかけの読み(確定したかな部分)から、より長い語・フレーズを予測する
            predicted = d.predict(&romaji::to_hiragana_partial(buffer).0, 8).into_iter().map(|(_, s)| s).collect();
        }
    }
    // 同じ読みの語が複数あるとき(対応/大王/滞欧)、読みの先頭が一致する「別の長い固有名詞・作品名」(太王四神記)は、
    // 最有力の語の次、残りの同音語の前に出す。最有力の語から派生しただけの語(対応付け)は対象にしない。
    if dict_cands.len() >= 3 && romaji::to_hiragana_partial(buffer).0.chars().count() >= 3 {
        let is_kanji = |ch: char| ('一'..='龥').contains(&ch);
        if let Some(i) = predicted.iter().position(|p| p.chars().count() >= 3 && p.chars().all(is_kanji) && !dict_cands.contains(p)) {
            let promoted = predicted.remove(i);
            dict_cands.insert(1, promoted);
        }
    }
    let mut kanji: Option<String> = None;
    if std::env::var("ARUARU_IME_QWEN").as_deref() == Ok("1") {
        if let Some(cl) = client {
            if !dict_cands.is_empty() || dict.is_none() {
                let web = if hira.chars().count() >= 2 { cl.search_snippets(&hira, 3) } else { vec![] };
                kanji = cl.generate(&kanji_prompt(&hira, ctx, &web), 12).and_then(|o| clean_llm(&o, &hira));
            }
        }
    }

    match pred.mode {
        Mode::Alpha => {
            push(&mut c, buffer.to_string(), "alpha");
            // プログラミング用語: 綴りの訂正(確信が高いものは autofix)と前方一致の補完。パスワード・URL・メール欄では何もしない。
            if !matches!(ctx.field_kind.as_deref(), Some("password" | "url" | "email")) {
                let langs = code::detect(ctx);
                for f in code::correct(buffer, &langs) {
                    push(&mut c, f.word, if f.confident { "autofix" } else { "fix" });
                }
                for w in code::complete(buffer, &langs, 6) {
                    push(&mut c, w, "code");
                }
            }
            for w in prefix_words(buffer) {
                push(&mut c, w.to_string(), "english");
            }
            // 英語を打っているときは、「をrd」のような、かなとローマ字が混ざった候補は出さない
            if !hira.chars().any(|ch| ch.is_ascii_alphabetic()) {
                push(&mut c, hira, "hiragana");
            }
        }
        Mode::Katakana => {
            // カタカナ入力でも、辞書の最有力候補(漢字かな交じり・カタカナ語)を先頭にする。純粋なカタカナは次に出す
            for d in &dict_cands {
                push(&mut c, d.clone(), "dict");
            }
            if let Some(k) = kanji {
                push(&mut c, k, "llm");
            }
            push(&mut c, kata, "katakana");
            push(&mut c, hira, "hiragana");
            push(&mut c, buffer.to_string(), "alpha");
            for p in &predicted {
                push(&mut c, p.clone(), "predict");
            }
        }
        Mode::Hiragana => {
            for d in &dict_cands {
                push(&mut c, d.clone(), "dict");
            }
            if let Some(k) = kanji {
                push(&mut c, k, "llm");
            }
            push(&mut c, hira, "hiragana");
            push(&mut c, kata, "katakana");
            push(&mut c, buffer.to_string(), "alpha");
            for p in &predicted {
                push(&mut c, p.clone(), "predict");
            }
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
    // 優先順位の調整(組み込み + 利用者の prefer.tsv)と、記号の一覧(はいふん → - － ‐ – — ― …)、ハイフンの種類
    if matches!(pred.mode, Mode::Hiragana | Mode::Katakana) && romaji::is_romaji_parsable(buffer) {
        let user = crate::prefer::load_user(&crate::components::data_dir());
        let want = crate::prefer::preferred(&hira_for_exact, &user);
        if !want.is_empty() {
            let mut texts: Vec<String> = c.iter().map(|x| x.text.clone()).collect();
            crate::prefer::reorder(&mut texts, &want);
            c = texts.into_iter().map(|t| Candidate { kind: if want.contains(&t) { "dict" } else { "other" }, text: t }).collect();
        }
        let syms = crate::prefer::symbols_for(&hira_for_exact);
        if !syms.is_empty() {
            let mut head: Vec<Candidate> = syms.iter().map(|s| Candidate { text: s.clone(), kind: "symbol" }).collect();
            c.retain(|x| !syms.contains(&x.text));
            head.extend(c);
            c = head;
        }
        if buffer.contains('-') {
            // 打った - を、半角・全角・ダッシュ・マイナスなどに替えた候補
            for h in crate::prefer::HYPHENS.chars() {
                if h != '-' {
                    let t = hira_for_exact.replace('ー', &h.to_string());
                    if !c.iter().any(|x| x.text == t) {
                        c.push(Candidate { text: t, kind: "symbol" });
                    }
                }
            }
            let half = hira_for_exact.replace('ー', "-");
            if !c.iter().any(|x| x.text == half) {
                c.push(Candidate { text: half, kind: "symbol" });
            }
        }
    }
    // 検索フォーム型の予測: 検索欄(または画面が検索/商品ページ)で、確定見込みの語から続きの検索語を出す
    let mut suggestions = vec![];
    if let Some(cl) = client {
        if wants_suggest(ctx) && !buffer.is_empty() {
            let base = match pred.mode {
                Mode::Alpha | Mode::Number => buffer.to_string(),
                _ => c.iter().find(|x| x.kind == "dict" || x.kind == "llm").map(|x| x.text.clone()).unwrap_or_else(|| romaji::to_hiragana(buffer)),
            };
            suggestions = cl.web_suggest(&base, if matches!(pred.mode, Mode::Alpha) { "en" } else { "ja" });
        }
    }
    let exact_dict = dict.is_some_and(|d| !d.exact(&hira_for_exact, 1).is_empty());
    Conversion { prediction: pred, candidates: c, suggestions, exact_dict }
}

#[cfg(test)]
mod tests {
    #[test]
    fn same_reading_puts_a_famous_title_second() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dict.bin");
        let Ok(d) = crate::kanji::Dict::load(&p) else { return };
        let r = convert("taiou", &cx(""), None, Some(&d));
        let t: Vec<&str> = r.candidates.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(&t[..4], ["対応", "太王四神記", "大王", "滞欧"], "{t:?}");
    }

    use super::*;
    fn cx(b: &str) -> Context {
        Context { before: b.into(), ..Default::default() }
    }
    #[test]
    fn offline_japanese() {
        let r = convert("konnnichiha", &cx("彼は"), None, None);
        assert_eq!(r.candidates[0].text, "こんにちは");
    }
    #[test]
    fn english_completion() {
        let r = convert("hell", &cx("Say "), None, None);
        assert_eq!(r.prediction.mode, Mode::Alpha);
        assert!(r.candidates.iter().any(|c| c.text == "hello"));
    }
    #[test]
    fn partial_romaji_is_not_mistaken_for_english() {
        // 入力途中(sak, sakur)で英字モードにちらつかない
        for w in ["sak", "sakur", "kyo", "konnnic"] {
            let r = convert(w, &cx(""), None, None);
            assert_ne!(r.prediction.mode, Mode::Alpha, "{w}");
        }
        // 本物の英単語は文脈なしでも英字
        assert_eq!(convert("github", &cx(""), None, None).prediction.mode, Mode::Alpha);
    }

    #[test]
    fn code_words_stay_alpha_and_get_completed_and_fixed() {
        let js = cx("function f() { const x = 1; console.log(x) }\n");
        let r = convert("unde", &js, None, None);
        assert_eq!(r.prediction.mode, Mode::Alpha);
        assert!(r.candidates.iter().any(|c| c.text == "undefined" && c.kind == "code"));
        let r = convert("fucntion", &js, None, None);
        assert!(r.candidates.iter().any(|c| c.text == "function" && c.kind == "autofix"));
        // パスワード欄では何もしない
        let pw = Context { before: js.before.clone(), field_kind: Some("password".into()), ..Default::default() };
        assert!(convert("fucntion", &pw, None, None).candidates.iter().all(|c| c.kind != "autofix"));
        // 日本語の文脈ではローマ字入力を奪わない
        assert_eq!(convert("kyouha", &cx("私は"), None, None).prediction.mode, Mode::Hiragana);
        assert_eq!(convert("do", &cx("私は"), None, None).prediction.mode, Mode::Hiragana);
    }

    #[test]
    fn kanji_word_beats_plain_katakana_transcription() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dict.bin");
        let Ok(d) = Dict::load(&p) else { return };
        let ctx = cx("タンゴ ト カンジ ヲ ");
        let r = convert("tango", &ctx, None, Some(&d));
        assert_eq!(r.candidates[0].text, "単語", "{:?}", r.candidates.iter().take(5).map(|c| &c.text).collect::<Vec<_>>());
        assert!(r.candidates.iter().any(|c| c.text == "タンゴ"), "カタカナも候補に残る");
        // 漢字の対応が無い外来語は、カタカナのまま先頭
        let r = convert("konpyu-ta", &ctx, None, Some(&d));
        assert_eq!(r.candidates[0].text, "コンピュータ");
    }

    #[test]
    fn numbers_fullwidth() {
        let r = convert("2026", &cx(""), None, None);
        assert!(r.candidates.iter().any(|c| c.text == "２０２６"));
    }
    #[test]
    fn search_field_detection() {
        assert!(wants_suggest(&Context { field_kind: Some("search".into()), ..Default::default() }));
        assert!(wants_suggest(&Context { screen_text: "Amazon.co.jp 商品を検索".into(), ..Default::default() }));
        assert!(!wants_suggest(&cx("hi")));
    }
    #[test]
    fn clean() {
        assert_eq!(clean_llm("天気\n文脈:...", "てんき").as_deref(), Some("天気"));
        assert_eq!(clean_llm("ながいながいながいながい", "て"), None);
    }
}
