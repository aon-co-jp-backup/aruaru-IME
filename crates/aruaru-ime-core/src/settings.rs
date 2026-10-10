//! 利用者設定(共有データ領域の `settings.json`)。IME 本体・常駐エンジン・CLI から同じファイルを使う。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// 候補窓の配色: "dark"(黒背景・白文字、既定)または "light"(白背景・黒文字)。
    #[serde(default = "default_theme")]
    pub theme: String,
    /// IME を自動でオフにするアプリの実行ファイル名(例: "game.exe")。
    #[serde(default)]
    pub bypass_apps: Vec<String>,
    /// ライブ変換(打つそばから漢字かな交じりを未確定表示にする)。既定オン。
    #[serde(default = "default_true")]
    pub live_conversion: bool,
    /// 候補窓の大きさの倍率(既定 1.8。文字・行の高さ・幅が全てこの倍率で大きくなる)。
    #[serde(default = "default_scale")]
    pub candidate_scale: f32,
    /// 自動アップデート(GitHub Releases の最新版を確認して更新)。既定オン。
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// プログラミング用語の自動訂正(確信が高い綴りミスを訂正後の語にする)。既定オン。
    #[serde(default = "default_true")]
    pub autocorrect: bool,
    /// 旧字体(國・學・體 など)で表示する。オフのときは、旧字体にできる候補の旧字体版を候補の末尾に足す。既定オフ。
    #[serde(default)]
    pub old_forms: bool,
    /// 使う入力言語(言語コード)。最大 `MAX_LANGUAGES` 個。日本語・英語は先頭に固定表示される。
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
    /// 辞書に無い固有名詞・作品名のために、変換を求めたとき Web のサジェスト(Google)を引く。入力中の文字を Google に送るので、不要ならオフ。既定オン。
    #[serde(default = "default_true")]
    pub web_suggest: bool,
    /// 漢字を優先する(既定オン)。ございます → 御座います、おねがいします → お願いします のように、語尾などを漢字にした候補を先頭にする。
    #[serde(default = "default_true")]
    pub prefer_kanji: bool,
    /// 自動バックアップ(起動後と、内容が変わったとき)。既定オン。保存先は Google ドライブの同期フォルダ(見つからなければ「ドキュメント」)。
    #[serde(default = "default_true")]
    pub auto_backup: bool,
    /// バックアップの保存先を指定する(空なら自動)。
    #[serde(default)]
    pub backup_dir: Option<String>,
}

/// 同時に選べる入力言語の最大数。
pub const MAX_LANGUAGES: usize = 7;

fn default_languages() -> Vec<String> {
    ["ja", "en", "zh-Hans", "ko", "ru", "ar", "fa"].iter().map(|s| s.to_string()).collect()
}

fn default_scale() -> f32 {
    1.8
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "dark".into()
}

impl Default for Settings {
    fn default() -> Self {
        Self { theme: default_theme(), bypass_apps: vec![], live_conversion: true, candidate_scale: default_scale(), auto_update: true, autocorrect: true, old_forms: false, languages: default_languages(), web_suggest: true, prefer_kanji: true, auto_backup: true, backup_dir: None }
    }
}

impl Settings {
    pub fn path() -> PathBuf {
        crate::components::data_dir().join("settings.json")
    }

    pub fn load_from(p: &std::path::Path) -> Self {
        std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    pub fn save_to(&self, p: &std::path::Path) -> std::io::Result<()> {
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(p, serde_json::to_string_pretty(self).unwrap_or_default())
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&Self::path())
    }

    /// 実際に使う倍率(極端な値は 1.0〜4.0 に丸める)。
    pub fn scale(&self) -> f32 {
        if self.candidate_scale.is_finite() { self.candidate_scale.clamp(1.0, 4.0) } else { 1.8 }
    }

    /// 選択中の言語を返す(未知のコードは捨て、重複を除き、最大数で切る)。日本語・英語が無ければ先頭に足す。
    pub fn selected_languages(&self) -> Vec<crate::translit::Lang> {
        let mut v: Vec<crate::translit::Lang> = vec![];
        for c in &self.languages {
            if let Some(l) = crate::translit::Lang::from_code(c) {
                if !v.contains(&l) {
                    v.push(l);
                }
            }
        }
        for must in [crate::translit::Lang::En, crate::translit::Lang::Ja] {
            if !v.contains(&must) {
                v.insert(0, must);
            }
        }
        v.truncate(MAX_LANGUAGES);
        v
    }

