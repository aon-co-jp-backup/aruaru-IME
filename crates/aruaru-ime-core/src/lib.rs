//! aruaru-IME core.
pub mod backup;
pub mod capture_store;
pub mod client;
pub mod code;
pub mod codevocab;
pub mod components;
pub mod convert;
pub mod docimport;
pub mod gloss;
pub mod kanji;
pub mod kyujitai;
pub mod mode;
pub mod romaji;
pub mod session;
pub mod translit;
pub mod settings;
pub mod update;
pub mod uservocab;
pub mod userdict;
pub mod ghbackup;
pub mod languages;
pub mod seasonal;
pub mod wish;
pub mod zh;

/// 試験で `ARUARU_DATA_DIR` を書き換えるテスト同士が並行して干渉しないようにする。
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub use convert::{convert, Candidate, Conversion};
pub use mode::{Context, Mode, Prediction};
