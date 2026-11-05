//! 文脈による候補の並べ替え(「もっと賢く」2026-11-06)。同じ読みの語(こうちょう → 校長 / 好調 / 紅潮 …)で、
//! 直前・直後の文章、画面の文字、手元の知識ベース(DATABASE: 消費税・核武装などの回答、自動調査した出典つき要約)に
//! 出てくる語を、先に出す。辞書の並びは、文脈に手がかりが無いとき(得点がすべて 0)は一切変えない。
//!
//! 安全側: 並べ替えるのは辞書の上位候補(漢字を含む語)だけで、得点が同じ語どうしの順序は辞書のまま。
//! 先頭の語を入れ替えるのは、文脈にその語の手がかりがあり、先頭には無いときだけ。

use crate::mode::Context;

fn has_kanji(s: &str) -> bool {
    s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

fn occurrences(hay: &str, needle: &str, cap: usize) -> usize {
    if needle.is_empty() {
        return 0;
    }
    hay.matches(needle).take(cap).count()
}

/// 文脈の手がかりの文字列(近い文章ほど重く数える)。
pub struct Evidence {
    near: String,
    screen: String,
    knowledge: String,
}

impl Evidence {
    pub fn from_context(ctx: &Context, knowledge: String) -> Self {
        let tail: String = {
            let n = ctx.before.chars().count();
            ctx.before.chars().skip(n.saturating_sub(300)).collect()
        };
        let head: String = ctx.after.chars().take(150).collect();
        Self { near: format!("{tail}\n{head}"), screen: ctx.screen_text.chars().take(4000).collect(), knowledge }
    }

    pub fn is_empty(&self) -> bool {
        self.near.trim().is_empty() && self.screen.trim().is_empty() && self.knowledge.trim().is_empty()
    }

    pub fn score(&self, cand: &str) -> usize {
        if cand.chars().count() < 2 || !has_kanji(cand) {
            return 0;
        }
        3 * occurrences(&self.near, cand, 3) + 2 * occurrences(&self.screen, cand, 3) + occurrences(&self.knowledge, cand, 3)
    }
}

/// 辞書の上位`window`件のうち、文脈の手がかりが多い語を前へ。得点が同じなら元の順序のまま。
pub fn rerank(cands: &mut Vec<String>, ev: &Evidence, window: usize) {
    if ev.is_empty() || cands.len() < 2 {
        return;
    }
    let w = window.min(cands.len());
    let mut head: Vec<(usize, usize, String)> = cands.drain(..w).enumerate().map(|(i, c)| (ev.score(&c), i, c)).collect();
    if head.iter().all(|(s, _, _)| *s == 0) {
        for (_, _, c) in head.drain(..).rev() {
            cands.insert(0, c);
        }
        return;
    }
    head.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, _, c) in head.into_iter().rev() {
        cands.insert(0, c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(before: &str, screen: &str) -> Context {
        Context { before: before.into(), screen_text: screen.into(), ..Default::default() }
    }
    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn context_pulls_the_fitting_homophone_forward() {
        let mut c = v(&["好調", "校長", "紅潮"]);
        let ev = Evidence::from_context(&ctx("小学校の校長先生が", ""), String::new());
        rerank(&mut c, &ev, 8);
        assert_eq!(c[0], "校長");
        assert_eq!(c[1], "好調", "他の順序は辞書のまま");
    }

    #[test]
    fn no_clue_keeps_dictionary_order() {
        let mut c = v(&["対応", "太王四神記", "大王", "滞欧"]);
        let ev = Evidence::from_context(&ctx("今日はいい天気ですね", ""), String::new());
        rerank(&mut c, &ev, 8);
        assert_eq!(c, v(&["対応", "太王四神記", "大王", "滞欧"]));
    }

    #[test]
    fn database_knowledge_counts_as_a_clue() {
        let mut c = v(&["格武装", "核武装"]);
        let ev = Evidence::from_context(&ctx("", ""), "日本では非核三原則があり、核武装については…".into());
        rerank(&mut c, &ev, 8);
        assert_eq!(c[0], "核武装");
    }

    #[test]
    fn kana_and_single_chars_are_never_boosted() {
        let ev = Evidence::from_context(&ctx("こうちょう 校 校 校", ""), String::new());
        assert_eq!(ev.score("こうちょう"), 0);
        assert_eq!(ev.score("校"), 0);
    }
}