    /// 言語の選択を切り替える。上限(7)を超える追加と、日本語・英語の解除は受け付けない。変えたら true。
    pub fn toggle_language(&mut self, lang: crate::translit::Lang) -> Result<(), String> {
        use crate::translit::Lang;
        let mut cur = self.selected_languages();
        if cur.contains(&lang) {
            if matches!(lang, Lang::Ja | Lang::En) {
                return Err("日本語と英語は外せません / Japanese and English cannot be removed".into());
            }
            cur.retain(|l| *l != lang);
        } else {
            if cur.len() >= MAX_LANGUAGES {
                return Err(format!("選べる言語は最大{MAX_LANGUAGES}つです / You can select up to {MAX_LANGUAGES} languages"));
            }
            cur.push(lang);
        }
        self.languages = cur.iter().map(|l| l.code().to_string()).collect();
        Ok(())
    }

    pub fn dark(&self) -> bool {
        self.theme != "light"
    }

    /// 配色を反対にして保存し、新しい配色名を返す。
    pub fn toggle_theme(&mut self) -> &str {
        self.theme = if self.dark() { "light".into() } else { "dark".into() };
        let _ = self.save();
        &self.theme
    }

    /// 指定した配色名("dark"/"light")だけを受け付けて設定する。
    pub fn set_theme(&mut self, name: &str) -> Result<(), String> {
        match name {
            "dark" | "light" => {
                self.theme = name.into();
                self.save().map_err(|e| e.to_string())
            }
            other => Err(format!("配色は dark か light です(指定: {other})")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark_and_roundtrips() {
        let p = std::env::temp_dir().join(format!("aruaru-set-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&p);
        let mut s = Settings::load_from(&p);
        assert!(s.dark(), "既定は黒背景・白文字");
        s.theme = "light".into();
        s.save_to(&p).unwrap();
        assert!(!Settings::load_from(&p).dark());
        // 壊れたファイル・未知の項目でも既定にフォールバックする
        std::fs::write(&p, "{ broken").unwrap();
        assert!(Settings::load_from(&p).dark());
        std::fs::write(&p, r#"{"future_field": 1}"#).unwrap();
        assert!(Settings::load_from(&p).dark());
        assert!(Settings::load_from(&p).live_conversion, "既定でライブ変換オン");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn candidate_panel_is_large_by_default_and_clamped() {
        let mut s = Settings::default();
        assert!(s.scale() >= 1.8, "候補窓は既定でかなり大きい");
        s.candidate_scale = 100.0;
        assert_eq!(s.scale(), 4.0);
        s.candidate_scale = f32::NAN;
        assert_eq!(s.scale(), 1.8);
    }

    #[test]
    fn language_selection_is_capped_at_seven_and_keeps_ja_en() {
        use crate::translit::Lang;
        let mut s = Settings::default();
        let d = s.selected_languages();
        assert_eq!(d.len(), 7);
        assert_eq!(&d[..2], &[Lang::Ja, Lang::En]);
        assert!(s.toggle_language(Lang::Fr).is_err(), "8つ目は追加できない");
        assert!(s.toggle_language(Lang::Ja).is_err() && s.toggle_language(Lang::En).is_err());
        assert!(s.toggle_language(Lang::Ru).is_ok(), "解除はできる");
        assert!(s.toggle_language(Lang::Fr).is_ok(), "空きができれば追加できる");
        assert!(s.selected_languages().contains(&Lang::Fr) && !s.selected_languages().contains(&Lang::Ru));
        // 壊れた設定でも日本語・英語は先頭に戻る
        s.languages = vec!["xx".into(), "ko".into()];
        assert_eq!(s.selected_languages(), vec![Lang::Ja, Lang::En, Lang::Ko]);
    }

    #[test]
    fn rejects_unknown_theme() {
        let mut s = Settings::default();
        assert!(s.set_theme("blue").is_err());
        assert!(s.dark());
    }
}
