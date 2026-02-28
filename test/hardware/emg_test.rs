#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------
    // Helpers
    // -------------------------

    /// Build an Emg with controllable thresholds and buffer, bypassing ADC hardware.
    fn make_emg(inner_threshold: f32, outer_threshold: f32) -> Emg {
        let mut emg = Emg::init();
        emg.inner_threshold = inner_threshold;
        emg.outer_threshold = outer_threshold;
        emg
    }

    // -------------------------
    // Unit Tests: init()
    // -------------------------

    #[test]
    fn test_init_default_values() {
        let emg = Emg::init();
        assert_eq!(emg.buffer_size, 256);
        assert!(emg.buffer.is_empty());
        assert_eq!(emg.inner_threshold, 450.0);
        assert_eq!(emg.outer_threshold, 450.0);
        assert_eq!(emg.current_channel_0, 0.0);
        assert_eq!(emg.current_channel_1, 0.0);
    }

    #[test]
    fn test_name() {
        assert_eq!(Emg::name(), "Emg");
    }

    // -------------------------
    // Unit Tests: configure()
    // -------------------------

    #[test]
    fn test_configure_sets_buffer_size() {
        let mut emg = Emg::init();
        emg.configure(512);
        assert_eq!(emg.buffer_size, 512);
    }

    #[test]
    fn test_configure_resets_buffer_to_empty() {
        let mut emg = Emg::init();
        emg.buffer = vec![1, 2, 3];
        emg.configure(128);
        // Vec::with_capacity creates an empty vec
        assert!(emg.buffer.is_empty());
    }

    #[test]
    fn test_configure_with_zero_buffer_size() {
        let mut emg = Emg::init();
        emg.configure(0);
        assert_eq!(emg.buffer_size, 0);
        assert!(emg.buffer.is_empty());
    }

    #[test]
    fn test_configure_with_large_buffer_size() {
        let mut emg = Emg::init();
        emg.configure(65536);
        assert_eq!(emg.buffer_size, 65536);
    }

    // -------------------------
    // Unit Tests: is_ready()
    // -------------------------

    #[test]
    fn test_is_ready_always_true() {
        let emg = Emg::init();
        assert!(emg.is_ready());
    }

    // -------------------------
    // Unit Tests: get_latest_samples()
    // -------------------------

    #[test]
    fn test_get_latest_samples_empty_on_init() {
        let emg = Emg::init();
        assert!(emg.get_latest_samples().is_empty());
    }

    #[test]
    fn test_get_latest_samples_returns_clone_of_buffer() {
        let mut emg = Emg::init();
        emg.buffer = vec![100, 200, 300];
        let samples = emg.get_latest_samples();
        assert_eq!(samples, vec![100, 200, 300]);
    }

    #[test]
    fn test_get_latest_samples_is_independent_clone() {
        let mut emg = Emg::init();
        emg.buffer = vec![10, 20];
        let mut samples = emg.get_latest_samples();
        samples.push(99); // mutate clone
        assert_eq!(emg.buffer, vec![10, 20]); // original unaffected
    }

    // -------------------------
    // Unit Tests: calibrate()
    // -------------------------

    #[test]
    fn test_calibrate_sets_thresholds() {
        let mut emg = Emg::init();
        emg.calibrate(300.0, 600.0);
        assert_eq!(emg.inner_threshold, 300.0);
        assert_eq!(emg.outer_threshold, 600.0);
    }

    #[test]
    fn test_calibrate_with_zero_thresholds() {
        let mut emg = Emg::init();
        emg.calibrate(0.0, 0.0);
        assert_eq!(emg.inner_threshold, 0.0);
        assert_eq!(emg.outer_threshold, 0.0);
    }

    #[test]
    fn test_calibrate_with_equal_thresholds() {
        let mut emg = Emg::init();
        emg.calibrate(500.0, 500.0);
        assert_eq!(emg.inner_threshold, 500.0);
        assert_eq!(emg.outer_threshold, 500.0);
    }

    #[test]
    fn test_calibrate_overwrites_previous() {
        let mut emg = Emg::init();
        emg.calibrate(100.0, 200.0);
        emg.calibrate(350.0, 700.0);
        assert_eq!(emg.inner_threshold, 350.0);
        assert_eq!(emg.outer_threshold, 700.0);
    }

    // -------------------------
    // Unit Tests: process_data()
    // -------------------------

    #[test]
    fn test_process_data_open_gesture() {
        // ch0 >= inner AND ch1 <= outer -> 1 (Open)
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[500.0, 400.0]).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_data_close_gesture() {
        // ch0 <= inner AND ch1 >= outer -> 0 (Close)
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[400.0, 500.0]).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_process_data_hold_no_change() {
        // Neither condition met -> -1 (Hold)
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[400.0, 400.0]).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    fn test_process_data_both_above_threshold_is_hold() {
        // Both high — neither Open nor Close condition is cleanly met
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[500.0, 500.0]).unwrap();
        // ch0 >= inner but ch1 <= outer (X, 500 == outer boundary)
        // ch0 <= inner so Close also fails
        // Falls through to Hold
        assert_eq!(result, -1);
    }

    #[test]
    fn test_process_data_exactly_at_inner_threshold_open() {
        // ch0 == inner_threshold counts as >= → Open if ch1 <= outer
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[450.0, 450.0]).unwrap();
        // ch0 >= 450 , ch1 <= 450 → Open wins (first branch)
        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_data_exactly_at_thresholds_close_branch() {
        // Force close branch: ch0 <= inner, ch1 >= outer, both at boundary
        let emg = make_emg(500.0, 400.0);
        let result = emg.process_data(&[450.0, 450.0]).unwrap();
        // ch0(450) >= inner(500)? No. ch0(450) <= inner(500)? Yes. ch1(450) >= outer(400)? Yes -> Close
        assert_eq!(result, 0);
    }

    #[test]
    fn test_process_data_error_on_empty_slice() {
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected 2 EMG values"));
    }

    #[test]
    fn test_process_data_error_on_one_value() {
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[300.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_data_error_on_three_values() {
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[300.0, 400.0, 500.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_data_with_zero_thresholds_all_above() {
        // With both thresholds at 0.0, any positive values -> Open
        let emg = make_emg(0.0, 0.0);
        let result = emg.process_data(&[1.0, 0.0]).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_data_with_negative_values() {
        // Negative sensor values (e.g. offset/noise) — should not panic
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[-100.0, -200.0]).unwrap();
        // Both below thresholds → Hold
        assert_eq!(result, -1);
    }

    #[test]
    fn test_process_data_with_f32_max() {
        let emg = make_emg(450.0, 450.0);
        let result = emg.process_data(&[f32::MAX, 0.0]).unwrap();
        assert_eq!(result, 1); // ch0 huge >= inner, ch1(0) <= outer -> Open
    }

    #[test]
    fn test_process_data_does_not_mutate_emg_state() {
        let emg = make_emg(450.0, 450.0);
        let _ = emg.process_data(&[500.0, 400.0]);
        // process_data takes &self — thresholds must be unchanged
        assert_eq!(emg.inner_threshold, 450.0);
        assert_eq!(emg.outer_threshold, 450.0);
    }

    // -------------------------
    // Integration Tests: configure -> get_latest_samples
    // -------------------------

    #[test]
    fn test_configure_then_get_latest_samples_empty() {
        let mut emg = Emg::init();
        emg.configure(64);
        assert!(emg.get_latest_samples().is_empty());
        assert_eq!(emg.buffer_size, 64);
    }

    #[test]
    fn test_calibrate_then_process_data_open() {
        let mut emg = Emg::init();
        emg.calibrate(300.0, 600.0);
        // ch0(350) >= inner(300), ch1(550) <= outer(600) -> Open
        let result = emg.process_data(&[350.0, 550.0]).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_calibrate_then_process_data_close() {
        let mut emg = Emg::init();
        emg.calibrate(300.0, 600.0);
        // ch0(250) <= inner(300), ch1(650) >= outer(600) -> Close
        let result = emg.process_data(&[250.0, 650.0]).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calibrate_then_process_data_hold() {
        let mut emg = Emg::init();
        emg.calibrate(300.0, 600.0);
        // Neither branch satisfied
        let result = emg.process_data(&[250.0, 550.0]).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    fn test_recalibrate_changes_process_data_outcome() {
        let mut emg = Emg::init();
        // With default thresholds (450/450), these values -> Hold
        let before = emg.process_data(&[400.0, 400.0]).unwrap();
        assert_eq!(before, -1);

        // After recalibrating lower, ch0(400) <= inner(300)? No. ch0 >= inner(300), ch1(400) <= outer(450) -> Open
        emg.calibrate(300.0, 450.0);
        let after = emg.process_data(&[400.0, 400.0]).unwrap();
        assert_eq!(after, 1);
    }
}