use rusqlite::Connection;

use crate::db::queries::delete_entry;
use crate::error::Result;

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    delete_entry(conn, id)?;
    println!("Deleted entry {}", id);
    Ok(())
}
