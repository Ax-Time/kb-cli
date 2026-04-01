use rusqlite::Connection;

use crate::db::queries::list_entries;
use crate::error::Result;

pub fn list(conn: &Connection) -> Result<()> {
    let entries = list_entries(conn)?;

    if entries.is_empty() {
        println!("No entries indexed.");
        return Ok(());
    }

    println!("{:>6}  {:<40}  {}", "ID", "Source", "Created At");
    println!("{:-<6}  {:-<40}  {:-<20}", "", "", "");

    let count = entries.len();
    for entry in entries {
        println!(
            "{:>6}  {:<40}  {}",
            entry.id,
            truncate(&entry.source, 40),
            entry.created_at
        );
    }

    println!("\n{} entry(ies) indexed.", count);
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
