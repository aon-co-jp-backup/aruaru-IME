//! 多言語入力: ラテン文字の入力を、各言語の文字に変換する(ロシア語・ウクライナ語・アラビア語・ペルシア語・ヘブライ語・
//! 韓国語・ヨーロッパ言語のアクセント記号)。右から左へ書く言語(アラビア語・ペルシア語・ヘブライ語)は `Lang::rtl()` で判別する。
//!
//! アラビア語・ペルシア語・ヘブライ語は短母音を書かないため、ラテン文字からの変換は一意に決まらない。
//! 規則による変換を先頭に出し、よく使う語の辞書(`words/<言語>.tsv` で利用者が追加可能)を候補に並べる。

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Ja,
    En,
    Ru,
    Uk,
    Ar,
    Fa,
    He,
    Ko,
    ZhHans,
    ZhHant,
    Es,
    Fr,
    De,
    It,
    Pt,
    Hi,
    Mr,
    Ne,
    Bn,
    As,
    Pa,
    Gu,
    Or,
    Ta,
    Te,
    Kn,
    Ml,
    El,
    Ka,
    Hy,
    Th,
    Lo,
    Km,
    My,
    Si,
    Am,
    Ti,
    Bo,
    Dz,
    Dv,
}

pub const ALL: &[Lang] = &[
    Lang::Ja, Lang::En, Lang::Ru, Lang::Uk, Lang::Ar, Lang::Fa, Lang::He, Lang::Ko, Lang::ZhHans, Lang::ZhHant, Lang::Es, Lang::Fr, Lang::De, Lang::It, Lang::Pt,
    Lang::Hi, Lang::Mr, Lang::Ne, Lang::Bn, Lang::As, Lang::Pa, Lang::Gu, Lang::Or, Lang::Ta, Lang::Te, Lang::Kn, Lang::Ml, Lang::El, Lang::Ka, Lang::Hy,
    Lang::Th, Lang::Lo, Lang::Km, Lang::My, Lang::Si, Lang::Am, Lang::Ti, Lang::Bo, Lang::Dz, Lang::Dv,
];

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ja => "ja",
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Uk => "uk",
            Lang::Ar => "ar",
            Lang::Fa => "fa",
            Lang::He => "he",
            Lang::Ko => "ko",
            Lang::ZhHans => "zh-Hans",
            Lang::ZhHant => "zh-Hant",
            Lang::Es => "es",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::It => "it",
            Lang::Pt => "pt",
            Lang::Hi => "hi",
            Lang::Mr => "mr",
            Lang::Ne => "ne",
            Lang::Bn => "bn",
            Lang::As => "as",
            Lang::Pa => "pa",
            Lang::Gu => "gu",
            Lang::Or => "or",
            Lang::Ta => "ta",
            Lang::Te => "te",
            Lang::Kn => "kn",
            Lang::Ml => "ml",
            Lang::El => "el",
            Lang::Ka => "ka",
            Lang::Hy => "hy",
            Lang::Th => "th",
            Lang::Lo => "lo",
            Lang::Km => "km",
            Lang::My => "my",
            Lang::Si => "si",
            Lang::Am => "am",
            Lang::Ti => "ti",
            Lang::Bo => "bo",
            Lang::Dz => "dz",
            Lang::Dv => "dv",
        }
    }

    pub fn from_code(c: &str) -> Option<Lang> {
        ALL.iter().copied().find(|l| l.code().eq_ignore_ascii_case(c))
    }

    /// 自分の言語での名前(メニュー表示用)。
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Ja => "日本語",
            Lang::En => "English",
            Lang::Ru => "Русский",
            Lang::Uk => "Українська",
            Lang::Ar => "العربية",
            Lang::Fa => "فارسی",
            Lang::He => "עברית",
            Lang::Ko => "한국어",
            Lang::ZhHans => "中文(简体)",
            Lang::ZhHant => "中文(繁體)",
            Lang::Es => "Español",
            Lang::Fr => "Français",
            Lang::De => "Deutsch",
            Lang::It => "Italiano",
            Lang::Pt => "Português",
            Lang::Hi => "हिन्दी",
            Lang::Mr => "मराठी",
            Lang::Ne => "नेपाली",
            Lang::Bn => "বাংলা",
            Lang::As => "অসমীয়া",
            Lang::Pa => "ਪੰਜਾਬੀ",
            Lang::Gu => "ગુજરાતી",
            Lang::Or => "ଓଡ଼ିଆ",
            Lang::Ta => "தமிழ்",
            Lang::Te => "తెలుగు",
            Lang::Kn => "ಕನ್ನಡ",
            Lang::Ml => "മലയാളം",
            Lang::El => "Ελληνικά",
            Lang::Ka => "ქართული",
            Lang::Hy => "Հայերեն",
            Lang::Th => "ไทย",
            Lang::Lo => "ລາວ",
            Lang::Km => "ខ្មែរ",
            Lang::My => "မြန်မာ",
            Lang::Si => "සිංහල",
            Lang::Am => "አማርኛ",
            Lang::Ti => "ትግርኛ",
            Lang::Bo => "བོད་ཡིག",
            Lang::Dz => "རྫོང་ཁ",
            Lang::Dv => "ދިވެހި",
        }
    }

    /// 英語での名前(メニューの併記用)。
    pub fn english_name(self) -> &'static str {
        match self {
            Lang::Ja => "Japanese",
            Lang::En => "English",
            Lang::Ru => "Russian",
            Lang::Uk => "Ukrainian",
            Lang::Ar => "Arabic",
            Lang::Fa => "Persian (Farsi)",
            Lang::He => "Hebrew",
            Lang::Ko => "Korean",
            Lang::ZhHans => "Chinese (Simplified)",
            Lang::ZhHant => "Chinese (Traditional)",
            Lang::Es => "Spanish",
            Lang::Fr => "French",
            Lang::De => "German",
            Lang::It => "Italian",
            Lang::Pt => "Portuguese",
            Lang::Hi => "Hindi",
            Lang::Mr => "Marathi",
            Lang::Ne => "Nepali",
            Lang::Bn => "Bengali",
            Lang::As => "Assamese",
            Lang::Pa => "Punjabi",
            Lang::Gu => "Gujarati",
            Lang::Or => "Odia",
            Lang::Ta => "Tamil",
            Lang::Te => "Telugu",
            Lang::Kn => "Kannada",
            Lang::Ml => "Malayalam",
            Lang::El => "Greek",
            Lang::Ka => "Georgian",
            Lang::Hy => "Armenian",
            Lang::Th => "Thai",
            Lang::Lo => "Lao",
            Lang::Km => "Khmer",
            Lang::My => "Burmese",
            Lang::Si => "Sinhala",
            Lang::Am => "Amharic",
            Lang::Ti => "Tigrinya",
            Lang::Bo => "Tibetan",
            Lang::Dz => "Dzongkha",
            Lang::Dv => "Dhivehi",
        }
    }

    /// この IME が入力に対応している言語か(中国語はピンイン辞書 zh.tsv を使う)。
    pub fn supported(self) -> bool {
        true
    }

    /// 右から左へ書く言語か。
    pub fn rtl(self) -> bool {
        matches!(self, Lang::Ar | Lang::Fa | Lang::He | Lang::Dv)
    }

    /// Windows の言語ID(LANGID)。
    pub fn langid(self) -> u16 {
        match self {
            Lang::Ja => 0x0411,
            Lang::En => 0x0409,
            Lang::Ru => 0x0419,
            Lang::Uk => 0x0422,
            Lang::Ar => 0x0401,
            Lang::Fa => 0x0429,
            Lang::He => 0x040D,
            Lang::Ko => 0x0412,
            Lang::ZhHans => 0x0804,
            Lang::ZhHant => 0x0404,
            Lang::Es => 0x0C0A,
            Lang::Fr => 0x040C,
            Lang::De => 0x0407,
            Lang::It => 0x0410,
            Lang::Pt => 0x0416,
            Lang::Hi => 0x0439,
            Lang::Mr => 0x044E,
            Lang::Ne => 0x0461,
            Lang::Bn => 0x0445,
            Lang::As => 0x044D,
            Lang::Pa => 0x0446,
            Lang::Gu => 0x0447,
            Lang::Or => 0x0448,
            Lang::Ta => 0x0449,
            Lang::Te => 0x044A,
            Lang::Kn => 0x044B,
            Lang::Ml => 0x044C,
            Lang::El => 0x0408,
            Lang::Ka => 0x0437,
            Lang::Hy => 0x042B,
            Lang::Th => 0x041E,
            Lang::Lo => 0x0454,
            Lang::Km => 0x0453,
            Lang::My => 0x0455,
            Lang::Si => 0x045B,
            Lang::Am => 0x045E,
            Lang::Ti => 0x0473,
            Lang::Bo => 0x0451,
            Lang::Dz => 0x0C51,
            Lang::Dv => 0x0465,
        }
    }

    /// 言語ごとの入力方式(プロファイル)を区別する GUID。日本語は従来のものと同じ。
    pub fn profile_guid(self) -> u128 {
        const JA: u128 = 0xC1D3E5F7_2A4B_4C6D_8E90_1F2A3B4C5D6E;
        if self == Lang::Ja {
            JA
        } else {
            (JA & !0xFFFF) | (0xA000 + ALL.iter().position(|l| *l == self).unwrap_or(0) as u128)
        }
    }

    pub fn from_profile_guid(g: u128) -> Option<Lang> {
        ALL.iter().copied().find(|l| l.profile_guid() == g)
    }

    /// この言語で、組み立て中の語に含めてよい文字(ラテン文字入力)。
    pub fn accepts(self, c: char, composing: bool) -> bool {
        if c.is_ascii_alphabetic() {
            return true;
        }
        match self {
            // アラビア語・ペルシア語・ヘブライ語の数字入力(2,3,5,6,7,8,9 が文字になる)
            Lang::Ar | Lang::Fa | Lang::He => c.is_ascii_digit() || (composing && c == '\''),
            Lang::Ru | Lang::Uk => c == '\'',
            Lang::Es | Lang::Fr | Lang::De | Lang::It | Lang::Pt => composing && "'`^\"~:/,".contains(c),
            Lang::En => composing && c == '\'',
            // インド系・ギリシャ・ジョージア・アルメニア: ~ ^ . ' _ (鼻音・アクセント・danda など)
            Lang::Hi | Lang::Mr | Lang::Ne | Lang::Bn | Lang::As | Lang::Pa | Lang::Gu | Lang::Or | Lang::Ta | Lang::Te | Lang::Kn | Lang::Ml | Lang::El | Lang::Ka | Lang::Hy | Lang::Th | Lang::Lo | Lang::Km | Lang::My | Lang::Si | Lang::Am | Lang::Ti | Lang::Bo | Lang::Dz | Lang::Dv => composing && "~^.'_".contains(c),
            Lang::Ko | Lang::Ja | Lang::ZhHans | Lang::ZhHant => composing && (c.is_ascii_digit() || c == '\'' || c == '-'),
        }
    }
}

