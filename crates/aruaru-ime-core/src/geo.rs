//! 世界の郵便番号 → 国名つきの住所。
//! 郵便番号(数字で始まるもの。`-` や空白は有り・無しの両方に対応)を打つと、国名と住所の候補が出る。
//! 候補の並びは、(1) IPアドレスから推定した国 → (2) 選択中の言語の国 → (3) その他、の順。
//! データは、利用者が選んだときだけ GeoNames の公開データ(CC BY 4.0, https://www.geonames.org/)から取得する(自動では取得しない)。
//! 国の推定は、取得時に1回だけ、IP から国を返す公開の窓口(Cloudflare の /cdn-cgi/trace)へ問い合わせた結果を端末に保存して使う。入力中の通信は無い。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// (国コード, 英語名, 日本語名, 中国語名, 韓国語名)
pub const COUNTRIES: &[(&str, &str, &str, &str, &str)] = &[
    ("JP", "Japan", "日本", "日本", "일본"),
    ("US", "United States", "アメリカ合衆国", "美国", "미국"),
    ("GB", "United Kingdom", "イギリス", "英国", "영국"),
    ("CA", "Canada", "カナダ", "加拿大", "캐나다"),
    ("AU", "Australia", "オーストラリア", "澳大利亚", "호주"),
    ("NZ", "New Zealand", "ニュージーランド", "新西兰", "뉴질랜드"),
    ("DE", "Germany", "ドイツ", "德国", "독일"),
    ("FR", "France", "フランス", "法国", "프랑스"),
    ("IT", "Italy", "イタリア", "意大利", "이탈리아"),
    ("ES", "Spain", "スペイン", "西班牙", "스페인"),
    ("PT", "Portugal", "ポルトガル", "葡萄牙", "포르투갈"),
    ("NL", "Netherlands", "オランダ", "荷兰", "네덜란드"),
    ("BE", "Belgium", "ベルギー", "比利时", "벨기에"),
    ("CH", "Switzerland", "スイス", "瑞士", "스위스"),
    ("AT", "Austria", "オーストリア", "奥地利", "오스트리아"),
    ("SE", "Sweden", "スウェーデン", "瑞典", "스웨덴"),
    ("NO", "Norway", "ノルウェー", "挪威", "노르웨이"),
    ("DK", "Denmark", "デンマーク", "丹麦", "덴마크"),
    ("FI", "Finland", "フィンランド", "芬兰", "핀란드"),
    ("IE", "Ireland", "アイルランド", "爱尔兰", "아일랜드"),
    ("PL", "Poland", "ポーランド", "波兰", "폴란드"),
    ("CZ", "Czechia", "チェコ", "捷克", "체코"),
    ("HU", "Hungary", "ハンガリー", "匈牙利", "헝가리"),
    ("RO", "Romania", "ルーマニア", "罗马尼亚", "루마니아"),
    ("BG", "Bulgaria", "ブルガリア", "保加利亚", "불가리아"),
    ("GR", "Greece", "ギリシャ", "希腊", "그리스"),
    ("RU", "Russia", "ロシア", "俄罗斯", "러시아"),
    ("UA", "Ukraine", "ウクライナ", "乌克兰", "우크라이나"),
    ("TR", "Türkiye", "トルコ", "土耳其", "튀르키예"),
    ("IN", "India", "インド", "印度", "인도"),
    ("PK", "Pakistan", "パキスタン", "巴基斯坦", "파키스탄"),
    ("BD", "Bangladesh", "バングラデシュ", "孟加拉国", "방글라데시"),
    ("LK", "Sri Lanka", "スリランカ", "斯里兰卡", "스리랑카"),
    ("TH", "Thailand", "タイ", "泰国", "태국"),
    ("MY", "Malaysia", "マレーシア", "马来西亚", "말레이시아"),
    ("SG", "Singapore", "シンガポール", "新加坡", "싱가포르"),
    ("PH", "Philippines", "フィリピン", "菲律宾", "필리핀"),
    ("ID", "Indonesia", "インドネシア", "印度尼西亚", "인도네시아"),
    ("KR", "South Korea", "韓国", "韩国", "대한민국"),
    ("CN", "China", "中国", "中国", "중국"),
    ("TW", "Taiwan", "台湾", "台湾", "대만"),
    ("BR", "Brazil", "ブラジル", "巴西", "브라질"),
    ("AR", "Argentina", "アルゼンチン", "阿根廷", "아르헨티나"),
    ("MX", "Mexico", "メキシコ", "墨西哥", "멕시코"),
    ("CO", "Colombia", "コロンビア", "哥伦比亚", "콜롬비아"),
    ("ZA", "South Africa", "南アフリカ", "南非", "남아프리카"),
];

