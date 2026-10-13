//! 毎朝の自動更新: 取得済みの郵便番号データ(日本郵便・GeoNames)を新しく保ち、国名・主要都市名(Wikidata, CC0)を利用者辞書へ入れ直す。
//!
//! - 時刻は、端末のローカル時刻の朝6時台〜8時台のうちの、端末ごとに決まる時刻(世界中の端末が同じ瞬間にサーバーへ集中しないように、
//!   データの置き場所から計算した 0〜120 分のずれを付ける)。その日にまだ実行していなければ、その時刻以降のいつでも実行する(起動が遅れても取りこぼさない)。
//! - 郵便番号は、**すでに取得してあるものだけ**を更新する(選んでいないデータを自動では取りに行かない)。変わっていなければ、条件付きの取得
//!   (ETag / Last-Modified)でダウンロードを省く。
//! - 国名・都市名は、Wikidata の公開クエリで取り、利用者辞書へ出所つき(`自動更新:国名` `自動更新:都市名`)で入れる。出所ごとに削除できる。
//! - 設定 `data_update` で止められる。

use crate::userdict::UserDict;
use crate::{geo, postal};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

const UA: &str = "aruaru-IME-data-updater/0.1 (https://github.com/aon-co-jp/aruaru-IME)";
pub const SRC_COUNTRIES: &str = "自動更新:国名";
pub const SRC_CITIES: &str = "自動更新:都市名";

