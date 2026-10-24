//! Compliance submodule — API 5CT tube specification validation (pure functions).
//!
//! Python passes in the spec (from `standard_specs` table) and tube params.
//! Rust remains stateless — no API 5CT tables baked in.

use pyo3::prelude::*;

// ---------------------------------------------------------------------------
// Data structures  (all `#[pyclass]` so Python can construct / inspect them)
// ---------------------------------------------------------------------------

/// Screen / slotted liner parameters (only relevant when `pipe_type == "screen_pipe"`).
#[pyclass]
#[derive(Clone)]
pub struct ScreenPipeParams {
    #[pyo3(get)]
    pub slot_width_mm: f64,
    #[pyo3(get)]
    pub slot_density: f64,
    #[pyo3(get)]
    pub filtration_rating_um: f64,
    #[pyo3(get)]
    pub slot_pattern: Option<String>,
}

#[pymethods]
impl ScreenPipeParams {
    #[new]
    #[pyo3(signature = (slot_width_mm, slot_density, filtration_rating_um, slot_pattern=None))]
    fn new(
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

/// Tube parameters (the "what" being checked).
#[pyclass]
pub struct TubeParams {
    #[pyo3(get)]
    pub pipe_type: String,
    #[pyo3(get)]
    pub outer_diameter_mm: f64,
    #[pyo3(get)]
    pub wall_thickness_mm: f64,
    #[pyo3(get)]
    pub length_m: f64,
    #[pyo3(get)]
    pub steel_grade: String,
    #[pyo3(get)]
    pub thread_type: Option<String>,
    #[pyo3(get)]
    pub screen_specs: Option<Py<ScreenPipeParams>>,
}

#[pymethods]
impl TubeParams {
    #[new]
    #[pyo3(signature = (pipe_type, outer_diameter_mm, wall_thickness_mm, length_m, steel_grade, thread_type=None, screen_specs=None))]
    fn new(
        pipe_type: String,
        outer_diameter_mm: f64,
        wall_thickness_mm: f64,
        length_m: f64,
        steel_grade: String,
        thread_type: Option<String>,
        screen_specs: Option<Py<ScreenPipeParams>>,
    ) -> Self {
        Self {
            pipe_type,
            outer_diameter_mm,
            wall_thickness_mm,
            length_m,
            steel_grade,
            thread_type,
            screen_specs,
        }
    }
}

/// Standard specification (the "against what" being checked).  
/// Populated by Python from the `standard_specs` DB table.
#[pyclass]
pub struct StandardSpec {
    #[pyo3(get)]
    pub grade: String,
    #[pyo3(get)]
    pub outer_diameter_mm: Option<f64>,
    #[pyo3(get)]
    pub min_wall_thickness_mm: Option<f64>,
    #[pyo3(get)]
    pub max_wall_thickness_mm: Option<f64>,
    #[pyo3(get)]
    pub length_tolerance_mm: Option<f64>,
    #[pyo3(get)]
    pub source: Option<String>,
}

#[pymethods]
impl StandardSpec {
    #[new]
    #[pyo3(signature = (grade, outer_diameter_mm=None, min_wall_thickness_mm=None, max_wall_thickness_mm=None, length_tolerance_mm=None, source=None))]
    fn new(
        grade: String,
        outer_diameter_mm: Option<f64>,
        min_wall_thickness_mm: Option<f64>,
        max_wall_thickness_mm: Option<f64>,
        length_tolerance_mm: Option<f64>,
        source: Option<String>,
    ) -> Self {
        Self {
            grade,
            outer_diameter_mm,
            min_wall_thickness_mm,
            max_wall_thickness_mm,
            length_tolerance_mm,
            source,
        }
    }
}

/// One compliance check result.
#[pyclass]
#[derive(Clone)]
pub struct ComplianceCheck {
    #[pyo3(get)]
    pub check_name: String,
    #[pyo3(get)]
    pub passed: bool,
    #[pyo3(get)]
    pub reason: Option<String>,
}

#[pymethods]
impl ComplianceCheck {
    #[new]
    #[pyo3(signature = (check_name, passed, reason=None))]
    fn new(check_name: String, passed: bool, reason: Option<String>) -> Self {
        Self {
            check_name,
            passed,
            reason,
        }
    }
}

/// Aggregate validation result (`.passed` is true when all checks pass).
#[pyclass]
pub struct ComplianceResult {
    #[pyo3(get, set)]
    pub passed: bool,
    #[pyo3(get, set)]
    pub checks: Vec<ComplianceCheck>,
}

#[pymethods]
impl ComplianceResult {
    #[new]
    fn new(passed: bool, checks: Vec<ComplianceCheck>) -> Self {
        Self { passed, checks }
    }
}

// ---------------------------------------------------------------------------
// Validation engine
// ---------------------------------------------------------------------------

/// Validate tube parameters against a standard spec.
///
/// Every applicable check produces a `ComplianceCheck`; `result.passed` is
/// `true` iff all checks passed.  A spec field that is `None` causes its
/// corresponding check to be skipped (recorded as passed).
#[pyfunction]
fn validate_tube(
    py: Python,
    params: &Bound<'_, TubeParams>,
    spec: &Bound<'_, StandardSpec>,
) -> PyResult<ComplianceResult> {
    let p = params.borrow();
    let s = spec.borrow();

    let mut checks: Vec<ComplianceCheck> = Vec::with_capacity(5);

    // 1.  steel_grade_match — exact string match
    let grade_ok = p.steel_grade == s.grade;
    checks.push(ComplianceCheck {
        check_name: "steel_grade_match".into(),
        passed: grade_ok,
        reason: if grade_ok {
            None
        } else {
            Some(format!(
                "expected grade '{}', got '{}'",
                s.grade, p.steel_grade
            ))
        },
    });

    // 2.  wall_thickness_range — optional [min, max] inclusive
    match (s.min_wall_thickness_mm, s.max_wall_thickness_mm) {
        (None, None) => {
            // no spec → skip
            checks.push(ComplianceCheck {
                check_name: "wall_thickness_range".into(),
                passed: true,
                reason: Some("no spec value provided, skipped".into()),
            });
        }
        (min, max) => {
            let wt_ok = match (min, max) {
                (Some(mn), Some(mx)) => p.wall_thickness_mm >= mn && p.wall_thickness_mm <= mx,
                (Some(mn), None) => p.wall_thickness_mm >= mn,
                (None, Some(mx)) => p.wall_thickness_mm <= mx,
                (None, None) => unreachable!(), // handled above
            };
            checks.push(ComplianceCheck {
                check_name: "wall_thickness_range".into(),
                passed: wt_ok,
                reason: if wt_ok {
                    None
                } else {
                    let detail = match (min, max) {
                        (Some(mn), Some(mx)) => {
                            format!("{:.3} not in [{:.3}, {:.3}]", p.wall_thickness_mm, mn, mx)
                        }
                        (Some(mn), None) => {
                            format!("{:.3} < min {:.3}", p.wall_thickness_mm, mn)
                        }
                        (None, Some(mx)) => {
                            format!("{:.3} > max {:.3}", p.wall_thickness_mm, mx)
                        }
                        (None, None) => unreachable!(),
                    };
                    Some(format!("wall thickness {}", detail))
                },
            });
        }
    }

    // 3.  outer_diameter_match — within ±1 % of spec value
    if let Some(od) = s.outer_diameter_mm {
        let rel_diff = (p.outer_diameter_mm - od).abs() / od.abs().max(1e-10);
        let od_ok = rel_diff <= 0.01; // ≤ 1 %
        checks.push(ComplianceCheck {
            check_name: "outer_diameter_match".into(),
            passed: od_ok,
            reason: if od_ok {
                None
            } else {
                Some(format!(
                    "outer diameter {:.3} deviates {:.2}% from spec {:.3} (> ±1%)",
                    p.outer_diameter_mm,
                    rel_diff * 100.0,
                    od
                ))
            },
        });
    } else {
        checks.push(ComplianceCheck {
            check_name: "outer_diameter_match".into(),
            passed: true,
            reason: Some("no spec value provided, skipped".into()),
        });
    }

    // 4.  length_positive — basic sanity
    let len_ok = p.length_m > 0.0;
    checks.push(ComplianceCheck {
        check_name: "length_positive".into(),
        passed: len_ok,
        reason: if len_ok {
            None
        } else {
            Some("length must be positive".into())
        },
    });

    // 5.  screen_specs_valid — only for screen_pipe
    if p.pipe_type == "screen_pipe" {
        match &p.screen_specs {
            Some(ss_ref) => {
                let ss = ss_ref.borrow(py);
                let sw_ok = ss.slot_width_mm > 0.0;
                let fr_ok = ss.filtration_rating_um > 0.0;
                let ss_ok = sw_ok && fr_ok;
                checks.push(ComplianceCheck {
                    check_name: "screen_specs_valid".into(),
                    passed: ss_ok,
                    reason: if ss_ok {
                        None
                    } else {
                        let mut parts: Vec<String> = Vec::new();
                        if !sw_ok {
                            parts.push("slot_width_mm must be > 0".into());
                        }
                        if !fr_ok {
                            parts.push("filtration_rating_um must be > 0".into());
                        }
                        Some(parts.join("; "))
                    },
                });
            }
            None => {
                checks.push(ComplianceCheck {
                    check_name: "screen_specs_valid".into(),
                    passed: false,
                    reason: Some("screen_pipe requires screen_specs but none provided".into()),
                });
            }
        }
    } else {
        checks.push(ComplianceCheck {
            check_name: "screen_specs_valid".into(),
            passed: true,
            reason: Some("not a screen_pipe, skipped".into()),
        });
    }

    let passed = checks.iter().all(|c| c.passed);
    Ok(ComplianceResult { passed, checks })
}

// ---------------------------------------------------------------------------
// Submodule registration
// ---------------------------------------------------------------------------
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TubeParams>()?;
    m.add_class::<ScreenPipeParams>()?;
    m.add_class::<StandardSpec>()?;
    m.add_class::<ComplianceCheck>()?;
    m.add_class::<ComplianceResult>()?;
    m.add_function(wrap_pyfunction!(validate_tube, m)?)?;
    Ok(())
}
