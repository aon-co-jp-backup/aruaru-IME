//! 用語辞書パック(`packs.rs`)の、中国語(簡体字・繁体字=台湾)・韓国語・ヨーロッパ言語の版。
//! パックを入れると、各言語の入力でも、その作品の語が候補に出る。
//!  - 中国語: `<データ領域>/zh-extra/<パック>.tsv`(`ピンイン<TAB>簡体<TAB>繁体`。声調なし、ü は v)。ピンイン辞書の読み込み時に合流する。
//!  - 韓国語: `<データ領域>/words/ko.<パック>.tsv`(`2ボル式のキー<TAB>ハングル`。キーはハングルから自動で作る)。
//!  - ヨーロッパ言語: `<データ領域>/words/<言語>.<パック>.tsv`(`キー<TAB>語`)。
//! **主要な語であり、網羅ではない**(公式の訳が定まっていない語は入れていない)。

use crate::translit::hangul_to_keys;
use std::path::Path;

pub struct I18n {
    pub id: &'static str,
    /// `ピンイン\t簡体\t繁体`
    pub zh: &'static str,
    /// ハングル(スペースを含んでよい)。キーは自動で作る。
    pub ko: &'static str,
    /// `言語\tキー\t語`(言語は es fr de it pt)。
    pub eu: &'static str,
}

