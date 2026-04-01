# kb — Local Knowledge Base CLI

A local-first knowledge base with hybrid full-text + vector similarity search.

## Usage

```bash
# Add content
kb add "Some text to remember"
kb add ./document.txt
kb add ./docs/ --recursive
kb add https://example.com/article

# Search
kb search "what was that thing about Rust"
kb search "Rust" --limit 5

# Delete
kb delete 42

# List indexed entries
kb list

# Show KB location
kb where
```

## KB Location

- **Local:** Creates `.kb/` in current directory
- **Global:** Uses `~/.kb/` (default if no local `.kb/` found)
- **Flags:** `--local` or `--global` to force a specific location

## First Run

On first use, kb downloads the `all-MiniLM-L6-v2` embedding model (~80MB) to `~/.kb/models/`. Subsequent runs use the cached model.

## Building

```bash
cargo build --release
```
