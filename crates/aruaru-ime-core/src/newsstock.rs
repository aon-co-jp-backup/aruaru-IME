//! ネットニュースのストック(maid-cafe.tokyo が回答に使っている、realdata.pro 由来の蓄積ニュース)を、
//! 変換候補に活かす(2026-11-06 ユーザー指示)。
//!
//! - 公開の `News/For`(国別の最新見出し)から見出しと抜粋を取り、`<データ領域>/news-stock.json` に持つ。
//! - 変換候補では、ニュースに出てくる語を「文脈の手がかり」として同音語の並べ替えに使う(`contextual.rs`)。
//! - ニュースに新しく出てきたカタカナ語(人名・新語・外来語)は、読みがそのまま分かるので利用者辞書へ
//!   出所つき(`自動更新:ニュース`)で入れる。出所ごとに削除できる。漢字語は読みが分からないため、辞書には入れない。
//! - 見出しだけを扱い、記事本文は複製しない。取得は公開エンドポイントのみ(個人情報は送らない)。

use std::path::{Path, PathBuf};

pub const SOURCE: &str = "自動更新:ニュース";
const COUNTRIES: &[&str] = &["Japan", "United States", "China", "Russia", "Ukraine", "Israel", "South Korea", "United Kingdom", "India"];
const MAX_LINES: usize = 600;
const MAX_KATAKANA_TERMS: usize = 150;

pub fn path(root: &Path) -> PathBuf {
    root.join("news-stock.json")
}

fn load(p: &Path) -> Vec<String> {
    std::fs::read_to_string(p).ok().and_then(|t| sonic_rs::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
}

/// ストックした見出し全体を1つの文字列に(文脈の手がかり用)。ファイルが変わったときだけ読み直す。
pub fn corpus() -> String {
    use std::sync::Mutex;
    use std::time::SystemTime;
    static CACHE: Mutex<Option<(SystemTime, String)>> = Mutex::new(None);
    let p = path(&crate::components::data_dir());
    let Some(mt) = std::fs::metadata(&p).and_then(|m| m.modified()).ok() else { return String::new() };
    let Ok(mut g) = CACHE.lock() else { return String::new() };
    if g.as_ref().map(|(t, _)| *t != mt).unwrap_or(true) {
        *g = Some((mt, load(&p).join("\n")));
    }
    g.as_ref().map(|(_, s)| s.clone()).unwrap_or_default()
}

/// 語を含む最新の見出しを1件(候補ウィンドウの補足表示用)。
pub fn headline_with(word: &str) -> Option<String> {
    let w = word.trim();
    if w.chars().count() < 2 {
        return None;
    }
    let line = corpus().lines().find(|l| l.contains(w))?.to_string();
    let t: String = line.chars().take(60).collect();
    Some(format!("📰 {}{}", t, if line.chars().count() > 60 { "…" } else { "" }))
}

/// 見出しの文章から、カタカナ語(3文字以上、長音・中黒を含む)を取り出す。
pub fn katakana_terms(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in lines {
        let mut cur = String::new();
        for c in l.chars().chain(std::iter::once(' ')) {
            if ('\u{30a1}'..='\u{30fa}').contains(&c) || c == 'ー' {
                cur.push(c);
            } else {
                if cur.chars().count() >= 3 && cur.chars().any(|k| k != 'ー') && !cur.starts_with('ー') {
                    out.push(std::mem::take(&mut cur));
                }
                cur.clear();
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn fetch_country(agent: &ureq::Agent, country: &str) -> Vec<String> {
    let url = format!("https://maid-cafe.tokyo/openenglish.v1.News/For?country={}", country.replace(' ', "%20"));
    let Ok(resp) = agent.post(&url).send_json(serde_json::json!({})) else { return vec![] };
    let Ok(v) = resp.into_json::<serde_json::Value>() else { return vec![] };
    v["items"]
        .as_array()
        .map(|a| {
            a.iter()
                .flat_map(|i| [i["title"].as_str(), i["snippet"].as_str()])
                .flatten()
                .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// ニュースのストックを取り込む。増えた(見出しの行数, 辞書に入れたカタカナ語)を返す。失敗は黙って 0。
pub fn sync() -> (usize, usize) {
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(12)).build();
    let root = crate::components::data_dir();
    let p = path(&root);
    let mut lines = load(&p);
    let before = lines.len();
    for c in COUNTRIES {
        for l in fetch_country(&agent, c) {
            if !lines.contains(&l) {
                lines.push(l);
            }
        }
    }
    // 新しいものを残す(古い方から捨てる)
    if lines.len() > MAX_LINES {
        lines.drain(..lines.len() - MAX_LINES);
    }
    let added = lines.len().saturating_sub(before).min(lines.len());
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let _ = std::fs::write(&p, serde_json::to_string(&lines).unwrap_or_default());
    // カタカナ語を利用者辞書へ(読み = ひらがな化)
    let pairs: Vec<(String, String)> = katakana_terms(&lines).into_iter().take(MAX_KATAKANA_TERMS).map(|k| (crate::romaji::kata_to_hira(&k), k)).collect();
    let mut user = crate::userdict::UserDict::load(&crate::userdict::UserDict::default_path());
    let n = user.add_many(&pairs, SOURCE).unwrap_or(0);
    (added, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_katakana_terms() {
        let l = vec!["ウクライナ情勢 ; ゼレンスキー大統領 ; AI ; ー ; ニュース".to_string()];
        let t = katakana_terms(&l);
        assert!(t.contains(&"ウクライナ".to_string()) && t.contains(&"ゼレンスキー".to_string()) && t.contains(&"ニュース".to_string()), "{t:?}");
        assert!(!t.iter().any(|x| x == "ー"));
    }

    /// 実ネットワーク確認。`cargo test -p aruaru-ime-core live_news -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn live_news_sync() {
        let (lines, kata) = sync();
        println!("lines +{lines}, katakana +{kata}, corpus {} chars", corpus().chars().count());
        assert!(corpus().chars().count() > 200);
    }
}
