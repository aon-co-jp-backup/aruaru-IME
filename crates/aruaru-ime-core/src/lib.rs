//! aruaru-IME core.
pub mod capture_store;
pub mod client;
pub mod convert;
pub mod kanji;
pub mod mode;
pub mod romaji;

pub use convert::{convert, Candidate, Conversion};
pub use mode::{Context, Mode, Prediction};
