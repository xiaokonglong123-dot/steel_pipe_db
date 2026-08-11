//! Workflow service — 数据驱动审批流引擎
//!
//! 设计原则（detailed-design §6 / ERPNext-style）：
//! - 工作流定义（workflows + workflow_states + workflow_transitions）纯配置；
//!   新增状态/动作不改代码，引擎读取配置驱动实例流转。
//! - `start_instance`：查找 active 工作流 → 取初始 state → 建实例 + 待办 task。
//!   若初始 state 即为 final，直接置实例 completed。
//! - `transition`：读取当前 state → 按动作查 transition → 角色/权限校验（可选）
//!   → 在单事务内更新实例 state、办结当前 task、若新 state 为 final 则置实例 completed
//!   否则插入下一条 pending task。
//! - 工作流定义的 update/delete 守卫：若有运行中实例则拒绝（Validation 10002）。
//!
//! P0-10 限制：task 的 assignee 暂留 NULL（通用待办）；
//! P0-11 将由 PO/SO submit 调用 start_instance 接入工作流。

use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::error::{AppError, ErrorCode};
use crate::middleware::auth::AuthUser;
use crate::repos::workflow_repo;
use crate::repos::workflow_repo::{
    WorkflowInstanceRow, WorkflowRow, WorkflowStateRow, WorkflowTaskRow, WorkflowTransitionRow,
};
use rust_decimal::Decimal;

// —— DTOs ——

#[derive(Debug, Clone)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub applies_to: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateWorkflowRequest {
    pub name: String,
    pub applies_to: String,
    pub is_active: bool,
}

// —— Definition management（admin only）——

pub async fn create_workflow(
    pool: &SqlitePool,
    dto: &CreateWorkflowRequest,
    _user: &AuthUser,
) -> Result<WorkflowRow, AppError> {
    validate_applies_to(&dto.applies_to)?;
    if dto.name.trim().is_empty() {
        return Err(AppError::validation("工作流名称不能为空"));
    }
    let is_active = if dto.is_active { 1 } else { 0 };
    let id = workflow_repo::insert_workflow(pool, &dto.name, &dto.applies_to, is_active).await?;
    workflow_repo::find_workflow_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "工作流创建后读取失败"))
}

pub async fn list_workflows(pool: &SqlitePool) -> Result<Vec<WorkflowRow>, AppError> {
    workflow_repo::list_workflows(pool).await
}

/// 取单个工作流定义 + states + transitions（供 GET /workflows/{id} 一次性返回完整定义）。
pub async fn get_workflow_definition(
    pool: &SqlitePool,
    id: i64,
) -> Result<WorkflowDefinition, AppError> {
    let workflow = workflow_repo::find_workflow_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::WorkflowNotFound, "审批流未找到"))?;
    let states = workflow_repo::list_states_for_workflow(pool, id).await?;
    let transitions = list_transitions_for_workflow(pool, id).await?;
    Ok(WorkflowDefinition {
        workflow,
        states,
        transitions,
    })
}

pub async fn update_workflow(
    pool: &SqlitePool,
    id: i64,
    dto: &UpdateWorkflowRequest,
    _user: &AuthUser,
) -> Result<WorkflowRow, AppError> {
    validate_applies_to(&dto.applies_to)?;
    let _existing = workflow_repo::find_workflow_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::WorkflowNotFound, "审批流未找到"))?;

    // 守卫：若有运行中实例则拒绝更新
    let active_count = workflow_repo::count_active_instances_for_workflow(pool, id).await?;
    if active_count > 0 {
        return Err(AppError::validation(format!(
            "工作流有 {active_count} 个运行中实例，不可更新（请先等待实例办结或取消）"
        )));
    }

    let is_active = if dto.is_active { 1 } else { 0 };
    workflow_repo::update_workflow(pool, id, &dto.name, &dto.applies_to, is_active).await?;
    workflow_repo::find_workflow_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "工作流更新后读取失败"))
}

