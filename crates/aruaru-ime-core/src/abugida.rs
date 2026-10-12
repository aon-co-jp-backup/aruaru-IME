//! タイ・ラオス・クメール・ミャンマー・シンハラ・チベット(ゾンカ)・ターナ(ディベヒ)・エチオピア(アムハラ・ティグリニャ)の文字の、
//! ローマ字入力。子音と母音を表で引く簡易な方式で、公式の正書法のすべてを再現するものではない
//! (チベット文字の積み重ね・タイ文字の声調記号などは扱わない)。誤りを見つけたら `words/<言語>.tsv` の語で直せる。
//!
//! 母音が子音の前に書かれる文字(タイ・ラオス)は、子音の前に置く(`ke` → เก)。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Abu {
    Thai,
    Lao,
    Khmer,
    Myanmar,
    Sinhala,
    Tibetan,
    Thaana,
}

type Cons = &'static [(&'static str, &'static str)];
/// (ローマ字, 子音に付く形, 単独の形, 子音の前に書くか)
type Vows = &'static [(&'static str, &'static str, &'static str, bool)];

struct Spec {
    cons: Cons,
    vows: Vows,
    /// 母音の付かない子音の後ろに付ける記号(シンハラの al-lakuna、ターナの sukun)。
    bare: &'static str,
}

const THAI: Spec = Spec {
    cons: &[
        ("ng", "ง"), ("kh", "ข"), ("ch", "ช"), ("th", "ท"), ("ph", "พ"), ("k", "ก"), ("c", "จ"), ("s", "ส"), ("d", "ด"), ("t", "ต"), ("n", "น"), ("b", "บ"), ("p", "ป"),
        ("f", "ฟ"), ("m", "ม"), ("y", "ย"), ("r", "ร"), ("l", "ล"), ("w", "ว"), ("h", "ห"), ("j", "จ"), ("z", "ซ"),
    ],
    vows: &[
        ("aa", "า", "อา", false), ("ii", "ี", "อี", false), ("uu", "ู", "อู", false), ("ue", "ื", "อื", false), ("ai", "ไ", "ไอ", true), ("ae", "แ", "แอ", true),
        ("a", "ะ", "อะ", false), ("i", "ิ", "อิ", false), ("u", "ุ", "อุ", false), ("e", "เ", "เอ", true), ("o", "โ", "โอ", true),
    ],
    bare: "",
};

const LAO: Spec = Spec {
    cons: &[
        ("ng", "ງ"), ("kh", "ຂ"), ("th", "ທ"), ("ph", "ພ"), ("ny", "ຍ"), ("k", "ກ"), ("c", "ຈ"), ("s", "ສ"), ("d", "ດ"), ("t", "ຕ"), ("n", "ນ"), ("b", "ບ"), ("p", "ປ"),
        ("f", "ຟ"), ("m", "ມ"), ("y", "ຢ"), ("r", "ຣ"), ("l", "ລ"), ("w", "ວ"), ("h", "ຫ"),
    ],
    vows: &[
        ("aa", "າ", "ອາ", false), ("ii", "ີ", "ອີ", false), ("uu", "ູ", "ອູ", false), ("ae", "ແ", "ແອ", true), ("ai", "ໄ", "ໄອ", true),
        ("a", "ະ", "ອະ", false), ("i", "ິ", "ອິ", false), ("u", "ຸ", "ອຸ", false), ("e", "ເ", "ເອ", true), ("o", "ໂ", "ໂອ", true),
    ],
    bare: "",
};

const KHMER: Spec = Spec {
    cons: &[
        ("ng", "ង"), ("kh", "ខ"), ("ch", "ឆ"), ("th", "ថ"), ("ph", "ផ"), ("ny", "ញ"), ("k", "ក"), ("g", "គ"), ("c", "ច"), ("j", "ជ"), ("d", "ដ"), ("t", "ត"), ("n", "ន"),
        ("b", "ប"), ("p", "ព"), ("m", "ម"), ("y", "យ"), ("r", "រ"), ("l", "ល"), ("v", "វ"), ("w", "វ"), ("s", "ស"), ("h", "ហ"), ("q", "អ"),
    ],
    vows: &[
        ("aa", "ា", "អា", false), ("ii", "ី", "ឦ", false), ("uu", "ូ", "ឩ", false), ("ae", "ែ", "ឯ", false),
        ("a", "", "អ", false), ("i", "ិ", "ឥ", false), ("u", "ុ", "ឧ", false), ("e", "េ", "អេ", false), ("o", "ោ", "អោ", false),
    ],
    bare: "",
};

