use super::Resource;
use crate::hal::traits::BmsDriver;

const MIN_CELL_VOLTAGE: f32 = 3.6;
const MAX_CELL_VOLTAGE: f32 = 4.2;
const MIN_PACK_VOLTAGE: f32 = MIN_CELL_VOLTAGE * 2.0;
const MAX_PACK_VOLTAGE: f32 = MAX_CELL_VOLTAGE * 2.0;

pub struct Bms {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
    pub charge_percentage: f32,
    pub cell1_voltage: f32,
    pub cell2_voltage: f32,
}

#[derive(Clone, Debug)]
pub struct BmsStatus {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
    pub charge_percentage: f32,
}

impl Resource for Bms {
    fn init() -> Self {
        let voltage = 7.8;
        let half = voltage * 0.5;
        let mut bms = Bms {
            voltage,
            current: 0.0,
            temperature: 25.0,
            is_healthy: true,
            charge_percentage: 0.0,
            cell1_voltage: half,
            cell2_voltage: half,
        };
        bms.charge_percentage = bms.calculate_charge_percentage();
        bms.recompute_health();
        bms
    }

    fn name() -> String {
        "Bms".to_string()
    }
}

impl Bms {
    pub fn get_status(&self) -> BmsStatus {
        BmsStatus {
            voltage: self.voltage,
            current: self.current,
            temperature: self.temperature,
            is_healthy: self.is_healthy,
            charge_percentage: self.charge_percentage,
        }
    }

    pub fn update(&mut self) {
        #[cfg(feature = "pi")]
        {
            self.update_from_max17049();
            return;
        }

        #[cfg(not(feature = "pi"))]
        {
            use rand::Rng;
            let mut rng = rand::rng();
            self.voltage = 7.2 + rng.random::<f32>() * 1.2; // ~2S pack span
            self.current = rng.random::<f32>() * 2.0;
            self.temperature = 20.0 + rng.random::<f32>() * 10.0;
            self.cell1_voltage = self.voltage * 0.5;
            self.cell2_voltage = self.voltage * 0.5;
            self.charge_percentage = self.calculate_charge_percentage();
            self.recompute_health();
        }
    }

    #[cfg(feature = "pi")]
    fn update_from_max17049(&mut self) {
        use rppal::i2c::I2c;

        let bus = std::env::var("GPM_BMS_I2C_BUS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_u8);

        let addr = std::env::var("GPM_MAX17049_ADDR")
            .ok()
            .and_then(|s| parse_i2c_addr(&s))
            .unwrap_or(crate::hardware::max17049::DEFAULT_SLAVE_ADDR);

        let result: Result<(f32, f32), rppal::i2c::Error> = (|| {
            let mut i2c = I2c::with_bus(bus)?;
            i2c.set_slave_address(addr)?;
            let v = crate::hardware::max17049::read_pack_voltage_v(&i2c)?;
            let soc = crate::hardware::max17049::read_soc_percent(&i2c)?;
            Ok((v, soc))
        })();

        match result {
            Ok((v, soc)) => {
                self.voltage = v;
                self.charge_percentage = soc;
                self.cell1_voltage = v * 0.5;
                self.cell2_voltage = v * 0.5;
                self.current = 0.0;
                self.recompute_health();
            }
            Err(e) => {
                log::warn!("MAX17049 I2C read failed (bus {bus}, addr 0x{addr:02x}): {e}");
                self.is_healthy = false;
            }
        }
    }

    fn recompute_health(&mut self) {
        let pack_ok =
            self.voltage >= MIN_PACK_VOLTAGE && self.voltage <= MAX_PACK_VOLTAGE;
        let cells_ok = (MIN_CELL_VOLTAGE..=MAX_CELL_VOLTAGE).contains(&self.cell1_voltage)
            && (MIN_CELL_VOLTAGE..=MAX_CELL_VOLTAGE).contains(&self.cell2_voltage);
        self.is_healthy = pack_ok && cells_ok;
    }

    fn calculate_charge_percentage(&self) -> f32 {
        ((self.voltage - MIN_PACK_VOLTAGE) / (MAX_PACK_VOLTAGE - MIN_PACK_VOLTAGE) * 100.0)
            .max(0.0)
            .min(100.0)
    }
}

#[cfg(feature = "pi")]
fn parse_i2c_addr(s: &str) -> Option<u16> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(rest, 16).ok()
    } else {
        s.parse().ok()
    }
}

impl BmsDriver for Bms {
    fn get_status(&self) -> BmsStatus {
        Bms::get_status(self)
    }

