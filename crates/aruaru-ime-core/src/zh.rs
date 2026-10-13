//! 中国語(簡体字・繁体字)のピンイン入力。
//!
//! データ: jieba の単語頻度表(MIT)・pinyin-data の読み(MIT)・phrase-pinyin-data の語の読み(MIT)・OpenCC の簡体→繁体の変換表(Apache-2.0)。
//! インストール時(と初回起動時)に GitHub から取得して `<データ領域>/zh.tsv` にまとめる。
//! 変換: ピンイン(声調なし。ü は v)を、単語の頻度(unigram)で最も確からしい語の並びに直す(Viterbi)。
//! 候補は「入力の先頭から何文字ぶんを使うか」つき。選ぶと、その分を確定して残りの入力が続く(部分確定)。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const SOURCES: &[(&str, &str)] = &[
    ("jieba-dict.txt", "https://raw.githubusercontent.com/fxsjy/jieba/master/jieba/dict.txt"),
    ("pinyin.txt", "https://raw.githubusercontent.com/mozillazg/pinyin-data/master/pinyin.txt"),
    ("large_pinyin.txt", "https://raw.githubusercontent.com/mozillazg/phrase-pinyin-data/master/large_pinyin.txt"),
    ("STCharacters.txt", "https://raw.githubusercontent.com/BYVoid/OpenCC/master/data/dictionary/STCharacters.txt"),
    ("STPhrases.txt", "https://raw.githubusercontent.com/BYVoid/OpenCC/master/data/dictionary/STPhrases.txt"),
    ("TWVariants.txt", "https://raw.githubusercontent.com/BYVoid/OpenCC/master/data/dictionary/TWVariants.txt"),
];

/// 声調記号つきの母音を、声調なしにする(ü は v)。
pub fn strip_tone(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ā' | 'á' | 'ǎ' | 'à' => 'a',
            'ē' | 'é' | 'ě' | 'è' | 'ê' => 'e',
            'ī' | 'í' | 'ǐ' | 'ì' => 'i',
            'ō' | 'ó' | 'ǒ' | 'ò' => 'o',
            'ū' | 'ú' | 'ǔ' | 'ù' => 'u',
            'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => 'v',
            'ń' | 'ň' | 'ǹ' => 'n',
            'ḿ' => 'm',
            c => c,
        })
        .collect()
}

fn parse_pinyin_txt(text: &str) -> HashMap<char, Vec<String>> {
    let mut m: HashMap<char, Vec<String>> = HashMap::new();
    for l in text.lines().filter(|l| !l.starts_with('#')) {
        let Some((cp, rest)) = l.split_once(':') else { continue };
        let Some(hex) = cp.trim().strip_prefix("U+") else { continue };
        let Some(ch) = u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) else { continue };
        let reads = rest.split('#').next().unwrap_or("").trim();
        let v: Vec<String> = reads.split(',').map(|r| strip_tone(r.trim())).filter(|r| !r.is_empty()).collect();
        if !v.is_empty() {
            m.insert(ch, v);
        }
    }
    m
}

/// OpenCC の辞書(`簡体<TAB>繁体1 繁体2…`)。最初の繁体を使う。
fn parse_opencc(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            let (k, v) = l.split_once('\t')?;
            Some((k.to_string(), v.split(' ').next()?.to_string()))
        })
        .collect()
}

/// 簡体字を繁体字にする(語を最長一致 → 字ごと)。
fn to_trad(s: &str, phrases: &HashMap<String, String>, chars: &HashMap<String, String>, tw: &HashMap<String, String>) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let mut hit = false;
        for len in (2..=cs.len().min(i + 8) - i).rev() {
            let k: String = cs[i..i + len].iter().collect();
            if let Some(v) = phrases.get(&k) {
                out.push_str(v);
                i += len;
                hit = true;
                break;
            }
        }
        if !hit {
            let k = cs[i].to_string();
            let t = chars.get(&k).cloned().unwrap_or(k);
            out.push_str(tw.get(&t).map(String::as_str).unwrap_or(&t));
            i += 1;
        }
    }
    out
}