pub const I18N: &[I18n] = &[
    I18n {
        id: "kimetsu",
        zh: "guimiezhiren\t鬼灭之刃\t鬼滅之刃\nzaomentanzhilang\t灶门炭治郎\t竈門炭治郎\ntanzhilang\t炭治郎\t炭治郎\nnidouzi\t祢豆子\t禰豆子\nwoqishanyi\t我妻善逸\t我妻善逸\n\
zuipingyizhizhu\t嘴平伊之助\t嘴平伊之助\nfuganyiyong\t富冈义勇\t富岡義勇\nhudiexin\t胡蝶忍\t胡蝶忍\nlianyuxingshoulang\t炼狱杏寿郎\t煉獄杏壽郎\nguishadui\t鬼杀队\t鬼殺隊\n\
guiwuzhiwucan\t鬼舞辻无惨\t鬼舞辻無慘\nrilundao\t日轮刀\t日輪刀\nwuxianlieche\t无限列车\t無限列車\nquanjizhong\t全集中\t全集中\n",
        ko: "귀멸의 칼날\n탄지로\n네즈코\n젠이츠\n이노스케\n기유\n시노부\n렌고쿠\n귀살대\n무잔\n",
        eu: "es\tkimetsu\tKimetsu no Yaiba\nes\tguardianes\tGuardianes de la Noche\nfr\tkimetsu\tKimetsu no Yaiba\nde\tkimetsu\tKimetsu no Yaiba\nit\tkimetsu\tKimetsu no Yaiba\npt\tkimetsu\tKimetsu no Yaiba\n",
    },
    I18n {
        id: "onepiece",
        zh: "haizeiwang\t海贼王\t海賊王\nlufei\t路飞\t路飛\nsuolong\t索隆\t索隆\nnamei\t娜美\t娜美\nwusuopu\t乌索普\t烏索普\nshanzhi\t山治\t山治\nqiaoba\t乔巴\t喬巴\n\
luobin\t罗宾\t羅賓\ncaomao\t草帽\t草帽\ncaomaoyihuo\t草帽一伙\t草帽一夥\nemoguoshi\t恶魔果实\t惡魔果實\nhaijun\t海军\t海軍\nsihuang\t四皇\t四皇\nqiwuhai\t七武海\t七武海\n",
        ko: "원피스\n루피\n조로\n나미\n우솝\n상디\n쵸파\n로빈\n프랑키\n브룩\n해적왕\n악마의 열매\n",
        eu: "es\tonepiece\tOne Piece\nfr\tonepiece\tOne Piece\nde\tonepiece\tOne Piece\nit\tonepiece\tOne Piece\npt\tonepiece\tOne Piece\nes\tpiratas\tPiratas del Sombrero de Paja\n",
    },
    I18n {
        id: "dragonball",
        zh: "longzhu\t龙珠\t龍珠\nsunwukong\t孙悟空\t孫悟空\nwukong\t悟空\t悟空\nsunwufan\t孙悟饭\t孫悟飯\nbeijita\t贝吉塔\t貝吉塔\nbuerma\t布尔玛\t布爾瑪\nbuma\t布玛\t布瑪\n\
kelin\t克林\t克林\nbike\t比克\t比克\nfulisa\t弗利萨\t弗利沙\nfulisha\t弗利萨\t弗利沙\nsaiyaren\t赛亚人\t賽亞人\nchaojisaiyaren\t超级赛亚人\t超級賽亞人\n\
guipaiqigong\t龟派气功\t龜派氣功\nyuanqidan\t元气弹\t元氣彈\nshenlong\t神龙\t神龍\njindouyun\t筋斗云\t筋斗雲\nniaoshanming\t鸟山明\t鳥山明\n",
        ko: "드래곤볼\n손오공\n오공\n손오반\n베지터\n부르마\n크리링\n피콜로\n프리저\n트랭크스\n슈퍼 사이어인\n",
        eu: "es\tdragonball\tDragon Ball\nfr\tdragonball\tDragon Ball\nde\tdragonball\tDragon Ball\nit\tdragonball\tDragon Ball\npt\tdragonball\tDragon Ball\n",
    },
    I18n {
        id: "sailormoon",
        zh: "meishaonvzhanshi\t美少女战士\t美少女戰士\nyueyetu\t月野兔\t月野兔\nshuiyeyamei\t水野亚美\t水野亞美\nhuoyeli\t火野丽\t火野麗\nmuyezhenqin\t木野真琴\t木野真琴\n\
aiyemeinaizi\t爱野美奈子\t愛野美奈子\nyelifujiamian\t夜礼服假面\t夜禮服假面\n",
        ko: "세일러문\n미소녀 전사 세일러문\n턱시도 가면\n세일러 머큐리\n세일러 마즈\n세일러 주피터\n세일러 비너스\n",
        eu: "es\tsailormoon\tSailor Moon\nfr\tsailormoon\tSailor Moon\nde\tsailormoon\tSailor Moon\nit\tsailormoon\tSailor Moon\npt\tsailormoon\tSailor Moon\n",
    },
    I18n {
        id: "grendizer",
        zh: "",
        ko: "그랜다이저\nUFO 로봇 그랜다이저\n",
        eu: "fr\tgoldorak\tGoldorak\nfr\tufo\tUFO Robot Goldorak\nfr\tactarus\tActarus\nfr\talcor\tAlcor\nit\tgoldrake\tGoldrake\nit\tufo\tUFO Robot Goldrake\nit\tactarus\tActarus\nes\tgoldorak\tGoldorak\nes\tgrendizer\tGrendizer\nde\tgrendizer\tUFO-Robot Grendizer\npt\tgoldorak\tGoldorak\n",
    },
    I18n {
        id: "mazinger",
        zh: "wudijintiegang\t无敌铁金刚\t無敵鐵金剛\n",
        ko: "마징가 Z\n마징가\n",
        eu: "it\tmazinga\tMazinga Z\nes\tmazinger\tMazinger Z\nfr\tmazinger\tMazinger Z\nde\tmazinger\tMazinger Z\npt\tmazinger\tMazinger Z\n",
    },
    I18n {
        id: "greatmazinger",
        zh: "",
        ko: "그레이트 마징가\n",
        eu: "it\tgrande\tIl Grande Mazinga\nes\tgreat\tGreat Mazinger\nfr\tgreat\tGreat Mazinger\n",
    },
    I18n {
        id: "gaiking",
        zh: "",
        ko: "가이킹\n",
        eu: "it\tgaiking\tGaiking\nes\tgaiking\tGaiking\nfr\tgaiking\tGaiking\n",
    },
    I18n {
        id: "jeeg",
        zh: "",
        ko: "",
        eu: "it\tjeeg\tJeeg Robot d'acciaio\nes\tjeeg\tJeeg\nfr\tjeeg\tJeeg\nde\tjeeg\tJeeg\n",
    },
    I18n {
        id: "hunterxhunter",
        zh: "quanzhiliehren\t全职猎人\t全職獵人\nxiaojie\t小杰\t小傑\nqike\t奇犽\t奇犽\nkulapika\t酷拉皮卡\t酷拉皮卡\nleiouli\t雷欧力\t雷歐力\nxisuo\t西索\t西索\n\
kuluoluo\t库洛洛\t庫洛洛\nhuanyinglvtuan\t幻影旅团\t幻影旅團\nnian\t念\t念\n",
        ko: "헌터×헌터\n곤\n키르아\n쿠라피카\n레오리오\n히소카\n크로로\n환영여단\n헌터 시험\n",
        eu: "es\thunter\tHunter × Hunter\nfr\thunter\tHunter × Hunter\nde\thunter\tHunter × Hunter\nit\thunter\tHunter × Hunter\npt\thunter\tHunter × Hunter\n",
    },
    I18n {
        id: "sazaneyes",
        zh: "sanzhiyan\t三只眼\t三隻眼\n",
        ko: "3×3 아이즈\n",
        eu: "es\t3x3eyes\t3×3 Eyes\nfr\t3x3eyes\t3×3 Eyes\nde\t3x3eyes\t3×3 Eyes\nit\t3x3eyes\t3×3 Eyes\npt\t3x3eyes\t3×3 Eyes\n",
    },
    I18n {
        id: "hannya",
        zh: "boruoboluomiduoxinjing\t般若波罗蜜多心经\t般若波羅蜜多心經\nxinjing\t心经\t心經\nboruo\t般若\t般若\nguanzizaipusa\t观自在菩萨\t觀自在菩薩\nsejishikong\t色即是空\t色即是空\n",
        ko: "반야심경\n반야바라밀다심경\n관자재보살\n색즉시공\n공즉시색\n",
        eu: "es\tcorazon\tSutra del Corazón\nfr\tcoeur\tSûtra du Cœur\nde\therz\tHerz-Sutra\nit\tcuore\tSutra del Cuore\npt\tcoracao\tSutra do Coração\n",
    },
];

