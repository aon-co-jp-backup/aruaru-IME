//! 希望用語: 言語・プログラミング言語・フレームワーク・漫画・アニメ・ゲーム・ドラマ・映画などの名前を記録し、
//! aruaru-llm の検索(Web)で調べて、読みと表記の組(辞書)を作る。
//!
//! 取り出し方: 検索結果の文章から「題名(よみ)」の形を拾う(強い手がかり)。次に、何度も出てくるカタカナの名前を拾い、
//! 読みは辞書の逆引き・カタカナ→ひらがなで作る。作った組は利用者辞書へ `用語:<種類>:<名前>` の出所つきで登録するので、
//! 右クリックメニューから出所ごとにまとめて削除できる。プログラミング言語・フレームワークは語彙(補完・訂正用)を作る。

use crate::client::Client;
use crate::romaji;
use crate::userdict::UserDict;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Language,
    ProgrammingLanguage,
    Framework,
    Manga,
    Anime,
    Game,
    Drama,
    Movie,
    Other,
}

pub const CATEGORIES: &[Category] = &[
    Category::Manga,
    Category::Anime,
    Category::Game,
    Category::Drama,
    Category::Movie,
    Category::ProgrammingLanguage,
    Category::Framework,
    Category::Language,
    Category::Other,
];

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::Language => "言語 / Language",
            Category::ProgrammingLanguage => "プログラミング言語 / Programming language",
            Category::Framework => "フレームワーク / Framework",
            Category::Manga => "漫画 / Manga",
            Category::Anime => "アニメ / Anime",
            Category::Game => "ゲーム / Game",
            Category::Drama => "ドラマ / Drama",
            Category::Movie => "映画 / Movie",
            Category::Other => "その他 / Other",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Category::Language => "language",
            Category::ProgrammingLanguage => "programming",
            Category::Framework => "framework",
            Category::Manga => "manga",
            Category::Anime => "anime",
            Category::Game => "game",
            Category::Drama => "drama",
            Category::Movie => "movie",
            Category::Other => "other",
        }
    }

    pub fn from_key(k: &str) -> Option<Category> {
        CATEGORIES.iter().copied().find(|c| c.key() == k)
    }

    /// 語彙(補完・訂正)を作る種類か。それ以外は、読みと表記の辞書を作る。
    pub fn is_code(self) -> bool {
        matches!(self, Category::ProgrammingLanguage | Category::Framework)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Done,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wish {
    pub term: String,
    pub category: Category,
    pub status: Status,
    #[serde(default)]
    pub words: usize,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub added: u64,
}

/// 検索が一時的に使えないときのメッセージ(この文面のときは失敗にせず、あとで再試行する)。
pub const SEARCH_UNAVAILABLE: &str = "検索を一時的に使えません(接続できないか、検索元が休止中です)。あとで自動でやり直します";

pub fn path(root: &Path) -> PathBuf {
    root.join("wishlist.json")
}

pub fn load(root: &Path) -> Vec<Wish> {
    std::fs::read_to_string(path(root)).ok().and_then(|s| sonic_rs::from_str(&s).ok()).unwrap_or_default()
}

pub fn save(root: &Path, v: &[Wish]) -> std::io::Result<()> {
    std::fs::create_dir_all(root)?;
    std::fs::write(path(root), serde_json::to_string_pretty(v).unwrap_or_default())
}

/// 用語を記録する。同じ(名前,種類)が既にあれば、状態を保ったまま何もしない。新しく記録したら true。
pub fn add(root: &Path, term: &str, category: Category) -> std::io::Result<bool> {
    let term = term.trim();
    if term.chars().count() < 2 || term.chars().count() > 60 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "名前は2〜60文字で入力してください"));
    }
    let mut v = load(root);
    if v.iter().any(|w| w.term == term && w.category == category) {
        return Ok(false);
    }
    v.push(Wish { term: term.to_string(), category, status: Status::Pending, words: 0, error: None, added: crate::update::now_secs() });
    save(root, &v)?;
    Ok(true)
}

pub fn set_status(root: &Path, term: &str, category: Category, status: Status, words: usize, error: Option<String>) {
    let mut v = load(root);
    if let Some(w) = v.iter_mut().find(|w| w.term == term && w.category == category) {
        w.status = status;
        w.words = words;
        w.error = error;
    }
    let _ = save(root, &v);
}

pub fn remove(root: &Path, term: &str, category: Category) -> std::io::Result<bool> {
    let mut v = load(root);
    let n = v.len();
    v.retain(|w| !(w.term == term && w.category == category));
    let changed = v.len() != n;
    if changed {
        save(root, &v)?;
    }
    Ok(changed)
}

/// 出所の名前(利用者辞書の `source`)。
pub fn source_of(category: Category, term: &str) -> String {
    format!("用語:{}:{}", category.key(), term)
}

