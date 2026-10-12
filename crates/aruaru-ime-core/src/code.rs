//! プログラミング用語の入力予測と誤字訂正。
//!
//! 直前の文章・画面の文字から、いま書いている言語・フレームワークを判定し(Python / Rust / TypeScript / React / PHP / Ruby / Perl / COBOL ほか)、
//! その語彙の中から補完と綴りの訂正を行う。日本語のローマ字入力(「ど」「いん」など)を奪わないよう、
//! 言語を判定できたときだけ補完・訂正し、短い語や日本語として自然な綴りは英字扱いにしない。
//!
//! 語彙は「組み込み(`codevocab`)」+「利用者が追加・自動学習したもの(`<データ領域>/vocab/*.json`)」。
//! 未知の言語・フレームワークは `uservocab` が aruaru-llm の検索結果から語彙を作って追加する(再起動不要で反映)。

use crate::codevocab::LANGS;
use crate::mode::Context;
use crate::romaji;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub struct Fix {
    pub word: String,
    /// 確信が高く、自動で置き換えてよい(距離1・5文字以上・最良が1つだけ)。
    pub confident: bool,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    signals: Vec<String>,
    words: Vec<String>,
    pub user: bool,
}

impl Entry {
    pub fn new(name: &str, signals: Vec<String>, words: Vec<String>, user: bool) -> Self {
        Self { name: name.to_string(), signals: signals.into_iter().map(|s| s.to_lowercase()).collect(), words, user }
    }
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}

fn base() -> &'static Vec<Entry> {
    static BASE: OnceLock<Vec<Entry>> = OnceLock::new();
    BASE.get_or_init(|| {
        LANGS
            .iter()
            .map(|(n, sig, words)| {
                Entry::new(n, sig.split('|').filter(|s| !s.is_empty()).map(str::to_string).collect(), words.split_whitespace().map(str::to_string).collect(), false)
            })
            .collect()
    })
}

pub fn vocab_dir() -> std::path::PathBuf {
    crate::components::data_dir().join("vocab")
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserFile {
    pub name: String,
    pub signals: Vec<String>,
    pub words: Vec<String>,
}

struct Cache {
    checked: Instant,
    stamp: Option<SystemTime>,
    all: Arc<Vec<Entry>>,
}

fn dir_stamp(dir: &std::path::Path) -> Option<SystemTime> {
    // ディレクトリ内の最新の更新時刻(ファイルの追加・更新・削除に追従する)
    let mut newest = std::fs::metadata(dir).and_then(|m| m.modified()).ok();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if let Ok(t) = e.metadata().and_then(|m| m.modified()) {
                newest = Some(newest.map_or(t, |n| n.max(t)));
            }
        }
    }
    newest
}

fn load_all(dir: &std::path::Path) -> Vec<Entry> {
    let mut v: Vec<Entry> = base().clone();
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut files: Vec<_> = rd.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "json")).collect();
        files.sort_by_key(|e| e.file_name());
        for e in files {
            if let Some(f) = std::fs::read_to_string(e.path()).ok().and_then(|t| serde_json::from_str::<UserFile>(&t).ok()) {
                v.push(Entry::new(&f.name, f.signals, f.words, true));
            }
        }
    }
    v
}

/// 組み込み+利用者追加の全語彙(2秒ごとに利用者追加分の変更を確認する)。
pub fn all() -> Arc<Vec<Entry>> {
    static CACHE: Mutex<Option<Cache>> = Mutex::new(None);
    let mut g = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let dir = vocab_dir();
    if let Some(c) = g.as_ref() {
        if c.checked.elapsed() < Duration::from_secs(2) {
            return Arc::clone(&c.all);
        }
    }
    let stamp = dir_stamp(&dir);
    if let Some(c) = g.as_mut() {
        if c.stamp == stamp {
            c.checked = Instant::now();
            return Arc::clone(&c.all);
        }
    }
    let all = Arc::new(load_all(&dir));
    *g = Some(Cache { checked: Instant::now(), stamp, all: Arc::clone(&all) });
    all
}

fn words(e: &Entry) -> impl Iterator<Item = &str> {
    e.words.iter().map(String::as_str)
}

