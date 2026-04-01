use std::path::Path;

use rusqlite::Connection;

use crate::error::{KbError, Result};

pub mod queries;

const SCHEMA_SQL: &str = include_str!("schema.sql");

/// Open a connection to the knowledge base database, initializing the schema if needed.
pub fn open_db(db_path: &Path) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    for stmt in SCHEMA_SQL.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        conn.execute_batch(stmt)?;
    }

    // Create FTS triggers if they don't already exist
    let triggers = [
        (
            "entries_ai",
            "CREATE TRIGGER entries_ai AFTER INSERT ON entries BEGIN
                INSERT INTO entries_fts(rowid, content) VALUES (new.id, new.content);
            END",
        ),
        (
            "entries_ad",
            "CREATE TRIGGER entries_ad AFTER DELETE ON entries BEGIN
                INSERT INTO entries_fts(entries_fts, rowid, content) VALUES('delete', old.id, old.content);
            END",
        ),
        (
            "entries_au",
            "CREATE TRIGGER entries_au AFTER UPDATE ON entries BEGIN
                INSERT INTO entries_fts(entries_fts, rowid, content) VALUES('delete', old.id, old.content);
                INSERT INTO entries_fts(rowid, content) VALUES (new.id, new.content);
            END",
        ),
    ];

    for (name, sql) in &triggers {
        let exists: bool = conn.query_row(
            "SELECT count(*) > 0 FROM sqlite_master WHERE type='trigger' AND name=?",
            [name],
            |r| r.get(0),
        )?;
        if !exists {
            conn.execute_batch(sql)?;
        }
    }

    Ok(conn)
}

/// Serialize an f32 vector to a BLOB (Vec<u8>) for SQLite storage.
pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Deserialize a BLOB from SQLite back to an f32 vector.
pub fn deserialize_embedding(blob: &[u8]) -> Result<Vec<f32>> {
    if blob.len() % 4 != 0 {
        return Err(KbError::EmbeddingError(format!(
            "invalid embedding blob length: {}",
            blob.len()
        )));
    }
    Ok(blob
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_db_creates_schema() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();

        // Verify entries table exists
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='entries'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify FTS table exists
        let fts_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='entries_fts'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fts_count, 1);
    }

    #[test]
    fn test_serialize_deserialize_embedding() {
        let original = vec![0.1f32, 0.2, 0.3, -0.5];
        let blob = serialize_embedding(&original);
        let restored = deserialize_embedding(&blob).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_deserialize_invalid_blob() {
        let result = deserialize_embedding(&[1, 2, 3]); // not divisible by 4
        assert!(result.is_err());
    }
}
