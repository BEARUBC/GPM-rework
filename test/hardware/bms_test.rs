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