/// 画面・直前の文章から、書いている言語を判定する(手がかりが多い順)。結果は `all()` の添字。
pub fn detect(ctx: &Context) -> Vec<usize> {
    let tail: String = ctx.before.chars().rev().take(600).collect::<Vec<_>>().into_iter().rev().collect();
    let screen: String = ctx.screen_text.chars().take(3000).collect();
    let hay = format!("{tail}\n{screen}").to_lowercase();
    let mut scored: Vec<(usize, usize)> = all()
        .iter()
        .enumerate()
        .map(|(i, e)| (i, e.signals.iter().filter(|s| hay.contains(s.as_str())).count()))
        .filter(|(_, n)| *n > 0)
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut out: Vec<usize> = scored.into_iter().map(|(i, _)| i).collect();
    // 利用者が「開発で使う」と選んだもの。文脈の手がかりが無くても対象にする(手がかりがあるものの後ろ)
    for name in &crate::settings::Settings::load().dev_stack {
        let n = name.trim().to_lowercase();
        if let Some(i) = all().iter().position(|e| e.name.to_lowercase() == n) {
            if !out.contains(&i) {
                out.push(i);
            }
        }
    }
    out
}

pub fn lang_name(i: usize) -> String {
    all().get(i).map(|e| e.name.clone()).unwrap_or_default()
}

/// 登録されている言語・フレームワークの名前と語数。
pub fn list() -> Vec<(String, usize, bool)> {
    all().iter().map(|e| (e.name.clone(), e.word_count(), e.user)).collect()
}

/// その名前(大文字小文字を区別しない)の言語・フレームワークをすでに知っているか。
pub fn knows(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    all().iter().any(|e| e.name.to_lowercase() == n || e.signals.iter().any(|s| *s == n))
}

fn lower(s: &str) -> String {
    s.to_lowercase()
}

/// どの言語の語彙にも載っている(完全一致、大文字小文字を区別しない)語か。
pub fn is_code_word(word: &str) -> bool {
    let w = lower(word);
    all().iter().any(|e| words(e).any(|v| lower(v) == w))
}

/// 英字として扱ってよいプログラミング用語か。日本語として自然な綴り(none→のね など)は、言語を判定できたときだけ。
pub fn is_code_word_for_alpha(word: &str, langs_detected: bool) -> bool {
    if word.chars().count() < 3 || !is_code_word(word) {
        return false;
    }
    langs_detected || !romaji::is_romaji_progress(word)
}

/// 前方一致の補完(言語の判定順・語彙の人気順)。入力そのものと同じ語は除く。
pub fn complete(prefix: &str, langs: &[usize], max: usize) -> Vec<String> {
    let p = lower(prefix);
    if p.chars().count() < 2 || langs.is_empty() {
        return vec![];
    }
    let snapshot = all();
    let mut out: Vec<String> = vec![];
    for &l in langs {
        let Some(e) = snapshot.get(l) else { continue };
        for v in words(e) {
            if lower(v).starts_with(&p) && lower(v) != p && !out.iter().any(|o| lower(o) == lower(v)) {
                out.push(v.to_string());
                if out.len() >= max {
                    return out;
                }
            }
        }
    }
    out
}

/// 最適文字列アライメント距離(置換・挿入・削除・隣り合う文字の入れ替えを各1)。
pub fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[n][m]
}

