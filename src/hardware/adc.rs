// ADC (MCP3008) interface for reading analog sensors
use crate::hal::traits::AdcDriver;
use anyhow::{Context, Error, Result};

#[cfg(feature = "pi")]
use rppal::gpio::{Gpio, OutputPin};
#[cfg(feature = "pi")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

/// Extracts the 10-bit ADC result from a 3-byte MCP3008 SPI response.
/// Bits 9–8 are the lower 2 bits of rx[1]; bits 7–0 are rx[2].
pub(crate) fn parse_adc_response(rx: [u8; 3]) -> u16 {
    ((rx[1] & 0b00000011) as u16) << 8 | rx[2] as u16
}

#[cfg(feature = "pi")]
pub(crate) trait SpiTransfer {
    fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> anyhow::Result<()>;
}

#[cfg(feature = "pi")]
impl SpiTransfer for Spi {
    fn transfer(&mut self, rx: &mut [u8], tx: &[u8]) -> anyhow::Result<()> {
        Spi::transfer(self, rx, tx)
            .context("SPI transfer failed during ADC read")
            .map(|_| ())
    }
}

#[cfg(feature = "pi")]
pub struct Adc<S: SpiTransfer = Spi> {
    pub spi: S,
    pub cs_pin: Option<OutputPin>,
}

#[cfg(not(feature = "pi"))]
pub struct Adc {
    _phantom: (),
}

#[cfg(feature = "pi")]
impl Adc<Spi> {
    pub fn init(pin: u8, clock_speed: u32) -> Self {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, clock_speed, Mode::Mode0)
            .expect("Failed to initialize SPI");

        let mut cs = Gpio::new()
            .expect("Failed to initialize manual CS")
            .get(pin)
            .expect("Failed to get GPIO pin for CS")
            .into_output();

        cs.set_high();

        Adc {
            spi,
            cs_pin: Some(cs),
        }
    }
}

