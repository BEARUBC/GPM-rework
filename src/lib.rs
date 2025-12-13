use pyo3::prelude::*;

// ============================================================================
// GPM Python Extension Module
// ============================================================================
// This module provides Python bindings to hardware drivers.
//
// ARCHITECTURE:
// - Hardware implementations: Will come from gpm_original (as submodule)
// - Python bindings: This crate (src/python_bindings/)
// - Python logic: application/ directory
//
// CURRENT STATUS: Using temporary local hardware until submodule is integrated
// See SUBMODULE_INTEGRATION.md for transition plan
// ============================================================================

// Temporary: local hardware module (will be replaced by gpm_original)
// TODO: Once gpm_original is integrated, remove this and update bindings to:
//       use gpm_original::resources::*;
mod hardware;

// Python bindings layer - wraps hardware implementations
mod python_bindings;

use python_bindings::{bms, emg, fsr, maestro, manager};

/// Grasp Primary Module - Hardware interface
///
/// This module provides Python bindings to the Rust hardware drivers.
/// The actual hardware implementations come from the gpm_original crate.
#[pymodule]
fn gpm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Hardware resource bindings
    m.add_class::<maestro::Maestro>()?;
    m.add_class::<emg::Emg>()?;
    m.add_class::<bms::Bms>()?;
    m.add_class::<bms::BmsStatus>()?;
    m.add_class::<fsr::Fsr>()?;
    m.add_class::<fsr::FsrReading>()?;

    // Manager pattern (future integration)
    m.add_class::<manager::ResourceManager>()?;

    Ok(())
}