pub async fn delete_workflow(pool: &SqlitePool, id: i64, _user: &AuthUser) -> Result<(), AppError> {
    // 先校验存在
    workflow_repo::find_workflow_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::WorkflowNotFound, "审批流未找到"))?;

    // 守卫：若有运行中实例则拒绝删除
    let active_count = workflow_repo::count_active_instances_for_workflow(pool, id).await?;
    if active_count > 0 {
        return Err(AppError::validation(format!(
            "工作流有 {active_count} 个运行中实例，不可删除"
        )));
    }

    workflow_repo::delete_workflow(pool, id).await
}

// —— Engine operations ——

/// 启动一个工作流实例。由 PO/SO submit 在 P0-11 接入。
/// - 找 active 工作流；找不到 → WorkflowNotFound(17001)
/// - 取初始 state（is_initial=1）；找不到 → 内部错误（工作流定义不完整）
/// - 单事务内：插实例（status='active', current_state=initial.state_key）
///   + 插 pending task（state_key=initial.state_key, assignee=NULL）
/// - 若初始 state is_final=1：单事务内同时把实例置 completed（无 task）
pub async fn start_instance(
    pool: &SqlitePool,
    business_type: &str,
    business_id: i64,
    _user: &AuthUser,
) -> Result<WorkflowInstanceRow, AppError> {
    let workflow = workflow_repo::find_active_workflow_by_type(pool, business_type)
        .await?
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::WorkflowNotFound,
                format!("未找到 business_type='{business_type}' 的 active 工作流"),
            )
        })?;

    let initial = workflow_repo::find_initial_state(pool, workflow.id)
        .await?
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::Internal,
                format!("工作流 {} 缺少初始状态（is_initial=1）", workflow.id),
            )
        })?;

    let mut tx = pool.begin().await?;

    let instance_id = workflow_repo::insert_instance(
        &mut *tx,
        workflow.id,
        business_type,
        business_id,
        &initial.state_key,
    )
    .await?;

    if initial.is_final == 1 {
        // 初始即终态：直接置 completed，不建 task
        workflow_repo::update_instance_state(
            &mut *tx,
            instance_id,
            &initial.state_key,
            Some("completed"),
        )
        .await?;
    } else {
        // 建待办 task（P0-10: assignee=NULL，通用待办）
        workflow_repo::insert_task(&mut *tx, instance_id, &initial.state_key, None).await?;
    }

    tx.commit().await?;

    workflow_repo::find_instance_by_id(pool, instance_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "审批实例创建后读取失败"))
}

/// Inserts a workflow instance and its initial task into an existing business transaction.
/// The definition and initial state must be read before the transaction begins.
pub async fn start_instance_in_tx(
    tx: &mut Transaction<'_, Sqlite>,
    workflow: &WorkflowRow,
    initial: &WorkflowStateRow,
    business_type: &str,
    business_id: i64,
) -> Result<(), AppError> {
    let instance_id = workflow_repo::insert_instance(
        &mut **tx,
        workflow.id,
        business_type,
        business_id,
        &initial.state_key,
    )
    .await?;

    if initial.is_final == 1 {
        workflow_repo::update_instance_state(
            &mut **tx,
            instance_id,
            &initial.state_key,
            Some("completed"),
        )
        .await?;
    } else {
        workflow_repo::insert_task(&mut **tx, instance_id, &initial.state_key, None).await?;
    }
    Ok(())
}

/// 按动作驱动实例流转。
/// - 实例非 active → Validation(10002)（"实例已办结/取消"）
/// - 当前 state 无匹配 transition → InvalidTransition(17002)
/// - transition.required_role 非空且用户无该权限 → Forbidden(11003)
/// - 单事务：更新实例 state (+status 若 final) → 办结当前 pending task
///   → 若新 state 非 final 则插新 pending task
pub async fn transition(
    pool: &SqlitePool,
    instance_id: i64,
    action: &str,
    user: &AuthUser,
    comment: Option<String>,
) -> Result<WorkflowInstanceRow, AppError> {
    transition_with_amount(pool, instance_id, action, user, comment, None).await
}

