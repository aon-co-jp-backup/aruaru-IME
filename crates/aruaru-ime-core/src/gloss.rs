//! 日本語 → 英語の訳語辞書(JMdict)。変換候補を「日本語」と「英語」の2系統で出すために使う。
//!
//! 元データは JMdict(© EDRDG、CC BY-SA 4.0)を JSON にした `scriptin/jmdict-simplified`。
//! インストール時(と初回起動時)に GitHub Releases から自動で取得して、`<データ領域>/glossary.tsv`(表記・読み・訳語)へ変換する。
//! 辞書データの出典・ライセンス表示は同じ場所の `JMdict-LICENSE.txt` に書く。IME 本体のライセンス(MIT/Apache)には影響しない。

use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const REPO: &str = "scriptin/jmdict-simplified";

#[derive(Deserialize)]
struct Root {
    words: Vec<Word>,
}
#[derive(Deserialize)]
struct Word {
    #[serde(default)]
    kanji: Vec<Kanji>,
    #[serde(default)]
    kana: Vec<Kana>,
    #[serde(default)]
    sense: Vec<Sense>,
}
#[derive(Deserialize)]
struct Kanji {
    #[serde(default)]
    common: bool,
    text: String,
}
#[derive(Deserialize)]
struct Kana {
    #[serde(default)]
    common: bool,
    text: String,
    #[serde(default, rename = "appliesToKanji")]
    applies_to_kanji: Vec<String>,
}
#[derive(Deserialize)]
struct Sense {
    #[serde(default, rename = "appliesToKanji")]
    applies_to_kanji: Vec<String>,
    #[serde(default, rename = "appliesToKana")]
    applies_to_kana: Vec<String>,
    #[serde(default)]
    gloss: Vec<Gloss>,
}
#[derive(Deserialize)]
struct Gloss {
    text: String,
}

fn applies(list: &[String], text: &str) -> bool {
    list.is_empty() || list.iter().any(|x| x == "*" || x == text)
}

/// 1語の訳語: 意味(sense)を最大3つ、各2語まで、重複を除いて最大6語。
fn glosses_for(w: &Word, kanji: Option<&str>, kana: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for s in w.sense.iter().filter(|s| kanji.map_or(true, |k| applies(&s.applies_to_kanji, k)) && applies(&s.applies_to_kana, kana)).take(3) {
        for g in s.gloss.iter().take(2) {
            let t = g.text.trim();
            if !t.is_empty() && t.chars().count() <= 48 && !t.contains('\t') && !t.contains('|') && !out.iter().any(|o| o == t) {
                out.push(t.to_string());
            }
        }
    }
    out.truncate(6);
    out
}

/// JMdict(jmdict-simplified の JSON)から `表記\t読み\t訳語|訳語…` の行を作る。よく使われる語を先に並べる。
pub fn build_tsv(json: &[u8]) -> Result<String, String> {
    let root: Root = serde_json::from_slice(json).map_err(|e| format!("JMdict を読めません: {e}"))?;
    let mut common_lines: Vec<String> = vec![];
    let mut other_lines: Vec<String> = vec![];
    for w in &root.words {
        let word_common = w.kanji.iter().any(|k| k.common) || w.kana.iter().any(|k| k.common);
        let mut push = |surface: &str, reading: &str, g: Vec<String>, common: bool| {
            if g.is_empty() {
                return;
            }
            let line = format!("{}\t{}\t{}", surface, crate::romaji::kata_to_hira(reading), g.join("|"));
            if common { common_lines.push(line) } else { other_lines.push(line) }
        };
        for k in &w.kanji {
            for r in w.kana.iter().filter(|r| applies(&r.applies_to_kanji, &k.text)) {
                push(&k.text, &r.text, glosses_for(w, Some(&k.text), &r.text), word_common && k.common);
            }
        }
        // かな書きの語(漢字が無い語・よく使われる語)は、かな表記でも引けるようにする
        if w.kanji.is_empty() || word_common {
            for r in &w.kana {
                push(&r.text, &r.text, glosses_for(w, None, &r.text), word_common && r.common);
            }
        }
    }
    common_lines.extend(other_lines);
    Ok(common_lines.join("\n"))
}

/// zip(中身は JMdict の JSON 1つ)から訳語辞書(TSV)を作る。
pub fn build_from_zip(zip_bytes: &[u8]) -> Result<String, String> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).map_err(|e| e.to_string())?;
    let mut json = Vec::new();
    z.by_index(0).map_err(|e| e.to_string())?.read_to_end(&mut json).map_err(|e| e.to_string())?;
    build_tsv(&json)
}

#[derive(Default)]
pub struct Glossary {
    /// 表記 → [(読み, 訳語…)](よく使われる順)。
    map: HashMap<String, Vec<(String, Vec<String>)>>,
    /// 英語の訳語(小文字) → 日本語の表記(よく使われる順)。英語から日本語への翻訳に使う。
    rev: HashMap<String, Vec<String>>,
}

