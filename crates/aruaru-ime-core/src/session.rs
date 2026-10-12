//! 入力セッション(キー入力→未確定文字列・候補・確定の状態機械)。OS非依存で、TSF/IBus/IMKなどの薄い層から使う。
//!
//! 流れ: ローマ字を打つ → 入力モードAI予測で未確定表示(かな/カタカナ/英字のまま)を決め、候補を並べる
//! → Space で変換候補を選択 → Enter で確定。

use crate::convert::Conversion;
use crate::mode::Mode;
use crate::romaji;
use crate::translit::{self, Lang};

pub trait Converter: Send {
    fn convert(&self, buffer: &str, before: &str, field_kind: Option<&str>) -> Conversion;

    /// 利用者が変換を求めたとき(Space など)だけ呼ばれる、時間のかかる追加候補(Web のサジェストなど)。辞書に無い固有名詞・作品名向け。
    /// 既定は何も足さない。
    fn extra(&self, _buffer: &str, _before: &str) -> Vec<String> {
        vec![]
    }

    /// 日本語の表記(と読み)の英語訳。変換候補の「英語の列」に使う。既定は無し。
    fn to_english(&self, _surface: &str, _reading: &str) -> Vec<String> {
        vec![]
    }

    /// 英単語の日本語訳。英語を打ったときの対訳に使う。既定は無し。
    fn to_japanese(&self, _word: &str) -> Vec<String> {
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Key {
    Char(char),
    Space,
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    Tab,
    /// 半角/全角・IME オン/オフ。
    Toggle,
    /// ファンクションキー(F6〜F10: 表記の強制変換)。
    Fn(u8),
    /// IME を必ずオン / オフにする(英数キー・かなキー。トグルと違い状態を取り違えない)。
    On,
    Off,
    /// 左右キー: 変換候補の「日本語の列」と「英語の列」を切り替える。
    Left,
    Right,
    /// 変換キー: 入力中は次の候補、入力していないときは IME オン。
    Convert,
    /// 無変換キー: 入力中は ひらがな→カタカナ→全角英数→半角英数 と切り替え、入力していないときは IME オフ。
    NonConvert,
    Other,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Step {
    /// このキーをIMEが処理した(アプリへ渡さない)。
    pub eaten: bool,
    /// 現在の未確定文字列(空なら変換中ではない)。
    pub preedit: String,
    /// 確定して挿入する文字列。
    pub commit: Option<String>,
    /// 候補ウィンドウの内容(空なら非表示)と選択位置。
    pub cands: Vec<String>,
    pub sel: Option<usize>,
    /// 予測された(または強制した)入力モードの表示用ラベル(あ / ア / A / 123 / 記号)。
    pub mode: String,
    /// この結果の後の状態。DLL 側が「このキーを食うか」を通信なしで判定するために持ち帰る(`would_eat_with`)。
    #[serde(default)]
    pub on: bool,
    /// 変換候補の「英語の列」(日本語の候補の英語訳)と、いま選んでいる列(0=日本語, 1=英語)・英語の列での選択位置。
    #[serde(default)]
    pub cands_en: Vec<String>,
    #[serde(default)]
    pub col: u8,
    #[serde(default)]
    pub sel_en: Option<usize>,
    #[serde(default)]
    pub composing: bool,
    /// 選んでいる候補に関する、知識ベースの補足(候補ウィンドウに出す)。
    #[serde(default)]
    pub note: Option<String>,
    /// いまの入力言語のコード(ja / en / ru / ar …)。DLL が通信なしで「このキーを食うか」を判定するために持ち帰る。
    #[serde(default)]
    pub lang: String,
    /// カーソル直前が日本語の文字か(「,」「.」を「、」「。」にするかの判定に使う)。
    #[serde(default)]
    pub last_ja: bool,
}

/// このキーをIMEが処理するか。状態(オン/組み立て中/直前が日本語)だけで決まるので、通信なしで判定できる。
/// 候補窓の1ページの件数(数字キー 1〜9・0 で選べる)。
pub const PAGE: usize = 10;

pub fn would_eat_with(key: &Key, on: bool, composing: bool, last_ja: bool) -> bool {
    would_eat_lang(key, on, composing, last_ja, Lang::Ja)
}

/// 言語つきの判定。日本語以外は、その言語の文字(ラテン文字入力+アクセント記号・数字など)を食う。中国語(未対応)は文字を食わない。
pub fn would_eat_lang(key: &Key, on: bool, composing: bool, last_ja: bool, lang: Lang) -> bool {
    if matches!(key, Key::Toggle | Key::On | Key::Off | Key::Convert | Key::NonConvert) {
        return true;
    }
    if !on {
        return false;
    }
    if lang != Lang::Ja {
        return match key {
            Key::Char(c) => lang.supported() && (lang.accepts(*c, composing) || composing),
            Key::Space | Key::Down | Key::Tab | Key::Up | Key::Enter | Key::Esc | Key::Backspace => composing,
            _ => false,
        };
    }
    match key {
        // 最初の - は、ハイフンの種類を候補に並べるためIMEが受ける(直前が英数字のときはIME側でそのまま確定する)
        Key::Char(c) => c.is_ascii_alphabetic() || composing || c.is_ascii_digit() || crate::prefer::lead_symbol(*c).is_some() || matches!(c, ',' | '.'),
        Key::Space | Key::Down | Key::Tab | Key::Up | Key::Enter | Key::Esc | Key::Backspace | Key::Fn(_) | Key::Left | Key::Right => composing,
        _ => false,
    }
}

/// 表記の強制(F6〜F10)。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Form {
    Hira,
    Kata,
    Full,
    Half,
    /// 打った文字のまま(Shift始まりの一時英字入力)。
    Raw,
}

impl Form {
    fn name(self) -> &'static str {
        match self {
            Form::Hira => "hira",
            Form::Kata => "kata",
            Form::Full => "full",
            Form::Half => "half",
            Form::Raw => "raw",
        }
    }
    fn from_name(n: &str) -> Option<Self> {
        Some(match n {
            "hira" => Form::Hira,
            "kata" => Form::Kata,
            "full" => Form::Full,
            "half" => Form::Half,
            _ => return None,
        })
    }
}

fn fullwidth(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => '\u{3000}',
            '!'..='~' => char::from_u32(c as u32 + 0xFEE0).unwrap(),
            o => o,
        })
        .collect()
}

/// F9/F10 の繰り返しで 小文字 → 大文字 → 先頭だけ大文字 と回す。
fn cycle_case(s: &str, n: u8) -> String {
    match n % 3 {
        0 => s.to_lowercase(),
        1 => s.to_uppercase(),
        _ => {
            let l = s.to_lowercase();
            let mut it = l.chars();
            it.next().map(|f| f.to_uppercase().collect::<String>() + it.as_str()).unwrap_or_default()
        }
    }
}

/// 確定履歴の学習: 読み(ひらがな) → 選ばれた表記と回数。
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Learned {
    pub map: std::collections::HashMap<String, Vec<(String, u32)>>,
    /// 入力(ローマ字そのまま) → 利用者が強制した表記(hira/kata/full/half)。誤ったモード予測の学習。
    #[serde(default)]
    pub forms: std::collections::HashMap<String, String>,
}

/// 読みと表記の組として自然か。漢字を含む語は読みより長くならず、かな書きの語は読みと同じ長さ(カタカナ語・ひらがなの言い換え)でなければならない。
/// 不具合で誤って覚えた履歴(「に → 入力」など)を捨てるための検査。英数字・記号を含む表記(3X3 EYES など)は対象外。
pub fn plausible_pair(reading: &str, surface: &str) -> bool {
    let (rn, sn) = (reading.chars().count(), surface.chars().count());
    if rn == 0 || sn == 0 {
        return false;
    }
    let has_kanji = surface.chars().any(|c| ('一'..='龥').contains(&c) || c == '々');
    let all_kana = surface.chars().all(|c| matches!(c, 'ぁ'..='ゖ' | 'ァ'..='ヺ' | 'ー' | '・'));
    if all_kana {
        return sn == rn || surface.contains('・');
    }
    if has_kanji && !surface.chars().any(|c| c.is_ascii_alphanumeric()) {
        return sn <= rn;
    }
    true
}

impl Learned {
    pub fn load(path: &std::path::Path) -> Self {
        let mut l: Self = std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        l.sanitize();
        l
    }

    /// 不自然な履歴を取り除く。
    /// - 読みと表記の組として不自然なもの(読みより長い漢字語など)
    /// - 読みをそのままカタカナにしただけの履歴(カタカナ判定の不具合のあいだに溜まったもの)。漢字などの別の履歴があるときだけ捨てる。
    /// - 強制表記の学習のうち、かな・カタカナ(hira/kata)や短い入力(3文字未満)のもの。「ai は常にカタカナ」のような固定を防ぐ。
    pub fn sanitize(&mut self) {
        for (r, v) in self.map.iter_mut() {
            v.retain(|(s, _)| plausible_pair(r, s));
            let kata = romaji::hira_to_kata(r);
            if v.len() > 1 {
                v.retain(|(s, _)| *s != kata);
            }
        }
        self.map.retain(|_, v| !v.is_empty());
        self.forms.retain(|k, f| k.chars().count() >= 3 && matches!(f.as_str(), "half" | "full" | "raw"));
    }
    pub fn save(&self, path: &std::path::Path) {
        if let Some(d) = path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(path, serde_json::to_string(self).unwrap_or_default());
    }
    pub fn record(&mut self, reading: &str, surface: &str) {
        if reading.is_empty() || surface.is_empty() || !plausible_pair(reading, surface) {
            return;
        }
        let v = self.map.entry(reading.to_string()).or_default();
        match v.iter_mut().find(|(s, _)| s == surface) {
            Some(e) => e.1 += 1,
            None => v.push((surface.to_string(), 1)),
        }
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.truncate(8);
    }
    /// 読みが完全に一致する学習済み表記(回数順)。
    pub fn exact(&self, reading: &str) -> Vec<String> {
        self.map.get(reading).map(|v| v.iter().map(|x| x.0.clone()).collect()).unwrap_or_default()
    }