pub async fn transition_with_amount(
    pool: &SqlitePool,
    instance_id: i64,
    action: &str,
    user: &AuthUser,
    comment: Option<String>,
    business_amount: Option<Decimal>,
) -> Result<WorkflowInstanceRow, AppError> {
    let instance = workflow_repo::find_instance_by_id(pool, instance_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "审批实例未找到"))?;
    if instance.status != "active" {
        return Err(AppError::validation(format!(
            "审批实例当前状态为 {}，不可流转",
            instance.status
        )));
    }

    let current_state =
        workflow_repo::find_state_by_key(pool, instance.workflow_id, &instance.current_state)
            .await?
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::Internal,
                    format!(
                        "工作流 {} 的当前 state '{}' 未找到",
                        instance.workflow_id, instance.current_state
                    ),
                )
            })?;

    let all_trs = workflow_repo::list_transitions_by_action(
        pool,
        instance.workflow_id,
        current_state.id,
        action,
    )
    .await?;

    if all_trs.is_empty() {
        return Err(AppError::new(
            ErrorCode::InvalidTransition,
            format!(
                "在 state '{}' 下不存在动作 '{}'",
                current_state.state_key, action
            ),
        ));
    }

    let tr = pick_transition_by_amount(&all_trs, business_amount).ok_or_else(|| {
        AppError::new(
            ErrorCode::InvalidTransition,
            format!(
                "在 state '{}' 下不存在适用于该金额的动作 '{}'",
                current_state.state_key, action
            ),
        )
    })?;

    if let Some(role) = &tr.required_role {
        if !role.is_empty() && !user.has_permission(role) {
            return Err(AppError::new(
                ErrorCode::Forbidden,
                format!("当前用户缺少权限 '{role}'，无法执行动作 '{action}'"),
            ));
        }
    }

    let to_state = workflow_repo::find_state_by_id(pool, tr.to_state_id)
        .await?
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::Internal,
                format!(
                    "transition {} 的目标 state {} 未找到",
                    tr.id, tr.to_state_id
                ),
            )
        })?;

    let mut tx = pool.begin().await?;

    let new_status = if to_state.is_final == 1 {
        Some("completed")
    } else {
        None
    };
    workflow_repo::update_instance_state(&mut *tx, instance_id, &to_state.state_key, new_status)
        .await?;

    if let Some(task) = workflow_repo::find_pending_task_for_instance(&mut *tx, instance_id).await?
    {
        workflow_repo::complete_task(&mut *tx, task.id, action, comment.as_deref()).await?;
    }

    if to_state.is_final == 0 {
        workflow_repo::insert_task(&mut *tx, instance_id, &to_state.state_key, None).await?;
    }

    tx.commit().await?;

    workflow_repo::find_instance_by_id(pool, instance_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "实例更新后丢失"))
}

fn pick_transition_by_amount(
    transitions: &[WorkflowTransitionRow],
    business_amount: Option<Decimal>,
) -> Option<&WorkflowTransitionRow> {
    let amt = business_amount.unwrap_or_default();
    let mut fallback: Option<&WorkflowTransitionRow> = None;
    for tr in transitions {
        match &tr.amount_threshold {
            Some(s) if !s.is_empty() => {
                if let Ok(threshold) = Decimal::from_str_radix(s, 10) {
                    if amt >= threshold {
                        return Some(tr);
                    }
                }
            }
            _ => {
                if fallback.is_none() {
                    fallback = Some(tr);
                }
            }
        }
    }
    fallback
}

// —— Task queries ——

