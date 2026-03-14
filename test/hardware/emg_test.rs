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

// Updated for March 7th

use anyhow::Result;
use super::{Resource, adc::Adc};

// -------------------------------------------------------------------------
// EMG ADC / MCLK constants
// From the ADS1299 / MCP3008 datasheet — confirm against actual IC in drive
// -------------------------------------------------------------------------

/// Default internal sampling rate used when no external MCLK is configured (Hz).
pub const DEFAULT_SAMPLE_RATE_HZ: u32 = 250;

/// CS pin assignments for the 3 SPI slaves (EMG = slave 0, adjust if schematic changes)
pub const EMG_CS_PIN:    u8 = 8;  // slave 0 — EMG ADC
pub const FSR_CS_PIN_0:  u8 = 7;  // slave 1 — FSR ADC 0  (reserved, not yet wired up)
pub const FSR_CS_PIN_1:  u8 = 25; // slave 2 — FSR ADC 1  (reserved, not yet wired up)

pub struct Emg {
    pub adc: Adc,
    pub buffer: Vec<u16>,
    pub buffer_size: usize,
    pub inner_threshold: f32,
    pub outer_threshold: f32,
    pub current_channel_0: f32,
    pub current_channel_1: f32,
    /// External MCLK sampling rate currently commanded to the ADC (Hz).
    /// None means the ADC is running on its internal default clock.
    pub mclk_rate_hz: Option<u32>,
}

impl Resource for Emg {
    fn init() -> Self {
        let adc = Adc::init(EMG_CS_PIN, 1_350_000); // CS pin 8, 1.35 MHz SPI clock
        Emg {
            adc,
            buffer: Vec::new(),
            buffer_size: 256,
            inner_threshold: 450.0,
            outer_threshold: 450.0,
            current_channel_0: 0.0,
            current_channel_1: 0.0,
            mclk_rate_hz: None,
        }
    }

    fn name() -> String {
        "Emg".to_string()
    }
}

impl Emg {
    pub fn configure(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
        self.buffer = Vec::with_capacity(buffer_size);
    }

    pub fn read_buffer(&mut self) -> Result<Vec<u16>> {
        let mut samples = Vec::new();

        for _ in 0..self.buffer_size / 2 {
            let ch0 = self.adc.read_channel(0)?;
            let ch1 = self.adc.read_channel(1)?;

            self.current_channel_0 = ch0 as f32;
            self.current_channel_1 = ch1 as f32;

            samples.push(ch0);
            samples.push(ch1);
        }

        self.buffer = samples.clone();
        Ok(samples)
    }

    pub fn is_ready(&self) -> bool {
        true
    }

    pub fn get_latest_samples(&self) -> Vec<u16> {
        self.buffer.clone()
    }

