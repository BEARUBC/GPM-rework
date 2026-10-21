pub mod maestro;
pub mod emg;
pub mod fsr;
pub mod bms;
pub mod adc;

pub trait Resource {
    fn init() -> anyhow::Result<Self> // Returns device on success or error on init failure
    where
        Self: Sized;
    fn name() -> String;
}
