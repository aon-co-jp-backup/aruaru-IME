//! 利用者辞書: 単語・熟語・定型句(読み → 表記)の登録。
//! 保存先は共有データ領域の `user_dict.json`。IME 本体(TSF)・常駐エンジン・CLI から同じファイルを使い、
//! 変更は他のプロセスにも即座に反映される(更新時刻を見て自動で読み直す)。

use crate::romaji;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UEntry {
    /// 読み(ひらがな)。
    pub reading: String,
    /// 表記(単語・熟語・長い定型文でもよい)。
    pub surface: String,
    /// 取り込み元のファイル名(手で登録した分は None)。ファイル単位の一括削除に使う。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub struct UserDict {
    path: PathBuf,
    entries: Vec<UEntry>,
    mtime: Option<SystemTime>,
}

/// 読みの正規化: 前後の空白を除き、ローマ字ならひらがなに、カタカナならひらがなにする。
pub fn normalize_reading(r: &str) -> String {
    let r = r.trim();
    if r.is_empty() {
        return String::new();
    }
    if r.chars().all(|c| c.is_ascii_alphabetic() || matches!(c, '-' | '\'')) {
        return romaji::to_hiragana(r);
    }
    romaji::kata_to_hira(r)
}

fn file_mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

impl UserDict {
    pub fn load(path: &Path) -> Self {
        let entries = std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        Self { path: path.to_path_buf(), entries, mtime: file_mtime(path) }
    }

    /// 共有データ領域の既定の保存先。
    pub fn default_path() -> PathBuf {
        crate::components::data_dir().join("user_dict.json")
    }

    /// ファイルが他のプロセスに更新されていたら読み直す。
    pub fn reload_if_changed(&mut self) {
        let m = file_mtime(&self.path);
        if m != self.mtime {
            *self = Self::load(&self.path);
        }
    }

    pub fn entries(&self) -> &[UEntry] {
        &self.entries
    }

    fn save(&mut self) -> std::io::Result<()> {
        if let Some(d) = self.path.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(&self.path, serde_json::to_string_pretty(&self.entries).unwrap_or_default())?;
        self.mtime = file_mtime(&self.path);
        Ok(())
    }

    /// 登録する。同じ(読み, 表記)が既にあれば何もしない。`Ok(true)` なら新規追加。
    pub fn add(&mut self, reading: &str, surface: &str) -> std::io::Result<bool> {
        self.add_from(reading, surface, None)
    }