const MYANMAR: Spec = Spec {
    cons: &[
        ("ng", "င"), ("ny", "ည"), ("kh", "ခ"), ("hs", "ဆ"), ("th", "သ"), ("ph", "ဖ"), ("k", "က"), ("g", "ဂ"), ("s", "စ"), ("z", "ဇ"), ("t", "တ"), ("d", "ဒ"), ("n", "န"),
        ("p", "ပ"), ("b", "ဗ"), ("m", "မ"), ("y", "ယ"), ("r", "ရ"), ("l", "လ"), ("w", "ဝ"), ("h", "ဟ"),
    ],
    vows: &[
        ("aa", "ာ", "အာ", false), ("ii", "ီ", "အီ", false), ("uu", "ူ", "အူ", false), ("ai", "ဲ", "အဲ", false),
        ("a", "", "အ", false), ("i", "ိ", "အိ", false), ("u", "ု", "အု", false), ("e", "ေ", "အေ", false), ("o", "ို", "အို", false),
    ],
    bare: "",
};

const SINHALA: Spec = Spec {
    cons: &[
        ("kh", "ඛ"), ("gh", "ඝ"), ("ch", "ච"), ("jh", "ඣ"), ("th", "ථ"), ("dh", "ධ"), ("ph", "ඵ"), ("bh", "භ"), ("sh", "ශ"), ("ng", "ඞ"), ("ny", "ඤ"),
        ("k", "ක"), ("g", "ග"), ("c", "ච"), ("j", "ජ"), ("t", "ත"), ("d", "ද"), ("n", "න"), ("p", "ප"), ("b", "බ"), ("m", "ම"), ("y", "ය"), ("r", "ර"),
        ("l", "ල"), ("v", "ව"), ("w", "ව"), ("s", "ස"), ("h", "හ"), ("f", "ෆ"),
    ],
    vows: &[
        ("aa", "ා", "ආ", false), ("ae", "ැ", "ඇ", false), ("ii", "ී", "ඊ", false), ("uu", "ූ", "ඌ", false),
        ("a", "", "අ", false), ("i", "ි", "ඉ", false), ("u", "ු", "උ", false), ("e", "ෙ", "එ", false), ("o", "ො", "ඔ", false),
    ],
    bare: "්",
};

const TIBETAN: Spec = Spec {
    cons: &[
        ("ng", "ང"), ("kh", "ཁ"), ("ch", "ཆ"), ("ny", "ཉ"), ("th", "ཐ"), ("ph", "ཕ"), ("ts", "ཙ"), ("dz", "ཛ"), ("zh", "ཞ"), ("sh", "ཤ"),
        ("k", "ཀ"), ("g", "ག"), ("c", "ཅ"), ("j", "ཇ"), ("t", "ཏ"), ("d", "ད"), ("n", "ན"), ("p", "པ"), ("b", "བ"), ("m", "མ"), ("w", "ཝ"), ("z", "ཟ"),
        ("y", "ཡ"), ("r", "ར"), ("l", "ལ"), ("s", "ས"), ("h", "ཧ"),
    ],
    vows: &[("a", "", "ཨ", false), ("i", "ི", "ཨི", false), ("u", "ུ", "ཨུ", false), ("e", "ེ", "ཨེ", false), ("o", "ོ", "ཨོ", false)],
    bare: "",
};

const THAANA: Spec = Spec {
    cons: &[
        ("sh", "ށ"), ("lh", "ޅ"), ("dh", "ދ"), ("th", "ތ"), ("gn", "ޏ"), ("ch", "ޗ"), ("h", "ހ"), ("n", "ނ"), ("r", "ރ"), ("b", "ބ"), ("k", "ކ"), ("v", "ވ"),
        ("m", "މ"), ("f", "ފ"), ("l", "ލ"), ("g", "ގ"), ("s", "ސ"), ("d", "ޑ"), ("z", "ޒ"), ("t", "ޓ"), ("y", "ޔ"), ("p", "ޕ"), ("j", "ޖ"),
    ],
    vows: &[
        ("aa", "ާ", "އާ", false), ("ee", "ީ", "އީ", false), ("oo", "ޫ", "އޫ", false), ("ey", "ޭ", "އޭ", false), ("oa", "ޯ", "އޯ", false),
        ("a", "ަ", "އަ", false), ("i", "ި", "އި", false), ("u", "ު", "އު", false), ("e", "ެ", "އެ", false), ("o", "ޮ", "އޮ", false),
    ],
    bare: "ް",
};

fn spec(s: Abu) -> &'static Spec {
    match s {
        Abu::Thai => &THAI,
        Abu::Lao => &LAO,
        Abu::Khmer => &KHMER,
        Abu::Myanmar => &MYANMAR,
        Abu::Sinhala => &SINHALA,
        Abu::Tibetan => &TIBETAN,
        Abu::Thaana => &THAANA,
    }
}

