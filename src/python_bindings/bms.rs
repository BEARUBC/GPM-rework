use pyo3::prelude::*;

// TODO: Once gpm_original is added as dependency/submodule:
// use gpm_original::resources::bms::{Bms as RustBms, BmsStatus as RustBmsStatus};
// use gpm_original::resources::Resource;

// Temporary: using local hardware module
use crate::hardware::bms::{Bms as RustBms, BmsStatus as RustBmsStatus};
use crate::hardware::Resource;

/// Python-exposed BMS status
#[pyclass(name = "BmsStatus")]
#[derive(Clone)]
pub struct BmsStatus {
    #[pyo3(get)]
    pub voltage: f32,
    #[pyo3(get)]
    pub current: f32,
    #[pyo3(get)]
    pub temperature: f32,
    #[pyo3(get)]
    pub is_healthy: bool,
    #[pyo3(get)]
    pub charge_percentage: f32,
}

impl From<RustBmsStatus> for BmsStatus {
    fn from(status: RustBmsStatus) -> Self {
        BmsStatus {
            voltage: status.voltage,
            current: status.current,
            temperature: status.temperature,
            is_healthy: status.is_healthy,
            charge_percentage: status.charge_percentage,
        }
    }
}

/// Python-exposed BMS (Battery Management System) interface
#[pyclass(name = "Bms")]
pub struct Bms {
    inner: RustBms,
}

#[pymethods]
impl Bms {
    /// Initialize the BMS
    #[new]
    pub fn new() -> PyResult<Self> {
        let inner = RustBms::init();
        Ok(Bms { inner })
    }

    /// Get current BMS status
    ///
    /// Returns:
    ///     BmsStatus object with voltage, current, temperature, health status
    pub fn get_status(&mut self) -> PyResult<BmsStatus> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                self.inner.update();
                Ok(self.inner.get_status().into())
            })
        })
    }

    /// Update BMS readings
    pub fn update(&mut self) -> PyResult<()> {
        self.inner.update();
        Ok(())
    }
}
