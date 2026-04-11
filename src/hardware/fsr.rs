use super::{adc::Adc, Resource};
use crate::hal::traits::{AdcDriver, FsrDriver};
use anyhow::Result;

pub struct Fsr {
    pub at_rest_threshold: u16,
    pub pressure_threshold: u16,
    pub clock_speed: u32,
    pub num_fsrs: usize,
    pub cs_pins: [u8; 3],
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
            cs_pins: [7, 8, 9], // Example CS pins
            num_channels: 8,
        }
    }

    fn name() -> String {
        "Fsr".to_string()
    }
}

impl Fsr {
    pub fn configure(&mut self, cs_pins: [u8; 3], at_rest: u16, pressure: u16) {
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

impl FsrDriver for Fsr {
    fn read_all(&mut self) -> Result<Vec<FsrReading>> {
        Fsr::read_all(self)
    }

    fn process_data(&mut self) -> Result<bool> {
        Fsr::process_data(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Threshold detection logic
    // #[test]
    // fn test_pressure_detected_just_below_at_rest_threshold() {
    //     let mut fsr = make_fsr_uniform(899); // one below threshold
    //     let readings = fsr.read_all().unwrap();
    //     assert!(readings.iter().all(|r| r.pressure_detected));
    // }

    // #[test]
    // fn test_pressure_detected_just_above_at_rest_threshold() {
    //     let mut fsr = make_fsr_uniform(901); // one above threshold
    //     let readings = fsr.read_all().unwrap();
    //     assert!(readings.iter().all(|r| r.pressure_detected));
    // }

    // #[test]
    // fn test_pressure_detected_at_max_value() {
    //     let mut fsr = make_fsr_uniform(1023); // max value
    //     let readings = fsr.read_all().unwrap();
    //     assert!(readings.iter().all(|r| r.pressure_detected));
    // }

    // #[test]
    // fn test_pressure_detected_at_hard_press_value() {
    //     // Prototype log: hard press produces ~100.
    //     let mut fsr = make_fsr_uniform(100);
    //     let readings = fsr.read_all().unwrap();
    //     assert!(readings.iter().all(|r| r.pressure_detected));
    // }

    // #[test]
    // fn test_pressure_detected_at_zero() {
    //     // Theoretical minimum ADC output.
    //     let mut fsr = make_fsr_uniform(0);
    //     let readings = fsr.read_all().unwrap();
    //     assert!(readings.iter().all(|r| r.pressure_detected));
    // }

    // TODO: Multiple sensor reading
    // #[test]
    // fn test_readings_span_both_adcs() {
    //     // Simulate ADC 0 (cs_pin 7) at rest, ADC 1 (cs_pin 8) pressed.
    //     // We use a stateful factory that alternates per call.
    //     let call_count = std::sync::Arc::new(std::sync::Mutex::new(0usize));
    //     let call_count_clone = call_count.clone();
    //
    //     let mut fsr: FsrWithDriver<MockAdc> = FsrWithDriver {
    //         at_rest_threshold: 900,
    //         pressure_threshold: 500,
    //         num_channels: 8,
    //         cs_pins: [7, 8, 9],
    //         num_fsrs: 3,
    //         adc_factory: Box::new(move |_cs_pin| {
    //             let mut count = call_count_clone.lock().unwrap();
    //             let adc = if *count == 0 {
    //                 MockAdc::uniform(990) // ADC 0 — at rest
    //             } else {
    //                 MockAdc::uniform(100) // ADC 1 & 2 — pressed
    //             };
    //             *count += 1;
    //             adc
    //         }),
    //     };
    //
    //     let readings = fsr.read_all().unwrap();
    //
    //     let adc0_readings: Vec<_> = readings.iter().filter(|r| r.fsr_id == 0).collect();
    //     let adc1_readings: Vec<_> = readings.iter().filter(|r| r.fsr_id == 1).collect();
    //
    //     assert!(
    //         adc0_readings.iter().all(|r| !r.pressure_detected),
    //         "ADC 0 should be at rest"
    //     );
    //     assert!(
    //         adc1_readings.iter().all(|r| r.pressure_detected),
    //         "ADC 1 should detect pressure"
    //     );
    // }

    // TODO: Pressure detection across all channels

    // #[test]
    // fn test_only_pressed_channels_flagged() {
    //     // Channel 0: below threshold (pressed), channels 1-7: at rest.
    //     let values = vec![100, 990, 990, 990, 990, 990, 990, 990];
    //     let mut fsr = make_fsr_per_channel(values);
    //     let readings = fsr.read_all().unwrap();
    //
    //     // Each ADC's channel 0 should be pressed.
    //     for r in &readings {
    //         if r.channel == 0 {
    //             assert!(r.pressure_detected, "Channel 0 should be pressed");
    //         } else {
    //             assert!(
    //                 !r.pressure_detected,
    //                 "Channel {} should be at rest",
    //                 r.channel
    //             );
    //         }
    //     }
    // }

    // #[test]
    // fn test_process_data_true_when_single_channel_pressed() {
    //     // Only channel 3 is pressed; process_data must still return true.
    //     let mut values = vec![990u16; 8];
    //     values[3] = 200;
    //     let mut fsr = make_fsr_per_channel(values);
    //     assert!(fsr.process_data().unwrap());
    // }

    // #[test]
    // fn test_process_data_false_at_exact_threshold_boundary() {
    //     // Value == threshold is not < threshold, so should be false.
    //     let mut fsr = make_fsr_uniform(900);
    //     assert!(!fsr.process_data().unwrap());
    // }

    // TODO: CS pin configuration
    // #[test]
    // fn test_num_fsrs_matches_cs_pin_count() {
    //     // num_fsrs must stay in sync with the number of CS pins;
    //     // a mismatch would cause read_all to iterate the wrong number of ADCs.
    //     let fsr = make_fsr_uniform(1023);
    //     assert_eq!(fsr.num_fsrs, fsr.cs_pins.len());
    // }

    // #[test]
    // fn test_configure_updates_cs_pins() {
    //     let mut fsr = make_fsr_uniform(1023);
    //     fsr.cs_pins = [1, 2, 3];
    //     fsr.num_fsrs = 3;
    //     assert_eq!(fsr.cs_pins, [1, 2, 3]);
    //     assert_eq!(fsr.num_fsrs, 3);
    // }
}
