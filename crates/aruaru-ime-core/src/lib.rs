//! aruaru-IME core.
pub mod capture_store;
pub mod client;
pub mod components;
pub mod convert;
pub mod docimport;
pub mod kanji;
pub mod mode;
pub mod romaji;
pub mod session;
pub mod settings;
pub mod userdict;

pub use convert::{convert, Candidate, Conversion};
pub use mode::{Context, Mode, Prediction};
