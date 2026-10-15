//! バックアップと復元。辞書・学習履歴・設定・用語辞書・言語/フレームワークの語彙を、1つの JSON ファイルにまとめて保存する。
//!
//! 保存先は Google ドライブ(デスクトップ版の同期フォルダ)を自動で探す。見つかれば、そこへ書くだけで Google 側が同期する
//! (OAuth・アカウント情報はこのアプリでは扱わない)。見つからなければ「ドキュメント/aruaru-IME-backup」へ書き、その旨を知らせる。
//! 復元は「統合」(いまの内容に足す)と「置き換え」を選べる。

use crate::userdict::UEntry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const KEEP: usize = 30;
const FOLDER: &str = "aruaru-IME-backup";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Bundle {
    pub format: u32,
    pub created: String,
    pub app_version: String,
    /// 共有データ領域からの相対パス → 内容(UTF-8 のテキスト)。
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Report {
    pub files: usize,
    pub merged_entries: usize,
}

/// バックアップ対象: 単独のファイルと、フォルダ(中の *.json / *.tsv)。
const SINGLE: &[&str] = &["user_dict.json", "learned.json", "settings.json", "wishlist.json"];
const DIRS: &[(&str, &str)] = &[("vocab", "json"), ("words", "tsv")];

/// UTC の `YYYYMMDD-HHMMSS`(外部クレートなしで暦を計算する)。
pub fn stamp(t: SystemTime) -> String {
    let secs = t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // civil_from_days(Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, (rem % 3600) / 60, rem % 60)
}

pub fn create(root: &Path) -> Bundle {
    let mut files = BTreeMap::new();
    for f in SINGLE {
        if let Ok(t) = std::fs::read_to_string(root.join(f)) {
            files.insert((*f).to_string(), t);
        }
    }
    for (dir, ext) in DIRS {
        if let Ok(rd) = std::fs::read_dir(root.join(dir)) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == *ext) {
                    if let (Some(name), Ok(t)) = (p.file_name().and_then(|n| n.to_str()), std::fs::read_to_string(&p)) {
                        files.insert(format!("{dir}/{name}"), t);
                    }
                }
            }
        }
    }
    Bundle { format: 1, created: stamp(SystemTime::now()), app_version: env!("CARGO_PKG_VERSION").to_string(), files }
}

/// 引っ越し用: 普段のバックアップの内容に、取得済みの郵便番号(`postal.tsv`)も加える。
/// 辞書(`dict.bin`)・用語の検索辞書・中国語の辞書は大きく、移った先で自動取得できるので入れない。
pub fn create_move(root: &Path) -> Bundle {
    let mut b = create(root);
    for f in ["postal.tsv"] {
        if let Ok(t) = std::fs::read_to_string(root.join(f)) {
            b.files.insert(f.to_string(), t);
        }
    }
    b
}

/// 引っ越し用の 1 ファイルを書き出す(`aruaru-ime-move-<日時>.json`)。入れる物が無ければ None。
pub fn write_move(root: &Path, dir: &Path) -> std::io::Result<Option<PathBuf>> {
    let b = create_move(root);
    if b.files.is_empty() {
        return Ok(None);
    }
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("aruaru-ime-move-{}.json", b.created));
    std::fs::write(&path, serde_json::to_string(&b).unwrap_or_default())?;
    Ok(Some(path))
}

/// 内容の指紋(変更があったかの判定用)。作成時刻は含めない。
pub fn fingerprint(b: &Bundle) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for (k, v) in &b.files {
        h.update(k.as_bytes());
        h.update([0]);
        h.update(v.as_bytes());
        h.update([0]);
    }
    h.finalize().iter().take(8).map(|x| format!("{x:02x}")).collect()
}

fn safe_relative(name: &str) -> bool {
    !name.is_empty() && !name.contains("..") && !name.starts_with('/') && !name.starts_with('\\') && !name.contains(':')
}