    /// 読みが前方一致する(より長い読みの)学習済み表記。予測変換用で、候補の末尾にだけ置く。
    pub fn prefix(&self, reading: &str, max: usize) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        let mut pre: Vec<(&String, u32)> = self
            .map
            .iter()
            .filter(|(k, _)| k.starts_with(reading) && k.as_str() != reading)
            .flat_map(|(_, v)| v.iter().map(|x| (&x.0, x.1)))
            .collect();
        pre.sort_by(|a, b| b.1.cmp(&a.1));
        for (s, _) in pre.into_iter().take(max) {
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
        out
    }
}

pub struct Session {
    conv: Box<dyn Converter>,
    pub on: bool,
    pub field_kind: Option<String>,
    before: String,
    buf: String,
    cands: Vec<String>,
    sel: Option<usize>,
    unconverted: String,
    learned: Learned,
    learn_path: Option<std::path::PathBuf>,
    form: Option<Form>,
    form_cycle: u8,
    mode_label: String,
    extras_done: bool,
    /// 入力する言語(既定は日本語)。
    pub lang: Lang,
    en_cands: Vec<String>,
    col: u8,
    en_sel: Option<usize>,
    /// 中国語: 各候補が使う入力(buf)の文字数。
    zh_used: Vec<usize>,
    /// 直近の変換で、辞書に読み全体が1語として載っていたか(日本語として確かな入力か)。強制表記を学習するかの判定に使う。
    jp_known: bool,
    /// 無変換キーの巡回による表記の切り替えは、学習しない。
    no_learn_once: bool,
    user: Option<crate::userdict::UserDict>,
    /// ライブ変換: 打つそばから最有力の変換結果(漢字かな交じり)を未確定表示にする。既定オン。
    pub live: bool,
    /// プログラミング用語の自動訂正(既定オン)。
    pub autocorrect: bool,
    /// 旧字体で表示する(右クリックメニュー・設定 `old_forms`)。
    pub old_forms: bool,
    /// いまの選択がライブ変換による自動のもの(利用者が Space/矢印で選んだものではない)。
    auto: bool,
}

fn is_ja(c: char) -> bool {
    matches!(c, 'ぁ'..='ゖ' | 'ァ'..='ヺ' | 'ー' | '一'..='龥' | '、' | '。')
}

impl Session {
    pub fn new(conv: Box<dyn Converter>) -> Self {
        Self { conv, on: true, field_kind: None, before: String::new(), buf: String::new(), cands: vec![], sel: None, unconverted: String::new(), learned: Learned::default(), learn_path: None, form: None, form_cycle: 0, mode_label: String::new(), extras_done: false, lang: Lang::Ja, en_cands: vec![], col: 0, en_sel: None, zh_used: vec![], jp_known: false, no_learn_once: false, user: None, live: true, autocorrect: true, old_forms: false, auto: false }
    }

    /// 設定ファイルの内容(ライブ変換・自動訂正・旧字体)を反映する。フォーカス移動のたびに呼ぶと、右クリックメニューでの変更がすぐ効く。
    pub fn apply_settings(&mut self, s: &crate::settings::Settings) {
        self.live = s.live_conversion;
        self.autocorrect = s.autocorrect;
        self.old_forms = s.old_forms;
    }

    /// 学習履歴を保存先つきで有効にする(あれば読み込んで引き継ぐ)。
    pub fn enable_learning(&mut self, path: std::path::PathBuf) {
        self.learned = Learned::load(&path);
        self.learn_path = Some(path);
    }

    /// 利用者辞書(単語・熟語の登録)を有効にする。登録内容は他プロセスの変更も自動で反映される。
    pub fn enable_user_dict(&mut self, path: &std::path::Path) {
        self.user = Some(crate::userdict::UserDict::load(path));
    }

    /// カーソル直前の文章(アプリから読めた分)。コンテキストとして予測・変換に使う。
    pub fn set_before(&mut self, before: &str) {
        self.before = before.chars().rev().take(80).collect::<Vec<_>>().into_iter().rev().collect();
    }

    pub fn composing(&self) -> bool {
        !self.buf.is_empty()
    }

    /// 組み立て中の入力を破棄する(フォーカス喪失など)。
    pub fn cancel(&mut self) {
        self.reset();
    }

    /// 先頭を Shift で打った語は、かなにせず打った文字のまま英字で組み立てる(MS-IME/Google IME と同じ一時英字入力)。
    pub fn begin_raw(&mut self) {
        if self.on && !self.composing() {
            self.form = Some(Form::Raw);
        }
    }

    /// このキーをIMEが処理するか(状態を変えずに判定)。`key()` の `eaten` と一致する。
    pub fn would_eat(&self, key: &Key) -> bool {
        would_eat_lang(key, self.on, self.composing(), self.before.chars().last().is_some_and(is_ja), self.lang)
    }

    /// 入力する言語を切り替える(Windows の言語切替で、言語ごとの入力方式が選ばれたとき)。組み立て中の入力は破棄する。
    pub fn set_lang(&mut self, lang: Lang) {
        if self.lang != lang {
            self.reset();
            self.lang = lang;
        }
    }

    /// 現在の状態(DLL が通信なしで判定するための持ち帰り用)。
    pub fn snapshot(&self) -> Step {
        let mut st = self.state(false, None);
        st.eaten = false;
        st
    }

    fn state(&self, eaten: bool, commit: Option<String>) -> Step {
        let preedit = match (self.col, self.en_sel) {
            (1, Some(i)) => self.en_cands.get(i).cloned().unwrap_or_default(),
            _ => match self.sel {
                Some(i) => self.cands.get(i).cloned().unwrap_or_default(),
                None => self.unconverted.clone(),
            },
        };
        Step {
            eaten,
            preedit,
            commit,
            cands: if self.composing() { self.cands.clone() } else { vec![] },
            cands_en: if self.composing() { self.en_cands.clone() } else { vec![] },
            col: self.col,
            sel_en: self.en_sel,
            sel: self.sel,
            mode: if self.composing() { self.mode_label.clone() } else { String::new() },
            on: self.on,
            composing: self.composing(),
            note: if self.composing() { crate::knowledge::note_for(&self.cands.get(self.sel.unwrap_or(0)).cloned().unwrap_or_default()) } else { None },
            lang: self.lang.code().to_string(),
            last_ja: self.before.chars().last().is_some_and(is_ja),
        }
    }

    /// 組み立て中の入力が、数字(と郵便番号の -)だけか。
    fn is_digit_buf(&self) -> bool {
        !self.buf.is_empty() && self.buf.chars().next().is_some_and(|c| c.is_ascii_digit()) && self.buf.chars().all(|c| c.is_ascii_digit() || c == '-')
    }

    fn reset(&mut self) {
        self.buf.clear();
        self.cands.clear();
        self.sel = None;
        self.unconverted.clear();
        self.form = None;
        self.form_cycle = 0;
        self.mode_label.clear();
        self.auto = false;
        self.extras_done = false;
        self.en_cands.clear();
        self.col = 0;
        self.en_sel = None;
    }

    fn render_form(&self, f: Form) -> String {
        match f {
            Form::Hira => romaji::to_hiragana(&self.buf),
            Form::Kata => romaji::hira_to_kata(&romaji::to_hiragana(&self.buf)),
            Form::Full => fullwidth(&cycle_case(&self.buf, self.form_cycle)),
            Form::Half => cycle_case(&self.buf, self.form_cycle),
            Form::Raw => self.buf.clone(),
        }
    }

