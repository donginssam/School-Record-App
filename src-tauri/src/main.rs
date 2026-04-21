// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

struct DbState(Mutex<Option<Connection>>);

// ── 직렬화 구조체 ────────────────────────────────────────────

/// AreaItem.activities 에서 사용하는 단순 항목
#[derive(Serialize, Clone)]
struct ActivityItem {
    id: i64,
    name: String,
}

#[derive(Serialize, Clone)]
struct AreaItem {
    id: i64,
    name: String,
    byte_limit: i64,
    activities: Vec<ActivityItem>,
}

/// ActivitySection 카드에서 사용하는 소속 영역 참조
#[derive(Serialize, Clone)]
struct AreaRef {
    id: i64,
    name: String,
}

#[derive(Serialize, Clone)]
struct StudentItem {
    id: i64,
    grade: i64,
    class_num: i64,
    number: i64,
    name: String,
}

/// get_activities 가 반환하는 풍부한 Activity 항목
#[derive(Serialize, Clone)]
struct ActivityDetail {
    id: i64,
    name: String,
    areas: Vec<AreaRef>,
    record_count: i64,
}

// ── 헬퍼 매크로: 커넥션 잠금 ─────────────────────────────────

macro_rules! conn {
    ($state:expr) => {{
        let guard = $state.0.lock().unwrap();
        guard
            .as_ref()
            .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())
            .map(|c| c as *const Connection)
    }};
}

// ── 프로젝트 커맨드 ──────────────────────────────────────────

#[tauri::command]
fn new_project(path: String, state: State<DbState>) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        return Err(format!("이미 파일이 존재합니다: {path}"));
    }
    let conn = db::create_new(p).map_err(|e| e.to_string())?;
    *state.0.lock().unwrap() = Some(conn);
    Ok(())
}

#[tauri::command]
fn open_project(path: String, state: State<DbState>) -> Result<(), String> {
    let conn = db::open_existing(std::path::Path::new(&path))
        .map_err(|e| e.to_string())?;
    *state.0.lock().unwrap() = Some(conn);
    Ok(())
}

// ── Area 커맨드 ──────────────────────────────────────────────

#[tauri::command]
fn get_areas(state: State<DbState>) -> Result<Vec<AreaItem>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.name, a.byte_limit,
                    act.id AS act_id, act.name AS act_name
             FROM Area a
             LEFT JOIN AreaActivity aa ON a.id = aa.area_id
             LEFT JOIN Activity act ON aa.activity_id = act.id
             ORDER BY a.id, aa.default_order",
        )
        .map_err(|e| e.to_string())?;

    let mut areas: Vec<AreaItem> = Vec::new();
    let mut index_map: HashMap<i64, usize> = HashMap::new();

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (area_id, area_name, byte_limit, act_id, act_name) =
            row.map_err(|e| e.to_string())?;

        let idx = if let Some(&i) = index_map.get(&area_id) {
            i
        } else {
            let i = areas.len();
            areas.push(AreaItem {
                id: area_id,
                name: area_name,
                byte_limit,
                activities: vec![],
            });
            index_map.insert(area_id, i);
            i
        };

        if let (Some(id), Some(name)) = (act_id, act_name) {
            areas[idx].activities.push(ActivityItem { id, name });
        }
    }

    Ok(areas)
}

