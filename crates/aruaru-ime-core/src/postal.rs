//! 郵便番号 → 住所。日本郵便の公開データ(`utf_ken_all.csv`)から作った `postal.tsv`(`7桁<TAB>住所|住所…`)を引く。
//! 7桁(または `123-4567`)の数字を打つと、候補に住所が出る。データは、利用者が右クリックの「郵便番号の住所辞書を取得」を選んだとき(または
//! `aruaru-ime postal-install`)だけ、日本郵便の公開ファイルから取得する(自動では取得しない)。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const SOURCE_URL: &str = "https://www.post.japanpost.jp/zipcode/dl/utf/zip/utf_ken_all.zip";

pub fn path(root: &Path) -> PathBuf {
    root.join("postal.tsv")
}

/// 町域名の整形: 括弧書き・「以下に掲載がない場合」「〜の次に番地がくる場合」などは空にする。
fn clean_town(t: &str) -> String {
    if t.contains("以下に掲載がない場合") || t.contains("場合") || t.contains("一円") && t.len() > 12 {
        return String::new();
    }
    let t = t.split('（').next().unwrap_or(t);
    t.trim().to_string()
}

fn field(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

/// `utf_ken_all.csv` を、7桁 → 住所の一覧にする。
pub fn parse_ken_all(csv: &str) -> BTreeMap<String, Vec<String>> {
    let mut m: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for l in csv.lines() {
        let f: Vec<&str> = l.split(',').collect();
        if f.len() < 9 {
            continue;
        }
        let zip = field(f[2]);
        if zip.len() != 7 || !zip.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let addr = format!("{}{}{}", field(f[6]), field(f[7]), clean_town(&field(f[8])));
        let v = m.entry(zip).or_default();
        if !v.contains(&addr) {
            v.push(addr);
        }
    }
    m
}

pub fn to_tsv(m: &BTreeMap<String, Vec<String>>) -> String {
    m.iter().map(|(z, a)| format!("{z}\t{}\n", a.join("|"))).collect()
}

/// 郵便番号の入力(7桁、または 123-4567)を、7桁にする。
pub fn normalize_zip(input: &str) -> Option<String> {
    let d: String = input.chars().filter(|c| *c != '-' && *c != '−' && *c != 'ー').collect();
    let shape_ok = input.chars().all(|c| c.is_ascii_digit() || c == '-') && (input.len() == 7 || (input.len() == 8 && input.as_bytes().get(3) == Some(&b'-')));
    (shape_ok && d.len() == 7 && d.chars().all(|c| c.is_ascii_digit())).then_some(d)
}

/// 住所を引く(データが無ければ空)。ファイルが変わったときだけ読み直す。
pub fn lookup(zip7: &str) -> Vec<String> {
    use std::sync::Mutex;
    use std::time::SystemTime;
    static CACHE: Mutex<Option<(SystemTime, std::collections::HashMap<String, Vec<String>>)>> = Mutex::new(None);
    let p = path(&crate::components::data_dir());
    let Ok(mt) = std::fs::metadata(&p).and_then(|m| m.modified()) else { return vec![] };
    let Ok(mut g) = CACHE.lock() else { return vec![] };
    if g.as_ref().map(|(t, _)| *t != mt).unwrap_or(true) {
        let mut h = std::collections::HashMap::new();
        if let Ok(t) = std::fs::read_to_string(&p) {
            for l in t.lines() {
                if let Some((z, a)) = l.split_once('\t') {
                    h.insert(z.to_string(), a.split('|').map(String::from).collect());
                }
            }
        }
        *g = Some((mt, h));
    }
    g.as_ref().and_then(|(_, h)| h.get(zip7).cloned()).unwrap_or_default()
}

pub fn is_installed(root: &Path) -> bool {
    path(root).is_file()
}

/// 日本郵便の公開ファイルを取得して `postal.tsv` を作る(利用者が選んだときだけ呼ぶ)。作った郵便番号の数を返す。
pub fn install(root: &Path) -> Result<usize, String> {
    use std::io::Read;
    let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(120)).user_agent("aruaru-ime").build();
    let mut zip_bytes = Vec::new();
    agent.get(SOURCE_URL).call().map_err(|e| format!("取得できません: {e}"))?.into_reader().take(30 * 1024 * 1024).read_to_end(&mut zip_bytes).map_err(|e| e.to_string())?;
    let mut ar = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).map_err(|e| e.to_string())?;
    let mut csv = String::new();
    {
        let mut f = ar.by_index(0).map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        f.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        csv.push_str(&String::from_utf8_lossy(&raw));
    }
    let m = parse_ken_all(&csv);
    if m.len() < 100_000 {
        return Err(format!("データが小さすぎます({}件)", m.len()));
    }
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    std::fs::write(path(root), to_tsv(&m)).map_err(|e| e.to_string())?;
    Ok(m.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
13101,\"100  \",\"1000001\",\"ﾄｳｷｮｳﾄ\",\"ﾁﾖﾀﾞｸ\",\"ﾁﾖﾀﾞ\",\"東京都\",\"千代田区\",\"千代田\",0,0,0,0,0,0\n\
13101,\"102  \",\"1020072\",\"ﾄｳｷｮｳﾄ\",\"ﾁﾖﾀﾞｸ\",\"ｲｲﾀﾞﾊﾞｼ\",\"東京都\",\"千代田区\",\"飯田橋\",0,0,0,0,0,0\n\
01101,\"060  \",\"0600000\",\"ﾎｯｶｲﾄﾞｳ\",\"ｻｯﾎﾟﾛｼﾁｭｳｵｳｸ\",\"ｲｶﾆｹｲｻｲｶﾞﾅｲﾊﾞｱｲ\",\"北海道\",\"札幌市中央区\",\"以下に掲載がない場合\",0,0,0,0,0,0\n\
13101,\"100  \",\"1000004\",\"ﾄｳｷｮｳﾄ\",\"ﾁﾖﾀﾞｸ\",\"ｵｵﾃﾏﾁ\",\"東京都\",\"千代田区\",\"大手町（次のビルを除く）\",0,0,0,0,0,0\n";

    #[test]
    fn parses_cleans_and_normalizes() {
        let m = parse_ken_all(SAMPLE);
        assert_eq!(m["1000001"], vec!["東京都千代田区千代田"]);
        assert_eq!(m["0600000"], vec!["北海道札幌市中央区"], "「以下に掲載がない場合」は町域を空にする");
        assert_eq!(m["1000004"], vec!["東京都千代田区大手町"], "括弧書きは落とす");
        assert!(to_tsv(&m).contains("1020072\t東京都千代田区飯田橋"));
        assert_eq!(normalize_zip("1000001").as_deref(), Some("1000001"));
        assert_eq!(normalize_zip("100-0001").as_deref(), Some("1000001"));
        assert_eq!(normalize_zip("10000"), None);
        assert_eq!(normalize_zip("100-00012"), None);
        assert_eq!(normalize_zip("abc-defg"), None);
    }
}
