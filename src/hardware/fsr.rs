use anyhow::Result;
use super::{Resource, adc::Adc};

pub struct Fsr {
    pub at_rest_threshold: u16,
    pub pressure_threshold: u16,
    pub clock_speed: u32,
    pub num_fsrs: usize,
    pub cs_pins: Vec<u8>,
    pub num_channels: u8,
}

#[derive(Clone, Debug)]
pub struct FsrReading {
    pub fsr_id: usize,
    pub channel: u8,
    pub value: u16,
    pub pressure_detected: bool,
}

impl Resource for Fsr {
    fn init() -> Self {
        Fsr {
            at_rest_threshold: 900,
            pressure_threshold: 500,
            clock_speed: 1350000,
            num_fsrs: 1,
            cs_pins: vec![7], // Example CS pin
            num_channels: 8,
        }
    }

    fn name() -> String {
        "Fsr".to_string()
    }
}

impl Fsr {
    pub fn configure(&mut self, cs_pins: Vec<u8>, at_rest: u16, pressure: u16) {
        self.cs_pins = cs_pins;
        self.num_fsrs = cs_pins.len();
        self.at_rest_threshold = at_rest;
        self.pressure_threshold = pressure;
    }

    pub fn read_all(&mut self) -> Result<Vec<FsrReading>> {
        let mut readings = Vec::new();

        for (fsr_id, &cs_pin) in self.cs_pins.iter().enumerate() {
            let mut adc = Adc::init(cs_pin, self.clock_speed);
            
            for channel in 0..self.num_channels {
                let value = adc.read_channel(channel)?;
                let pressure_detected = value < self.at_rest_threshold;
                
                readings.push(FsrReading {
                    fsr_id,
                    channel,
                    value,
                    pressure_detected,
                });
            }
        }

        Ok(readings)
    }

    pub fn process_data(&mut self) -> Result<bool> {
        let readings = self.read_all()?;
        
        // Return true if any sensor detects pressure
        Ok(readings.iter().any(|r| r.pressure_detected))
    }
}
