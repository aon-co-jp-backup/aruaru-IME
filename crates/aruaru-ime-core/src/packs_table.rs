//! 表(`data/packs/*.tsv`)から作る用語辞書パック: 花の名前・国名・主要な都市名・日本の駅名。
//! 列: `よみ  日本語  English  簡体  繁体  ピンイン  한글  fr  es  de  it  pt`(空の列は省略可。駅名は先頭の3列だけ)。
//! 日本語の読みと表記に加えて、英語(ローマ字のつづりそのまま)・中国語(簡体・繁体)・韓国語・ヨーロッパ言語の版も作る。
//! 作ったパックは `packs::all()` に合流し、既定でインストールされる。

use crate::packs::Pack;
use crate::packs_i18n::I18n;
use crate::wish::Category;

struct Table {
    id: &'static str,
    title: &'static str,
    src: &'static str,
    /// 駅名: 表記の後ろに「駅」(読みは「えき」)も入れる。
    station: bool,
}

const TABLES: &[Table] = &[
    Table { id: "flowers", title: "花の名前", src: include_str!("../pack_tables/flowers.tsv"), station: false },
    Table { id: "countries", title: "国名", src: include_str!("../pack_tables/countries.tsv"), station: false },
    Table { id: "cities", title: "主要な都市名", src: include_str!("../pack_tables/cities.tsv"), station: false },
    Table { id: "stations", title: "日本の駅名", src: include_str!("../pack_tables/stations.tsv"), station: true },
];

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// 英語のつづり(スペースなし・英数字のみ)を、利用者辞書に入れられる形にする。
fn ascii_key(en: &str) -> Option<String> {
    let k: String = en.chars().filter(|c| !c.is_whitespace()).collect();
    (!k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric())).then(|| k.to_lowercase())
}

pub fn build() -> (Vec<Pack>, Vec<I18n>) {
    let mut packs = vec![];
    let mut i18n = vec![];
    for t in TABLES {
        let mut entries = String::new();
        let (mut zh, mut ko, mut eu) = (String::new(), String::new(), String::new());
        for l in t.src.lines().filter(|l| !l.trim().is_empty()) {
            let c: Vec<&str> = l.split('\t').collect();
            let col = |i: usize| c.get(i).map(|s| s.trim()).unwrap_or("");
            let (kana, ja, en) = (col(0), col(1), col(2));
            if kana.is_empty() || ja.is_empty() {
                continue;
            }
            entries.push_str(&format!("{kana}\t{ja}\n"));
            if t.station {
                entries.push_str(&format!("{kana}えき\t{ja}駅\n"));
            }
            // 中国語(簡体・繁体)
            if !col(3).is_empty() && !col(5).is_empty() {
                zh.push_str(&format!("{}\t{}\t{}\n", col(5).replace(' ', ""), col(3), if col(4).is_empty() { col(3) } else { col(4) }));
            }
            // 韓国語
            if !col(6).is_empty() {
                ko.push_str(&format!("{}\n", col(6)));
            }
            // ヨーロッパ言語: キーは、英語のつづり(小文字・スペースなし)。各国語のつづりに対応する
            if let Some(k) = ascii_key(en) {
                // 英語: 入力した小文字のつづりを、大文字小文字つきの名前にする
                if en != k {
                    eu.push_str(&format!("en\t{k}\t{en}\n"));
                }
                for (i, lang) in ["fr", "es", "de", "it", "pt"].iter().enumerate() {
                    let v = col(7 + i);
                    if !v.is_empty() && v != en {
                        eu.push_str(&format!("{lang}\t{k}\t{v}\n"));
                    }
                }
            }
            // 英語の表記(大文字小文字つき)は、フランス語等の語の表に、英語名そのままも出せるよう
        }
        let id: &'static str = t.id;
        packs.push(Pack { id, title: t.title, category: Category::Other, entries: leak(entries) });
        i18n.push(I18n { id, zh: leak(zh), ko: leak(ko), eu: leak(eu) });
    }
    (packs, i18n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_build_into_packs_with_every_language() {
        let (packs, i18n) = build();
        assert_eq!(packs.len(), 4);
        let c = packs.iter().find(|p| p.id == "countries").unwrap();
        assert!(c.entries.contains("にほん\t日本\n"));
        let s = packs.iter().find(|p| p.id == "stations").unwrap();
        assert!(s.entries.contains("しぶやえき\t渋谷駅"));
        let ci = i18n.iter().find(|x| x.id == "cities").unwrap();
        assert!(ci.zh.contains("dongjing\t东京\t東京") && ci.ko.contains("도쿄"));
        assert!(ci.eu.contains("en\tnewyork\tNewYork") || ci.eu.contains("en\ttokyo\tTokyo"));
        assert!(ci.eu.contains("it\tnewyork\tNew York") || ci.eu.contains("it\tnewyork\t"));
        let f = i18n.iter().find(|x| x.id == "flowers").unwrap();
        assert!(f.zh.contains("yinghua\t樱花\t櫻花"));
    }
}