// ---------------------------------------------------------------- 規則による変換

type Table = &'static [(&'static str, &'static str)];

const RU: Table = &[
    ("shch", "щ"), ("sch", "щ"), ("yo", "ё"), ("jo", "ё"), ("zh", "ж"), ("kh", "х"), ("ts", "ц"), ("ch", "ч"), ("sh", "ш"), ("yu", "ю"), ("ju", "ю"),
    ("ya", "я"), ("ja", "я"), ("ye", "е"), ("''", "ъ"), ("'", "ь"),
    ("a", "а"), ("b", "б"), ("v", "в"), ("g", "г"), ("d", "д"), ("e", "е"), ("z", "з"), ("i", "и"), ("j", "й"), ("k", "к"), ("l", "л"), ("m", "м"),
    ("n", "н"), ("o", "о"), ("p", "п"), ("r", "р"), ("s", "с"), ("t", "т"), ("u", "у"), ("f", "ф"), ("h", "х"), ("c", "ц"), ("w", "в"), ("x", "кс"),
    ("q", "к"), ("y", "ы"),
];

const UK: Table = &[
    ("shch", "щ"), ("sch", "щ"), ("zh", "ж"), ("kh", "х"), ("ts", "ц"), ("ch", "ч"), ("sh", "ш"), ("yu", "ю"), ("ju", "ю"), ("ya", "я"), ("ja", "я"),
    ("ye", "є"), ("yi", "ї"), ("gh", "ґ"), ("'", "ь"),
    ("a", "а"), ("b", "б"), ("v", "в"), ("h", "г"), ("g", "ґ"), ("d", "д"), ("e", "е"), ("z", "з"), ("i", "і"), ("j", "й"), ("k", "к"), ("l", "л"),
    ("m", "м"), ("n", "н"), ("o", "о"), ("p", "п"), ("r", "р"), ("s", "с"), ("t", "т"), ("u", "у"), ("f", "ф"), ("c", "ц"), ("w", "в"), ("x", "кс"),
    ("q", "к"), ("y", "и"),
];