/// 種類ごとの調べ方(検索語)。
pub fn queries(category: Category, term: &str) -> Vec<String> {
    match category {
        Category::Manga => vec![format!("{term} 漫画 登場人物 キャラクター 一覧 読み方"), format!("{term} 漫画 用語 技 組織 読み"), format!("{term} 作者 あらすじ 登場人物")],
        Category::Anime => vec![format!("{term} アニメ 登場人物 キャラクター 声優 一覧"), format!("{term} アニメ 用語 設定 読み"), format!("{term} アニメ 主題歌 あらすじ")],
        Category::Game => vec![format!("{term} ゲーム 登場人物 キャラクター 一覧"), format!("{term} ゲーム 用語 アイテム 技 読み"), format!("{term} ゲーム 攻略 wiki")],
        Category::Drama => vec![format!("{term} ドラマ 登場人物 キャスト 一覧"), format!("{term} ドラマ あらすじ 用語 読み")],
        Category::Movie => vec![format!("{term} 映画 登場人物 キャスト 一覧"), format!("{term} 映画 あらすじ 用語 読み")],
        Category::Language => vec![format!("{term} 言語 挨拶 基本単語 一覧"), format!("{term} 日本語 読み方")],
        Category::ProgrammingLanguage | Category::Framework => vec![format!("{term} API reference functions methods"), format!("{term} cheat sheet keywords syntax")],
        Category::Other => vec![format!("{term} 用語 一覧 読み"), format!("{term} とは 読み方")],
    }
}

fn is_kana(c: char) -> bool {
    matches!(c, 'ぁ'..='ゖ' | 'ァ'..='ヺ' | 'ー' | '・')
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, 'ー' | '・' | '々' | '×' | '-' | '+' | '☆' | '♪') || ('一'..='龥').contains(&c) || is_kana(c)
}

/// 括弧の直前の文字列から、読みの長さに一番合う「名前」の範囲を選ぶ(前にある「〜は」「〜の」などの助詞を巻き込まない)。
/// 漢字は1字あたり約1.6かな、それ以外は1字1かなとして、読みの長さに近い接尾辞を選ぶ。同じ近さなら長いほう。
fn best_surface(before: &[char], reading_len: usize, parts: usize) -> String {
    let mut best: Option<(f32, usize, usize)> = None; // (差, 長さ, 開始位置)
    let n = before.len();
    let mut k = n;
    let mut len = 0;
    while k > 0 && len < 24 {
        let c = before[k - 1];
        let cjk = |ch: char| ('一'..='龥').contains(&ch) || matches!(ch, 'ぁ'..='ゖ' | 'ァ'..='ヺ' | '々');
        let ok = is_name_char(c)
            || (c == ' ' && k >= 2 && k < n && ((before[k - 2].is_ascii_alphanumeric() && before[k].is_ascii_alphanumeric()) || (cjk(before[k - 2]) && cjk(before[k]))));
        if !ok {
            break;
        }
        k -= 1;
        len += 1;
        let first = before[k];
        // 先頭が助詞になりうるひらがなや空白の接尾辞は、名前の始まりにしない
        if matches!(first, 'ぁ'..='ゖ') || first == ' ' || len < 2 {
            continue;
        }
        // 読みが「かまど ねずこ」のように空白で区切られているときは、表記も同じ数の区切り(空白)を持つものだけを選ぶ
        if before[k..n].iter().filter(|ch| **ch == ' ').count() != parts.saturating_sub(1) && before[k..n].iter().any(|ch| *ch == ' ') != (parts > 1) {
            continue;
        }
        if parts > 1 && before[k..n].iter().filter(|ch| **ch == ' ').count() != parts - 1 {
            continue;
        }
        let est: f32 = before[k..n].iter().map(|ch| if ('一'..='龥').contains(ch) || *ch == '々' { 1.6 } else if *ch == ' ' { 0.0 } else { 1.0 }).sum();
        let diff = (est - reading_len as f32).abs();
        if best.is_none_or(|(d, _, _)| diff < d || (diff - d).abs() < 1e-6) {
            best = Some((diff, len, k));
        }
    }
    let raw: String = best.map(|(_, _, k)| before[k..n].iter().collect()).unwrap_or_default();
    // 「竈門 炭治郎」のように漢字・かなの間にある空白は取る(英数字の間の空白は残す)
    let cs: Vec<char> = raw.chars().collect();
    cs.iter().enumerate().filter(|(i, c)| !(**c == ' ' && *i > 0 && *i + 1 < cs.len() && !cs[*i - 1].is_ascii_alphanumeric() && !cs[*i + 1].is_ascii_alphanumeric())).map(|(_, c)| *c).collect()
}

