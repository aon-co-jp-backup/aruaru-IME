//! ファイルから単語・熟語・文章を取り込む: .txt / .csv / .tsv / .md / .xlsx / .docx / .pptx / .pdf。
//!
//! 1行(段落・セル)を1項目として取り出し、読みは「読み<TAB>表記」ならその読みを使い、
//! そうでなければ Mozc 辞書の逆引き(表記→読み)で自動生成する。読みを作れない項目は取り込まずに数える。
//! 取り込んだ項目には元ファイル名を記録するので、ファイル単位でまとめて削除できる。

use crate::kanji::Dict;
use crate::romaji;
use crate::userdict::{normalize_reading, UserDict};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

pub const MAX_ITEMS: usize = 5000;
const MAX_ITEM_CHARS: usize = 200;

#[derive(Debug, Default, PartialEq)]
pub struct Report {
    /// ファイルから取り出した項目数。
    pub items: usize,
    /// 新しく登録した数。
    pub added: usize,
    /// すでに登録済みだった数。
    pub existing: usize,
    /// 読みを決められず取り込まなかった数。
    pub no_reading: usize,
}

fn decode_text(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.trim_start_matches('\u{feff}').to_string();
    }
    // UTF-8 でなければ Shift_JIS(Windows の既定のメモ帳・Excel の CSV)とみなす
    encoding_rs::SHIFT_JIS.decode(bytes).0.into_owned()
}

fn xml_paragraphs(xml: &[u8], para: &[u8], text: &[u8]) -> Vec<String> {
    let mut r = Reader::from_reader(xml);
    let mut out = vec![];
    let mut cur = String::new();
    let mut in_t = false;
    let mut buf = Vec::new();
    loop {
        match r.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                if e.local_name().as_ref() == text {
                    in_t = true;
                }
            }
            Ok(Event::End(e)) => {
                let n = e.local_name();
                if n.as_ref() == text {
                    in_t = false;
                } else if n.as_ref() == para {
                    out.push(std::mem::take(&mut cur));
                }
            }
            Ok(Event::Text(t)) if in_t => {
                if let Ok(s) = t.unescape() {
                    cur.push_str(&s);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn read_zip_file(z: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>, name: &str) -> Option<Vec<u8>> {
    let mut f = z.by_name(name).ok()?;
    let mut b = Vec::new();
    f.read_to_end(&mut b).ok()?;
    Some(b)
}

/// ファイルから文字列の項目(行・段落・セル)を取り出す。
pub fn extract_lines(path: &Path) -> Result<Vec<String>, String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let bytes = std::fs::read(path).map_err(|e| format!("読み込めません: {e}"))?;
    let zip_of = |bytes: Vec<u8>| zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("ファイルを開けません: {e}"));
    match ext.as_str() {
        "txt" | "text" | "csv" | "tsv" | "md" | "log" => Ok(decode_text(&bytes).lines().map(str::to_string).collect()),
        "docx" => {
            let mut z = zip_of(bytes)?;
            let doc = read_zip_file(&mut z, "word/document.xml").ok_or("word/document.xml がありません")?;
            Ok(xml_paragraphs(&doc, b"p", b"t"))
        }
        "pptx" => {
            let mut z = zip_of(bytes)?;
            let mut names: Vec<String> = (0..z.len()).filter_map(|i| z.by_index(i).ok().map(|f| f.name().to_string())).filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml")).collect();
            names.sort_by_key(|n| n.trim_start_matches("ppt/slides/slide").trim_end_matches(".xml").parse::<u32>().unwrap_or(0));
            let mut out = vec![];
            for n in names {
                if let Some(x) = read_zip_file(&mut z, &n) {
                    out.extend(xml_paragraphs(&x, b"p", b"t"));
                }
            }
            Ok(out)
        }
        "xlsx" => {
            let mut z = zip_of(bytes)?;
            // 文字列セルは共有文字列表にまとまっている(各 <si> が1セル)
            let ss = read_zip_file(&mut z, "xl/sharedStrings.xml").ok_or("文字列セルがありません")?;
            Ok(xml_paragraphs(&ss, b"si", b"t"))
        }
        "pdf" => {
            let doc = lopdf::Document::load_mem(&bytes).map_err(|e| format!("PDFを開けません: {e}"))?;
            let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
            let text = doc.extract_text(&pages).map_err(|e| format!("PDFから文字を取り出せません(画像だけのPDFは未対応): {e}"))?;
            Ok(text.lines().map(str::to_string).collect())
        }
        other => Err(format!("対応していない形式です: .{other}(txt/csv/xlsx/docx/pptx/pdf)")),
    }
}

/// 長い行は文末(。！？)で区切り、短すぎる・長すぎる項目は捨てる。重複は除く。
pub fn split_items(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for l in lines {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        let parts: Vec<String> = if l.chars().count() > 60 && !l.contains('\t') {
            let mut v = vec![];
            let mut cur = String::new();
            for c in l.chars() {
                cur.push(c);
                if matches!(c, '。' | '！' | '？' | '!' | '?') {
                    v.push(std::mem::take(&mut cur));
                }
            }
            if !cur.trim().is_empty() {
                v.push(cur);
            }
            v
        } else {
            vec![l.to_string()]
        };
        for p in parts {
            let p = p.trim().to_string();
            let n = p.chars().count();
            if (2..=MAX_ITEM_CHARS).contains(&n) && !out.contains(&p) {
                out.push(p);
            }
            if out.len() >= MAX_ITEMS {
                return out;
            }
        }
    }
    out
}

/// 表記→読み(最小コストの1つ)の索引。取り込み時だけ作る(約100MBのメモリ)。
pub fn surface_index(d: &Dict) -> HashMap<String, (u16, String)> {
    d.surface_index()
}

fn is_punct_or_space(c: char) -> bool {
    c.is_whitespace() || (!c.is_alphanumeric() && !matches!(c, 'ー' | 'ぁ'..='ゖ' | 'ァ'..='ヺ'))
}

/// 数字の直後に付く助数詞の読み(辞書の最小コストは「年→とし」のように数字の後では不自然なため)。
fn counter_reading(p: &str) -> Option<&'static str> {
    Some(match p {
        "年" => "ねん",
        "月" => "がつ",
        "日" => "にち",
        "時" => "じ",
        "時間" => "じかん",
        "分" => "ふん",
        "秒" => "びょう",
        "円" => "えん",
        "人" => "にん",
        "個" => "こ",
        "回" => "かい",
        "件" => "けん",
        "歳" => "さい",
        "枚" => "まい",
        "台" => "だい",
        "本" => "ほん",
        _ => return None,
    })
}

