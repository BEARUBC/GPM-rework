use pyo3::prelude::*;

// TODO: Once gpm_original is added as dependency/submodule:
// use gpm_original::resources::fsr::{Fsr as RustFsr, FsrReading as RustFsrReading};
// use gpm_original::resources::Resource;

// Temporary: using local hardware module
use crate::hardware::fsr::{Fsr as RustFsr, FsrReading as RustFsrReading};
use crate::hardware::Resource;

/// Python-exposed FSR reading
#[pyclass(name = "FsrReading")]
#[derive(Clone)]
pub struct FsrReading {
    #[pyo3(get)]
    pub fsr_id: usize,
    #[pyo3(get)]
    pub channel: u8,
    #[pyo3(get)]
    pub value: u16,
    #[pyo3(get)]
    pub pressure_detected: bool,
}

impl From<RustFsrReading> for FsrReading {
    fn from(reading: RustFsrReading) -> Self {
        FsrReading {
            fsr_id: reading.fsr_id,
            channel: reading.channel,
            value: reading.value,
            pressure_detected: reading.pressure_detected,
        }
    }
}

/// Python-exposed FSR sensor interface
#[pyclass(name = "Fsr")]
pub struct Fsr {
    inner: RustFsr,
}

#[pymethods]
impl Fsr {
    /// Initialize the FSR sensor
    #[new]
    pub fn new() -> PyResult<Self> {
        let inner = RustFsr::init();
        Ok(Fsr { inner })
    }

    /// Configure FSR sensors
    ///
    /// Args:
    ///     cs_pins: List of CS pin numbers
    ///     at_rest_threshold: ADC value threshold for at-rest state
    ///     pressure_threshold: ADC value threshold for pressure detection
    pub fn configure(&mut self, cs_pins: Vec<u8>, at_rest_threshold: u16, pressure_threshold: u16) -> PyResult<()> {
        self.inner.configure(cs_pins, at_rest_threshold, pressure_threshold);
        Ok(())
    }

    /// Read all FSR sensors
    ///
    /// Returns:
    ///     List of FsrReading objects
    pub fn read_all(&mut self) -> PyResult<Vec<FsrReading>> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                self.inner
                    .read_all()
                    .map(|readings| readings.into_iter().map(|r| r.into()).collect())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("FSR read error: {}", e)))
            })
        })
    }

    /// Process FSR data to detect pressure
    ///
    /// Returns:
    ///     True if pressure detected on any sensor
    pub fn process_data(&mut self) -> PyResult<bool> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                self.inner
                    .process_data()
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("FSR process error: {}", e)))
            })
        })
    }
}