/// 復元する。`replace` が偽なら「統合」(いまの内容を残して足す)、真なら「置き換え」。
pub fn restore(b: &Bundle, root: &Path, replace: bool) -> std::io::Result<Report> {
    let mut rep = Report::default();
    for (name, text) in &b.files {
        if !safe_relative(name) {
            continue; // 細工されたバックアップで、データ領域の外へ書かせない
        }
        let path = root.join(name);
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        let current = std::fs::read_to_string(&path).ok();
        let out = match (replace, current) {
            (true, _) | (false, None) => text.clone(),
            (false, Some(cur)) => match merge_file(name, &cur, text, &mut rep.merged_entries) {
                Some(m) => m,
                None => continue, // 設定など、統合では現在の内容を優先する
            },
        };
        std::fs::write(&path, out)?;
        rep.files += 1;
    }
    Ok(rep)
}

/// 統合: 種類ごとに和集合を作る。統合しない種類(設定)は None。
fn merge_file(name: &str, cur: &str, inc: &str, count: &mut usize) -> Option<String> {
    match name {
        "user_dict.json" => {
            let mut a: Vec<UEntry> = sonic_rs::from_str(cur).unwrap_or_default();
            let b: Vec<UEntry> = sonic_rs::from_str(inc).ok()?;
            for e in b {
                if !a.iter().any(|x| x.reading == e.reading && x.surface == e.surface) {
                    a.push(e);
                    *count += 1;
                }
            }
            serde_json::to_string_pretty(&a).ok()
        }
        "learned.json" => {
            let mut a: crate::session::Learned = sonic_rs::from_str(cur).unwrap_or_default();
            let b: crate::session::Learned = sonic_rs::from_str(inc).ok()?;
            for (r, v) in b.map {
                for (s, n) in v {
                    let slot = a.map.entry(r.clone()).or_default();
                    match slot.iter_mut().find(|(x, _)| *x == s) {
                        Some(e) => e.1 = e.1.max(n),
                        None => {
                            slot.push((s, n));
                            *count += 1;
                        }
                    }
                }
            }
            for (k, v) in b.forms {
                a.forms.entry(k).or_insert(v);
            }
            serde_json::to_string(&a).ok()
        }
        "wishlist.json" => {
            let mut a: Vec<crate::wish::Wish> = sonic_rs::from_str(cur).unwrap_or_default();
            let b: Vec<crate::wish::Wish> = sonic_rs::from_str(inc).ok()?;
            for w in b {
                if !a.iter().any(|x| x.term == w.term && x.category == w.category) {
                    a.push(w);
                    *count += 1;
                }
            }
            serde_json::to_string_pretty(&a).ok()
        }
        n if n.starts_with("vocab/") => {
            let mut a: crate::code::UserFile = sonic_rs::from_str(cur).ok()?;
            let b: crate::code::UserFile = sonic_rs::from_str(inc).ok()?;
            for w in b.words {
                if !a.words.contains(&w) {
                    a.words.push(w);
                    *count += 1;
                }
            }
            for s in b.signals {
                if !a.signals.contains(&s) {
                    a.signals.push(s);
                }
            }
            serde_json::to_string_pretty(&a).ok()
        }
        n if n.starts_with("words/") => {
            let mut lines: Vec<String> = cur.lines().map(str::to_string).collect();
            for l in inc.lines() {
                if !lines.iter().any(|x| x == l) {
                    lines.push(l.to_string());
                    *count += 1;
                }
            }
            Some(lines.join("\n"))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------- 保存先

/// Google ドライブ(デスクトップ版)の同期フォルダを探す。
pub fn drive_root() -> Option<PathBuf> {
    let names = ["My Drive", "マイドライブ"];
    let mut cands: Vec<PathBuf> = vec![];
    if cfg!(windows) {
        for l in b'D'..=b'Z' {
            for n in names {
                cands.push(PathBuf::from(format!("{}:\\{n}", l as char)));
            }
        }
    }
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from) {
        for n in ["Google Drive", "My Drive", "マイドライブ", "Google ドライブ"] {
            cands.push(home.join(n));
        }
        // macOS: ~/Library/CloudStorage/GoogleDrive-<アカウント>/My Drive
        if let Ok(rd) = std::fs::read_dir(home.join("Library").join("CloudStorage")) {
            for e in rd.flatten() {
                if e.file_name().to_string_lossy().starts_with("GoogleDrive-") {
                    for n in names {
                        cands.push(e.path().join(n));
                    }
                }
            }
        }
    }
    cands.into_iter().find(|p| p.is_dir())
}

/// バックアップの保存先。(フォルダ, Google ドライブ内か)
pub fn backup_dir(custom: Option<&str>) -> (PathBuf, bool) {
    if let Some(c) = custom.filter(|c| !c.trim().is_empty()) {
        return (PathBuf::from(c), false);
    }
    if let Some(d) = drive_root() {
        return (d.join(FOLDER), true);
    }
    let docs = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_default().join("Documents");
    (docs.join(FOLDER), false)
}

pub fn list_backups(dir: &Path) -> Vec<(PathBuf, u64)> {
    let mut v: Vec<(PathBuf, u64)> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    n.starts_with("aruaru-ime-backup-") && n.ends_with(".json")
                })
                .map(|e| (e.path(), e.metadata().map(|m| m.len()).unwrap_or(0)))
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.0.cmp(&a.0)); // ファイル名に UTC の日時が入っているので、名前の降順が新しい順
    v
}