    fn update(&mut self) {
        Bms::update(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charge_percentage_calculation() {
        let mut bms = Bms::init();
        bms.voltage = 7.2;
        assert_eq!(bms.calculate_charge_percentage(), 0.0);

        bms.voltage = 8.4;
        assert_eq!(bms.calculate_charge_percentage(), 100.0);

        bms.voltage = 7.8;
        let pct = bms.calculate_charge_percentage();
        assert!((pct - 50.0).abs() < 0.02, "mid-pack SOC ~50%, got {pct}");
    }

    #[test]
    fn test_charge_calculation_out_of_range() {
        let mut bms = Bms::init();
        bms.voltage = 6.0;
        assert_eq!(bms.calculate_charge_percentage(), 0.0);

        bms.voltage = 9.0;
        assert_eq!(bms.calculate_charge_percentage(), 100.0);
    }

    #[test]
    fn test_health_status_logic() {
        let mut bms = Bms::init();

        bms.voltage = 7.8;
        bms.cell1_voltage = 3.9;
        bms.cell2_voltage = 3.9;
        bms.recompute_health();
        assert!(bms.is_healthy);

        bms.voltage = 8.6;
        bms.cell1_voltage = 4.3;
        bms.cell2_voltage = 4.3;
        bms.recompute_health();
        assert!(!bms.is_healthy);

        bms.voltage = 6.9;
        bms.cell1_voltage = 3.45;
        bms.cell2_voltage = 3.45;
        bms.temperature = 50.0;
        bms.recompute_health();
        assert!(!bms.is_healthy);
    }

    #[test]
    fn test_status_struct_population() {
        let mut bms = Bms::init();
        bms.voltage = 7.6;
        bms.temperature = 20.0;
        bms.is_healthy = true;
        bms.charge_percentage = 40.0;

        let status = bms.get_status();
        assert_eq!(status.voltage, 7.6);
        assert_eq!(status.temperature, 20.0);
        assert!(status.is_healthy);
        assert_eq!(status.charge_percentage, 40.0);
    }

    #[test]
    fn test_per_cell_voltage_bounds() {
        let mut bms = Bms::init();

        bms.voltage = 7.2;
        bms.cell1_voltage = 3.6;
        bms.cell2_voltage = 3.6;
        bms.recompute_health();
        assert!(bms.is_healthy);

        bms.voltage = 8.4;
        bms.cell1_voltage = 4.2;
        bms.cell2_voltage = 4.2;
        bms.recompute_health();
        assert!(bms.is_healthy);

        bms.voltage = 8.0;
        bms.cell1_voltage = 4.3;
        bms.cell2_voltage = 4.0;
        bms.recompute_health();
        assert!(!bms.is_healthy);

        bms.voltage = 7.6;
        bms.cell1_voltage = 3.5;
        bms.cell2_voltage = 4.0;
        bms.recompute_health();
        assert!(!bms.is_healthy);
    }
}

/// On-device smoke test: `cargo test --features pi bms_max17049_i2c_smoke -- --ignored`
#[cfg(all(test, feature = "pi", target_os = "linux"))]
mod pi_smoke_tests {
    use super::*;
    use rppal::i2c::I2c;

    #[test]
    #[ignore = "Requires Raspberry Pi with MAX17049 on I2C; run: cargo test --features pi bms_max17049_i2c_smoke -- --ignored"]
    fn bms_max17049_i2c_smoke() {
        let bus = std::env::var("GPM_BMS_I2C_BUS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_u8);

        let addr = std::env::var("GPM_MAX17049_ADDR")
            .ok()
            .and_then(|s| super::parse_i2c_addr(&s))
            .unwrap_or(crate::hardware::max17049::DEFAULT_SLAVE_ADDR);

        let mut i2c = I2c::with_bus(bus).expect("open I2C bus");
        i2c.set_slave_address(addr).expect("set MAX17049 address");

        let version = crate::hardware::max17049::read_version(&i2c).expect("read VERSION");
        assert_ne!(version, 0, "VERSION register should be non-zero");

        let v = crate::hardware::max17049::read_pack_voltage_v(&i2c).expect("read VCELL");
        assert!(
            (5.0..=10.5).contains(&v),
            "MAX17049 pack voltage {v} V out of expected range (check wiring / 2S pack)"
        );

        let soc = crate::hardware::max17049::read_soc_percent(&i2c).expect("read SOC");
        assert!((0.0..=100.0).contains(&soc));

        let mut bms = Bms::init();
        bms.update();
        assert!(
            bms.voltage > 0.0,
            "Bms::update should populate voltage from gauge"
        );
    }
}