/// 取得済みの元データから、`key<TAB>簡体<TAB>繁体<TAB>頻度` の行を作る。頻度の高い順。
pub fn build_tsv(src: &Path) -> Result<String, String> {
    let rd = |n: &str| std::fs::read_to_string(src.join(n)).map_err(|e| format!("{n}: {e}"));
    let chars = parse_pinyin_txt(&rd("pinyin.txt")?);
    let mut phrase_read: HashMap<String, Vec<String>> = HashMap::new();
    for l in rd("large_pinyin.txt")?.lines().filter(|l| !l.starts_with('#')) {
        if let Some((w, r)) = l.split_once(':') {
            let syl: Vec<String> = r.split_whitespace().map(strip_tone).collect();
            if syl.len() == w.chars().count() {
                phrase_read.entry(w.trim().to_string()).or_insert(syl);
            }
        }
    }
    let phrases = parse_opencc(&rd("STPhrases.txt")?);
    let cmap = parse_opencc(&rd("STCharacters.txt")?);
    let tw = parse_opencc(&rd("TWVariants.txt")?);
    let mut rows: Vec<(String, String, String, u64)> = vec![];
    let mut seen_char: std::collections::HashSet<(String, char)> = Default::default();
    for l in rd("jieba-dict.txt")?.lines() {
        let mut it = l.split_whitespace();
        let (Some(w), Some(f)) = (it.next(), it.next()) else { continue };
        let Ok(freq) = f.parse::<u64>() else { continue };
        let wc: Vec<char> = w.chars().collect();
        if wc.is_empty() || wc.len() > 8 || !wc.iter().all(|c| ('\u{4E00}'..='\u{9FFF}').contains(c)) {
            continue;
        }
        let syl: Vec<String> = match phrase_read.get(w) {
            Some(s) => s.clone(),
            None => {
                let mut v = vec![];
                for c in &wc {
                    match chars.get(c).and_then(|r| r.first()) {
                        Some(r) => v.push(r.clone()),
                        None => {
                            v.clear();
                            break;
                        }
                    }
                }
                v
            }
        };
        if syl.len() != wc.len() {
            continue;
        }
        if wc.len() == 1 {
            seen_char.insert((syl[0].clone(), wc[0]));
        }
        rows.push((syl.concat(), w.to_string(), to_trad(w, &phrases, &cmap, &tw), freq));
    }
    // 多音字・頻度表に無い漢字も、1字の候補として入れる(読みごと)
    for (c, reads) in &chars {
        if !('\u{4E00}'..='\u{9FFF}').contains(c) {
            continue;
        }
        for r in reads {
            if seen_char.insert((r.clone(), *c)) {
                let s = c.to_string();
                rows.push((r.clone(), s.clone(), to_trad(&s, &phrases, &cmap, &tw), 1));
            }
        }
    }
    if rows.len() < 50_000 {
        return Err(format!("辞書が小さすぎます({}件)。元データを確認してください", rows.len()));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0).then(b.3.cmp(&a.3)));
    Ok(rows.into_iter().map(|(k, s, t, f)| format!("{k}\t{s}\t{t}\t{f}")).collect::<Vec<_>>().join("\n"))
}

static GLOBAL: std::sync::RwLock<Option<ZhDict>> = std::sync::RwLock::new(None);

/// 常駐エンジンが読み込んだ辞書を登録する。
pub fn set_global(d: ZhDict) {
    if let Ok(mut w) = GLOBAL.write() {
        *w = Some(d);
    }
}

/// 登録済みの辞書で候補を作る(未登録なら空)。
pub fn candidates_global(input: &str, trad: bool, max: usize) -> Vec<(String, usize)> {
    GLOBAL.read().ok().and_then(|g| g.as_ref().map(|d| d.candidates(input, trad, max))).unwrap_or_default()
}

pub fn global_ready() -> bool {
    GLOBAL.read().map(|g| g.is_some()).unwrap_or(false)
}

pub struct ZhDict {
    /// ピンイン(声調なし・連結) → [(簡体, 繁体, 頻度)](頻度の高い順)。
    map: HashMap<String, Vec<(String, String, u64)>>,
    total_ln: f64,
}

impl ZhDict {
    pub fn parse(tsv: &str) -> Self {
        let mut map: HashMap<String, Vec<(String, String, u64)>> = HashMap::with_capacity(tsv.len() / 40);
        let mut total: u64 = 0;
        for l in tsv.lines() {
            let mut it = l.split('\t');
            if let (Some(k), Some(s), Some(t), Some(f)) = (it.next(), it.next(), it.next(), it.next()) {
                let f: u64 = f.parse().unwrap_or(1);
                total += f;
                map.entry(k.to_string()).or_default().push((s.to_string(), t.to_string(), f));
            }
        }
        for v in map.values_mut() {
            v.sort_by(|a, b| b.2.cmp(&a.2));
        }
        Self { map, total_ln: (total.max(2) as f64).ln() }
    }

