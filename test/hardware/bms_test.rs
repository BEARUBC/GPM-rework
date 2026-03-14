#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------
    // Helper
    // -------------------------
    fn make_bms(voltage: f32, current: f32, temperature: f32, is_healthy: bool) -> Bms {
        Bms { voltage, current, temperature, is_healthy }
    }

    // -------------------------
    // Unit Tests: init()
    // -------------------------
    #[test]
    fn test_init_default_values() {
        let bms = Bms::init();
        assert_eq!(bms.voltage, 12.0);
        assert_eq!(bms.current, 0.0);
        assert_eq!(bms.temperature, 25.0);
        assert!(bms.is_healthy);
    }

    #[test]
    fn test_name() {
        assert_eq!(Bms::name(), "Bms");
    }

    // -------------------------
    // Unit Tests: get_status()
    // -------------------------
    #[test]
    fn test_get_status_reflects_fields() {
        let bms = make_bms(12.0, 1.0, 25.0, true);
        let status = bms.get_status();

        assert_eq!(status.voltage, 12.0);
        assert_eq!(status.current, 1.0);
        assert_eq!(status.temperature, 25.0);
        assert!(status.is_healthy);
    }

    #[test]
    fn test_get_status_charge_percentage_included() {
        let bms = make_bms(12.0, 0.0, 25.0, true);
        let status = bms.get_status();
        // 12.0V → (12.0 - 10.0) / 2.6 * 100 ≈ 76.92%
        assert!((status.charge_percentage - 76.92).abs() < 0.1);
    }

    // -------------------------
    // Unit Tests: calculate_charge_percentage()
    // -------------------------
    #[test]
    fn test_charge_percentage_full() {
        let bms = make_bms(12.6, 0.0, 25.0, true);
        let status = bms.get_status();
        assert_eq!(status.charge_percentage, 100.0);
    }

    #[test]
    fn test_charge_percentage_empty() {
        let bms = make_bms(10.0, 0.0, 25.0, true);
        let status = bms.get_status();
        assert_eq!(status.charge_percentage, 0.0);
    }

    #[test]
    fn test_charge_percentage_midpoint() {
        let mid_voltage = 11.3; // (10.0 + 12.6) / 2
        let bms = make_bms(mid_voltage, 0.0, 25.0, true);
        let status = bms.get_status();
        assert!((status.charge_percentage - 50.0).abs() < 0.1);
    }

    // -------------------------
    // Edge Cases: voltage clamping
    // -------------------------
    #[test]
    fn test_charge_percentage_below_min_voltage_clamped_to_zero() {
        let bms = make_bms(5.0, 0.0, 25.0, false); // way below 10V
        let status = bms.get_status();
        assert_eq!(status.charge_percentage, 0.0);
    }

    #[test]
    fn test_charge_percentage_above_max_voltage_clamped_to_100() {
        let bms = make_bms(15.0, 0.0, 25.0, true); // above 12.6V
        let status = bms.get_status();
        assert_eq!(status.charge_percentage, 100.0);
    }

    #[test]
    fn test_charge_percentage_exactly_at_min_boundary() {
        let bms = make_bms(10.0, 0.0, 25.0, false);
        assert_eq!(bms.get_status().charge_percentage, 0.0);
    }

    #[test]
    fn test_charge_percentage_exactly_at_max_boundary() {
        let bms = make_bms(12.6, 0.0, 25.0, true);
        assert_eq!(bms.get_status().charge_percentage, 100.0);
    }

    // -------------------------
    // Edge Cases: is_healthy flag
    // -------------------------
    #[test]
    fn test_unhealthy_bms_status_reflects_correctly() {
        let bms = make_bms(9.0, 0.5, 60.0, false);
        let status = bms.get_status();
        assert!(!status.is_healthy);
        assert_eq!(status.charge_percentage, 0.0); // clamped
    }

    #[test]
    fn test_healthy_flag_not_auto_computed_by_get_status() {
        // is_healthy is stored as-is; get_status doesn't recompute it
        let bms = make_bms(9.0, 0.0, 25.0, true); // low voltage but manually set healthy
        let status = bms.get_status();
        assert!(status.is_healthy); // reflects stored value, not recomputed
    }

    // -------------------------
    // Edge Cases: negative / zero current
    // -------------------------
    #[test]
    fn test_negative_current_passes_through() {
        let bms = make_bms(12.0, -1.5, 25.0, true); // charging current
        let status = bms.get_status();
        assert_eq!(status.current, -1.5);
    }

    #[test]
    fn test_zero_current() {
        let bms = make_bms(12.0, 0.0, 25.0, true);
        assert_eq!(bms.get_status().current, 0.0);
    }

    // -------------------------
    // Integration Tests: update() (non-pi / simulated)
    // -------------------------
    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_voltage_in_expected_range() {
        let mut bms = Bms::init();
        for _ in 0..50 {
            bms.update();
            assert!(
                bms.voltage >= 11.5 && bms.voltage <= 12.5,
                "voltage out of range: {}",
                bms.voltage
            );
        }
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_current_in_expected_range() {
        let mut bms = Bms::init();
        for _ in 0..50 {
            bms.update();
            assert!(
                bms.current >= 0.0 && bms.current <= 2.0,
                "current out of range: {}",
                bms.current
            );
        }
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_temperature_in_expected_range() {
        let mut bms = Bms::init();
        for _ in 0..50 {
            bms.update();
            assert!(
                bms.temperature >= 20.0 && bms.temperature <= 30.0,
                "temperature out of range: {}",
                bms.temperature
            );
        }
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_is_healthy_consistent_with_simulated_values() {
        let mut bms = Bms::init();
        for _ in 0..50 {
            bms.update();
            // In simulation range (11.5–12.5V, 20–30°C), should always be healthy
            assert!(
                bms.is_healthy,
                "expected healthy but got unhealthy: voltage={}, temp={}",
                bms.voltage, bms.temperature
            );
        }
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_then_get_status_charge_percentage_reasonable() {
        let mut bms = Bms::init();
        bms.update();
        let status = bms.get_status();
        // Simulated voltage 11.5–12.5V -> charge ~57%–96%
        assert!(
            status.charge_percentage >= 57.0 && status.charge_percentage <= 97.0,
            "charge_percentage out of expected range: {}",
            status.charge_percentage
        );
    }

    // -------------------------
    // Integration: BmsStatus clone/debug
    // -------------------------
    #[test]
    fn test_bms_status_clone() {
        let bms = make_bms(12.0, 1.0, 25.0, true);
        let status = bms.get_status();
        let cloned = status.clone();
        assert_eq!(cloned.voltage, status.voltage);
        assert_eq!(cloned.charge_percentage, status.charge_percentage);
    }

    #[test]
    fn test_bms_status_debug_format() {
        let bms = make_bms(12.0, 1.0, 25.0, true);
        let status = bms.get_status();
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("voltage"));
        assert!(debug_str.contains("charge_percentage"));
    }
}

// Updated for March 7th

use super::Resource;

// -------------------------------------------------------------------------
// Voltage alert thresholds — adjust here if hardware spec changes
// Matched to calculate_charge_percentage() bounds (0 % = 10 V, 100 % = 12.6 V)
// -------------------------------------------------------------------------
pub const UNDERVOLTAGE_THRESHOLD: f32 = 10.0; // volts
pub const OVERVOLTAGE_THRESHOLD: f32  = 12.6; // volts

// -------------------------------------------------------------------------
// I2C slave registry
// -------------------------------------------------------------------------
/// Known I2C addresses on the BMS bus.
/// Extend this list as ICs are confirmed with the hardware team.
pub const BMS_I2C_ADDRESSES: &[u8] = &[
    0x36, // MAX17048 fuel-gauge (example)
    0x6B, // BQ25896 charger IC (example)
    // Add addresses here once confirmed - Ask per integration notes
];

pub struct Bms {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
}

#[derive(Clone, Debug)]
pub struct BmsStatus {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
    pub charge_percentage: f32,
}

// -------------------------------------------------------------------------
// Voltage alert type returned by check_voltage_alerts()
// -------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
pub enum VoltageAlert {
    /// Voltage is within the safe operating window
    Ok,
    /// Voltage has dropped below UNDERVOLTAGE_THRESHOLD - risk of cell damage
    Undervoltage,
    /// Voltage has exceeded OVERVOLTAGE_THRESHOLD - risk of overcharge
    Overvoltage,
}

impl Resource for Bms {
    fn init() -> Self {
        Bms {
            voltage: 12.0,
            current: 0.0,
            temperature: 25.0,
            is_healthy: true,
        }
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
            charge_percentage: self.calculate_charge_percentage(),
        }
    }

    pub fn update(&mut self) {
        #[cfg(not(feature = "pi"))]
        {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            self.voltage     = 11.5 + rng.gen::<f32>() * 1.0; // 11.5–12.5 V
            self.current     = rng.gen::<f32>() * 2.0;         // 0–2 A
            self.temperature = 20.0 + rng.gen::<f32>() * 10.0; // 20–30 degrees C
            self.is_healthy  = self.voltage > UNDERVOLTAGE_THRESHOLD
                && self.temperature < 50.0;
        }
    }

    fn calculate_charge_percentage(&self) -> f32 {
        let min_voltage = UNDERVOLTAGE_THRESHOLD;
        let max_voltage = OVERVOLTAGE_THRESHOLD;
        ((self.voltage - min_voltage) / (max_voltage - min_voltage) * 100.0)
            .max(0.0)
            .min(100.0)
    }

    // -------------------------------------------------------------------------
    // Integration: voltage alert
    // -------------------------------------------------------------------------

    /// Compare the current voltage reading against the defined thresholds and
    /// return the appropriate ['VoltageAlert'] variant.
    ///
    /// This is the canonical place to detect over/undervoltage conditions
    /// before propagating a fault to the carry board or triggering an EMG stop.
    pub fn check_voltage_alerts(&self) -> VoltageAlert {
        if self.voltage < UNDERVOLTAGE_THRESHOLD {
            VoltageAlert::Undervoltage
        } else if self.voltage > OVERVOLTAGE_THRESHOLD {
            VoltageAlert::Overvoltage
        } else {
            VoltageAlert::Ok
        }
    }

    // -------------------------------------------------------------------------
    // Integration: I2C address polling
    // -------------------------------------------------------------------------

    /// Poll a single I2C address and return the raw bytes read from that slave.
    ///
    /// Each IC on the BMS bus has a unique address (see 'BMS_I2C_ADDRESSES').
    /// The caller decides how to interpret the payload - e.g. a fuel-gauge
    /// returns SOC registers while a charger IC returns fault/status flags.
    ///
    /// On non-pi builds this returns a fixed 2-byte stub so unit tests work
    /// without real hardware.
    ///
    /// # Arguments
    /// * 'address' - 7-bit I2C slave address (e.g. 0x36)
    #[cfg(feature = "pi")]
    pub fn poll_i2c_address(&self, address: u8) -> anyhow::Result<Vec<u8>> {
        use rppal::i2c::I2c;
        let mut i2c = I2c::new().context("Failed to open I2C bus")?;
        i2c.set_slave_address(address as u16)
            .with_context(|| format!("Failed to set I2C slave address 0x{:02X}", address))?;

        let mut buf = [0u8; 2];
        i2c.read(&mut buf)
            .with_context(|| format!("I2C read failed for address 0x{:02X}", address))?;
        Ok(buf.to_vec())
    }

    #[cfg(not(feature = "pi"))]
    pub fn poll_i2c_address(&self, address: u8) -> anyhow::Result<Vec<u8>> {
        // Simulation: return a recognisable stub payload so tests can assert
        // on structure without needing a real I2C bus.
        Ok(vec![address, 0xAB])
    }

    /// Convenience: poll every address in `BMS_I2C_ADDRESSES` and return a
    /// map of address → raw bytes.  Errors on individual slaves are collected
    /// rather than short-circuiting so a single bad IC doesn't hide the rest.
    pub fn poll_all_i2c(&self) -> Vec<(u8, anyhow::Result<Vec<u8>>)> {
        BMS_I2C_ADDRESSES
            .iter()
            .map(|&addr| (addr, self.poll_i2c_address(addr)))
            .collect()
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn make_bms(voltage: f32, current: f32, temperature: f32, is_healthy: bool) -> Bms {
        Bms { voltage, current, temperature, is_healthy }
    }

    // -------------------------------------------------------------------------
    // Existing tests (unchanged)
    // -------------------------------------------------------------------------

    #[test]
    fn test_init_default_values() {
        let bms = Bms::init();
        assert_eq!(bms.voltage, 12.0);
        assert_eq!(bms.current, 0.0);
        assert_eq!(bms.temperature, 25.0);
        assert!(bms.is_healthy);
    }

    #[test]
    fn test_name() {
        assert_eq!(Bms::name(), "Bms");
    }

    #[test]
    fn test_get_status_reflects_fields() {
        let bms = make_bms(12.0, 1.0, 25.0, true);
        let status = bms.get_status();
        assert_eq!(status.voltage, 12.0);
        assert_eq!(status.current, 1.0);
        assert_eq!(status.temperature, 25.0);
        assert!(status.is_healthy);
    }

    #[test]
    fn test_charge_percentage_full() {
        let bms = make_bms(12.6, 0.0, 25.0, true);
        assert_eq!(bms.get_status().charge_percentage, 100.0);
    }

    #[test]
    fn test_charge_percentage_empty() {
        let bms = make_bms(10.0, 0.0, 25.0, true);
        assert_eq!(bms.get_status().charge_percentage, 0.0);
    }

    // -------------------------------------------------------------------------
    // check_voltage_alerts()
    // -------------------------------------------------------------------------

    #[test]
    fn test_voltage_alert_ok_at_nominal() {
        let bms = make_bms(12.0, 0.0, 25.0, true);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Ok);
    }

    #[test]
    fn test_voltage_alert_ok_exactly_at_lower_threshold() {
        // Boundary: 10.0 V is NOT below threshold, so still Ok
        let bms = make_bms(UNDERVOLTAGE_THRESHOLD, 0.0, 25.0, true);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Ok);
    }

    #[test]
    fn test_voltage_alert_ok_exactly_at_upper_threshold() {
        // Boundary: 12.6 V is NOT above threshold, so still Ok
        let bms = make_bms(OVERVOLTAGE_THRESHOLD, 0.0, 25.0, true);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Ok);
    }

    #[test]
    fn test_voltage_alert_undervoltage() {
        let bms = make_bms(9.5, 0.0, 25.0, false);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Undervoltage);
    }

    #[test]
    fn test_voltage_alert_overvoltage() {
        let bms = make_bms(13.0, 0.0, 25.0, false);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Overvoltage);
    }

    #[test]
    fn test_voltage_alert_just_below_min() {
        let bms = make_bms(UNDERVOLTAGE_THRESHOLD - 0.01, 0.0, 25.0, false);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Undervoltage);
    }

    #[test]
    fn test_voltage_alert_just_above_max() {
        let bms = make_bms(OVERVOLTAGE_THRESHOLD + 0.01, 0.0, 25.0, false);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Overvoltage);
    }

    // -------------------------------------------------------------------------
    // poll_i2c_address() — simulation
    // -------------------------------------------------------------------------

    #[test]
    fn test_poll_i2c_address_returns_ok() {
        let bms = Bms::init();
        assert!(bms.poll_i2c_address(0x36).is_ok());
    }

    #[test]
    fn test_poll_i2c_address_stub_contains_address_byte() {
        let bms = Bms::init();
        let bytes = bms.poll_i2c_address(0x36).unwrap();
        // Simulation stub: first byte echoes the address
        assert_eq!(bytes[0], 0x36);
    }

    #[test]
    fn test_poll_i2c_address_stub_second_byte_fixed() {
        let bms = Bms::init();
        let bytes = bms.poll_i2c_address(0x6B).unwrap();
        assert_eq!(bytes[1], 0xAB);
    }

    #[test]
    fn test_poll_i2c_different_addresses_return_different_first_byte() {
        let bms = Bms::init();
        let a = bms.poll_i2c_address(0x36).unwrap();
        let b = bms.poll_i2c_address(0x6B).unwrap();
        assert_ne!(a[0], b[0]);
    }

    // -------------------------------------------------------------------------
    // poll_all_i2c()
    // -------------------------------------------------------------------------

    #[test]
    fn test_poll_all_i2c_returns_entry_per_address() {
        let bms = Bms::init();
        let results = bms.poll_all_i2c();
        assert_eq!(results.len(), BMS_I2C_ADDRESSES.len());
    }

    #[test]
    fn test_poll_all_i2c_all_succeed_in_simulation() {
        let bms = Bms::init();
        for (addr, result) in bms.poll_all_i2c() {
            assert!(result.is_ok(), "poll failed for address 0x{:02X}", addr);
        }
    }

    #[test]
    fn test_poll_all_i2c_addresses_match_registry() {
        let bms = Bms::init();
        let results = bms.poll_all_i2c();
        for (i, &expected_addr) in BMS_I2C_ADDRESSES.iter().enumerate() {
            assert_eq!(results[i].0, expected_addr);
        }
    }

    // -------------------------------------------------------------------------
    // Integration: alert + i2c together
    // -------------------------------------------------------------------------

    #[test]
    fn test_undervoltage_triggers_alert_and_i2c_still_readable() {
        let bms = make_bms(9.0, 0.0, 25.0, false);
        assert_eq!(bms.check_voltage_alerts(), VoltageAlert::Undervoltage);
        // Even in fault state the I2C bus should still respond
        assert!(bms.poll_i2c_address(0x36).is_ok());
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_then_check_alerts_always_ok_in_simulation() {
        let mut bms = Bms::init();
        for _ in 0..50 {
            bms.update();
            // Simulated voltage is 11.5–12.5 V, always within 10.0–12.6 V window
            assert_eq!(
                bms.check_voltage_alerts(),
                VoltageAlert::Ok,
                "unexpected alert at voltage {}",
                bms.voltage
            );
        }
    }
}