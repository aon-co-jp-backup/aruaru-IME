//! 版の食い違い防止: 版を書いている場所が、ワークスペースの版(ルートの Cargo.toml)と一致しているかを確かめる。
//! リリースのたびに起きがちな「Android の画面・インストーラーの既定値・配布ページだけ古い版のまま」を、`cargo test` で検出する。

use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// `version = "x.y.z"` の最初の行(ルートの Cargo.toml では [workspace.package] のもの)。
fn workspace_version() -> String {
    let t = read("Cargo.toml");
    let line = t.lines().find(|l| l.trim_start().starts_with("version") && l.contains('"')).expect("version in Cargo.toml");
    line.split('"').nth(1).expect("quoted version").to_string()
}

fn v(s: &str) -> (u32, u32, u32) {
    let mut it = s.trim_start_matches('v').split('.').map(|x| x.parse::<u32>().unwrap_or(0));
    (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0))
}

#[test]
fn android_version_name_matches_workspace() {
    let g = read("android/app/build.gradle");
    let line = g.lines().find(|l| l.contains("versionName")).expect("versionName");
    let name = line.split('\'').nth(1).or_else(|| line.split('"').nth(1)).expect("quoted versionName");
    assert_eq!(name, workspace_version(), "android/app/build.gradle の versionName が Cargo.toml の版と違います");
}

#[test]
fn installer_default_version_matches_workspace() {
    let s = read("installer/aruaru-ime.iss");
    let line = s.lines().find(|l| l.contains("#define MyAppVersion")).expect("MyAppVersion default");
    let ver = line.split('"').nth(1).expect("quoted default");
    assert_eq!(ver, workspace_version(), "installer/aruaru-ime.iss の MyAppVersion(既定値)が Cargo.toml の版と違います");
}

#[test]
fn download_page_points_to_one_release_not_newer_than_workspace() {
    let h = read("webpage/index.html");
    // `releases/tag/vX` と `releases/download/vX/…` のどちらからも、版のタグ(vX.Y.Z)だけを集める
    let mut found: Vec<String> = vec![];
    for part in h.split("/releases/").skip(1) {
        let seg: Vec<&str> = part.split(['/', '"']).collect();
        if let Some(t) = seg.iter().find(|s| s.starts_with('v') && s[1..].chars().next().map_or(false, |c| c.is_ascii_digit())) {
            found.push((*t).to_string());
        }
    }
    assert!(!found.is_empty(), "配布ページにリリースへのリンクがありません");
    found.dedup();
    assert!(found.iter().all(|t| *t == found[0]), "配布ページのリンクが複数の版にまたがっています: {found:?}");
    assert!(v(&found[0]) <= v(&workspace_version()), "配布ページが、まだ無い版 {} を指しています", found[0]);
}
