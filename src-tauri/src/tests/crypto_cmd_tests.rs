use crate::commands::record::{
    get_area_grid_impl, get_record_history_impl, upsert_record_impl,
};
use crate::commands::student::{
    bulk_upsert_students_impl, create_student_impl, get_students_impl, update_student_impl,
};
use crate::crypto::derive_key;
use crate::engine::get_records_for_scope;
use crate::types::StudentInput;
use super::{insert_activity, insert_area, insert_student, setup_test_db};

fn test_key() -> [u8; 32] {
    derive_key("password", &[42u8; 16])
}

// ── 학생 이름 암호화 ──────────────────────────────────────────────

#[test]
fn test_create_student_with_key_stores_encrypted_name() {
    let conn = setup_test_db();
    let key = test_key();
    create_student_impl(&conn, 1, 1, 1, "홍길동", Some(key)).unwrap();

    // DB에는 암호화된 값이 저장되어야 한다
    let raw_name: String = conn
        .query_row("SELECT name FROM Student WHERE grade=1", [], |r| r.get(0))
        .unwrap();
    assert_ne!(raw_name, "홍길동", "DB에 평문이 저장되면 안 된다");
    assert!(raw_name.contains(':'), "nonce:ciphertext 형식이어야 한다");
}

#[test]
fn test_get_students_with_key_decrypts_name() {
    let conn = setup_test_db();
    let key = test_key();
    create_student_impl(&conn, 1, 1, 1, "홍길동", Some(key)).unwrap();

    let students = get_students_impl(&conn, Some(key)).unwrap();
    assert_eq!(students[0].name, "홍길동");
}

#[test]
fn test_get_students_without_key_returns_encrypted_value() {
    let conn = setup_test_db();
    let key = test_key();
    create_student_impl(&conn, 1, 1, 1, "홍길동", Some(key)).unwrap();

    // 키 없이 조회하면 암호화된 raw 값이 그대로 나온다
    let students = get_students_impl(&conn, None).unwrap();
    assert_ne!(students[0].name, "홍길동");
}

#[test]
fn test_update_student_with_key_stores_and_reads_correctly() {
    let conn = setup_test_db();
    let key = test_key();
    let id = create_student_impl(&conn, 1, 1, 1, "원래이름", Some(key)).unwrap();
    update_student_impl(&conn, id, 1, 1, 1, "변경이름", Some(key)).unwrap();

    let students = get_students_impl(&conn, Some(key)).unwrap();
    assert_eq!(students[0].name, "변경이름");
}

#[test]
fn test_get_students_sorted_order_preserved_with_encryption() {
    let conn = setup_test_db();
    let key = test_key();
    create_student_impl(&conn, 2, 1, 1, "세번째", Some(key)).unwrap();
    create_student_impl(&conn, 1, 2, 1, "두번째", Some(key)).unwrap();
    create_student_impl(&conn, 1, 1, 1, "첫번째", Some(key)).unwrap();

    let students = get_students_impl(&conn, Some(key)).unwrap();
    assert_eq!(students[0].name, "첫번째");
    assert_eq!(students[1].name, "두번째");
    assert_eq!(students[2].name, "세번째");
}

#[test]
fn test_bulk_upsert_students_with_key() {
    let conn = setup_test_db();
    let key = test_key();
    let inputs = vec![
        StudentInput { grade: 1, class_num: 1, number: 1, name: "가".to_string() },
        StudentInput { grade: 1, class_num: 1, number: 2, name: "나".to_string() },
    ];
    let result = bulk_upsert_students_impl(&conn, &inputs, Some(key)).unwrap();
    assert_eq!(result.inserted, 2);

    let students = get_students_impl(&conn, Some(key)).unwrap();
    let names: Vec<&str> = students.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"가"));
    assert!(names.contains(&"나"));
}

#[test]
fn test_bulk_upsert_students_update_overwrites_with_encryption() {
    let conn = setup_test_db();
    let key = test_key();
    // 1차 삽입
    bulk_upsert_students_impl(
        &conn,
        &[StudentInput { grade: 1, class_num: 1, number: 1, name: "원래".to_string() }],
        Some(key),
    ).unwrap();
    // 2차 갱신
    bulk_upsert_students_impl(
        &conn,
        &[StudentInput { grade: 1, class_num: 1, number: 1, name: "변경".to_string() }],
        Some(key),
    ).unwrap();

    let students = get_students_impl(&conn, Some(key)).unwrap();
    assert_eq!(students[0].name, "변경");
}

// ── 기록 content 암호화 ───────────────────────────────────────────

