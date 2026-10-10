//! 未知の言語・プログラミング言語・フレームワークへの自動対応。
//!
//! 名前(例: `Svelte`)を受け取ると、aruaru-llm の検索 API で APIリファレンス・チートシートの文章を集め、
//! その中に繰り返し出てくる識別子(関数名・クラス名・キーワード)を語彙として `<データ領域>/vocab/<名前>.json` に保存する。
//! 保存した語彙は再起動なしで補完・訂正・言語判定に使われる。Qwen は使わない(1回数十秒かかるため)。
//! きっかけは `npm install svelte` のような記述、利用者の指示(右クリックメニュー・CLI)。

use crate::client::Client;
use crate::code::{self, UserFile};
use std::collections::HashMap;
use std::path::PathBuf;

pub const MIN_WORDS: usize = 12;
const MAX_WORDS: usize = 240;

/// ファイル名にできる形(英数字・`-`・`_`・`.`・`+`)にする。
pub fn slug(name: &str) -> String {
    let s: String = name.trim().to_lowercase().chars().map(|c| if c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '+') { c } else { '-' }).collect();
    s.trim_matches('-').to_string()
}

const STOP: &[&str] = &[
    "the", "and", "for", "with", "that", "this", "from", "have", "are", "you", "your", "not", "but", "can", "will", "how", "what", "when", "where", "which", "use",
    "using", "used", "more", "all", "any", "one", "new", "get", "see", "also", "its", "than", "into", "out", "about", "docs", "documentation", "guide", "tutorial",
    "example", "examples", "reference", "learn", "official", "site", "page", "home", "http", "https", "www", "com", "org", "html", "cheat", "sheet", "api", "free",
    "best", "top", "vs", "was", "were", "has", "had", "our", "their", "they", "them", "then", "these", "those", "just", "like", "such", "each", "other", "some",
];

/// 文章から、API らしい識別子を頻度順に取り出す。同じ語が2回以上、または camelCase / snake_case / ドット・@・$ 付きの語を拾う。
pub fn extract_words(text: &str) -> Vec<String> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for raw in text.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '@' | '.' | '-' | ':'))) {
        let t = raw.trim_matches(|c: char| matches!(c, '.' | '-' | ':'));
        let n = t.chars().count();
        if !(3..=32).contains(&n) || !t.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || matches!(c, '@' | '$')) {
            continue;
        }
        if STOP.contains(&t.to_lowercase().as_str()) || t.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        *freq.entry(t.to_string()).or_default() += 1;
    }
    let mut v: Vec<(String, usize)> = freq
        .into_iter()
        .filter(|(w, c)| {
            let camel = w.chars().skip(1).any(|c| c.is_ascii_uppercase()) && w.chars().any(|c| c.is_ascii_lowercase());
            let api_like = camel || w.contains('_') || w.contains('$') || w.starts_with('@') || w.contains('.');
            *c >= 2 || api_like
        })
        .collect();
    // 頻度 → API らしさ → 短い順
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.len().cmp(&b.0.len())).then(a.0.cmp(&b.0)));
    v.into_iter().map(|(w, _)| w).take(MAX_WORDS).collect()
}

/// 語彙を保存する。保存先のパスを返す。
pub fn save(name: &str, signals: Vec<String>, words: Vec<String>) -> std::io::Result<PathBuf> {
    let dir = code::vocab_dir();
    std::fs::create_dir_all(&dir)?;
    let p = dir.join(format!("{}.json", slug(name)));
    let f = UserFile { name: name.trim().to_string(), signals, words };
    std::fs::write(&p, serde_json::to_string_pretty(&f).unwrap_or_default())?;
    Ok(p)
}

pub fn remove(name: &str) -> std::io::Result<bool> {
    let p = code::vocab_dir().join(format!("{}.json", slug(name)));
    if p.is_file() {
        std::fs::remove_file(p)?;
        return Ok(true);
    }
    Ok(false)
}

/// 言語判定の手がかり: 名前そのもの・空白なし・`.拡張子`・`import 名前` など。
pub fn default_signals(name: &str) -> Vec<String> {
    let n = name.trim().to_lowercase();
    let compact: String = n.chars().filter(|c| !c.is_whitespace()).collect();
    let mut s = vec![n.clone()];
    if compact != n {
        s.push(compact.clone());
    }
    s.push(format!(".{}", compact.trim_end_matches(".js")));
    s.push(format!("import {compact}"));
    s.push(format!("from '{compact}'"));
    s.push(format!("{compact} "));
    s.sort();
    s.dedup();
    s
}

/// 集めた文章から語彙を作って保存する(通信を含まない部分。試験しやすいよう分離)。
pub fn learn_from_text(name: &str, text: &str) -> Result<(PathBuf, usize), String> {
    let name = name.trim();
    if name.chars().count() < 2 || name.chars().count() > 40 {
        return Err("名前は2〜40文字で指定してください".into());
    }
    let mut words = extract_words(text);
    // 名前そのもの・大文字小文字違いも語彙に入れる
    for w in [name.to_string(), name.to_lowercase()] {
        if !words.contains(&w) {
            words.insert(0, w);
        }
    }
    if words.len() < MIN_WORDS {
        return Err(format!("{name}: 手がかりになる語が少なすぎます({}語)。名前を正確に入力してください", words.len()));
    }
    let n = words.len();
    let p = save(name, default_signals(name), words).map_err(|e| e.to_string())?;
    Ok((p, n))
}

