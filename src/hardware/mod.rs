// ============================================================================
// TEMPORARY HARDWARE IMPLEMENTATIONS
// ============================================================================
// WARNING: These are placeholder implementations for development only.
// The actual hardware drivers will come from the gpm_original submodule.
//
// DO NOT MODIFY THESE FILES - they will be removed when gpm_original is integrated.
//
// Rust team responsibility: Complete gpm_original submodule with proper hardware drivers
// Python team responsibility: Ensure python_bindings/ works with gpm_original's API
//
// See SUBMODULE_INTEGRATION.md for integration steps.
// ============================================================================

pub mod maestro;
pub mod emg;
pub mod fsr;
pub mod bms;
pub mod adc;

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