/// 選択中の言語に対応する国(優先して候補の上に出す)。
pub fn lang_countries(lang: &str) -> &'static [&'static str] {
    match lang {
        "ja" => &["JP"],
        "en" => &["US", "GB", "CA", "AU", "NZ", "IE"],
        "zh-Hans" => &["CN", "SG"],
        "zh-Hant" => &["TW"],
        "ko" => &["KR"],
        "de" => &["DE", "AT", "CH"],
        "fr" => &["FR", "BE", "CH", "CA"],
        "it" => &["IT", "CH"],
        "es" => &["ES", "MX", "AR", "CO"],
        "pt" => &["BR", "PT"],
        "ru" => &["RU"],
        "uk" => &["UA"],
        "hi" | "mr" => &["IN"],
        _ => &[],
    }
}

/// 国名(選択中の言語で。無ければ英語名、それも無ければ国コード)。
pub fn country_name(cc: &str, lang: &str) -> String {
    for (c, en, ja, zh, ko) in COUNTRIES {
        if *c == cc {
            return match lang {
                "ja" => ja,
                "zh-Hans" | "zh-Hant" => zh,
                "ko" => ko,
                _ => en,
            }
            .to_string();
        }
    }
    cc.to_string()
}

fn dir(root: &Path) -> PathBuf {
    root.join("postal-world")
}
fn tsv_path(root: &Path, cc: &str) -> PathBuf {
    dir(root).join(format!("{cc}.tsv"))
}
pub fn country_file(root: &Path) -> PathBuf {
    root.join("geo-country.txt")
}

/// 郵便番号の入力(`-`・空白・全角の `－` `ー` は有っても無くても同じ)を、照合用の英数字(大文字)にする。
pub fn normalize(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect()
}

/// IPアドレスから推定して保存した国コード(未取得なら None)。
pub fn ip_country(root: &Path) -> Option<String> {
    let s = std::fs::read_to_string(country_file(root)).ok()?;
    let c = s.trim().to_ascii_uppercase();
    (c.len() == 2).then_some(c)
}

pub fn installed_countries(root: &Path) -> Vec<String> {
    COUNTRIES.iter().map(|c| c.0.to_string()).filter(|c| tsv_path(root, c).is_file()).collect()
}

pub fn is_installed(root: &Path) -> bool {
    !installed_countries(root).is_empty()
}

type Table = HashMap<String, Vec<String>>;

fn table(root: &Path, cc: &str) -> Option<std::sync::Arc<Table>> {
    static CACHE: Mutex<Option<HashMap<String, (SystemTime, std::sync::Arc<Table>)>>> = Mutex::new(None);
    let p = tsv_path(root, cc);
    let mt = std::fs::metadata(&p).and_then(|m| m.modified()).ok()?;
    let mut g = CACHE.lock().ok()?;
    let map = g.get_or_insert_with(HashMap::new);
    let key = format!("{}|{cc}", root.display());
    if let Some((t, v)) = map.get(&key) {
        if *t == mt {
            return Some(v.clone());
        }
    }
    let mut h: Table = HashMap::new();
    for l in std::fs::read_to_string(&p).ok()?.lines() {
        if let Some((z, a)) = l.split_once('\t') {
            h.insert(z.to_string(), a.split('|').map(String::from).collect());
        }
    }
    let a = std::sync::Arc::new(h);
    map.insert(key, (mt, a.clone()));
    Some(a)
}

/// 国の並び順: IPから推定した国 → 選択中の言語の国 → その他(表の順)。
pub fn order(root: &Path, lang: &str) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    if let Some(c) = ip_country(root) {
        v.push(c);
    }
    for c in lang_countries(lang) {
        if !v.iter().any(|x| x == c) {
            v.push(c.to_string());
        }
    }
    for c in COUNTRIES {
        if !v.iter().any(|x| x == c.0) {
            v.push(c.0.to_string());
        }
    }
    v
}