/// 「題名(よみ)」「題名（よみ）」の形から (読み, 表記) を取り出す。読みはひらがなに直す。
pub fn extract_pairs(text: &str) -> Vec<(String, String)> {
    let cs: Vec<char> = text.chars().collect();
    let mut out: Vec<(String, String)> = vec![];
    let mut i = 0;
    while i < cs.len() {
        if matches!(cs[i], '(' | '（') {
            // 括弧の中(読み)
            let mut j = i + 1;
            while j < cs.len() && !matches!(cs[j], ')' | '）') && j - i <= 40 {
                j += 1;
            }
            if j < cs.len() && matches!(cs[j], ')' | '）') && j > i + 2 {
                let inner: String = cs[i + 1..j].iter().collect();
                let inner = inner.trim().trim_start_matches("読み").trim_start_matches(['：', ':', ' ']).to_string();
                if inner.chars().count() >= 2 && inner.chars().all(|c| is_kana(c) || c == ' ') && !inner.is_empty() {
                    let reading = romaji::kata_to_hira(&inner.replace(' ', "").replace('・', ""));
                    let surface = best_surface(&cs[..i], reading.chars().count(), inner.split_whitespace().count());
                    if surface.chars().count() >= 2 && surface != reading && !surface.chars().all(|c| matches!(c, 'ぁ'..='ゖ')) && reading.chars().count() >= 2 {
                        let pair = (reading, surface);
                        if !out.contains(&pair) {
                            out.push(pair);
                        }
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// 何度も出てくるカタカナの名前(3文字以上)を (ひらがな読み, カタカナ表記) で返す。
pub fn katakana_names(text: &str, min_count: usize) -> Vec<(String, String)> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    let mut cur = String::new();
    for c in text.chars().chain(Some(' ')) {
        if matches!(c, 'ァ'..='ヺ' | 'ー' | '・') {
            cur.push(c);
        } else {
            let t = cur.trim_matches(['ー', '・']).to_string();
            if t.chars().count() >= 3 && t.chars().count() <= 24 && t.chars().any(|ch| matches!(ch, 'ァ'..='ヺ')) {
                *freq.entry(t).or_default() += 1;
            }
            cur.clear();
        }
    }
    let mut v: Vec<(usize, String)> = freq.into_iter().filter(|(_, n)| *n >= min_count).map(|(s, n)| (n, s)).collect();
    v.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    v.into_iter().take(150).map(|(_, s)| (romaji::kata_to_hira(&s.replace('・', "")), s)).collect()
}

/// 集めた文章から辞書の組を作り、利用者辞書へ出所つきで登録する(通信を含まない部分)。登録件数を返す。
pub fn build_dictionary(
    user: &mut UserDict,
    category: Category,
    term: &str,
    text: &str,
    index: Option<&HashMap<String, (u16, String)>>,
) -> Result<usize, String> {
    let source = source_of(category, term);
    let mut pairs = extract_pairs(text);
    for p in katakana_names(text, 2) {
        if !pairs.iter().any(|x| x.1 == p.1) {
            pairs.push(p);
        }
    }
    // 名前そのもの(読みが分かれば)
    let empty = HashMap::new();
    if let Some(r) = crate::docimport::reading_of(term, index.unwrap_or(&empty)) {
        if !pairs.iter().any(|x| x.1 == term) {
            pairs.insert(0, (r, term.to_string()));
        }
    }
    if pairs.len() < 2 {
        return Err(format!("{term}: 読みと表記の組を見つけられませんでした({}組)。名前を正確に入力するか、別の表記で試してください", pairs.len()));
    }
    user.remove_source(&source).map_err(|e| e.to_string())?; // 調べ直しは入れ替え
    let mut n = 0;
    for (r, s) in pairs.into_iter().take(400) {
        if user.add_from(&r, &s, Some(&source)).map_err(|e| e.to_string())? {
            n += 1;
        }
    }
    Ok(n)
}

/// 1件を調べて辞書にする。記録の状態も更新する。戻り値は登録した語数。
pub fn research(
    client: &Client,
    root: &Path,
    term: &str,
    category: Category,
    index: Option<&HashMap<String, (u16, String)>>,
) -> Result<usize, String> {
    let result = (|| {
        if category.is_code() {
            return crate::uservocab::learn(client, term).map(|(_, n)| n);
        }
        let mut text = String::new();
        for (n, q) in queries(category, term).into_iter().enumerate() {
            if n > 0 {
                std::thread::sleep(std::time::Duration::from_secs(3)); // 検索元に拒否されないよう、間隔を空ける
            }
            for s in client.search_snippets(&q, 10) {
                text.push_str(&s);
                text.push('\n');
            }
        }
        if text.trim().is_empty() {
            return Err(SEARCH_UNAVAILABLE.to_string());
        }
        let mut user = UserDict::load(&root.join("user_dict.json"));
        build_dictionary(&mut user, category, term, &text, index)
    })();
    match &result {
        Ok(n) => set_status(root, term, category, Status::Done, *n, None),
        // 検索が一時的に使えないだけなら、失敗にせず「未了」のままにして、あとで自動で再試行する
        Err(e) if e == SEARCH_UNAVAILABLE => set_status(root, term, category, Status::Pending, 0, Some(e.clone())),
        Err(e) => set_status(root, term, category, Status::Failed, 0, Some(e.clone())),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-wish-{n}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    const TEXT: &str = "3×3 EYES(サザンアイズ)は高田裕三による漫画。三只眼吽迦羅(さんじやんうんから)と呼ばれる。\
        主人公は藤井八雲(ふじいやくも)、ヒロインはパイ。パイ は三只眼吽迦羅の末裔。パイの師は鬼眼王(きがんおう)。\
        カマクラ カマクラ ガルーダ ガルーダ。";

    #[test]
    fn extracts_title_reading_pairs() {
        let p = extract_pairs(TEXT);
        assert!(p.contains(&("さんじやんうんから".to_string(), "三只眼吽迦羅".to_string())), "{p:?}");
        assert!(p.contains(&("ふじいやくも".to_string(), "藤井八雲".to_string())), "{p:?}");
        assert!(p.contains(&("きがんおう".to_string(), "鬼眼王".to_string())), "{p:?}");
        // かなだけの表記や、読みと同じ表記は拾わない
        assert!(extract_pairs("あいうえお(あいうえお)").is_empty());
    }

    #[test]
    fn names_with_a_space_between_family_and_given_name() {
        let p = extract_pairs("主要キャラクター 竈門 炭治郎(かまど たんじろう) 主人公 竈門 禰豆子(かまど ねずこ) 炭治郎の妹 我妻 善逸(あがつま ぜんいつ)");
        assert!(p.contains(&("かまどたんじろう".to_string(), "竈門炭治郎".to_string())), "{p:?}");
        assert!(p.contains(&("あがつまぜんいつ".to_string(), "我妻善逸".to_string())), "{p:?}");
    }

    #[test]
    fn picks_repeated_katakana_names() {
        let k = katakana_names(TEXT, 2);
        assert!(k.contains(&("かまくら".to_string(), "カマクラ".to_string())));
        assert!(k.contains(&("がるーだ".to_string(), "ガルーダ".to_string())));
        assert!(!k.iter().any(|(_, s)| s == "パイ"), "3文字未満は拾わない");
    }

    #[test]
    fn builds_dictionary_with_source_and_replaces_on_rebuild() {
        let root = tmp("build");
        let mut u = UserDict::load(&root.join("user_dict.json"));
        let n = build_dictionary(&mut u, Category::Manga, "3x3 EYES", TEXT, None).unwrap();
        assert!(n >= 5);
        assert_eq!(u.exact("さんじやんうんから"), vec!["三只眼吽迦羅"]);
        assert!(u.sources().iter().any(|(s, _)| s == "用語:manga:3x3 EYES"));
        let n2 = build_dictionary(&mut u, Category::Manga, "3x3 EYES", TEXT, None).unwrap();
        assert_eq!(n, n2, "調べ直しは入れ替え(重複しない)");
        assert_eq!(u.remove_source("用語:manga:3x3 EYES").unwrap(), n);
        assert!(build_dictionary(&mut u, Category::Anime, "X", "何もない文章です", None).is_err());
    }

    #[test]
    fn wishlist_records_dedups_and_tracks_status() {
        let root = tmp("list");
        assert!(add(&root, "ドラゴンボール", Category::Anime).unwrap());
        assert!(!add(&root, "ドラゴンボール", Category::Anime).unwrap());
        assert!(add(&root, "ドラゴンボール", Category::Game).unwrap(), "種類が違えば別");
        assert!(add(&root, "x", Category::Anime).is_err());
        set_status(&root, "ドラゴンボール", Category::Anime, Status::Done, 42, None);
        let v = load(&root);
        assert_eq!(v.len(), 2);
        assert!(v.iter().any(|w| w.category == Category::Anime && w.status == Status::Done && w.words == 42));
        assert!(remove(&root, "ドラゴンボール", Category::Game).unwrap());
        assert_eq!(load(&root).len(), 1);
    }

    #[test]
    fn queries_depend_on_category() {
        assert!(queries(Category::Manga, "X").iter().any(|q| q.contains("漫画")));
        assert!(queries(Category::Framework, "Svelte").iter().any(|q| q.contains("API reference")));
        assert!(Category::Framework.is_code() && !Category::Manga.is_code());
        for c in CATEGORIES {
            assert_eq!(Category::from_key(c.key()), Some(*c));
        }
    }
}