/// 英語の訳語を引くときの正規化: 小文字、先頭の `to `(動詞)・`a `/`an `/`the ` を除く。
fn norm_en(s: &str) -> String {
    let t = s.trim().to_lowercase();
    for p in ["to ", "a ", "an ", "the "] {
        if let Some(r) = t.strip_prefix(p) {
            return r.trim().to_string();
        }
    }
    t
}

impl Glossary {
    pub fn parse(tsv: &str) -> Self {
        let mut map: HashMap<String, Vec<(String, Vec<String>)>> = HashMap::with_capacity(tsv.len() / 60);
        let mut rev: HashMap<String, Vec<String>> = HashMap::with_capacity(tsv.len() / 80);
        for l in tsv.lines() {
            let mut it = l.splitn(3, '\t');
            if let (Some(s), Some(r), Some(g)) = (it.next(), it.next(), it.next()) {
                let glosses: Vec<String> = g.split('|').map(str::to_string).collect();
                // 逆引き: 各訳語の最初の3つ。かな書きだけの表記(読みと同じ)は、漢字などの表記があるときは入れない
                for gl in glosses.iter().take(3) {
                    let k = norm_en(gl);
                    if k.is_empty() || k.chars().count() > 28 || k.split(' ').count() > 3 {
                        continue;
                    }
                    let v = rev.entry(k).or_default();
                    if v.len() < 8 && !v.iter().any(|x| x == s) {
                        v.push(s.to_string());
                    }
                }
                let v = map.entry(s.to_string()).or_default();
                if v.len() < 6 {
                    v.push((r.to_string(), glosses));
                }
            }
        }
        Self { map, rev }
    }