// アラビア語(チャットの慣用ローマ字: 3=ع 7=ح 5=خ 6=ط 9=ص 2=ء)。短母音は書かない。
const AR: Table = &[
    ("sh", "ش"), ("kh", "خ"), ("gh", "غ"), ("th", "ث"), ("dh", "ذ"), ("ch", "تش"), ("aa", "ا"), ("ee", "ي"), ("ii", "ي"), ("oo", "و"), ("uu", "و"),
    ("3'", "غ"), ("7'", "خ"), ("6'", "ظ"), ("9'", "ض"), ("2", "ء"), ("3", "ع"), ("5", "خ"), ("6", "ط"), ("7", "ح"), ("8", "غ"), ("9", "ص"),
    ("b", "ب"), ("t", "ت"), ("j", "ج"), ("h", "ه"), ("d", "د"), ("r", "ر"), ("z", "ز"), ("s", "س"), ("f", "ف"), ("q", "ق"), ("k", "ك"), ("l", "ل"),
    ("m", "م"), ("n", "ن"), ("w", "و"), ("y", "ي"), ("g", "غ"), ("p", "ب"), ("v", "ف"), ("x", "كس"), ("c", "ك"),
];

const FA: Table = &[
    ("sh", "ش"), ("kh", "خ"), ("gh", "ق"), ("ch", "چ"), ("zh", "ژ"), ("aa", "آ"), ("ee", "ی"), ("ii", "ی"), ("oo", "و"), ("uu", "و"),
    ("2", "ء"), ("3", "ع"), ("5", "خ"), ("6", "ط"), ("7", "ح"), ("8", "غ"), ("9", "ص"),
    ("b", "ب"), ("p", "پ"), ("t", "ت"), ("j", "ج"), ("h", "ه"), ("d", "د"), ("r", "ر"), ("z", "ز"), ("s", "س"), ("f", "ف"), ("q", "ق"), ("k", "ک"),
    ("g", "گ"), ("l", "ل"), ("m", "م"), ("n", "ن"), ("v", "و"), ("w", "و"), ("y", "ی"), ("x", "خ"), ("c", "ک"),
];

const HE: Table = &[
    ("sh", "ש"), ("ch", "ח"), ("kh", "ח"), ("ts", "צ"), ("tz", "צ"), ("th", "ת"), ("3", "ע"), ("7", "ח"), ("9", "צ"),
    ("b", "ב"), ("g", "ג"), ("d", "ד"), ("h", "ה"), ("v", "ו"), ("w", "ו"), ("z", "ז"), ("t", "ט"), ("y", "י"), ("k", "כ"), ("l", "ל"), ("m", "מ"),
    ("n", "נ"), ("s", "ס"), ("p", "פ"), ("f", "פ"), ("q", "ק"), ("r", "ר"), ("x", "קס"), ("c", "כ"), ("j", "ג"),
];

/// 最長一致で表を引く。表に無い文字(母音など)は `vowel` に従って処理する。
fn apply(table: Table, input: &str, vowel: impl Fn(char, bool) -> Option<&'static str>) -> String {
    let s: Vec<char> = input.to_lowercase().chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < s.len() {
        let mut hit = false;
        for len in (1..=4).rev() {
            if i + len > s.len() {
                continue;
            }
            let key: String = s[i..i + len].iter().collect();
            if let Some((_, v)) = table.iter().find(|(k, _)| *k == key) {
                out.push_str(v);
                i += len;
                hit = true;
                break;
            }
        }
        if hit {
            continue;
        }
        let c = s[i];
        match vowel(c, out.is_empty()) {
            Some(v) => out.push_str(v),
            None if "aeiou".contains(c) => {}
            None => out.push(c),
        }
        i += 1;
    }
    out
}

