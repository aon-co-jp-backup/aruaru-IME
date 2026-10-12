//! 自動アップデート: GitHub Releases の最新版を確認 → 取得 → 版ごとのフォルダへ展開 → 新しい版へ切り替え。
//!
//! 使用中の DLL・実行ファイルは上書きできないため、`<データ領域>/app/versions/<版>/` に並べて置き、
//! IME の登録(Windows は HKCU・管理者権限不要)を新しい版へ向ける。起動中のアプリは再起動すると新しい版になる。
//! 信頼の根拠は GitHub への TLS と、リリースに `.sha256` が付いていればそのハッシュ検証(電子署名ではない)。

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const REPO: &str = "aon-co-jp/aruaru-IME";
/// 展開後に必ず存在していなければならないファイル(無ければ切り替えない)。
pub const REQUIRED_FILES: &[&str] = &["aruaru_ime_tsf.dll", "aruaru-ime-daemon.exe"];

pub type Version = (u32, u32, u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    pub tag: String,
    pub version: Version,
    pub asset_url: String,
    pub sha256_url: Option<String>,
}

/// "v1.2.3" / "1.2.3-beta" → (1,2,3)。解釈できなければ None。
pub fn parse_version(s: &str) -> Option<Version> {
    let core = s.trim().trim_start_matches(['v', 'V']).split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u32>());
    let v = (it.next()?.ok()?, it.next().unwrap_or(Ok(0)).ok()?, it.next().unwrap_or(Ok(0)).ok()?);
    Some(v)
}

pub fn current_version() -> Version {
    parse_version(env!("CARGO_PKG_VERSION")).unwrap_or((0, 0, 0))
}

/// GitHub の release JSON から、現在より新しく、この環境向けの資産(`platform_pat` を名前に含む .zip)を持つ版を選ぶ。
/// 下書きは選ばない。プレリリース(アルファ版)は、`allow_pre`(更新チャンネルが alpha)のときだけ選ぶ。
pub fn pick_release(json: &Value, current: Version, platform_pat: &str, allow_pre: bool) -> Option<Release> {
    if json["draft"].as_bool().unwrap_or(false) || (!allow_pre && json["prerelease"].as_bool().unwrap_or(false)) {
        return None;
    }
    let tag = json["tag_name"].as_str()?.to_string();
    let version = parse_version(&tag)?;
    if version <= current {
        return None;
    }
    let assets = json["assets"].as_array()?;
    let zip = assets.iter().find(|a| a["name"].as_str().is_some_and(|n| n.ends_with(".zip") && n.contains(platform_pat)))?;
    let zip_name = zip["name"].as_str()?;
    let asset_url = zip["browser_download_url"].as_str()?.to_string();
    let sha256_url = assets
        .iter()
        .find(|a| a["name"].as_str() == Some(&format!("{zip_name}.sha256")))
        .and_then(|a| a["browser_download_url"].as_str().map(str::to_string));
    // 取得先は GitHub の配布ドメインだけを許す(release JSON を細工されても任意のサイトへ飛ばない)
    if !is_trusted_url(&asset_url) || sha256_url.as_deref().is_some_and(|u| !is_trusted_url(u)) {
        return None;
    }
    Some(Release { tag, version, asset_url, sha256_url })
}

pub fn is_trusted_url(u: &str) -> bool {
    let Some(rest) = u.strip_prefix("https://") else { return false };
    let host = rest.split('/').next().unwrap_or("");
    host == "github.com" || host.ends_with(".githubusercontent.com")
}

pub fn platform_pattern() -> &'static str {
    if cfg!(windows) {
        "windows-x86_64"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux-x86_64"
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(600)).user_agent("aruaru-ime-updater").build()
}