pub fn transliterate(script: Abu, input: &str) -> String {
    let sp = spec(script);
    let cs: Vec<char> = input.to_lowercase().chars().collect();
    let mut out = String::new();
    let mut pending: Option<&'static str> = None;
    let starts = |i: usize, t: &str| -> bool {
        let tc: Vec<char> = t.chars().collect();
        i + tc.len() <= cs.len() && cs[i..i + tc.len()] == tc[..]
    };
    let flush = |out: &mut String, pending: &mut Option<&'static str>| {
        if let Some(c) = pending.take() {
            out.push_str(c);
            out.push_str(sp.bare);
        }
    };
    let mut i = 0;
    while i < cs.len() {
        if let Some((t, v)) = sp.vows.iter().find(|(t, _, _, _)| starts(i, t)).map(|(t, d, ind, lead)| (*t, (*d, *ind, *lead))) {
            let (dep, ind, lead) = v;
            match pending.take() {
                Some(c) => {
                    if lead {
                        out.push_str(dep);
                        out.push_str(c);
                    } else {
                        out.push_str(c);
                        out.push_str(dep);
                    }
                }
                None => out.push_str(ind),
            }
            i += t.chars().count();
            continue;
        }
        if let Some((t, c)) = sp.cons.iter().find(|(t, _)| starts(i, t)) {
            flush(&mut out, &mut pending);
            pending = Some(c);
            i += t.chars().count();
            continue;
        }
        flush(&mut out, &mut pending);
        out.push(cs[i]);
        i += 1;
    }
    flush(&mut out, &mut pending);
    out
}

// ---------------------------------------------------------------- エチオピア文字(アムハラ語・ティグリニャ語)

/// 子音の列(ローマ字, 列の先頭の符号位置)。各列は 8 字(第1〜第7 形 + 第8 形)。
const ETHIOPIC_ROWS: &[(&str, u32)] = &[
    ("sh", 0x1238), ("ch", 0x1278), ("ny", 0x1298), ("zh", 0x12E0), ("h", 0x1200), ("l", 0x1208), ("m", 0x1218), ("r", 0x1228), ("s", 0x1230), ("q", 0x1240),
    ("b", 0x1260), ("v", 0x1268), ("t", 0x1270), ("n", 0x1290), ("k", 0x12A8), ("w", 0x12C8), ("z", 0x12D8), ("y", 0x12E8), ("d", 0x12F0), ("j", 0x1300),
    ("g", 0x1308), ("f", 0x1348), ("p", 0x1350), ("x", 0x1230),
];

/// 母音(ローマ字, 列の中の順番): e=第1(ä) u=第2 i=第3 a=第4 E=第5 ə=第6 o=第7。子音だけのときは第6形(ə)。
const ETHIOPIC_VOWELS: &[(&str, u32)] = &[("ee", 4), ("ie", 4), ("e", 0), ("u", 1), ("i", 2), ("a", 3), ("o", 6)];

pub fn transliterate_ethiopic(input: &str) -> String {
    let cs: Vec<char> = input.to_lowercase().chars().collect();
    let starts = |i: usize, t: &str| -> bool {
        let tc: Vec<char> = t.chars().collect();
        i + tc.len() <= cs.len() && cs[i..i + tc.len()] == tc[..]
    };
    let mut out = String::new();
    let mut pending: Option<u32> = None;
    let put = |out: &mut String, base: u32, ord: u32| {
        if let Some(c) = char::from_u32(base + ord) {
            out.push(c);
        }
    };
    let mut i = 0;
    while i < cs.len() {
        if let Some((t, ord)) = ETHIOPIC_VOWELS.iter().find(|(t, _)| starts(i, t)) {
            match pending.take() {
                Some(b) => put(&mut out, b, *ord),
                None => put(&mut out, 0x12A0, *ord), // 母音だけ(አ の列)
            }
            i += t.chars().count();
            continue;
        }
        if let Some((t, base)) = ETHIOPIC_ROWS.iter().find(|(t, _)| starts(i, t)) {
            if let Some(b) = pending.take() {
                put(&mut out, b, 5);
            }
            pending = Some(*base);
            i += t.chars().count();
            continue;
        }
        if let Some(b) = pending.take() {
            put(&mut out, b, 5);
        }
        out.push(cs[i]);
        i += 1;
    }
    if let Some(b) = pending.take() {
        put(&mut out, b, 5);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amharic_selam() {
        assert_eq!(transliterate_ethiopic("selam"), "ሰላም");
        assert_eq!(transliterate_ethiopic("ba"), "ባ");
    }

    #[test]
    fn sinhala_and_thaana_bare_consonants_get_their_marks() {
        assert_eq!(transliterate(Abu::Sinhala, "ayubowan"), "අයුබොවන්");
        assert_eq!(transliterate(Abu::Thaana, "ba"), "ބަ");
        assert_eq!(transliterate(Abu::Thaana, "b"), "ބް");
    }

    #[test]
    fn thai_and_lao_put_leading_vowels_before_the_consonant() {
        assert_eq!(transliterate(Abu::Thai, "ke"), "เก");
        assert_eq!(transliterate(Abu::Thai, "kaa"), "กา");
        assert_eq!(transliterate(Abu::Lao, "kai"), "ໄກ");
    }

    #[test]
    fn khmer_myanmar_tibetan_basic() {
        assert_eq!(transliterate(Abu::Khmer, "kaa"), "កា");
        assert_eq!(transliterate(Abu::Myanmar, "ka"), "က");
        assert_eq!(transliterate(Abu::Myanmar, "ki"), "ကိ");
        assert_eq!(transliterate(Abu::Tibetan, "ka"), "ཀ");
        assert_eq!(transliterate(Abu::Tibetan, "ko"), "ཀོ");
    }
}