/// ロシア語: y は母音の直後なら й、それ以外は ы(`ay`→ай、`ty`→ты)。
fn russian_fix_y(s: &str, cyr_y: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if cyr_y.contains(c) && out.chars().last().is_some_and(|p| "аеёиоуыэюя".contains(p)) {
            out.push('й');
        } else {
            out.push(c);
        }
    }
    out
}

fn he_final_forms(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if let Some(last) = chars.last_mut() {
        *last = match *last {
            'כ' => 'ך',
            'מ' => 'ם',
            'נ' => 'ן',
            'פ' => 'ף',
            'צ' => 'ץ',
            c => c,
        };
    }
    chars.into_iter().collect()
}

fn capitalize_like(input: &str, out: String) -> String {
    let first_upper = input.chars().next().is_some_and(|c| c.is_ascii_uppercase());
    let rest_upper = input.chars().skip(1).any(|c| c.is_ascii_uppercase());
    if first_upper && rest_upper && input.chars().count() > 1 {
        return out.to_uppercase();
    }
    if first_upper {
        let mut it = out.chars();
        return it.next().map(|f| f.to_uppercase().collect::<String>() + it.as_str()).unwrap_or_default();
    }
    out
}

// ---------------------------------------------------------------- 韓国語(2벌식)

fn jamo_of(c: char) -> Option<char> {
    Some(match c {
        'q' => 'ㅂ', 'w' => 'ㅈ', 'e' => 'ㄷ', 'r' => 'ㄱ', 't' => 'ㅅ', 'y' => 'ㅛ', 'u' => 'ㅕ', 'i' => 'ㅑ', 'o' => 'ㅐ', 'p' => 'ㅔ',
        'a' => 'ㅁ', 's' => 'ㄴ', 'd' => 'ㅇ', 'f' => 'ㄹ', 'g' => 'ㅎ', 'h' => 'ㅗ', 'j' => 'ㅓ', 'k' => 'ㅏ', 'l' => 'ㅣ',
        'z' => 'ㅋ', 'x' => 'ㅌ', 'c' => 'ㅊ', 'v' => 'ㅍ', 'b' => 'ㅠ', 'n' => 'ㅜ', 'm' => 'ㅡ',
        'Q' => 'ㅃ', 'W' => 'ㅉ', 'E' => 'ㄸ', 'R' => 'ㄲ', 'T' => 'ㅆ', 'O' => 'ㅒ', 'P' => 'ㅖ',
        _ => return None,
    })
}

const CHO: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
const JUNG: &str = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ";
const JONG: &str = " ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ";

fn idx(set: &str, c: char) -> Option<usize> {
    set.chars().position(|x| x == c)
}

fn is_vowel_jamo(c: char) -> bool {
    idx(JUNG, c).is_some()
}

fn combine_vowel(a: char, b: char) -> Option<char> {
    Some(match (a, b) {
        ('ㅗ', 'ㅏ') => 'ㅘ', ('ㅗ', 'ㅐ') => 'ㅙ', ('ㅗ', 'ㅣ') => 'ㅚ', ('ㅜ', 'ㅓ') => 'ㅝ', ('ㅜ', 'ㅔ') => 'ㅞ', ('ㅜ', 'ㅣ') => 'ㅟ', ('ㅡ', 'ㅣ') => 'ㅢ',
        _ => return None,
    })
}

fn combine_final(a: char, b: char) -> Option<char> {
    Some(match (a, b) {
        ('ㄱ', 'ㅅ') => 'ㄳ', ('ㄴ', 'ㅈ') => 'ㄵ', ('ㄴ', 'ㅎ') => 'ㄶ', ('ㄹ', 'ㄱ') => 'ㄺ', ('ㄹ', 'ㅁ') => 'ㄻ', ('ㄹ', 'ㅂ') => 'ㄼ', ('ㄹ', 'ㅅ') => 'ㄽ',
        ('ㄹ', 'ㅌ') => 'ㄾ', ('ㄹ', 'ㅍ') => 'ㄿ', ('ㄹ', 'ㅎ') => 'ㅀ', ('ㅂ', 'ㅅ') => 'ㅄ',
        _ => return None,
    })
}

fn split_final(c: char) -> Option<(char, char)> {
    Some(match c {
        'ㄳ' => ('ㄱ', 'ㅅ'), 'ㄵ' => ('ㄴ', 'ㅈ'), 'ㄶ' => ('ㄴ', 'ㅎ'), 'ㄺ' => ('ㄹ', 'ㄱ'), 'ㄻ' => ('ㄹ', 'ㅁ'), 'ㄼ' => ('ㄹ', 'ㅂ'), 'ㄽ' => ('ㄹ', 'ㅅ'),
        'ㄾ' => ('ㄹ', 'ㅌ'), 'ㄿ' => ('ㄹ', 'ㅍ'), 'ㅀ' => ('ㄹ', 'ㅎ'), 'ㅄ' => ('ㅂ', 'ㅅ'),
        _ => return None,
    })
}

