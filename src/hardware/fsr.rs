use super::{adc::Adc, Resource};
use anyhow::Result;

pub struct Fsr {
    pub adc: Adc,
    pub at_rest_threshold: u16,
    pub pressure_threshold: u16,
    pub clock_speed: u32,
    pub num_fsrs: usize,
    pub cs_pins: [u8; 1],
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
    fn init() -> Result<Self> {
        anyhow::bail!("No CS pin configured for FSR ADC")
    }

    fn name() -> String {
        "Fsr".to_string()
    }
}

impl Fsr {
    pub fn init_with_cs(cs_pin: u8, spi_bus: u8) -> Result<Self> {
        let adc = Adc::init(cs_pin, 1350000, spi_bus)?;
        Ok(Fsr {
            adc,
            at_rest_threshold: 900,
            pressure_threshold: 500,
            clock_speed: 1350000,
            num_fsrs: 1,
            cs_pins: [cs_pin],
            num_channels: 8,
        })
    }

    pub fn configure(&mut self, at_rest: u16, pressure: u16) {
        self.at_rest_threshold = at_rest;
        self.pressure_threshold = pressure;
    }

    pub fn read_all(&mut self) -> Result<Vec<FsrReading>> {
        let mut readings = Vec::new();

        for (fsr_id, _) in self.cs_pins.iter().enumerate() {
            for channel in 0..self.num_channels {
                let value = self.adc.read_channel(channel)?;
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
