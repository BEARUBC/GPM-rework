// ADC (MCP3008) interface for reading analog sensors
use anyhow::{Context, Error, Result};

#[cfg(feature = "pi")]
use rppal::gpio::{Gpio, OutputPin};
#[cfg(feature = "pi")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub struct Adc {
    #[cfg(feature = "pi")]
    pub spi: Spi,
    #[cfg(feature = "pi")]
    pub cs_pin: OutputPin,
    #[cfg(not(feature = "pi"))]
    _phantom: (),
    /// Currently active CS pin number (for multi-slave tracking)
    pub active_cs_pin: u8,
}

impl Adc {
    #[cfg(feature = "pi")]
    pub fn init(pin: u8, clock_speed: u32) -> Self {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, clock_speed, Mode::Mode0)
            .expect("Failed to initialize SPI");

        let mut cs = Gpio::new()
            .expect("Failed to initialize manual CS")
            .get(pin)
            .expect("Failed to get GPIO pin for CS")
            .into_output();

        cs.set_high();

        Adc { spi, cs_pin: cs, active_cs_pin: pin }
    }

    #[cfg(not(feature = "pi"))]
    pub fn init(_pin: u8, _clock_speed: u32) -> Self {
        Adc { _phantom: (), active_cs_pin: _pin }
    }

    // -------------------------------------------------------------------------
    // Multi-slave chip-select orchestration
    // -------------------------------------------------------------------------

    /// Switch the active chip-select to a different slave.
    ///
    /// On real hardware this reconfigures the GPIO CS pin; in simulation it
    /// just records which slave is logically selected so tests can assert on it.
    ///
    /// SPI protocol: CS is active-LOW. Pulling the old pin HIGH de-selects the
    /// current slave before pulling the new pin LOW to select the next one.
    ///
    /// # Arguments
    /// * pin - GPIO pin number of the slave to select (0–27 on RPi)
    #[cfg(feature = "pi")]
    pub fn set_chip_select(&mut self, pin: u8) -> Result<()> {
        // De-select current slave
        self.cs_pin.set_high();

        // Re-acquire GPIO for the new CS pin
        let mut new_cs = Gpio::new()
            .context("Failed to init GPIO for CS switch")?
            .get(pin)
            .with_context(|| format!("Failed to get GPIO pin {} for CS", pin))?
            .into_output();

        new_cs.set_high(); // start de-selected; read_channel will pull low
        self.cs_pin = new_cs;
        self.active_cs_pin = pin;
        Ok(())
    }

    #[cfg(not(feature = "pi"))]
    pub fn set_chip_select(&mut self, pin: u8) -> Result<()> {
        // Simulation: just record which slave is active
        self.active_cs_pin = pin;
        Ok(())
    }

    #[cfg(feature = "pi")]
    pub fn read_channel(&mut self, channel: u8) -> Result<u16> {
        if channel > 7 {
            return Err(Error::msg(format!(
                "Invalid ADC channel: {}. Must be between 0 and 7.",
                channel
            )));
        }

        let start_bit = 0b00000001;
        let config_bits = 0b10000000 | (channel << 4);
        let tx = [start_bit, config_bits, 0x00];
        let mut rx = [0u8; 3];

        self.cs_pin.set_low();
        self.spi
            .transfer(&mut rx, &tx)
            .context("SPI transfer failed during ADC read")?;
        self.cs_pin.set_high();

        let result = ((rx[1] & 0b00000011) as u16) << 8 | (rx[2] as u16);
        Ok(result)
    }

    #[cfg(not(feature = "pi"))]
    pub fn read_channel(&mut self, channel: u8) -> Result<u16> {
        if channel > 7 {
            return Err(Error::msg(format!(
                "Invalid ADC channel: {}. Must be between 0 and 7.",
                channel
            )));
        }
        use rand::Rng;
        Ok(rand::thread_rng().gen_range(400..600))
    }

    pub fn read_channels(&mut self, channels: &[u8]) -> Result<Vec<u16>> {
        channels
            .iter()
            .map(|&channel| {
                self.read_channel(channel)
                    .with_context(|| format!("Failed to read from ADC channel {}", channel))
            })
            .collect()
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn make_adc() -> Adc {
        Adc::init(8, 1_350_000)
    }

    // -------------------------------------------------------------------------
    // set_chip_select
    // -------------------------------------------------------------------------

    #[test]
    fn test_set_chip_select_updates_active_pin() {
        let mut adc = make_adc();
        adc.set_chip_select(7).unwrap();
        assert_eq!(adc.active_cs_pin, 7);
    }

    #[test]
    fn test_set_chip_select_can_switch_multiple_times() {
        let mut adc = make_adc();
        // Simulate 3 slaves on pins 8, 7, 25
        for &pin in &[8u8, 7, 25] {
            adc.set_chip_select(pin).unwrap();
            assert_eq!(adc.active_cs_pin, pin);
        }
    }

    #[test]
    fn test_set_chip_select_returns_ok() {
        let mut adc = make_adc();
        assert!(adc.set_chip_select(7).is_ok());
    }

    #[test]
    fn test_read_channel_after_cs_switch_still_works() {
        let mut adc = make_adc();
        adc.set_chip_select(7).unwrap();
        // Should still read without panicking
        assert!(adc.read_channel(0).is_ok());
    }

    // -------------------------------------------------------------------------
    // Existing channel validation (unchanged behaviour)
    // -------------------------------------------------------------------------

    #[test]
    fn test_read_channel_invalid_returns_err() {
        let mut adc = make_adc();
        assert!(adc.read_channel(8).is_err());
    }

    #[test]
    fn test_read_channel_valid_range() {
        let mut adc = make_adc();
        for ch in 0..=7 {
            let val = adc.read_channel(ch).unwrap();
            assert!(val >= 400 && val < 600, "ch{} out of sim range: {}", ch, val);
        }
    }
}