use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{KbError, Result};

const MODEL_URL: &str = "https://huggingface.co/optimum/all-MiniLM-L6-v2/resolve/main/model.onnx";

const TOKENIZER_URL: &str =
    "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json";

const MODEL_SHA256: &str = "4a64cee3d4134bbdc86eed96e1a660efec58975417204ecfcf134140edb6e0e2";

const TOKENIZER_SHA256: &str = "be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037";

pub fn model_path(models_dir: &Path) -> PathBuf {
    models_dir.join("all-MiniLM-L6-v2")
}

pub fn download_model(models_dir: &Path) -> Result<PathBuf> {
    let dest = model_path(models_dir);
    fs::create_dir_all(&dest)?;

    let model_file = dest.join("model.onnx");
    let tokenizer_file = dest.join("tokenizer.json");

    if !model_file.exists() {
        download_file(MODEL_URL, &model_file, MODEL_SHA256)?;
    }

    if !tokenizer_file.exists() {
        download_file(TOKENIZER_URL, &tokenizer_file, TOKENIZER_SHA256)?;
    }

    Ok(dest)
}

fn download_file(url: &str, dest: &Path, expected_sha: &str) -> Result<()> {
    eprintln!("Downloading {}...", url);

    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| KbError::HttpError(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| KbError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(KbError::ModelDownloadFailed(format!(
            "HTTP {}: {}",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|e| KbError::HttpError(e.to_string()))?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual_sha = format!("{:x}", hasher.finalize());

    if actual_sha != expected_sha {
        return Err(KbError::ModelDownloadFailed(format!(
            "SHA256 mismatch: expected {}, got {}",
            expected_sha, actual_sha
        )));
    }

    fs::write(dest, &bytes)?;
    eprintln!("Downloaded to {}", dest.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_path() {
        let models_dir = PathBuf::from("/tmp/kb/models");
        let path = model_path(&models_dir);
        assert_eq!(path, PathBuf::from("/tmp/kb/models/all-MiniLM-L6-v2"));
    }
}
