use rusqlite::{Connection, Result};
use std::path::Path;

/// 새 DB 파일 생성 후 스키마 초기화
pub fn create_new(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(include_str!("schema.sql"))?;
    Ok(conn)
}

/// 기존 DB 파일 열기 (스키마 초기화 없음)
pub fn open_existing(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}