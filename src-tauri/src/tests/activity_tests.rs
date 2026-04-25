use crate::commands::activity::{
    create_activity_impl, delete_activity_impl, get_activities_impl, set_activity_areas_impl,
    update_activity_impl,
};
use super::{insert_activity, insert_area, insert_record, insert_student, setup_test_db};

#[test]
fn test_create_activity_returns_id() {
    let conn = setup_test_db();
    let id = create_activity_impl(&conn, "수행평가").unwrap();
    assert!(id > 0);
}

#[test]
fn test_create_activity_duplicate_name_error() {
    let conn = setup_test_db();
    create_activity_impl(&conn, "수행평가").unwrap();
    let err = create_activity_impl(&conn, "수행평가").unwrap_err();
    assert!(err.contains("이미 같은 이름의 활동"), "에러 메시지: {err}");
}

#[test]
fn test_get_activities_empty_db() {
    let conn = setup_test_db();
    let acts = get_activities_impl(&conn).unwrap();
    assert!(acts.is_empty());
}

#[test]
fn test_get_activities_with_areas() {
    let conn = setup_test_db();
    let act_id = insert_activity(&conn, "발표");
    let area_id = insert_area(&conn, "국어", 500);
    conn.execute(
        "INSERT INTO AreaActivity (area_id, activity_id) VALUES (?1, ?2)",
        rusqlite::params![area_id, act_id],
    )
    .unwrap();

    let acts = get_activities_impl(&conn).unwrap();
    assert_eq!(acts.len(), 1);
    assert_eq!(acts[0].areas.len(), 1);
    assert_eq!(acts[0].areas[0].name, "국어");
}

#[test]
fn test_get_activities_record_count_zero() {
    let conn = setup_test_db();
    insert_activity(&conn, "보고서");
    let acts = get_activities_impl(&conn).unwrap();
    assert_eq!(acts[0].record_count, 0);
}

#[test]
fn test_get_activities_record_count_nonzero() {
    let conn = setup_test_db();
    let act_id = insert_activity(&conn, "실험");
    let stu1 = insert_student(&conn, 1, 1, 1, "홍길동");
    let stu2 = insert_student(&conn, 1, 1, 2, "김철수");
    insert_record(&conn, act_id, stu1, "실험 내용");
    insert_record(&conn, act_id, stu2, "두 번째 내용");

    let acts = get_activities_impl(&conn).unwrap();
    assert_eq!(acts[0].record_count, 2);
}

#[test]
fn test_update_activity_name() {
    let conn = setup_test_db();
    let id = create_activity_impl(&conn, "발표").unwrap();
    update_activity_impl(&conn, id, "발표(개정)").unwrap();

    let acts = get_activities_impl(&conn).unwrap();
    assert_eq!(acts[0].name, "발표(개정)");
}

#[test]
fn test_update_activity_duplicate_name_error() {
    let conn = setup_test_db();
    let id1 = create_activity_impl(&conn, "발표").unwrap();
    let id2 = create_activity_impl(&conn, "보고서").unwrap();
    let _ = id1;
    let err = update_activity_impl(&conn, id2, "발표").unwrap_err();
    assert!(err.contains("이미 같은 이름의 활동"), "에러 메시지: {err}");
}

#[test]
fn test_delete_activity_cascades_activity_records() {
    let conn = setup_test_db();
    let act_id = create_activity_impl(&conn, "삭제될활동").unwrap();
    let stu_id = insert_student(&conn, 1, 1, 1, "홍길동");
    insert_record(&conn, act_id, stu_id, "내용");

    delete_activity_impl(&conn, act_id).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM ActivityRecord WHERE activity_id=?1",
            rusqlite::params![act_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_set_activity_areas_replaces() {
    let conn = setup_test_db();
    let act_id = insert_activity(&conn, "과제");
    let area1 = insert_area(&conn, "국어", 500);
    let area2 = insert_area(&conn, "수학", 500);

    set_activity_areas_impl(&conn, act_id, &[area1]).unwrap();
    set_activity_areas_impl(&conn, act_id, &[area2]).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM AreaActivity WHERE activity_id=?1",
            rusqlite::params![act_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    let linked_area: i64 = conn
        .query_row(
            "SELECT area_id FROM AreaActivity WHERE activity_id=?1",
            rusqlite::params![act_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(linked_area, area2);
}

#[test]
fn test_set_activity_areas_empty_clears_all() {
    let conn = setup_test_db();
    let act_id = insert_activity(&conn, "과제");
    let area_id = insert_area(&conn, "영어", 400);
    set_activity_areas_impl(&conn, act_id, &[area_id]).unwrap();

    set_activity_areas_impl(&conn, act_id, &[]).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM AreaActivity WHERE activity_id=?1",
            rusqlite::params![act_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}
