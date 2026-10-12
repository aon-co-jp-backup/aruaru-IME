//! 世界の主要な約130言語の一覧(BCP 47 コード, 日本語名, その言語での名称)。出典: realdata.pro の languages.rs。
//! 入力エンジンがある言語は `engine()` で `Lang` に対応づけ、無いものは「準備中」として一覧にだけ出す。

use crate::translit::Lang;

pub const LANGUAGES: &[(&str, &str, &str)] = &[
    ("ja", "日本語", "日本語"),
    ("en", "英語", "English"),
    ("zh-Hans", "中国語(簡体字)", "简体中文"),
    ("zh-Hant", "中国語(繁体字)", "繁體中文"),
    ("ko", "韓国語", "한국어"),
    ("es", "スペイン語", "Español"),
    ("pt-BR", "ポルトガル語(ブラジル)", "Português (Brasil)"),
    ("pt-PT", "ポルトガル語(ポルトガル)", "Português (Portugal)"),
    ("fr", "フランス語", "Français"),
    ("de", "ドイツ語", "Deutsch"),
    ("it", "イタリア語", "Italiano"),
    ("ru", "ロシア語", "Русский"),
    ("uk", "ウクライナ語", "Українська"),
    ("ar", "アラビア語", "العربية"),
    ("he", "ヘブライ語", "עברית"),
    ("fa", "ペルシア語", "فارسی"),
    ("tr", "トルコ語", "Türkçe"),
    ("hi", "ヒンディー語", "हिन्दी"),
    ("bn", "ベンガル語", "বাংলা"),
    ("ur", "ウルドゥー語", "اردو"),
    ("pa", "パンジャブ語", "ਪੰਜਾਬੀ"),
    ("gu", "グジャラート語", "ગુજરાતી"),
    ("mr", "マラーティー語", "मराठी"),
    ("ta", "タミル語", "தமிழ்"),
    ("te", "テルグ語", "తెలుగు"),
    ("kn", "カンナダ語", "ಕನ್ನಡ"),
    ("ml", "マラヤーラム語", "മലയാളം"),
    ("or", "オディア語", "ଓଡ଼ିଆ"),
    ("as", "アッサム語", "অসমীয়া"),
    ("ne", "ネパール語", "नेपाली"),
    ("si", "シンハラ語", "සිංහල"),
    ("th", "タイ語", "ไทย"),
    ("lo", "ラオ語", "ລາວ"),
    ("km", "クメール語", "ខ្មែរ"),
    ("my", "ミャンマー語", "မြန်မာ"),
    ("vi", "ベトナム語", "Tiếng Việt"),
    ("id", "インドネシア語", "Bahasa Indonesia"),
    ("ms", "マレー語", "Bahasa Melayu"),
    ("fil", "フィリピン語", "Filipino"),
    ("jv", "ジャワ語", "Basa Jawa"),
    ("su", "スンダ語", "Basa Sunda"),
    ("mn", "モンゴル語", "Монгол"),
    ("kk", "カザフ語", "Қазақ"),
    ("uz", "ウズベク語", "Oʻzbek"),
    ("ky", "キルギス語", "Кыргызча"),
    ("tg", "タジク語", "Тоҷикӣ"),
    ("tk", "トルクメン語", "Türkmen"),
    ("az", "アゼルバイジャン語", "Azərbaycan"),
    ("hy", "アルメニア語", "Հայերեն"),
    ("ka", "ジョージア語", "ქართული"),
    ("ps", "パシュトー語", "پښتو"),
    ("ku", "クルド語", "Kurdî"),
    ("ckb", "中央クルド語(ソラニー)", "کوردیی ناوەندی"),
    ("dv", "ディベヒ語", "ދިވެހި"),
    ("bo", "チベット語", "བོད་ཡིག"),
    ("ug", "ウイグル語", "ئۇيغۇرچە"),
    ("nl", "オランダ語", "Nederlands"),
    ("sv", "スウェーデン語", "Svenska"),
    ("no", "ノルウェー語", "Norsk"),
    ("da", "デンマーク語", "Dansk"),
    ("fi", "フィンランド語", "Suomi"),
    ("is", "アイスランド語", "Íslenska"),
    ("fo", "フェロー語", "Føroyskt"),
    ("et", "エストニア語", "Eesti"),
    ("lv", "ラトビア語", "Latviešu"),
    ("lt", "リトアニア語", "Lietuvių"),
    ("pl", "ポーランド語", "Polski"),
    ("cs", "チェコ語", "Čeština"),
    ("sk", "スロバキア語", "Slovenčina"),
    ("hu", "ハンガリー語", "Magyar"),
    ("ro", "ルーマニア語", "Română"),
    ("bg", "ブルガリア語", "Български"),
    ("sr", "セルビア語", "Српски"),
    ("hr", "クロアチア語", "Hrvatski"),
    ("bs", "ボスニア語", "Bosanski"),
    ("sl", "スロベニア語", "Slovenščina"),
    ("mk", "マケドニア語", "Македонски"),
    ("sq", "アルバニア語", "Shqip"),
    ("el", "ギリシャ語", "Ελληνικά"),
    ("mt", "マルタ語", "Malti"),
    ("be", "ベラルーシ語", "Беларуская"),
    ("ca", "カタルーニャ語", "Català"),
    ("eu", "バスク語", "Euskara"),
    ("gl", "ガリシア語", "Galego"),
    ("ga", "アイルランド語", "Gaeilge"),
    ("gd", "スコットランド・ゲール語", "Gàidhlig"),
    ("cy", "ウェールズ語", "Cymraeg"),
    ("br", "ブルトン語", "Brezhoneg"),
    ("lb", "ルクセンブルク語", "Lëtzebuergesch"),
    ("fy", "西フリジア語", "Frysk"),
    ("rm", "ロマンシュ語", "Rumantsch"),
    ("co", "コルシカ語", "Corsu"),
    ("eo", "エスペラント", "Esperanto"),
    ("la", "ラテン語", "Latina"),
    ("yi", "イディッシュ語", "ייִדיש"),
    ("sw", "スワヒリ語", "Kiswahili"),
    ("am", "アムハラ語", "አማርኛ"),
    ("ti", "ティグリニャ語", "ትግርኛ"),
    ("om", "オロモ語", "Afaan Oromoo"),
    ("so", "ソマリ語", "Soomaali"),
    ("ha", "ハウサ語", "Hausa"),
    ("yo", "ヨルバ語", "Yorùbá"),
    ("ig", "イボ語", "Igbo"),
    ("ff", "フラ語", "Fulfulde"),
    ("wo", "ウォロフ語", "Wolof"),
    ("ak", "アカン語", "Akan"),
    ("ee", "エウェ語", "Eʋegbe"),
    ("ln", "リンガラ語", "Lingála"),
    ("kg", "コンゴ語", "Kikongo"),
    ("rw", "キニヤルワンダ語", "Kinyarwanda"),
    ("rn", "キルンディ語", "Ikirundi"),
    ("lg", "ガンダ語", "Luganda"),
    ("mg", "マダガスカル語", "Malagasy"),
    ("ny", "チェワ語", "Chichewa"),
    ("sn", "ショナ語", "chiShona"),
    ("zu", "ズールー語", "isiZulu"),
    ("xh", "コサ語", "isiXhosa"),
    ("af", "アフリカーンス語", "Afrikaans"),
    ("st", "ソト語", "Sesotho"),
    ("tn", "ツワナ語", "Setswana"),
    ("ht", "ハイチ・クレオール語", "Kreyòl ayisyen"),
    ("qu", "ケチュア語", "Runasimi"),
    ("ay", "アイマラ語", "Aymar aru"),
    ("gn", "グアラニー語", "Avañeʼẽ"),
    ("mi", "マオリ語", "Te Reo Māori"),
    ("sm", "サモア語", "Gagana Sāmoa"),
    ("to", "トンガ語", "Lea faka-Tonga"),
    ("fj", "フィジー語", "Na Vosa Vakaviti"),
    ("haw", "ハワイ語", "ʻŌlelo Hawaiʻi"),
    ("tpi", "トク・ピシン", "Tok Pisin"),
    ("tet", "テトゥン語", "Tetun"),
    ("dz", "ゾンカ語", "རྫོང་ཁ"),
];

