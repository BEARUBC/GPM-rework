pub mod maestro;
pub mod emg;
pub mod fsr;
pub mod bms;
pub mod adc;

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
