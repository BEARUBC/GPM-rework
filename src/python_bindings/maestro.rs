use pyo3::prelude::*;

// TODO: Once gpm_original is added as dependency/submodule:
// use gpm_original::resources::Maestro as RustMaestro;
// use gpm_original::resources::Resource;

// Temporary: using local hardware module
use crate::hardware::maestro::Maestro as RustMaestro;
use crate::hardware::Resource;

/// Python-exposed Maestro servo controller
#[pyclass(name = "Maestro")]
pub struct Maestro {
    inner: RustMaestro,
}

#[pymethods]
impl Maestro {
    /// Initialize the Maestro controller
    #[new]
    pub fn new() -> PyResult<Self> {
        let inner = RustMaestro::init();
        Ok(Maestro { inner })
    }

    /// Set target PWM for a servo channel
    ///
    /// Args:
    ///     channel: Servo channel (0-5)
    ///     pwm_value: PWM value (typically 1000-2000)
    pub fn set_target(&mut self, channel: u8, pwm_value: u16) -> PyResult<()> {
        self.inner
            .set_target(channel, pwm_value)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Maestro error: {}", e)))
    }

    /// Move to predefined grip type
    ///
    /// Args:
    ///     grip_type: One of "rest", "pinch", "power", "open"
    pub fn move_to_grip(&mut self, grip_type: &str) -> PyResult<()> {
        self.inner
            .move_to_grip(grip_type)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Move failed: {}", e)))
    }

    /// Get current PWM value for channel
    ///
    /// Args:
    ///     channel: Servo channel (0-5)
    ///
    /// Returns:
    ///     Current PWM value
    pub fn current_pwm(&self, channel: u8) -> PyResult<u16> {
        self.inner
            .current_pwm(channel)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Read failed: {}", e)))
    }
}