    pub fn load(path: &Path) -> Option<Self> {
        std::fs::read_to_string(path).ok().map(|t| Self::parse(&t))
    }

    /// 辞書本体に、`extras_dir` の `*.tsv`(用語辞書パックの中国語の版。`ピンイン<TAB>簡体<TAB>繁体<TAB>頻度`)を合流して読み込む。
    pub fn load_with_extras(path: &Path, extras_dir: &Path) -> Option<Self> {
        let mut t = std::fs::read_to_string(path).ok()?;
        if let Ok(rd) = std::fs::read_dir(extras_dir) {
            let mut files: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "tsv")).collect();
            files.sort();
            for f in files {
                if let Ok(x) = std::fs::read_to_string(f) {
                    t.push('\n');
                    t.push_str(&x);
                }
            }
        }
        Some(Self::parse(&t))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn pick(v: &[(String, String, u64)], trad: bool, i: usize) -> Option<&String> {
        v.get(i).map(|e| if trad { &e.1 } else { &e.0 })
    }

    /// 入力(小文字のピンイン。`'` は音節の区切り)から、候補 `(文字列, 使う入力の文字数)` を返す。
    /// 先頭は、入力全体を最も確からしい語の並びにした文(部分確定が必要なら使う分だけ)。続いて、先頭から使う長さごとの語の候補。
    pub fn candidates(&self, input: &str, trad: bool, max: usize) -> Vec<(String, usize)> {
        // 区切り記号を除いた文字列と、元の入力での位置の対応
        let mut letters: Vec<char> = vec![];
        let mut pos: Vec<usize> = vec![]; // letters[i] が元の入力の何文字目の次か
        for (idx, c) in input.chars().enumerate() {
            if c.is_ascii_alphabetic() {
                letters.push(c.to_ascii_lowercase());
                pos.push(idx + 1);
            }
        }
        let n = letters.len();
        if n == 0 {
            return vec![];
        }
        let consumed = |letters_used: usize| -> usize { if letters_used == 0 { 0 } else { pos[letters_used - 1] } };
        let key = |a: usize, b: usize| -> String { letters[a..b].iter().collect() };
        // Viterbi: best[i] = 先頭 i 文字を語の並びにしたときの最良の (点数, 直前の位置, 語)
        let mut best: Vec<Option<(f64, usize, &String)>> = vec![None; n + 1];
        let mut best0: Vec<f64> = vec![f64::NEG_INFINITY; n + 1];
        best0[0] = 0.0;
        for i in 0..n {
            if best0[i] == f64::NEG_INFINITY {
                continue;
            }
            for j in i + 1..=n.min(i + 24) {
                if let Some(v) = self.map.get(&key(i, j)) {
                    if let Some(w) = Self::pick(v, trad, 0) {
                        let score = best0[i] + (v[0].2 as f64).ln() - self.total_ln;
                        if score > best0[j] {
                            best0[j] = score;
                            best[j] = Some((score, i, w));
                        }
                    }
                }
            }
        }
        let mut out: Vec<(String, usize)> = vec![];
        let mut add = |t: String, used: usize, out: &mut Vec<(String, usize)>| {
            if !t.is_empty() && !out.iter().any(|(x, u)| *x == t && *u == used) {
                out.push((t, used));
            }
        };
        // 入力全体の文(語を2つ以上つなぐ場合も含む)
        if best[n].is_some() {
            let mut words: Vec<&String> = vec![];
            let mut i = n;
            while i > 0 {
                let Some((_, from, w)) = best[i] else { break };
                words.push(w);
                i = from;
            }
            words.reverse();
            add(words.into_iter().map(String::as_str).collect::<String>(), consumed(n), &mut out);
        } else {
            // 最後まで読めないときは、読めるところまでの最長の並び
            if let Some(m) = (1..=n).rev().find(|k| best[*k].is_some()) {
                let mut words: Vec<&String> = vec![];
                let mut i = m;
                while i > 0 {
                    let Some((_, from, w)) = best[i] else { break };
                    words.push(w);
                    i = from;
                }
                words.reverse();
                add(words.into_iter().map(String::as_str).collect::<String>(), consumed(m), &mut out);
            }
        }
        // 先頭から使う長さごとの語(長い順)
        for len in (1..=n.min(24)).rev() {
            if let Some(v) = self.map.get(&key(0, len)) {
                for i in 0..v.len().min(6) {
                    if let Some(w) = Self::pick(v, trad, i) {
                        add(w.clone(), consumed(len), &mut out);
                    }
                }
            }
            if out.len() >= max {
                break;
            }
        }
        out.truncate(max);
        out
    }
}

