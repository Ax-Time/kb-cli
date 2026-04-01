use std::path::Path;

use rusqlite::Connection;

use crate::embed::Embedder;
use crate::error::Result;
use crate::indexer::{detect_source, ContentSource, FileSource};

pub fn add(conn: &Connection, embedder: &mut Embedder, input: &str, recursive: bool) -> Result<()> {
    let source = detect_source(input);

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

fn add_directory(conn: &Connection, embedder: &mut Embedder, dir: &Path) -> Result<()> {
    let walker = ignore::WalkBuilder::new(dir).hidden(false).build();

    let mut count = 0;
    let mut errors = 0;

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: {}", e);
                errors += 1;
                continue;
            }
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let source = FileSource::new(path.clone());

        if source.is_binary() {
            continue;
        }

        match add_single(conn, embedder, &source) {
            Ok(()) => count += 1,
            Err(e) => {
                eprintln!("Warning: {}: {}", path.display(), e);
                errors += 1;
            }
        }
    }

    println!("Indexed {} files ({} errors)", count, errors);
    Ok(())
}
