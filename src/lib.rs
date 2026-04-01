pub mod commands;
pub mod config;
pub mod db;
pub mod embed;
pub mod error;
pub mod indexer;

pub use config::{db_path, models_dir, resolve_kb_path};
pub use db::open_db;
pub use embed::{download_model, Embedder};
pub use error::{KbError, Result};
