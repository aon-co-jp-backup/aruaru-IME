//! 最後に選ばれた入力方式の記憶。aruaru-IME を選んだら記録し、次回の起動(再起動後)に常駐エンジンが自動で選び直す。
//! 記録: `<データ領域>/last-ime.txt` に `aruaru <プロファイルGUID(32桁16進)>` か `other`。

use std::path::{Path, PathBuf};

pub fn path(root: &Path) -> PathBuf {
    root.join("last-ime.txt")
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Last {
    /// aruaru-IME(言語プロファイル)が最後に選ばれた。
    Aruaru(u128),
    /// ほかの入力方式が最後に選ばれた(自動では選ばない)。
    Other,
    /// 記録なし。
    Unknown,
}

pub fn read(root: &Path) -> Last {
    let t = std::fs::read_to_string(path(root)).unwrap_or_default();
    let t = t.trim();
    if let Some(g) = t.strip_prefix("aruaru ") {
        if let Ok(v) = u128::from_str_radix(g.trim(), 16) {
            return Last::Aruaru(v);
        }
    }
    if t == "other" { Last::Other } else { Last::Unknown }
}

pub fn write(root: &Path, last: Last) {
    let body = match last {
        Last::Aruaru(g) => format!("aruaru {g:032x}"),
        Last::Other => "other".to_string(),
        Last::Unknown => return,
    };
    if read(root) == last {
        return;
    }
    let _ = std::fs::create_dir_all(root);
    let _ = std::fs::write(path(root), body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let d = std::env::temp_dir().join(format!("aruaru-last-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert_eq!(read(&d), Last::Unknown);
        write(&d, Last::Aruaru(0xC1D3E5F72A4B4C6D8E901F2A3B4C5D6E));
        assert_eq!(read(&d), Last::Aruaru(0xC1D3E5F72A4B4C6D8E901F2A3B4C5D6E));
        write(&d, Last::Other);
        assert_eq!(read(&d), Last::Other, "ほかを選んだら自動選択をやめる");
    }
}
