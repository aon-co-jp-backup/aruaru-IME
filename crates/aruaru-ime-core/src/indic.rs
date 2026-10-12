//! インド系文字(デーヴァナーガリー・ベンガル・グルムキー・グジャラート・オリヤー・タミル・テルグ・カンナダ・マラヤーラム)の
//! ラテン文字入力(ITRANS 風)と、ギリシャ文字・ジョージア文字・アルメニア文字の表。
//!
//! インド系の文字は、Unicode の配置が共通(ブロックの先頭からの相対位置が同じ)なので、1つの変換器で全部を扱う。
//! 例(デーヴァナーガリー): `namaste` → नमस्ते、`bhaarat` → भारत、`kshatriya` → क्षत्रिय、`hindii` → हिन्दी。
//! 大文字と小文字を区別する: `T Th D Dh N`(そり舌音)、`sh`(श)と `Sh`(ष)、`aa/A ii/I uu/U`(長母音)、`e/E o/O`(ドラヴィダ系の短・長)、
//! `M`(アヌスヴァーラ)、`H`(ヴィサルガ)、`~N ~n`(鼻音)、`.`(danda)。子音の連続は、自動で結合文字(ヴィラーマ)になる。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Deva,
    Beng,
    Assam,
    Guru,
    Gujr,
    Orya,
    Taml,
    Telu,
    Knda,
    Mlym,
}

impl Script {
    fn base(self) -> u32 {
        match self {
            Script::Deva => 0x0900,
            Script::Beng | Script::Assam => 0x0980,
            Script::Guru => 0x0A00,
            Script::Gujr => 0x0A80,
            Script::Orya => 0x0B00,
            Script::Taml => 0x0B80,
            Script::Telu => 0x0C00,
            Script::Knda => 0x0C80,
            Script::Mlym => 0x0D00,
        }
    }

    /// e / o が、短い母音と長い母音に分かれる(ドラヴィダ系)。
    fn dravidian(self) -> bool {
        matches!(self, Script::Taml | Script::Telu | Script::Knda | Script::Mlym)
    }
}

/// 子音の相対位置(ブロックの先頭からの差)。長いつづりを先に。
const CONS: &[(&str, u32)] = &[
    ("chh", 0x1B), ("Th", 0x20), ("Dh", 0x22), ("kh", 0x16), ("gh", 0x18), ("ch", 0x1A), ("jh", 0x1D), ("th", 0x25), ("dh", 0x27), ("ph", 0x2B), ("bh", 0x2D),
    ("sh", 0x36), ("Sh", 0x37), ("zh", 0x34), ("~N", 0x19), ("~n", 0x1E),
    ("k", 0x15), ("g", 0x17), ("c", 0x1A), ("j", 0x1C), ("T", 0x1F), ("D", 0x21), ("N", 0x23), ("t", 0x24), ("d", 0x26), ("n", 0x28),
    ("p", 0x2A), ("b", 0x2C), ("m", 0x2E), ("y", 0x2F), ("r", 0x30), ("l", 0x32), ("L", 0x33), ("v", 0x35), ("w", 0x35), ("s", 0x38), ("h", 0x39),
    ("R", 0x31), ("q", 0x15), ("f", 0x2B), ("z", 0x1C),
];

/// 母音(独立形の相対位置, 母音記号の相対位置)。`None` は、子音に内在する a。長いつづりを先に。
const VOWELS: &[(&str, u32, Option<u32>)] = &[
    ("RRi", 0x0B, Some(0x43)), ("R^i", 0x0B, Some(0x43)),
    ("aa", 0x06, Some(0x3E)), ("ii", 0x08, Some(0x40)), ("uu", 0x0A, Some(0x42)), ("ai", 0x10, Some(0x48)), ("au", 0x14, Some(0x4C)),
    ("a", 0x05, None), ("A", 0x06, Some(0x3E)), ("i", 0x07, Some(0x3F)), ("I", 0x08, Some(0x40)), ("u", 0x09, Some(0x41)), ("U", 0x0A, Some(0x42)),
];

fn put(out: &mut String, base: u32, off: u32) {
    if let Some(c) = char::from_u32(base + off) {
        out.push(c);
    }
}

/// 文字体系ごとの子音の読み替え(その文字に無い子音を、近い子音にする)。
fn fix_consonant(s: Script, off: u32) -> u32 {
    match s {
        // タミル語は、有気音・有声音の字が無い
        Script::Taml => match off {
            0x16 | 0x17 | 0x18 => 0x15,
            0x1B => 0x1A,
            0x1D => 0x1C,
            0x20 | 0x21 | 0x22 => 0x1F,
            0x25 | 0x26 | 0x27 => 0x24,
            0x2B | 0x2C | 0x2D => 0x2A,
            o => o,
        },
        // ベンガル語・アッサム語には、va と la(ळ)が無い
        Script::Beng | Script::Assam => match off {
            0x35 => 0x2C,
            0x33 => 0x32,
            0x31 | 0x34 => 0x30,
            o => o,
        },
        // グルムキー・グジャラート・オリヤー語の zh・R は、近い字に
        Script::Guru | Script::Gujr | Script::Orya => match off {
            0x34 => 0x33,
            0x31 => 0x30,
            o => o,
        },
        // デーヴァナーガリーにも zh は無い
        Script::Deva => match off {
            0x34 => 0x33,
            o => o,
        },
        _ => off,
    }
}

