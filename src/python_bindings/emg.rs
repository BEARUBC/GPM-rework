use pyo3::prelude::*;

// TODO: Once gpm_original is added as dependency/submodule:
// use gpm_original::resources::Emg as RustEmg;
// use gpm_original::resources::Resource;

// Temporary: using local hardware module
use crate::hardware::emg::Emg as RustEmg;
use crate::hardware::Resource;

/// Python-exposed EMG sensor interface
#[pyclass(name = "Emg")]
pub struct Emg {
    inner: RustEmg,
}

#[pymethods]
impl Emg {
    /// Initialize the EMG sensor
    #[new]
    pub fn new() -> PyResult<Self> {
        let inner = RustEmg::init();
        Ok(Emg { inner })
    }

    /// Configure EMG buffer size
    ///
    /// Args:
    ///     buffer_size: Number of samples to buffer
    pub fn configure(&mut self, buffer_size: usize) -> PyResult<()> {
        self.inner.configure(buffer_size);
        Ok(())
    }

    /// Read buffer of EMG samples
    ///
    /// Returns:
    ///     List of ADC values from both channels
    pub fn read_buffer(&mut self) -> PyResult<Vec<u16>> {
        Python::with_gil(|py| {
            py.allow_threads(|| {
                self.inner
                    .read_buffer()
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("EMG read error: {}", e)))
            })
        })
    }

    /// Check if EMG is ready to read
    ///
    /// Returns:
    ///     True if ready
    pub fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    /// Get latest samples without reading new data
    ///
    /// Returns:
    ///     List of most recent samples
    pub fn get_latest_samples(&self) -> PyResult<Vec<u16>> {
        Ok(self.inner.get_latest_samples())
    }

    /// Calibrate EMG thresholds
    ///
    /// Args:
    ///     inner_threshold: Threshold for inner channel
    ///     outer_threshold: Threshold for outer channel
    pub fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32) -> PyResult<()> {
        self.inner.calibrate(inner_threshold, outer_threshold);
        Ok(())
    }

    /// Process EMG values to detect gesture
    ///
    /// Args:
    ///     values: List of two float values [channel_0, channel_1]
    ///
    /// Returns:
    ///     -1 (hold), 0 (close), or 1 (open)
    pub fn process_data(&self, values: Vec<f32>) -> PyResult<i32> {
        self.inner
            .process_data(&values)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Process error: {}", e)))
    }
}
