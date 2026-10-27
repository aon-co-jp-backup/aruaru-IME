//! 定型文。フリマ・オークションのサイト(メルカリ・ラクマ・ヤフオク・Yahoo!フリマ)を使っているときに、
//! 読み(例: ありがとう / ねさげ / はっそう)から、取引でよく使う文章を変換候補に出す。
//! 「ていけい」(定型文)と打つと、そのサイトの定型文をすべて並べる。
//!
//! 利用者の定型文は、データ領域の `templates.tsv` に書く(組み込みより先に出る)。
//! 書式: `読み<TAB>文章<TAB>サイト`(サイトは省略可。`mrya` の文字の組み合わせ、省略は全サイト)。

/// 利用しているサイト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site {
    Mercari,
    Rakuma,
    YahooAuction,
    YahooFlea,
}

impl Site {
    fn code(self) -> char {
        match self {
            Site::Mercari => 'm',
            Site::Rakuma => 'r',
            Site::YahooAuction => 'y',
            Site::YahooFlea => 'f',
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Site::Mercari => "メルカリ",
            Site::Rakuma => "ラクマ",
            Site::YahooAuction => "ヤフオク",
            Site::YahooFlea => "Yahoo!フリマ",
        }
    }
}

/// 画面の文字(ウィンドウの題名・アドレス・ページの文字)から、利用中のサイトを見分ける。
/// 「ヤフオク」より前に「Yahoo!フリマ」を調べる(どちらも Yahoo を含むため)。
pub fn detect_site(screen_text: &str) -> Option<Site> {
    let s = screen_text.to_lowercase();
    let has = |keys: &[&str]| keys.iter().any(|k| s.contains(k));
    if has(&["メルカリ", "mercari", "kouzoh"]) {
        Some(Site::Mercari)
    } else if has(&["ラクマ", "rakuma", "fril"]) {
        Some(Site::Rakuma)
    } else if has(&["yahoo!フリマ", "yahooフリマ", "ヤフーフリマ", "paypayフリマ", "paypay フリマ", "paypayfleamarket", "paypayfleamarket"]) {
        Some(Site::YahooFlea)
    } else if has(&["ヤフオク", "yahoo!オークション", "yahooオークション", "auctions.yahoo", "yauction"]) {
        Some(Site::YahooAuction)
    } else {
        None
    }
}

/// 組み込みの定型文: `読み,読み|サイト|文章`。サイトは `*`(全部)か `mrya` の組み合わせ。
const BUILTIN: &str = "\
はじめまして,こんにちは,ごあいさつ|*|はじめまして。商品を拝見し、購入を検討しております。よろしくお願いいたします。
こうにゅうまえ,こめんと,かいます|mrf|購入前にコメント失礼いたします。こちらの商品はまだ購入可能でしょうか?
にゅうさつ,しつもん|y|はじめまして。商品について質問がございます。入札前にお答えいただけますでしょうか。
ねさげ,ねびき,ねだん|mrf|コメント失礼いたします。ご検討いただけるようでしたら、お値下げは可能でしょうか?難しい場合は、そのままで構いません。
ねさげ,ねびき,ねだん|y|お世話になります。即決価格での落札は可能でしょうか?ご検討のほど、よろしくお願いいたします。
ねさげへんしん,ねさげおことわり|*|コメントありがとうございます。申し訳ございませんが、お値下げは難しいため、この金額でお願いいたします。
ねさげへんしん,ねさげおーけー|*|コメントありがとうございます。この金額でしたら承知いたしました。ご購入手続きをお願いいたします。
そくこうにゅう,そくこう|mrf|即購入OKです。ご希望の方は、そのままご購入ください。
そくけつ|y|即決価格での落札も歓迎です。ご検討よろしくお願いいたします。
しんぴん,みしよう|*|新品・未使用です。開封のみ、使用はしておりません。
ちゅうこ,のーくれーむ|*|中古品のため、使用感がございます。神経質な方はご遠慮ください。ノークレーム・ノーリターンでお願いいたします。
そうりょうこみ|*|送料込み(出品者負担)の価格です。
まとめがい,まとめ|*|まとめてご購入の場合は、お値引きいたします。コメントにてご相談ください。
こうにゅうありがとう,ありがとう,ごこうにゅう|mrf|ご購入いただき、ありがとうございます。丁寧に梱包し、できるだけ早く発送いたします。どうぞよろしくお願いいたします。
らくさつありがとう,ありがとう,らくさつ|y|ご落札いただき、ありがとうございます。お支払いの確認後、丁寧に梱包して発送いたします。どうぞよろしくお願いいたします。
はっそう,はっそうしました|*|本日、発送いたしました。到着まで今しばらくお待ちください。
はっそうおくれ,はっそうおくれる|*|申し訳ございません。発送が少し遅れております。準備ができ次第、すぐに発送いたします。
とどいた,じゅりょう,とうちゃく|*|商品が無事に届きました。丁寧にご対応いただき、ありがとうございました。
ひょうか,とりひき|*|迅速かつ丁寧なお取引をありがとうございました。とても気持ちよくお取引ができました。今後ともよろしくお願いいたします。
おわび,もうしわけ|*|ご迷惑をおかけして、申し訳ございません。誠意をもって対応いたします。
きゃんせる|*|大変申し訳ございませんが、事情により、お取引のキャンセルをお願いできないでしょうか。ご迷惑をおかけし、申し訳ございません。
";

