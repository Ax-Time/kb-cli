use rusqlite::Connection;

use crate::db::{deserialize_embedding, serialize_embedding};
use crate::error::Result;

pub struct SearchResult {
    pub id: i64,
    pub source: String,
    pub snippet: String,
    pub score: f64,
}

pub struct Entry {
    pub id: i64,
    pub source: String,
    pub created_at: String,
}

pub fn list_entries(conn: &Connection) -> Result<Vec<Entry>> {
    let mut stmt = conn.prepare("SELECT id, source, created_at FROM entries ORDER BY id")?;
    let entries = stmt
        .query_map([], |row| {
            Ok(Entry {
                id: row.get(0)?,
                source: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(entries)
}

pub fn insert_entry(
    conn: &Connection,
    source: &str,
    content: &str,
    embedding: &[f32],
) -> Result<i64> {
    let blob = serialize_embedding(embedding);
    conn.execute(
        "INSERT INTO entries (source, content, embedding) VALUES (?1, ?2, ?3)",
        rusqlite::params![source, content, blob],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    let rows = conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
    if rows == 0 {
        return Err(crate::error::KbError::NotFound(id));
    }
    Ok(())
}

pub fn search_entries(
    conn: &Connection,
    query_embedding: &[f32],
    query_text: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let fts_results: Vec<(i64, String, String, f64)> = {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.source, substr(e.content, 1, 200), rank
             FROM entries e
             JOIN entries_fts f ON e.id = f.rowid
             WHERE entries_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![query_text, limit as i64], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, f64>(3).unwrap_or(0.0),
            ))
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let all_entries: Vec<(i64, Vec<u8>)> = {
        let mut stmt = conn.prepare("SELECT id, embedding FROM entries")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let mut similarity_scores: std::collections::HashMap<i64, f64> =
        std::collections::HashMap::new();
    for (id, blob) in &all_entries {
        if let Ok(entry_embedding) = deserialize_embedding(blob) {
            let sim = cosine_similarity(query_embedding, &entry_embedding);
            similarity_scores.insert(*id, sim);
        }
    }

    let max_fts_rank = fts_results
        .iter()
        .map(|(_, _, _, rank)| rank.abs())
        .fold(0.0f64, f64::max);

    let mut results: Vec<SearchResult> = fts_results
        .into_iter()
        .map(|(id, source, snippet, fts_rank)| {
            let normalized_fts = if max_fts_rank > 0.0 {
                1.0 - (fts_rank.abs() / max_fts_rank)
            } else {
                0.0
            };
            let cosine = similarity_scores.get(&id).copied().unwrap_or(0.0);
            let score = 0.5 * normalized_fts + 0.5 * cosine;
            SearchResult {
                id,
                source,
                snippet,
                score,
            }
        })
        .collect();

    for (id, sim) in &similarity_scores {
        if !results.iter().any(|r| r.id == *id) && *sim > 0.0 {
            let (source, snippet): (String, String) = conn.query_row(
                "SELECT source, substr(content, 1, 200) FROM entries WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            results.push(SearchResult {
                id: *id,
                source,
                snippet,
                score: 0.5 * sim,
            });
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);

    Ok(results)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as f64) * (*y as f64))
        .sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db;
    use tempfile::TempDir;

    fn setup_db() -> (TempDir, Connection) {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        (temp, conn)
    }

    #[test]
    fn test_insert_and_delete() {
        let (_temp, conn) = setup_db();
        let embedding = vec![0.1f32; 384];
        let id = insert_entry(&conn, "test.txt", "hello world", &embedding).unwrap();
        assert!(id > 0);

        delete_entry(&conn, id).unwrap();

        let result = delete_entry(&conn, id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0f32, 0.0, 1.0];
        let b = vec![1.0f32, 0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_search_returns_results() {
        let (_temp, conn) = setup_db();
        let embedding = vec![0.1f32; 384];

        insert_entry(&conn, "file1.txt", "Rust programming language", &embedding).unwrap();
        insert_entry(
            &conn,
            "file2.txt",
            "Python programming language",
            &embedding,
        )
        .unwrap();
        insert_entry(&conn, "file3.txt", "Cooking recipes", &embedding).unwrap();

        let query_emb = vec![0.1f32; 384];
        let results = search_entries(&conn, &query_emb, "Rust", 10).unwrap();
        assert!(!results.is_empty());
    }
}
