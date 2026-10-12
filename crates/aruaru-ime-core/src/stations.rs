//! 全国の駅名の取り込み。OpenStreetMap の公開データ(Overpass API、ODbL)から、日本の `railway=station|halt` の名前・読み・英語名を取得し、
//! 利用者辞書に出所つき(`用語:駅名:全国`)で入れる。読みが無い駅は、英語名(ヘボン式ローマ字)から読みを作る(長音の表記ゆれで誤ることがある)。
//! データ © OpenStreetMap contributors (ODbL 1.0)。利用者が選んだときだけ取得する。

use crate::romaji;
use crate::userdict::UserDict;
use std::collections::BTreeSet;

pub const SOURCE: &str = "用語:駅名:全国";
/// 混んでいて失敗することがあるので、順に試す。
const ENDPOINTS: &[&str] = &["https://overpass-api.de/api/interpreter", "https://overpass.kumi.systems/api/interpreter", "https://overpass-api.de/api/interpreter"];
const QUERY: &str = "[out:csv(\"name\",\"name:ja-Hira\",\"name:ja_kana\",\"name:en\";false;\"\\t\")][timeout:180];\narea[\"ISO3166-1\"=\"JP\"][admin_level=2]->.jp;\n(node[\"railway\"=\"station\"](area.jp);node[\"railway\"=\"halt\"](area.jp););\nout;";

/// ヘボン式ローマ字(OSM の name:en)から、ひらがなの読みを作る。作れなければ None。
pub fn reading_from_romaji(en: &str) -> Option<String> {
    let mut s = String::new();
    for c in en.chars() {
        match c {
            'ā' | 'Ā' => s.push_str("aa"),
            'ī' | 'Ī' => s.push_str("ii"),
            'ū' | 'Ū' => s.push_str("uu"),
            'ē' | 'Ē' => s.push_str("ei"),
            'ō' | 'Ō' => s.push_str("ou"),
            ' ' | '-' | '.' | '・' => {}
            c if c.is_ascii_alphabetic() || c == '\'' => s.push(c.to_ascii_lowercase()),
            _ => return None,
        }
    }
    let s = s.trim_end_matches("station").to_string();
    if s.is_empty() || !romaji::is_romaji_parsable(&s) {
        return None;
    }
    let h = romaji::to_hiragana(&s);
    (!h.is_empty() && h.chars().all(|c| matches!(c, 'ぁ'..='ゖ' | 'ー'))).then_some(h)
}

/// 取得した CSV(`name \t name:ja-Hira \t name:ja_kana \t name:en`)から、(読み, 駅名)の一覧を作る。
pub fn parse(csv: &str) -> Vec<(String, String)> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for l in csv.lines() {
        let f: Vec<&str> = l.split('\t').collect();
        let name = f.first().map(|s| s.trim()).unwrap_or("");
        if name.is_empty() || name.chars().count() > 20 || name.chars().any(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        let name = name.trim_end_matches('駅').to_string();
        if name.chars().count() < 1 {
            continue;
        }
        let hira = f.get(1).map(|s| s.trim()).unwrap_or("");
        let kana = f.get(2).map(|s| s.trim()).unwrap_or("");
        let en = f.get(3).map(|s| s.trim()).unwrap_or("");
        let reading = if !hira.is_empty() && hira.chars().all(|c| matches!(c, 'ぁ'..='ゖ' | 'ー')) {
            Some(hira.trim_end_matches("えき").to_string())
        } else if !kana.is_empty() {
            let h = romaji::kata_to_hira(kana);
            h.chars().all(|c| matches!(c, 'ぁ'..='ゖ' | 'ー')).then(|| h.trim_end_matches("えき").to_string())
        } else {
            reading_from_romaji(en)
        };
        if let Some(r) = reading.filter(|r| !r.is_empty()) {
            seen.insert((r, name));
        }
    }
    seen.into_iter().collect()
}

/// 取得して、利用者辞書に入れる(調べ直しは入れ替え)。入れた駅数を返す。
pub fn install(user: &mut UserDict) -> Result<usize, String> {
    use std::io::Read;
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(240)).user_agent("aruaru-ime/0.2 (https://github.com/aon-co-jp/aruaru-IME)").build();
    let mut body = String::new();
    let mut last = String::new();
    for (i, url) in ENDPOINTS.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        match agent.post(url).set("Accept", "*/*").send_form(&[("data", QUERY)]) {
            Ok(r) => {
                body.clear();
                if r.into_reader().take(20 * 1024 * 1024).read_to_string(&mut body).is_ok() && body.lines().count() > 1000 {
                    break;
                }
                last = "応答が不完全でした".into();
            }
            Err(e) => last = format!("{url}: {e}"),
        }
    }
    if body.lines().count() <= 1000 {
        return Err(format!("取得できません: {last}"));
    }
    let pairs = parse(&body);
    if pairs.len() < 1000 {
        return Err(format!("駅名が少なすぎます({}件)", pairs.len()));
    }
    user.remove_source(SOURCE).map_err(|e| e.to_string())?;
    user.add_many(&pairs, SOURCE).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readings_from_hiragana_katakana_and_romaji() {
        let csv = "新子安\tしんこやす\t\tShin-Koyasu\n西平内\t\t\tNishi-Hiranai\n福島駅\tふくしまえき\t\tFukushima\n大崎\t\tオオサキ\tOsaki\nStation\t\t\tX\n";
        let p = parse(csv);
        assert!(p.contains(&("しんこやす".to_string(), "新子安".to_string())));
        assert!(p.contains(&("にしひらない".to_string(), "西平内".to_string())), "{p:?}");
        assert!(p.contains(&("ふくしま".to_string(), "福島".to_string())), "「駅」は落とす");
        assert!(p.contains(&("おおさき".to_string(), "大崎".to_string())), "カタカナの読みはひらがなにする");
        assert_eq!(reading_from_romaji("Tōkyō").as_deref(), Some("とうきょう"));
        assert_eq!(reading_from_romaji("Shin'ya").as_deref(), Some("しんや"));
        assert_eq!(reading_from_romaji("Koyasu Station").as_deref(), Some("こやす"));
    }
}
