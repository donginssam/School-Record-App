use crate::commands::config::{get_config_impl, set_config_impl};
use crate::crypto::{decrypt, derive_key, encrypt, generate_salt};
use crate::state::{CryptoStateHandle, DbState};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rusqlite::Connection;
use tauri::State;

const VERIFY_PLAINTEXT: &str = "school-record-verify";
const KEY_ENCRYPTION_ENABLED: &str = "encryption_enabled";
const KEY_PBKDF2_SALT: &str = "encryption_pbkdf2_salt";
const KEY_VERIFY_TOKEN: &str = "encryption_verify_token";

#[derive(serde::Serialize)]
pub struct EncryptionStatus {
    pub enabled: bool,
    pub unlocked: bool,
}

fn fetch_id_text(conn: &Connection, sql: &str) -> Result<Vec<(i64, String)>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn encrypt_all_data(conn: &Connection, key: [u8; 32]) -> Result<(), String> {
    let students = fetch_id_text(conn, "SELECT id, name FROM Student")?;
    for (id, name) in students {
        let encrypted = encrypt(&name, &key)?;
        conn.execute(
            "UPDATE Student SET name=?1 WHERE id=?2",
            rusqlite::params![encrypted, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let records =
        fetch_id_text(conn, "SELECT id, content FROM ActivityRecord WHERE content != ''")?;
    for (id, content) in records {
        let encrypted = encrypt(&content, &key)?;
        conn.execute(
            "UPDATE ActivityRecord SET content=?1 WHERE id=?2",
            rusqlite::params![encrypted, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let history = fetch_id_text(
        conn,
        "SELECT id, content FROM ActivityRecordHistory WHERE content != ''",
    )?;
    for (id, content) in history {
        let encrypted = encrypt(&content, &key)?;
        conn.execute(
            "UPDATE ActivityRecordHistory SET content=?1 WHERE id=?2",
            rusqlite::params![encrypted, id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn decrypt_all_data(conn: &Connection, key: [u8; 32]) -> Result<(), String> {
    let students = fetch_id_text(conn, "SELECT id, name FROM Student")?;
    for (id, encrypted_name) in students {
        let plain = decrypt(&encrypted_name, &key)?;
        conn.execute(
            "UPDATE Student SET name=?1 WHERE id=?2",
            rusqlite::params![plain, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let records =
        fetch_id_text(conn, "SELECT id, content FROM ActivityRecord WHERE content != ''")?;
    for (id, content) in records {
        let plain = decrypt(&content, &key)?;
        conn.execute(
            "UPDATE ActivityRecord SET content=?1 WHERE id=?2",
            rusqlite::params![plain, id],
        )
        .map_err(|e| e.to_string())?;
    }

    let history = fetch_id_text(
        conn,
        "SELECT id, content FROM ActivityRecordHistory WHERE content != ''",
    )?;
    for (id, content) in history {
        let plain = decrypt(&content, &key)?;
        conn.execute(
            "UPDATE ActivityRecordHistory SET content=?1 WHERE id=?2",
            rusqlite::params![plain, id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_encryption_status(
    db: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<EncryptionStatus, String> {
    let enabled = {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;
        get_config_impl(conn, KEY_ENCRYPTION_ENABLED)?.as_deref() == Some("true")
    };
    let unlocked = {
        let guard = crypto.lock().map_err(|e| e.to_string())?;
        guard.key.is_some()
    };
    Ok(EncryptionStatus { enabled, unlocked })
}

#[tauri::command]
pub fn unlock_encryption(
    password: String,
    db: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<(), String> {
    let (salt, verify_token) = {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;
        let salt_b64 = get_config_impl(conn, KEY_PBKDF2_SALT)?
            .ok_or("암호화 설정이 없습니다.")?;
        let salt = B64.decode(&salt_b64).map_err(|e| format!("salt 디코딩 실패: {e}"))?;
        let token = get_config_impl(conn, KEY_VERIFY_TOKEN)?
            .ok_or("검증 토큰이 없습니다.")?;
        (salt, token)
    };

    let key = derive_key(&password, &salt);

    let verified = decrypt(&verify_token, &key)
        .map(|s| s == VERIFY_PLAINTEXT)
        .unwrap_or(false);
    if !verified {
        return Err("비밀번호가 올바르지 않습니다.".to_string());
    }

    let mut guard = crypto.lock().map_err(|e| e.to_string())?;
    guard.key = Some(key);
    guard.salt = Some(salt);
    Ok(())
}

#[tauri::command]
pub fn enable_encryption(
    password: String,
    db: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<(), String> {
    let salt = generate_salt();
    let key = derive_key(&password, &salt);
    let salt_b64 = B64.encode(salt);
    let verify_token = encrypt(VERIFY_PLAINTEXT, &key)?;

    {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;

        if get_config_impl(conn, KEY_ENCRYPTION_ENABLED)?.as_deref() == Some("true") {
            return Err("이미 암호화가 활성화되어 있습니다.".to_string());
        }

        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            encrypt_all_data(conn, key)?;
            set_config_impl(conn, KEY_PBKDF2_SALT, &salt_b64)?;
            set_config_impl(conn, KEY_VERIFY_TOKEN, &verify_token)?;
            set_config_impl(conn, KEY_ENCRYPTION_ENABLED, "true")?;
            Ok(())
        })();
        match result {
            Ok(_) => conn.execute_batch("COMMIT").map_err(|e| e.to_string())?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    }

    let mut guard = crypto.lock().map_err(|e| e.to_string())?;
    guard.key = Some(key);
    guard.salt = Some(B64.decode(&salt_b64).unwrap_or_default());
    Ok(())
}

#[tauri::command]
pub fn disable_encryption(
    db: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<(), String> {
    let key = {
        let guard = crypto.lock().map_err(|e| e.to_string())?;
        guard.key.ok_or("암호화가 잠금 상태입니다.")?
    };

    {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;
        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            decrypt_all_data(conn, key)?;
            conn.execute(
                "DELETE FROM APP_CONFIGS WHERE config_key IN (?1, ?2, ?3)",
                rusqlite::params![KEY_ENCRYPTION_ENABLED, KEY_PBKDF2_SALT, KEY_VERIFY_TOKEN],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(_) => conn.execute_batch("COMMIT").map_err(|e| e.to_string())?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    }

    let mut guard = crypto.lock().map_err(|e| e.to_string())?;
    guard.key = None;
    guard.salt = None;
    Ok(())
}

#[tauri::command]
pub fn change_encryption_password(
    old_password: String,
    new_password: String,
    db: State<DbState>,
    crypto: State<CryptoStateHandle>,
) -> Result<(), String> {
    let (salt, verify_token) = {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;
        let salt_b64 = get_config_impl(conn, KEY_PBKDF2_SALT)?
            .ok_or("암호화 설정이 없습니다.")?;
        let salt = B64.decode(&salt_b64).map_err(|e| e.to_string())?;
        let token = get_config_impl(conn, KEY_VERIFY_TOKEN)?
            .ok_or("검증 토큰이 없습니다.")?;
        (salt, token)
    };
    let old_key = derive_key(&old_password, &salt);
    let verified = decrypt(&verify_token, &old_key)
        .map(|s| s == VERIFY_PLAINTEXT)
        .unwrap_or(false);
    if !verified {
        return Err("현재 비밀번호가 올바르지 않습니다.".to_string());
    }

    let new_salt = generate_salt();
    let new_key = derive_key(&new_password, &new_salt);
    let new_salt_b64 = B64.encode(new_salt);
    let new_verify_token = encrypt(VERIFY_PLAINTEXT, &new_key)?;

    {
        let guard = db.0.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB가 열려있지 않습니다.")?;
        conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            decrypt_all_data(conn, old_key)?;
            encrypt_all_data(conn, new_key)?;
            set_config_impl(conn, KEY_PBKDF2_SALT, &new_salt_b64)?;
            set_config_impl(conn, KEY_VERIFY_TOKEN, &new_verify_token)?;
            Ok(())
        })();
        match result {
            Ok(_) => conn.execute_batch("COMMIT").map_err(|e| e.to_string())?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    }

    let mut guard = crypto.lock().map_err(|e| e.to_string())?;
    guard.key = Some(new_key);
    guard.salt = Some(B64.decode(&new_salt_b64).unwrap_or_default());
    Ok(())
}
