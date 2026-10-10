//! GitHub へのバックアップ(任意)。利用者が自分のトークンと保存先リポジトリ(`owner/name`)を指定する。
//! トークンは設定ファイルには書かず、`<データ領域>/github-token.txt`(利用者が置く。値は表示しない)から読む。
//! 公開リポジトリのときは、入力履歴(learned.json)と設定を含めない(個人情報の流出を防ぐ)。

use crate::backup::{self, Bundle};
use base64::Engine;
use std::path::Path;

const API: &str = "https://api.github.com";
const REMOTE_FILE: &str = "aruaru-ime-backup.json";

pub fn token_path(root: &Path) -> std::path::PathBuf {
    root.join("github-token.txt")
}

fn token(root: &Path) -> Result<String, String> {
    let t = std::fs::read_to_string(token_path(root)).map_err(|_| format!("トークンのファイルがありません: {}", token_path(root).display()))?;
    let t = t.trim().to_string();
    if t.is_empty() { Err("トークンのファイルが空です".into()) } else { Ok(t) }
}

pub fn valid_repo(repo: &str) -> bool {
    let mut it = repo.split('/');
    let ok = |s: &str| !s.is_empty() && s.len() <= 100 && s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) && s != "." && s != "..";
    matches!((it.next(), it.next(), it.next()), (Some(o), Some(n), None) if ok(o) && ok(n))
}

/// 公開用に、個人の入力履歴・設定を取り除く。
pub fn sanitize_for_public(b: &mut Bundle) {
    b.files.retain(|k, _| k != "learned.json" && k != "settings.json");
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(60)).user_agent("aruaru-ime").build()
}

/// リポジトリが公開かどうか(GitHub に問い合わせる)。
fn is_public(root: &Path, repo: &str) -> Result<bool, String> {
    let r: serde_json::Value = agent()
        .get(&format!("{API}/repos/{repo}"))
        .set("Authorization", &format!("Bearer {}", token(root)?))
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("リポジトリを確認できません: {e}"))?
        .into_json()
        .map_err(|e| e.to_string())?;
    Ok(!r["private"].as_bool().unwrap_or(true))
}

/// バックアップを GitHub へ上げる。リポジトリが公開なら自動で個人データを除く。
pub fn push(root: &Path, repo: &str) -> Result<String, String> {
    if !valid_repo(repo) {
        return Err("リポジトリは owner/name の形で指定してください".into());
    }
    let tok = token(root)?;
    let public = is_public(root, repo)?;
    let mut b = backup::create(root);
    if public {
        sanitize_for_public(&mut b);
    }
    if b.files.is_empty() {
        return Err("バックアップするデータがありません".into());
    }
    let body = serde_json::to_string(&b).map_err(|e| e.to_string())?;
    let url = format!("{API}/repos/{repo}/contents/{REMOTE_FILE}");
    let a = agent();
    // 既存ファイルの sha(更新に必要)
    let sha = a
        .get(&url)
        .set("Authorization", &format!("Bearer {tok}"))
        .call()
        .ok()
        .and_then(|r| r.into_json::<serde_json::Value>().ok())
        .and_then(|v| v["sha"].as_str().map(String::from));
    let mut req = serde_json::json!({
        "message": format!("backup {}", b.created),
        "content": base64::engine::general_purpose::STANDARD.encode(body),
    });
    if let Some(s) = sha {
        req["sha"] = s.into();
    }
    a.put(&url)
        .set("Authorization", &format!("Bearer {tok}"))
        .set("Accept", "application/vnd.github+json")
        .send_json(req)
        .map_err(|e| format!("アップロードに失敗: {e}"))?;
    Ok(format!("{repo} へ保存しました({}、{})", b.files.len(), if public { "公開リポジトリのため履歴・設定は除外" } else { "非公開" }))
}

/// GitHub から取り込む(`replace` なら置き換え、でなければ統合)。
pub fn pull(root: &Path, repo: &str, replace: bool) -> Result<backup::Report, String> {
    if !valid_repo(repo) {
        return Err("リポジトリは owner/name の形で指定してください".into());
    }
    let tok = token(root)?;
    let v: serde_json::Value = agent()
        .get(&format!("{API}/repos/{repo}/contents/{REMOTE_FILE}"))
        .set("Authorization", &format!("Bearer {tok}"))
        .call()
        .map_err(|e| format!("取得に失敗: {e}"))?
        .into_json()
        .map_err(|e| e.to_string())?;
    let raw = v["content"].as_str().ok_or("内容がありません")?.replace('\n', "");
    let bytes = base64::engine::general_purpose::STANDARD.decode(raw).map_err(|e| e.to_string())?;
    let b: Bundle = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    backup::restore(&b, root, replace).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_names_are_validated() {
        assert!(valid_repo("aon-co-jp/my.backup_1"));
        for bad in ["", "a", "a/b/c", "../x", "a/..", "a b/c", "a/b?x=1"] {
            assert!(!valid_repo(bad), "{bad}");
        }
    }

    #[test]
    fn public_bundle_drops_private_history() {
        let mut b = Bundle::default();
        for k in ["learned.json", "settings.json", "user_dict.json"] {
            b.files.insert(k.into(), "{}".into());
        }
        sanitize_for_public(&mut b);
        assert_eq!(b.files.keys().collect::<Vec<_>>(), vec!["user_dict.json"]);
    }
}
