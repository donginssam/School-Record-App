use crate::state::{DbState, unique_err};
use crate::types::{ActivityItem, AreaItem};
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn get_areas(state: State<DbState>) -> Result<Vec<AreaItem>, String> {
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
             ORDER BY a.id",
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
pub fn create_area(name: String, byte_limit: i64, state: State<DbState>) -> Result<i64, String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "INSERT INTO Area (name, byte_limit) VALUES (?1, ?2)",
        rusqlite::params![name, byte_limit],
    )
    .map_err(|e| unique_err(&e, &format!("이미 같은 이름의 영역이 있습니다: {name}")))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_area(id: i64, name: String, byte_limit: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute(
        "UPDATE Area SET name = ?1, byte_limit = ?2 WHERE id = ?3",
        rusqlite::params![name, byte_limit, id],
    )
    .map_err(|e| unique_err(&e, &format!("이미 같은 이름의 영역이 있습니다: {name}")))?;

    Ok(())
}

#[tauri::command]
pub fn delete_area(id: i64, state: State<DbState>) -> Result<(), String> {
    let guard = state.0.lock().unwrap();
    let conn = guard
        .as_ref()
        .ok_or_else(|| "DB가 열려있지 않습니다.".to_string())?;

    conn.execute("DELETE FROM Area WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