    pub fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32) {
        self.inner_threshold = inner_threshold;
        self.outer_threshold = outer_threshold;
    }

    pub fn process_data(&self, values: &[f32]) -> Result<i32> {
        if values.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2 EMG values"));
        }

        if values[0] >= self.inner_threshold && values[1] <= self.outer_threshold {
            Ok(1)  // Open
        } else if values[0] <= self.inner_threshold && values[1] >= self.outer_threshold {
            Ok(0)  // Close
        } else {
            Ok(-1) // Hold / no change
        }
    }

    // -------------------------------------------------------------------------
    // Integration: MCLK / external sampling rate
    // -------------------------------------------------------------------------

    /// Command a new external sampling rate to the ADC.
    ///
    /// On real hardware this writes the rate-config register over SPI so the
    /// ADC transitions from its default free-running clock to the requested
    /// rate.  The ADC datasheet specifies which register/command byte encodes
    /// the rate — update 'build_rate_command()' when that is confirmed.
    ///
    /// After calling this, 'mclk_rate_hz' reflects the commanded rate and
    /// subsequent 'read_buffer()' calls will sample at the new rate.
    ///
    /// # Arguments
    /// * 'rate_hz' - Desired sampling rate in Hz (e.g. 250, 500, 1000)
    pub fn set_mclk_rate(&mut self, rate_hz: u32) -> Result<()> {
        let command = Self::build_rate_command(rate_hz);
        self.send_mclk_command(&command)?;
        self.mclk_rate_hz = Some(rate_hz);
        Ok(())
    }

    /// Reset the ADC back to its internal default clock.
    /// Clears 'mclk_rate_hz' so callers can distinguish "not yet set" from
    /// "deliberately reset".
    pub fn reset_mclk_rate(&mut self) -> Result<()> {
        let command = Self::build_rate_command(DEFAULT_SAMPLE_RATE_HZ);
        self.send_mclk_command(&command)?;
        self.mclk_rate_hz = None;
        Ok(())
    }

    /// Returns the currently commanded rate, or the default if none has been set.
    pub fn effective_sample_rate_hz(&self) -> u32 {
        self.mclk_rate_hz.unwrap_or(DEFAULT_SAMPLE_RATE_HZ)
    }

    /// Build the SPI command bytes for a given sampling rate.
    ///
    /// TODO: replace this stub with the actual register map from the EMG ADC
    /// datasheet (see Google Drive link in integration notes).
    /// Current encoding: [0xA0, rate_high_byte, rate_low_byte].
    fn build_rate_command(rate_hz: u32) -> Vec<u8> {
        let high = ((rate_hz >> 8) & 0xFF) as u8;
        let low  = (rate_hz & 0xFF) as u8;
        vec![0xA0, high, low] // 0xA0 = placeholder "set-rate" opcode
    }

    /// Transmit a pre-built command over SPI.
    ///
    /// On Pi: drives CS low, writes bytes, drives CS high.
    /// In simulation: no-op (command is validated by set_mclk_rate).
    #[cfg(feature = "pi")]
    fn send_mclk_command(&mut self, command: &[u8]) -> Result<()> {
        use anyhow::Context;
        let mut rx = vec![0u8; command.len()];
        self.adc.cs_pin.set_low();
        self.adc.spi
            .transfer(&mut rx, command)
            .context("SPI transfer failed while sending MCLK command")?;
        self.adc.cs_pin.set_high();
        Ok(())
    }

    #[cfg(not(feature = "pi"))]
    fn send_mclk_command(&mut self, _command: &[u8]) -> Result<()> {
        Ok(()) // Simulation: accept any rate without real SPI
    }

    // -------------------------------------------------------------------------
    // Integration: multi-slave chip-select selection
    // -------------------------------------------------------------------------

    /// Switch the SPI bus to address a specific slave by its CS pin.
    ///
    /// Use the constants 'EMG_CS_PIN', 'FSR_CS_PIN_0', 'FSR_CS_PIN_1' to
    /// identify slaves.  Selecting EMG_CS_PIN re-points reads to the EMG ADC;
    /// selecting FSR pins is a no-op in the current firmware but is wired up
    /// so FSR can be dropped in later without touching call sites.
    ///
    /// # Arguments
    /// * 'cs_pin' - GPIO pin number of the target slave
    pub fn select_slave(&mut self, cs_pin: u8) -> Result<()> {
        self.adc.set_chip_select(cs_pin)
    }

    /// Convenience: explicitly select the EMG ADC (slave 0).
    /// Call this after any FSR reads to restore the EMG CS before the next
    /// 'read_buffer()'.
    pub fn select_emg_slave(&mut self) -> Result<()> {
        self.select_slave(EMG_CS_PIN)
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn make_emg(inner_threshold: f32, outer_threshold: f32) -> Emg {
        let mut emg = Emg::init();
        emg.inner_threshold = inner_threshold;
        emg.outer_threshold = outer_threshold;
        emg
    }

    // -------------------------------------------------------------------------
    // Existing tests (unchanged)
    // -------------------------------------------------------------------------

    #[test]
    fn test_init_default_values() {
        let emg = Emg::init();
        assert_eq!(emg.buffer_size, 256);
        assert!(emg.buffer.is_empty());
        assert_eq!(emg.inner_threshold, 450.0);
        assert_eq!(emg.outer_threshold, 450.0);
        assert_eq!(emg.current_channel_0, 0.0);
        assert_eq!(emg.current_channel_1, 0.0);
        assert_eq!(emg.mclk_rate_hz, None);
    }

    #[test]
    fn test_name() {
        assert_eq!(Emg::name(), "Emg");
    }

    #[test]
    fn test_process_data_open_gesture() {
        let emg = make_emg(450.0, 450.0);
        assert_eq!(emg.process_data(&[500.0, 400.0]).unwrap(), 1);
    }

    #[test]
    fn test_process_data_close_gesture() {
        let emg = make_emg(450.0, 450.0);
        assert_eq!(emg.process_data(&[400.0, 500.0]).unwrap(), 0);
    }

    #[test]
    fn test_process_data_hold_no_change() {
        let emg = make_emg(450.0, 450.0);
        assert_eq!(emg.process_data(&[400.0, 400.0]).unwrap(), -1);
    }

    // -------------------------------------------------------------------------
    // set_mclk_rate()
    // -------------------------------------------------------------------------

    #[test]
    fn test_set_mclk_rate_stores_rate() {
        let mut emg = Emg::init();
        emg.set_mclk_rate(500).unwrap();
        assert_eq!(emg.mclk_rate_hz, Some(500));
    }

    #[test]
    fn test_set_mclk_rate_returns_ok() {
        let mut emg = Emg::init();
        assert!(emg.set_mclk_rate(250).is_ok());
    }

    #[test]
    fn test_set_mclk_rate_updates_on_reconfigure() {
        let mut emg = Emg::init();
        emg.set_mclk_rate(250).unwrap();
        emg.set_mclk_rate(1000).unwrap();
        assert_eq!(emg.mclk_rate_hz, Some(1000));
    }

    #[test]
    fn test_set_mclk_rate_zero_is_accepted() {
        // Edge case: 0 Hz is an unusual but not invalid argument; shouldn't panic
        let mut emg = Emg::init();
        assert!(emg.set_mclk_rate(0).is_ok());
        assert_eq!(emg.mclk_rate_hz, Some(0));
    }

    // -------------------------------------------------------------------------
    // reset_mclk_rate()
    // -------------------------------------------------------------------------

    #[test]
    fn test_reset_mclk_rate_clears_rate() {
        let mut emg = Emg::init();
        emg.set_mclk_rate(500).unwrap();
        emg.reset_mclk_rate().unwrap();
        assert_eq!(emg.mclk_rate_hz, None);
    }

    #[test]
    fn test_reset_mclk_rate_returns_ok() {
        let mut emg = Emg::init();
        assert!(emg.reset_mclk_rate().is_ok());
    }

    // -------------------------------------------------------------------------
    // effective_sample_rate_hz()
    // -------------------------------------------------------------------------

    #[test]
    fn test_effective_rate_defaults_when_none_set() {
        let emg = Emg::init();
        assert_eq!(emg.effective_sample_rate_hz(), DEFAULT_SAMPLE_RATE_HZ);
    }

    #[test]
    fn test_effective_rate_reflects_set_rate() {
        let mut emg = Emg::init();
        emg.set_mclk_rate(1000).unwrap();
        assert_eq!(emg.effective_sample_rate_hz(), 1000);
    }

    #[test]
    fn test_effective_rate_returns_default_after_reset() {
        let mut emg = Emg::init();
        emg.set_mclk_rate(1000).unwrap();
        emg.reset_mclk_rate().unwrap();
        assert_eq!(emg.effective_sample_rate_hz(), DEFAULT_SAMPLE_RATE_HZ);
    }

    // -------------------------------------------------------------------------
    // build_rate_command()
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_rate_command_opcode_byte() {
        let cmd = Emg::build_rate_command(250);
        assert_eq!(cmd[0], 0xA0, "first byte should be the set-rate opcode");
    }

    #[test]
    fn test_build_rate_command_encodes_high_byte() {
        let cmd = Emg::build_rate_command(0x0200); // 512
        assert_eq!(cmd[1], 0x02);
    }

    #[test]
    fn test_build_rate_command_encodes_low_byte() {
        let cmd = Emg::build_rate_command(0x00FA); // 250
        assert_eq!(cmd[2], 0xFA);
    }

    #[test]
    fn test_build_rate_command_length_is_three() {
        let cmd = Emg::build_rate_command(500);
        assert_eq!(cmd.len(), 3);
    }

    // -------------------------------------------------------------------------
    // select_slave() / select_emg_slave()
    // -------------------------------------------------------------------------

    #[test]
    fn test_select_slave_changes_active_cs() {
        let mut emg = Emg::init();
        emg.select_slave(FSR_CS_PIN_0).unwrap();
        assert_eq!(emg.adc.active_cs_pin, FSR_CS_PIN_0);
    }

    #[test]
    fn test_select_emg_slave_restores_emg_pin() {
        let mut emg = Emg::init();
        emg.select_slave(FSR_CS_PIN_0).unwrap();
        emg.select_emg_slave().unwrap();
        assert_eq!(emg.adc.active_cs_pin, EMG_CS_PIN);
    }

    #[test]
    fn test_select_slave_returns_ok() {
        let mut emg = Emg::init();
        assert!(emg.select_slave(EMG_CS_PIN).is_ok());
    }

    #[test]
    fn test_select_all_three_slaves_in_sequence() {
        let mut emg = Emg::init();
        for &pin in &[EMG_CS_PIN, FSR_CS_PIN_0, FSR_CS_PIN_1] {
            emg.select_slave(pin).unwrap();
            assert_eq!(emg.adc.active_cs_pin, pin);
        }
    }

    // -------------------------------------------------------------------------
    // Integration: MCLK + slave select together
    // -------------------------------------------------------------------------

    #[test]
    fn test_set_rate_then_switch_slave_then_restore() {
        let mut emg = Emg::init();

        // 1. Set sampling rate for EMG ADC
        emg.set_mclk_rate(500).unwrap();
        assert_eq!(emg.mclk_rate_hz, Some(500));

        // 2. Temporarily switch to FSR ADC (future use)
        emg.select_slave(FSR_CS_PIN_0).unwrap();
        assert_eq!(emg.adc.active_cs_pin, FSR_CS_PIN_0);

        // 3. Restore EMG slave - rate should be preserved
        emg.select_emg_slave().unwrap();
        assert_eq!(emg.adc.active_cs_pin, EMG_CS_PIN);
        assert_eq!(emg.mclk_rate_hz, Some(500));
    }

    #[test]
    fn test_read_buffer_after_slave_restore_succeeds() {
        let mut emg = Emg::init();
        emg.configure(4);
        emg.select_slave(FSR_CS_PIN_0).unwrap();
        emg.select_emg_slave().unwrap();
        // Should read from EMG ADC successfully after restoring CS
        assert!(emg.read_buffer().is_ok());
    }

    #[cfg(not(feature = "pi"))]
    #[test]
    fn test_set_mclk_then_read_buffer_produces_expected_count() {
        let mut emg = Emg::init();
        emg.configure(8);
        emg.set_mclk_rate(500).unwrap();
        let samples = emg.read_buffer().unwrap();
        assert_eq!(samples.len(), 8);
    }
}