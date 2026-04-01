use rusqlite::Connection;

use crate::db::queries::{search_entries, SearchResult};
use crate::embed::Embedder;
use crate::error::Result;

pub fn search(
    conn: &Connection,
    embedder: &mut Embedder,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let query_embedding = embedder.embed(query)?;
    let results = search_entries(conn, &query_embedding, query, limit)?;

    if results.is_empty() {
        println!("No results found.");
    } else {
        for (i, result) in results.iter().enumerate() {
            println!(
                "\n--- Result {} (score: {:.3}, id: {}) ---",
                i + 1,
                result.score,
                result.id
            );
            println!("Source: {}", result.source);
            println!("{}", result.snippet);
        }
        println!("\n{} result(s) found.", results.len());
    }

    Ok(results)
}
