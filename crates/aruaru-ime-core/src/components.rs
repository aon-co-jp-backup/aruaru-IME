//! 共有コンポーネント(aruaru-llm / open-easy-web)の「重複インストールしない」管理。
//!
//! 順序: (1) すでに稼働中(ヘルスチェック成功)なら何もしない → (2) 共有台帳・既知の場所に
//! 実行ファイルがあれば再利用(必要なら起動) → (3) どちらも無いときだけ GitHub Releases から取得。
//! 台帳(`components.json`)は aruaru 系アプリ全体で共有する想定で、誰がいつ入れたかを記録する。
//! 同時に走る別インストーラーとの二重取得は、ロックファイルで防ぐ。
//! open-cuda は aruaru-llm に静的リンク済みのため、aruaru-llm を満たせば open-cuda も満たされる。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Spec {
    pub name: &'static str,
    pub repo: &'static str,
    pub exe: &'static str,
    pub health_url: &'static str,
    /// リリース資産名に含まれる文字列(Windows / Linux)。
    pub asset_win: &'static str,
    pub asset_linux: &'static str,
    /// Windows の Inno Setup が書くアンインストール用キー(`{AppId}_is1`)の AppId。
    pub inno_app_id: &'static str,
}

pub const ARUARU_LLM: Spec = Spec {
    name: "aruaru-llm",
    repo: "aon-co-jp/aruaru-llm",
    exe: "aruaru-llm",
    health_url: "http://127.0.0.1:4600/v1/runtime",
    asset_win: "windows-x86_64.zip",
    asset_linux: "linux-x86_64.tar.gz",
    inno_app_id: "{3E7A1C4F-9B2D-4A6E-8F31-5D9C2E7B0A44}",
};

pub const OPEN_EASY_WEB: Spec = Spec {
    name: "open-easy-web",
    repo: "aon-co-jp/open-easy-web",
    exe: "open-easy-web-server",
    health_url: "http://127.0.0.1:8090/healthz",
    asset_win: "windows",
    asset_linux: "linux",
    inno_app_id: "{B6E2B6B1-6B7B-4C2E-9C77-OPENEASYWEB1}",
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub version: String,
    pub installed_by: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ledger {
    pub components: Vec<Entry>,
}

#[derive(Debug, PartialEq)]
pub enum Outcome {
    /// すでに稼働中。何もしなかった。
    AlreadyRunning,
    /// 既存のインストールを見つけて再利用した(台帳に記録済み)。
    Reused(PathBuf),
    /// 新規に取得した。
    Installed(PathBuf),
    /// 取得できなかった(理由つき)。IME自体は規則だけで動き続ける。
    Unavailable(String),
}

/// 共有データ領域: `%LOCALAPPDATA%\aruaru` / `~/.local/share/aruaru` / macOS は `~/Library/Application Support/aruaru`。
pub fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("ARUARU_DATA_DIR") {
        return PathBuf::from(d);
    }
    if cfg!(windows) {
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into())).join("aruaru")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join("Library/Application Support/aruaru")
    } else {
        PathBuf::from(std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", std::env::var("HOME").unwrap_or_else(|_| ".".into())))).join("aruaru")
    }
}

impl Ledger {
    pub fn path(root: &Path) -> PathBuf {
        root.join("components.json")
    }
    pub fn load(root: &Path) -> Self {
        fs::read_to_string(Self::path(root)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        fs::create_dir_all(root)?;
        fs::write(Self::path(root), serde_json::to_string_pretty(self).unwrap())
    }
    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.components.iter().find(|e| e.name == name && Path::new(&e.path).exists())
    }
    pub fn record(&mut self, e: Entry) {
        self.components.retain(|x| x.name != e.name);
        self.components.push(e);
    }
}

/// ヘルスチェックに応答するか。
pub fn is_running(url: &str) -> bool {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(2)).build().get(url).call().is_ok()
}

fn exe_name(exe: &str) -> String {
    if cfg!(windows) {
        format!("{exe}.exe")
    } else {
        exe.to_string()
    }
}