/// 子音そのもの(アッサム語の r・w は専用の字)。
fn put_consonant(out: &mut String, s: Script, off: u32) {
    if s == Script::Assam {
        match off {
            0x30 => {
                out.push('\u{09F0}');
                return;
            }
            0x35 => {
                out.push('\u{09F1}');
                return;
            }
            _ => {}
        }
    }
    put(out, s.base(), fix_consonant(s, off));
}

/// ラテン文字の入力を、インド系の文字にする。
pub fn transliterate(s: Script, input: &str) -> String {
    let cs: Vec<char> = input.chars().collect();
    let base = s.base();
    let mut out = String::new();
    let mut i = 0;
    // 直前が子音(母音記号・ヴィラーマをまだ付けていない)か
    let mut pending = false;
    let starts = |i: usize, t: &str| -> bool {
        let tc: Vec<char> = t.chars().collect();
        i + tc.len() <= cs.len() && cs[i..i + tc.len()] == tc[..]
    };
    while i < cs.len() {
        // ドラヴィダ系の e/E o/O、インド・アーリア系の e/o(長母音)
        if cs[i] == 'e' || cs[i] == 'E' || cs[i] == 'o' || cs[i] == 'O' {
            // ai / au は、母音の表で先に扱う(a が先なのでここには来ない)
            let long = cs[i] == 'E' || cs[i] == 'O';
            let is_e = cs[i] == 'e' || cs[i] == 'E';
            let (ind, mat) = match (is_e, s.dravidian() && !long) {
                (true, true) => (0x0E, 0x46),
                (true, false) => (0x0F, 0x47),
                (false, true) => (0x12, 0x4A),
                (false, false) => (0x13, 0x4B),
            };
            if pending {
                put(&mut out, base, mat);
            } else {
                put(&mut out, base, ind);
            }
            pending = false;
            i += 1;
            continue;
        }
        // 母音
        if let Some((t, ind, mat)) = VOWELS.iter().find(|(t, _, _)| starts(i, t)) {
            match (pending, mat) {
                (true, Some(m)) => put(&mut out, base, *m),
                (true, None) => {} // 子音の内在する a
                (false, _) => put(&mut out, base, *ind),
            }
            pending = false;
            i += t.chars().count();
            continue;
        }
        // 特別な連続: ksh / x → क्ष、GY / jn → ज्ञ
        if starts(i, "ksh") || starts(i, "x") {
            if pending {
                put(&mut out, base, 0x4D);
            }
            put_consonant(&mut out, s, 0x15);
            put(&mut out, base, 0x4D);
            put_consonant(&mut out, s, 0x37);
            pending = true;
            i += if starts(i, "x") { 1 } else { 3 };
            continue;
        }
        if starts(i, "GY") {
            if pending {
                put(&mut out, base, 0x4D);
            }
            put_consonant(&mut out, s, 0x1C);
            put(&mut out, base, 0x4D);
            put_consonant(&mut out, s, 0x1E);
            pending = true;
            i += 2;
            continue;
        }
        // 子音
        if let Some((t, off)) = CONS.iter().find(|(t, _)| starts(i, t)) {
            // R と zh は、ドラヴィダ系とマラヤーラム語の専用の字。それ以外の文字では近い字になる(fix_consonant)
            if pending {
                put(&mut out, base, 0x4D); // ヴィラーマ(子音の連続を結合文字にする)
            }
            put_consonant(&mut out, s, *off);
            pending = true;
            i += t.chars().count();
            continue;
        }
        // 記号
        let c = cs[i];
        match c {
            'M' => put(&mut out, base, 0x02),
            'H' => put(&mut out, base, 0x03),
            '.' if matches!(s, Script::Deva) => out.push('\u{0964}'),
            _ => out.push(c),
        }
        pending = false;
        i += 1;
    }
    out
}

// ---------------------------------------------------------------- ギリシャ・ジョージア・アルメニア(1字ずつの表)

pub const GREEK: &[(&str, &str)] = &[
    ("th", "θ"), ("ch", "χ"), ("ps", "ψ"), ("ph", "φ"), ("ks", "ξ"), ("ee", "η"), ("oo", "ω"),
    ("a'", "ά"), ("e'", "έ"), ("h'", "ή"), ("i'", "ί"), ("o'", "ό"), ("u'", "ύ"), ("y'", "ύ"), ("w'", "ώ"),
    ("a", "α"), ("b", "β"), ("g", "γ"), ("d", "δ"), ("e", "ε"), ("z", "ζ"), ("h", "η"), ("i", "ι"), ("k", "κ"), ("l", "λ"), ("m", "μ"), ("n", "ν"),
    ("x", "ξ"), ("o", "ο"), ("p", "π"), ("r", "ρ"), ("s", "σ"), ("t", "τ"), ("u", "υ"), ("y", "υ"), ("f", "φ"), ("w", "ω"), ("c", "κ"), ("j", "ι"), ("q", "κ"),
];