    pub fn load(path: &Path) -> Option<Self> {
        std::fs::read_to_string(path).ok().map(|t| Self::parse(&t))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 英語の単語から日本語の表記を引く(よく使われる順)。例: word → 単語。
    pub fn to_japanese(&self, word: &str) -> Vec<String> {
        self.rev.get(&norm_en(word)).cloned().unwrap_or_default()
    }

    /// 表記(と読み)から英語の訳語を引く。読みが一致するものを優先し、無ければ最もよく使われる語の訳語。
    pub fn lookup(&self, surface: &str, reading: &str) -> Vec<String> {
        let Some(v) = self.map.get(surface) else { return vec![] };
        v.iter().find(|(r, _)| r == reading).or_else(|| v.first()).map(|(_, g)| g.clone()).unwrap_or_default()
    }
}

// ---------------------------------------------------------------- 自動取得

pub fn glossary_path(root: &Path) -> PathBuf {
    root.join("glossary.tsv")
}

const LICENSE_TEXT: &str = "JMdict / EDICT 辞書データについて (About the JMdict data)\n\n\
この IME の英語訳(変換候補の英語の列)は、JMdict を元にしています。\n\
This IME's English glosses are derived from the JMdict dictionary.\n\n\
 - JMdict / EDICT: (c) Electronic Dictionary Research and Development Group (EDRDG), https://www.edrdg.org/\n\
 - JSON 版: jmdict-simplified (https://github.com/scriptin/jmdict-simplified)\n\
 - ライセンス / License: Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0)\n\
   https://creativecommons.org/licenses/by-sa/4.0/\n\
 - 変更 / Changes: 表記・読み・英語訳だけを抜き出して glossary.tsv に変換しました。\n\
   Converted into glossary.tsv (surface, reading, English glosses only).\n\n\
このデータファイル(glossary.tsv)は CC BY-SA 4.0 のまま再配布できます。IME 本体のライセンス(MIT/Apache-2.0)には影響しません。\n\
This data file (glossary.tsv) remains under CC BY-SA 4.0; it does not affect the IME's own license (MIT/Apache-2.0).\n";

#[derive(Debug, PartialEq)]
pub enum Ensured {
    AlreadyInstalled,
    Installed(usize),
    Unavailable(String),
}

/// 最新の JMdict(jmdict-simplified の英語版 JSON zip)を GitHub Releases から取得し、訳語辞書とライセンス表示を置く。
/// すでにあれば何もしない。取得できなくても IME は訳語なしで動き続ける。
pub fn ensure(root: &Path, allow_download: bool) -> Ensured {
    let p = glossary_path(root);
    if p.is_file() {
        return Ensured::AlreadyInstalled;
    }
    if !allow_download {
        return Ensured::Unavailable("英語訳の辞書(JMdict)は未インストールです".into());
    }
    let result = (|| -> Result<usize, String> {
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(600)).user_agent("aruaru-ime").build();
        let rel: serde_json::Value = agent
            .get(&format!("https://api.github.com/repos/{REPO}/releases/latest"))
            .call()
            .map_err(|e| format!("リリースを調べられません: {e}"))?
            .into_json()
            .map_err(|e| e.to_string())?;
        let url = rel["assets"]
            .as_array()
            .and_then(|a| a.iter().find(|x| x["name"].as_str().is_some_and(|n| n.starts_with("jmdict-eng-") && !n.contains("common") && n.ends_with(".json.zip"))))
            .and_then(|x| x["browser_download_url"].as_str())
            .ok_or("JMdict の配布ファイルが見つかりません")?
            .to_string();
        if !crate::update::is_trusted_url(&url) {
            return Err("取得先が GitHub ではないため中止しました".into());
        }
        let mut buf = Vec::new();
        agent.get(&url).call().map_err(|e| e.to_string())?.into_reader().take(200 * 1024 * 1024).read_to_end(&mut buf).map_err(|e| e.to_string())?;
        let tsv = build_from_zip(&buf)?;
        let n = tsv.lines().count();
        if n < 10_000 {
            return Err(format!("訳語が少なすぎます({n}件)。取得内容を確認してください"));
        }
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        std::fs::write(root.join("JMdict-LICENSE.txt"), LICENSE_TEXT).map_err(|e| e.to_string())?;
        std::fs::write(&p, tsv).map_err(|e| e.to_string())?;
        Ok(n)
    })();
    match result {
        Ok(n) => Ensured::Installed(n),
        Err(e) => Ensured::Unavailable(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"words":[
      {"id":1,"kanji":[{"common":true,"text":"単語","tags":[]}],"kana":[{"common":true,"text":"たんご","tags":[],"appliesToKanji":["*"]}],
       "sense":[{"partOfSpeech":["n"],"appliesToKanji":["*"],"appliesToKana":["*"],"gloss":[{"lang":"eng","text":"word"},{"lang":"eng","text":"vocabulary"}]},
                {"partOfSpeech":["n"],"appliesToKanji":["*"],"appliesToKana":["*"],"gloss":[{"lang":"eng","text":"term"}]}]},
      {"id":2,"kanji":[{"common":false,"text":"丹後","tags":[]}],"kana":[{"common":false,"text":"たんご","tags":[],"appliesToKanji":["*"]}],
       "sense":[{"partOfSpeech":["n"],"appliesToKanji":["*"],"appliesToKana":["*"],"gloss":[{"lang":"eng","text":"Tango (former province)"}]}]},
      {"id":3,"kanji":[],"kana":[{"common":true,"text":"タンゴ","tags":[],"appliesToKanji":[]}],
       "sense":[{"partOfSpeech":["n"],"appliesToKanji":[],"appliesToKana":["*"],"gloss":[{"lang":"eng","text":"tango (dance)"}]}]}
    ]}"#;

    #[test]
    fn builds_and_looks_up_glosses_by_surface_and_reading() {
        let tsv = build_tsv(SAMPLE.as_bytes()).unwrap();
        let g = Glossary::parse(&tsv);
        assert_eq!(g.lookup("単語", "たんご"), vec!["word", "vocabulary", "term"]);
        assert_eq!(g.lookup("丹後", "たんご"), vec!["Tango (former province)"]);
        assert_eq!(g.lookup("タンゴ", "たんご"), vec!["tango (dance)"], "カタカナ語もそのまま引ける");
        assert_eq!(g.lookup("たんご", "たんご"), vec!["word", "vocabulary", "term"], "よく使われる語のかな書きでも引ける");
        assert!(g.lookup("存在しない", "x").is_empty());
        // 英語 → 日本語
        assert_eq!(g.to_japanese("word").first().map(String::as_str), Some("単語"));
        assert!(g.to_japanese("Vocabulary").contains(&"単語".to_string()), "大文字小文字を区別しない");
        assert!(g.to_japanese("tango (dance)").contains(&"タンゴ".to_string()));
        assert!(g.to_japanese("zzzz").is_empty());
        // よく使われる語が先に並ぶ(同じ表記が複数あるときの既定)
        assert!(tsv.lines().next().unwrap().starts_with("単語"));
    }

    #[test]
    fn zip_roundtrip() {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            z.start_file("jmdict-eng-3.6.2.json", zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(SAMPLE.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        assert!(build_from_zip(&buf).unwrap().contains("単語\tたんご\tword|vocabulary|term"));
    }

    #[test]
    fn real_jmdict_if_downloaded() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/jmdict-src/jmdict-eng.json.zip");
        let Ok(bytes) = std::fs::read(p) else { return };
        let tsv = build_from_zip(&bytes).unwrap();
        let g = Glossary::parse(&tsv);
        assert!(g.len() > 100_000, "{}", g.len());
        assert!(g.lookup("単語", "たんご").iter().any(|x| x.contains("word")));
        assert!(g.lookup("漢字", "かんじ").iter().any(|x| x.to_lowercase().contains("kanji") || x.contains("character")));
        assert!(g.lookup("変換", "へんかん").iter().any(|x| x.contains("convert") || x.contains("transform") || x.contains("conversion")));
    }

    #[test]
    fn ensure_without_permission_is_unavailable_and_existing_is_kept() {
        let d = std::env::temp_dir().join(format!("aruaru-gl-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        let _ = std::fs::remove_file(glossary_path(&d));
        assert!(matches!(ensure(&d, false), Ensured::Unavailable(_)));
        std::fs::write(glossary_path(&d), "a\tb\tc").unwrap();
        assert_eq!(ensure(&d, true), Ensured::AlreadyInstalled);
        let _ = std::fs::remove_dir_all(&d);
    }
}
