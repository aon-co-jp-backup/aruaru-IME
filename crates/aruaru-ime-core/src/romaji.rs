//! ローマ字→ひらがな変換とかな変換ユーティリティ。

const TABLE: &[(&str, &str)] = &[
    ("kya","きゃ"),("kyu","きゅ"),("kyo","きょ"),("sha","しゃ"),("shu","しゅ"),("sho","しょ"),("shi","し"),
    ("cha","ちゃ"),("chu","ちゅ"),("cho","ちょ"),("chi","ち"),("tsu","つ"),("nya","にゃ"),("nyu","にゅ"),("nyo","にょ"),
    ("hya","ひゃ"),("hyu","ひゅ"),("hyo","ひょ"),("mya","みゃ"),("myu","みゅ"),("myo","みょ"),("rya","りゃ"),("ryu","りゅ"),("ryo","りょ"),
    ("gya","ぎゃ"),("gyu","ぎゅ"),("gyo","ぎょ"),("bya","びゃ"),("byu","びゅ"),("byo","びょ"),("pya","ぴゃ"),("pyu","ぴゅ"),("pyo","ぴょ"),
    ("ja","じゃ"),("ju","じゅ"),("jo","じょ"),("jya","じゃ"),("jyu","じゅ"),("jyo","じょ"),("zya","じゃ"),("zyu","じゅ"),("zyo","じょ"),
    ("tya","ちゃ"),("tyu","ちゅ"),("tyo","ちょ"),("sya","しゃ"),("syu","しゅ"),("syo","しょ"),("dya","ぢゃ"),("dyu","ぢゅ"),("dyo","ぢょ"),
    ("fa","ふぁ"),("fi","ふぃ"),("fe","ふぇ"),("fo","ふぉ"),("fu","ふ"),("thi","てぃ"),("dhi","でぃ"),("tsa","つぁ"),
    ("wi","うぃ"),("we","うぇ"),("va","ゔぁ"),("vi","ゔぃ"),("vu","ゔ"),("ve","ゔぇ"),("vo","ゔぉ"),("je","じぇ"),("che","ちぇ"),("she","しぇ"),
    ("xtu","っ"),("ltu","っ"),("xtsu","っ"),("xya","ゃ"),("xyu","ゅ"),("xyo","ょ"),("lya","ゃ"),("lyu","ゅ"),("lyo","ょ"),
    ("xa","ぁ"),("xi","ぃ"),("xu","ぅ"),("xe","ぇ"),("xo","ぉ"),("la","ぁ"),("li","ぃ"),("lu","ぅ"),("le","ぇ"),("lo","ぉ"),
    ("ka","か"),("ki","き"),("ku","く"),("ke","け"),("ko","こ"),("sa","さ"),("si","し"),("su","す"),("se","せ"),("so","そ"),
    ("ta","た"),("ti","ち"),("tu","つ"),("te","て"),("to","と"),("na","な"),("ni","に"),("nu","ぬ"),("ne","ね"),("no","の"),
    ("ha","は"),("hi","ひ"),("hu","ふ"),("he","へ"),("ho","ほ"),("ma","ま"),("mi","み"),("mu","む"),("me","め"),("mo","も"),
    ("ya","や"),("yu","ゆ"),("yo","よ"),("ra","ら"),("ri","り"),("ru","る"),("re","れ"),("ro","ろ"),("wa","わ"),("wo","を"),
    ("ga","が"),("gi","ぎ"),("gu","ぐ"),("ge","げ"),("go","ご"),("za","ざ"),("zi","じ"),("zu","ず"),("ze","ぜ"),("zo","ぞ"),
    ("da","だ"),("di","ぢ"),("du","づ"),("de","で"),("do","ど"),("ba","ば"),("bi","び"),("bu","ぶ"),("be","べ"),("bo","ぼ"),
    ("pa","ぱ"),("pi","ぴ"),("pu","ぷ"),("pe","ぺ"),("po","ぽ"),("ji","じ"),
    ("a","あ"),("i","い"),("u","う"),("e","え"),("o","お"),
    ("-","ー"),(",","、"),(".","。"),("?","？"),("!","！"),("~","〜"),("[","「"),("]","」"),
];

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'i' | 'u' | 'e' | 'o')
}

/// 変換結果: (ひらがな, 変換しきれず残ったローマ字)。
pub fn to_hiragana_partial(input: &str) -> (String, String) {
    let s: Vec<char> = input.to_ascii_lowercase().chars().collect();
    let mut out = String::new();
    let mut i = 0;
    'outer: while i < s.len() {
        let c = s[i];
        // 促音: 同じ子音の連続(n以外)
        if i + 1 < s.len() && c == s[i + 1] && c.is_ascii_alphabetic() && !is_vowel(c) && c != 'n' {
            out.push('っ');
            i += 1;
            continue;
        }
        if c == 'n' {
            if i + 1 >= s.len() {
                return (out, "n".into());
            }
            let nx = s[i + 1];
            if nx == '\'' {
                out.push('ん');
                i += 2;
                continue;
            }
            if nx == 'n' {
                out.push('ん');
                i += 2;
                continue;
            }
            if !is_vowel(nx) && nx != 'y' {
                out.push('ん');
                i += 1;
                continue;
            }
        }
        for len in (1..=4).rev() {
            if i + len > s.len() {
                continue;
            }
            let key: String = s[i..i + len].iter().collect();
            if let Some((_, v)) = TABLE.iter().find(|(k, _)| *k == key) {
                out.push_str(v);
                i += len;
                continue 'outer;
            }
        }
        if !c.is_ascii_alphabetic() {
            out.push(c);
            i += 1;
            continue;
        }
        // 次の入力待ち、または不正なローマ字。いずれも残りを返す。
        return (out, s[i..].iter().collect());
    }
    (out, String::new())
}

/// 完全にかな化できるか(残りが無いか)。
pub fn is_romaji_parsable(input: &str) -> bool {
    // 'l' 始まりの小書きかな表記は日本語入力ではまれで、英単語(hello, world)と区別できないため除外。
    if input.to_ascii_lowercase().contains('l') {
        return false;
    }
    let (_, rest) = to_hiragana_partial(input);
    rest.is_empty() || rest == "n"
}

pub fn to_hiragana(input: &str) -> String {
    let (mut h, rest) = to_hiragana_partial(input);
    if rest == "n" {
        h.push('ん');
    } else {
        h.push_str(&rest);
    }
    h
}

pub fn hira_to_kata(s: &str) -> String {
    s.chars()
        .map(|c| if ('ぁ'..='ゖ').contains(&c) { char::from_u32(c as u32 + 0x60).unwrap() } else { c })
        .collect()
}

pub fn kata_to_hira(s: &str) -> String {
    s.chars()
        .map(|c| if ('ァ'..='ヶ').contains(&c) { char::from_u32(c as u32 - 0x60).unwrap() } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        assert_eq!(to_hiragana("konnnichiha"), "こんにちは");
        assert_eq!(to_hiragana("kitte"), "きって");
        assert_eq!(to_hiragana("shinbun"), "しんぶん");
        assert_eq!(to_hiragana("nihon"), "にほん");
        assert_eq!(to_hiragana("kyoto"), "きょと");
        assert_eq!(to_hiragana("konpyu-ta"), "こんぴゅーた");
        assert_eq!(hira_to_kata("こんぴゅーた"), "コンピュータ");
    }
    #[test]
    fn parsable() {
        assert!(is_romaji_parsable("sakura"));
        assert!(!is_romaji_parsable("hello"));
        assert!(!is_romaji_parsable("world"));
    }
}