pub const GEORGIAN: &[(&str, &str)] = &[
    ("zh", "ჟ"), ("sh", "შ"), ("ch'", "ჭ"), ("ch", "ჩ"), ("ts'", "წ"), ("ts", "ც"), ("dz", "ძ"), ("kh", "ხ"), ("gh", "ღ"), ("ph", "ფ"), ("t'", "ტ"),
    ("a", "ა"), ("b", "ბ"), ("g", "გ"), ("d", "დ"), ("e", "ე"), ("v", "ვ"), ("z", "ზ"), ("t", "თ"), ("i", "ი"), ("k", "კ"), ("l", "ლ"), ("m", "მ"),
    ("n", "ნ"), ("o", "ო"), ("p", "პ"), ("r", "რ"), ("s", "ს"), ("u", "უ"), ("f", "ფ"), ("q", "ქ"), ("y", "ყ"), ("j", "ჯ"), ("h", "ჰ"), ("c", "ც"), ("w", "ვ"), ("x", "ხ"),
];

pub const ARMENIAN: &[(&str, &str)] = &[
    ("zh", "ժ"), ("gh", "ղ"), ("ch", "ճ"), ("sh", "շ"), ("dz", "ձ"), ("ts", "ց"), ("th", "թ"), ("ph", "փ"), ("kh", "խ"), ("vo", "ո"), ("ye", "ե"),
    ("a", "ա"), ("b", "բ"), ("g", "գ"), ("d", "դ"), ("e", "ե"), ("z", "զ"), ("y", "ը"), ("t", "տ"), ("i", "ի"), ("l", "լ"), ("x", "խ"), ("c", "ծ"),
    ("k", "կ"), ("h", "հ"), ("j", "ջ"), ("m", "մ"), ("n", "ն"), ("o", "օ"), ("p", "պ"), ("r", "ր"), ("s", "ս"), ("v", "վ"), ("u", "ու"), ("f", "ֆ"), ("q", "ք"), ("w", "ւ"),
];

/// ギリシャ語の語末の σ は ς にする。
pub fn greek_final_sigma(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::new();
    for (i, c) in cs.iter().enumerate() {
        let end_of_word = i + 1 == cs.len() || !cs[i + 1].is_alphabetic();
        if *c == 'σ' && end_of_word && i > 0 && cs[i - 1].is_alphabetic() {
            out.push('ς');
        } else {
            out.push(*c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devanagari_words() {
        assert_eq!(transliterate(Script::Deva, "namaste"), "नमस्ते");
        assert_eq!(transliterate(Script::Deva, "bhaarat"), "भारत");
        assert_eq!(transliterate(Script::Deva, "hindii"), "हिन्दी");
        assert_eq!(transliterate(Script::Deva, "kshatriya"), "क्षत्रिय");
        assert_eq!(transliterate(Script::Deva, "bhaaShaa"), "भाषा");
        assert_eq!(transliterate(Script::Deva, "aap"), "आप");
        assert_eq!(transliterate(Script::Deva, "hai."), "है।");
    }

    #[test]
    fn other_indic_scripts_use_the_same_layout() {
        assert_eq!(transliterate(Script::Beng, "bhaaShaa"), "ভাষা");
        assert_eq!(transliterate(Script::Telu, "telugu"), "తెలుగు");
        assert_eq!(transliterate(Script::Knda, "kannaDa"), "ಕನ್ನಡ");
        assert_eq!(transliterate(Script::Mlym, "malayaaLaM"), "മലയാളം");
        assert_eq!(transliterate(Script::Gujr, "gujaraatii"), "ગુજરાતી");
        assert_eq!(transliterate(Script::Guru, "panjaabii"), "ਪਨ੍ਜਾਬੀ");
        assert_eq!(transliterate(Script::Taml, "tamiz"), "தமிஜ");
    }

    #[test]
    fn dravidian_short_and_long_e() {
        // タミル語の e は短母音(ெ)、E は長母音(ே)
        assert_eq!(transliterate(Script::Taml, "ke"), "கெ");
        assert_eq!(transliterate(Script::Taml, "kE"), "கே");
        // ヒンディー語の e は ए の仲間(े)
        assert_eq!(transliterate(Script::Deva, "ke"), "के");
    }

    #[test]
    fn greek_final_sigma_rule() {
        assert_eq!(greek_final_sigma("λογοσ"), "λογος");
        assert_eq!(greek_final_sigma("σοφια"), "σοφια");
    }
}
