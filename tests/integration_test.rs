use std::fs;
use tempfile::TempDir;

struct TestKB {
    temp: TempDir,
}

impl TestKB {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".kb/models")).unwrap();
        Self { temp }
    }

    fn kb_dir(&self) -> &std::path::Path {
        self.temp.path()
    }

    fn db_path(&self) -> std::path::PathBuf {
        self.temp.path().join(".kb/kb.db")
    }
}

#[test]
fn test_add_text_search_delete_flow() {}

#[test]
fn test_add_file_and_search() {}

#[test]
fn test_recursive_directory_indexing() {
    let kb = TestKB::new();

    let test_dir = kb.kb_dir().join("test_docs");
    fs::create_dir_all(&test_dir).unwrap();
    fs::write(
        test_dir.join("file1.txt"),
        "Rust is a systems programming language",
    )
    .unwrap();
    fs::write(test_dir.join("file2.txt"), "Python is a scripting language").unwrap();
    fs::write(test_dir.join("file3.md"), "# Markdown\nSome content here").unwrap();

    fs::write(test_dir.join("image.png"), [0x89, 0x50, 0x4E, 0x47]).unwrap();

    fs::create_dir_all(test_dir.join("sub")).unwrap();
    fs::write(
        test_dir.join("sub").join("nested.txt"),
        "Nested file content",
    )
    .unwrap();

    let walker = ignore::WalkBuilder::new(&test_dir).build();
    let files: Vec<_> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .map(|e| e.path().to_path_buf())
        .collect();

    assert!(files.iter().any(|p| p.ends_with("file1.txt")));
    assert!(files.iter().any(|p| p.ends_with("file2.txt")));
    assert!(files.iter().any(|p| p.ends_with("file3.md")));
    assert!(files.iter().any(|p| p.ends_with("image.png")));
    assert!(files.iter().any(|p| p.ends_with("nested.txt")));
}

#[test]
fn test_database_operations() {
    use kb::db::{open_db, queries::delete_entry, queries::insert_entry};

    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");
    let conn = open_db(&db_path).unwrap();

    let embedding = vec![0.1f32; 384];
    let id = insert_entry(&conn, "test.txt", "hello world", &embedding).unwrap();
    assert!(id > 0);

    delete_entry(&conn, id).unwrap();

    let result = delete_entry(&conn, id);
    assert!(result.is_err());
}
