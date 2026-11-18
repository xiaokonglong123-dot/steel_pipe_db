//! 仓储契约：D 域不变量（TOCTOU 守卫、软删、used_count、单据级联、规格唯一）。

use std::path::PathBuf;

use steelpipe_compliance::ScreenPipeParams;
use steelpipe_domain::tube::TubeDraft;
use steelpipe_domain::{QualityMark, TransactionType, TubeStatus, TubeType};
use steelpipe_infra::repo::{locations, standards, transactions, tubes};
use steelpipe_infra::uow::Uow;
use steelpipe_infra::{DbPool, InfraError, time};

struct Fixture {
    pool: DbPool,
    path: PathBuf,
}

async fn fixture(tag: &str) -> Fixture {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("steelpipe-repo-{tag}-{nanos}.db"));
    let pool = steelpipe_infra::connect(&format!("sqlite://{}", path.display()), 8)
        .await
        .unwrap();
    steelpipe_infra::migrate(&pool).await.unwrap();
    Fixture { pool, path }
}

impl Fixture {
    async fn cleanup(self) {
        self.pool.close().await;
        let _ = std::fs::remove_file(&self.path);
    }
}

fn draft(pipe_id: &str, location: &str) -> TubeDraft {
    TubeDraft {
        pipe_id: pipe_id.into(),
        pipe_type: TubeType::SeamlessPipe,
        outer_diameter_mm: 88.9,
        wall_thickness_mm: 7.0,
        length_m: 9.5,
        steel_grade: "L80".into(),
        thread_type: Some("EUE".into()),
        manufacturer: "宝钢".into(),
        batch_no: Some("B1".into()),
        heat_no: None,
        heat_treat_method: None,
        location: location.into(),
        api_5ct_cert: None,
        quality_mark: QualityMark::Normal,
        quality_note: None,
    }
}

async fn stock_in(fx: &Fixture, pipe_id: &str, location: &str) {
    let now = time::now_rfc3339();
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let d = draft(pipe_id, location);
    let normalized = d.validate().unwrap();
    tubes::insert(uow.conn(), &normalized, &d, true, &now, &now)
        .await
        .unwrap();
    locations::adjust_used_count(uow.conn(), location, 1)
        .await
        .unwrap();
    uow.commit().await.unwrap();
}

#[tokio::test]
async fn insert_rejects_duplicate_pipe_id() {
    let fx = fixture("dup").await;
    let now = time::now_rfc3339();
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    locations::insert(uow.conn(), "A1", Some("A1"), None)
        .await
        .unwrap();
    let d = draft("P-1", "A1");
    tubes::insert(uow.conn(), "P-1", &d, true, &now, &now)
        .await
        .unwrap();
    let err = tubes::insert(uow.conn(), "P-1", &d, true, &now, &now)
        .await
        .unwrap_err();
    assert!(matches!(err, InfraError::Conflict(_)), "实际：{err:?}");

    fx.cleanup().await;
}

#[tokio::test]
async fn guarded_status_update_detects_concurrent_change() {
    let fx = fixture("guard").await;
    let now = time::now_rfc3339();
    {
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, None)
            .await
            .unwrap();
        uow.commit().await.unwrap();
    }
    stock_in(&fx, "P-2", "A1").await;

    let now2 = time::now_rfc3339();
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    tubes::update_status_guarded(
        uow.conn(),
        "P-2",
        TubeStatus::InStock,
        TubeStatus::InTransfer,
        &now2,
    )
    .await
    .expect("第一次应成功");
    uow.commit().await.unwrap();

    // 第二个写者用同样的 from 状态再推一次：行数 0 → Conflict
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let err = tubes::update_status_guarded(
        uow.conn(),
        "P-2",
        TubeStatus::InStock,
        TubeStatus::InTransfer,
        &now2,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, InfraError::Conflict(_)), "实际：{err:?}");
    uow.rollback().await.unwrap();

    let row = tubes::find(&fx.pool, "P-2").await.unwrap().unwrap();
    assert_eq!(row.status().unwrap(), TubeStatus::InTransfer);
    assert!(row.stock_in_time.is_some(), "入库时间必须落库");
    let _ = now;

    fx.cleanup().await;
}

