use crate::error::{KbError, Result};

mod file;
mod text;
mod url;

pub use file::FileSource;
pub use text::TextSource;
pub use url::UrlSource;

pub trait ContentSource {
    fn extract(&self) -> Result<String>;
    fn source_id(&self) -> String;
}

pub fn detect_source(input: &str) -> Box<dyn ContentSource> {
    if input.starts_with("http://") || input.starts_with("https://") {
        return Box::new(UrlSource::new(input));
    }

    let path = std::path::Path::new(input);
    if path.exists() {
        return Box::new(FileSource::new(path.to_path_buf()));
    }

    Box::new(TextSource::new(input.to_string()))
}

const BINARY_EXTENSIONS: &[&str] = &[
    "pdf", "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "ico", "svg", "exe", "dll", "so",
    "dylib", "a", "lib", "zip", "tar", "gz", "bz2", "xz", "rar", "7z", "mp3", "mp4", "avi", "mov",
    "mkv", "wav", "flac", "bin", "dat", "db", "sqlite", "wasm", "o", "class",
];

pub fn is_binary_extension(ext: &str) -> bool {
    BINARY_EXTENSIONS.contains(&ext)
}
