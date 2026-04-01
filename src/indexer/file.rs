use std::fs;
use std::path::PathBuf;

use super::{is_binary_extension, ContentSource, Result};
use crate::error::KbError;

pub struct FileSource {
    path: PathBuf,
}

impl FileSource {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn is_binary(&self) -> bool {
        if let Some(ext) = self.path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            return is_binary_extension(&ext);
        }
        self.has_binary_magic()
    }

    fn has_binary_magic(&self) -> bool {
        if let Ok(mut file) = fs::File::open(&self.path) {
            use std::io::Read;
            let mut buf = [0u8; 4];
            if file.read_exact(&mut buf).is_ok() {
                return matches!(
                    &buf,
                    [0x89, 0x50, 0x4E, 0x47]
                        | [0xFF, 0xD8, 0xFF, _]
                        | [0x50, 0x4B, 0x03, 0x04]
                        | [0x25, 0x50, 0x44, 0x46]
                        | [0x7F, 0x45, 0x4C, 0x46]
                        | [0x4D, 0x5A, _, _]
                );
            }
        }
        false
    }
}

impl ContentSource for FileSource {
    fn extract(&self) -> Result<String> {
        if self.is_binary() {
            return Err(KbError::IndexingFailed(
                self.path.display().to_string(),
                "binary file".to_string(),
            ));
        }

        let bytes = fs::read(&self.path).map_err(|e| {
            KbError::IndexingFailed(
                self.path.display().to_string(),
                format!("failed to read file: {}", e),
            )
        })?;

        let _content = String::from_utf8(bytes.clone())
            .or_else(|_err: std::string::FromUtf8Error| {
                Ok(bytes.iter().map(|&b| b as char).collect::<String>())
            })
            .map_err(|_err: std::string::FromUtf8Error| {
                KbError::IndexingFailed(
                    self.path.display().to_string(),
                    "invalid encoding (not UTF-8 or Latin-1)".to_string(),
                )
            })?;

        let content = String::from_utf8(bytes.clone())
            .or_else(|_err: std::string::FromUtf8Error| {
                Ok(bytes.iter().map(|&b| b as char).collect::<String>())
            })
            .map_err(|_err: std::string::FromUtf8Error| {
                KbError::IndexingFailed(
                    self.path.display().to_string(),
                    "invalid encoding (not UTF-8 or Latin-1)".to_string(),
                )
            })?;

        Ok(content)
    }

    fn source_id(&self) -> String {
        self.path.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_text_file_extraction() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let source = FileSource::new(file_path);
        assert!(!source.is_binary());
        assert_eq!(source.extract().unwrap(), "hello world");
    }

    #[test]
    fn test_binary_extension_detection() {
        let source = FileSource::new(PathBuf::from("image.png"));
        assert!(source.is_binary());
    }

    #[test]
    fn test_non_binary_extension() {
        let source = FileSource::new(PathBuf::from("readme.md"));
        assert!(!source.is_binary());
    }
}