/// 表記から読みを作る。辞書の逆引きで最少の区切りに分けてつなぐ。作れなければ None。
pub fn reading_of(surface: &str, idx: &HashMap<String, (u16, String)>) -> Option<String> {
    let chars: Vec<char> = surface.chars().filter(|c| !is_punct_or_space(*c)).collect();
    if chars.is_empty() {
        return None;
    }
    // 全部が英数字ならそのまま(小文字)読みにする(英単語・型番などの登録)
    if chars.iter().all(|c| c.is_ascii_alphanumeric()) {
        return Some(chars.iter().collect::<String>().to_lowercase());
    }
    // 全部かなならそのまま
    if chars.iter().all(|c| matches!(c, 'ー' | 'ぁ'..='ゖ' | 'ァ'..='ヺ')) {
        return Some(romaji::kata_to_hira(&chars.iter().collect::<String>()));
    }
    // DP: dp[i] = (区切り数, コスト, 読み)
    let n = chars.len();
    let mut dp: Vec<Option<(usize, u32, String)>> = vec![None; n + 1];
    dp[0] = Some((0, 0, String::new()));
    for i in 0..n {
        let Some((segs, cost, rd)) = dp[i].clone() else { continue };
        for j in i + 1..=n.min(i + 24) {
            let piece: String = chars[i..j].iter().collect();
            let has_digit = piece.chars().any(|c| c.is_ascii_digit());
            let counter = if i > 0 && chars[i - 1].is_ascii_digit() { counter_reading(&piece) } else { None };
            let cand = if let Some(r) = counter {
                Some((0, r.to_string()))
            } else if has_digit && piece.chars().all(|c| c.is_ascii_digit()) {
                // 数字の並びはそのまま読みにする(辞書の「2026年」→漢数字読み などを避ける)
                Some((0, piece.clone()))
            } else if has_digit {
                None
            } else if let Some((c, r)) = idx.get(&piece) {
                Some((*c as u32, r.clone()))
            } else if piece.chars().count() == 1 && matches!(chars[i], 'ー' | 'ぁ'..='ゖ' | 'ァ'..='ヺ') {
                Some((3000, romaji::kata_to_hira(&piece)))
            } else if piece.chars().count() == 1 && chars[i].is_ascii_digit() {
                Some((2000, piece.clone()))
            } else {
                None
            };
            if let Some((c, r)) = cand {
                let new = (segs + 1, cost + c, format!("{rd}{r}"));
                if dp[j].as_ref().map_or(true, |old| (new.0, new.1) < (old.0, old.1)) {
                    dp[j] = Some(new);
                }
            }
        }
    }
    dp[n].clone().map(|x| x.2)
}

/// 項目を利用者辞書へ登録する。同じファイルを再取り込みすると、そのファイル由来の古い登録を入れ替える。
pub fn import_items(user: &mut UserDict, source: &str, items: &[String], idx: Option<&HashMap<String, (u16, String)>>) -> std::io::Result<Report> {
    let mut rep = Report { items: items.len(), ..Default::default() };
    user.remove_source(source)?;
    let empty = HashMap::new();
    let idx = idx.unwrap_or(&empty);
    for it in items {
        let (reading, surface) = match it.split_once('\t') {
            Some((r, s)) if !normalize_reading(r).is_empty() && !s.trim().is_empty() => (normalize_reading(r), s.trim().to_string()),
            _ => match reading_of(it, idx) {
                Some(r) => (r, it.clone()),
                None => {
                    rep.no_reading += 1;
                    continue;
                }
            },
        };
        if user.add_from(&reading, &surface, Some(source))? {
            rep.added += 1;
        } else {
            rep.existing += 1;
        }
    }
    Ok(rep)
}