/// 2벌식の自動組み立て(자모 → 音節)。
pub fn hangul_compose(jamos: &[char]) -> String {
    let mut out = String::new();
    let (mut cho, mut jung, mut jong): (Option<char>, Option<char>, Option<char>) = (None, None, None);
    let flush = |out: &mut String, cho: &mut Option<char>, jung: &mut Option<char>, jong: &mut Option<char>| {
        match (*cho, *jung) {
            (Some(c), Some(j)) => {
                let code = 0xAC00 + (idx(CHO, c).unwrap() * 21 + idx(JUNG, j).unwrap()) * 28 + jong.and_then(|f| idx(JONG, f)).unwrap_or(0);
                out.push(char::from_u32(code as u32).unwrap());
            }
            (Some(c), None) => out.push(c),
            (None, Some(j)) => out.push(j),
            _ => {}
        }
        *cho = None;
        *jung = None;
        *jong = None;
    };
    for &c in jamos {
        if is_vowel_jamo(c) {
            match (cho, jung, jong) {
                (Some(_), None, _) => jung = Some(c),
                (Some(_), Some(j), None) => match combine_vowel(j, c) {
                    Some(v) => jung = Some(v),
                    None => {
                        flush(&mut out, &mut cho, &mut jung, &mut jong);
                        jung = Some(c);
                    }
                },
                (Some(_), Some(_), Some(f)) => {
                    // 終声を次の音節の初声にする(안ㅕ → 아녀)。複合終声は後ろ半分だけ移す
                    let (keep, moved) = match split_final(f) {
                        Some((a, b)) => (Some(a), b),
                        None => (None, f),
                    };
                    jong = keep;
                    flush(&mut out, &mut cho, &mut jung, &mut jong);
                    if idx(CHO, moved).is_some() {
                        cho = Some(moved);
                    }
                    jung = Some(c);
                }
                _ => {
                    flush(&mut out, &mut cho, &mut jung, &mut jong);
                    jung = Some(c);
                }
            }
        } else if idx(CHO, c).is_some() || idx(JONG, c).is_some() {
            match (cho, jung, jong) {
                (None, None, _) => cho = Some(c),
                (Some(_), None, _) => {
                    flush(&mut out, &mut cho, &mut jung, &mut jong);
                    cho = Some(c);
                }
                (Some(_), Some(_), None) if idx(JONG, c).is_some() => jong = Some(c),
                (Some(_), Some(_), Some(f)) => match combine_final(f, c) {
                    Some(v) => jong = Some(v),
                    None => {
                        flush(&mut out, &mut cho, &mut jung, &mut jong);
                        cho = Some(c);
                    }
                },
                _ => {
                    flush(&mut out, &mut cho, &mut jung, &mut jong);
                    cho = Some(c);
                }
            }
        } else {
            flush(&mut out, &mut cho, &mut jung, &mut jong);
            out.push(c);
        }
    }
    flush(&mut out, &mut cho, &mut jung, &mut jong);
    out
}

// ---------------------------------------------------------------- ヨーロッパ言語のアクセント

fn accent(base: char, mark: char) -> Option<char> {
    let table: &[(char, char, char)] = &[
        ('\'', 'a', 'á'), ('\'', 'e', 'é'), ('\'', 'i', 'í'), ('\'', 'o', 'ó'), ('\'', 'u', 'ú'), ('\'', 'y', 'ý'), ('\'', 'c', 'ć'), ('\'', 'n', 'ń'), ('\'', 's', 'ś'), ('\'', 'z', 'ź'),
        ('`', 'a', 'à'), ('`', 'e', 'è'), ('`', 'i', 'ì'), ('`', 'o', 'ò'), ('`', 'u', 'ù'),
        ('^', 'a', 'â'), ('^', 'e', 'ê'), ('^', 'i', 'î'), ('^', 'o', 'ô'), ('^', 'u', 'û'),
        ('"', 'a', 'ä'), ('"', 'e', 'ë'), ('"', 'i', 'ï'), ('"', 'o', 'ö'), ('"', 'u', 'ü'), ('"', 'y', 'ÿ'),
        (':', 'a', 'ä'), (':', 'e', 'ë'), (':', 'i', 'ï'), (':', 'o', 'ö'), (':', 'u', 'ü'),
        ('~', 'a', 'ã'), ('~', 'o', 'õ'), ('~', 'n', 'ñ'),
        (',', 'c', 'ç'), ('/', 'o', 'ø'), ('/', 'l', 'ł'), ('/', 'd', 'đ'),
    ];
    let lower = base.to_ascii_lowercase();
    let r = table.iter().find(|(m, b, _)| *m == mark && *b == lower).map(|x| x.2)?;
    Some(if base.is_uppercase() { r.to_uppercase().next().unwrap_or(r) } else { r })
}

/// `e'` → é、`n~` → ñ、`u:` → ü のように、文字の直後の記号でアクセントを付ける。
pub fn compose_accents(input: &str) -> String {
    let mut out: Vec<char> = vec![];
    for c in input.chars() {
        if let Some(&prev) = out.last() {
            if let Some(a) = accent(prev, c) {
                out.pop();
                out.push(a);
                continue;
            }
        }
        out.push(c);
    }
    out.into_iter().collect()
}

/// ドイツ語の ae/oe/ue/ss 表記から ä/ö/ü/ß を作る(すべて置き換えた1通り)。
fn german_umlauts(s: &str) -> String {
    s.replace("ae", "ä").replace("oe", "ö").replace("ue", "ü").replace("ss", "ß").replace("Ae", "Ä").replace("Oe", "Ö").replace("Ue", "Ü")
}

// ---------------------------------------------------------------- 公開API