/// 新しい版があれば返す。ネットワークエラーは Err(静かに次回へ)。リリースがまだ無い場合は Ok(None)。
pub fn check(repo: &str, current: Version) -> Result<Option<Release>, String> {
    // /releases/latest はプレリリースを返さないので、一覧(新しい順)から、現在より新しい最大の版を選ぶ
    let allow_pre = crate::settings::Settings::load().update_channel != "stable";
    let resp = agent().get(&format!("https://api.github.com/repos/{repo}/releases?per_page=15")).call();
    let v: Value = match resp {
        Ok(r) => r.into_json().map_err(|e| e.to_string())?,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    Ok(pick_newest(v.as_array().map(|a| a.as_slice()).unwrap_or(&[]), current, platform_pattern(), allow_pre))
}

/// 一覧から、現在より新しく、この環境向けの資産を持つ最大の版を選ぶ。
pub fn pick_newest(list: &[Value], current: Version, platform_pat: &str, allow_pre: bool) -> Option<Release> {
    list.iter().filter_map(|j| pick_release(j, current, platform_pat, allow_pre)).max_by_key(|r| r.version)
}

fn read_all(url: &str) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    agent().get(url).call().map_err(|e| e.to_string())?.into_reader().take(400 * 1024 * 1024).read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

pub fn sha256_hex(b: &[u8]) -> String {
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// 資産を取得し、`.sha256` があれば検証する。
pub fn download(rel: &Release) -> Result<Vec<u8>, String> {
    let bytes = read_all(&rel.asset_url)?;
    if let Some(u) = &rel.sha256_url {
        let want = String::from_utf8_lossy(&read_all(u)?).split_whitespace().next().unwrap_or("").to_lowercase();
        let got = sha256_hex(&bytes);
        if want != got {
            return Err(format!("ハッシュが一致しません(期待 {want} / 実際 {got})。更新を中止しました"));
        }
    }
    Ok(bytes)
}

/// zip を `versions_root/<tag>/` へ展開する。必須ファイルが揃わなければ失敗(中途半端な版は残さない)。
pub fn stage(zip_bytes: &[u8], versions_root: &Path, tag: &str) -> Result<PathBuf, String> {
    if tag.contains(['/', '\\']) || tag.contains("..") || tag.is_empty() {
        return Err("不正なタグ名です".into());
    }
    let dest = versions_root.join(tag);
    let tmp = versions_root.join(format!(".{tag}.partial"));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).map_err(|e| format!("zip を開けません: {e}"))?;
    z.extract(&tmp).map_err(|e| format!("展開に失敗: {e}"))?; // 親ディレクトリへ抜けるパスは拒否される
    // zip の中身が1つのフォルダに入っている場合は、その中を版フォルダとして扱う
    let root = {
        let entries: Vec<_> = std::fs::read_dir(&tmp).map_err(|e| e.to_string())?.flatten().collect();
        if entries.len() == 1 && entries[0].path().is_dir() { entries[0].path() } else { tmp.clone() }
    };
    for f in REQUIRED_FILES {
        if !root.join(f).is_file() {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(format!("更新ファイルに {f} がありません。更新を中止しました"));
        }
    }
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::rename(&root, &dest).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(dest)
}

/// 古い版のフォルダを消す(使用中で消せないものは次回に回す)。新しい順に `keep` 個と `keep_tag` は残す。
pub fn prune(versions_root: &Path, keep_tag: &str, keep: usize) -> usize {
    let mut dirs: Vec<(Version, PathBuf, String)> = std::fs::read_dir(versions_root)
        .map(|rd| rd.flatten().filter(|e| e.path().is_dir()).filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            parse_version(&name).map(|v| (v, e.path(), name))
        }).collect())
        .unwrap_or_default();
    dirs.sort_by(|a, b| b.0.cmp(&a.0));
    let mut removed = 0;
    for (_, p, name) in dirs.into_iter().skip(keep) {
        if name != keep_tag && std::fs::remove_dir_all(&p).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// 利用者単位(HKCU、管理者権限不要)で IME の登録を新しい DLL へ向ける。新しく起動するアプリから新しい版になる。
#[cfg(windows)]
pub fn activate(dll: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    let key = r"HKCU\Software\Classes\CLSID\{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}\InprocServer32";
    let run = |args: &[&str]| -> std::io::Result<()> {
        let st = std::process::Command::new("reg").args(args).creation_flags(0x0800_0000).status()?;
        if st.success() { Ok(()) } else { Err(std::io::Error::other("reg.exe に失敗しました")) }
    };
    run(&["add", key, "/ve", "/d", &dll.display().to_string(), "/f"])?;
    run(&["add", key, "/v", "ThreadingModel", "/d", "Apartment", "/f"])
}

#[cfg(not(windows))]
pub fn activate(_dll: &Path) -> std::io::Result<()> {
    Ok(())
}

/// 更新の状況(`update.json`)。
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Status {
    pub checked_at: u64,
    pub installed_tag: Option<String>,
    pub last_error: Option<String>,
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 1回分の更新処理: 確認 → 取得 → 展開 → 切り替え。更新したら新しい版のフォルダを返す。
pub fn run_once(repo: &str, app_root: &Path) -> Result<Option<(String, PathBuf)>, String> {
    let Some(rel) = check(repo, current_version())? else { return Ok(None) };
    let versions = app_root.join("versions");
    std::fs::create_dir_all(&versions).map_err(|e| e.to_string())?;
    if versions.join(&rel.tag).join(REQUIRED_FILES[0]).is_file() {
        // 取得済み: 切り替えだけやり直す
        let dir = versions.join(&rel.tag);
        activate(&dir.join(REQUIRED_FILES[0])).map_err(|e| e.to_string())?;
        return Ok(Some((rel.tag, dir)));
    }
    let bytes = download(&rel)?;
    let dir = stage(&bytes, &versions, &rel.tag)?;
    activate(&dir.join(REQUIRED_FILES[0])).map_err(|e| e.to_string())?;
    prune(&versions, &rel.tag, 2);
    Ok(Some((rel.tag, dir)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    fn release_json(tag: &str, pre: bool) -> Value {
        json!({
            "tag_name": tag, "draft": false, "prerelease": pre,
            "assets": [
                {"name": "aruaru-ime-windows-x86_64.zip", "browser_download_url": "https://github.com/aon-co-jp/aruaru-IME/releases/download/x/aruaru-ime-windows-x86_64.zip"},
                {"name": "aruaru-ime-windows-x86_64.zip.sha256", "browser_download_url": "https://github.com/aon-co-jp/aruaru-IME/releases/download/x/aruaru-ime-windows-x86_64.zip.sha256"},
                {"name": "aruaru-ime-linux-x86_64.zip", "browser_download_url": "https://github.com/a/b/releases/download/x/l.zip"}
            ]
        })
    }

    #[test]
    fn version_parsing_and_ordering() {
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("0.10"), Some((0, 10, 0)));
        assert_eq!(parse_version("1.0.0-beta.1"), Some((1, 0, 0)));
        assert_eq!(parse_version("nightly"), None);
        assert!(parse_version("v0.10.0") > parse_version("v0.9.9"), "数値として比較する(文字列順ではない)");
    }

    #[test]
    fn picks_only_newer_stable_for_this_platform() {
        let cur = (0, 1, 0);
        let r = pick_release(&release_json("v0.2.0", false), cur, "windows-x86_64", false).unwrap();
        assert_eq!(r.tag, "v0.2.0");
        assert!(r.sha256_url.is_some());
        assert!(pick_release(&release_json("v0.1.0", false), cur, "windows-x86_64", false).is_none(), "同じ版は更新しない");
        assert!(pick_release(&release_json("v0.0.9", false), cur, "windows-x86_64", false).is_none(), "古い版へは戻さない");
        assert!(pick_release(&release_json("v0.3.0", true), cur, "windows-x86_64", false).is_none(), "プレリリースは対象外");
        assert!(pick_release(&release_json("v0.2.0", false), cur, "macos", false).is_none(), "この環境向けの資産が無ければ更新しない");
    }

    #[test]
    fn alpha_channel_takes_prereleases_and_picks_the_newest() {
        let cur = (0, 2, 7);
        assert!(pick_release(&release_json("v0.2.8", true), cur, "windows-x86_64", true).is_some(), "alpha チャンネルはプレリリースも対象");
        assert!(pick_release(&release_json("v0.2.8", true), cur, "windows-x86_64", false).is_none(), "stable チャンネルは対象外");
        let list = vec![release_json("v0.2.8", true), release_json("v0.2.10", true), release_json("v0.2.9", true), release_json("v0.2.7", true)];
        assert_eq!(pick_newest(&list, cur, "windows-x86_64", true).unwrap().tag, "v0.2.10", "数値として最大");
        assert!(pick_newest(&list, (0, 2, 10), "windows-x86_64", true).is_none());
    }

    #[test]
    fn rejects_untrusted_download_hosts() {
        let mut j = release_json("v0.2.0", false);
        j["assets"][0]["browser_download_url"] = json!("https://evil.example.com/a-windows-x86_64.zip");
        assert!(pick_release(&j, (0, 1, 0), "windows-x86_64", false).is_none());
        assert!(!is_trusted_url("http://github.com/x"), "http は拒否");
        assert!(!is_trusted_url("https://github.com.evil.com/x"));
        assert!(is_trusted_url("https://objects.githubusercontent.com/x"));
    }

    fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
            for (n, b) in files {
                z.start_file(*n, zip::write::SimpleFileOptions::default()).unwrap();
                z.write_all(b).unwrap();
            }
            z.finish().unwrap();
        }
        buf
    }

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("aruaru-upd-{n}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn stages_complete_release_and_rejects_incomplete() {
        let root = tmp("stage");
        let ok = make_zip(&[("aruaru_ime_tsf.dll", b"dll"), ("aruaru-ime-daemon.exe", b"exe"), ("README.txt", b"r")]);
        let dir = stage(&ok, &root, "v0.2.0").unwrap();
        assert!(dir.join("aruaru_ime_tsf.dll").is_file());
        // 1フォルダに入った zip も扱える
        let nested = make_zip(&[("aruaru-ime/aruaru_ime_tsf.dll", b"dll"), ("aruaru-ime/aruaru-ime-daemon.exe", b"exe")]);
        assert!(stage(&nested, &root, "v0.2.1").unwrap().join("aruaru-ime-daemon.exe").is_file());
        // 必須ファイルが欠けた更新は、版フォルダを作らずに失敗する
        let bad = make_zip(&[("aruaru_ime_tsf.dll", b"dll")]);
        assert!(stage(&bad, &root, "v0.3.0").is_err());
        assert!(!root.join("v0.3.0").exists());
        assert!(stage(&ok, &root, "../evil").is_err(), "タグ名経由のパス抜けは拒否");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prune_keeps_newest_and_current() {
        let root = tmp("prune");
        for t in ["v0.1.0", "v0.2.0", "v0.3.0", "v0.4.0"] {
            std::fs::create_dir_all(root.join(t)).unwrap();
        }
        assert_eq!(prune(&root, "v0.4.0", 2), 2);
        assert!(root.join("v0.4.0").exists() && root.join("v0.3.0").exists());
        assert!(!root.join("v0.1.0").exists() && !root.join("v0.2.0").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn sha256_known_value() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