/// 英字で始まる郵便番号の形か(英国 SW1A 1AA、カナダ K1A 0B1、アイルランド D02 AF30 など)。`-`・空白は有っても無くても同じ。
pub fn looks_like_alnum_postal(input: &str) -> bool {
    if !input.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-') {
        return false;
    }
    let n = normalize(input);
    (5..=8).contains(&n.len()) && n.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) && n.chars().any(|c| c.is_ascii_digit())
}

/// GeoNames が「地区」までしか持たない国(英国=外側コード、カナダ・アイルランド=先頭3文字)の、完全な郵便番号からの絞り込み。
fn area_key(cc: &str, n: &str) -> Option<String> {
    match cc {
        "GB" if n.len() >= 5 => Some(n[..n.len() - 3].to_string()),
        "CA" | "IE" if n.len() >= 3 => Some(n[..3].to_string()),
        _ => None,
    }
}

/// 候補の総数の上限(国名つきの住所)。同じ番号が多くの国にあっても、候補窓があふれないようにする。
const MAX_CANDIDATES: usize = 8;

/// 郵便番号に合う「国名 住所」の候補(国の優先順)。数字で始まる入力、または英字で始まる郵便番号の形の入力が対象。
/// IPの国・選択中の言語の国は最大3件ずつ、それ以外の国は1件ずつ(全体で8件まで)。
pub fn candidates(root: &Path, input: &str, lang: &str) -> Vec<String> {
    let n = normalize(input);
    let digit_start = input.chars().next().is_some_and(|c| c.is_ascii_digit());
    if n.len() < 3 || !(digit_start || looks_like_alnum_postal(input)) {
        return vec![];
    }
    let mut priority: Vec<String> = lang_countries(lang).iter().map(|c| c.to_string()).collect();
    if let Some(c) = ip_country(root) {
        priority.push(c);
    }
    let mut out: Vec<String> = vec![];
    for cc in order(root, lang) {
        if out.len() >= MAX_CANDIDATES {
            break;
        }
        let take = if priority.contains(&cc) { 3 } else { 1 };
        let name = country_name(&cc, lang);
        // 日本は、日本郵便の漢字の住所データ(右クリックで取得済みなら)を優先する。無ければ GeoNames(ローマ字)
        if cc == "JP" && n.len() == 7 {
            let jp = crate::postal::lookup(&n);
            if !jp.is_empty() {
                out.extend(jp.iter().take(take).map(|a| format!("{name} {a}")));
                continue;
            }
        }
        let Some(t) = table(root, &cc) else { continue };
        let hit = t.get(&n).or_else(|| area_key(&cc, &n).and_then(|k| t.get(&k)));
        if let Some(addrs) = hit {
            out.extend(addrs.iter().take(take).map(|a| format!("{name} {a}")));
        }
    }
    out.truncate(MAX_CANDIDATES);
    out
}

/// GeoNames の `<CC>.txt`(タブ区切り)を、`郵便番号<TAB>住所|住所…` にする。
pub fn parse_geonames(txt: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut m: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for l in txt.lines() {
        let f: Vec<&str> = l.split('\t').collect();
        if f.len() < 5 {
            continue;
        }
        let z = normalize(f[1]);
        if z.is_empty() {
            continue;
        }
        // 住所: 州/県 + 郡/市 + 地名(同じ語の連続は1つに)
        let mut parts: Vec<&str> = vec![];
        for p in [f[3], f.get(5).copied().unwrap_or(""), f[2]] {
            let p = p.trim();
            if !p.is_empty() && parts.last() != Some(&p) && !parts.contains(&p) {
                parts.push(p);
            }
        }
        let addr = parts.join(" ");
        let v = m.entry(z).or_default();
        if !addr.is_empty() && !v.contains(&addr) {
            v.push(addr);
        }
    }
    m
}

fn fetch_zip(cc: &str) -> Result<String, String> {
    use std::io::Read;
    let url = format!("https://download.geonames.org/export/zip/{cc}.zip");
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(120)).user_agent("aruaru-ime").build();
    let mut bytes = Vec::new();
    agent.get(&url).call().map_err(|e| format!("{cc}: {e}"))?.into_reader().take(60 * 1024 * 1024).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let mut ar = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut f = ar.by_name(&format!("{cc}.txt")).map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    f.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&raw).to_string())
}