/// aruaru-llm の検索で資料を集め、語彙を作る。
pub fn learn(client: &Client, name: &str) -> Result<(PathBuf, usize), String> {
    if code::knows(name) && !code::list().iter().any(|(n, _, user)| n.eq_ignore_ascii_case(name) && *user) {
        return Err(format!("{name} は最初から対応しています"));
    }
    let queries = [
        format!("{name} API reference functions methods"),
        format!("{name} cheat sheet keywords syntax"),
        format!("{name} getting started example code"),
    ];
    let mut text = String::new();
    for q in &queries {
        for s in client.search_snippets(q, 10) {
            text.push_str(&s);
            text.push('\n');
        }
    }
    if text.trim().is_empty() {
        return Err("検索結果を取得できませんでした(aruaru-llm に接続できないか、検索が使えません)".into());
    }
    learn_from_text(name, &text)
}

/// `npm install svelte` / `pip install fastapi` / `cargo add serde` などから、導入しようとしているライブラリ名を拾う。
pub fn install_targets(text: &str) -> Vec<String> {
    const CMDS: &[&str] = &[
        "npm install ", "npm i ", "npm add ", "yarn add ", "pnpm add ", "pnpm install ", "bun add ", "pip install ", "pip3 install ", "cargo add ", "gem install ",
        "composer require ", "go get ", "dotnet add package ", "poetry add ", "uv add ", "flutter pub add ",
    ];
    let lower = text.to_lowercase();
    let mut out: Vec<String> = vec![];
    for cmd in CMDS {
        let mut from = 0;
        while let Some(i) = lower[from..].find(cmd) {
            let start = from + i + cmd.len();
            for tok in lower[start..].split_whitespace().take(4) {
                if tok.starts_with('-') {
                    continue;
                }
                // @scope/name@1.2 → name / github.com/a/b → b
                let t = tok.trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | ';' | '`'));
                let t = t.split('@').filter(|p| !p.is_empty()).next_back().map(|_| t).unwrap_or(t);
                let t = t.rsplit('/').next().unwrap_or(t);
                let t = t.split('@').next().unwrap_or(t).split("==").next().unwrap_or(t);
                let t = t.trim_matches(|c: char| matches!(c, '.' | '-' | '_'));
                if (3..=40).contains(&t.chars().count()) && t.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) && !out.iter().any(|o| o == t) {
                    out.push(t.to_string());
                }
                break;
            }
            from = start;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVELTE_TEXT: &str = "Svelte API reference. onMount runs after the component is first rendered to the DOM. \
        Use writable stores: import { writable, derived, readable } from 'svelte/store'. The $state rune declares reactive state; \
        $derived computes values; $effect runs side effects. Use bind:value for two-way binding, {#if} {#each} blocks, \
        onDestroy, beforeUpdate, afterUpdate, tick and createEventDispatcher. SvelteKit adds +page.svelte, +layout.svelte and load functions. \
        onMount onMount writable writable derived derived";

    #[test]
    fn extracts_api_like_words() {
        let w = extract_words(SVELTE_TEXT);
        for want in ["onMount", "writable", "derived", "onDestroy", "beforeUpdate", "createEventDispatcher", "$state", "$derived"] {
            assert!(w.iter().any(|x| x == want), "{want} in {w:?}");
        }
        assert!(!w.iter().any(|x| x.eq_ignore_ascii_case("the") || x.eq_ignore_ascii_case("and")), "ストップワードは除く");
        assert!(w.len() <= MAX_WORDS);
    }

    #[test]
    fn learns_and_is_used_immediately() {
        let _g = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join(format!("aruaru-uv-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::env::set_var("ARUARU_DATA_DIR", &dir);
        let (p, n) = learn_from_text("Svelte", SVELTE_TEXT).unwrap();
        assert!(p.is_file() && n >= MIN_WORDS);
        std::thread::sleep(std::time::Duration::from_millis(2100));
        assert!(code::knows("svelte"));
        let ctx = crate::mode::Context { before: "import { onMount } from 'svelte'".into(), ..Default::default() };
        let langs = code::detect(&ctx);
        assert!(code::complete("createEv", &langs, 3).contains(&"createEventDispatcher".to_string()));
        assert!(remove("Svelte").unwrap());
        assert!(!remove("Svelte").unwrap());
        std::env::remove_var("ARUARU_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_thin_or_invalid_input() {
        assert!(learn_from_text("X", "a b c").is_err());
        assert!(learn_from_text("Foo", "foo bar").is_err(), "語が少なすぎるなら保存しない");
    }

    #[test]
    fn finds_install_targets() {
        let t = install_targets("$ npm install svelte --save\n$ pip install fastapi==0.110 uvicorn\ncargo add serde\n$ npm i @sveltejs/kit@latest\ngo get github.com/gin-gonic/gin");
        for want in ["svelte", "fastapi", "serde", "kit", "gin"] {
            assert!(t.iter().any(|x| x == want), "{want} in {t:?}");
        }
        assert!(install_targets("今日は npm の話をしました").is_empty());
    }

    #[test]
    fn slug_is_filesystem_safe() {
        assert_eq!(slug("Next.js"), "next.js");
        assert_eq!(slug("C++ / STL"), "c++---stl");
        assert!(!slug("../etc/passwd").contains('/'));
    }
}
