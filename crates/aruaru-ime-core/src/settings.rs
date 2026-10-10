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
        Self { theme: default_theme(), bypass_apps: vec![], live_conversion: true, candidate_scale: default_scale() }
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
    fn rejects_unknown_theme() {
        let mut s = Settings::default();
        assert!(s.set_theme("blue").is_err());
        assert!(s.dark());
    }
}
