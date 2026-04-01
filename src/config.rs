use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{KbError, Result};

const KB_DIR_NAME: &str = ".kb";
const GLOBAL_KB_ENV_VAR: &str = "KB_GLOBAL_PATH";

/// Resolve the knowledge base directory path.
///
/// Priority:
/// 1. --local flag → .kb/ in current directory
/// 2. --global flag → ~/.kb/ (or KB_GLOBAL_PATH if set)
/// 3. Walk up from cwd looking for .kb/
/// 4. Fall back to ~/.kb/
pub fn resolve_kb_path(force_local: bool, force_global: bool) -> Result<PathBuf> {
    if force_local {
        let path = std::env::current_dir()?.join(KB_DIR_NAME);
        fs::create_dir_all(&path)?;
        return Ok(path);
    }

    if force_global {
        return global_kb_path();
    }

    // Walk up from cwd looking for .kb/
    if let Some(path) = find_local_kb() {
        return Ok(path);
    }

    // Fall back to global
    global_kb_path()
}

fn global_kb_path() -> Result<PathBuf> {
    let path = if let Ok(env_path) = std::env::var(GLOBAL_KB_ENV_VAR) {
        PathBuf::from(env_path)
    } else {
        home::home_dir()
            .ok_or_else(|| {
                KbError::IoError(std::io::Error::other("could not determine home directory"))
            })?
            .join(KB_DIR_NAME)
    };
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn find_local_kb() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let candidate = current.join(KB_DIR_NAME);
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Return the path to the database file within the KB directory.
pub fn db_path(kb_dir: &Path) -> PathBuf {
    kb_dir.join("kb.db")
}

/// Return the path to the models directory within the KB directory.
pub fn models_dir(kb_dir: &Path) -> PathBuf {
    kb_dir.join("models")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_db_path() {
        let kb_dir = PathBuf::from("/tmp/test_kb");
        assert_eq!(db_path(&kb_dir), PathBuf::from("/tmp/test_kb/kb.db"));
    }

    #[test]
    fn test_models_dir() {
        let kb_dir = PathBuf::from("/tmp/test_kb");
        assert_eq!(models_dir(&kb_dir), PathBuf::from("/tmp/test_kb/models"));
    }

    #[test]
    fn test_force_local_creates_dir() {
        let temp = TempDir::new().unwrap();
        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp).unwrap();
        let result = resolve_kb_path(true, false);
        std::env::set_current_dir(&original_cwd).unwrap();
        let path = result.unwrap();
        assert!(path.ends_with(".kb"));
        assert!(path.is_dir());
    }
}
