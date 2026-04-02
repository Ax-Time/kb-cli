# Agent Guidelines for kb-cli

A local-first knowledge base CLI with hybrid full-text + vector similarity search. Built with Rust.

## Build / Lint / Test Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build (slower but optimized)

# Run
cargo run -- add "text"           # Add content
cargo run -- search "query"       # Search
cargo run -- list                 # List entries
cargo run -- delete <id>          # Delete entry

# Test (single test)
cargo test <test_name>            # Run specific test
cargo test test_add_text_search_delete_flow  # Example

# All tests
cargo test

# Lint / Format
cargo fmt                         # Format code
cargo fmt -- --check              # Check formatting without changes
cargo clippy                      # Run clippy lints
cargo clippy -- -D warnings        # Treat warnings as errors
```

## Code Style

### Formatting
- Use `cargo fmt` before committing
- 4-space indentation (Rust default)
- 100 character line length

### Imports
- Group imports by crate: external, then internal
- Use `use` with full paths for clarity
- Prefer `crate::` for internal imports

```rust
use std::path::Path;
use rayon::prelude::*;
use rusqlite::Connection;

use crate::embed::Embedder;
use crate::error::Result;
```

### Error Handling
- Use `thiserror` crate via `KbError` enum in `src/error.rs`
- Public functions return `Result<T>` alias: `pub type Result<T> = std::result::Result<T, KbError>;`
- Use `#[from]` for automatic error conversion
- Propagate errors with `?` operator

```rust
#[derive(Error, Debug)]
pub enum KbError {
    #[error("failed to index {0}: {1}")]
    IndexingFailed(String, String),

    #[error("database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Types
- Use explicit type signatures in public functions
- Prefer `&Path` over `&str` for path parameters
- Use `i64` for database IDs
- Use `Vec<f32>` for embedding vectors (384 dimensions, `EMBEDDING_DIM` constant)

### Naming Conventions
- Modules: `snake_case` (e.g., `db`, `embed`, `indexer`)
- Functions: `snake_case` (e.g., `add_single`, `search_entries`)
- Types/Enums: `PascalCase` (e.g., `KbError`, `Commands`, `Embedder`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `EMBEDDING_DIM`, `KB_DIR_NAME`)
- Variables: `snake_case` (e.g., `query_embedding`, `kb_dir`)

### Struct Organization
```rust
pub struct Embedder {
    session: Mutex<Session>,      // Thread-safe interior mutability
    tokenizer: Mutex<Tokenizer>,
}

unsafe impl Send for Embedder {}  // Mark as thread-safe if needed
unsafe impl Sync for Embedder {}
```

### Module Structure
- `src/lib.rs`: Library root with public re-exports
- `src/main.rs`: Binary entry point, CLI parsing
- `src/error.rs`: Error types (`KbError`, `Result`)
- `src/config.rs`: Configuration and path resolution
- `src/commands/`: CLI command implementations (`add`, `search`, `delete`, `list`)
- `src/db/`: Database layer (connection, queries, schema)
- `src/embed/`: ML embedding functionality
- `src/indexer/`: Content detection and extraction

### Database
- SQLite with FTS5 (full-text search) and vector embeddings
- Schema in `src/db/schema.sql`
- WAL journal mode enabled
- Use `rusqlite` with bundled feature

### Concurrency
- Use `rayon` for data-parallel operations
- Wrap non-thread-safe resources in `Mutex` or `RwLock`
- Use `Arc` when sharing across rayon threads

### Testing
- Unit tests: `#[cfg(test)] mod tests` within source files
- Integration tests: `tests/integration_test.rs`
- Use `tempfile::TempDir` for isolated test environments
- Model tests are `#[ignore]` by default (require downloaded model)

### CLI Pattern
- Use `clap` with `#[derive(Parser, Subcommand)]`
- Commands defined as enum variants
- Global flags (`--local`, `--global`) before subcommand

### Key Files
- `Cargo.toml`: Dependencies and package config
- `src/main.rs`: CLI entry, `run()` function, error handling
- `src/error.rs`: Error types and `Result` alias
- `src/db/queries.rs`: All database queries
- `src/embed/mod.rs`: `Embedder` struct, `embed()` method

## Architecture Notes

The KB can be local (`.kb/` in cwd) or global (`~/.kb/`). Path resolution priority:
1. `--local` flag → `.kb/` in current directory
2. `--global` flag → `~/.kb/` (or `KB_GLOBAL_PATH` env var)
3. Walk up from cwd looking for `.kb/`
4. Fall back to `~/.kb/`

On first run, downloads `all-MiniLM-L6-v2` embedding model (~80MB) to `~/.kb/models/`.
