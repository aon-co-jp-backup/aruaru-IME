//! 定型文の自動調査。aruaru-llm(open-cuda / open-cpu で動く Qwen)に、
//!  (1) 利用者が確定した長い文章から、使い回せる定型文を作らせる
//!  (2) よく使うサイト(メルカリなど)の定型文の例を、検索の抜粋から拾わせる
//! ことで、定型文を自動で増やし、変換候補へ組み込む(`templates::frequent` / `templates::lookup` が拾う)。
//!
//! 利用者の文章を外へ出さないため、(1) は aruaru-llm が**この PC の中(127.0.0.1 など)**にあるときだけ行う。
//! (2) の検索語は、サイト名と一般的な言葉だけで、利用者の文章は含めない。設定 `template_mining` で止められる。

use crate::client::Client;
use crate::templates::{self, Site};
use std::path::Path;

const MINED: &str = "templates-mined.txt";
const RESEARCHED: &str = "templates-research.txt";
/// 一度に調べる、利用者の文章の数。
const BATCH: usize = 10;
/// サイトの調査を繰り返す間隔(日)。
const RESEARCH_DAYS: u64 = 14;

fn is_hira(s: &str) -> bool {
    s.chars().all(|c| ('ぁ'..='ん').contains(&c) || c == 'ー')
}

/// モデルの出力(1行に `読み|定型文`)から、使える提案だけを取り出す。
/// 読みはひらがな 2〜12 文字、定型文は 10〜200 文字で改行・タブを含まないもの。
pub fn parse_proposals(out: &str) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = vec![];
    for l in out.lines() {
        let l = l.trim().trim_start_matches(|c: char| c.is_ascii_digit() || matches!(c, '.' | ')' | '-' | '*' | '・' | ' '));
        let Some((r, t)) = l.split_once('|') else { continue };
        let (r, t) = (r.trim(), t.trim());
        let (rn, tn) = (r.chars().count(), t.chars().count());
        if (2..=12).contains(&rn) && is_hira(r) && (10..=200).contains(&tn) && !t.contains(['\t', '|']) && !v.iter().any(|x| x.1 == t) {
            v.push((r.to_string(), t.to_string()));
        }
    }
    v.truncate(8);
    v
}

fn user_prompt(texts: &[String]) -> String {
    let mut p = String::from(
        "次の文章は、ある人が実際によく書く文章です。ここから、別の場面でも使い回せる定型文(テンプレート)を最大5つ作ってください。\n\
         固有名詞・日付・金額・人名は「○○」に置き換えます。丁寧な日本語で、1つの定型文は1行にします。\n\
         出力形式は1行に1つ、「読み|定型文」。読みは、その定型文の先頭の言葉をひらがなで2〜8文字。説明は書かないでください。\n\n文章:\n",
    );
    for t in texts {
        p.push_str("- ");
        p.push_str(t);
        p.push('\n');
    }
    p.push_str("\n定型文:\n");
    p
}

fn site_prompt(site: Site, snippets: &[String]) -> String {
    let mut p = format!(
        "次は、{}での取引メッセージの例文を探した検索結果の抜粋です。ここから、購入者・出品者がそのまま使える丁寧な定型文を最大5つ作ってください。\n\
         固有名詞・日付・金額は「○○」にします。1つの定型文は1行、出力形式は「読み|定型文」。読みは定型文の先頭の言葉のひらがな2〜8文字。説明は不要です。\n\n抜粋:\n",
        site.name()
    );
    for s in snippets.iter().take(8) {
        p.push_str("- ");
        p.push_str(s);
        p.push('\n');
    }
    p.push_str("\n定型文:\n");
    p
}

fn read_lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path).unwrap_or_default().lines().map(str::to_string).collect()
}

/// (1) 利用者の長い文章から定型文を作る。戻り値は、新しく足した定型文の数。
pub fn mine_user_texts(client: &Client, root: &Path) -> usize {
    let done = read_lines(&root.join(MINED));
    let rows = templates::learned_rows(root);
    let fresh: Vec<String> = rows.iter().map(|r| r.text.clone()).filter(|t| !done.contains(t)).take(BATCH).collect();
    if fresh.len() < 3 {
        return 0;
    }
    let Some(out) = client.generate_long(&user_prompt(&fresh), 500, 180) else { return 0 };
    let mut added = 0;
    for (r, t) in parse_proposals(&out) {
        if templates::add_mined(root, &r, &t, "-") {
            added += 1;
        }
    }
    // 結果が空でも、同じ文章を繰り返し調べないよう、調べたことを覚える
    let mut all = done;
    all.extend(fresh);
    let keep = all.len().saturating_sub(500);
    let _ = std::fs::write(root.join(MINED), all[keep..].join("\n"));
    added
}