/// ラテン文字の入力を、その言語の文字にする(規則による変換)。日本語・中国語は別の変換器が扱うので、そのまま返す。
pub fn transliterate(lang: Lang, input: &str) -> String {
    let none = |_c: char, _first: bool| None;
    match lang {
        Lang::Ru => capitalize_like(input, russian_fix_y(&apply(RU, input, none), "ы")),
        Lang::Uk => capitalize_like(input, apply(UK, input, none)),
        Lang::Ar => {
            // 語頭の母音は ا、語中・語末の短母音は書かない。
            capitalize_like(input, apply(AR, input, |c, first| if first && "aeiou".contains(c) { Some("ا") } else { None }))
        }
        Lang::Fa => apply(FA, input, |c, first| if first && "aeiou".contains(c) { Some("ا") } else { None }),
        Lang::He => he_final_forms(&apply(HE, input, |c, first| if first && "aeiou".contains(c) { Some("א") } else { None })),
        Lang::Ko => {
            let jamos: Vec<char> = input.chars().map(|c| jamo_of(c).unwrap_or(c)).collect();
            hangul_compose(&jamos)
        }
        Lang::Es | Lang::Fr | Lang::De | Lang::It | Lang::Pt => compose_accents(input),
        Lang::Hi => crate::indic::transliterate(crate::indic::Script::Deva, input),
        Lang::Mr => crate::indic::transliterate(crate::indic::Script::Deva, input),
        Lang::Ne => crate::indic::transliterate(crate::indic::Script::Deva, input),
        Lang::Bn => crate::indic::transliterate(crate::indic::Script::Beng, input),
        Lang::As => crate::indic::transliterate(crate::indic::Script::Assam, input),
        Lang::Pa => crate::indic::transliterate(crate::indic::Script::Guru, input),
        Lang::Gu => crate::indic::transliterate(crate::indic::Script::Gujr, input),
        Lang::Or => crate::indic::transliterate(crate::indic::Script::Orya, input),
        Lang::Ta => crate::indic::transliterate(crate::indic::Script::Taml, input),
        Lang::Te => crate::indic::transliterate(crate::indic::Script::Telu, input),
        Lang::Kn => crate::indic::transliterate(crate::indic::Script::Knda, input),
        Lang::Ml => crate::indic::transliterate(crate::indic::Script::Mlym, input),
        Lang::El => crate::indic::greek_final_sigma(&apply(crate::indic::GREEK, input, none)),
        Lang::Ka => apply(crate::indic::GEORGIAN, input, none),
        Lang::Hy => apply(crate::indic::ARMENIAN, input, none),
        Lang::Th => crate::abugida::transliterate(crate::abugida::Abu::Thai, input),
        Lang::Lo => crate::abugida::transliterate(crate::abugida::Abu::Lao, input),
        Lang::Km => crate::abugida::transliterate(crate::abugida::Abu::Khmer, input),
        Lang::My => crate::abugida::transliterate(crate::abugida::Abu::Myanmar, input),
        Lang::Si => crate::abugida::transliterate(crate::abugida::Abu::Sinhala, input),
        Lang::Am => crate::abugida::transliterate_ethiopic(input),
        Lang::Ti => crate::abugida::transliterate_ethiopic(input),
        Lang::Bo => crate::abugida::transliterate(crate::abugida::Abu::Tibetan, input),
        Lang::Dz => crate::abugida::transliterate(crate::abugida::Abu::Tibetan, input),
        Lang::Dv => crate::abugida::transliterate(crate::abugida::Abu::Thaana, input),
        Lang::En | Lang::Ja | Lang::ZhHans | Lang::ZhHant => input.to_string(),
    }
}

const RU_WORDS: &str = "privet привет\nspasibo спасибо\npozhaluysta пожалуйста\ndobroe доброе\nutro утро\ndobryy добрый\nden день\nvecher вечер\nzdravstvuyte здравствуйте\ndo до\nsvidaniya свидания\nda да\nnet нет\nya я\nty ты\nmy мы\nvy вы\nrossiya россия\nmoskva москва\nyazyk язык\ndruzya друзья\nlyublyu люблю\nkak как\ndela дела\nkhorosho хорошо\nplokho плохо\nsegodnya сегодня\nzavtra завтра\nsmysl смысл\nrabota работа\ndom дом\nvremya время\nchelovek человек\nzhizn жизнь";
const UK_WORDS: &str = "pryvit привіт\ndyakuyu дякую\nbud'laska будь ласка\nukrayina україна\nkyyiv київ\nlyublyu люблю\nhorilka горілка\ndobryi добрий\nden день\nslava слава\ndruzi друзі\nmova мова\nroboto робота";
const AR_WORDS: &str = "salam سلام\nsalaam سلام\nmarhaba مرحبا\nahlan أهلا\nshukran شكرا\nafwan عفوا\nnaam نعم\nla لا\nkayf كيف\nhalak حالك\nalhamdulillah الحمد لله\ninshallah إن شاء الله\nmasalama مع السلامة\nsabah صباح\nkhayr خير\nsadiq صديق\nhabibi حبيبي\nbayt بيت\nkitab كتاب\nmadrasa مدرسة\nalarabiya العربية\nmisr مصر\nsaudia السعودية";
const FA_WORDS: &str = "salam سلام\nmersi مرسی\nmamnun ممنون\nkhoda خدا\nkhodahafez خداحافظ\nbale بله\nnah نه\nchetori چطوری\nkhub خوب\nbad بد\niran ایران\ntehran تهران\nfarsi فارسی\ndoost دوست\ndustan دوستان\nshab شب\nrooz روز\nsobh صبح\nbekheir بخیر\nkhosh خوش\nomadid آمدید\nashegh عاشق\nbarname برنامه\nketab کتاب\nmadrese مدرسه\nirani ایرانی";
const HE_WORDS: &str = "shalom שלום\ntoda תודה\nbevakasha בבקשה\nken כן\nlo לא\nboker בוקר\ntov טוב\nlayla לילה\nisrael ישראל\nyerushalayim ירושלים\nivrit עברית\nchaver חבר\nahava אהבה\nbayit בית\nsefer ספר";