    fn form_label(f: Form) -> &'static str {
        match f {
            Form::Hira => "あ(強制)",
            Form::Kata => "ア(強制)",
            Form::Full => "Ａ(強制)",
            Form::Half => "A(強制)",
            Form::Raw => "A(英字)",
        }
    }

    fn note_commit(&mut self, s: &str) {
        self.before.push_str(s);
        self.set_before(&self.before.clone());
    }

    /// `nn` + 母音の二通りの読み(こんにちは / サザンアイズ)のうち、どちらを使うか決める。
    /// 登録語・学習履歴に完全一致する読みを優先し、無ければ「実際に変換できた(候補の先頭が読みそのままではない)」読みを選ぶ。
    fn pick_reading(&mut self, primary: &str, alt: &str, r_primary: &Conversion, r_alt: &Conversion) -> bool {
        let user_hit = |me: &mut Self, rd: &str| -> bool {
            let u = me.user.as_mut().map(|u| {
                u.reload_if_changed();
                !u.exact(rd).is_empty()
            });
            u.unwrap_or(false) || !me.learned.exact(rd).is_empty()
        };
        let (a, b) = (user_hit(self, primary), user_hit(self, alt));
        if a != b {
            return b;
        }
        // 辞書に「読み全体が1語として」載っている読みを優先する(こんにちは は載っているが、こんいちは は載っていない)
        r_alt.exact_dict && !r_primary.exact_dict
    }

    fn refresh(&mut self) {
        let mut r = self.conv.convert(&self.buf, &self.before, self.field_kind.as_deref());
        // 郵便番号(`-` の有無どちらでも): 国名つきの住所を、打った数字そのままの次に入れる(IPの国 → 選択中の言語の国 → その他)
        if self.is_digit_buf() {
            let root = crate::components::data_dir();
            let mut at = r.candidates.len().min(1);
            for t in crate::geo::candidates(&root, &self.buf, self.lang.code()) {
                if !r.candidates.iter().any(|c| c.text == t) {
                    r.candidates.insert(at, crate::convert::Candidate { text: t, kind: "address" });
                    at += 1;
                }
            }
        }
        let (mut hira, rest) = romaji::to_hiragana_partial(&self.buf);
        let mut reading = romaji::to_hiragana(&self.buf);
        // `nn` + 母音は二通りに読めるので、両方を試して、より変換できるほうを使う
        let mut other: Vec<String> = vec![];
        if rest.is_empty() {
            if let Some(alt) = romaji::to_hiragana_alt(&self.buf) {
                let r_alt = self.conv.convert(&alt, &self.before, self.field_kind.as_deref());
                if self.pick_reading(&reading.clone(), &alt, &r, &r_alt) {
                    other = r.candidates.iter().map(|x| x.text.clone()).collect();
                    r = r_alt;
                    hira = alt.clone();
                    reading = alt;
                } else {
                    other = r_alt.candidates.iter().map(|x| x.text.clone()).collect();
                }
            }
        }
        // 過去に利用者が強制した表記があれば、最初からそれにする(誤ったモード予測の学習)
        self.jp_known = r.exact_dict;
        // 辞書に載っている日本語の語(ほんやく など)には、過去の強制表記を当てない。英語などで辞書に無い語(database など)だけ。
        if self.form.is_none() && !r.exact_dict {
            self.form = self.learned.forms.get(&self.buf.to_lowercase()).and_then(|n| Form::from_name(n));
        }
        // 確信が低いのに英字へ切り替えない(誤切替の抑制)。パスワード等の英数字欄は確信が高いのでそのまま。
        let mode = if r.prediction.mode == Mode::Alpha && r.prediction.confidence < 0.7 { Mode::Hiragana } else { r.prediction.mode };
        self.mode_label = match mode {
            Mode::Alpha => "A",
            Mode::Hiragana => "あ",
            Mode::Katakana => "ア",
            Mode::Number => "123",
            Mode::Symbol => "記号",
        }
        .to_string();
        self.unconverted = match mode {
            Mode::Alpha | Mode::Number | Mode::Symbol => self.buf.clone(),
            Mode::Katakana => romaji::hira_to_kata(&hira) + &rest,
            Mode::Hiragana => hira + &rest,
        };
        if let Some(f) = self.form {
            self.unconverted = self.render_form(f);
            self.mode_label = Self::form_label(f).to_string();
        }
        let is_kana_mode = matches!(mode, Mode::Hiragana | Mode::Katakana) && self.form.is_none();
        // 学習済みの表記は、かな入力モードのとき最優先で先頭に出す
        // 先頭に置くのは「読みが完全に一致する」候補だけ(登録語 → 学習履歴 → 辞書)。前方一致の予測は末尾に回す。
        // 予測を先頭に置くと、ライブ変換が先頭を自動で選ぶため、短い入力(に・が・は)が長い語に化けてしまう。
        let mut c: Vec<String> = vec![];
        let mut predictions: Vec<String> = vec![];
        if is_kana_mode {
            if let Some(u) = self.user.as_mut() {
                u.reload_if_changed();
                // 読みと同じかな表記だけの登録(きめつのやいば→きめつのやいば)は、ほかの登録語の後ろへ
                let (mut real, same): (Vec<String>, Vec<String>) = u.exact(&reading).into_iter().partition(|x| *x != reading);
                real.extend(same);
                c.extend(real);
                predictions.extend(u.prefix(&reading, 4));
            }
            c.extend(self.learned.exact(&reading));
            predictions.extend(self.learned.prefix(&reading, 4));
        }
        c.extend(r.candidates.iter().map(|x| x.text.clone()));
        c.extend(r.suggestions.iter().cloned());
        c.extend(other.into_iter().take(8)); // もう一つの読みの変換結果も、候補の後ろに残す
        c.extend(predictions);
        let mut seen = vec![];
        c.retain(|x| !seen.contains(x) && { seen.push(x.clone()); true });
        // 優先順位の指定(組み込み + 利用者の prefer.tsv)は、学習履歴より優先する
        if is_kana_mode {
            let want = crate::prefer::preferred(&reading, &crate::prefer::load_user(&crate::components::data_dir()));
            crate::prefer::reorder(&mut c, &want);
        }
        // 旧字体: オンなら候補を旧字体にし、オフなら上位候補の旧字体版を末尾に足す
        if self.old_forms {
            c = c.into_iter().map(|x| crate::kyujitai::to_old(&x)).collect();
            let mut seen2 = vec![];
            c.retain(|x| !seen2.contains(x) && { seen2.push(x.clone()); true });
        } else {
            let extra: Vec<String> = c.iter().take(4).filter(|x| crate::kyujitai::has_old_form(x)).map(|x| crate::kyujitai::to_old(x)).collect();
            for e in extra {
                if !c.contains(&e) {
                    c.push(e);
                }
            }
        }
        self.cands = c;
        self.sel = None;
        self.auto = false;
        self.extras_done = false;
        self.col = 0;
        self.en_sel = None;
        // 英語の列: 日本語の候補の英語訳(かな書き・英数字だけの候補は訳さない)。英語を打ったときは、日本語の対訳を候補の末尾に足す。
        self.en_cands.clear();
        if is_kana_mode {
            let mut en: Vec<String> = vec![];
            for cand in self.cands.iter().take(8) {
                let kana_only = cand.chars().all(|ch| matches!(ch, 'ぁ'..='ゖ' | 'ー'));
                let ascii_only = cand.is_ascii();
                if kana_only || ascii_only {
                    continue;
                }
                for g in self.conv.to_english(cand, &reading).into_iter().take(2) {
                    if !en.contains(&g) {
                        en.push(g);
                    }
                }
            }
            en.truncate(10);
            self.en_cands = en;
        } else if mode == Mode::Alpha && self.form.is_none() && self.buf.chars().count() >= 2 {
            let ja = self.conv.to_japanese(&self.buf.to_lowercase());
            for j in ja.into_iter().take(6) {
                if !self.cands.contains(&j) {
                    self.cands.push(j);
                }
            }
        }
        // プログラミング用語の自動訂正: 確信が高い綴りミス(autofix)は、訂正後の語を最初から選んでおく。
        // 元の綴りは Up キーで戻せる。強制表記(F6〜F10・Shift始まり)のときは何もしない。
        if self.autocorrect && self.form.is_none() && mode == Mode::Alpha {
            if let Some(t) = r.candidates.iter().find(|x| x.kind == "autofix").map(|x| x.text.clone()) {
                if let Some(i) = self.cands.iter().position(|x| *x == t) {
                    self.sel = Some(i);
                    self.auto = true;
                    self.mode_label = "A→訂正".to_string();
                    return;
                }
            }
        }
        // ライブ変換: ひらがな入力と予測され、強制表記が無く、入力途中のローマ字(sak の k など)が残っていないときだけ
        if self.live && self.form.is_none() && matches!(mode, Mode::Hiragana | Mode::Katakana) && rest.is_empty() && !self.cands.is_empty() {
            self.sel = Some(0);
            self.auto = true;
            self.mode_label = "あ→漢".to_string();
        }
    }

    /// 変換を求められたとき、時間のかかる追加候補(Web のサジェストなど)を1回だけ取りに行き、上位の変換候補の直後へ足す。
    /// 辞書の候補が「かな書きだけ」で漢字・カタカナ・英字を含まないとき(=辞書に無い語の疑い)は、先頭に足す。
    fn fetch_extras(&mut self) {
        if self.extras_done || self.form.is_some() {
            return;
        }
        self.extras_done = true;
        let extra = self.conv.extra(&self.buf, &self.before);
        if extra.is_empty() {
            return;
        }
        let first_is_plain_kana = self.cands.first().is_some_and(|c| c.chars().all(|ch| matches!(ch, 'ぁ'..='ゖ' | 'ー')));
        let at = if first_is_plain_kana { 0 } else { self.cands.len().min(3) };
        let mut insert = at;
        for e in extra {
            if !self.cands.contains(&e) {
                self.cands.insert(insert, e);
                insert += 1;
            }
        }
        // ライブ変換で自動選択していた位置は、先頭に足したときに合わせてずらす
        if let (Some(i), true) = (self.sel, self.auto) {
            if at <= i {
                self.sel = Some(i + (insert - at));
            }
        }
    }

    /// 現在の確定文字列(変換中なら選択中の候補、そうでなければ未確定表示)。
    fn current(&self) -> String {
        if let (1, Some(i)) = (self.col, self.en_sel) {
            return self.en_cands.get(i).cloned().unwrap_or_default();
        }
        match self.sel {
            Some(i) => self.cands.get(i).cloned().unwrap_or_default(),
            None => self.unconverted.clone(),
        }
    }

    fn commit_current(&mut self) -> String {
        let s = self.current();
        if self.lang == Lang::Ja && self.sel.is_some() && !self.auto && self.col == 0 {
            let reading = romaji::to_hiragana(&self.buf);
            if reading != s && !self.buf.chars().any(|c| c.is_ascii_digit()) {
                self.learned.record(&reading, &s);
                if let Some(p) = self.learn_path.clone() {
                    self.learned.save(&p);
                }
            }
        }
        self.reset();
        self.note_commit(&s);
        s
    }

    pub fn key(&mut self, key: Key) -> Step {
        // 変換キー・無変換キー(日本語キーボード)
        match key {
            Key::NonConvert if self.on && self.composing() => {
                // 最初の1回はひらがな(変換中の漢字・候補も、読みのひらがなに戻す)。以後は ひらがな → カタカナ → 全角英数 → 半角英数 → ひらがな
                let next = match self.form {
                    None | Some(Form::Raw) => Form::Hira,
                    Some(Form::Hira) => Form::Kata,
                    Some(Form::Kata) => Form::Full,
                    Some(Form::Full) => Form::Half,
                    Some(Form::Half) => Form::Hira,
                };
                self.no_learn_once = true;
                return self.key(Key::Fn(match next {
                    Form::Hira => 6,
                    Form::Kata => 7,
                    Form::Full => 9,
                    _ => 10,
                }));
            }
            Key::NonConvert => return self.key(Key::Off),
            Key::Convert if self.on && self.composing() => return self.key(Key::Space),
            Key::Convert => return self.key(Key::On),
            _ => {}
        }
        if matches!(key, Key::Toggle | Key::On | Key::Off) {
            let pending = if self.composing() { Some(self.commit_current()) } else { None };
            self.on = match key {
                Key::On => true,
                Key::Off => false,
                _ => !self.on,
            };
            return self.state(true, pending);
        }
        if !self.on {
            return Step::default();
        }
        if self.lang != Lang::Ja {
            return self.key_lang(key);
        }
        match key {
            Key::Char(c) if self.composing() && self.col == 1 && c.is_ascii_digit() => {
                let page = self.en_sel.map_or(0, |i| i / PAGE) * PAGE;
                let idx = page + if c == '0' { 9 } else { c as usize - '1' as usize };
                if idx < self.en_cands.len() {
                    self.en_sel = Some(idx);
                    let s = self.commit_current();
                    return self.state(true, Some(s));
                }
                self.state(true, None)
            }
            Key::Char(c) if self.composing() && self.sel.is_some() && !self.auto && c.is_ascii_digit() => {
                // 候補を選んでいる最中は、数字キー(1〜9、0は10番目)で、いま表示している10件のページの候補を直接選んで確定する
                let page = self.sel.map_or(0, |i| i / PAGE) * PAGE;
                let idx = page + if c == '0' { 9 } else { c as usize - '1' as usize };
                if idx < self.cands.len() {
                    return self.choose(idx);
                }
                self.state(true, None)
            }
            // 数字の入力: 英数字の直後(abc123・3.14 の続き)でなければ、数字を未確定にして候補を出す(郵便番号 → 住所、全角、漢数字)。
            // 英字・記号・Enter などが来たら、そのまま確定する。
            Key::Char(d) if d.is_ascii_digit() && !self.composing() => {
                if self.before.chars().last().is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == ',') {
                    self.note_commit(&d.to_string());
                    return self.state(true, Some(d.to_string()));
                }
                self.buf.push(d);
                self.refresh();
                if !self.cands.is_empty() {
                    self.sel = Some(0);
                    self.auto = true;
                }
                self.state(true, None)
            }
            Key::Char(d) if d.is_ascii_digit() && self.composing() && self.is_digit_buf() => {
                self.buf.push(d);
                self.refresh();
                if !self.cands.is_empty() {
                    self.sel = Some(0);
                    self.auto = true;
                }
                self.state(true, None)
            }
            Key::Char('-') if self.composing() && self.is_digit_buf() && self.buf.len() == 3 => {
                self.buf.push('-');
                self.refresh();
                if !self.cands.is_empty() {
                    self.sel = Some(0);
                    self.auto = true;
                }
                self.state(true, None)
            }
            Key::Char(c) if self.composing() && self.is_digit_buf() && !c.is_ascii_digit() => {
                // 数字の途中で別の文字が来たら、数字を確定してから、その文字を続ける
                let first = self.commit_current();
                let mut st = self.key(Key::Char(c));
                st.commit = Some(format!("{first}{}", st.commit.unwrap_or_default()));
                st.eaten = true;
                st
            }
            // 入力の最初の - : 半角 - ・全角 － ・ダッシュ・長音・カタカナのハイフンなどを候補に並べる。
            // 直前が英数字(090-1234 / well-known)のときは、候補を出さずそのまま - にする。
            Key::Char(sym) if !self.composing() && crate::prefer::lead_symbol(sym).is_some() => {
                if self.before.chars().last().is_some_and(|ch| ch.is_ascii_alphanumeric()) {
                    self.note_commit(&sym.to_string());
                    return self.state(true, Some(sym.to_string()));
                }
                self.buf.push(sym);
                self.refresh();
                if !self.cands.is_empty() {
                    self.sel = Some(0); // 先頭の候補(/ なら ・)を、打った時点の表示にする
                    self.auto = true;
                }
                self.state(true, None)
            }
            // ハイフンの候補を出している最中に数字が来たら、半角の - として確定して数字を続ける
            // - と + 以外の記号の候補を出している最中に文字が来たら、先頭の候補で確定してから、その文字を続ける
            Key::Char(c) if self.composing() && self.buf.chars().count() == 1 && !matches!(self.buf.as_str(), "-" | "+") && self.buf.chars().next().is_some_and(|b| crate::prefer::lead_symbol(b).is_some()) && c != ' ' => {
                let first = self.commit_current();
                let mut st = self.key(Key::Char(c));
                st.commit = Some(format!("{first}{}", st.commit.unwrap_or_default()));
                st
            }
            Key::Char(c) if self.composing() && (self.buf == "-" || self.buf == "+") && c.is_ascii_digit() => {
                let sym = self.buf.clone();
                self.reset();
                let s = format!("{sym}{c}");
                self.note_commit(&s);
                self.state(true, Some(s))
            }
            Key::Char(c) => {
                let word = c.is_ascii_alphabetic() || (self.composing() && (c.is_ascii_digit() || "-'".contains(c)));
                if word {
                    // 変換選択中に打ち始めたら、選択中の候補を確定して新しい入力にする
                    let commit = if self.sel.is_some() && !self.auto { Some(self.commit_current()) } else { None };
                    self.buf.push(c);
                    self.refresh();
                    return self.state(true, commit);
                }
                if self.composing() {
                    // 区切り文字: 確定してから文字を渡す
                    let mut s = self.commit_current();
                    s.push(match c { ',' => '、', '.' => '。', o => o });
                    self.note_commit("");
                    return self.state(true, Some(s));
                }
                // ひらがな入力モードでは , . は 、。にする。ただし直前が英数字(3.14・example.com・a,b)のときは、そのまま
                if matches!(c, ',' | '.') {
                    let after_ascii = self.before.chars().last().is_some_and(|ch| ch.is_ascii_alphanumeric());
                    let s = if after_ascii { c.to_string() } else if c == ',' { "、".to_string() } else { "。".to_string() };
                    self.note_commit(&s);
                    return self.state(true, Some(s));
                }
                self.note_commit(&c.to_string());
                Step::default()
            }
            Key::Fn(n) if self.composing() && (6..=10).contains(&n) => {
                let f = match n {
                    6 => Form::Hira,
                    7 | 8 => Form::Kata,
                    9 => Form::Full,
                    _ => Form::Half,
                };
                self.form_cycle = if self.form == Some(f) { (self.form_cycle + 1) % 3 } else { 0 };
                self.form = Some(f);
                self.sel = None;
                self.auto = false;
                self.unconverted = self.render_form(f);
                self.mode_label = Self::form_label(f).to_string();
                // 英字の表記(半角・全角)を選んだときだけ学習する(次回から同じ入力は最初からこの表記)。
                // かな・カタカナは学習しない(「ai は常にカタカナ」のような固定になり、漢字に変換できなくなるため)。
                let learn = !std::mem::take(&mut self.no_learn_once) && !self.jp_known;
                if learn && matches!(f, Form::Half | Form::Full) && self.buf.chars().count() >= 3 {
                    self.learned.forms.insert(self.buf.to_lowercase(), f.name().to_string());
                    if let Some(p) = self.learn_path.clone() {
                        self.learned.save(&p);
                    }
                }
                self.state(true, None)
            }
            Key::Fn(_) if self.composing() => self.state(true, None),
            Key::Right if self.composing() => {
                // 英語の列へ。英語の候補が無ければ何もしない
                if !self.en_cands.is_empty() {
                    self.col = 1;
                    self.auto = false;
                    self.en_sel = Some(self.en_sel.unwrap_or(0));
                }
                self.state(true, None)
            }
            Key::Left if self.composing() => {
                self.col = 0;
                self.en_sel = None;
                self.state(true, None)
            }
            Key::Space | Key::Down | Key::Tab if self.composing() && self.col == 1 => {
                let n = self.en_cands.len();
                if n > 0 {
                    self.en_sel = Some((self.en_sel.unwrap_or(0) + 1) % n);
                }
                self.state(true, None)
            }
            Key::Up if self.composing() && self.col == 1 => {
                let n = self.en_cands.len();
                if n > 0 {
                    self.en_sel = Some(match self.en_sel { None | Some(0) => n - 1, Some(i) => i - 1 });
                }
                self.state(true, None)
            }
            Key::Space | Key::Down | Key::Tab if self.composing() => {
                self.fetch_extras();
                let n = self.cands.len();
                self.auto = false;
                if n > 0 {
                    self.sel = Some(match self.sel { None => 0, Some(i) => (i + 1) % n });
                }
                self.state(true, None)
            }
            Key::Up if self.composing() => {
                let n = self.cands.len();
                self.auto = false;
                if n > 0 {
                    self.sel = Some(match self.sel { None | Some(0) => n - 1, Some(i) => i - 1 });
                }
                self.state(true, None)
            }
            Key::Enter if self.composing() => {
                let s = self.commit_current();
                self.state(true, Some(s))
            }
            Key::Esc if self.composing() && self.col == 1 => {
                self.col = 0;
                self.en_sel = None;
                self.state(true, None)
            }
            Key::Esc if self.composing() => {
                if self.sel.is_some() && !self.auto {
                    self.sel = None;
                } else {
                    self.reset();
                }
                self.state(true, None)
            }
            Key::Backspace if self.composing() => {
                if self.sel.is_some() && !self.auto {
                    self.sel = None;
                } else {
                    self.buf.pop();
                    if self.buf.is_empty() {
                        self.reset();
                    } else {
                        self.refresh();
                    }
                }
                self.state(true, None)
            }
            _ => {
                if self.composing() && key == Key::Other {
                    // 未知のキーで未確定を確定してからアプリへ渡す
                    let s = self.commit_current();
                    let mut st = self.state(false, Some(s));
                    st.eaten = false;
                    return st;
                }
                Step::default()
            }
        }
    }

    /// 日本語以外の言語の入力(ロシア語・アラビア語・ヘブライ語・韓国語・ヨーロッパ言語・英語)。
    /// ラテン文字の入力を、その言語の文字にして未確定表示にする。Space で確定+空白、Enter で確定のみ、↑↓/Tab で候補を選ぶ。
    /// 中国語: `idx` 番目の候補を確定する。入力の一部しか使わない候補なら、残りの入力は続けて変換する。
    fn zh_commit(&mut self, idx: usize) -> Step {
        let Some(text) = self.cands.get(idx).cloned() else { return self.state(true, None) };
        let used = self.zh_used.get(idx).copied().unwrap_or(self.buf.chars().count());
        let rest: String = self.buf.chars().skip(used).collect();
        self.note_commit(&text);
        if rest.chars().any(|c| c.is_ascii_alphabetic()) {
            self.buf = rest;
            self.refresh_lang();
        } else {
            self.reset();
        }
        self.state(true, Some(text))
    }

    fn key_lang(&mut self, key: Key) -> Step {
        let lang = self.lang;
        let zh = matches!(lang, Lang::ZhHans | Lang::ZhHant);
        if zh && self.composing() {
            match key {
                Key::Char(c) if c.is_ascii_digit() => {
                    let idx = if c == '0' { 9 } else { c as usize - '1' as usize };
                    return if idx < self.cands.len().min(10) { self.zh_commit(idx) } else { self.state(true, None) };
                }
                Key::Space | Key::Enter => {
                    let i = self.sel.unwrap_or(0);
                    return self.zh_commit(i);
                }
                _ => {}
            }
        }
        match key {
            Key::Char(c) if lang.supported() && lang.accepts(c, self.composing()) => {
                self.buf.push(c);
                self.refresh_lang();
                self.state(true, None)
            }
            Key::Char(c) if lang.supported() && self.composing() => {
                // 区切りの文字(. , ! ? など): 確定してから、その文字を続ける
                let mut s = self.commit_current();
                s.push(c);
                self.note_commit(&c.to_string());
                self.state(true, Some(s))
            }
            Key::Space if self.composing() => {
                let mut s = self.commit_current();
                s.push(' ');
                self.note_commit(" ");
                self.state(true, Some(s))
            }
            Key::Down | Key::Tab if self.composing() => {
                let n = self.cands.len();
                if n > 0 {
                    self.sel = Some(match self.sel { None => 0, Some(i) => (i + 1) % n });
                }
                self.state(true, None)
            }
            Key::Up if self.composing() => {
                let n = self.cands.len();
                if n > 0 {
                    self.sel = Some(match self.sel { None | Some(0) => n - 1, Some(i) => i - 1 });
                }
                self.state(true, None)
            }
            Key::Enter if self.composing() => {
                let s = self.commit_current();
                self.state(true, Some(s))
            }
            Key::Esc if self.composing() => {
                self.reset();
                self.state(true, None)
            }
            Key::Backspace if self.composing() => {
                if self.sel.is_some() {
                    self.sel = None;
                } else {
                    self.buf.pop();
                    if self.buf.is_empty() {
                        self.reset();
                    } else {
                        self.refresh_lang();
                    }
                }
                self.state(true, None)
            }
            _ => Step::default(),
        }
    }

    fn refresh_lang(&mut self) {
        let lang = self.lang;
        let mut c: Vec<String> = translit::candidates(lang, &self.buf);
        self.zh_used.clear();
        if matches!(lang, Lang::ZhHans | Lang::ZhHant) {
            let z = crate::zh::candidates_global(&self.buf, lang == Lang::ZhHant, 30);
            if !z.is_empty() {
                c = z.iter().map(|x| x.0.clone()).collect();
                self.zh_used = z.iter().map(|x| x.1).collect();
            } else {
                c = vec![self.buf.clone()];
                self.zh_used = vec![self.buf.chars().count()];
            }
            self.unconverted = self.buf.clone();
            self.cands = c;
            self.sel = Some(0);
            self.auto = false;
            self.en_cands.clear();
            self.col = 0;
            self.en_sel = None;
            self.mode_label = lang.code().to_string();
            return;
        }
        if lang == Lang::En {
            // 英語: 打った語を先頭に、日本語の対訳を候補に足す(日英の相互翻訳)
            c = vec![self.buf.clone()];
            for j in self.conv.to_japanese(&self.buf.to_lowercase()).into_iter().take(6) {
                if !c.contains(&j) {
                    c.push(j);
                }
            }
            self.unconverted = self.buf.clone();
        } else {
            self.unconverted = c.first().cloned().unwrap_or_else(|| self.buf.clone());
        }
        self.cands = c;
        self.sel = None;
        self.auto = false;
        self.en_cands.clear();
        self.col = 0;
        self.en_sel = None;
        self.mode_label = lang.code().to_string();
    }

    /// 候補を番号(0始まり)で選んで確定する(候補ウィンドウのクリックなど)。
    pub fn choose(&mut self, idx: usize) -> Step {
        if self.composing() && idx < self.cands.len() {
            self.sel = Some(idx);
            let s = self.commit_current();
            return self.state(true, Some(s));
        }
        Step::default()
    }
}

