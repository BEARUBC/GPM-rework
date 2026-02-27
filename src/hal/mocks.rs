use crate::hal::traits::{AdcDriver, BmsDriver, EmgDriver, FsrDriver, MaestroDriver};
use crate::hardware::bms::BmsStatus;
use crate::hardware::fsr::FsrReading;
use anyhow::Result;

pub struct MockAdcDriver {
    pub channel_values: Vec<u16>,
}

impl MockAdcDriver {
    pub fn new(channel_values: Vec<u16>) -> Self {
        Self { channel_values }
    }
}

impl AdcDriver for MockAdcDriver {
    fn read_channel(&mut self, channel: u8) -> Result<u16> {
        self.channel_values
            .get(channel as usize)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("MockAdcDriver: channel {} out of range", channel))
    }
}

pub struct MockBmsDriver {
    pub status: BmsStatus,
    pub update_count: u32,
}

impl MockBmsDriver {
    pub fn new(status: BmsStatus) -> Self {
        Self {
            status,
            update_count: 0,
        }
    }
}

impl BmsDriver for MockBmsDriver {
    fn get_status(&self) -> BmsStatus {
        self.status.clone()
    }

    fn update(&mut self) {
        self.update_count += 1;
    }
}

pub struct MockEmgDriver {
    pub buffer: Vec<u16>,
    pub ready: bool,
    pub inner_threshold: f32,
    pub outer_threshold: f32,
}

impl MockEmgDriver {
    pub fn new(buffer: Vec<u16>) -> Self {
        Self {
            buffer,
            ready: true,
            inner_threshold: 450.0,
            outer_threshold: 450.0,
        }
    }
}

impl EmgDriver for MockEmgDriver {
    fn read_buffer(&mut self) -> Result<Vec<u16>> {
        Ok(self.buffer.clone())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn get_latest_samples(&self) -> Vec<u16> {
        self.buffer.clone()
    }

    fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32) {
        self.inner_threshold = inner_threshold;
        self.outer_threshold = outer_threshold;
    }

    fn process_data(&self, values: &[f32]) -> Result<i32> {
        if values.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2 EMG values"));
        }
        if values[0] >= self.inner_threshold && values[1] <= self.outer_threshold {
            Ok(1) // Open
        } else if values[0] <= self.inner_threshold && values[1] >= self.outer_threshold {
            Ok(0) // Close
        } else {
            Ok(-1) // Hold/No change
        }
    }
}

pub struct MockFsrDriver {
    pub readings: Vec<FsrReading>,
    pub pressure_detected: bool,
}

impl MockFsrDriver {
    pub fn new(readings: Vec<FsrReading>, pressure_detected: bool) -> Self {
        Self {
            readings,
            pressure_detected,
        }
    }
}

impl FsrDriver for MockFsrDriver {
    fn read_all(&mut self) -> Result<Vec<FsrReading>> {
        Ok(self.readings.clone())
    }

    fn process_data(&mut self) -> Result<bool> {
        Ok(self.pressure_detected)
    }
}

pub struct MockMaestroDriver {
    pub pwm_values: [u16; 6],
    pub last_grip: Option<String>,
}

impl MockMaestroDriver {
    pub fn new() -> Self {
        Self {
            pwm_values: [1500; 6],
            last_grip: None,
        }
    }
}

impl MaestroDriver for MockMaestroDriver {
    fn set_target(&mut self, channel: u8, pwm_value: u16) -> Result<()> {
        if channel > 5 {
            return Err(anyhow::anyhow!("Invalid channel: {}", channel));
        }
        self.pwm_values[channel as usize] = pwm_value;
        Ok(())
    }

    fn current_pwm(&self, channel: u8) -> Result<u16> {
        if channel > 5 {
            return Err(anyhow::anyhow!("Invalid channel: {}", channel));
        }
        Ok(self.pwm_values[channel as usize])
    }

    fn move_to_grip(&mut self, grip_type: &str) -> Result<()> {
        self.last_grip = Some(grip_type.to_string());
        match grip_type {
            "rest" | "pinch" | "power" | "open" => Ok(()),
            _ => Err(anyhow::anyhow!("Unknown grip type: {}", grip_type)),
        }
    }
}