#[test]
fn test_upsert_record_with_key_stores_encrypted_content() {
    let conn = setup_test_db();
    let key = test_key();
    let act_id = insert_activity(&conn, "발표");
    let stu_id = insert_student(&conn, 1, 1, 1, "홍길동");

    upsert_record_impl(&conn, act_id, stu_id, "리더십이 뛰어난 학생입니다.", Some(key)).unwrap();

    let raw: String = conn
        .query_row(
            "SELECT content FROM ActivityRecord WHERE activity_id=?1 AND student_id=?2",
            rusqlite::params![act_id, stu_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_ne!(raw, "리더십이 뛰어난 학생입니다.", "DB에 평문이 저장되면 안 된다");
    assert!(raw.contains(':'), "nonce:ciphertext 형식이어야 한다");
}

#[test]
fn test_upsert_record_empty_content_not_encrypted() {
    let conn = setup_test_db();
    let key = test_key();
    let act_id = insert_activity(&conn, "발표");
    let stu_id = insert_student(&conn, 1, 1, 1, "홍길동");

    upsert_record_impl(&conn, act_id, stu_id, "", Some(key)).unwrap();

    let raw: String = conn
        .query_row(
            "SELECT content FROM ActivityRecord WHERE activity_id=?1 AND student_id=?2",
            rusqlite::params![act_id, stu_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(raw, "", "빈 문자열은 암호화하지 않아야 한다");
}

#[test]
fn test_get_area_grid_with_key_decrypts_content_and_name() {
    let conn = setup_test_db();
    let key = test_key();
    let area_id = insert_area(&conn, "국어", 500);
    let act_id = insert_activity(&conn, "독서");
    let stu_id = create_student_impl(&conn, 1, 1, 1, "김철수", Some(key)).unwrap();

    conn.execute(
        "INSERT INTO AreaActivity (area_id, activity_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, act_id],
    ).unwrap();
    conn.execute(
        "INSERT INTO AreaStudent (area_id, student_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, stu_id],
    ).unwrap();
    upsert_record_impl(&conn, act_id, stu_id, "독후감 내용", Some(key)).unwrap();

    let grid = get_area_grid_impl(&conn, area_id, Some(key)).unwrap();
    assert_eq!(grid.students[0].name, "김철수");
    assert_eq!(grid.records[0].content, "독후감 내용");
}

#[test]
fn test_get_record_history_with_key_decrypts_content() {
    let conn = setup_test_db();
    let key = test_key();
    let act_id = insert_activity(&conn, "발표");
    let stu_id = insert_student(&conn, 1, 1, 1, "홍길동");

    upsert_record_impl(&conn, act_id, stu_id, "발표 내용", Some(key)).unwrap();

    // 히스토리에 암호화된 content를 직접 삽입
    let encrypted_content: String = conn
        .query_row(
            "SELECT content FROM ActivityRecord WHERE activity_id=?1 AND student_id=?2",
            rusqlite::params![act_id, stu_id],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "INSERT INTO ActivityRecordHistory (activity_record_id, content, changed_at, note)
         SELECT id, content, '2024-01-01 10:00:00', NULL FROM ActivityRecord
         WHERE activity_id=?1 AND student_id=?2",
        rusqlite::params![act_id, stu_id],
    ).unwrap();

    let history = get_record_history_impl(&conn, act_id, stu_id, 10, 0, Some(key)).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].content, "발표 내용", "복호화된 히스토리 content여야 한다");
    // DB에는 여전히 암호화 값이 저장되어 있어야 한다
    assert!(encrypted_content.contains(':'));
}

// ── engine: get_records_for_scope ─────────────────────────────────

#[test]
fn test_get_records_for_scope_all_with_key_decrypts() {
    let conn = setup_test_db();
    let key = test_key();
    let act_id = insert_activity(&conn, "활동");
    let stu_id = insert_student(&conn, 1, 1, 1, "학생");
    upsert_record_impl(&conn, act_id, stu_id, "기록 내용", Some(key)).unwrap();

    let records = get_records_for_scope(&conn, "all", &[], Some(key)).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].content, "기록 내용");
}

#[test]
fn test_get_records_for_scope_all_without_key_returns_encrypted() {
    let conn = setup_test_db();
    let key = test_key();
    let act_id = insert_activity(&conn, "활동");
    let stu_id = insert_student(&conn, 1, 1, 1, "학생");
    upsert_record_impl(&conn, act_id, stu_id, "기록 내용", Some(key)).unwrap();

    let records = get_records_for_scope(&conn, "all", &[], None).unwrap();
    assert_eq!(records.len(), 1);
    assert_ne!(records[0].content, "기록 내용", "키 없이 조회하면 암호화된 값이 나와야 한다");
}

