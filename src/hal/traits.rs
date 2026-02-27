use crate::hardware::bms::BmsStatus;
use crate::hardware::fsr::FsrReading;
use anyhow::Result;

pub trait AdcDriver {
    fn read_channel(&mut self, channel: u8) -> Result<u16>;
}

pub trait BmsDriver {
    fn get_status(&self) -> BmsStatus;
    fn update(&mut self);
}

pub trait EmgDriver {
    fn read_buffer(&mut self) -> Result<Vec<u16>>;
    fn is_ready(&self) -> bool;
    fn get_latest_samples(&self) -> Vec<u16>;
    fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32);
    fn process_data(&self, values: &[f32]) -> Result<i32>;
}

pub trait FsrDriver {
    fn read_all(&mut self) -> Result<Vec<FsrReading>>;
    fn process_data(&mut self) -> Result<bool>;
}

pub trait MaestroDriver {
    fn set_target(&mut self, channel: u8, pwm_value: u16) -> Result<()>;
    fn current_pwm(&self, channel: u8) -> Result<u16>;
    fn move_to_grip(&mut self, grip_type: &str) -> Result<()>;
}
