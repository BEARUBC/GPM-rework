// ADC (MCP3008) interface for reading analog sensors
use anyhow::{Context, Error, Result};

#[cfg(feature = "pi")]
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub struct Adc {
    #[cfg(feature = "pi")]
    pub spi: Spi,
    #[cfg(not(feature = "pi"))]
    _phantom: (),
}

impl Adc {
    #[cfg(feature = "pi")]
    pub fn init(pin: u8, clock_speed: u32) -> Self {
        let slave_select = match pin {
            8 => SlaveSelect::Ss0, // EMG ADC: SPI0 CE0
            7 => SlaveSelect::Ss1, // FSR ADC: SPI0 CE1
            _ => panic!("Unsupported SPI0 CS GPIO {pin}; expected 8 (CE0) or 7 (CE1)"),
        };

        let spi = Spi::new(Bus::Spi0, slave_select, clock_speed, Mode::Mode0)
            .expect("Failed to initialize SPI");

        Adc { spi }
    }

    #[cfg(not(feature = "pi"))]
    pub fn init(_pin: u8, _clock_speed: u32) -> Self {
        Adc { _phantom: () }
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

        self.spi
            .transfer(&mut rx, &tx)
            .context("SPI transfer failed during ADC read")?;

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
        Ok(rand::rng().random_range(400..600))
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
