use super::{adc::Adc, Resource};
use crate::hal::traits::{AdcDriver, EmgDriver};
use anyhow::Result;

pub struct Emg {
    pub adc: Adc,
    pub buffer: Vec<u16>,
    pub buffer_size: usize,
    pub inner_threshold: f32,
    pub outer_threshold: f32,
    pub current_channel_0: f32,
    pub current_channel_1: f32,
}

impl Resource for Emg {
    fn init() -> Self {
        let adc = Adc::init(8, 1350000); // CS pin 8, 1.35 MHz clock
        Emg {
            adc,
            buffer: Vec::new(),
            buffer_size: 256,
            inner_threshold: 450.0,
            outer_threshold: 450.0,
            current_channel_0: 0.0,
            current_channel_1: 0.0,
        }
    }

    fn name() -> String {
        "Emg".to_string()
    }
}

impl Emg {
    pub fn configure(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        self.buffer = Vec::with_capacity(buffer_size);
    }

    pub fn read_buffer(&mut self) -> Result<Vec<u16>> {
        self.buffer.clear();
        self.buffer
            .reserve(self.buffer_size.saturating_sub(self.buffer.capacity()));

        // Read from both channels alternately
        for _ in 0..self.buffer_size / 2 {
            let ch0 = self.adc.read_channel(0)?;
            let ch1 = self.adc.read_channel(1)?;

            self.current_channel_0 = ch0 as f32;
            self.current_channel_1 = ch1 as f32;

            self.buffer.push(ch0);
            self.buffer.push(ch1);
        }

        Ok(self.buffer.clone())
    }

    pub fn is_ready(&self) -> bool {
        true // Always ready in this implementation
    }

    pub fn get_latest_samples(&self) -> Vec<u16> {
        // Clone required: caller (PyO3) needs owned data to convert to a Python list.
        self.buffer.clone()
    }

    pub fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32) {
        self.inner_threshold = inner_threshold;
        self.outer_threshold = outer_threshold;
    }

    pub fn process_data(&self, values: &[f32]) -> Result<i32> {
        if values.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2 EMG values"));
        }

        // Simple threshold-based classification
        // 0 = close, 1 = open
        if values[0] >= self.inner_threshold && values[1] <= self.outer_threshold {
            Ok(1) // Open
        } else if values[0] <= self.inner_threshold && values[1] >= self.outer_threshold {
            Ok(0) // Close
        } else {
            Ok(-1) // Hold/No change
        }
    }
}

impl EmgDriver for Emg {
    fn read_buffer(&mut self) -> Result<Vec<u16>> {
        Emg::read_buffer(self)
    }

    fn is_ready(&self) -> bool {
        Emg::is_ready(self)
    }

    fn get_latest_samples(&self) -> Vec<u16> {
        Emg::get_latest_samples(self)
    }

    fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32) {
        Emg::calibrate(self, inner_threshold, outer_threshold)
    }

    fn process_data(&self, values: &[f32]) -> Result<i32> {
        Emg::process_data(self, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_size_configuration() {
        let mut emg = Emg::init();
        emg.buffer_size = 10;
        assert_eq!(emg.buffer_size, 10);

        emg.buffer_size = 20;
        assert_eq!(emg.buffer_size, 20);
    }

    #[test]
    fn test_threshold_calibration() {
        let mut emg = Emg::init();
        emg.calibrate(0.5, 1.5);
        assert_eq!(emg.inner_threshold, 0.5);
        assert_eq!(emg.outer_threshold, 1.5);
    }

    #[test]
    fn test_process_data_classification() {
        let mut emg = Emg::init();
        emg.calibrate(0.5, 1.5);
        assert_eq!(emg.process_data(&[0.6, 1.4]).unwrap(), 1); // Open
        assert_eq!(emg.process_data(&[0.4, 1.6]).unwrap(), 0); // Close
        assert_eq!(emg.process_data(&[0.5, 1.5]).unwrap(), -1); // Hold/No change
    }

    #[test]
    fn test_invalid_input_handling() {
        let mut emg = Emg::init();
        emg.calibrate(0.5, 1.5);
        assert!(emg.process_data(&[0.6]).is_err());
        assert!(emg.process_data(&[0.6, 1.4, 2.0]).is_err());
    }

    // TODO: Channel reading alternation
    // notes: what are the different channel values?
    #[test]
    fn test_channel_reading_alternation() {
        let mut emg = Emg::init();
        emg.calibrate(0.5, 1.5);
        assert_eq!(emg.process_data(&[0.6, 1.4]).unwrap(), 1); // Open
        assert_eq!(emg.process_data(&[0.4, 1.6]).unwrap(), 0); // Close
        assert_eq!(emg.process_data(&[0.5, 1.5]).unwrap(), -1); // Hold/No change
    }
}