fn builtin_words(lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => RU_WORDS,
        Lang::Uk => UK_WORDS,
        Lang::Ar => AR_WORDS,
        Lang::Fa => FA_WORDS,
        Lang::He => HE_WORDS,
        _ => "",
    }
}

/// よく使う語の辞書(組み込み+利用者が `<データ領域>/words/<言語>.tsv` に足した分)。キー(ラテン文字) → 語。
pub fn word_list(lang: Lang) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = builtin_words(lang)
        .lines()
        .filter_map(|l| l.split_once(' ').map(|(k, w)| (k.to_string(), w.to_string())))
        .collect();
    // 利用者の語(`<言語>.tsv`)と、用語辞書パックの語(`<言語>.<パック>.tsv`)
    let dir = crate::components::data_dir().join("words");
    let mut files: Vec<std::path::PathBuf> = vec![dir.join(format!("{}.tsv", lang.code()))];
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let prefix = format!("{}.", lang.code());
        let mut extra: Vec<std::path::PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".tsv")))
            .collect();
        extra.sort();
        files.extend(extra);
    }
    for p in files {
        if let Ok(t) = std::fs::read_to_string(p) {
            for l in t.lines() {
                if let Some((k, w)) = l.split_once('\t') {
                    v.insert(0, (k.trim().to_lowercase(), w.trim().to_string()));
                }
            }
        }
    }
    v
}

/// 候補: 規則による変換 → 辞書の語(キーが一致、続いて前方一致)→ ヨーロッパ言語の別表記 → 入力そのまま。
pub fn candidates(lang: Lang, input: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let mut add = |s: String| {
        if !s.is_empty() && !out.contains(&s) {
            out.push(s);
        }
    };
    let rule = transliterate(lang, input);
    let key = input.to_lowercase();
    let words = word_list(lang);
    let exact: Vec<String> = words.iter().filter(|(k, _)| *k == key).map(|(_, w)| capitalize_like(input, w.clone())).collect();
    // 辞書にある語は、規則による変換より前に出す(短母音を書く言語は規則だけでは決まらないため)
    for w in exact {
        add(w);
    }
    add(rule);
    if key.chars().count() >= 2 {
        let mut n = 0;
        for (k, w) in &words {
            if k.starts_with(&key) && *k != key && n < 5 {
                add(w.clone());
                n += 1;
            }
        }
    }
    if lang == Lang::De {
        add(german_umlauts(input));
    }
    add(input.to_string());
    out
}

/// 文章の文字種から、書かれている言語を推定する(キリル文字→ロシア語/ウクライナ語、アラビア文字→アラビア語/ペルシア語、ヘブライ文字、ハングル)。
pub fn detect_script(text: &str) -> Option<Lang> {
    let letters: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    let tail: Vec<char> = letters.iter().rev().take(40).copied().collect();
    let n = tail.len();
    if n == 0 {
        return None;
    }
    let share = |f: &dyn Fn(char) -> bool| tail.iter().filter(|c| f(**c)).count() as f32 / n as f32;
    if share(&|c| ('\u{0400}'..='\u{04FF}').contains(&c)) > 0.5 {
        let uk = tail.iter().any(|c| "іїєґІЇЄҐ".contains(*c));
        return Some(if uk { Lang::Uk } else { Lang::Ru });
    }
    if share(&|c| ('\u{0600}'..='\u{06FF}').contains(&c) || ('\u{0750}'..='\u{077F}').contains(&c)) > 0.5 {
        let fa = tail.iter().any(|c| "پچژگکی".contains(*c));
        return Some(if fa { Lang::Fa } else { Lang::Ar });
    }
    if share(&|c| ('\u{0590}'..='\u{05FF}').contains(&c)) > 0.5 {
        return Some(Lang::He);
    }
    if share(&|c| ('\u{AC00}'..='\u{D7A3}').contains(&c) || ('\u{1100}'..='\u{11FF}').contains(&c)) > 0.5 {
        return Some(Lang::Ko);
    }
    None
}

/// 右から左へ書く文字(アラビア文字・ヘブライ文字)を含むか。候補窓を右から左の表示にするかの判定に使う。
pub fn is_rtl_text(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '\u{0590}'..='\u{08FF}' | '\u{FB1D}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}'))
}

