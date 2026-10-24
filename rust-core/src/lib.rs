use pyo3::prelude::*;

mod auth;
mod audit;
mod compliance;

/// Rust_core — Ayanami_Rei native extension (PyO3).
///
/// Three submodules exposed at the Python level:
///   from rust_core import auth as rust_auth
///   from rust_core import compliance as rust_compliance
///   from rust_core import audit as rust_audit
#[pymodule]
fn rust_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // --- auth submodule ---
    {
        let sub = PyModule::new(m.py(), "auth")?;
        auth::register(&sub)?;
        m.add_submodule(&sub)?;
    }

    // --- compliance submodule ---
    {
        let sub = PyModule::new(m.py(), "compliance")?;
        compliance::register(&sub)?;
        m.add_submodule(&sub)?;
    }

    // --- audit submodule ---
    {
        let sub = PyModule::new(m.py(), "audit")?;
        audit::register(&sub)?;
        m.add_submodule(&sub)?;
    }

    Ok(())
}