fn today() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() / 86400).unwrap_or(0)
}

/// (2) 利用者が使っているサイトの定型文の例を、検索の抜粋から拾う。サイトごとに 14 日おき。
pub fn research_sites(client: &Client, root: &Path) -> usize {
    let mut state: Vec<(String, u64)> = read_lines(&root.join(RESEARCHED))
        .iter()
        .filter_map(|l| {
            let (c, d) = l.split_once('\t')?;
            Some((c.to_string(), d.parse().ok()?))
        })
        .collect();
    let used = templates::used_site_codes(root);
    let mut added = 0;
    for site in [Site::Mercari, Site::Rakuma, Site::YahooAuction, Site::YahooFlea] {
        let code = site.code().to_string();
        if !used.contains(&site.code()) {
            continue;
        }
        if state.iter().any(|(c, d)| *c == code && today().saturating_sub(*d) < RESEARCH_DAYS) {
            continue;
        }
        let snippets = client.search_snippets(&format!("{} 取引メッセージ 定型文 例文 購入者 出品者", site.name()), 6);
        if snippets.is_empty() {
            continue;
        }
        if let Some(out) = client.generate_long(&site_prompt(site, &snippets), 500, 180) {
            for (r, t) in parse_proposals(&out) {
                if templates::add_mined(root, &r, &t, &code) {
                    added += 1;
                }
            }
            state.retain(|(c, _)| *c != code);
            state.push((code, today()));
        }
    }
    let body: String = state.iter().map(|(c, d)| format!("{c}\t{d}")).collect::<Vec<_>>().join("\n");
    let _ = std::fs::write(root.join(RESEARCHED), body);
    added
}

/// 自動調査の1回分。aruaru-llm が動いていて、(1) は PC の中にあるときだけ行う。足した数を返す。
pub fn run(client: &Client, root: &Path) -> usize {
    if !client.alive() {
        return 0;
    }
    let mut n = 0;
    if client.is_local() {
        n += mine_user_texts(client, root);
    }
    n + research_sites(client, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_well_formed_proposals() {
        let out = "1. ありがとう|ご購入いただき、ありがとうございます。○○を丁寧に梱包して発送いたします。\n\
                   - ねさげ|お値下げは?\n\
                   説明: 以下です\n\
                   Thanks|英語の読みは使えない。これは十分に長い文章ですが読みが不正です。\n\
                   はっそう|本日、○○を発送いたしました。到着まで今しばらくお待ちください。\n\
                   はっそう|本日、○○を発送いたしました。到着まで今しばらくお待ちください。\n";
        let v = parse_proposals(out);
        assert_eq!(v.len(), 2, "{v:?}");
        assert_eq!(v[0].0, "ありがとう");
        assert_eq!(v[1].0, "はっそう");
    }

    #[test]
    fn prompts_carry_the_texts_and_never_a_raw_screen() {
        let p = user_prompt(&["会議の日程が決まりましたので、ご連絡いたします。".to_string()]);
        assert!(p.contains("会議の日程") && p.contains("読み|定型文"));
        let s = site_prompt(Site::Mercari, &["値下げ交渉のコメント例".to_string()]);
        assert!(s.contains("メルカリ") && s.contains("値下げ交渉"));
    }

    #[test]
    fn mined_templates_appear_as_frequent_and_per_site() {
        let dir = std::env::temp_dir().join(format!("aruaru-mine-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(templates::add_mined(&dir, "かいぎ", "会議の日程が決まりましたので、○○にご連絡いたします。", "-"));
        assert!(!templates::add_mined(&dir, "かいぎ", "会議の日程が決まりましたので、○○にご連絡いたします。", "-"), "同じ文は足さない");
        // 全サイトで、よく使う文として出る
        assert_eq!(templates::frequent(&dir, "かいぎ", 3).len(), 1);
        // サイト指定のものは、そのサイトの定型文として出る
        assert!(templates::add_mined(&dir, "ほうこく", "取引が完了しましたので、ご報告いたします。ありがとうございました。", "m"));
        let all = templates::load_all(&dir);
        assert!(templates::lookup("ほうこく", Site::Mercari, &all, 5).iter().any(|x| x.contains("取引が完了")));
        assert!(!templates::lookup("ほうこく", Site::Rakuma, &all, 5).iter().any(|x| x.contains("取引が完了")));
        assert!(templates::used_site_codes(&dir).contains(&'m'));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
