use crate::db;
use rusqlite::Connection;

fn temp_path(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let mut p = std::env::temp_dir();
    p.push(format!("school_test_{}_{}.db", label, nanos));
    p
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        rusqlite::params![name],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

#[test]
fn test_create_new_creates_required_tables() {
    let path = temp_path("create_tables");
    let conn = db::create_new(&path).unwrap();

    let required = [
        "Student",
        "Area",
        "Activity",
        "ActivityRecord",
        "ActivityRecordHistory",
        "Snapshot",
        "ReplaceRule",
        "SynonymGroup",
        "SynonymItem",
        "APP_CONFIGS",
    ];
    for table in &required {
        assert!(table_exists(&conn, table), "테이블 없음: {table}");
    }

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_create_new_sets_schema_version() {
    let path = temp_path("schema_version");
    let conn = db::create_new(&path).unwrap();

    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, db::SCHEMA_VERSION);

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_open_existing_same_version_ok() {
    let path = temp_path("open_existing");
    {
        let conn = db::create_new(&path).unwrap();
        drop(conn);
    }

    let result = db::open_existing(&path);
    assert!(result.is_ok(), "open_existing 실패: {:?}", result.err());

    let conn = result.unwrap();
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, db::SCHEMA_VERSION);

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_open_too_new_returns_error() {
    let path = temp_path("too_new");
    {
        let conn = db::create_new(&path).unwrap();
        // user_version을 앱 버전보다 높게 수동 설정
        conn.pragma_update(None, "user_version", db::SCHEMA_VERSION + 1)
            .unwrap();
        drop(conn);
    }

    let result = db::open_existing(&path);
    assert!(
        matches!(result, Err(db::OpenError::TooNew { .. })),
        "TooNew 에러 예상, 실제: {:?}",
        result.map(|_| ())
    );

    let _ = std::fs::remove_file(&path);
}