#[derive(Clone, Default, Serialize, Deserialize)]
struct Validator {
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
struct Meta {
    /// 最後に更新を完了した日(端末のローカル日付 YYYY-MM-DD)。
    last_run: String,
    files: BTreeMap<String, Validator>,
}

fn meta_path(root: &Path) -> PathBuf {
    root.join("data-update.json")
}

fn load_meta(root: &Path) -> Meta {
    std::fs::read_to_string(meta_path(root)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_meta(root: &Path, m: &Meta) {
    let _ = std::fs::create_dir_all(root);
    let _ = std::fs::write(meta_path(root), serde_json::to_string_pretty(m).unwrap_or_default());
}

/// 端末ごとに決まる、朝6時からのずれ(分、0〜119)。
pub fn jitter_minutes(root: &Path) -> u32 {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(root.to_string_lossy().as_bytes());
    h.update(std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_default().as_bytes());
    let d = h.finalize();
    (u32::from_be_bytes([d[0], d[1], d[2], d[3]])) % 120
}

/// いまが、今日の更新時刻を過ぎていて、今日はまだ実行していないか。`now` は端末のローカル時刻。
pub fn is_due(root: &Path, now: chrono::NaiveDateTime) -> bool {
    use chrono::Timelike;
    let start = 6 * 60 + jitter_minutes(root);
    let minutes = now.hour() * 60 + now.minute();
    minutes >= start && load_meta(root).last_run != now.format("%Y-%m-%d").to_string()
}

pub fn due_now(root: &Path) -> bool {
    is_due(root, chrono::Local::now().naive_local())
}

#[derive(Debug, Default)]
pub struct Report {
    pub world_updated: usize,
    pub world_unchanged: usize,
    pub japan: Option<bool>,
    pub countries: usize,
    pub cities: usize,
    pub current_country: Option<String>,
    pub errors: Vec<String>,
}

/// 条件付きで取得する。`Ok(None)` は「前回から変わっていない」(304)。
fn conditional_get(url: &str, prev: Option<&Validator>, limit: u64) -> Result<Option<(Vec<u8>, Validator)>, String> {
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(180)).user_agent(UA).build();
    let mut req = agent.get(url);
    if let Some(p) = prev {
        if let Some(e) = &p.etag {
            req = req.set("If-None-Match", e);
        }
        if let Some(m) = &p.last_modified {
            req = req.set("If-Modified-Since", m);
        }
    }
    match req.call() {
        // ureq は 304(変更なし)をエラーにせず、本文の無い応答として返す
        Ok(r) if r.status() == 304 => Ok(None),
        Ok(r) => {
            let v = Validator { etag: r.header("ETag").map(String::from), last_modified: r.header("Last-Modified").map(String::from) };
            let mut b = Vec::new();
            r.into_reader().take(limit).read_to_end(&mut b).map_err(|e| e.to_string())?;
            Ok(Some((b, v)))
        }
        Err(ureq::Error::Status(304, _)) => Ok(None),
        Err(e) => Err(format!("{url}: {e}")),
    }
}

/// SPARQL の結果(TSV)から、(日本語表記, 英語表記) の組を取り出す。
fn parse_sparql_tsv(tsv: &str) -> Vec<(String, String)> {
    fn lit(v: &str) -> String {
        let v = v.trim();
        if let Some(rest) = v.strip_prefix('"') {
            if let Some(end) = rest.rfind('"') {
                return rest[..end].to_string();
            }
        }
        v.to_string()
    }
    tsv.lines().skip(1).filter_map(|l| {
        let mut c = l.split('\t');
        let ja = lit(c.next()?);
        let en = lit(c.next().unwrap_or(""));
        (!ja.is_empty()).then_some((ja, en))
    }).collect()
}

fn kana_only(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| matches!(c, '\u{3041}'..='\u{3096}' | '\u{30A1}'..='\u{30FA}' | 'ー' | '・' | '･' | '·' | ' ' | '=' | '＝'))
}

/// カタカナ・ひらがなの表記から、読み(ひらがな、中黒・空白なし)を作る。漢字などを含む表記は None。
pub fn reading_of(label: &str) -> Option<String> {
    if !kana_only(label) {
        return None;
    }
    let r: String = label
        .chars()
        .filter(|c| !matches!(c, '・' | '･' | '·' | ' ' | '=' | '＝'))
        .map(|c| if ('\u{30A1}'..='\u{30F6}').contains(&c) { char::from_u32(c as u32 - 0x60).unwrap_or(c) } else { c })
        .collect();
    (!r.is_empty()).then_some(r)
}

fn sparql(query: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(120)).user_agent(UA).build();
    agent
        .get("https://query.wikidata.org/sparql")
        .query("query", query)
        .set("Accept", "text/tab-separated-values")
        .call()
        .map_err(|e| format!("Wikidata: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())
}

const Q_COUNTRIES: &str = r#"SELECT ?ja ?en WHERE { ?c wdt:P31 wd:Q3624078. ?c rdfs:label ?ja. FILTER(LANG(?ja)="ja")
 OPTIONAL { ?c rdfs:label ?e1 FILTER(LANG(?e1)="en") } OPTIONAL { ?c rdfs:label ?e2 FILTER(LANG(?e2)="mul") } BIND(COALESCE(?e1,?e2) AS ?en)
 FILTER NOT EXISTS { ?c wdt:P576 ?end } } LIMIT 400"#;

const Q_CITIES: &str = r#"SELECT ?ja ?en WHERE { ?c wdt:P31 wd:Q1549591; wdt:P1082 ?pop. FILTER(?pop >= 1000000)
 ?c rdfs:label ?ja. FILTER(LANG(?ja)="ja")
 OPTIONAL { ?c rdfs:label ?e1 FILTER(LANG(?e1)="en") } OPTIONAL { ?c rdfs:label ?e2 FILTER(LANG(?e2)="mul") } BIND(COALESCE(?e1,?e2) AS ?en)
 } ORDER BY DESC(?pop) LIMIT 1500"#;

/// 国名・主要都市名(人口100万以上)を取り、利用者辞書の該当する出所を入れ替える。読みが分かるもの(カタカナ・ひらがな表記)だけ。
fn update_names(user: &mut UserDict, rep: &mut Report) {
    for (src, q, is_country) in [(SRC_COUNTRIES, Q_COUNTRIES, true), (SRC_CITIES, Q_CITIES, false)] {
        match sparql(q) {
            Ok(tsv) => {
                let pairs: Vec<(String, String)> = parse_sparql_tsv(&tsv).into_iter().filter_map(|(ja, _)| reading_of(&ja).map(|r| (r, ja))).collect();
                // 取得に失敗した・空の結果で、いまの辞書を消さない
                if pairs.len() < if is_country { 50 } else { 30 } {
                    rep.errors.push(format!("{src}: 結果が少なすぎます({}件)ので更新しません", pairs.len()));
                    continue;
                }
                let _ = user.remove_source(src);
                match user.add_many(&pairs, src) {
                    Ok(n) => {
                        if is_country {
                            rep.countries = n
                        } else {
                            rep.cities = n
                        }
                    }
                    Err(e) => rep.errors.push(format!("{src}: {e}")),
                }
            }
            Err(e) => rep.errors.push(e),
        }
    }
}

