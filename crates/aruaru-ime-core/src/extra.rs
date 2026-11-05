//! 語彙の拡張(案A): JMdict(名詞・サ変名詞)と JMnedict(人名・地名・組織名・駅名・作品名など)から、
//! Mozc 辞書に無い「読み+表記」の組を Mozc 形式の `extra00.txt` として作る。`kanji::build` は同じ場所の
//! `extra*.txt` を自動で読み込む。元データは jmdict-simplified(© EDRDG、CC BY-SA 4.0。表示は `installer/LICENSES.txt`)。

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Root {
    words: Vec<Word>,
}
#[derive(Deserialize)]
struct Word {
    #[serde(default)]
    kanji: Vec<Form>,
    #[serde(default)]
    kana: Vec<Kana>,
    #[serde(default)]
    sense: Vec<Tags>,
    #[serde(default)]
    translation: Vec<Tags>,
}
#[derive(Deserialize)]
struct Form {
    text: String,
    #[serde(default)]
    common: bool,
}
#[derive(Deserialize)]
struct Kana {
    text: String,
    #[serde(default, rename = "appliesToKanji")]
    applies: Vec<String>,
}
/// JMdict の sense(品詞)または JMnedict の translation(種別)。
#[derive(Deserialize, Default)]
struct Tags {
    #[serde(default, rename = "partOfSpeech")]
    pos: Vec<String>,
    #[serde(default, rename = "type")]
    kind: Vec<String>,
}

// Mozc の id.def の品詞 ID。
const ID_NOUN: u16 = 1851;
const ID_SAHEN: u16 = 1841;
const ID_PERSON: u16 = 1921;
const ID_GIVEN: u16 = 1922;
const ID_SURNAME: u16 = 1923;
const ID_PLACE: u16 = 1924;
const ID_ORG: u16 = 1929;
const ID_PROPER: u16 = 1920;

fn to_hira(s: &str) -> Option<String> {
    let mut o = String::new();
    for c in s.chars() {
        match c {
            'ァ'..='ヶ' => o.push(char::from_u32(c as u32 - 0x60)?),
            'ぁ'..='ゖ' | 'ー' => o.push(c),
            _ => return None,
        }
    }
    (!o.is_empty()).then_some(o)
}

/// かな・長音だけでできていない(漢字・英数字などを含む)表記か。
fn is_not_pure_kana(s: &str) -> bool {
    s.chars().any(|c| !matches!(c, 'ぁ'..='ゖ' | 'ァ'..='ヶ' | 'ー'))
}

fn existing(src_dir: &Path) -> std::io::Result<HashSet<(String, String)>> {
    let mut set = HashSet::new();
    for i in 0..10 {
        for line in fs::read_to_string(src_dir.join(format!("dictionary{i:02}.txt")))?.lines() {
            let mut it = line.split('\t');
            if let (Some(r), _, _, _, Some(s)) = (it.next(), it.next(), it.next(), it.next(), it.next()) {
                set.insert((r.to_string(), s.to_string()));
            }
        }
    }
    Ok(set)
}

fn classify_ned(tags: &[Tags]) -> (u16, u32) {
    let kinds: Vec<&str> = tags.iter().flat_map(|t| t.kind.iter().map(String::as_str)).collect();
    let any = |ks: &[&str]| kinds.iter().any(|k| ks.contains(k));
    if any(&["surname"]) {
        (ID_SURNAME, 8600)
    } else if any(&["given", "fem", "masc"]) {
        (ID_GIVEN, 8600)
    } else if any(&["person"]) {
        (ID_PERSON, 8800)
    } else if any(&["place", "station"]) {
        (ID_PLACE, 8400)
    } else if any(&["company", "organization"]) {
        (ID_ORG, 8600)
    } else {
        (ID_PROPER, 9000)
    }
}

fn collect(path: &Path, ned: bool, have: &HashSet<(String, String)>, seen: &mut HashSet<(String, String)>, out: &mut String) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let root: Root = sonic_rs::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    for w in &root.words {
        let (id, base) = if ned {
            classify_ned(&w.translation)
        } else {
            let pos: Vec<&str> = w.sense.iter().flat_map(|t| t.pos.iter().map(String::as_str)).collect();
            if pos.contains(&"n") && pos.contains(&"vs") {
                (ID_SAHEN, 9000)
            } else if pos.contains(&"n") {
                (ID_NOUN, 9000)
            } else {
                continue;
            }
        };
        for k in &w.kanji {
            if !is_not_pure_kana(&k.text) || k.text.len() > 120 {
                continue;
            }
            let cost = base + if k.common { 0 } else { 500 };
            for ka in &w.kana {
                if !ka.applies.is_empty() && !ka.applies.iter().any(|a| a == "*" || a == &k.text) {
                    continue;
                }
                let Some(r) = to_hira(&ka.text) else { continue };
                let key = (r, k.text.clone());
                if have.contains(&key) || !seen.insert(key.clone()) {
                    continue;
                }
                out.push_str(&format!("{}\t{id}\t{id}\t{cost}\t{}\n", key.0, key.1));
            }
        }
    }
    Ok(())
}

/// 追加した行数を返す。`jmdict`・`jmnedict` は jmdict-simplified の JSON ファイル。
pub fn build_extra(src_dir: &Path, jmdict: &Path, jmnedict: &Path) -> Result<usize, String> {
    let have = existing(src_dir).map_err(|e| e.to_string())?;
    let mut seen = HashSet::new();
    let mut out = String::new();
    collect(jmnedict, true, &have, &mut seen, &mut out)?;
    collect(jmdict, false, &have, &mut seen, &mut out)?;
    let n = out.lines().count();
    fs::write(src_dir.join("extra00.txt"), out).map_err(|e| e.to_string())?;
    Ok(n)
}
