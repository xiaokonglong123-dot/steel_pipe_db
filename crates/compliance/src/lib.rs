//! API 5CT 合规判定：纯函数，五项检查。
//!
//! 搬自旧 `rust-core/src/compliance.rs`（342 行，其中 32 行 pyo3 胶水），
//! 判定逻辑与提示文案逐条保留：`check_name` 与 `reason` 是用户可见内容，
//! 改动会让存量审计记录的解释失配。
//!
//! 五项检查固定顺序、恒定产出 5 条结果（跳过项也占位，`reason` 写明原因）：
//!
//! 1. `steel_grade_match` —— 钢级精确匹配
//! 2. `wall_thickness_range` —— 壁厚落在 [min, max]，边界含
//! 3. `outer_diameter_match` —— 外径偏差 ≤ 1%
//! 4. `length_positive` —— 长度 > 0
//! 5. `screen_specs_valid` —— 筛管必须有筛管参数且槽宽/过滤精度 > 0；非筛管跳过

#![forbid(unsafe_code)]

use serde::Serialize;
use steelpipe_domain::TubeType;

/// 筛管/割缝管参数（仅 `screen_pipe` 相关）。
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScreenPipeParams {
    pub slot_width_mm: f64,
    pub slot_density: f64,
    pub filtration_rating_um: f64,
    pub slot_pattern: Option<String>,
}

impl ScreenPipeParams {
    pub fn new(
        slot_width_mm: f64,
        slot_density: f64,
        filtration_rating_um: f64,
        slot_pattern: Option<String>,
    ) -> Self {
        Self {
            slot_width_mm,
            slot_density,
            filtration_rating_um,
            slot_pattern,
        }
    }
}

/// 入库管子参数。
#[derive(Debug, Clone, Serialize)]
pub struct TubeParams {
    pub pipe_type: TubeType,
    pub outer_diameter_mm: f64,
    pub wall_thickness_mm: f64,
    pub length_m: f64,
    pub steel_grade: String,
    pub thread_type: Option<String>,
    pub screen_specs: Option<ScreenPipeParams>,
}

impl TubeParams {
    pub fn new(
        pipe_type: TubeType,
        outer_diameter_mm: f64,
        wall_thickness_mm: f64,
        length_m: f64,
        steel_grade: impl Into<String>,
    ) -> Self {
        Self {
            pipe_type,
            outer_diameter_mm,
            wall_thickness_mm,
            length_m,
            steel_grade: steel_grade.into(),
            thread_type: None,
            screen_specs: None,
        }
    }

    pub fn with_screen_specs(mut self, specs: ScreenPipeParams) -> Self {
        self.screen_specs = Some(specs);
        self
    }

    pub fn with_thread_type(mut self, thread_type: impl Into<String>) -> Self {
        self.thread_type = Some(thread_type.into());
        self
    }
}