/// 利用者の定型文を読み込む。
pub fn load_user(root: &std::path::Path) -> String {
    std::fs::read_to_string(root.join("templates.tsv")).unwrap_or_default().trim_start_matches('\u{feff}').to_string()
}

/// 自動学習した定型文のファイル(`reading<TAB>text<TAB>サイト<TAB>回数`)。
const LEARNED: &str = "templates-learned.tsv";
const LEARN_MIN_CHARS: usize = 10;
const LEARN_MAX_CHARS: usize = 400;
const LEARN_MAX_ENTRIES: usize = 300;

/// 利用者の定型文 + 自動学習した定型文(よく使うものが先)。`lookup` に渡す。
pub fn load_all(root: &std::path::Path) -> String {
    let mut out = load_user(root);
    let mut learned: Vec<(u32, String)> = std::fs::read_to_string(root.join(LEARNED))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() >= 4).then(|| (f[3].parse().unwrap_or(1), format!("{}\t{}\t{}", f[0], f[1], f[2])))
        })
        .collect();
    learned.sort_by(|a, b| b.0.cmp(&a.0));
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    for (_, l) in learned {
        out.push_str(&l);
        out.push('\n');
    }
    out
}

/// フリマ・オークションのサイトで確定した、ある程度長い文章を覚える(同じ文章を使うほど先に出る)。
/// 読みは、打った読み(ひらがな)。パスワード欄などはここへ来ない(呼び出し側で除く)。
pub fn learn(root: &std::path::Path, reading: &str, text: &str, site: Site) -> bool {
    let text = text.trim();
    let n = text.chars().count();
    if !(LEARN_MIN_CHARS..=LEARN_MAX_CHARS).contains(&n) || text.contains(['\n', '\t']) || reading.chars().count() < 2 || reading.contains('\t') {
        return false;
    }
    let path = root.join(LEARNED);
    let mut rows: Vec<(String, String, String, u32)> = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            (f.len() >= 4).then(|| (f[0].to_string(), f[1].to_string(), f[2].to_string(), f[3].parse().unwrap_or(1)))
        })
        .collect();
    let code = site.code().to_string();
    if let Some(r) = rows.iter_mut().find(|r| r.1 == text) {
        r.3 += 1;
        if !r.2.contains(site.code()) {
            r.2.push(site.code());
        }
        if r.0 != reading && !r.0.split(',').any(|k| k == reading) && r.0.split(',').count() < 5 {
            r.0 = format!("{},{}", r.0, reading);
        }
    } else {
        rows.push((reading.to_string(), text.to_string(), code, 1));
    }
    if rows.len() > LEARN_MAX_ENTRIES {
        rows.sort_by(|a, b| b.3.cmp(&a.3));
        rows.truncate(LEARN_MAX_ENTRIES);
    }
    let body: String = rows.iter().map(|r| format!("{}\t{}\t{}\t{}\n", r.0, r.1, r.2, r.3)).collect();
    std::fs::create_dir_all(root).ok();
    std::fs::write(path, body).is_ok()
}

fn site_ok(spec: &str, site: Site) -> bool {
    let spec = spec.trim();
    spec.is_empty() || spec == "*" || spec.contains(site.code())
}

