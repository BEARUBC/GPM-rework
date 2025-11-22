use anyhow::Result;
use super::Resource;

#[cfg(feature = "pi")]
use raestro::maestro::{
    builder::Builder,
    constants::{Baudrate, Channel},
};
#[cfg(feature = "pi")]
use std::time::Duration;

pub struct Maestro {
    #[cfg(feature = "pi")]
    pub controller: raestro::maestro::Maestro,
    #[cfg(not(feature = "pi"))]
    pwm_values: [u16; 6],
}

impl Resource for Maestro {
    #[cfg(feature = "pi")]
    fn init() -> Self {
        let controller: raestro::maestro::Maestro = Builder::default()
            .baudrate(Baudrate::Baudrate11520)
            .block_duration(Duration::from_millis(100))
            .try_into()
            .expect("Could not initialize Raestro");
        Maestro { controller }
    }

    #[cfg(not(feature = "pi"))]
    fn init() -> Self {
        Maestro {
            pwm_values: [1500; 6],
        }
    }

    fn name() -> String {
        "Maestro".to_string()
    }
}

impl Maestro {
    #[cfg(feature = "pi")]
    pub fn set_target(&mut self, channel: u8, pwm_value: u16) -> Result<()> {
        let ch = match channel {
            0 => Channel::Channel0,
            1 => Channel::Channel1,
            2 => Channel::Channel2,
            3 => Channel::Channel3,
            4 => Channel::Channel4,
            5 => Channel::Channel5,
            _ => return Err(anyhow::anyhow!("Invalid channel: {}", channel)),
        };
        self.controller.set_target(ch, pwm_value)?;
        Ok(())
    }

    #[cfg(not(feature = "pi"))]
    pub fn set_target(&mut self, channel: u8, pwm_value: u16) -> Result<()> {
        if channel > 5 {
            return Err(anyhow::anyhow!("Invalid channel: {}", channel));
        }
        self.pwm_values[channel as usize] = pwm_value;
        Ok(())
    }

    #[cfg(feature = "pi")]
    pub fn current_pwm(&self, channel: u8) -> Result<u16> {
        let ch = match channel {
            0 => Channel::Channel0,
            1 => Channel::Channel1,
            2 => Channel::Channel2,
            3 => Channel::Channel3,
            4 => Channel::Channel4,
            5 => Channel::Channel5,
            _ => return Err(anyhow::anyhow!("Invalid channel: {}", channel)),
        };
        Ok(self.controller.get_position(ch)?)
    }

    #[cfg(not(feature = "pi"))]
    pub fn current_pwm(&self, channel: u8) -> Result<u16> {
        if channel > 5 {
            return Err(anyhow::anyhow!("Invalid channel: {}", channel));
        }
        Ok(self.pwm_values[channel as usize])
    }

    pub fn move_to_grip(&mut self, grip_type: &str) -> Result<()> {
        // Define grip positions (PWM values for each servo)
        // These are example values - should be calibrated for actual hardware
        match grip_type {
            "rest" => {
                self.set_target(0, 1500)?;
                self.set_target(1, 1500)?;
                self.set_target(2, 1500)?;
            }
            "pinch" => {
                self.set_target(0, 2000)?;
                self.set_target(1, 1800)?;
                self.set_target(2, 1500)?;
            }
            "power" => {
                self.set_target(0, 2200)?;
                self.set_target(1, 2200)?;
                self.set_target(2, 2200)?;
            }
            "open" => {
                self.set_target(0, 1000)?;
                self.set_target(1, 1000)?;
                self.set_target(2, 1000)?;
            }
            _ => return Err(anyhow::anyhow!("Unknown grip type: {}", grip_type)),
        }
        Ok(())
    }
}