/// 更新を1回行う。エラーが無ければ、今日の分を完了として記録する(エラーがあれば次の機会に再試行)。
pub fn run(root: &Path, user: &mut UserDict) -> Report {
    let mut rep = Report::default();
    let mut meta = load_meta(root);

    // 郵便番号(世界): 取得済みの国だけ
    for cc in geo::installed_countries(root) {
        let key = format!("geonames:{cc}");
        match conditional_get(&geo::zip_url(&cc), meta.files.get(&key), 60 * 1024 * 1024) {
            Ok(None) => rep.world_unchanged += 1,
            Ok(Some((bytes, v))) => match geo::store_country_zip(root, &cc, bytes) {
                Ok(_) => {
                    rep.world_updated += 1;
                    meta.files.insert(key, v);
                }
                Err(e) => rep.errors.push(e),
            },
            Err(e) => rep.errors.push(e),
        }
        std::thread::sleep(std::time::Duration::from_millis(800)); // サーバーへの配慮
    }
    // 郵便番号(日本): 取得済みなら
    if postal::is_installed(root) {
        match conditional_get(postal::SOURCE_URL, meta.files.get("japanpost"), 30 * 1024 * 1024) {
            Ok(None) => rep.japan = Some(false),
            Ok(Some((bytes, v))) => match postal::store_zip(root, bytes) {
                Ok(_) => {
                    rep.japan = Some(true);
                    meta.files.insert("japanpost".into(), v);
                }
                Err(e) => rep.errors.push(e),
            },
            Err(e) => rep.errors.push(e),
        }
    }
    // 現在地の国(移動した場合に追従する)。世界の郵便番号を取得してある人だけ
    if geo::is_installed(root) {
        match geo::detect_country() {
            Ok(c) => {
                if geo::ip_country(root).as_deref() != Some(c.as_str()) {
                    let _ = std::fs::write(geo::country_file(root), &c);
                }
                rep.current_country = Some(c);
            }
            Err(e) => rep.errors.push(e),
        }
    }
    update_names(user, &mut rep);

    if rep.errors.is_empty() {
        meta.last_run = chrono::Local::now().format("%Y-%m-%d").to_string();
    }
    save_meta(root, &meta);
    rep
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-du-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn due_only_after_the_morning_start_and_once_a_day() {
        let root = tmp("due");
        let j = jitter_minutes(&root);
        assert!(j < 120);
        let day = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();
        let start = 6 * 60 + j;
        let at = |m: u32| day.and_hms_opt(m / 60, m % 60, 0).unwrap();
        assert!(!is_due(&root, at(start - 1)), "開始時刻の前は実行しない");
        assert!(is_due(&root, at(start)), "開始時刻になったら実行する");
        assert!(is_due(&root, at(23 * 60)), "起動が遅れても、その日のうちは取りこぼさない");
        let mut m = Meta::default();
        m.last_run = "2026-10-14".into();
        save_meta(&root, &m);
        assert!(!is_due(&root, at(23 * 60)), "今日の分を完了していれば実行しない");
        assert!(is_due(&root, day.succ_opt().unwrap().and_hms_opt(12, 0, 0).unwrap()), "翌日は実行する");
        // 端末(データの置き場所)が違えば、ずれも違う(同じ瞬間に集中しない)
        let other: Vec<u32> = (0..20).map(|i| jitter_minutes(&root.join(format!("x{i}")))).collect();
        assert!(other.iter().any(|x| *x != other[0]), "{other:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn readings_only_for_kana_labels() {
        assert_eq!(reading_of("ニューヨーク").as_deref(), Some("にゅーよーく"));
        assert_eq!(reading_of("セントクリストファー・ネイビス").as_deref(), Some("せんとくりすとふぁーねいびす"));
        assert_eq!(reading_of("上海市"), None);
        assert_eq!(reading_of("São Paulo"), None);
        assert_eq!(reading_of(""), None);
    }

    #[test]
    fn parses_sparql_tsv_literals() {
        let t = "?ja\t?en\n\"ジャマイカ\"@ja\t\"Jamaica\"@en\n\"上海市\"@ja\t\"Shanghai\"@en\t24870895.0\n\"X\"@ja\t\n";
        let v = parse_sparql_tsv(t);
        assert_eq!(v[0], ("ジャマイカ".to_string(), "Jamaica".to_string()));
        assert_eq!(v[1].0, "上海市");
        assert_eq!(v.len(), 3);
    }
}