/// 綴りの訂正候補(`fucntion` → `function`)。語彙に載っている語は訂正しない。言語を判定できないときは何もしない。
pub fn correct(word: &str, langs: &[usize]) -> Vec<Fix> {
    let w = lower(word);
    let len = w.chars().count();
    if len < 4 || langs.is_empty() || is_code_word(word) {
        return vec![];
    }
    let max_d = if len >= 9 { 2 } else { 1 };
    let snapshot = all();
    let mut best: Vec<(usize, String)> = vec![];
    for &l in langs {
        let Some(e) = snapshot.get(l) else { continue };
        for v in words(e) {
            let lv = lower(v);
            if lv.chars().count().abs_diff(len) > max_d || lv.starts_with(&w) {
                continue; // 打ちかけの語(前方一致)は、訂正ではなく補完の対象
            }
            let d = distance(&w, &lv);
            if d >= 1 && d <= max_d && !best.iter().any(|(_, x)| lower(x) == lv) {
                best.push((d, v.to_string()));
            }
        }
    }
    best.sort_by_key(|(d, _)| *d); // 同じ距離では、言語の判定順・語彙の人気順を保つ(安定ソート)
    let min = best.first().map(|(d, _)| *d);
    let n_min = best.iter().filter(|(d, _)| Some(*d) == min).count();
    best.into_iter()
        .take(4)
        .enumerate()
        .map(|(i, (d, v))| Fix { confident: i == 0 && d == 1 && n_min == 1 && len >= 5, word: v })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(before: &str) -> Context {
        Context { before: before.into(), ..Default::default() }
    }

    fn names(ctx: &Context) -> Vec<String> {
        detect(ctx).into_iter().map(lang_name).collect()
    }

    fn has(ctx_text: &str, lang: &str) -> bool {
        names(&ctx(ctx_text)).iter().any(|n| n == lang)
    }

    #[test]
    fn detects_languages_from_context() {
        assert_eq!(names(&ctx("fn main() {\n    let mut x = Vec::new();"))[0], "Rust");
        assert_eq!(names(&ctx("def hello(self):\n    print(1)"))[0], "Python");
        assert!(has("<?php\nnamespace App;\n$this->name", "PHP"));
        assert!(has("IDENTIFICATION DIVISION.\nPROGRAM-ID. HELLO.", "COBOL"));
        assert!(has("my $x = 1;\nuse strict;", "Perl"));
        assert!(has("const [a, setA] = useState(0);\nimport React from 'react'", "React"));
        assert!(has("interface User { name: string }", "TypeScript"));
        assert!(has("class Foo\n  attr_accessor :a\nend", "Ruby"));
        assert!(names(&ctx("今日は天気がいいですね")).is_empty(), "普通の文章では言語を判定しない");
    }

    #[test]
    fn detects_from_screen_text_too() {
        let c = Context { screen_text: "main.rs  - Visual Studio Code\nfn main() {}".into(), ..Default::default() };
        assert!(names(&c).iter().any(|n| n == "Rust"));
    }

    #[test]
    fn completes_by_prefix() {
        let rust = detect(&ctx("fn main() { let mut x = 1; }"));
        assert_eq!(complete("unwr", &rust, 5), vec!["unwrap"]);
        let py = detect(&ctx("def f(self):"));
        assert!(complete("enum", &py, 5).contains(&"enumerate".to_string()));
        let cobol = detect(&ctx("IDENTIFICATION DIVISION."));
        assert!(complete("WORKING", &cobol, 5).contains(&"WORKING-STORAGE".to_string()));
        assert!(complete("use", &detect(&ctx("const [a, setA] = useState(0); import react")), 8).iter().any(|w| w == "useEffect"));
        assert!(complete("unwr", &[], 5).is_empty(), "言語を判定できなければ補完しない");
    }

    #[test]
    fn corrects_typos_confidently_only_when_clear() {
        let js = detect(&ctx("function f() { const x = 1; console.log(x) }"));
        let f = correct("fucntion", &js);
        assert_eq!(f[0].word, "function");
        assert!(f[0].confident, "隣り合う文字の入れ替えで最良が1つ → 自動訂正してよい");
        let py = detect(&ctx("def f(self):\n    import os"));
        assert_eq!(correct("improt", &py)[0].word, "import");
        assert_eq!(correct("retrun", &detect(&ctx("def f(self):")))[0].word, "return");
        assert!(correct("print", &py).is_empty());
        assert!(correct("fnc", &js).is_empty());
        assert!(correct("fucntion", &[]).is_empty());
    }

    #[test]
    fn does_not_hijack_japanese_romaji() {
        for w in ["do", "in", "is", "none", "main", "data"] {
            assert!(!is_code_word_for_alpha(w, false), "{w}");
        }
        assert!(is_code_word_for_alpha("none", true));
        assert!(is_code_word_for_alpha("function", false));
        assert!(is_code_word_for_alpha("println", false));
    }

    #[test]
    fn new_frameworks_and_databases_are_known() {
        assert!(has("export default defineNuxtConfig({})", "Nuxt.js"));
        assert!(has("import Link from 'next/link'\n'use client'", "Next.js"));
        assert!(has("query { users { id } }\ntype Query {", "GraphQL"));
        assert!(has("wgc subgraph publish --namespace prod", "WunderGraph Cosmo"));
        assert!(has("SELECT * FROM users LIMIT 10;\nCREATE TABLE t (id int)", "SQL"));
        assert!(has("CREATE TABLE a (id SERIAL PRIMARY KEY); -- postgres\npsql -d db", "PostgreSQL"));
        assert!(has("SELECT SYSDATE FROM DUAL; -- oracle\nVARCHAR2(10)", "Oracle PL/SQL"));
        assert!(has("PRAGMA journal_mode=WAL; -- sqlite", "SQLite"));
        assert!(has("ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 -- mysql", "MySQL/MariaDB"));
        assert!(has("db.collection.find({}) // mongodb", "NoSQL/Redis"));
        assert!(has("<!DOCTYPE html>\n<html lang=\"ja\">\n<section>", "HTML5"));
        assert!(has("a { display: flex; margin: 0 } @media (min-width: 1px)", "CSS3"));
        assert!(has("<div class=\"container\"><button class=\"btn btn-primary\">", "Bootstrap"));
        let g = detect(&ctx("query { users { id } }\ntype Query {"));
        assert!(complete("subscr", &g, 3).contains(&"subscription".to_string()));
        assert_eq!(correct("mutaiton", &g)[0].word, "mutation");
        let n = detect(&ctx("export default defineNuxtConfig({})"));
        assert!(complete("useAsy", &n, 3).contains(&"useAsyncData".to_string()));
        let sql = detect(&ctx("SELECT * FROM users WHERE id = 1 GROUP BY name"));
        assert!(complete("INNE", &sql, 3).contains(&"INNER".to_string()));
        assert_eq!(correct("SELCT", &sql)[0].word, "SELECT");
        assert!(correct("INT", &sql).is_empty(), "短い語は訂正しない");
        assert_eq!(correct("delte", &detect(&ctx("DELETE FROM users WHERE id = 1; SELECT * FROM t")))[0].word.to_lowercase(), "delete");
        let css = detect(&ctx("a { display: flex; margin: 0 } @media (min-width: 1px)"));
        assert!(complete("justify-c", &css, 3).contains(&"justify-content".to_string()));
        let bs = detect(&ctx("<div class=\"container\"><button class=\"btn btn-primary\">"));
        assert!(complete("navbar-ex", &bs, 3).contains(&"navbar-expand-lg".to_string()));
    }

    #[test]
    fn prefix_is_completion_not_typo() {
        let js = detect(&ctx("function f() { const x = 1 }"));
        assert!(correct("retru", &js).is_empty(), "retru は return の前方一致なので訂正ではなく補完");
    }

    #[test]
    fn distance_basics() {
        assert_eq!(distance("function", "fucntion"), 1);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("abc", "abc"), 0);
    }

    #[test]
    fn user_added_vocab_is_picked_up_without_restart() {
        let _g = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("aruaru-vocab-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::env::set_var("ARUARU_DATA_DIR", &dir);
        assert!(!knows("Svelte"));
        std::fs::create_dir_all(vocab_dir()).unwrap();
        let f = UserFile { name: "Svelte".into(), signals: vec!["svelte".into(), ".svelte".into()], words: vec!["onMount".into(), "writable".into(), "derived".into(), "$state".into()] };
        std::fs::write(vocab_dir().join("svelte.json"), serde_json::to_string(&f).unwrap()).unwrap();
        std::thread::sleep(Duration::from_millis(2100)); // キャッシュの確認間隔
        assert!(knows("svelte"));
        let langs = detect(&ctx("<script>\n import { onMount } from 'svelte'\n</script>"));
        assert!(names(&ctx("import { onMount } from 'svelte'")).iter().any(|n| n == "Svelte"));
        assert!(complete("onMo", &langs, 3).contains(&"onMount".to_string()));
        std::env::remove_var("ARUARU_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