/// 読み(ひらがな)に合う定型文。読みの先頭が、見出しの読みの先頭と一致するもの(2文字以上)。
/// `ていけい` は、そのサイトの定型文をすべて返す。利用者の定型文が先。
pub fn lookup(hira: &str, site: Site, user_tsv: &str, max: usize) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    if hira.chars().count() < 2 {
        return out;
    }
    let all = "ていけいぶん".starts_with(hira) || "ていけい".starts_with(hira);
    let mut add = |text: &str| {
        if !text.is_empty() && !out.iter().any(|x| x == text) {
            out.push(text.to_string());
        }
    };
    // 利用者: 読み<TAB>文章<TAB>サイト
    for l in user_tsv.lines() {
        let mut it = l.split('\t');
        let (Some(r), Some(t)) = (it.next(), it.next()) else { continue };
        if !site_ok(it.next().unwrap_or(""), site) {
            continue;
        }
        if all || r.split(',').any(|k| k.trim().starts_with(hira)) {
            add(t.trim());
        }
    }
    // 組み込み: 読み,読み|サイト|文章
    for l in BUILTIN.lines() {
        let mut it = l.splitn(3, '|');
        let (Some(keys), Some(spec), Some(t)) = (it.next(), it.next(), it.next()) else { continue };
        if !site_ok(spec, site) {
            continue;
        }
        if all || keys.split(',').any(|k| k.starts_with(hira)) {
            add(t);
        }
    }
    out.truncate(max);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_the_four_sites() {
        assert_eq!(detect_site("メルカリ - 日本最大のフリマサービス"), Some(Site::Mercari));
        assert_eq!(detect_site("https://jp.mercari.com/item/m123"), Some(Site::Mercari));
        assert_eq!(detect_site("ラクマ（旧フリル）"), Some(Site::Rakuma));
        assert_eq!(detect_site("fril.jp rakuma.rakuten.co.jp"), Some(Site::Rakuma));
        assert_eq!(detect_site("Yahoo!フリマ（旧PayPayフリマ）"), Some(Site::YahooFlea));
        assert_eq!(detect_site("ヤフオク! - 商品の出品"), Some(Site::YahooAuction));
        assert_eq!(detect_site("auctions.yahoo.co.jp"), Some(Site::YahooAuction));
        assert_eq!(detect_site("Googleで検索"), None);
    }

    #[test]
    fn gives_site_specific_templates() {
        let t = lookup("ありがとう", Site::Mercari, "", 10);
        assert!(t.iter().any(|x| x.contains("ご購入いただき")), "{t:?}");
        assert!(!t.iter().any(|x| x.contains("ご落札")), "メルカリに落札の文は出さない: {t:?}");
        let t = lookup("ありがとう", Site::YahooAuction, "", 10);
        assert!(t.iter().any(|x| x.contains("ご落札")), "{t:?}");
        assert!(!t.iter().any(|x| x.contains("ご購入いただき")), "{t:?}");
    }

    #[test]
    fn prefix_and_all_templates() {
        // 打ちかけの読みでも出る
        assert!(!lookup("はっ", Site::Rakuma, "", 10).is_empty());
        // 1文字では出さない
        assert!(lookup("は", Site::Rakuma, "", 10).is_empty());
        // ていけい → そのサイトの定型文を全部
        let all = lookup("ていけい", Site::Mercari, "", 100);
        assert!(all.len() >= 10, "{}", all.len());
        assert!(lookup("ていけい", Site::YahooAuction, "", 100).iter().any(|x| x.contains("ご落札")));
    }

    #[test]
    fn learns_long_texts_per_site_and_ranks_by_use() {
        let dir = std::env::temp_dir().join(format!("aruaru-tpl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // 短い文・改行入りは覚えない
        assert!(!learn(&dir, "ありがとう", "ありがとう", Site::Mercari));
        assert!(!learn(&dir, "ありがとう", "一行目です、長い文章\n二行目です", Site::Mercari));
        assert!(learn(&dir, "とりひきあいさつ", "この度はお取引いただき、誠にありがとうございます。", Site::Mercari));
        assert!(learn(&dir, "ぷろふぃーる", "プロフィールをご一読のうえ、ご購入をお願いいたします。", Site::Mercari));
        // 2回使った方が先
        assert!(learn(&dir, "ぷろふぃーる", "プロフィールをご一読のうえ、ご購入をお願いいたします。", Site::Mercari));
        let all = load_all(&dir);
        let t = lookup("ぷろ", Site::Mercari, &all, 5);
        assert_eq!(t[0], "プロフィールをご一読のうえ、ご購入をお願いいたします。");
        // ていけい で、学習分も並ぶ。別のサイトには出ない
        assert!(lookup("ていけい", Site::Mercari, &all, 100).iter().any(|x| x.contains("お取引いただき")));
        assert!(!lookup("とりひき", Site::YahooAuction, &all, 100).iter().any(|x| x.contains("お取引いただき、誠に")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn user_templates_come_first() {
        let u = "ありがとう\tうちのオリジナル文\tm\nありがとう\tヤフオクだけの文\ty\n";
        let t = lookup("ありがとう", Site::Mercari, u, 10);
        assert_eq!(t[0], "うちのオリジナル文");
        assert!(!t.contains(&"ヤフオクだけの文".to_string()));
    }
}
