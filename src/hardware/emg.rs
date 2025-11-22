use anyhow::Result;
use super::{Resource, adc::Adc};

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
        let mut samples = Vec::new();
        
        // Read from both channels alternately
        for _ in 0..self.buffer_size / 2 {
            let ch0 = self.adc.read_channel(0)?;
            let ch1 = self.adc.read_channel(1)?;
            
            self.current_channel_0 = ch0 as f32;
            self.current_channel_1 = ch1 as f32;
            
            samples.push(ch0);
            samples.push(ch1);
        }
        
        self.buffer = samples.clone();
        Ok(samples)
    }

    pub fn is_ready(&self) -> bool {
        true // Always ready in this implementation
    }

    pub fn get_latest_samples(&self) -> Vec<u16> {
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