    pub fn add_from(&mut self, reading: &str, surface: &str, source: Option<&str>) -> std::io::Result<bool> {
        self.reload_if_changed();
        let surface = surface.trim().to_string();
        // 英数字だけの語(GitHub など)は、読みもそのまま小文字で持つ(ローマ字→かな変換しない)
        let raw = reading.trim().to_lowercase();
        let reading = if !raw.is_empty() && raw.chars().all(|c| c.is_ascii_alphanumeric()) && surface.to_lowercase() == raw { raw } else { normalize_reading(reading) };
        if reading.is_empty() || surface.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "読みと表記の両方が必要です"));
        }
        if self.entries.iter().any(|e| e.reading == reading && e.surface == surface) {
            return Ok(false);
        }
        self.entries.push(UEntry { reading, surface, source: source.map(str::to_string) });
        self.save()?;
        Ok(true)
    }

    /// まとめて登録する(ファイルへの保存は最後に1回)。新しく入れた件数を返す。
    pub fn add_many(&mut self, pairs: &[(String, String)], source: &str) -> std::io::Result<usize> {
        self.reload_if_changed();
        let mut have: std::collections::HashSet<(String, String)> = self.entries.iter().map(|e| (e.reading.clone(), e.surface.clone())).collect();
        let mut n = 0;
        for (r, s) in pairs {
            let reading = normalize_reading(r);
            let surface = s.trim().to_string();
            if reading.is_empty() || surface.is_empty() || !have.insert((reading.clone(), surface.clone())) {
                continue;
            }
            self.entries.push(UEntry { reading, surface, source: Some(source.to_string()) });
            n += 1;
        }
        if n > 0 {
            self.save()?;
        }
        Ok(n)
    }

    /// (読み, 表記)を削除する。表記を省略(None)すると、その読みの登録を全て削除。削除件数を返す。
    pub fn remove(&mut self, reading: &str, surface: Option<&str>) -> std::io::Result<usize> {
        self.reload_if_changed();
        let reading = normalize_reading(reading);
        let before = self.entries.len();
        self.entries.retain(|e| !(e.reading == reading && surface.map_or(true, |s| e.surface == s.trim())));
        let n = before - self.entries.len();
        if n > 0 {
            self.save()?;
        }
        Ok(n)
    }

    /// 指定したファイル由来の登録を全て削除する。削除件数を返す。
    pub fn remove_source(&mut self, source: &str) -> std::io::Result<usize> {
        self.reload_if_changed();
        let before = self.entries.len();
        self.entries.retain(|e| e.source.as_deref() != Some(source));
        let n = before - self.entries.len();
        if n > 0 {
            self.save()?;
        }
        Ok(n)
    }

    /// 取り込み元ファイルごとの登録件数。
    pub fn sources(&self) -> Vec<(String, usize)> {
        let mut m: std::collections::BTreeMap<String, usize> = Default::default();
        for e in &self.entries {
            if let Some(s) = &e.source {
                *m.entry(s.clone()).or_default() += 1;
            }
        }
        m.into_iter().collect()
    }

    /// 読みが完全一致する登録(新しい登録ほど先)。
    pub fn exact(&self, reading: &str) -> Vec<String> {
        self.entries.iter().rev().filter(|e| e.reading == reading).map(|e| e.surface.clone()).collect()
    }

    /// 読みが前方一致する登録(より長い読み)。予測変換用。
    pub fn prefix(&self, prefix: &str, max: usize) -> Vec<String> {
        if prefix.is_empty() {
            return vec![];
        }
        self.entries.iter().rev().filter(|e| e.reading.starts_with(prefix) && e.reading != prefix).map(|e| e.surface.clone()).take(max).collect()
    }

    /// 登録内容を「読み<TAB>表記」のテキストで書き出す / 取り込む(他のIMEや別PCとの受け渡し用)。
    pub fn export_tsv(&self) -> String {
        self.entries.iter().map(|e| format!("{}\t{}", e.reading, e.surface)).collect::<Vec<_>>().join("\n")
    }

    pub fn import_tsv(&mut self, text: &str) -> std::io::Result<usize> {
        let mut n = 0;
        for line in text.lines() {
            if let Some((r, s)) = line.split_once('\t') {
                if self.add(r, s).unwrap_or(false) {
                    n += 1;
                }
            }
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-ud-{n}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("user_dict.json")
    }

    #[test]
    fn add_lookup_remove_and_persist() {
        let p = tmp("basic");
        let mut u = UserDict::load(&p);
        assert!(u.add("aon", "株式会社AON").unwrap());
        assert!(!u.add("あおん", "株式会社AON").unwrap(), "重複は追加しない(ローマ字もかなに正規化)");
        assert!(u.add("おせわになっております", "お世話になっております。").unwrap());
        assert_eq!(u.exact("あおん"), vec!["株式会社AON"]);
        assert_eq!(u.prefix("おせわに", 5), vec!["お世話になっております。"]);
        // 別プロセスを想定し、読み直しても残っている
        let u2 = UserDict::load(&p);
        assert_eq!(u2.entries().len(), 2);
        assert_eq!(u.remove("あおん", None).unwrap(), 1);
        assert_eq!(UserDict::load(&p).entries().len(), 1);
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn picks_up_external_changes() {
        let p = tmp("ext");
        let mut a = UserDict::load(&p);
        let mut b = UserDict::load(&p);
        b.add("てすと", "テスト用語").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        a.reload_if_changed();
        assert_eq!(a.exact("てすと"), vec!["テスト用語"]);
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn tsv_roundtrip_and_validation() {
        let p = tmp("tsv");
        let mut u = UserDict::load(&p);
        assert!(u.add("", "x").is_err());
        assert_eq!(u.import_tsv("かいしゃ\t会社\nこうざ\t講座\nbad line").unwrap(), 2);
        let t = u.export_tsv();
        assert!(t.contains("かいしゃ\t会社"));
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }
}