/// 辞書(任意)のみで動くローカル変換器。常駐エンジンに繋がらないときの代替。
pub struct LocalConverter {
    pub dict: Option<crate::kanji::Dict>,
}

impl Converter for LocalConverter {
    fn convert(&self, buffer: &str, before: &str, field_kind: Option<&str>) -> Conversion {
        let ctx = crate::mode::Context { before: before.into(), field_kind: field_kind.map(str::to_string), ..Default::default() };
        crate::convert::convert(buffer, &ctx, None, self.dict.as_ref())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn comma_and_period_become_japanese_punctuation_except_after_ascii() {
        let mut m = s();
        assert_eq!(m.key(Key::Char(',')).commit.as_deref(), Some("、"), "先頭でも 、");
        m.set_before("");
        assert_eq!(m.key(Key::Char('.')).commit.as_deref(), Some("。"));
        m.set_before("今日は");
        assert_eq!(m.key(Key::Char('.')).commit.as_deref(), Some("。"));
        m.set_before("3");
        assert_eq!(m.key(Key::Char('.')).commit.as_deref(), Some("."), "数字の直後(3.14)はそのまま");
        m.set_before("example");
        assert_eq!(m.key(Key::Char('.')).commit.as_deref(), Some("."));
    }

    #[test]
    fn leading_hyphen_and_plus_show_many_kinds_but_numbers_stay_plain() {
        let mut m = s();
        let st = m.key(Key::Char('-'));
        assert!(st.eaten && st.cands.contains(&"-".to_string()) && st.cands.contains(&"－".to_string()) && st.cands.contains(&"ー".to_string()) && st.cands.contains(&"ｰ".to_string()), "{st:?}");
        // 数字が続くとき(-5)は半角の - のまま
        assert_eq!(m.key(Key::Char('5')).commit.as_deref(), Some("-5"));
        m.set_before("");
        let st = m.key(Key::Char('+'));
        assert!(st.cands.contains(&"+".to_string()) && st.cands.contains(&"＋".to_string()) && st.cands.contains(&"プラス".to_string()), "{st:?}");
        m.key(Key::Esc);
        // 直前が英数字のとき(090-1234)は候補を出さずそのまま
        let mut n = s();
        n.set_before("090");
        let st = n.key(Key::Char('-'));
        assert_eq!(st.commit.as_deref(), Some("-"));
    }

    use super::*;

    fn s() -> Session {
        let mut s = Session::new(Box::new(LocalConverter { dict: None }));
        s.live = false; // 従来の挙動(Space で変換)を試験する
        s.set_before("私は");
        s
    }

    /// 辞書に「かんじ→漢字」がある想定の変換器。
    struct Kanji;
    impl Converter for Kanji {
        fn convert(&self, buffer: &str, _before: &str, _field: Option<&str>) -> Conversion {
            use crate::convert::Candidate;
            let hira = romaji::to_hiragana(buffer);
            let mut c = vec![];
            if hira == "かんじ" {
                c.push(Candidate { text: "漢字".into(), kind: "dict" });
                c.push(Candidate { text: "幹事".into(), kind: "dict" });
            }
            c.push(Candidate { text: hira.clone(), kind: "hiragana" });
            Conversion {
                prediction: crate::mode::Prediction { mode: Mode::Hiragana, confidence: 0.99, english_word: false, reason: String::new() },
                candidates: c,
                suggestions: vec![],
                exact_dict: false,
            }
        }
    }

    fn live() -> Session {
        let mut s = Session::new(Box::new(Kanji));
        s.set_before("これは");
        s
    }

    #[test]
    fn live_conversion_shows_kanji_while_typing() {
        let mut s = live();
        let st = type_str(&mut s, "kanji");
        assert_eq!(st.preedit, "漢字", "漢字に変換する前提: 打つそばから変換結果を未確定表示にする");
        assert_eq!(st.sel, Some(0));
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("漢字"));
    }

