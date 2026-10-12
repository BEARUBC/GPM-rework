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
    pub fn init(pin: u8, clock_speed: u32, spi_bus: u8) -> Result<Self> {
        let (slave_select, bus) = match (pin, spi_bus) {
            (8, 0) => (SlaveSelect::Ss0, Bus::Spi0), // EMG ADC: SPI0 CS0
            (7, 0) => (SlaveSelect::Ss1, Bus::Spi0), // FSR ADC: SPI0 CS1
            _ => anyhow::bail!("Unsupported SPI-CS configuration: SPI bus {spi_bus}, CS on GPIO {pin}"),
        };

        let spi = Spi::new(bus, slave_select, clock_speed, Mode::Mode0)
            .with_context(|| format!("Failed to initialize SPI{spi_bus} on CS GPIO {pin}"))?;

        Ok(Adc { spi })
    }

    #[cfg(not(feature = "pi"))]
    pub fn init(pin: u8, _clock_speed: u32, spi_bus: u8) -> Result<Self> {
        if !matches!((pin, spi_bus), (8, 0) | (7, 0)) {
            anyhow::bail!("Unsupported SPI-CS configuration: SPI bus {spi_bus}, CS on GPIO {pin}");
        }
        Ok(Adc { _phantom: () })
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