/// 既知の場所から実行ファイルを探す(共有台帳 → 共有 components → Program Files / PATH)。
pub fn find_installed(spec: &Spec, root: &Path) -> Option<PathBuf> {
    if let Some(e) = Ledger::load(root).find(spec.name) {
        return Some(PathBuf::from(&e.path));
    }
    let exe = exe_name(spec.exe);
    let mut dirs: Vec<PathBuf> = vec![root.join("components").join(spec.name)];
    if cfg!(windows) {
        for v in ["ProgramFiles", "LOCALAPPDATA"] {
            if let Ok(p) = std::env::var(v) {
                let base = if v == "LOCALAPPDATA" { PathBuf::from(p).join("Programs") } else { PathBuf::from(p) };
                dirs.push(base.join(spec.name));
            }
        }
    } else {
        dirs.push(PathBuf::from("/opt").join(spec.name));
        dirs.push(PathBuf::from("/usr/local/bin"));
    }
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    dirs.into_iter().map(|d| d.join(&exe)).find(|p| p.is_file())
}

/// 排他ロック(ロックファイルの作成成功=取得)。古いロック(10分超)は無効とみなして奪う。
pub struct Lock(PathBuf);
impl Lock {
    pub fn acquire(root: &Path, name: &str) -> Option<Lock> {
        fs::create_dir_all(root).ok()?;
        let p = root.join(format!("{name}.lock"));
        if let Ok(m) = fs::metadata(&p) {
            if m.modified().ok().and_then(|t| t.elapsed().ok()).is_some_and(|e| e > Duration::from_secs(600)) {
                let _ = fs::remove_file(&p);
            }
        }
        fs::OpenOptions::new().write(true).create_new(true).open(&p).ok()?;
        Some(Lock(p))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// GitHub Releases の最新版から該当資産を取得して展開する。
fn download(spec: &Spec, dest: &Path) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(600)).user_agent("aruaru-ime").build();
    let rel: serde_json::Value = agent
        .get(&format!("https://api.github.com/repos/{}/releases/latest", spec.repo))
        .call()
        .map_err(|e| format!("release lookup failed: {e}"))?
        .into_json()
        .map_err(|e| e.to_string())?;
    let pat = if cfg!(windows) { spec.asset_win } else { spec.asset_linux };
    let asset = rel["assets"]
        .as_array()
        .and_then(|a| a.iter().find(|x| x["name"].as_str().is_some_and(|n| n.contains(pat) && !n.contains("installer"))))
        .ok_or_else(|| format!("no release asset matching '{pat}' in {}", spec.repo))?;
    let url = asset["browser_download_url"].as_str().ok_or("no download url")?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut agent.get(url).call().map_err(|e| e.to_string())?.into_reader(), &mut buf).map_err(|e| e.to_string())?;
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    if url.ends_with(".zip") {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(buf)).map_err(|e| e.to_string())?;
        z.extract(dest).map_err(|e| e.to_string())?;
    } else {
        let gz = flate2::read::GzDecoder::new(std::io::Cursor::new(buf));
        tar::Archive::new(gz).unpack(dest).map_err(|e| e.to_string())?;
    }
    Ok(rel["tag_name"].as_str().unwrap_or("unknown").to_string())
}

fn locate_exe(dir: &Path, exe: &str) -> Option<PathBuf> {
    let target = exe_name(exe);
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).ok()?.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().is_some_and(|n| n == target.as_str()) {
                return Some(p);
            }
        }
    }
    None
}

/// `<root>/<name>.env`(KEY=VALUE の行)があれば環境変数として渡して起動する(失敗しても無視)。
/// 秘密や利用者固有の設定(open-easy-web の固定アカウント等)は、インストーラーが利用者に入力してもらってここへ書く。
pub fn start_with_env(exe: &Path, env_file: Option<&Path>) {
    let mut cmd = std::process::Command::new(exe);
    cmd.current_dir(exe.parent().unwrap_or(Path::new(".")));
    if let Some(Ok(text)) = env_file.map(fs::read_to_string) {
        for l in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            if let Some((k, v)) = l.split_once('=') {
                cmd.env(k.trim(), v.trim());
            }
        }
    }
    let _ = cmd.spawn();
}

fn start(spec: &Spec, exe: &Path, root: &Path) {
    let env = root.join(format!("{}.env", spec.name));
    // open-easy-web は利用者のアカウント設定が無いと起動できない(パニックする)ため、設定が無ければ起動しない。
    if spec.name == "open-easy-web" && !env.is_file() {
        return;
    }
    start_with_env(exe, env.is_file().then_some(env.as_path()));
}

