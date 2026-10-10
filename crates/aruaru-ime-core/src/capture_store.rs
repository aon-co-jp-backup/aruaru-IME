//! 画面キャプチャの一時保管庫。
//!
//! キャプチャ画像はOS一時フォルダ内の専用ディレクトリにだけ保存し、
//! 新しい1枚を保存するたびに「60秒より古い」または「30枚を超えた」古いものから自動削除する。
//! 終了時(Drop)にも全て削除する。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const MAX_FILES: usize = 30;
pub const MAX_AGE: Duration = Duration::from_secs(60);

pub struct CaptureStore {
    dir: PathBuf,
    max_files: usize,
    max_age: Duration,
    seq: u64,
}

impl CaptureStore {
    /// `<TEMP>/aruaru-ime-captures-<pid>` を作る。
    pub fn new() -> std::io::Result<Self> {
        Self::sweep_stale(&std::env::temp_dir(), MAX_AGE * 2);
        let dir = std::env::temp_dir().join(format!("aruaru-ime-captures-{}", std::process::id()));
        Self::with_dir(dir, MAX_FILES, MAX_AGE)
    }

    /// 強制終了などで残った過去の専用ディレクトリ(`aruaru-ime-captures-*`)を、更新が `older_than` より古ければ削除する。
    pub fn sweep_stale(tmp: &Path, older_than: Duration) -> usize {
        let own = format!("aruaru-ime-captures-{}", std::process::id());
        let mut n = 0;
        if let Ok(rd) = fs::read_dir(tmp) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if !name.starts_with("aruaru-ime-captures-") || name == own {
                    continue;
                }
                let old = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|d| d > older_than);
                if old && fs::remove_dir_all(e.path()).is_ok() {
                    n += 1;
                }
            }
        }
        n
    }

    pub fn with_dir(dir: PathBuf, max_files: usize, max_age: Duration) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir, max_files, max_age, seq: 0 })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// PNGバイト列を保存し、保存後に古いものを掃除する。保存先パスを返す。
    pub fn save(&mut self, png: &[u8]) -> std::io::Result<PathBuf> {
        self.seq += 1;
        let path = self.dir.join(format!("cap-{:08}.png", self.seq));
        fs::write(&path, png)?;
        self.prune();
        Ok(path)
    }

    /// 期限切れ・枚数超過を古い順に削除する。削除した枚数を返す。
    pub fn prune(&self) -> usize {
        let mut files: Vec<(PathBuf, SystemTime)> = match fs::read_dir(&self.dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let m = e.metadata().ok()?;
                    m.is_file().then(|| (e.path(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
                })
                .collect(),
            Err(_) => return 0,
        };
        // 新しい順(同時刻はファイル名=連番の大きい方が新しい)
        files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
        let now = SystemTime::now();
        let mut removed = 0;
        for (i, (p, t)) in files.iter().enumerate() {
            let too_old = now.duration_since(*t).map(|d| d > self.max_age).unwrap_or(false);
            if i >= self.max_files || too_old {
                if fs::remove_file(p).is_ok() {
                    removed += 1;
                }
            }
        }
        removed
    }

    pub fn count(&self) -> usize {
        fs::read_dir(&self.dir).map(|rd| rd.filter_map(|e| e.ok()).count()).unwrap_or(0)
    }
}

impl Drop for CaptureStore {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("aruaru-ime-test-{}-{}", name, std::process::id()))
    }

    #[test]
    fn keeps_only_newest_ten() {
        let mut s = CaptureStore::with_dir(tmp("count"), MAX_FILES, MAX_AGE).unwrap();
        let mut last = PathBuf::new();
        for i in 0..45u8 {
            last = s.save(&[i]).unwrap();
        }
        assert_eq!(s.count(), 30);
        assert!(last.exists());
        assert!(!s.dir().join("cap-00000001.png").exists());
    }

    #[test]
    fn removes_expired() {
        let mut s = CaptureStore::with_dir(tmp("age"), 100, Duration::from_millis(50)).unwrap();
        s.save(b"a").unwrap();
        std::thread::sleep(Duration::from_millis(120));
        s.save(b"b").unwrap();
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn sweeps_orphans() {
        let root = tmp("sweep");
        fs::create_dir_all(root.join("aruaru-ime-captures-99999999")).unwrap();
        fs::create_dir_all(root.join("other-dir")).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(CaptureStore::sweep_stale(&root, Duration::from_millis(10)), 1);
        assert!(root.join("other-dir").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cleans_on_drop() {
        let d;
        {
            let mut s = CaptureStore::with_dir(tmp("drop"), 10, MAX_AGE).unwrap();
            s.save(b"x").unwrap();
            d = s.dir().to_path_buf();
        }
        assert!(!d.exists());
    }
}