/// 現在地の国を、IP から国を返す公開の窓口に1回だけ問い合わせて調べる(利用者が選んだときだけ呼ぶ)。
pub fn detect_country() -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(10)).user_agent("aruaru-ime").build();
    let body = agent.get("https://www.cloudflare.com/cdn-cgi/trace").call().map_err(|e| format!("国を調べられません: {e}"))?.into_string().map_err(|e| e.to_string())?;
    body.lines()
        .find_map(|l| l.strip_prefix("loc="))
        .map(|c| c.trim().to_ascii_uppercase())
        .filter(|c| c.len() == 2 && c != "XX")
        .ok_or_else(|| "国を判定できませんでした".to_string())
}

/// 世界の郵便番号データを取得して保存し、現在地の国も調べる(利用者が選んだときだけ呼ぶ)。(取り込んだ国の数, 現在地の国) を返す。
pub fn install(root: &Path) -> Result<(usize, Option<String>), String> {
    std::fs::create_dir_all(dir(root)).map_err(|e| e.to_string())?;
    let mut ok = 0;
    let mut last_err = String::new();
    for (cc, ..) in COUNTRIES {
        match fetch_zip(cc) {
            Ok(txt) => {
                let m = parse_geonames(&txt);
                if m.is_empty() {
                    continue;
                }
                let tsv: String = m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect();
                std::fs::write(tsv_path(root, cc), tsv).map_err(|e| e.to_string())?;
                ok += 1;
            }
            Err(e) => last_err = e,
        }
    }
    if ok == 0 {
        return Err(format!("取得できませんでした: {last_err}"));
    }
    let country = detect_country().ok();
    if let Some(c) = &country {
        let _ = std::fs::write(country_file(root), c);
    }
    Ok((ok, country))
}

/// 国ごとの郵便番号データの取得先(GeoNames)。
pub fn zip_url(cc: &str) -> String {
    format!("https://download.geonames.org/export/zip/{cc}.zip")
}