/// この言語の入力エンジン(無ければ None=準備中)。
pub fn engine(code: &str) -> Option<Lang> {
    let l = match code {
        "ja" => Lang::Ja,
        "en" => Lang::En,
        "ru" => Lang::Ru,
        "uk" => Lang::Uk,
        "ar" => Lang::Ar,
        "fa" => Lang::Fa,
        "he" => Lang::He,
        "ko" => Lang::Ko,
        "zh-Hans" => Lang::ZhHans,
        "zh-Hant" => Lang::ZhHant,
        "es" => Lang::Es,
        "fr" => Lang::Fr,
        "de" => Lang::De,
        "it" => Lang::It,
        "pt-BR" | "pt-PT" => Lang::Pt,
        _ => return None,
    };
    Some(l)
}

/// 専用エンジンは無いが、同じ文字体系の言語の入力方式で近似できる言語(キリル文字→ロシア語、アラビア文字→アラビア語、
/// ラテン文字→アクセント記号つき入力)。アクセント記号つき入力は ' ` ^ " ~ : / , の組み合わせで á à â ä ã ü ç などを打つ。
pub fn approx(code: &str) -> Option<Lang> {
    const CYRILLIC: &[&str] = &["mn", "kk", "ky", "tg", "bg", "sr", "mk", "be"];
    const ARABIC: &[&str] = &["ur", "ps", "ckb", "ku", "ug"];
    const HEBREW: &[&str] = &["yi"];
    const LATIN: &[&str] = &[
        "tr", "vi", "id", "ms", "fil", "jv", "su", "uz", "tk", "az", "nl", "sv", "no", "da", "fi", "is", "fo", "et", "lv", "lt", "pl", "cs", "sk", "hu", "ro", "hr", "bs",
        "sl", "sq", "mt", "ca", "eu", "gl", "ga", "gd", "cy", "br", "lb", "fy", "rm", "co", "eo", "la", "sw", "om", "so", "ha", "yo", "ig", "ff", "wo", "ak", "ee", "ln",
        "kg", "rw", "rn", "lg", "mg", "ny", "sn", "zu", "xh", "af", "st", "tn", "ht", "qu", "ay", "gn", "mi", "sm", "to", "fj", "haw", "tpi", "tet",
    ];
    if CYRILLIC.contains(&code) {
        Some(Lang::Ru)
    } else if ARABIC.contains(&code) {
        Some(Lang::Ar)
    } else if HEBREW.contains(&code) {
        Some(Lang::He)
    } else if LATIN.contains(&code) {
        Some(Lang::Es)
    } else {
        None
    }
}

/// 入力に使う入力方式(専用エンジン、なければ近似)。
pub fn base(code: &str) -> Option<Lang> {
    engine(code).or_else(|| approx(code))
}

/// 入力の状態: 専用エンジン / 近似 / 準備中。
pub fn status(code: &str) -> &'static str {
    if engine(code).is_some() {
        "対応 / ready"
    } else if approx(code).is_some() {
        "近似 / approximate"
    } else {
        "準備中 / planned"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_is_complete_and_engines_map() {
        assert!(LANGUAGES.len() >= 120, "{}", LANGUAGES.len());
        assert_eq!(engine("ko"), Some(Lang::Ko));
        assert_eq!(engine("sw"), None);
        assert_eq!(base("bg"), Some(Lang::Ru));
        assert_eq!(base("ur"), Some(Lang::Ar));
        assert_eq!(base("pl"), Some(Lang::Es));
        assert_eq!(base("hi"), None);
        let planned = LANGUAGES.iter().filter(|l| base(l.0).is_none()).count();
        assert!(planned < 40, "{planned}");
        assert!(LANGUAGES.iter().any(|l| l.0 == "sw"));
    }
}
