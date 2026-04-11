pub mod maestro;
pub mod emg;
pub mod fsr;
pub mod bms;
pub mod adc;
#[cfg(feature = "pi")]
pub(crate) mod max17049;

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