/// 标准规格（来自 `standard_specs` 表）。
#[derive(Debug, Clone, Serialize)]
pub struct StandardSpec {
    pub grade: String,
    pub outer_diameter_mm: Option<f64>,
    pub min_wall_thickness_mm: Option<f64>,
    pub max_wall_thickness_mm: Option<f64>,
    pub length_tolerance_mm: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceCheck {
    pub check_name: String,
    pub passed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceResult {
    pub passed: bool,
    pub checks: Vec<ComplianceCheck>,
}

impl ComplianceResult {
    /// 失败的检查项（用于错误提示与审计 detail）。
    pub fn failures(&self) -> impl Iterator<Item = &ComplianceCheck> {
        self.checks.iter().filter(|check| !check.passed)
    }
}

fn check(name: &str, passed: bool, reason: Option<String>) -> ComplianceCheck {
    ComplianceCheck {
        check_name: name.to_owned(),
        passed,
        reason,
    }
}

/// 执行五项检查。调用方负责：无匹配规格时拒绝入库（不得静默通过）。
pub fn validate_tube(params: &TubeParams, spec: &StandardSpec) -> ComplianceResult {
    let mut checks = Vec::with_capacity(5);

    // 1. 钢级精确匹配
    let grade_ok = params.steel_grade == spec.grade;
    checks.push(check(
        "steel_grade_match",
        grade_ok,
        if grade_ok {
            None
        } else {
            Some(format!(
                "expected grade '{}', got '{}'",
                spec.grade, params.steel_grade
            ))
        },
    ));

    // 2. 壁厚区间（可选，边界含）
    match (spec.min_wall_thickness_mm, spec.max_wall_thickness_mm) {
        (None, None) => checks.push(check(
            "wall_thickness_range",
            true,
            Some("no spec value provided, skipped".into()),
        )),
        (min, max) => {
            let wt = params.wall_thickness_mm;
            let ok = match (min, max) {
                (Some(mn), Some(mx)) => wt >= mn && wt <= mx,
                (Some(mn), None) => wt >= mn,
                (None, Some(mx)) => wt <= mx,
                (None, None) => unreachable!("上面已处理双 None"),
            };
            let reason = if ok {
                None
            } else {
                let detail = match (min, max) {
                    (Some(mn), Some(mx)) => format!("{wt:.3} not in [{mn:.3}, {mx:.3}]"),
                    (Some(mn), None) => format!("{wt:.3} < min {mn:.3}"),
                    (None, Some(mx)) => format!("{wt:.3} > max {mx:.3}"),
                    (None, None) => unreachable!("上面已处理双 None"),
                };
                Some(format!("wall thickness {detail}"))
            };
            checks.push(check("wall_thickness_range", ok, reason));
        }
    }

    // 3. 外径偏差 ≤ 1%（无规格则跳过）
    match spec.outer_diameter_mm {
        Some(od) => {
            let rel_diff = (params.outer_diameter_mm - od).abs() / od.abs().max(1e-10);
            let ok = rel_diff <= 0.01;
            checks.push(check(
                "outer_diameter_match",
                ok,
                if ok {
                    None
                } else {
                    Some(format!(
                        "outer diameter {:.3} deviates {:.2}% from spec {:.3} (> ±1%)",
                        params.outer_diameter_mm,
                        rel_diff * 100.0,
                        od
                    ))
                },
            ));
        }
        None => checks.push(check(
            "outer_diameter_match",
            true,
            Some("no spec value provided, skipped".into()),
        )),
    }

    // 4. 长度必须为正
    let len_ok = params.length_m > 0.0;
    checks.push(check(
        "length_positive",
        len_ok,
        if len_ok {
            None
        } else {
            Some("length must be positive".into())
        },
    ));

    // 5. 筛管参数（仅 screen_pipe）
    if params.pipe_type == TubeType::ScreenPipe {
        match &params.screen_specs {
            Some(specs) => {
                let slot_ok = specs.slot_width_mm > 0.0;
                let rating_ok = specs.filtration_rating_um > 0.0;
                let ok = slot_ok && rating_ok;
                let reason = if ok {
                    None
                } else {
                    let mut parts: Vec<String> = Vec::new();
                    if !slot_ok {
                        parts.push("slot_width_mm must be > 0".to_owned());
                    }
                    if !rating_ok {
                        parts.push("filtration_rating_um must be > 0".to_owned());
                    }
                    Some(parts.join("; "))
                };
                checks.push(check("screen_specs_valid", ok, reason));
            }
            None => checks.push(check(
                "screen_specs_valid",
                false,
                Some("screen_pipe requires screen_specs but none provided".into()),
            )),
        }
    } else {
        checks.push(check(
            "screen_specs_valid",
            true,
            Some("not a screen_pipe, skipped".into()),
        ));
    }

    let passed = checks.iter().all(|check| check.passed);
    ComplianceResult { passed, checks }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> StandardSpec {
        StandardSpec {
            grade: "L80".into(),
            outer_diameter_mm: Some(88.9),
            min_wall_thickness_mm: Some(6.45),
            max_wall_thickness_mm: Some(7.34),
            length_tolerance_mm: Some(500.0),
        }
    }

    fn seamless() -> TubeParams {
        TubeParams::new(TubeType::SeamlessPipe, 88.9, 7.0, 9.5, "L80")
    }

    fn by_name<'a>(result: &'a ComplianceResult, name: &str) -> &'a ComplianceCheck {
        result
            .checks
            .iter()
            .find(|check| check.check_name == name)
            .unwrap_or_else(|| panic!("缺少检查项 {name}"))
    }