/// 当前用户的待办任务（assignee_id = user.id 或 assignee_id IS NULL）。
pub async fn list_my_tasks(
    pool: &SqlitePool,
    user: &AuthUser,
) -> Result<Vec<WorkflowTaskRow>, AppError> {
    workflow_repo::list_pending_tasks_for_user(pool, user.id).await
}

pub async fn list_tasks_for_instance(
    pool: &SqlitePool,
    instance_id: i64,
) -> Result<Vec<WorkflowTaskRow>, AppError> {
    workflow_repo::list_tasks_for_instance(pool, instance_id).await
}

// —— Instance queries ——

pub async fn find_active_instance_for(
    pool: &SqlitePool,
    business_type: &str,
    business_id: i64,
) -> Result<Option<WorkflowInstanceRow>, AppError> {
    workflow_repo::find_active_instance_for(pool, business_type, business_id).await
}

pub async fn list_instances(
    pool: &SqlitePool,
    business_type_filter: Option<String>,
    status_filter: Option<String>,
) -> Result<Vec<WorkflowInstanceRow>, AppError> {
    workflow_repo::list_instances(
        pool,
        business_type_filter.as_deref(),
        status_filter.as_deref(),
    )
    .await
}

/// GET /workflow-instances/{id} 返回结构：实例 + states（工作流定义）+ tasks
pub async fn get_instance_detail(
    pool: &SqlitePool,
    instance_id: i64,
) -> Result<InstanceDetail, AppError> {
    let instance = workflow_repo::find_instance_by_id(pool, instance_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "审批实例未找到"))?;
    let states = workflow_repo::list_states_for_workflow(pool, instance.workflow_id).await?;
    let tasks = workflow_repo::list_tasks_for_instance(pool, instance_id).await?;
    Ok(InstanceDetail {
        instance,
        states,
        tasks,
    })
}

/// 通过 task 完成接口驱动流转（POST /workflow-tasks/{id}/complete）。
/// 找到 task → 实例 → 调用 transition(instance, action, user, comment)。
/// 这样前端只需"我办理这条待办、我用 X 动作"，引擎自行解析目标 state。
pub async fn complete_task(
    pool: &SqlitePool,
    task_id: i64,
    action: &str,
    user: &AuthUser,
    comment: Option<String>,
) -> Result<WorkflowInstanceRow, AppError> {
    let task = workflow_repo::find_task_by_id(pool, task_id)
        .await?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "任务未找到"))?;
    if task.status != "pending" {
        return Err(AppError::validation(format!(
            "任务当前状态为 {}，不可办结",
            task.status
        )));
    }
    transition(pool, task.instance_id, action, user, comment).await
}

// —— 辅助 ——

/// 列出工作流的全部 transitions（供 definition 详情使用）。
async fn list_transitions_for_workflow(
    pool: &SqlitePool,
    workflow_id: i64,
) -> Result<Vec<WorkflowTransitionRow>, AppError> {
    let rows = sqlx::query_as::<_, WorkflowTransitionRow>(
        "SELECT id, workflow_id, from_state_id, to_state_id, action, required_role, is_auto
         FROM workflow_transitions WHERE workflow_id = ? ORDER BY id",
    )
    .bind(workflow_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 校验 applies_to 取值（对齐 008_workflow.sql CHECK 约束）。
fn validate_applies_to(v: &str) -> Result<(), AppError> {
    match v {
        "purchase_order" | "sales_order" | "inbound_record" | "outbound_record" => Ok(()),
        other => Err(AppError::validation(format!(
            "applies_to 仅支持 purchase_order|sales_order|inbound_record|outbound_record，收到 '{other}'"
        ))),
    }
}

// —— 组合响应类型 ——

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkflowDefinition {
    pub workflow: WorkflowRow,
    pub states: Vec<WorkflowStateRow>,
    pub transitions: Vec<WorkflowTransitionRow>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstanceDetail {
    pub instance: WorkflowInstanceRow,
    pub states: Vec<WorkflowStateRow>,
    pub tasks: Vec<WorkflowTaskRow>,
}
