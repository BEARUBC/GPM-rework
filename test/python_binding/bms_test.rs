#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::bms::BmsStatus as RustBmsStatus;

    // -------------------------
    // Helper
    // -------------------------
    fn make_rust_bms_status(
        voltage: f32,
        current: f32,
        temperature: f32,
        is_healthy: bool,
        charge_percentage: f32,
    ) -> RustBmsStatus {
        RustBmsStatus {
            voltage,
            current,
            temperature,
            is_healthy,
            charge_percentage,
        }
    }

    // -------------------------
    // Unit Tests: From<RustBmsStatus> for BmsStatus
    // -------------------------
    #[test]
    fn test_from_rust_bms_status_maps_all_fields() {
        let rust_status = make_rust_bms_status(12.0, 1.5, 25.0, true, 76.92);
        let py_status = BmsStatus::from(rust_status);

        assert_eq!(py_status.voltage, 12.0);
        assert_eq!(py_status.current, 1.5);
        assert_eq!(py_status.temperature, 25.0);
        assert!(py_status.is_healthy);
        assert!((py_status.charge_percentage - 76.92).abs() < 0.001);
    }

    #[test]
    fn test_from_rust_bms_status_unhealthy() {
        let rust_status = make_rust_bms_status(9.0, 0.0, 60.0, false, 0.0);
        let py_status = BmsStatus::from(rust_status);

        assert!(!py_status.is_healthy);
        assert_eq!(py_status.charge_percentage, 0.0);
    }

    #[test]
    fn test_from_rust_bms_status_full_charge() {
        let rust_status = make_rust_bms_status(12.6, 0.0, 22.0, true, 100.0);
        let py_status = BmsStatus::from(rust_status);

        assert_eq!(py_status.voltage, 12.6);
        assert_eq!(py_status.charge_percentage, 100.0);
    }

    #[test]
    fn test_from_rust_bms_status_negative_current() {
        // Negative current = charging scenario
        let rust_status = make_rust_bms_status(12.3, -1.0, 23.0, true, 88.46);
        let py_status = BmsStatus::from(rust_status);

        assert_eq!(py_status.current, -1.0);
    }

    #[test]
    fn test_from_rust_bms_status_zero_values() {
        let rust_status = make_rust_bms_status(0.0, 0.0, 0.0, false, 0.0);
        let py_status = BmsStatus::from(rust_status);

        assert_eq!(py_status.voltage, 0.0);
        assert_eq!(py_status.current, 0.0);
        assert_eq!(py_status.temperature, 0.0);
        assert!(!py_status.is_healthy);
        assert_eq!(py_status.charge_percentage, 0.0);
    }

    // -------------------------
    // Unit Tests: BmsStatus Clone
    // -------------------------
    #[test]
    fn test_bms_status_clone_preserves_fields() {
        let rust_status = make_rust_bms_status(11.8, 0.5, 27.0, true, 69.23);
        let py_status = BmsStatus::from(rust_status);
        let cloned = py_status.clone();

        assert_eq!(cloned.voltage, py_status.voltage);
        assert_eq!(cloned.current, py_status.current);
        assert_eq!(cloned.temperature, py_status.temperature);
        assert_eq!(cloned.is_healthy, py_status.is_healthy);
        assert_eq!(cloned.charge_percentage, py_status.charge_percentage);
    }

    // -------------------------
    // Unit Tests: Bms::new()
    // -------------------------
    #[test]
    fn test_bms_new_succeeds() {
        let bms = Bms::new();
        assert!(bms.is_ok(), "Bms::new() should return Ok");
    }

    #[test]
    fn test_bms_new_inner_initialized_with_defaults() {
        let bms = Bms::new().unwrap();
        // The inner RustBms is initialized via Resource::init()
        // Default values: voltage=12.0, current=0.0, temp=25.0, healthy=true
        assert_eq!(bms.inner.voltage, 12.0);
        assert_eq!(bms.inner.current, 0.0);
        assert_eq!(bms.inner.temperature, 25.0);
        assert!(bms.inner.is_healthy);
    }

    // -------------------------
    // Unit Tests: Bms::update()
    // -------------------------
    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_returns_ok() {
        let mut bms = Bms::new().unwrap();
        assert!(bms.update().is_ok());
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_changes_voltage() {
        let mut bms = Bms::new().unwrap();
        let initial_voltage = bms.inner.voltage;
        // Run multiple times; at least one should differ from init default
        let mut changed = false;
        for _ in 0..20 {
            bms.update().unwrap();
            if (bms.inner.voltage - initial_voltage).abs() > f32::EPSILON {
                changed = true;
                break;
            }
        }
        assert!(changed, "voltage should change after update()");
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_update_voltage_stays_in_simulated_range() {
        let mut bms = Bms::new().unwrap();
        for _ in 0..50 {
            bms.update().unwrap();
            assert!(
                bms.inner.voltage >= 11.5 && bms.inner.voltage <= 12.5,
                "voltage out of simulated range: {}",
                bms.inner.voltage
            );
        }
    }

    // -------------------------
    // Integration Tests: Bms::get_status()
    // -------------------------
    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_get_status_returns_ok() {
        pyo3::prepare_freethreaded_python();
        let mut bms = Bms::new().unwrap();
        let result = bms.get_status();
        assert!(result.is_ok(), "get_status() should return Ok");
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_get_status_calls_update_internally() {
        pyo3::prepare_freethreaded_python();
        let mut bms = Bms::new().unwrap();
        let init_voltage = bms.inner.voltage; // 12.0 from init

        let mut status_changed = false;
        for _ in 0..20 {
            let status = bms.get_status().unwrap();
            if (status.voltage - init_voltage).abs() > f32::EPSILON {
                status_changed = true;
                break;
            }
        }
        assert!(status_changed, "get_status() should update readings internally");
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_get_status_voltage_in_expected_range() {
        pyo3::prepare_freethreaded_python();
        let mut bms = Bms::new().unwrap();
        for _ in 0..30 {
            let status = bms.get_status().unwrap();
            assert!(
                status.voltage >= 11.5 && status.voltage <= 12.5,
                "unexpected voltage in status: {}",
                status.voltage
            );
        }
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_get_status_is_healthy_true_in_simulation() {
        pyo3::prepare_freethreaded_python();
        let mut bms = Bms::new().unwrap();
        for _ in 0..30 {
            let status = bms.get_status().unwrap();
            assert!(
                status.is_healthy,
                "expected healthy in simulation range, got unhealthy"
            );
        }
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_get_status_charge_percentage_reasonable_in_simulation() {
        pyo3::prepare_freethreaded_python();
        let mut bms = Bms::new().unwrap();
        for _ in 0..30 {
            let status = bms.get_status().unwrap();
            // Simulated voltage 11.5–12.5V -> ~57%–96%
            assert!(
                status.charge_percentage >= 57.0 && status.charge_percentage <= 97.0,
                "unexpected charge_percentage: {}",
                status.charge_percentage
            );
        }
    }

    // -------------------------
    // Edge Cases: f32 extremes
    // -------------------------
    #[test]
    fn test_from_rust_bms_status_with_f32_max() {
        let rust_status = make_rust_bms_status(f32::MAX, f32::MAX, f32::MAX, false, 100.0);
        let py_status = BmsStatus::from(rust_status);
        assert_eq!(py_status.voltage, f32::MAX);
    }

    #[test]
    fn test_from_rust_bms_status_with_nan_charge() {
        // NaN can appear from bad sensor reads — verify it passes through without panic
        let rust_status = make_rust_bms_status(f32::NAN, 0.0, 25.0, false, f32::NAN);
        let py_status = BmsStatus::from(rust_status);
        assert!(py_status.voltage.is_nan());
        assert!(py_status.charge_percentage.is_nan());
    }

    #[test]
    fn test_from_rust_bms_status_with_infinity() {
        let rust_status = make_rust_bms_status(f32::INFINITY, 0.0, 25.0, false, 100.0);
        let py_status = BmsStatus::from(rust_status);
        assert!(py_status.voltage.is_infinite());
    }
}