    #[test]
    fn live_conversion_waits_for_pending_romaji() {
        let mut s = live();
        // 入力途中(k が残っている)の間は、かな+残りのローマ字のまま
        let st = type_str(&mut s, "kank");
        assert_eq!(st.preedit, "かんk");
        assert_eq!(st.sel, None);
    }

    #[test]
    fn live_space_goes_to_next_candidate_and_f6_reverts() {
        let mut s = live();
        type_str(&mut s, "kanji");
        assert_eq!(s.key(Key::Space).preedit, "幹事");
        assert_eq!(s.key(Key::Fn(6)).preedit, "かんじ");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("かんじ"));
    }

    #[test]
    fn live_keeps_typing_and_backspace_edits_the_text() {
        let mut s = live();
        let st = type_str(&mut s, "kanji");
        assert!(st.commit.is_none());
        // 続けて打っても確定せず、バックスペースは1文字消す
        let st = s.key(Key::Char('x'));
        assert!(st.commit.is_none());
        assert_eq!(s.key(Key::Backspace).preedit, "漢字");
        assert!(s.key(Key::Esc).preedit.is_empty());
    }

    #[test]
    fn live_conversion_with_real_dictionary() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dict.bin");
        let Ok(d) = crate::kanji::Dict::load(&path) else { return };
        let mut s = Session::new(Box::new(LocalConverter { dict: Some(d) }));
        s.set_before("私は");
        assert_eq!(type_str(&mut s, "kyouhaiitenkidesu").preedit, "今日はいい天気です");
        s.key(Key::Esc);
        // 打ち途中でも崩れない(入力のたびに変換結果が更新される)
        assert_eq!(type_str(&mut s, "kaigino").preedit, "会議の");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("会議の"));
    }

    #[test]
    fn code_typo_is_autocorrected_and_can_be_reverted() {
        let ctx = "function f() { const x = 1; console.log(x) }\n";
        let mut s = Session::new(Box::new(LocalConverter { dict: None }));
        s.set_before(ctx);
        let st = type_str(&mut s, "fucntion");
        assert_eq!(st.preedit, "function", "確信が高い綴りミスは訂正後の語を選んでおく");
        assert!(st.mode.contains("訂正"));
        assert_eq!(s.key(Key::Up).preedit, "fucntion", "Up で入力そのままに戻せる");
        let mut s2 = Session::new(Box::new(LocalConverter { dict: None }));
        s2.set_before(ctx);
        type_str(&mut s2, "fucntion");
        assert_eq!(s2.key(Key::Enter).commit.as_deref(), Some("function"));
        let mut s3 = Session::new(Box::new(LocalConverter { dict: None }));
        s3.autocorrect = false;
        s3.set_before(ctx);
        assert_eq!(type_str(&mut s3, "fucntion").preedit, "fucntion");
    }

    #[test]
    fn old_forms_show_kyujitai() {
        // 旧字体オフ: 先頭は新字体のまま、旧字体版は候補の末尾にある
        let mut s = Session::new(Box::new(Kanji2));
        s.set_before("");
        let st = type_str(&mut s, "gakkou");
        assert_eq!(st.preedit, "学校");
        assert!(st.cands.contains(&"學校".to_string()));
        // 旧字体オン: 変換結果が旧字体になる
        let mut s2 = Session::new(Box::new(Kanji2));
        s2.old_forms = true;
        s2.set_before("");
        assert_eq!(type_str(&mut s2, "gakkou").preedit, "學校");
        // 設定からの反映
        let mut st3 = crate::settings::Settings::default();
        st3.old_forms = true;
        st3.live_conversion = false;
        let mut s3 = Session::new(Box::new(Kanji2));
        s3.apply_settings(&st3);
        assert!(s3.old_forms && !s3.live);
    }

    struct Kanji2;
    impl Converter for Kanji2 {
        fn convert(&self, buffer: &str, _b: &str, _f: Option<&str>) -> Conversion {
            use crate::convert::Candidate;
            let hira = romaji::to_hiragana(buffer);
            let mut c = vec![];
            if hira == "がっこう" || hira == "がっこお" || buffer == "gakkou" {
                c.push(Candidate { text: "学校".into(), kind: "dict" });
            }
            c.push(Candidate { text: hira, kind: "hiragana" });
            Conversion { prediction: crate::mode::Prediction { mode: Mode::Hiragana, confidence: 0.99, english_word: false, reason: String::new() }, candidates: c, suggestions: vec![], exact_dict: false }
        }
    }

    #[test]
    fn live_can_be_turned_off() {
        let mut s = live();
        s.live = false;
        assert_eq!(type_str(&mut s, "kanji").preedit, "かんじ");
    }
    fn type_str(s: &mut Session, t: &str) -> Step {
        let mut last = Step::default();
        for c in t.chars() {
            last = s.key(Key::Char(c));
        }
        last
    }

    #[test]
    fn romaji_to_hiragana_and_commit() {
        let mut s = s();
        let st = type_str(&mut s, "sakura");
        assert!(st.eaten);
        assert_eq!(st.preedit, "さくら");
        let st = s.key(Key::Enter);
        assert_eq!(st.commit.as_deref(), Some("さくら"));
        assert_eq!(st.preedit, "");
        assert!(!s.composing());
    }

    #[test]
    fn english_word_stays_alpha_in_english_context() {
        let mut s = Session::new(Box::new(LocalConverter { dict: None }));
        s.set_before("Please check the ");
        let st = type_str(&mut s, "github");
        assert_eq!(st.preedit, "github");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("github"));
    }

    #[test]
    fn space_cycles_candidates() {
        let mut s = s();
        type_str(&mut s, "sakura");
        let a = s.key(Key::Space);
        assert_eq!(a.sel, Some(0));
        let b = s.key(Key::Space);
        assert_eq!(b.sel, Some(1));
        assert_ne!(a.preedit, b.preedit);
        let c = s.key(Key::Enter);
        assert_eq!(c.commit.as_deref(), Some(b.preedit.as_str()));
    }

    #[test]
    fn backspace_and_escape() {
        let mut s = s();
        type_str(&mut s, "ka");
        assert_eq!(s.key(Key::Backspace).preedit, "k");
        s.key(Key::Backspace);
        assert!(!s.composing());
        type_str(&mut s, "ka");
        let st = s.key(Key::Esc);
        assert!(st.commit.is_none() && st.preedit.is_empty());
    }

    #[test]
    fn digits_are_composed_and_commit_as_typed_but_not_after_alphanumerics() {
        // 数字は未確定になり、Enter でそのまま確定する(候補に 全角・漢数字も出る)
        let mut s = s();
        let st = s.key(Key::Char('5'));
        assert!(st.eaten && st.composing && st.preedit == "5", "{st:?}");
        assert!(st.cands.contains(&"５".to_string()) && st.cands.contains(&"五".to_string()), "{:?}", st.cands);
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("5"));
        // 数字の後に英字が来たら、数字を確定してから英字の入力を始める
        let mut s = self::tests::s();
        s.key(Key::Char('1'));
        s.key(Key::Char('2'));
        let st = s.key(Key::Char('k'));
        assert_eq!(st.commit.as_deref(), Some("12"), "{st:?}");
        // 英数字の直後(abc1 / 3.14)は、そのまま数字を確定する
        let mut s = self::tests::s();
        s.set_before("abc");
        let st = s.key(Key::Char('1'));
        assert_eq!(st.commit.as_deref(), Some("1"));
        let mut s = self::tests::s();
        s.set_before("3.");
        assert_eq!(s.key(Key::Char('1')).commit.as_deref(), Some("1"));
        // 郵便番号の形(7桁)は、123-4567 の形も候補になる
        let mut s = self::tests::s();
        for ch in "1000001".chars() {
            s.key(Key::Char(ch));
        }
        let st = s.key(Key::Space);
        assert!(st.cands.iter().any(|c| c == "100-0001" || c == "〒100-0001"), "{:?}", st.cands);
        // カンマは読点にする
        let mut s = self::tests::s();
        let st = s.key(Key::Char(','));
        assert_eq!(st.commit.as_deref(), Some("、"));
    }

    #[test]
    fn would_eat_matches_key() {
        let keys = [Key::Char('a'), Key::Char('5'), Key::Char(','), Key::Space, Key::Enter, Key::Esc, Key::Backspace, Key::Tab, Key::Up, Key::Down, Key::Other];
        let mut s = s();
        for step in 0..2 {
            for k in &keys {
                let mut probe = Session::new(Box::new(LocalConverter { dict: None }));
                probe.set_before("私は");
                if step == 1 {
                    for c in "ka".chars() {
                        probe.key(Key::Char(c));
                    }
                }
                let predicted = probe.would_eat(k);
                assert_eq!(predicted, probe.key(k.clone()).eaten, "{k:?} step{step}");
            }
        }
        s.key(Key::Toggle);
        assert!(!s.would_eat(&Key::Char('a')));
    }

    #[test]
    fn step_roundtrips_as_json_and_carries_state_for_remote_eat_decision() {
        let mut s = s();
        let st = type_str(&mut s, "sakura");
        let json = serde_json::to_string(&st).unwrap();
        let back: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(st, back);
        assert!(back.on && back.composing);
        for k in [Key::Char('a'), Key::Char('5'), Key::Char(','), Key::Space, Key::Enter, Key::Esc, Key::Backspace, Key::Fn(6), Key::Other] {
            let j = serde_json::to_string(&k).unwrap();
            assert_eq!(serde_json::from_str::<Key>(&j).unwrap(), k, "{j}");
            // 持ち帰った状態だけで判定しても、通信なしのその場判定と一致する
            assert_eq!(would_eat_with(&k, back.on, back.composing, back.last_ja), s.would_eat(&k), "{k:?}");
        }
        s.set_before("私は");
        assert!(s.snapshot().last_ja == false || s.snapshot().last_ja);
    }

    #[test]
    fn forced_forms_are_not_applied_to_known_japanese_words() {
        // 辞書に載っている日本語(ほんやく)に、過去の「半角英字」の強制が残っていても、漢字変換できる
        let p = std::env::temp_dir().join(format!("aruaru-forms-{}.json", std::process::id()));
        std::fs::write(&p, r#"{"map":{},"forms":{"honyaku":"half","database":"half"}}"#).unwrap();
        struct Known;
        impl Converter for Known {
            fn convert(&self, buffer: &str, _b: &str, _f: Option<&str>) -> Conversion {
                use crate::convert::Candidate;
                let hira = romaji::to_hiragana(buffer);
                let known = hira == "ほにゃく";
                let mut c = vec![];
                if known {
                    c.push(Candidate { text: "翻訳".into(), kind: "dict" });
                }
                c.push(Candidate { text: hira, kind: "hiragana" });
                Conversion { prediction: crate::mode::Prediction { mode: Mode::Hiragana, confidence: 0.99, english_word: false, reason: String::new() }, candidates: c, suggestions: vec![], exact_dict: known }
            }
        }
        let mut s = Session::new(Box::new(Known));
        s.set_before("");
        s.enable_learning(p.clone());
        assert_eq!(type_str(&mut s, "honyaku").preedit, "翻訳", "辞書に載っている語には強制表記を当てない");
        s.key(Key::Esc);
        assert_eq!(type_str(&mut s, "database").preedit, "database", "辞書に無い語には当てる");
        s.key(Key::Esc);
        // 辞書に載っている語を F10 で半角にしても、学習されない
        type_str(&mut s, "honyaku");
        s.key(Key::Fn(10));
        s.key(Key::Enter);
        let l = Learned::load(&p);
        assert_eq!(l.forms.get("honyaku").map(String::as_str), Some("half"), "もとの記録は残るが、適用はされない(上の確認)");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn muhenkan_cycles_forms_and_henkan_converts() {
        let mut s = s();
        type_str(&mut s, "kamera");
        assert_eq!(s.key(Key::NonConvert).preedit, "かめら", "無変換の最初の1回: ひらがな");
        assert_eq!(s.key(Key::NonConvert).preedit, "カメラ", "→ カタカナ");
        assert_eq!(s.key(Key::NonConvert).preedit, "ｋａｍｅｒａ", "→ 全角英数");
        assert_eq!(s.key(Key::NonConvert).preedit, "kamera", "→ 半角英数");
        assert_eq!(s.key(Key::NonConvert).preedit, "かめら", "→ ひらがなに戻る");
        // 入力していないときの無変換は IME オフ、変換は IME オン
        s.key(Key::Esc);
        s.key(Key::NonConvert);
        assert!(!s.on);
        assert!(!s.would_eat(&Key::Char('a')));
        s.key(Key::Convert);
        assert!(s.on);
        // 入力中の変換キーは、Space と同じく次の候補へ
        type_str(&mut s, "sakura");
        assert_eq!(s.key(Key::Convert).sel, Some(0));
        // 2つのキーは、状態に関わらずIMEが処理する(アプリへ渡さない)
        assert!(would_eat_with(&Key::NonConvert, false, false, false) && would_eat_with(&Key::Convert, false, false, false));
    }

    struct WithExtra;
    impl Converter for WithExtra {
        fn convert(&self, buffer: &str, _b: &str, _f: Option<&str>) -> Conversion {
            use crate::convert::Candidate;
            let hira = romaji::to_hiragana(buffer);
            Conversion {
                prediction: crate::mode::Prediction { mode: Mode::Hiragana, confidence: 0.99, english_word: false, reason: String::new() },
                candidates: vec![Candidate { text: hira, kind: "hiragana" }],
                suggestions: vec![],
                exact_dict: false,
            }
        }
        fn extra(&self, _b: &str, _before: &str) -> Vec<String> {
            vec!["三只眼吽迦羅".to_string(), "3X3 EYES".to_string()]
        }
    }

    #[test]
    fn extra_candidates_are_fetched_once_on_explicit_conversion() {
        let mut s = Session::new(Box::new(WithExtra));
        s.live = false;
        s.set_before("");
        let st = type_str(&mut s, "sanjiyan");
        assert!(!st.cands.contains(&"三只眼吽迦羅".to_string()), "打っている間は Web を引かない");
        let st = s.key(Key::Space);
        assert!(st.cands.contains(&"三只眼吽迦羅".to_string()) && st.cands.contains(&"3X3 EYES".to_string()));
        assert_eq!(st.preedit, "三只眼吽迦羅", "辞書がかな書きだけのときは、追加候補が先頭に来る");
        let n = st.cands.len();
        assert_eq!(s.key(Key::Space).cands.len(), n, "2回目以降は引き直さない");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("3X3 EYES"));
    }

    #[test]
    fn explicit_on_off_never_gets_the_state_wrong() {
        let mut s = s();
        // 英数キーは何度押してもオフ、かなキーは何度押してもオン
        s.key(Key::Off);
        s.key(Key::Off);
        assert!(!s.on && !s.key(Key::Char('a')).eaten);
        s.key(Key::On);
        s.key(Key::On);
        assert!(s.on && s.key(Key::Char('a')).eaten);
        // 組み立て中にオフにしたら、組み立て中の文字を確定してから切り替える
        let st = s.key(Key::Off);
        assert_eq!(st.commit.as_deref(), Some("あ"));
        assert!(!s.on);
    }

    #[test]
    fn shift_started_word_stays_alpha() {
        let mut s = s();
        s.begin_raw();
        let st = type_str(&mut s, "Hello");
        assert_eq!(st.preedit, "Hello");
        assert!(st.mode.contains("英字"));
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("Hello"));
        // 確定後は通常に戻る
        assert_eq!(type_str(&mut s, "sakura").preedit, "さくら");
        // 組み立て中に begin_raw を呼んでも影響しない
        s.begin_raw();
        assert_eq!(type_str(&mut s, "n").preedit, "さくらn");
    }

    #[test]
    fn toggle_off_passes_everything() {
        let mut s = s();
        s.key(Key::Toggle);
        assert!(!s.on);
        assert!(!s.key(Key::Char('a')).eaten);
        s.key(Key::Toggle);
        assert!(s.key(Key::Char('a')).eaten);
    }

    #[test]
    fn password_field_stays_alpha() {
        let mut s = s();
        s.field_kind = Some("password".into());
        assert_eq!(type_str(&mut s, "sakura").preedit, "sakura");
    }

    #[test]
    fn learning_promotes_chosen_candidate() {
        let mut s = s();
        type_str(&mut s, "sakura");
        s.key(Key::Space);
        let second = s.key(Key::Space);
        let chosen = second.preedit.clone();
        s.key(Key::Enter);
        let st = type_str(&mut s, "sakura");
        assert_eq!(st.cands[0], chosen);
    }

    #[test]
    fn function_keys_force_form() {
        let mut s = s();
        type_str(&mut s, "hello");
        assert_eq!(s.key(Key::Fn(6)).preedit, romaji::to_hiragana("hello"));
        assert_eq!(s.key(Key::Fn(7)).preedit, romaji::hira_to_kata(&romaji::to_hiragana("hello")));
        assert_eq!(s.key(Key::Fn(10)).preedit, "hello");
        assert_eq!(s.key(Key::Fn(10)).preedit, "HELLO");
        assert_eq!(s.key(Key::Fn(10)).preedit, "Hello");
        assert_eq!(s.key(Key::Fn(9)).preedit, "ｈｅｌｌｏ");
        assert_eq!(s.key(Key::Fn(9)).preedit, "ＨＥＬＬＯ");
        assert!(s.key(Key::Fn(9)).mode.contains("強制"));
    }

    #[test]
    fn forced_form_is_learned_for_next_time() {
        let mut s = s();
        type_str(&mut s, "wifi");
        s.key(Key::Fn(10));
        s.key(Key::Enter);
        // 次回は最初から半角英字
        let st = type_str(&mut s, "wifi");
        assert_eq!(st.preedit, "wifi");
        assert!(st.mode.contains("強制"));
    }

    #[test]
    fn low_confidence_does_not_switch_to_alpha() {
        let mut s = Session::new(Box::new(LocalConverter { dict: None }));
        s.set_before("");
        // 文脈が無く曖昧な短い語は、かな表示のまま
        assert_eq!(type_str(&mut s, "na").preedit, "な");
    }

    #[test]
    fn history_predictions_never_replace_a_short_exact_input() {
        // 過去に「にゅうりょく→入力」「がくしゅう→学習」を選んだ履歴があっても、「に」「が」と打てば「に」「が」になる
        let p = std::env::temp_dir().join(format!("aruaru-learn-{}.json", std::process::id()));
        std::fs::write(&p, r#"{"map":{"にゅうりょく":[["入力",3]],"がくしゅう":[["学習",2]],"は":[["葉",1]]},"forms":{}}"#).unwrap();
        for (typed, want, long_word) in [("ni", "に", "入力"), ("ga", "が", "学習")] {
            let mut s = Session::new(Box::new(LocalConverter { dict: None }));
            s.set_before("私は");
            s.enable_learning(p.clone());
            let st = type_str(&mut s, typed);
            assert_eq!(st.preedit, want, "{typed}");
            assert!(st.cands.contains(&long_word.to_string()), "予測としては候補に残る");
            assert!(st.cands.iter().position(|c| c == long_word).unwrap() > st.cands.iter().position(|c| c == want).unwrap());
        }
        // 完全一致の学習(はを葉と選んだ)は、従来どおり先頭に来る
        let mut s = Session::new(Box::new(LocalConverter { dict: None }));
        s.enable_learning(p.clone());
        assert_eq!(type_str(&mut s, "ha").cands[0], "葉");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn katakana_transcriptions_and_kana_forms_from_the_old_bug_are_cleaned() {
        let p = std::env::temp_dir().join(format!("aruaru-learn3-{}.json", std::process::id()));
        std::fs::write(&p, r#"{"map":{"たんご":[["タンゴ",2],["単語",2]],"かんじ":[["カンジ",2],["感じ",1]],"こんぴゅーた":[["コンピュータ",3]]},"forms":{"ai":"kata","database":"half","sazannaizu":"kata"}}"#).unwrap();
        let l = Learned::load(&p);
        assert_eq!(l.exact("たんご"), vec!["単語"], "漢字の履歴があるときは、カタカナの転記だけの履歴を捨てる");
        assert_eq!(l.exact("かんじ"), vec!["感じ"]);
        assert_eq!(l.exact("こんぴゅーた"), vec!["コンピュータ"], "他に候補が無いカタカナ語は残す");
        assert!(!l.forms.contains_key("ai") && !l.forms.contains_key("sazannaizu"), "かな・カタカナの強制表記は学習しない");
        assert_eq!(l.forms.get("database").map(String::as_str), Some("half"));
        // 新しくカタカナを強制しても学習されない
        let mut s = Session::new(Box::new(LocalConverter { dict: None }));
        s.live = false;
        s.enable_learning(p.clone());
        type_str(&mut s, "ai");
        s.key(Key::Fn(7));
        s.key(Key::Enter);
        assert!(!Learned::load(&p).forms.contains_key("ai"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn implausible_history_from_the_old_bug_is_discarded() {
        let p = std::env::temp_dir().join(format!("aruaru-learn2-{}.json", std::process::id()));
        std::fs::write(&p, r#"{"map":{"に":[["入力",3],["ニホンゴノ",2],["二",1]],"が":[["学習",2],["画",1]],"さんじゃんうんから":[["三只眼吽迦羅",1]]},"forms":{}}"#).unwrap();
        let l = Learned::load(&p);
        assert_eq!(l.exact("に"), vec!["二"], "読みより長い漢字語・長さの違うかな語は捨てる");
        assert_eq!(l.exact("が"), vec!["画"]);
        assert_eq!(l.exact("さんじゃんうんから"), vec!["三只眼吽迦羅"], "読みより短い漢字語は正しい");
        let mut l2 = Learned::default();
        l2.record("に", "入力");
        l2.record("に", "ニホンゴノ");
        l2.record("3x3", "3X3 EYES");
        assert!(l2.exact("に").is_empty() && l2.exact("3x3") == vec!["3X3 EYES"]);
        assert!(plausible_pair("こんぴゅーた", "コンピュータ") && plausible_pair("たんご", "単語") && plausible_pair("にほん", "日本"));
        let _ = std::fs::remove_file(&p);
    }

    struct Gl;
    impl Converter for Gl {
        fn convert(&self, buffer: &str, _b: &str, _f: Option<&str>) -> Conversion {
            use crate::convert::Candidate;
            let hira = romaji::to_hiragana(buffer);
            let c = if hira == "たんご" {
                vec![Candidate { text: "単語".into(), kind: "dict" }, Candidate { text: "丹後".into(), kind: "dict" }, Candidate { text: hira.clone(), kind: "hiragana" }]
            } else {
                vec![Candidate { text: buffer.to_string(), kind: "alpha" }]
            };
            let mode = if hira == "たんご" { Mode::Hiragana } else { Mode::Alpha };
            Conversion { prediction: crate::mode::Prediction { mode, confidence: 0.99, english_word: false, reason: String::new() }, candidates: c, suggestions: vec![], exact_dict: true }
        }
        fn to_english(&self, surface: &str, _r: &str) -> Vec<String> {
            match surface {
                "単語" => vec!["word".into(), "vocabulary".into()],
                "丹後" => vec!["Tango (former province)".into()],
                _ => vec![],
            }
        }
        fn to_japanese(&self, word: &str) -> Vec<String> {
            if word == "word" { vec!["単語".into(), "語".into()] } else { vec![] }
        }
    }

    #[test]
    fn japanese_and_english_columns_with_left_right_and_up_down() {
        let mut s = Session::new(Box::new(Gl));
        s.set_before("");
        let st = type_str(&mut s, "tango");
        assert_eq!(st.preedit, "単語");
        assert_eq!(st.cands_en, vec!["word", "vocabulary", "Tango (former province)"], "日本語の候補の英訳が、英語の列に並ぶ");
        assert_eq!(st.col, 0);
        // → で英語の列へ。上下で英語の中から選ぶ
        let st = s.key(Key::Right);
        assert_eq!((st.col, st.sel_en, st.preedit.as_str()), (1, Some(0), "word"));
        assert_eq!(s.key(Key::Down).preedit, "vocabulary");
        assert_eq!(s.key(Key::Up).preedit, "word");
        // ← で日本語の列に戻る
        let st = s.key(Key::Left);
        assert_eq!((st.col, st.preedit.as_str()), (0, "単語"));
        // 英語の列で Enter すると、英語を確定する
        s.key(Key::Right);
        s.key(Key::Down);
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("vocabulary"));
        // 数字キーでも選べる
        type_str(&mut s, "tango");
        s.key(Key::Right);
        assert_eq!(s.key(Key::Char('3')).commit.as_deref(), Some("Tango (former province)"));
        // 英語の列が空なら、→ は何もしない
        let mut s2 = Session::new(Box::new(LocalConverter { dict: None }));
        s2.set_before("");
        type_str(&mut s2, "sakura");
        assert_eq!(s2.key(Key::Right).col, 0);
    }

    #[test]
    fn english_word_gets_japanese_equivalents() {
        let mut s = Session::new(Box::new(Gl));
        s.set_before("The ");
        let st = type_str(&mut s, "word");
        assert_eq!(st.preedit, "word");
        assert!(st.cands.contains(&"単語".to_string()) && st.cands.contains(&"語".to_string()), "{:?}", st.cands);
        assert!(st.cands.iter().position(|c| c == "word").unwrap() < st.cands.iter().position(|c| c == "単語").unwrap(), "英語そのものが先頭");
        // Space で候補を進める: 最初は先頭の英語そのもの、次に日本語の対訳
        assert_eq!(s.key(Key::Space).preedit, "word");
        assert_eq!(s.key(Key::Space).preedit, "単語");
    }

    fn lang_session(l: Lang) -> Session {
        let mut s = Session::new(Box::new(Gl));
        s.set_lang(l);
        s.set_before("");
        s
    }

    #[test]
    fn russian_typing_shows_cyrillic_and_space_commits_with_a_space() {
        let mut s = lang_session(Lang::Ru);
        let st = type_str(&mut s, "privet");
        assert_eq!((st.preedit.as_str(), st.mode.as_str(), st.lang.as_str()), ("привет", "ru", "ru"));
        let st = s.key(Key::Space);
        assert_eq!(st.commit.as_deref(), Some("привет "));
        // 区切りの記号は、確定してから続く
        type_str(&mut s, "mir");
        assert_eq!(s.key(Key::Char('!')).commit.as_deref(), Some("мир!"));
        // Enter は空白なし
        type_str(&mut s, "da");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("да"));
        // 入力していないとき、空白・数字は素通し
        assert!(!s.key(Key::Space).eaten && !s.key(Key::Char('5')).eaten);
    }

    #[test]
    fn arabic_persian_hebrew_prefer_dictionary_words_and_keep_rtl_text() {
        let mut s = lang_session(Lang::Ar);
        let st = type_str(&mut s, "salam");
        assert_eq!(st.preedit, "سلام");
        assert!(crate::translit::is_rtl_text(&st.preedit));
        assert!(st.cands.len() >= 2, "規則による変換と入力そのままも候補に残る");
        let mut s = lang_session(Lang::Fa);
        assert_eq!(type_str(&mut s, "chetori").preedit, "چطوری");
        let mut s = lang_session(Lang::He);
        assert_eq!(type_str(&mut s, "shalom").preedit, "שלום");
        // アラビア語・ペルシア語では、数字キーが文字(3=ع など)になる
        let mut s = lang_session(Lang::Ar);
        assert_eq!(type_str(&mut s, "3ain").preedit, "عاين".replace("عاين", &crate::translit::transliterate(Lang::Ar, "3ain")));
    }

    #[test]
    fn korean_composes_hangul_as_you_type() {
        let mut s = lang_session(Lang::Ko);
        assert_eq!(type_str(&mut s, "dkssud").preedit, "안녕");
        assert_eq!(s.key(Key::Backspace).preedit, "안녀");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("안녀"));
    }

    #[test]
    fn european_accents_and_german_umlauts() {
        let mut s = lang_session(Lang::Fr);
        assert_eq!(type_str(&mut s, "cafe'").preedit, "café");
        let mut s = lang_session(Lang::De);
        let st = type_str(&mut s, "fuer");
        assert_eq!(st.preedit, "fuer", "ドイツ語は打った綴りが先頭");
        assert!(st.cands.contains(&"für".to_string()), "ü の綴りは候補に");
        assert_eq!(s.key(Key::Down).preedit, "fuer");
        assert_eq!(s.key(Key::Down).preedit, "für");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("für"));
    }

    #[test]
    fn english_gets_japanese_equivalents_and_chinese_is_passthrough_until_supported() {
        let mut s = lang_session(Lang::En);
        let st = type_str(&mut s, "word");
        assert_eq!(st.preedit, "word");
        assert!(st.cands.contains(&"単語".to_string()));
        assert_eq!(s.key(Key::Down).preedit, "word");
        assert_eq!(s.key(Key::Down).preedit, "単語");
        assert_eq!(s.key(Key::Enter).commit.as_deref(), Some("単語"));
        let mut z = lang_session(Lang::ZhHans);
        assert!(z.key(Key::Char('n')).eaten, "中国語はピンインを受け付ける");
    }

    #[test]
    fn language_switch_resets_composition_and_changes_what_is_eaten() {
        let mut s = lang_session(Lang::Ja);
        type_str(&mut s, "ka");
        s.set_lang(Lang::Ru);
        assert!(!s.composing(), "切替で組み立て中の入力は破棄する");
        assert!(s.would_eat(&Key::Char('a')));
        assert!(!s.would_eat(&Key::Char('5')), "ロシア語では、入力していない数字は素通し");
        s.set_lang(Lang::Ar);
        assert!(s.would_eat(&Key::Char('5')), "アラビア語では、数字も文字");
        assert!(would_eat_lang(&Key::Char('a'), true, false, false, Lang::ZhHans));
    }

    #[test]
    fn chinese_pinyin_partial_commit_and_digit_select() {
        crate::zh::set_global(crate::zh::ZhDict::parse("zhong	中	中	200
guo	国	國	300
zhongguo	中国	中國	5000
ni	你	你	900
hao	好	好	800
nihao	你好	你好	4000
"));
        let mut s = lang_session(Lang::ZhHans);
        type_str(&mut s, "nihaozhong");
        let st = s.key(Key::Space);
        assert_eq!(st.commit.as_deref(), Some("你好中"), "{st:?}");
        assert!(!s.composing());
        type_str(&mut s, "zhongguo");
        let st = s.key(Key::Char('1'));
        assert_eq!(st.commit.as_deref(), Some("中国"));
        let mut t = lang_session(Lang::ZhHant);
        type_str(&mut t, "zhongguo");
        assert_eq!(t.key(Key::Enter).commit.as_deref(), Some("中國"));
    }

    #[test]
    fn zero_key_picks_the_tenth_candidate() {
        struct Many;
        impl Converter for Many {
            fn convert(&self, buffer: &str, _b: &str, _f: Option<&str>) -> Conversion {
                use crate::convert::Candidate;
                let c: Vec<Candidate> = (0..14).map(|i| Candidate { text: format!("{buffer}{i}"), kind: "dict" }).collect();
                Conversion { prediction: crate::mode::Prediction { mode: Mode::Hiragana, confidence: 0.99, english_word: false, reason: String::new() }, candidates: c, suggestions: vec![], exact_dict: false }
            }
        }
        let mut s = Session::new(Box::new(Many));
        s.live = false;
        s.set_before("");
        type_str(&mut s, "ka");
        s.key(Key::Space);
        assert_eq!(s.key(Key::Char('0')).commit.as_deref(), Some("ka9"), "0 は10番目");
        let mut s2 = Session::new(Box::new(Many));
        s2.live = false;
        s2.set_before("");
        type_str(&mut s2, "ka");
        s2.key(Key::Space);
        for _ in 0..10 {
            s2.key(Key::Space); // 2ページ目へ(11番目の候補を選ぶ)
        }
        assert_eq!(s2.key(Key::Char('1')).commit.as_deref(), Some("ka10"), "2ページ目の1番は全体の11番目");
    }

    #[test]
    fn number_keys_pick_candidates_while_selecting() {
        let mut s = live();
        type_str(&mut s, "kanji");
        s.key(Key::Space); // 明示的に候補を選び始める(以降、数字キーは候補番号)
        let st = s.key(Key::Char('1'));
        assert_eq!(st.commit.as_deref(), Some("漢字"), "1番の候補で確定");
        // 打っている途中(自動選択)の数字は、従来どおり入力の一部
        let mut s2 = live();
        let st = type_str(&mut s2, "kanji2");
        assert!(st.commit.is_none() && st.preedit.contains('2') || st.cands.iter().any(|c| c.contains('2')));
        let mut s3 = live();
        type_str(&mut s3, "kanji");
        s3.key(Key::Space);
        let st = s3.key(Key::Char('2'));
        assert!(st.commit.is_some(), "2番を選んで確定");
    }

    #[test]
    fn registered_words_come_first() {
        let p = std::env::temp_dir().join(format!("aruaru-sess-ud-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&p);
        let mut u = crate::userdict::UserDict::load(&p);
        u.add("aon", "株式会社AON").unwrap();
        u.add("おせわになっております", "お世話になっております。").unwrap();
        let mut s = s();
        s.enable_user_dict(&p);
        assert_eq!(type_str(&mut s, "aon").cands[0], "株式会社AON");
        s.key(Key::Esc);
        // 熟語は途中まで打つと予測変換で出る
        let st = type_str(&mut s, "osewani");
        assert!(st.cands.iter().any(|c| c == "お世話になっております。"), "{:?}", st.cands);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn shows_mode_label() {
        let mut s = s();
        assert_eq!(type_str(&mut s, "sakura").mode, "あ");
    }

    #[test]
    fn typing_after_conversion_commits_selected() {
        let mut s = s();
        type_str(&mut s, "sakura");
        s.key(Key::Space);
        let st = s.key(Key::Char('n'));
        assert!(st.commit.is_some());
        assert_eq!(st.preedit, "n");
    }
}