// ---------------------------------------------------------------- 自動取得

/// 漢字(簡体・繁体どちらでも)→ ピンイン(声調なし)の逆引き。辞書の中で頻度が最も高い読みを使う。
/// 人名辞書の中国語の表記(簡体・繁体)に、入力用のピンインを付けるために使う。
pub struct PinyinRev {
    map: HashMap<String, (String, u64)>,
    max_chars: usize,
}

impl PinyinRev {
    pub fn from_dict(d: &ZhDict) -> Self {
        let mut map: HashMap<String, (String, u64)> = HashMap::with_capacity(d.map.len() * 2);
        let mut max_chars = 1;
        for (py, v) in &d.map {
            for (s, t, f) in v {
                for w in [s, t] {
                    max_chars = max_chars.max(w.chars().count());
                    let better = map.get(w).map_or(true, |(_, bf)| f > bf);
                    if better {
                        map.insert(w.clone(), (py.clone(), *f));
                    }
                }
            }
        }
        Self { map, max_chars: max_chars.min(8) }
    }

    /// 漢字列のピンイン(連結)。中黒・空白は読み飛ばす。最長一致で区切り、全体を覆えなければ None(誤った読みを付けない)。
    pub fn pinyin(&self, hanzi: &str) -> Option<String> {
        let cs: Vec<char> = hanzi.chars().filter(|c| !matches!(c, '·' | '・' | '･' | '•' | ' ' | '　')).collect();
        if cs.is_empty() {
            return None;
        }
        let mut out = String::new();
        let mut i = 0;
        while i < cs.len() {
            let mut hit = None;
            for len in (1..=self.max_chars.min(cs.len() - i)).rev() {
                let w: String = cs[i..i + len].iter().collect();
                if let Some((py, _)) = self.map.get(&w) {
                    hit = Some((py.as_str(), len));
                    break;
                }
            }
            let (py, len) = hit?;
            out.push_str(py);
            i += len;
        }
        Some(out)
    }
}

#[cfg(test)]
mod pinyin_rev_tests {
    use super::*;

    #[test]
    fn reverse_lookup_longest_match_and_gaps() {
        let d = ZhDict::parse("ai\t爱\t愛\t900\nyin\t因\t因\t800\nsi\t斯\t斯\t700\ntan\t坦\t坦\t600\naiyinsitan\t爱因斯坦\t愛因斯坦\t50\nle\t乐\t樂\t900\nyue\t乐\t樂\t100\n");
        let r = PinyinRev::from_dict(&d);
        assert_eq!(r.pinyin("爱因斯坦").as_deref(), Some("aiyinsitan"));
        assert_eq!(r.pinyin("愛因斯坦").as_deref(), Some("aiyinsitan"), "繁体も引ける");
        assert_eq!(r.pinyin("爱·斯坦").as_deref(), Some("aisitan"), "中黒は読み飛ばす");
        assert_eq!(r.pinyin("乐").as_deref(), Some("le"), "頻度の高い読み");
        assert_eq!(r.pinyin("爱龘"), None, "辞書に無い字を含むなら読みを付けない");
        assert_eq!(r.pinyin(""), None);
    }
}

pub fn zh_path(root: &Path) -> PathBuf {
    root.join("zh.tsv")
}

const LICENSE_TEXT: &str = "中国語のピンイン辞書について / About the Chinese pinyin dictionary\n\n\
 - jieba (c) Sun Junyi, MIT License: https://github.com/fxsjy/jieba  (単語の頻度 / word frequencies)\n\
 - pinyin-data / phrase-pinyin-data (c) mozillazg, MIT License: https://github.com/mozillazg/pinyin-data , https://github.com/mozillazg/phrase-pinyin-data  (読み / readings)\n\
 - OpenCC (c) BYVoid, Apache License 2.0: https://github.com/BYVoid/OpenCC  (簡体字→繁体字 / Simplified→Traditional)\n\n\
取得した元データから zh.tsv を作っています(表記・ピンイン・頻度だけ)。 / zh.tsv is derived from the above (surface, pinyin key, frequency only).\n";

#[derive(Debug, PartialEq)]
pub enum Ensured {
    AlreadyInstalled,
    Installed(usize),
    Unavailable(String),
}

