use crate::state::DbState;
use tauri::State;

#[tauri::command]
pub fn new_project(path: String, state: State<DbState>) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if p.exists() {
        return Err(format!("이미 파일이 존재합니다: {path}"));
    }
    let conn = crate::db::create_new(p).map_err(|e| e.to_string())?;
    *state.0.lock().unwrap() = Some(conn);
    Ok(())
}

#[tauri::command]
pub fn open_project(path: String, state: State<DbState>) -> Result<(), String> {
    let src = std::path::Path::new(&path);

    if let Some(parent) = src.parent() {
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("backup");
        let ts = chrono::Local::now().format("%y%m%d-%H%M").to_string();
        let bak_name = format!("{stem}.{ts}.db.backup");
        let _ = std::fs::copy(src, parent.join(bak_name));
    }

    let conn = crate::db::open_existing(src).map_err(|e| e.to_string())?;
    *state.0.lock().unwrap() = Some(conn);
    Ok(())
}