pub fn lookup_table() -> HashMap<&'static str, Lang> {
    ALL.iter().map(|l| (l.code(), *l)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn russian() {
        assert_eq!(transliterate(Lang::Ru, "privet"), "привет");
        assert_eq!(transliterate(Lang::Ru, "spasibo"), "спасибо");
        assert_eq!(transliterate(Lang::Ru, "shchi"), "щи");
        assert_eq!(transliterate(Lang::Ru, "zhizn'"), "жизнь");
        assert_eq!(transliterate(Lang::Ru, "Moskva"), "Москва");
        assert_eq!(transliterate(Lang::Ru, "yazyk"), "язык");
        assert_eq!(transliterate(Lang::Ru, "dobryy"), "добрый");
        assert_eq!(transliterate(Lang::Ru, "kakoy"), "какой");
        assert_eq!(transliterate(Lang::Ru, "chto"), "что");
    }

    #[test]
    fn ukrainian() {
        assert_eq!(transliterate(Lang::Uk, "pryvit"), "привіт");
        assert_eq!(transliterate(Lang::Uk, "dyakuyu"), "дякую");
        assert_eq!(transliterate(Lang::Uk, "ukrayina"), "україна");
    }

    #[test]
    fn arabic_and_persian() {
        assert_eq!(transliterate(Lang::Ar, "salam"), "سلم");
        assert_eq!(candidates(Lang::Ar, "salam")[0], "سلام", "辞書の語を規則変換より先に出す");
        assert_eq!(transliterate(Lang::Ar, "7abibi"), "حبب");
        assert_eq!(transliterate(Lang::Ar, "shukran"), "شكرن");
        assert_eq!(candidates(Lang::Ar, "shukran")[0], "شكرا");
        assert_eq!(transliterate(Lang::Fa, "chetori"), "چتر");
        assert_eq!(candidates(Lang::Fa, "chetori")[0], "چطوری");
        assert_eq!(candidates(Lang::Fa, "mersi")[0], "مرسی");
        assert!(transliterate(Lang::Fa, "pargar").starts_with("پر"));
        assert!(transliterate(Lang::Fa, "gozar").starts_with("گ"));
    }

    #[test]
    fn hebrew_with_final_forms() {
        assert_eq!(candidates(Lang::He, "shalom")[0], "שלום");
        assert_eq!(transliterate(Lang::He, "shlm"), "שלם", "語末の מ は終わり形 ם");
        assert_eq!(transliterate(Lang::He, "bn"), "בן", "語末の נ は ן");
    }

    #[test]
    fn korean_two_set() {
        let t = |s: &str| transliterate(Lang::Ko, s);
        assert_eq!(t("dkssud"), "안녕");
        assert_eq!(t("gksrmf"), "한글");
        assert_eq!(t("rkskek"), "가나다");
        assert_eq!(t("rhk"), "과");
        assert_eq!(t("qlqjs"), "비번");
        assert_eq!(t("dnjfk"), "워라");
        // 複合終声(ㄺ)と、母音が続いたときの終声の繰り上がり
        assert_eq!(t("fr"), "ㄹㄱ");
        assert_eq!(t("dkfrk"), "알가");
    }

    #[test]
    fn european_accents() {
        assert_eq!(compose_accents("cafe'"), "café");
        assert_eq!(compose_accents("manana"), "manana");
        assert_eq!(compose_accents("n~"), "ñ");
        assert_eq!(compose_accents("u:ber"), "über");
        assert_eq!(compose_accents("fac,ade"), "façade");
        assert_eq!(compose_accents("o/"), "ø");
        assert_eq!(compose_accents("E'cole"), "École");
        assert!(candidates(Lang::De, "fuer").contains(&"für".to_string()));
        assert!(candidates(Lang::De, "strasse").contains(&"straße".to_string()));
    }

    #[test]
    fn script_detection_and_rtl() {
        assert_eq!(detect_script("Привет, как дела"), Some(Lang::Ru));
        assert_eq!(detect_script("Привіт, як справи їжак"), Some(Lang::Uk));
        assert_eq!(detect_script("مرحبا بكم في العالم"), Some(Lang::Ar));
        assert_eq!(detect_script("سلام چطوری گل"), Some(Lang::Fa));
        assert_eq!(detect_script("שלום עולם"), Some(Lang::He));
        assert_eq!(detect_script("안녕하세요"), Some(Lang::Ko));
        assert_eq!(detect_script("Hello world"), None);
        assert!(is_rtl_text("سلام") && is_rtl_text("שלום") && !is_rtl_text("привет"));
        assert!(Lang::Ar.rtl() && Lang::Fa.rtl() && Lang::He.rtl() && !Lang::Ru.rtl());
    }

    #[test]
    fn language_profile_ids_are_unique() {
        let mut g: Vec<u128> = ALL.iter().map(|l| l.profile_guid()).collect();
        g.sort();
        g.dedup();
        assert_eq!(g.len(), ALL.len());
        for l in ALL {
            assert_eq!(Lang::from_profile_guid(l.profile_guid()), Some(*l));
            assert_eq!(Lang::from_code(l.code()), Some(*l));
        }
        assert_eq!(Lang::Ja.profile_guid(), 0xC1D3E5F7_2A4B_4C6D_8E90_1F2A3B4C5D6E, "日本語は従来の登録と同じ");
    }
}



/// ハングルを、2ボル式キーボードで打つキー(英字)に直す。スペースなどハングル以外は読み飛ばす。
pub fn hangul_to_keys(s: &str) -> String {
    const L: [&str; 19] = ["r", "R", "s", "e", "E", "f", "a", "q", "Q", "t", "T", "d", "w", "W", "c", "z", "x", "v", "g"];
    const V: [&str; 21] = ["k", "o", "i", "O", "j", "p", "u", "P", "h", "hk", "ho", "hl", "y", "n", "nj", "np", "nl", "b", "m", "ml", "l"];
    const T: [&str; 28] = ["", "r", "R", "rt", "s", "sw", "sg", "e", "f", "fr", "fa", "fq", "ft", "fx", "fv", "fg", "a", "q", "qt", "t", "T", "d", "w", "c", "z", "x", "v", "g"];
    let mut out = String::new();
    for c in s.chars() {
        let u = c as u32;
        if (0xAC00..=0xD7A3).contains(&u) {
            let i = (u - 0xAC00) as usize;
            out.push_str(L[i / 588]);
            out.push_str(V[(i % 588) / 28]);
            out.push_str(T[i % 28]);
        } else if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}