#[tokio::test]
async fn soft_delete_hides_from_list_but_keeps_row() {
    let fx = fixture("soft").await;
    {
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, None)
            .await
            .unwrap();
        uow.commit().await.unwrap();
    }
    stock_in(&fx, "P-3", "A1").await;

    let now = time::now_rfc3339();
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    tubes::soft_delete(uow.conn(), "P-3", &now).await.unwrap();
    uow.commit().await.unwrap();

    let visible = tubes::list(&fx.pool, &tubes::TubeFilter::default())
        .await
        .unwrap();
    assert!(visible.is_empty(), "软删后不应出现在列表");

    let with_deleted = tubes::list(
        &fx.pool,
        &tubes::TubeFilter {
            include_deleted: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(with_deleted.len(), 1);
    assert!(with_deleted[0].deleted_at.is_some());

    // 记录还在（不是物理删除）
    assert!(tubes::find(&fx.pool, "P-3").await.unwrap().is_some());

    fx.cleanup().await;
}

#[tokio::test]
async fn used_count_tracks_in_stock_tubes_and_detects_drift() {
    let fx = fixture("used").await;
    {
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, Some(10))
            .await
            .unwrap();
        locations::insert(uow.conn(), "A2", None, None)
            .await
            .unwrap();
        uow.commit().await.unwrap();
    }
    stock_in(&fx, "P-10", "A1").await;
    stock_in(&fx, "P-11", "A1").await;

    let a1 = locations::find(&fx.pool, "A1").await.unwrap().unwrap();
    assert_eq!(a1.used_count, 2);
    assert!(locations::drift(&fx.pool).await.unwrap().is_empty());

    // 人为制造漂移
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    locations::adjust_used_count(uow.conn(), "A1", -1)
        .await
        .unwrap();
    uow.commit().await.unwrap();
    let drift = locations::drift(&fx.pool).await.unwrap();
    assert_eq!(drift, vec![("A1".to_owned(), 1, 2)]);

    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let actual = locations::refresh_used_count(uow.conn(), "A1")
        .await
        .unwrap();
    uow.commit().await.unwrap();
    assert_eq!(actual, 2);
    assert!(locations::drift(&fx.pool).await.unwrap().is_empty());

    // 减到负数被 CHECK 拦下
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let err = locations::adjust_used_count(uow.conn(), "A1", -100)
        .await
        .unwrap_err();
    assert!(matches!(err, InfraError::Conflict(_)), "实际：{err:?}");
    uow.rollback().await.unwrap();

    fx.cleanup().await;
}

#[tokio::test]
async fn transaction_items_cascade_with_header() {
    let fx = fixture("txn").await;
    let now = time::now_rfc3339();
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let id = transactions::insert(uow.conn(), TransactionType::In, 1, Some("R-1"), None, &now)
        .await
        .unwrap();
    transactions::insert_item(uow.conn(), id, "P-1", None, Some("A1"))
        .await
        .unwrap();
    transactions::insert_item(uow.conn(), id, "P-2", None, Some("A1"))
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let items = transactions::items(&fx.pool, id).await.unwrap();
    assert_eq!(items.len(), 2);

    // 删除单据头 → 明细级联删除（设计 §4.5）
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(uow.conn())
        .await
        .unwrap();
    uow.commit().await.unwrap();
    assert!(transactions::items(&fx.pool, id).await.unwrap().is_empty());

    fx.cleanup().await;
}

#[tokio::test]
async fn standards_grade_is_unique_and_update_conflicts() {
    let fx = fixture("std").await;

    // 1) 先提交两条规格
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let id = standards::insert(
        uow.conn(),
        "L80",
        Some(88.9),
        Some(6.45),
        Some(7.34),
        None,
        Some("API 5CT §..."),
    )
    .await
    .unwrap();
    let err = standards::insert(uow.conn(), "L80", None, None, None, None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, InfraError::Conflict(_)), "实际：{err:?}");
    standards::insert(uow.conn(), "N80", None, None, None, None, None)
        .await
        .unwrap();
    uow.commit().await.unwrap();

    // 2) 把 L80 改成已存在的 N80 → 冲突，只回滚这一次更新
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    let err = standards::update(uow.conn(), id, "N80", None, None, None, None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, InfraError::Conflict(_)), "实际：{err:?}");
    uow.rollback().await.unwrap();

    let row = standards::find_by_grade(&fx.pool, "L80")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.to_spec().grade, "L80");

    fx.cleanup().await;
}