pub fn find(id: &str) -> Option<&'static I18n> {
    I18N.iter().find(|x| x.id == id).or_else(|| crate::packs::table_i18n().iter().find(|x| x.id == id))
}

/// 各言語のファイルを書く。書いたファイル数を返す。
pub fn install(root: &Path, id: &str) -> usize {
    let Some(x) = find(id) else { return 0 };
    let mut n = 0;
    if !x.zh.is_empty() {
        let d = root.join("zh-extra");
        if std::fs::create_dir_all(&d).is_ok() {
            let t: String = x.zh.lines().map(|l| format!("{l}\t5000\n")).collect();
            n += usize::from(std::fs::write(d.join(format!("{id}.tsv")), t).is_ok());
        }
    }
    let words = root.join("words");
    if !x.ko.is_empty() && std::fs::create_dir_all(&words).is_ok() {
        let t: String = x
            .ko
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| format!("{}\t{}\n", hangul_to_keys(l).to_lowercase(), l))
            .collect();
        n += usize::from(std::fs::write(words.join(format!("ko.{id}.tsv")), t).is_ok());
    }
    for lang in ["en", "es", "fr", "de", "it", "pt"] {
        let t: String = x.eu.lines().filter_map(|l| l.strip_prefix(&format!("{lang}\t"))).map(|l| format!("{l}\n")).collect();
        if !t.is_empty() && std::fs::create_dir_all(&words).is_ok() {
            n += usize::from(std::fs::write(words.join(format!("{lang}.{id}.tsv")), t).is_ok());
        }
    }
    n
}

pub fn uninstall(root: &Path, id: &str) {
    let _ = std::fs::remove_file(root.join("zh-extra").join(format!("{id}.tsv")));
    for lang in ["ko", "en", "es", "fr", "de", "it", "pt"] {
        let _ = std::fs::remove_file(root.join("words").join(format!("{lang}.{id}.tsv")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pack_has_localized_entries_and_files_roundtrip() {
        for p in crate::packs::all() {
            assert!(find(p.id).is_some(), "{}: 多言語の版が無い", p.id);
        }
        let d = std::env::temp_dir().join(format!("aruaru-i18n-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert!(install(&d, "dragonball") >= 4);
        let ko = std::fs::read_to_string(d.join("words/ko.dragonball.tsv")).unwrap();
        assert!(ko.contains("드래곤볼"), "{ko}");
        let it = std::fs::read_to_string(d.join("words/it.dragonball.tsv")).unwrap();
        assert!(it.contains("Dragon Ball"));
        let zh = std::fs::read_to_string(d.join("zh-extra/dragonball.tsv")).unwrap();
        assert!(zh.contains("longzhu\t龙珠\t龍珠\t5000"));
        uninstall(&d, "dragonball");
        assert!(!d.join("words/ko.dragonball.tsv").exists() && !d.join("zh-extra/dragonball.tsv").exists());
    }

    #[test]
    fn korean_keys_follow_the_two_set_layout() {
        // 원피스 = dnjsvltm
        assert_eq!(hangul_to_keys("원피스"), "dnjsvltm");
        assert_eq!(hangul_to_keys("드래곤볼"), "emforhsqhf");
    }
}
