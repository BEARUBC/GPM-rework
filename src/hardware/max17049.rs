//! MAX17048 / MAX17049 fuel gauge over I²C (register map: Analog MAX17048–MAX17049 datasheet).
use rppal::i2c::I2c;

/// Default 7-bit slave address (same for MAX17048/49 unless strapped differently).
pub const DEFAULT_SLAVE_ADDR: u16 = 0x36;

const REG_VCELL: u8 = 0x02;
const REG_SOC: u8 = 0x04;
const REG_VERSION: u8 = 0x08;

/// VCELL LSB is 78.125 µV (datasheet Table 2). For MAX17049, VCELL is pack voltage (CELL–GND).
const VCELL_LSB_V: f32 = 78.125e-6;

/// Read one 16-bit register (MSB first per datasheet).
pub fn read_reg16(i2c: &I2c, reg: u8) -> rppal::i2c::Result<u16> {
    let mut buf = [0u8; 2];
    i2c.write_read(&[reg], &mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub fn read_pack_voltage_v(i2c: &I2c) -> rppal::i2c::Result<f32> {
    let raw = read_reg16(i2c, REG_VCELL)?;
    Ok(raw as f32 * VCELL_LSB_V)
}

/// State of charge 0–100% (register LSb = 1/256 %).
pub fn read_soc_percent(i2c: &I2c) -> rppal::i2c::Result<f32> {
    let raw = read_reg16(i2c, REG_SOC)?;
    Ok((raw as f32 / 256.0).clamp(0.0, 100.0))
}

pub fn read_version(i2c: &I2c) -> rppal::i2c::Result<u16> {
    read_reg16(i2c, REG_VERSION)
}