#[test]
fn test_get_records_for_scope_areas_with_key_decrypts() {
    let conn = setup_test_db();
    let key = test_key();
    let area_id = insert_area(&conn, "국어", 500);
    let act_id = insert_activity(&conn, "활동");
    let stu_id = insert_student(&conn, 1, 1, 1, "학생");
    conn.execute(
        "INSERT INTO AreaActivity (area_id, activity_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, act_id],
    ).unwrap();
    conn.execute(
        "INSERT INTO AreaStudent (area_id, student_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, stu_id],
    ).unwrap();
    upsert_record_impl(&conn, act_id, stu_id, "영역별 내용", Some(key)).unwrap();

    let records = get_records_for_scope(&conn, "areas", &[area_id], Some(key)).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].content, "영역별 내용");
}

// ── 암호화 없을 때 기존 동작 보존 ─────────────────────────────────

#[test]
fn test_create_get_student_without_encryption_unchanged() {
    let conn = setup_test_db();
    create_student_impl(&conn, 1, 1, 1, "홍길동", None).unwrap();
    let students = get_students_impl(&conn, None).unwrap();
    assert_eq!(students[0].name, "홍길동");
}

#[test]
fn test_upsert_get_record_without_encryption_unchanged() {
    let conn = setup_test_db();
    let act_id = insert_activity(&conn, "발표");
    let stu_id = insert_student(&conn, 1, 1, 1, "홍길동");
    upsert_record_impl(&conn, act_id, stu_id, "발표 내용", None).unwrap();

    let content: String = conn
        .query_row(
            "SELECT content FROM ActivityRecord WHERE activity_id=?1",
            rusqlite::params![act_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(content, "발표 내용");
}

// ── 기존 평문 데이터 → 키 적용 시 에러 (일관성 검증) ───────────────

#[test]
fn test_decrypt_plaintext_with_key_returns_error() {
    let conn = setup_test_db();
    let key = test_key();
    // 평문으로 저장된 학생 이름을 key로 복호화하려 하면 에러여야 한다
    insert_student(&conn, 1, 1, 1, "평문이름");

    let result = get_students_impl(&conn, Some(key));
    // "잘못된 암호화 형식" 또는 "복호화 실패" 에러
    assert!(result.is_err(), "평문을 키로 복호화하면 에러여야 한다");
}

// ── enable_all_data / disable_all_data 흐름 통합 검증 ──────────────

#[test]
fn test_encrypt_then_decrypt_all_data_restores_plaintext() {
    use crate::crypto::{decrypt, encrypt};

    let conn = setup_test_db();
    let key = test_key();

    // 1. 평문으로 데이터 삽입
    let act_id = insert_activity(&conn, "활동");
    let stu_id = insert_student(&conn, 1, 1, 1, "홍길동");
    conn.execute(
        "INSERT INTO ActivityRecord (activity_id, student_id, content) VALUES (?1, ?2, ?3)",
        rusqlite::params![act_id, stu_id, "활동 기록"],
    ).unwrap();

    // 2. 수동으로 각 테이블 암호화 (enable_encryption의 핵심 로직)
    let raw_name: String = conn
        .query_row("SELECT name FROM Student WHERE id=?1", rusqlite::params![stu_id], |r| r.get(0))
        .unwrap();
    conn.execute(
        "UPDATE Student SET name=?1 WHERE id=?2",
        rusqlite::params![encrypt(&raw_name, &key).unwrap(), stu_id],
    ).unwrap();

    let raw_content: String = conn
        .query_row(
            "SELECT content FROM ActivityRecord WHERE activity_id=?1 AND student_id=?2",
            rusqlite::params![act_id, stu_id],
            |r| r.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE ActivityRecord SET content=?1 WHERE activity_id=?2 AND student_id=?3",
        rusqlite::params![encrypt(&raw_content, &key).unwrap(), act_id, stu_id],
    ).unwrap();

    // 3. 암호화 상태에서 _impl 으로 읽기
    let students = get_students_impl(&conn, Some(key)).unwrap();
    assert_eq!(students[0].name, "홍길동");

    // 4. 수동으로 복호화 (disable_encryption의 핵심 로직)
    let enc_name: String = conn
        .query_row("SELECT name FROM Student WHERE id=?1", rusqlite::params![stu_id], |r| r.get(0))
        .unwrap();
    conn.execute(
        "UPDATE Student SET name=?1 WHERE id=?2",
        rusqlite::params![decrypt(&enc_name, &key).unwrap(), stu_id],
    ).unwrap();

    // 5. 복호화 후 None 키로 읽으면 평문이 나와야 한다
    let students = get_students_impl(&conn, None).unwrap();
    assert_eq!(students[0].name, "홍길동");
}