/// ファイルを丸ごと取り込む(取り出し → 項目化 → 読み生成 → 登録)。
pub fn import_file(user: &mut UserDict, path: &Path, dict: Option<&Dict>) -> Result<Report, String> {
    let items = split_items(&extract_lines(path)?);
    let idx = dict.map(|d| d.surface_index());
    let source = path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
    import_items(user, &source, &items, idx.as_ref()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmpdir(n: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-doc-{n}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn make_zip(path: &Path, files: &[(&str, &str)]) {
        let f = std::fs::File::create(path).unwrap();
        let mut z = zip::ZipWriter::new(f);
        for (name, body) in files {
            z.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(body.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }

    #[test]
    fn extracts_docx_pptx_xlsx() {
        let d = tmpdir("ext");
        make_zip(&d.join("a.docx"), &[("word/document.xml", r#"<w:document xmlns:w="x"><w:body><w:p><w:r><w:t>お世話になっております。</w:t></w:r></w:p><w:p><w:r><w:t>よろしく</w:t></w:r><w:r><w:t>お願いします</w:t></w:r></w:p></w:body></w:document>"#)]);
        assert_eq!(extract_lines(&d.join("a.docx")).unwrap(), vec!["お世話になっております。", "よろしくお願いします"]);
        make_zip(&d.join("a.pptx"), &[("ppt/slides/slide2.xml", r#"<p:sld xmlns:a="x" xmlns:p="y"><a:p><a:r><a:t>二枚目</a:t></a:r></a:p></p:sld>"#), ("ppt/slides/slide1.xml", r#"<p:sld xmlns:a="x" xmlns:p="y"><a:p><a:r><a:t>一枚目</a:t></a:r></a:p></p:sld>"#)]);
        assert_eq!(extract_lines(&d.join("a.pptx")).unwrap(), vec!["一枚目", "二枚目"]);
        make_zip(&d.join("a.xlsx"), &[("xl/sharedStrings.xml", r#"<sst xmlns="x"><si><t>請求書</t></si><si><r><t>納</t></r><r><t>品書</t></r></si></sst>"#)]);
        assert_eq!(extract_lines(&d.join("a.xlsx")).unwrap(), vec!["請求書", "納品書"]);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn txt_utf8_and_sjis() {
        let d = tmpdir("txt");
        std::fs::write(d.join("u.txt"), "かいしゃ\t会社\n株式会社AON\n").unwrap();
        assert_eq!(extract_lines(&d.join("u.txt")).unwrap().len(), 2);
        let (sj, _, _) = encoding_rs::SHIFT_JIS.encode("お疲れ様です\n");
        std::fs::write(d.join("s.txt"), &*sj).unwrap();
        assert_eq!(extract_lines(&d.join("s.txt")).unwrap(), vec!["お疲れ様です"]);
        assert!(extract_lines(&d.join("x.exe")).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn splits_long_lines_and_dedups() {
        let long = format!("{}。{}。", "あ".repeat(40), "い".repeat(40));
        let items = split_items(&[long, "短".to_string(), "同じ文".to_string(), "同じ文".to_string()]);
        assert_eq!(items.len(), 3);
        assert!(!items.contains(&"短".to_string()));
    }

    #[test]
    fn import_replace_and_remove_by_file() {
        let d = tmpdir("imp");
        let mut u = UserDict::load(&d.join("u.json"));
        // 辞書なしでも、かな・英数字・「読み<TAB>表記」は取り込める
        let items = vec!["ありがとうございます".to_string(), "GitHub".to_string(), "かいしゃ\t会社".to_string(), "難読漢字語".to_string()];
        let r = import_items(&mut u, "list.txt", &items, None).unwrap();
        assert_eq!((r.added, r.no_reading), (3, 1));
        assert_eq!(u.exact("github"), vec!["GitHub"]);
        // 再取り込みで古い分は入れ替え(重複しない)
        let r2 = import_items(&mut u, "list.txt", &items[..1], None).unwrap();
        assert_eq!(r2.added, 1);
        assert_eq!(u.entries().len(), 1);
        assert_eq!(u.remove_source("list.txt").unwrap(), 1);
        assert!(u.entries().is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn reading_from_real_dictionary() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dict.bin");
        let Ok(d) = Dict::load(&p) else { return };
        let idx = d.surface_index();
        assert_eq!(reading_of("お世話になっております。", &idx).as_deref(), Some("おせわになっております"));
        assert_eq!(reading_of("会議の資料", &idx).as_deref(), Some("かいぎのしりょう"));
        assert_eq!(reading_of("2026年", &idx).as_deref(), Some("2026ねん"));
    }
}
