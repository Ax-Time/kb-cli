use std::path::Path;

use rayon::prelude::*;
use rusqlite::Connection;
use std::sync::Arc;

use crate::embed::Embedder;
use crate::error::Result;
use crate::indexer::{ContentSource, FileSource};

pub fn add(conn: &Connection, embedder: &mut Embedder, input: &str, recursive: bool) -> Result<()> {
    let source = crate::indexer::detect_source(input);

    if recursive {
        let path = Path::new(input);
        if path.is_dir() {
            add_directory(conn, embedder, path)?;
            return Ok(());
        }
        eprintln!("Warning: --recursive flag given but input is not a directory");
    }

    add_single(conn, embedder, source.as_ref())
}

fn add_single(
    conn: &Connection,
    embedder: &mut Embedder,
    source: &dyn ContentSource,
) -> Result<()> {
    let content = source.extract()?;
    if content.trim().is_empty() {
        eprintln!("Warning: {} is empty, skipping", source.source_id());
        return Ok(());
    }

    let embedding = embedder.embed(&content)?;
    let id = crate::db::queries::insert_entry(conn, &source.source_id(), &content, &embedding)?;

    println!("Indexed: {} (id={})", source.source_id(), id);
    Ok(())
}

struct IndexResult {
    source: String,
    content: String,
    embedding: Vec<f32>,
    success: bool,
    error: Option<String>,
}

fn add_directory(conn: &Connection, embedder: &mut Embedder, dir: &Path) -> Result<()> {
    let walker = ignore::WalkBuilder::new(dir).hidden(false).build();

    let paths: Vec<_> = walker
        .filter_map(|entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Warning: {}", e);
                    return None;
                }
            };
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                return None;
            }
            let path = entry.path().to_path_buf();
            let source = FileSource::new(path.clone());
            if source.is_binary() {
                return None;
            }
            Some(path)
        })
        .collect();

    if paths.is_empty() {
        println!("No files to index.");
        return Ok(());
    }

    println!("Processing {} files...", paths.len());

    let embedder = Arc::new(embedder);
    let num_threads = rayon::current_num_threads();

    let results: Vec<IndexResult> = paths
        .par_iter()
        .map(|path| {
            let source = FileSource::new(path.clone());
            match source.extract() {
                Ok(content) if content.trim().is_empty() => IndexResult {
                    source: path.display().to_string(),
                    content,
                    embedding: vec![],
                    success: false,
                    error: Some("empty content".to_string()),
                },
                Ok(content) => match embedder.embed(&content) {
                    Ok(embedding) => IndexResult {
                        source: path.display().to_string(),
                        content,
                        embedding,
                        success: true,
                        error: None,
                    },
                    Err(e) => IndexResult {
                        source: path.display().to_string(),
                        content,
                        embedding: vec![],
                        success: false,
                        error: Some(e.to_string()),
                    },
                },
                Err(e) => IndexResult {
                    source: path.display().to_string(),
                    content: String::new(),
                    embedding: vec![],
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect();

    let mut count = 0;
    let mut errors = 0;

    for result in results {
        if result.success {
            match crate::db::queries::insert_entry(
                conn,
                &result.source,
                &result.content,
                &result.embedding,
            ) {
                Ok(id) => {
                    println!("Indexed: {} (id={})", result.source, id);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: {}: database error: {}", result.source, e);
                    errors += 1;
                }
            }
        } else {
            if let Some(ref error) = result.error {
                if error != "empty content" {
                    eprintln!("Warning: {}: {}", result.source, error);
                }
            }
            errors += 1;
        }
    }

    println!(
        "Indexed {} files ({} errors, {} threads)",
        count, errors, num_threads
    );
    Ok(())
}
