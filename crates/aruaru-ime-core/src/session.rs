//! 入力セッション(キー入力→未確定文字列・候補・確定の状態機械)。OS非依存で、TSF/IBus/IMKなどの薄い層から使う。
//!
//! 流れ: ローマ字を打つ → 入力モードAI予測で未確定表示(かな/カタカナ/英字のまま)を決め、候補を並べる
//! → Space で変換候補を選択 → Enter で確定。

use crate::convert::Conversion;
use crate::mode::Mode;
use crate::romaji;

pub trait Converter {
    fn convert(&self, buffer: &str, before: &str, field_kind: Option<&str>) -> Conversion;
}

#[derive(Debug, Clone, PartialEq)]
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
    Other,
}

#[derive(Debug, Default, Clone, PartialEq)]
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

impl Learned {
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn save(&self, path: &std::path::Path) {
        if let Some(d) = path.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = std::fs::write(path, serde_json::to_string(self).unwrap_or_default());
    }
    pub fn record(&mut self, reading: &str, surface: &str) {
        if reading.is_empty() || surface.is_empty() {
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
    /// 読みが一致する学習済み表記(回数順)、続いて読みが前方一致する表記。
    pub fn lookup(&self, reading: &str) -> Vec<String> {
        let mut out: Vec<String> = self.map.get(reading).map(|v| v.iter().map(|x| x.0.clone()).collect()).unwrap_or_default();
        let mut pre: Vec<(&String, u32)> = self
            .map
            .iter()
            .filter(|(k, _)| k.starts_with(reading) && k.as_str() != reading)
            .flat_map(|(_, v)| v.iter().map(|x| (&x.0, x.1)))
            .collect();
        pre.sort_by(|a, b| b.1.cmp(&a.1));
        for (s, _) in pre.into_iter().take(4) {
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
    user: Option<crate::userdict::UserDict>,
    /// ライブ変換: 打つそばから最有力の変換結果(漢字かな交じり)を未確定表示にする。既定オン。
    pub live: bool,
    /// いまの選択がライブ変換による自動のもの(利用者が Space/矢印で選んだものではない)。
    auto: bool,
}

fn is_ja(c: char) -> bool {
    matches!(c, 'ぁ'..='ゖ' | 'ァ'..='ヺ' | 'ー' | '一'..='龥' | '、' | '。')
}

impl Session {
    pub fn new(conv: Box<dyn Converter>) -> Self {
        Self { conv, on: true, field_kind: None, before: String::new(), buf: String::new(), cands: vec![], sel: None, unconverted: String::new(), learned: Learned::default(), learn_path: None, form: None, form_cycle: 0, mode_label: String::new(), user: None, live: true, auto: false }
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
        if matches!(key, Key::Toggle | Key::On | Key::Off) {
            return true;
        }
        if !self.on {
            return false;
        }
        match key {
            Key::Char(c) => {
                c.is_ascii_alphabetic()
                    || self.composing()
                    || (matches!(c, ',' | '.') && self.before.chars().last().is_some_and(is_ja))
            }
            Key::Space | Key::Down | Key::Tab | Key::Up | Key::Enter | Key::Esc | Key::Backspace | Key::Fn(_) => self.composing(),
            _ => false,
        }
    }

    fn state(&self, eaten: bool, commit: Option<String>) -> Step {
        let preedit = match self.sel {
            Some(i) => self.cands.get(i).cloned().unwrap_or_default(),
            None => self.unconverted.clone(),
        };
        Step { eaten, preedit, commit, cands: if self.composing() { self.cands.clone() } else { vec![] }, sel: self.sel, mode: if self.composing() { self.mode_label.clone() } else { String::new() } }
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

    fn refresh(&mut self) {
        let r = self.conv.convert(&self.buf, &self.before, self.field_kind.as_deref());
        let (hira, rest) = romaji::to_hiragana_partial(&self.buf);
        // 過去に利用者が強制した表記があれば、最初からそれにする(誤ったモード予測の学習)
        if self.form.is_none() {
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
        let reading = romaji::to_hiragana(&self.buf);
        let is_kana_mode = matches!(mode, Mode::Hiragana | Mode::Katakana) && self.form.is_none();
        // 学習済みの表記は、かな入力モードのとき最優先で先頭に出す
        let mut c: Vec<String> = vec![];
        if is_kana_mode {
            // 登録した単語・熟語が最優先(完全一致 → 前方一致)、次に学習履歴
            if let Some(u) = self.user.as_mut() {
                u.reload_if_changed();
                c.extend(u.exact(&reading));
                c.extend(u.prefix(&reading, 4));
            }
            c.extend(self.learned.lookup(&reading));
        }
        c.extend(r.candidates.iter().map(|x| x.text.clone()));
        c.extend(r.suggestions.iter().cloned());
        let mut seen = vec![];
        c.retain(|x| !seen.contains(x) && { seen.push(x.clone()); true });
        self.cands = c;
        self.sel = None;
        self.auto = false;
        // ライブ変換: ひらがな入力と予測され、強制表記が無く、入力途中のローマ字(sak の k など)が残っていないときだけ
        if self.live && self.form.is_none() && mode == Mode::Hiragana && rest.is_empty() && !self.cands.is_empty() {
            self.sel = Some(0);
            self.auto = true;
            self.mode_label = "あ→漢".to_string();
        }
    }

    /// 現在の確定文字列(変換中なら選択中の候補、そうでなければ未確定表示)。
    fn current(&self) -> String {
        match self.sel {
            Some(i) => self.cands.get(i).cloned().unwrap_or_default(),
            None => self.unconverted.clone(),
        }
    }

    fn commit_current(&mut self) -> String {
        let s = self.current();
        if self.sel.is_some() && !self.auto {
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
        match key {
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
                // 日本語の直後の , . は 、。にする
                if matches!(c, ',' | '.') && self.before.chars().last().is_some_and(is_ja) {
                    let s = if c == ',' { "、" } else { "。" };
                    self.note_commit(s);
                    return self.state(true, Some(s.into()));
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
                // 選んだ表記を学習する(次回から同じ入力は最初からこの表記)
                self.learned.forms.insert(self.buf.to_lowercase(), f.name().to_string());
                if let Some(p) = self.learn_path.clone() {
                    self.learned.save(&p);
                }
                self.state(true, None)
            }
            Key::Fn(_) if self.composing() => self.state(true, None),
            Key::Space | Key::Down | Key::Tab if self.composing() => {
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
    fn digits_and_symbols_pass_through_when_idle() {
        let mut s = s();
        let st = s.key(Key::Char('5'));
        assert!(!st.eaten);
        // 日本語の直後のカンマは読点にする
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