#[tauri::command]
fn create_area(name: String, byte_limit: i64, state: State<DbState>) -> Result<i64, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO Area (name, byte_limit) VALUES (?1, ?2)",
        rusqlite::params![name, byte_limit],
    )
        .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn update_area(id: i64, name: String, byte_limit: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "UPDATE Area SET name = ?1, byte_limit = ?2 WHERE id = ?3",
        rusqlite::params![name, byte_limit, id],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn delete_area(id: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute("DELETE FROM Area WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── Activity 커맨드 ──────────────────────────────────────────

/// 모든 Activity를 소속 Area 목록과 함께 반환
#[tauri::command]
fn get_activities(state: State<DbState>) -> Result<Vec<ActivityDetail>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT act.id, act.name, a.id AS area_id, a.name AS area_name,
                    (SELECT COUNT(*) FROM ActivityRecord ar WHERE ar.activity_id = act.id) AS record_count
             FROM Activity act
             LEFT JOIN AreaActivity aa ON act.id = aa.activity_id
             LEFT JOIN Area a ON aa.area_id = a.id
             ORDER BY act.id, a.id",
        )
        .map_err(|e| e.to_string())?;

    let mut activities: Vec<ActivityDetail> = Vec::new();
    let mut index_map: HashMap<i64, usize> = HashMap::new();

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    for row in rows {
        let (act_id, act_name, area_id, area_name, record_count) = row.map_err(|e| e.to_string())?;

        let idx = if let Some(&i) = index_map.get(&act_id) {
            i
        } else {
            let i = activities.len();
            activities.push(ActivityDetail {
                id: act_id,
                name: act_name,
                areas: vec![],
                record_count,
            });
            index_map.insert(act_id, i);
            i
        };

        if let (Some(id), Some(name)) = (area_id, area_name) {
            activities[idx].areas.push(AreaRef { id, name });
        }
    }

    Ok(activities)
}

#[tauri::command]
fn create_activity(name: String, state: State<DbState>) -> Result<i64, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO Activity (name) VALUES (?1)",
        rusqlite::params![name],
    )
        .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn update_activity(id: i64, name: String, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "UPDATE Activity SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn delete_activity(id: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "DELETE FROM Activity WHERE id = ?1",
        rusqlite::params![id],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_activity_areas(
    activity_id: i64,
    area_ids: Vec<i64>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    // 이 활동의 기존 영역 매핑 전체 제거
    conn.execute(
        "DELETE FROM AreaActivity WHERE activity_id = ?1",
        rusqlite::params![activity_id],
    )
        .map_err(|e| e.to_string())?;

    // 각 영역에 활동 추가 (해당 영역의 마지막 순서로)
    for area_id in area_ids.iter() {
        conn.execute(
            "INSERT INTO AreaActivity (area_id, activity_id, default_order)
             VALUES (?1, ?2,
               COALESCE((SELECT MAX(default_order) FROM AreaActivity WHERE area_id = ?1), 0) + 1)",
            rusqlite::params![area_id, activity_id],
        )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ── Student 커맨드 ────────────────────────────────────────────

#[tauri::command]
fn get_students(state: State<DbState>) -> Result<Vec<StudentItem>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, grade, class_num, number, name
             FROM Student
             ORDER BY grade, class_num, number",
        )
        .map_err(|e| e.to_string())?;

    let students = stmt
        .query_map([], |row| {
            Ok(StudentItem {
                id: row.get(0)?,
                grade: row.get(1)?,
                class_num: row.get(2)?,
                number: row.get(3)?,
                name: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(students)
}

#[tauri::command]
fn create_student(
    grade: i64,
    class_num: i64,
    number: i64,
    name: String,
    state: State<DbState>,
) -> Result<i64, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO Student (grade, class_num, number, name) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![grade, class_num, number, name],
    )
        .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn update_student(
    id: i64,
    grade: i64,
    class_num: i64,
    number: i64,
    name: String,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "UPDATE Student SET grade = ?1, class_num = ?2, number = ?3, name = ?4 WHERE id = ?5",
        rusqlite::params![grade, class_num, number, name, id],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn delete_student(id: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute("DELETE FROM Student WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_area_activities(
    area_id: i64,
    activity_ids: Vec<i64>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "DELETE FROM AreaActivity WHERE area_id = ?1",
        rusqlite::params![area_id],
    )
        .map_err(|e| e.to_string())?;

    for (order, act_id) in activity_ids.iter().enumerate() {
        conn.execute(
            "INSERT INTO AreaActivity (area_id, activity_id, default_order) VALUES (?1, ?2, ?3)",
            rusqlite::params![area_id, act_id, (order + 1) as i64],
        )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct StudentInput {
    grade: i64,
    class_num: i64,
    number: i64,
    name: String,
}

#[derive(Serialize)]
struct BulkUpsertResult {
    inserted: i64,
    updated: i64,
}

#[tauri::command]
fn bulk_upsert_students(
    students: Vec<StudentInput>,
    state: State<DbState>,
) -> Result<BulkUpsertResult, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut inserted: i64 = 0;
    let mut updated: i64 = 0;
    for s in students.iter() {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                rusqlite::params![s.grade, s.class_num, s.number],
                |row| row.get::<_, i32>(0),
            )
            .map_err(|e| e.to_string())?
            > 0;

        conn.execute(
            "INSERT INTO Student (grade, class_num, number, name)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (grade, class_num, number) DO UPDATE SET name = excluded.name",
            rusqlite::params![s.grade, s.class_num, s.number, s.name],
        )
            .map_err(|e| e.to_string())?;

        if exists {
            updated += 1;
        } else {
            inserted += 1;
        }
    }
    Ok(BulkUpsertResult { inserted, updated })
}

// ── AreaStudent 커맨드 ────────────────────────────────────────

#[tauri::command]
fn get_area_students(area_id: i64, state: State<DbState>) -> Result<Vec<i64>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare("SELECT student_id FROM AreaStudent WHERE area_id = ?1")
        .map_err(|e| e.to_string())?;

    let ids = stmt
        .query_map(rusqlite::params![area_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<i64>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(ids)
}

#[tauri::command]
fn set_area_students(
    area_id: i64,
    student_ids: Vec<i64>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "DELETE FROM AreaStudent WHERE area_id = ?1",
        rusqlite::params![area_id],
    )
        .map_err(|e| e.to_string())?;

    for student_id in student_ids.iter() {
        conn.execute(
            "INSERT INTO AreaStudent (area_id, student_id, is_order_customized) VALUES (?1, ?2, 0)",
            rusqlite::params![area_id, student_id],
        )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ── 기록 그리드 커맨드 ────────────────────────────────────────

#[derive(Serialize, Clone)]
struct RecordCell {
    activity_id: i64,
    student_id: i64,
    content: String,
}

#[derive(Serialize, Clone)]
struct AreaGridData {
    activities: Vec<ActivityItem>,
    students: Vec<StudentItem>,
    records: Vec<RecordCell>,
}

#[tauri::command]
fn get_area_grid(area_id: i64, state: State<DbState>) -> Result<AreaGridData, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    // 영역에 속한 활동 목록
    let mut stmt = conn
        .prepare(
            "SELECT act.id, act.name
             FROM Activity act
             JOIN AreaActivity aa ON act.id = aa.activity_id
             WHERE aa.area_id = ?1
             ORDER BY aa.default_order",
        )
        .map_err(|e| e.to_string())?;

    let activities = stmt
        .query_map(rusqlite::params![area_id], |row| {
            Ok(ActivityItem {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 영역에 배정된 학생 목록
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.grade, s.class_num, s.number, s.name
             FROM Student s
             JOIN AreaStudent as_ ON s.id = as_.student_id
             WHERE as_.area_id = ?1
             ORDER BY s.grade, s.class_num, s.number",
        )
        .map_err(|e| e.to_string())?;

    let students = stmt
        .query_map(rusqlite::params![area_id], |row| {
            Ok(StudentItem {
                id: row.get(0)?,
                grade: row.get(1)?,
                class_num: row.get(2)?,
                number: row.get(3)?,
                name: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 해당 활동들의 기록
    let activity_ids: Vec<i64> = activities.iter().map(|a| a.id).collect();
    let records = if activity_ids.is_empty() {
        vec![]
    } else {
        let placeholders = activity_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT activity_id, student_id, content
             FROM ActivityRecord
             WHERE activity_id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(activity_ids.iter()), |row| {
                Ok(RecordCell {
                    activity_id: row.get(0)?,
                    student_id: row.get(1)?,
                    content: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        rows
    };

    Ok(AreaGridData { activities, students, records })
}

#[tauri::command]
fn upsert_record(
    activity_id: i64,
    student_id: i64,
    content: String,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO ActivityRecord (activity_id, student_id, content, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(activity_id, student_id) DO UPDATE SET
           content = excluded.content,
           updated_at = excluded.updated_at",
        rusqlite::params![activity_id, student_id, content],
    )
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ── 히스토리 커맨드 ──────────────────────────────────────────

#[derive(Serialize)]
struct HistoryEntry {
    id: i64,
    content: String,
    changed_at: String,
    note: Option<String>,
}

/// 특정 셀의 히스토리 목록 조회 (최신순, 페이지네이션)
#[tauri::command]
fn get_record_history(
    activity_id: i64,
    student_id: i64,
    limit: i64,
    offset: i64,
    state: State<DbState>,
) -> Result<Vec<HistoryEntry>, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.content, h.changed_at, h.note
             FROM ActivityRecordHistory h
             JOIN ActivityRecord r ON r.id = h.activity_record_id
             WHERE r.activity_id = ?1 AND r.student_id = ?2
             ORDER BY h.changed_at DESC
             LIMIT ?3 OFFSET ?4",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(rusqlite::params![activity_id, student_id, limit, offset], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                changed_at: row.get(2)?,
                note: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

/// 현재 DB content를 히스토리에 스냅샷으로 기록.
/// note = null → 자동 저장, note = Some(text) → 수동 저장.
/// ActivityRecord가 없거나 content가 비어 있으면 아무 것도 하지 않음.
fn save_snapshot_internal(
    conn: &Connection,
    activity_id: i64,
    student_id: i64,
    note: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO ActivityRecordHistory (activity_record_id, content, changed_at, note)
         SELECT r.id, r.content, datetime('now'), ?3
         FROM ActivityRecord r
         WHERE r.activity_id = ?1 AND r.student_id = ?2
           AND r.content != ''",
        rusqlite::params![activity_id, student_id, note],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_history_snapshot(
    activity_id: i64,
    student_id: i64,
    note: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    save_snapshot_internal(conn, activity_id, student_id, note.as_deref())
}

// ── 일괄 기록 가져오기 ────────────────────────────────────────

#[derive(Deserialize)]
struct ImportRecordInput {
    grade: i64,
    class_num: i64,
    number: i64,
    name: Option<String>,
    activity_id: i64,
    content: String,
}

#[derive(Serialize)]
struct BulkImportResult {
    students_created: i64,
    students_updated: i64,
    records_saved: i64,
}

#[tauri::command]
fn bulk_import_records(
    records: Vec<ImportRecordInput>,
    state: State<DbState>,
) -> Result<BulkImportResult, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    let result = (|| -> Result<BulkImportResult, String> {
        let mut students_created: i64 = 0;
        let mut students_updated: i64 = 0;
        let mut records_saved: i64 = 0;
        let mut student_cache: HashMap<(i64, i64, i64), i64> = HashMap::new();

        for r in records.iter() {
            let key = (r.grade, r.class_num, r.number);

            if !student_cache.contains_key(&key) {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                        rusqlite::params![r.grade, r.class_num, r.number],
                        |row| row.get::<_, i32>(0),
                    )
                    .map_err(|e| e.to_string())?
                    > 0;

                if exists {
                    if let Some(ref n) = r.name {
                        if !n.is_empty() {
                            conn.execute(
                                "UPDATE Student SET name = ?1 WHERE grade=?2 AND class_num=?3 AND number=?4",
                                rusqlite::params![n, r.grade, r.class_num, r.number],
                            )
                                .map_err(|e| e.to_string())?;
                            students_updated += 1;
                        }
                    }
                } else {
                    let name = r.name.as_deref().unwrap_or("");
                    conn.execute(
                        "INSERT INTO Student (grade, class_num, number, name) VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![r.grade, r.class_num, r.number, name],
                    )
                        .map_err(|e| e.to_string())?;
                    students_created += 1;
                }

                let student_id: i64 = conn
                    .query_row(
                        "SELECT id FROM Student WHERE grade=?1 AND class_num=?2 AND number=?3",
                        rusqlite::params![r.grade, r.class_num, r.number],
                        |row| row.get(0),
                    )
                    .map_err(|e| e.to_string())?;

                student_cache.insert(key, student_id);
            }

            let &student_id = student_cache.get(&key).unwrap();

            save_snapshot_internal(conn, r.activity_id, student_id, Some("import"))?;

            conn.execute(
                "INSERT INTO ActivityRecord (activity_id, student_id, content, updated_at)
                 VALUES (?1, ?2, ?3, datetime('now'))
                 ON CONFLICT(activity_id, student_id) DO UPDATE SET
                   content = excluded.content,
                   updated_at = excluded.updated_at",
                rusqlite::params![r.activity_id, student_id, r.content],
            )
                .map_err(|e| e.to_string())?;
            records_saved += 1;
        }

        Ok(BulkImportResult { students_created, students_updated, records_saved })
    })();

    match result {
        Ok(r) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(r)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

// ── 파일 유틸 ────────────────────────────────────────────────

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_bytes_file(path: String, data: String) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}

// ── 앱 진입점 ────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DbState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            new_project,
            open_project,
            get_areas,
            create_area,
            update_area,
            delete_area,
            get_activities,
            set_area_activities,
            create_activity,
            update_activity,
            delete_activity,
            set_activity_areas,
            get_students,
            create_student,
            update_student,
            delete_student,
            bulk_upsert_students,
            get_area_students,
            set_area_students,
            get_area_grid,
            upsert_record,
            get_record_history,
            save_history_snapshot,
            bulk_import_records,
            write_text_file,
            write_bytes_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