/// バックアップを書き出す。直前のバックアップと内容が同じなら書かない(`force` で強制)。書いたファイルを返す。
pub fn write_backup(root: &Path, dir: &Path, force: bool) -> std::io::Result<Option<PathBuf>> {
    let b = create(root);
    if b.files.is_empty() {
        return Ok(None);
    }
    std::fs::create_dir_all(dir)?;
    let fp = fingerprint(&b);
    if !force {
        if let Some((latest, _)) = list_backups(dir).first() {
            if let Some(old) = read_backup(latest) {
                if fingerprint(&old) == fp {
                    return Ok(None);
                }
            }
        }
    }
    let path = dir.join(format!("aruaru-ime-backup-{}.json", b.created));
    std::fs::write(&path, serde_json::to_string(&b).unwrap_or_default())?;
    // 古いものを削除(新しい順に KEEP 個を残す)
    for (old, _) in list_backups(dir).into_iter().skip(KEEP) {
        let _ = std::fs::remove_file(old);
    }
    Ok(Some(path))
}

pub fn read_backup(path: &Path) -> Option<Bundle> {
    sonic_rs::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-bk-{n}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn seed(root: &Path) {
        std::fs::write(root.join("user_dict.json"), r#"[{"reading":"さんじゃんうんから","surface":"三只眼吽迦羅"}]"#).unwrap();
        std::fs::create_dir_all(root.join("vocab")).unwrap();
        std::fs::write(root.join("vocab").join("svelte.json"), r#"{"name":"Svelte","signals":["svelte"],"words":["onMount","writable"]}"#).unwrap();
        std::fs::write(root.join("settings.json"), r#"{"theme":"light"}"#).unwrap();
    }

    #[test]
    fn move_file_carries_everything_to_a_fresh_machine() {
        let (from, to, out) = (tmp("mv-from"), tmp("mv-to"), tmp("mv-out"));
        seed(&from);
        std::fs::write(from.join("postal.tsv"), "1000001	東京都千代田区千代田
").unwrap();
        let f = write_move(&from, &out).unwrap().unwrap();
        // 新しい PC(空のデータ領域)に取り込む
        let b = read_backup(&f).unwrap();
        let rep = restore(&b, &to, true).unwrap();
        assert!(rep.files >= 3, "{rep:?}");
        assert!(std::fs::read_to_string(to.join("user_dict.json")).unwrap().contains("三只眼吽迦羅"));
        assert!(std::fs::read_to_string(to.join("vocab").join("svelte.json")).unwrap().contains("onMount"));
        assert!(std::fs::read_to_string(to.join("postal.tsv")).unwrap().contains("千代田"));
        // 空のデータ領域では、何も書き出さない
        assert!(write_move(&tmp("mv-empty"), &out).unwrap().is_none());
    }

    #[test]
    fn stamp_is_correct_utc() {
        assert_eq!(stamp(SystemTime::UNIX_EPOCH), "19700101-000000");
        assert_eq!(stamp(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_791_640_000)), "20261010-134640");
        assert_eq!(stamp(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(951_782_400)), "20000229-000000", "うるう日");
    }

    #[test]
    fn backup_then_restore_into_empty_dir_recovers_everything() {
        let (a, b, dir) = (tmp("a"), tmp("b"), tmp("dir"));
        seed(&a);
        let path = write_backup(&a, &dir, false).unwrap().unwrap();
        assert!(path.is_file());
        assert_eq!(write_backup(&a, &dir, false).unwrap(), None, "内容が同じなら新しく書かない");
        let bundle = read_backup(&path).unwrap();
        let rep = restore(&bundle, &b, false).unwrap();
        assert_eq!(rep.files, 3);
        assert!(std::fs::read_to_string(b.join("user_dict.json")).unwrap().contains("三只眼吽迦羅"));
        assert!(b.join("vocab").join("svelte.json").is_file());
    }

    #[test]
    fn merge_keeps_current_and_adds_missing_replace_overwrites() {
        let (a, b, dir) = (tmp("ma"), tmp("mb"), tmp("mdir"));
        seed(&a);
        let path = write_backup(&a, &dir, true).unwrap().unwrap();
        std::fs::write(b.join("user_dict.json"), r#"[{"reading":"てすと","surface":"テスト"}]"#).unwrap();
        std::fs::write(b.join("settings.json"), r#"{"theme":"dark"}"#).unwrap();
        let rep = restore(&read_backup(&path).unwrap(), &b, false).unwrap();
        let ud = std::fs::read_to_string(b.join("user_dict.json")).unwrap();
        assert!(ud.contains("テスト") && ud.contains("三只眼吽迦羅"), "統合: 現在の登録を残して足す");
        assert!(std::fs::read_to_string(b.join("settings.json")).unwrap().contains("dark"), "統合では現在の設定を優先");
        assert!(rep.merged_entries >= 1);
        // 置き換え
        restore(&read_backup(&path).unwrap(), &b, true).unwrap();
        assert!(!std::fs::read_to_string(b.join("user_dict.json")).unwrap().contains("テスト"));
        assert!(std::fs::read_to_string(b.join("settings.json")).unwrap().contains("light"));
    }

    #[test]
    fn rejects_path_traversal_in_a_crafted_backup() {
        let (root, outside) = (tmp("root"), tmp("outside"));
        let mut b = Bundle { format: 1, ..Default::default() };
        b.files.insert("../evil.txt".into(), "x".into());
        b.files.insert("C:\\evil.txt".into(), "x".into());
        b.files.insert("user_dict.json".into(), "[]".into());
        let rep = restore(&b, &root.join("data"), true).unwrap();
        assert_eq!(rep.files, 1);
        assert!(!outside.join("evil.txt").exists() && !root.join("evil.txt").exists());
    }

    #[test]
    fn keeps_only_newest_backups() {
        let (a, dir) = (tmp("ka"), tmp("kdir"));
        seed(&a);
        for i in 0..(KEEP + 5) {
            std::fs::write(dir.join(format!("aruaru-ime-backup-20200101-{i:06}.json")), "{}").unwrap();
        }
        write_backup(&a, &dir, true).unwrap();
        let l = list_backups(&dir);
        assert_eq!(l.len(), KEEP);
        let newest = l[0].0.file_name().unwrap().to_string_lossy().to_string();
        assert!(newest.as_str() > "aruaru-ime-backup-20200101-999999.json", "新しいものが残る: {newest}");
    }

    #[test]
    fn falls_back_to_documents_when_no_drive() {
        let (d, in_drive) = backup_dir(Some("X:\\custom"));
        assert_eq!(d, PathBuf::from("X:\\custom"));
        assert!(!in_drive);
    }
}