    #[test]
    fn always_returns_five_checks_in_fixed_order() {
        let result = validate_tube(&seamless(), &spec());
        let names: Vec<&str> = result
            .checks
            .iter()
            .map(|check| check.check_name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "steel_grade_match",
                "wall_thickness_range",
                "outer_diameter_match",
                "length_positive",
                "screen_specs_valid"
            ]
        );
    }

    #[test]
    fn compliant_seamless_pipe_passes() {
        let result = validate_tube(&seamless(), &spec());
        assert!(
            result.passed,
            "失败项：{:?}",
            result.failures().collect::<Vec<_>>()
        );
        assert!(by_name(&result, "screen_specs_valid").passed);
        assert_eq!(
            by_name(&result, "screen_specs_valid").reason.as_deref(),
            Some("not a screen_pipe, skipped")
        );
    }

    #[test]
    fn grade_mismatch_fails_with_expected_text() {
        let params = TubeParams::new(TubeType::SeamlessPipe, 88.9, 7.0, 9.5, "N80");
        let result = validate_tube(&params, &spec());
        assert!(!result.passed);
        let check = by_name(&result, "steel_grade_match");
        assert!(!check.passed);
        assert_eq!(
            check.reason.as_deref(),
            Some("expected grade 'L80', got 'N80'")
        );
    }

    #[test]
    fn wall_thickness_bounds_are_inclusive() {
        for wall in [6.45, 7.34] {
            let params = TubeParams::new(TubeType::SeamlessPipe, 88.9, wall, 9.5, "L80");
            let result = validate_tube(&params, &spec());
            assert!(
                by_name(&result, "wall_thickness_range").passed,
                "壁厚 {wall} 应通过"
            );
        }

        let below = TubeParams::new(TubeType::SeamlessPipe, 88.9, 6.44, 9.5, "L80");
        let result = validate_tube(&below, &spec());
        let check = by_name(&result, "wall_thickness_range");
        assert!(!check.passed);
        assert!(
            check
                .reason
                .as_deref()
                .unwrap()
                .contains("not in [6.450, 7.340]")
        );

        let above = TubeParams::new(TubeType::SeamlessPipe, 88.9, 7.35, 9.5, "L80");
        let above_result = validate_tube(&above, &spec());
        assert!(!by_name(&above_result, "wall_thickness_range").passed);
    }

    #[test]
    fn wall_range_supports_one_sided_specs() {
        let mut min_only = spec();
        min_only.max_wall_thickness_mm = None;
        assert!(
            by_name(
                &validate_tube(&seamless(), &min_only),
                "wall_thickness_range"
            )
            .passed
        );

        let mut max_only = spec();
        max_only.min_wall_thickness_mm = None;
        assert!(
            by_name(
                &validate_tube(&seamless(), &max_only),
                "wall_thickness_range"
            )
            .passed
        );

        let mut none = spec();
        none.min_wall_thickness_mm = None;
        none.max_wall_thickness_mm = None;
        let none_result = validate_tube(&seamless(), &none);
        let check = by_name(&none_result, "wall_thickness_range");
        assert!(check.passed);
        assert_eq!(
            check.reason.as_deref(),
            Some("no spec value provided, skipped")
        );
    }

    #[test]
    fn outer_diameter_is_plus_minus_one_percent() {
        let at_limit = TubeParams::new(TubeType::SeamlessPipe, 88.9 * 1.01, 7.0, 9.5, "L80");
        assert!(
            by_name(&validate_tube(&at_limit, &spec()), "outer_diameter_match").passed,
            "1.00% 偏差应通过"
        );

        let over = TubeParams::new(TubeType::SeamlessPipe, 88.9 * 1.011, 7.0, 9.5, "L80");
        let result = validate_tube(&over, &spec());
        let check = by_name(&result, "outer_diameter_match");
        assert!(!check.passed);
        assert!(check.reason.as_deref().unwrap().contains("> ±1%"));

        let under = TubeParams::new(TubeType::SeamlessPipe, 88.9 * 0.98, 7.0, 9.5, "L80");
        let under_result = validate_tube(&under, &spec());
        assert!(!by_name(&under_result, "outer_diameter_match").passed);
    }

    #[test]
    fn non_positive_length_fails() {
        for length in [0.0, -1.0] {
            let params = TubeParams::new(TubeType::SeamlessPipe, 88.9, 7.0, length, "L80");
            let result = validate_tube(&params, &spec());
            let check = by_name(&result, "length_positive");
            assert!(!check.passed, "长度 {length} 应失败");
            assert_eq!(check.reason.as_deref(), Some("length must be positive"));
        }
    }

    #[test]
    fn screen_pipe_without_specs_fails() {
        let params = TubeParams::new(TubeType::ScreenPipe, 88.9, 7.0, 9.5, "L80");
        let result = validate_tube(&params, &spec());
        assert!(!result.passed);
        let check = by_name(&result, "screen_specs_valid");
        assert_eq!(
            check.reason.as_deref(),
            Some("screen_pipe requires screen_specs but none provided")
        );
    }

    #[test]
    fn screen_pipe_specs_must_be_positive() {
        let params = TubeParams::new(TubeType::ScreenPipe, 88.9, 7.0, 9.5, "L80")
            .with_screen_specs(ScreenPipeParams::new(0.0, 40.0, -1.0, None));
        let result = validate_tube(&params, &spec());
        let check = by_name(&result, "screen_specs_valid");
        assert!(!check.passed);
        assert_eq!(
            check.reason.as_deref(),
            Some("slot_width_mm must be > 0; filtration_rating_um must be > 0")
        );

        let good = TubeParams::new(TubeType::ScreenPipe, 88.9, 7.0, 9.5, "L80")
            .with_screen_specs(ScreenPipeParams::new(0.5, 40.0, 120.0, Some("直缝".into())));
        let result = validate_tube(&good, &spec());
        assert!(
            result.passed,
            "失败项：{:?}",
            result.failures().collect::<Vec<_>>()
        );
    }

    #[test]
    fn skipped_checks_still_report_reason() {
        let mut loose = spec();
        loose.outer_diameter_mm = None;
        let result = validate_tube(&seamless(), &loose);
        let check = by_name(&result, "outer_diameter_match");
        assert!(check.passed);
        assert_eq!(
            check.reason.as_deref(),
            Some("no spec value provided, skipped")
        );
    }

    #[test]
    fn failures_helper_lists_only_failing_checks() {
        let params = TubeParams::new(TubeType::SeamlessPipe, 200.0, 2.0, 0.0, "N80");
        let result = validate_tube(&params, &spec());
        let names: Vec<&str> = result
            .failures()
            .map(|check| check.check_name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "steel_grade_match",
                "wall_thickness_range",
                "outer_diameter_match",
                "length_positive"
            ]
        );
    }
}
