//! 候補の優先順位の調整。読みごとに「この順で出したい」表記を持つ(組み込み + 利用者の `prefer.tsv`)と、
//! ハイフン・ダッシュ類のような記号の一覧(はいふん → - － ‐ – — ― ー …)。

use std::path::Path;

/// 読み → 優先して出す表記(この順)。組み込み分。利用者は `<データ領域>/prefer.tsv` に `読み<TAB>表記,表記,…` で足せる。
pub const BUILTIN: &str = "\
きゅう\t急,休暇,級,休息,休息時間,急速,球速\n\
きゅうそ\t休息,休息時間,窮鼠\n\
たいおう\t対応,太王四神記,大王,滞欧\n\
でない\t出ない\n\
でなかった\t出なかった\n";

/// 記号の一覧。読み → 並べる文字(1文字ずつ)。
pub const SYMBOLS: &[(&str, &str)] = &[
    ("はいふん", "-－‐‑–—―ーｰ−─"),
    ("だっしゅ", "–—―‐-－ー─"),
    ("まいなす", "-−－–ー"),
    ("ちょうおん", "ーｰ〜～―"),
    ("ぷらす", "+＋➕±﹢"),
    ("なみせん", "〜～∼~"),
    ("てん", "・･．.、,"),
];

/// ハイフン類。`-` を打ったときの候補に足す。
pub const HYPHENS: &str = "-－‐‑–—―−─";

pub fn load_user(root: &Path) -> String {
    std::fs::read_to_string(root.join("prefer.tsv")).unwrap_or_default().trim_start_matches('\u{feff}').to_string()
}

/// `reading` に対する優先表記(利用者の指定が先、組み込みが次)。
pub fn preferred(reading: &str, user_tsv: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for src in [user_tsv, BUILTIN] {
        for l in src.lines() {
            if let Some((r, v)) = l.split_once('\t') {
                if r.trim() == reading {
                    for s in v.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                        if !out.iter().any(|x| x == s) {
                            out.push(s.to_string());
                        }
                    }
                }
            }
        }
    }
    out
}

pub fn symbols_for(reading: &str) -> Vec<String> {
    SYMBOLS.iter().find(|(r, _)| *r == reading).map(|(_, s)| s.chars().map(|c| c.to_string()).collect()).unwrap_or_default()
}

/// 候補列を、優先表記が指定の順で先頭に来るように並べ替える(無い表記も足す)。
pub fn reorder(cands: &mut Vec<String>, prefer: &[String]) {
    if prefer.is_empty() {
        return;
    }
    cands.retain(|c| !prefer.contains(c));
    for (i, p) in prefer.iter().enumerate() {
        cands.insert(i, p.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_order_wins_and_missing_are_added() {
        let p = preferred("きゅう", "きゅう\t九,急\n");
        assert_eq!(&p[..3], ["九", "急", "休暇"]);
        let mut c: Vec<String> = vec!["級".into(), "急".into(), "球".into()];
        reorder(&mut c, &["急".into(), "休暇".into()]);
        assert_eq!(c, vec!["急", "休暇", "級", "球"]);
    }

    #[test]
    fn hyphens_have_many_kinds() {
        let h = symbols_for("はいふん");
        assert!(h.contains(&"-".to_string()) && h.contains(&"－".to_string()) && h.contains(&"‐".to_string()) && h.len() >= 8);
    }
}
