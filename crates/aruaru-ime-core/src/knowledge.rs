//! 知識ベース辞書。open-english の `knowledge.json` と同じ形式(id / text / tags / source / created_unix)で、
//! 利用者が覚えさせた知識を `<データ領域>/knowledge.json` に持ち、キーワードで検索する(変換候補の補足説明に使う)。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub id: u64,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub created_unix: u64,
}

pub fn path(root: &Path) -> PathBuf {
    root.join("knowledge.json")
}

pub fn load(p: &Path) -> Vec<Item> {
    std::fs::read_to_string(p).ok().and_then(|t| sonic_rs::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
}

fn save(p: &Path, v: &[Item]) -> std::io::Result<()> {
    std::fs::write(p, serde_json::to_string_pretty(v).unwrap_or_default())
}

/// 追加する。同じ文は重複させない。追加したら true。
pub fn add(p: &Path, text: &str, tags: &[String], source: &str) -> std::io::Result<bool> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > 2000 {
        return Ok(false);
    }
    let mut v = load(p);
    if v.iter().any(|i| i.text == text) {
        return Ok(false);
    }
    let id = v.iter().map(|i| i.id).max().unwrap_or(0) + 1;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    v.push(Item { id, text: text.to_string(), tags: tags.to_vec(), source: source.to_string(), created_unix: now });
    save(p, &v)?;
    Ok(true)
}

/// open-english などの同形式ファイルを取り込む。増えた件数を返す。
pub fn import(p: &Path, from: &Path) -> std::io::Result<usize> {
    let mut n = 0;
    for i in load(from) {
        if add(p, &i.text, &i.tags, &i.source)? {
            n += 1;
        }
    }
    Ok(n)
}

/// キーワード(空白区切り、すべて含む)で検索。本文とタグが対象。新しい順。
pub fn search(items: &[Item], query: &str, max: usize) -> Vec<Item> {
    let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
    if words.is_empty() {
        return vec![];
    }
    let mut v: Vec<Item> = items
        .iter()
        .filter(|i| {
            let hay = format!("{} {}", i.text.to_lowercase(), i.tags.join(" ").to_lowercase());
            words.iter().all(|w| hay.contains(w))
        })
        .cloned()
        .collect();
    v.sort_by(|a, b| b.created_unix.cmp(&a.created_unix));
    v.truncate(max);
    v
}

/// 候補ウィンドウの補足表示用。選んでいる候補(語)を含む知識を1件、短くして返す。ファイルが変わったときだけ読み直す。
pub fn note_for(word: &str) -> Option<String> {
    use std::sync::Mutex;
    use std::time::SystemTime;
    static CACHE: Mutex<Option<(SystemTime, Vec<Item>)>> = Mutex::new(None);
    let w = word.trim();
    // 1文字の英数字では引かない(漢字・かな1字は引く)
    if w.is_empty() || (w.chars().count() < 2 && w.is_ascii()) {
        return None;
    }
    let p = path(&crate::components::data_dir());
    let mt = std::fs::metadata(&p).and_then(|m| m.modified()).ok()?;
    let mut g = CACHE.lock().ok()?;
    if g.as_ref().map(|(t, _)| *t != mt).unwrap_or(true) {
        *g = Some((mt, load(&p)));
    }
    let items = &g.as_ref()?.1;
    let hit = items.iter().rev().find(|i| i.text.contains(w) || i.tags.iter().any(|t| t == w))?;
    let t: String = hit.text.chars().take(60).collect();
    Some(if hit.text.chars().count() > 60 { format!("{t}…") } else { t })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_search_import() {
        let d = std::env::temp_dir().join(format!("aruaru-kb-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        let p = d.join("k.json");
        let _ = std::fs::remove_file(&p);
        assert!(add(&p, "桜の開花は3月下旬", &["春".into()], "").unwrap());
        assert!(!add(&p, "桜の開花は3月下旬", &[], "").unwrap(), "重複は足さない");
        assert!(!add(&p, "  ", &[], "").unwrap());
        let items = load(&p);
        assert_eq!(search(&items, "桜 開花", 5).len(), 1);
        assert_eq!(search(&items, "春", 5).len(), 1, "タグでも引ける");
        assert!(search(&items, "夏", 5).is_empty());
        let q = d.join("q.json");
        let _ = std::fs::remove_file(&q);
        assert_eq!(import(&q, &p).unwrap(), 1);
    }
}