/// ピンイン辞書の元データ(6ファイル)を GitHub から取得して `zh.tsv` を作る。すでにあれば何もしない。
pub fn ensure(root: &Path, allow_download: bool) -> Ensured {
    let p = zh_path(root);
    if p.is_file() {
        return Ensured::AlreadyInstalled;
    }
    if !allow_download {
        return Ensured::Unavailable("中国語の辞書は未インストールです".into());
    }
    let r = (|| -> Result<usize, String> {
        let src = root.join("zh-src");
        std::fs::create_dir_all(&src).map_err(|e| e.to_string())?;
        let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(300)).user_agent("aruaru-ime").build();
        for (name, url) in SOURCES {
            if !crate::update::is_trusted_url(url) {
                return Err(format!("取得先が許可されていません: {url}"));
            }
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut agent.get(url).call().map_err(|e| format!("{name}: {e}"))?.into_reader().take(64 * 1024 * 1024), &mut buf).map_err(|e| e.to_string())?;
            std::fs::write(src.join(name), buf).map_err(|e| e.to_string())?;
        }
        let tsv = build_tsv(&src)?;
        let n = tsv.lines().count();
        std::fs::write(root.join("ZH-LICENSES.txt"), LICENSE_TEXT).map_err(|e| e.to_string())?;
        std::fs::write(&p, tsv).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_dir_all(&src); // 元データは、辞書を作ったら消す
        Ok(n)
    })();
    match r {
        Ok(n) => Ensured::Installed(n),
        Err(e) => Ensured::Unavailable(e),
    }
}

use std::io::Read;

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
zhong\t中\t中\t200\nzhong\t种\t種\t50\nguo\t国\t國\t300\nzhongguo\t中国\t中國\t5000\nwen\t文\t文\t100\nzhongwen\t中文\t中文\t3000\n\
ren\t人\t人\t400\nmin\t民\t民\t80\nrenmin\t人民\t人民\t2000\nni\t你\t你\t900\nhao\t好\t好\t800\nnihao\t你好\t你好\t4000\nzhongguoren\t中国人\t中國人\t2500\n";

    #[test]
    fn strips_tones_and_umlaut() {
        assert_eq!(strip_tone("zhōng"), "zhong");
        assert_eq!(strip_tone("lǜ"), "lv");
        assert_eq!(strip_tone("nǚ"), "nv");
    }

    #[test]
    fn sentence_viterbi_and_partial_candidates() {
        let d = ZhDict::parse(SAMPLE);
        let c = d.candidates("zhongguorenmin", false, 10);
        assert_eq!(c[0], ("中国人民".to_string(), 14), "{c:?}");
        let c = d.candidates("nihao", false, 10);
        assert_eq!(c[0], ("你好".to_string(), 5));
        // 繁体字
        let c = d.candidates("zhongguo", true, 10);
        assert_eq!(c[0].0, "中國");
        // 途中までしか読めない入力(zhongguor)は、読めるところまでの最良から始まる
        let c = d.candidates("zhongguor", false, 10);
        assert_eq!(c[0].0, "中国", "{c:?}");
        assert_eq!(c[0].1, 8, "使う入力は先頭の8文字");
        // 区切り記号 ' は、使う文字数の数えに含まれる
        let c = d.candidates("ni'hao", false, 10);
        assert_eq!(c[0], ("你好".to_string(), 6));
        assert!(d.candidates("", false, 5).is_empty() && d.candidates("zzz", false, 5).is_empty());
    }

    #[test]
    fn real_data_if_downloaded() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/zh-src");
        if !src.join("jieba-dict.txt").is_file() {
            return;
        }
        let tsv = build_tsv(&src).unwrap();
        let d = ZhDict::parse(&tsv);
        assert!(d.len() > 100_000, "{}", d.len());
        let c = d.candidates("zhongguorenmin", false, 5);
        assert_eq!(c[0].0, "中国人民", "{c:?}");
        let c = d.candidates("woaizhongwen", false, 5);
        assert_eq!(c[0].0, "我爱中文", "{c:?}");
        assert!(d.candidates("zhongguo", true, 8).iter().any(|(t, _)| t == "中國"));
        let c = d.candidates("nihao", false, 8);
        assert_eq!(c[0].0, "你好", "{c:?}");
        assert!(d.candidates("lv", false, 8).iter().any(|(t, _)| t == "绿" || t == "律" || t == "旅"));
    }
}
