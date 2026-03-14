#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------
    // Helper
    // -------------------------

    fn make_emg_with_thresholds(inner: f32, outer: f32) -> Emg {
        let mut emg = Emg::new().unwrap();
        emg.calibrate(inner, outer).unwrap();
        emg
    }

    // -------------------------
    // Unit Tests: new()
    // -------------------------

    #[test]
    fn test_new_returns_ok() {
        assert!(Emg::new().is_ok());
    }

    #[test]
    fn test_new_inner_initialized_with_defaults() {
        let emg = Emg::new().unwrap();
        assert_eq!(emg.inner.buffer_size, 256);
        assert!(emg.inner.buffer.is_empty());
        assert_eq!(emg.inner.inner_threshold, 450.0);
        assert_eq!(emg.inner.outer_threshold, 450.0);
        assert_eq!(emg.inner.current_channel_0, 0.0);
        assert_eq!(emg.inner.current_channel_1, 0.0);
    }

    // -------------------------
    // Unit Tests: configure()
    // -------------------------

    #[test]
    fn test_configure_returns_ok() {
        let mut emg = Emg::new().unwrap();
        assert!(emg.configure(512).is_ok());
    }

    #[test]
    fn test_configure_updates_buffer_size() {
        let mut emg = Emg::new().unwrap();
        emg.configure(128).unwrap();
        assert_eq!(emg.inner.buffer_size, 128);
    }

    #[test]
    fn test_configure_clears_buffer() {
        let mut emg = Emg::new().unwrap();
        emg.inner.buffer = vec![1, 2, 3];
        emg.configure(64).unwrap();
        assert!(emg.inner.buffer.is_empty());
    }

    #[test]
    fn test_configure_zero_buffer_size() {
        let mut emg = Emg::new().unwrap();
        assert!(emg.configure(0).is_ok());
        assert_eq!(emg.inner.buffer_size, 0);
    }

    #[test]
    fn test_configure_large_buffer_size() {
        let mut emg = Emg::new().unwrap();
        assert!(emg.configure(65536).is_ok());
        assert_eq!(emg.inner.buffer_size, 65536);
    }

    // -------------------------
    // Unit Tests: is_ready()
    // -------------------------

    #[test]
    fn test_is_ready_always_true() {
        let emg = Emg::new().unwrap();
        assert!(emg.is_ready());
    }

    #[test]
    fn test_is_ready_after_configure() {
        let mut emg = Emg::new().unwrap();
        emg.configure(512).unwrap();
        assert!(emg.is_ready());
    }

    // -------------------------
    // Unit Tests: get_latest_samples()
    // -------------------------

    #[test]
    fn test_get_latest_samples_empty_on_init() {
        let emg = Emg::new().unwrap();
        let samples = emg.get_latest_samples().unwrap();
        assert!(samples.is_empty());
    }

    #[test]
    fn test_get_latest_samples_reflects_inner_buffer() {
        let mut emg = Emg::new().unwrap();
        emg.inner.buffer = vec![10, 20, 30];
        let samples = emg.get_latest_samples().unwrap();
        assert_eq!(samples, vec![10, 20, 30]);
    }

    #[test]
    fn test_get_latest_samples_returns_independent_clone() {
        let mut emg = Emg::new().unwrap();
        emg.inner.buffer = vec![100, 200];
        let mut samples = emg.get_latest_samples().unwrap();
        samples.push(999); // mutate clone
        assert_eq!(emg.inner.buffer, vec![100, 200]); // inner unchanged
    }

    // -------------------------
    // Unit Tests: calibrate()
    // -------------------------

    #[test]
    fn test_calibrate_returns_ok() {
        let mut emg = Emg::new().unwrap();
        assert!(emg.calibrate(300.0, 600.0).is_ok());
    }

    #[test]
    fn test_calibrate_sets_inner_threshold() {
        let mut emg = Emg::new().unwrap();
        emg.calibrate(300.0, 600.0).unwrap();
        assert_eq!(emg.inner.inner_threshold, 300.0);
    }

    #[test]
    fn test_calibrate_sets_outer_threshold() {
        let mut emg = Emg::new().unwrap();
        emg.calibrate(300.0, 600.0).unwrap();
        assert_eq!(emg.inner.outer_threshold, 600.0);
    }

    #[test]
    fn test_calibrate_zero_thresholds() {
        let mut emg = Emg::new().unwrap();
        emg.calibrate(0.0, 0.0).unwrap();
        assert_eq!(emg.inner.inner_threshold, 0.0);
        assert_eq!(emg.inner.outer_threshold, 0.0);
    }

    #[test]
    fn test_calibrate_overwrites_previous_values() {
        let mut emg = Emg::new().unwrap();
        emg.calibrate(100.0, 200.0).unwrap();
        emg.calibrate(350.0, 700.0).unwrap();
        assert_eq!(emg.inner.inner_threshold, 350.0);
        assert_eq!(emg.inner.outer_threshold, 700.0);
    }

    // -------------------------
    // Unit Tests: process_data()
    // -------------------------

    #[test]
    fn test_process_data_open_gesture() {
        let emg = make_emg_with_thresholds(450.0, 450.0);
        assert_eq!(emg.process_data(vec![500.0, 400.0]).unwrap(), 1);
    }

    #[test]
    fn test_process_data_close_gesture() {
        let emg = make_emg_with_thresholds(450.0, 450.0);
        assert_eq!(emg.process_data(vec![400.0, 500.0]).unwrap(), 0);
    }

    #[test]
    fn test_process_data_hold_gesture() {
        let emg = make_emg_with_thresholds(450.0, 450.0);
        assert_eq!(emg.process_data(vec![400.0, 400.0]).unwrap(), -1);
    }

    #[test]
    fn test_process_data_error_on_empty_vec() {
        let emg = Emg::new().unwrap();
        let result = emg.process_data(vec![]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Process error"));
    }

    #[test]
    fn test_process_data_error_on_one_value() {
        let emg = Emg::new().unwrap();
        assert!(emg.process_data(vec![300.0]).is_err());
    }

    #[test]
    fn test_process_data_error_on_three_values() {
        let emg = Emg::new().unwrap();
        assert!(emg.process_data(vec![300.0, 400.0, 500.0]).is_err());
    }

    #[test]
    fn test_process_data_at_exact_threshold_boundary() {
        // ch0 == inner AND ch1 == outer -> Open branch fires first
        let emg = make_emg_with_thresholds(450.0, 450.0);
        assert_eq!(emg.process_data(vec![450.0, 450.0]).unwrap(), 1);
    }

    #[test]
    fn test_process_data_negative_values_no_panic() {
        let emg = make_emg_with_thresholds(450.0, 450.0);
        let result = emg.process_data(vec![-100.0, -200.0]).unwrap();
        assert_eq!(result, -1); // both below threshold -> Hold
    }

    #[test]
    fn test_process_data_f32_max_no_panic() {
        let emg = make_emg_with_thresholds(450.0, 450.0);
        let result = emg.process_data(vec![f32::MAX, 0.0]).unwrap();
        assert_eq!(result, 1); // ch0 huge -> Open
    }

    #[test]
    fn test_process_data_does_not_mutate_state() {
        let emg = make_emg_with_thresholds(450.0, 450.0);
        let _ = emg.process_data(vec![500.0, 400.0]);
        assert_eq!(emg.inner.inner_threshold, 450.0);
        assert_eq!(emg.inner.outer_threshold, 450.0);
    }

    // -------------------------
    // Integration Tests: configure -> get_latest_samples
    // -------------------------

    #[test]
    fn test_configure_then_get_latest_samples_empty() {
        let mut emg = Emg::new().unwrap();
        emg.configure(64).unwrap();
        assert!(emg.get_latest_samples().unwrap().is_empty());
    }

    // -------------------------
    // Integration Tests: calibrate -> process_data
    // -------------------------

    #[test]
    fn test_calibrate_then_process_data_open() {
        let emg = make_emg_with_thresholds(300.0, 600.0);
        // ch0(350) >= inner(300), ch1(550) <= outer(600) -> Open
        assert_eq!(emg.process_data(vec![350.0, 550.0]).unwrap(), 1);
    }

    #[test]
    fn test_calibrate_then_process_data_close() {
        let emg = make_emg_with_thresholds(300.0, 600.0);
        // ch0(250) <= inner(300), ch1(650) >= outer(600) -> Close
        assert_eq!(emg.process_data(vec![250.0, 650.0]).unwrap(), 0);
    }

    #[test]
    fn test_calibrate_then_process_data_hold() {
        let emg = make_emg_with_thresholds(300.0, 600.0);
        assert_eq!(emg.process_data(vec![250.0, 550.0]).unwrap(), -1);
    }

    #[test]
    fn test_recalibrate_changes_outcome() {
        let mut emg = Emg::new().unwrap();
        // Default (450/450): [400, 400] → Hold
        assert_eq!(emg.process_data(vec![400.0, 400.0]).unwrap(), -1);
        // After recalibrate: ch0(400) >= inner(300), ch1(400) <= outer(450) -> Open
        emg.calibrate(300.0, 450.0).unwrap();
        assert_eq!(emg.process_data(vec![400.0, 400.0]).unwrap(), 1);
    }

    // -------------------------
    // Integration Tests: read_buffer() (non-pi / simulated)
    // -------------------------

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_read_buffer_returns_ok() {
        pyo3::prepare_freethreaded_python();
        let mut emg = Emg::new().unwrap();
        emg.configure(4).unwrap(); // small buffer to avoid slow test
        let result = emg.read_buffer();
        assert!(result.is_ok(), "read_buffer() should succeed in simulation");
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_read_buffer_returns_correct_sample_count() {
        pyo3::prepare_freethreaded_python();
        let mut emg = Emg::new().unwrap();
        emg.configure(8).unwrap();
        let samples = emg.read_buffer().unwrap();
        // buffer_size / 2 iterations × 2 channels = buffer_size samples
        assert_eq!(samples.len(), 8);
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_read_buffer_updates_inner_buffer() {
        pyo3::prepare_freethreaded_python();
        let mut emg = Emg::new().unwrap();
        emg.configure(4).unwrap();
        emg.read_buffer().unwrap();
        assert_eq!(emg.inner.buffer.len(), 4);
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_read_buffer_then_get_latest_samples_consistent() {
        pyo3::prepare_freethreaded_python();
        let mut emg = Emg::new().unwrap();
        emg.configure(4).unwrap();
        let from_read = emg.read_buffer().unwrap();
        let from_latest = emg.get_latest_samples().unwrap();
        assert_eq!(from_read, from_latest);
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_read_buffer_zero_buffer_size_returns_empty() {
        pyo3::prepare_freethreaded_python();
        let mut emg = Emg::new().unwrap();
        emg.configure(0).unwrap();
        let samples = emg.read_buffer().unwrap();
        assert!(samples.is_empty());
    }
}