#[cfg(not(feature = "pi"))]
impl Adc {
    pub fn init(_pin: u8, _clock_speed: u32) -> Self {
        Adc { _phantom: () }
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

#[cfg(feature = "pi")]
impl<S: SpiTransfer> Adc<S> {
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

#[cfg(not(feature = "pi"))]
impl AdcDriver for Adc {
    fn read_channel(&mut self, channel: u8) -> Result<u16> {
        if channel > 7 {
            return Err(Error::msg(format!(
                "Invalid ADC channel: {}. Must be between 0 and 7.",
                channel
            )));
        }
        use rand::Rng;
        Ok(rand::rng().random_range(400..600))
    }
}

#[cfg(feature = "pi")]
impl<S: SpiTransfer> AdcDriver for Adc<S> {
    fn read_channel(&mut self, channel: u8) -> Result<u16> {
        if channel > 7 {
            return Err(Error::msg(format!(
                "Invalid ADC channel: {}. Must be between 0 and 7.",
                channel
            )));
        }

        let start_bit = 0b00000001u8;
        let config_bits = 0b10000000u8 | (channel << 4);
        let tx = [start_bit, config_bits, 0x00u8];
        let mut rx = [0u8; 3];

        if let Some(ref mut cs) = self.cs_pin {
            cs.set_low();
        }
        self.spi.transfer(&mut rx, &tx)?;
        if let Some(ref mut cs) = self.cs_pin {
            cs.set_high();
        }

        Ok(parse_adc_response(rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_adc_response tests (always compiled) ---

    #[test]
    fn parse_response_zero() {
        assert_eq!(parse_adc_response([0x00, 0x00, 0x00]), 0);
    }

    #[test]
    fn parse_response_max() {
        assert_eq!(parse_adc_response([0x00, 0x03, 0xFF]), 1023);
    }

    #[test]
    fn parse_response_low_byte_only() {
        assert_eq!(parse_adc_response([0x00, 0x00, 0xAB]), 0x00AB);
    }

    #[test]
    fn parse_response_high_bits_only() {
        assert_eq!(parse_adc_response([0x00, 0x03, 0x00]), 0x0300);
    }

    #[test]
    fn parse_response_high_bits_masked() {
        // Upper bits of rx[1] beyond the lower 2 are discarded
        assert_eq!(parse_adc_response([0x00, 0xFF, 0x00]), 0x0300);
    }

    // --- Channel validation tests (both builds) ---

    #[test]
    fn init_succeeds() {
        let _adc = Adc::init(0, 1_000_000);
    }

    #[test]
    fn read_valid_channels() {
        let mut adc = Adc::init(0, 1_000_000);
        for ch in 0..=7u8 {
            let result = adc.read_channel(ch);
            assert!(result.is_ok(), "channel {ch} should be valid");
        }
    }

    #[test]
    fn read_channel_7_boundary() {
        let mut adc = Adc::init(0, 1_000_000);
        assert!(adc.read_channel(7).is_ok(), "channel 7 should be valid");
    }

    #[test]
    fn read_channel_8_boundary() {
        let mut adc = Adc::init(0, 1_000_000);
        assert!(adc.read_channel(8).is_err(), "channel 8 should be invalid");
    }

    #[test]
    fn read_channel_invalid_error_message() {
        let mut adc = Adc::init(0, 1_000_000);
        let err = adc.read_channel(42).unwrap_err();
        assert!(
            err.to_string().contains("42"),
            "error message should contain the bad channel number"
        );
    }

    #[test]
    fn read_channel_value_in_adc_range() {
        let mut adc = Adc::init(0, 1_000_000);
        let val = adc.read_channel(0).unwrap();
        assert!(val <= 1023, "ADC value {val} exceeds 10-bit max");
    }

    // --- read_channels length tests (both builds) ---

    #[test]
    fn read_channels_empty_slice() {
        let mut adc = Adc::init(0, 1_000_000);
        let result = adc.read_channels(&[]).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn read_channels_single() {
        let mut adc = Adc::init(0, 1_000_000);
        let result = adc.read_channels(&[0]).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn read_channels_three() {
        let mut adc = Adc::init(0, 1_000_000);
        let result = adc.read_channels(&[0, 1, 2]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn read_channels_all_eight() {
        let mut adc = Adc::init(0, 1_000_000);
        let result = adc.read_channels(&[0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn read_channels_repeated() {
        let mut adc = Adc::init(0, 1_000_000);
        let result = adc.read_channels(&[0, 0, 0]).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn read_channels_propagates_error() {
        let mut adc = Adc::init(0, 1_000_000);
        assert!(adc.read_channels(&[0, 8]).is_err());
    }

    // --- SPI mock tests (pi only) ---

    #[cfg(feature = "pi")]
    struct MockSpi {
        response: [u8; 3],
        should_fail: bool,
    }

    #[cfg(feature = "pi")]
    impl SpiTransfer for MockSpi {
        fn transfer(&mut self, rx: &mut [u8], _tx: &[u8]) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::Error::msg("mock SPI failure"));
            }
            rx.copy_from_slice(&self.response);
            Ok(())
        }
    }

    #[cfg(feature = "pi")]
    fn make_mock_adc(mock_spi: MockSpi) -> Adc<MockSpi> {
        Adc {
            spi: mock_spi,
            cs_pin: None,
        }
    }

    #[cfg(feature = "pi")]
    #[test]
    fn spi_error_propagates() {
        let mut adc = make_mock_adc(MockSpi {
            response: [0; 3],
            should_fail: true,
        });
        assert!(adc.read_channel(0).is_err());
    }

    #[cfg(feature = "pi")]
    #[test]
    fn spi_mock_parses_response() {
        let mut adc = make_mock_adc(MockSpi {
            response: [0x00, 0x03, 0xFF],
            should_fail: false,
        });
        assert_eq!(adc.read_channel(0).unwrap(), 1023);
    }

    #[cfg(feature = "pi")]
    #[test]
    fn spi_error_propagates_through_channels() {
        let mut adc = make_mock_adc(MockSpi {
            response: [0; 3],
            should_fail: true,
        });
        assert!(adc.read_channels(&[0, 1, 2]).is_err());
    }
}