#[tokio::test]
async fn screen_and_seamless_specs_attach_to_tube() {
    let fx = fixture("specs").await;
    {
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, None)
            .await
            .unwrap();
        uow.commit().await.unwrap();
    }
    stock_in(&fx, "P-20", "A1").await;

    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    tubes::insert_seamless_spec(uow.conn(), "P-20", Some("磷化"), Some("塑料帽"))
        .await
        .unwrap();
    let screen = ScreenPipeParams::new(0.4, 40.0, 120.0, Some("直缝".into()));
    // 筛管参数表对任意管子都建立了 1:1 记录（业务上由调用方决定是否写）
    tubes::insert_screen_spec(uow.conn(), "P-20", &screen, Some("slotted"), Some("L80"))
        .await
        .unwrap();
    uow.commit().await.unwrap();

    // 主表删除管子（物理）会级联，但业务走软删；这里直接验证 FK 存在
    let mut uow = Uow::begin(&fx.pool).await.unwrap();
    sqlx::query("DELETE FROM tubes WHERE pipe_id = ?")
        .bind("P-20")
        .execute(uow.conn())
        .await
        .unwrap();
    uow.commit().await.unwrap();

    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM seamless_pipe_specs")
        .fetch_one(&fx.pool)
        .await
        .unwrap();
    assert_eq!(left, 0, "扩展表应随管子级联删除");

    fx.cleanup().await;
}

#[tokio::test]
async fn sorting_follows_whitelist_with_stable_id_tiebreak() {
    use steelpipe_infra::repo::tubes::TubeSort;

    let fx = fixture("sort").await;
    {
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        locations::insert(uow.conn(), "A1", None, None)
            .await
            .unwrap();
        uow.commit().await.unwrap();
    }
    for (pipe_id, grade) in [("P-1", "L80"), ("P-2", "N80"), ("P-3", "P110")] {
        let mut uow = Uow::begin(&fx.pool).await.unwrap();
        let mut d = draft(pipe_id, "A1");
        d.steel_grade = grade.into();
        tubes::insert(
            uow.conn(),
            pipe_id,
            &d,
            true,
            "2026-11-17T00:00:00.000000+00:00",
            "2026-11-17T00:00:00.000000+00:00",
        )
        .await
        .unwrap();
        uow.commit().await.unwrap();
    }

    let grade_asc = tubes::list(
        &fx.pool,
        &tubes::TubeFilter {
            sort: Some(TubeSort::SteelGrade),
            limit: 10,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        grade_asc
            .iter()
            .map(|row| row.steel_grade.as_str())
            .collect::<Vec<_>>(),
        ["L80", "N80", "P110"]
    );

    let grade_desc = tubes::list(
        &fx.pool,
        &tubes::TubeFilter {
            sort: Some(TubeSort::SteelGrade),
            descending: true,
            limit: 10,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        grade_desc
            .iter()
            .map(|row| row.steel_grade.as_str())
            .collect::<Vec<_>>(),
        ["P110", "N80", "L80"]
    );

    // 无排序 → 按 id 升序（稳定）
    let by_id = tubes::list(
        &fx.pool,
        &tubes::TubeFilter {
            limit: 10,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        by_id
            .iter()
            .map(|row| row.pipe_id.as_str())
            .collect::<Vec<_>>(),
        ["P-1", "P-2", "P-3"]
    );

    // 分页稳定：同值字段（location 全相同）按 id 兜底，两页拼接无重复
    let page1 = tubes::list(
        &fx.pool,
        &tubes::TubeFilter {
            sort: Some(TubeSort::Location),
            limit: 2,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let page2 = tubes::list(
        &fx.pool,
        &tubes::TubeFilter {
            sort: Some(TubeSort::Location),
            limit: 2,
            offset: 2,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let mut ids: Vec<&str> = page1
        .iter()
        .chain(page2.iter())
        .map(|row| row.pipe_id.as_str())
        .collect();
    assert_eq!(ids, ["P-1", "P-2", "P-3"]);
    ids.dedup();
    assert_eq!(ids.len(), 3, "分页拼接不得重复");

    fx.cleanup().await;
}