/// 1コンポーネントを「無ければ入れる・有れば再利用」で満たす。
pub fn ensure(spec: &Spec, root: &Path, installed_by: &str, allow_download: bool) -> Outcome {
    if is_running(spec.health_url) {
        return Outcome::AlreadyRunning;
    }
    let Some(_lock) = Lock::acquire(root, spec.name) else {
        return Outcome::Unavailable(format!("{}: 別のインストーラーが取得中です", spec.name));
    };
    // ロック取得中に別プロセスが完了した場合に備え、再確認
    if is_running(spec.health_url) {
        return Outcome::AlreadyRunning;
    }
    let mut ledger = Ledger::load(root);
    if let Some(exe) = find_installed(spec, root) {
        if ledger.find(spec.name).is_none() {
            ledger.record(Entry { name: spec.name.into(), path: exe.display().to_string(), version: "existing".into(), installed_by: "detected".into() });
            let _ = ledger.save(root);
        }
        start(spec, &exe, root);
        return Outcome::Reused(exe);
    }
    if !allow_download {
        return Outcome::Unavailable(format!("{}: 未インストール(取得は許可されていません)", spec.name));
    }
    let dest = root.join("components").join(spec.name);
    match download(spec, &dest) {
        Ok(tag) => match locate_exe(&dest, spec.exe) {
            Some(exe) => {
                ledger.record(Entry { name: spec.name.into(), path: exe.display().to_string(), version: tag, installed_by: installed_by.into() });
                let _ = ledger.save(root);
                start(spec, &exe, root);
                Outcome::Installed(exe)
            }
            None => Outcome::Unavailable(format!("{}: 展開後に実行ファイルが見つかりません", spec.name)),
        },
        Err(e) => Outcome::Unavailable(format!("{}: {e}", spec.name)),
    }
}

/// aruaru-llm に Qwen を導入・選択させる(強制同梱: 無ければ取得する)。
pub fn ensure_qwen(base_url: &str, model_id: &str) -> Result<(), String> {
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(3600)).build();
    let sel = |agent: &ureq::Agent| agent.post(&format!("{base_url}/v1/qwen/select")).send_json(serde_json::json!({"id": model_id}));
    if sel(&agent).is_ok() {
        return Ok(());
    }
    agent.post(&format!("{base_url}/v1/qwen/install")).send_json(serde_json::json!({"id": model_id})).map_err(|e| format!("qwen install: {e}"))?;
    sel(&agent).map(|_| ()).map_err(|e| format!("qwen select: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("aruaru-comp-{n}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        p
    }

    const DEAD: Spec = Spec { name: "dead", repo: "x/y", exe: "dead-exe", health_url: "http://127.0.0.1:1/none", asset_win: "w", asset_linux: "l", inno_app_id: "" };

    #[test]
    fn ledger_roundtrip_and_stale_entry() {
        let root = tmp("ledger");
        let mut l = Ledger::default();
        l.record(Entry { name: "a".into(), path: "Z:/nonexistent".into(), version: "1".into(), installed_by: "t".into() });
        l.save(&root).unwrap();
        let l2 = Ledger::load(&root);
        assert_eq!(l2.components.len(), 1);
        assert!(l2.find("a").is_none(), "存在しないパスは見つかった扱いにしない");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn lock_is_exclusive() {
        let root = tmp("lock");
        let a = Lock::acquire(&root, "x").unwrap();
        assert!(Lock::acquire(&root, "x").is_none());
        drop(a);
        assert!(Lock::acquire(&root, "x").is_some());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn existing_install_is_reused_not_downloaded() {
        let root = tmp("reuse");
        let dir = root.join("components").join("dead");
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join(exe_name("dead-exe"));
        fs::write(&exe, b"").unwrap();
        // 起動に失敗しても(空ファイル)再利用扱いになり、ダウンロードはしない
        match ensure(&DEAD, &root, "test", false) {
            Outcome::Reused(p) => assert_eq!(p, exe),
            o => panic!("{o:?}"),
        }
        assert_eq!(Ledger::load(&root).components.len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_without_permission_is_unavailable() {
        let root = tmp("missing");
        assert!(matches!(ensure(&DEAD, &root, "test", false), Outcome::Unavailable(_)));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn running_service_short_circuits() {
        // 無関係のサービス(自前の簡易HTTPサーバ)が応答していれば何もしない
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for s in l.incoming().flatten().take(4) {
                use std::io::{Read, Write};
                let mut s = s;
                let mut b = [0u8; 512];
                let _ = s.read(&mut b);
                let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
            }
        });
        let url: &'static str = Box::leak(format!("http://127.0.0.1:{port}/").into_boxed_str());
        let spec = Spec { health_url: url, ..DEAD };
        let root = tmp("running");
        assert_eq!(ensure(&spec, &root, "test", true), Outcome::AlreadyRunning);
        assert!(!root.join("components").exists(), "稼働中なら何も作らない");
        let _ = fs::remove_dir_all(&root);
    }
}