/// 取得した `<CC>.zip` から、その国の郵便番号の表を作って保存する。作った郵便番号の数を返す。
pub fn store_country_zip(root: &Path, cc: &str, zip_bytes: Vec<u8>) -> Result<usize, String> {
    use std::io::Read;
    let mut ar = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).map_err(|e| e.to_string())?;
    let mut f = ar.by_name(&format!("{cc}.txt")).map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    f.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    let m = parse_geonames(&String::from_utf8_lossy(&raw));
    if m.is_empty() {
        return Err(format!("{cc}: 空のデータ"));
    }
    std::fs::create_dir_all(dir(root)).map_err(|e| e.to_string())?;
    let tsv: String = m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect();
    std::fs::write(tsv_path(root, cc), tsv).map_err(|e| e.to_string())?;
    Ok(m.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-geo-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(dir(&d)).unwrap();
        d
    }

    #[test]
    fn normalizes_with_and_without_separators() {
        assert_eq!(normalize("100-0001"), "1000001");
        assert_eq!(normalize("1000001"), "1000001");
        assert_eq!(normalize("01310-100"), "01310100");
        assert_eq!(normalize("00 950"), "00950");
        assert_eq!(normalize("sw1a 1aa"), "SW1A1AA");
    }

    #[test]
    fn parses_geonames_and_orders_by_ip_then_language() {
        let root = tmp("order");
        let us = "US\t10001\tNew York\tNew York\tNY\tNew York\t061\t\t\t40.75\t-73.99\t4\n";
        let de = "DE\t10001\tBerlin\tBerlin\tBE\t\t\t\t\t52.5\t13.4\t4\n";
        for (cc, t) in [("US", us), ("DE", de)] {
            let m = parse_geonames(t);
            let s: String = m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect();
            std::fs::write(tsv_path(&root, cc), s).unwrap();
        }
        // 言語が日本語・IPの国が未取得: 表の順(US が DE より前)
        let c = candidates(&root, "10001", "ja");
        assert_eq!(c, vec!["アメリカ合衆国 New York".to_string(), "ドイツ Berlin".to_string()]);
        // IP が DE と分かっていれば DE が先頭
        std::fs::write(country_file(&root), "DE\n").unwrap();
        let c = candidates(&root, "10001", "ja");
        assert!(c[0].starts_with("ドイツ "), "{c:?}");
        // 言語が英語でもIPの国が最優先
        let c = candidates(&root, "10-001", "en");
        assert!(c[0].starts_with("Germany "), "{c:?}");
        // 数字で始まらない・短すぎる入力は対象外
        assert!(candidates(&root, "SW1A1AA", "en").is_empty());
        assert!(candidates(&root, "12", "en").is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn alnum_postal_codes_for_uk_canada_ireland_with_and_without_separators() {
        let root = tmp("alnum");
        for (cc, t) in [
            ("GB", "GB\tSW1A\tWestminster\tEngland\tENG\tGreater London\tGLA\t\t\t51.5\t-0.13\t4\n"),
            ("CA", "CA\tK1A\tOttawa\tOntario\tON\t\t\t\t\t45.4\t-75.7\t4\n"),
            ("IE", "IE\tD02\tDublin 2\tLeinster\tL\tDublin\tD\t\t\t53.3\t-6.2\t4\n"),
        ] {
            let m = parse_geonames(t);
            let s: String = m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect();
            std::fs::write(tsv_path(&root, cc), s).unwrap();
        }
        for input in ["SW1A 1AA", "SW1A1AA", "sw1a-1aa"] {
            let c = candidates(&root, input, "en");
            assert!(c.iter().any(|x| x.starts_with("United Kingdom ") && x.contains("Westminster")), "{input}: {c:?}");
        }
        for input in ["K1A 0B1", "K1A0B1", "k1a-0b1"] {
            let c = candidates(&root, input, "en");
            assert!(c.iter().any(|x| x.starts_with("Canada ") && x.contains("Ottawa")), "{input}: {c:?}");
        }
        assert!(candidates(&root, "D02 AF30", "en").iter().any(|x| x.starts_with("Ireland ")));
        // 郵便番号の形ではない(短い・英字だけ・記号入り)入力は対象外
        for input in ["hello", "abc", "a1", "SW1A!1AA", "okinawa"] {
            assert!(candidates(&root, input, "en").is_empty(), "{input}");
        }
        assert!(looks_like_alnum_postal("SW1A 1AA") && looks_like_alnum_postal("K1A-0B1") && !looks_like_alnum_postal("100-0001"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn candidates_are_capped_and_priority_countries_get_more() {
        let root = tmp("cap");
        // 同じ番号が多くの国にあり、各国に住所が5件ある
        for cc in ["US", "DE", "FR", "ES", "IT", "NL", "BE", "AT", "CH", "PL", "CZ"] {
            let t: String = (1..=5).map(|i| format!("{cc}\t12345\tPlace{i}\tState\tS\t\t\t\t\t0\t0\t4\n")).collect();
            let m = parse_geonames(&t);
            let s: String = m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect();
            std::fs::write(tsv_path(&root, cc), s).unwrap();
        }
        let c = candidates(&root, "12345", "en");
        assert!(c.len() <= 8, "{} 件: {c:?}", c.len());
        // 言語の国(en → US)は3件、IPの国(DE)も3件まで、他の国は1件ずつ
        assert_eq!(c.iter().filter(|x| x.starts_with("United States ")).count(), 3, "{c:?}");
        std::fs::write(country_file(&root), "DE\n").unwrap();
        let c = candidates(&root, "12345", "en");
        assert_eq!(c.iter().filter(|x| x.starts_with("Germany ")).count(), 3, "{c:?}");
        assert!(c.len() <= 8);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn language_country_comes_before_others() {
        let root = tmp("lang");
        for (cc, t) in [("US", "US\t90210\tBeverly Hills\tCalifornia\tCA\t\t\t\t\t0\t0\t4\n"), ("FR", "FR\t90210\tParis\tÎle-de-France\t11\t\t\t\t\t0\t0\t4\n")] {
            let m = parse_geonames(t);
            let s: String = m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect();
            std::fs::write(tsv_path(&root, cc), s).unwrap();
        }
        assert!(candidates(&root, "90210", "fr")[0].starts_with("France "));
        assert!(candidates(&root, "90210", "en")[0].starts_with("United States "));
        let _ = std::fs::remove_dir_all(&root);
    }
}
