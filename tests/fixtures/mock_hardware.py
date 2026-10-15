"""Mock hardware implementations for testing without physical hardware

These mock classes simulate the behavior of real hardware (Maestro, EMG, FSR, BMS)
for development and testing purposes. They provide the same interface as the real
PyO3 bindings but with controllable, predictable behavior.
"""
import random
from types import SimpleNamespace


class MockMaestro:
    """Mock Maestro servo controller

    Simulates a 6-channel servo controller with PWM values.
    Tracks call history for test assertions.
    """

    def __init__(self):
        """Initialize mock Maestro with default positions"""
        self.channels = {i: 1500 for i in range(6)}  # Default center position
        self.call_history = []

    def set_target(self, channel: int, pwm_value: int):
        """Set target PWM for a servo channel

        Args:
            channel: Servo channel (0-5)
            pwm_value: PWM value (typically 1000-2000)
        """
        self.call_history.append(('set_target', channel, pwm_value))
        if 0 <= channel < 6:
            self.channels[channel] = pwm_value
        else:
            raise ValueError(f"Channel {channel} out of range (0-5)")

    def move_to_grip(self, grip_type: str):
        """Move to predefined grip type

        Args:
            grip_type: One of "rest", "pinch", "power", "open"
        """
        self.call_history.append(('move_to_grip', grip_type))

        # Simulate grip positions
        positions = {
            'rest': 1000,
            'pinch': 1300,
            'power': 1700,
            'open': 2000,
        }

        if grip_type not in positions:
            raise ValueError(f"Unknown grip type: {grip_type}")

        target_pwm = positions[grip_type]
        for channel in self.channels:
            self.channels[channel] = target_pwm

    def current_pwm(self, channel: int) -> int:
        """Get current PWM value for channel

        Args:
            channel: Servo channel (0-5)

        Returns:
            Current PWM value
        """
        if 0 <= channel < 6:
            return self.channels[channel]
        else:
            raise ValueError(f"Channel {channel} out of range (0-5)")


class MockEmg:
    """Mock EMG sensor

    Simulates dual-channel EMG sensor with realistic noise and spikes.
    Useful for testing gesture detection without physical sensors.
    """

    def __init__(self):
        """Initialize mock EMG with simulated data"""
        self.buffer_size = 256
        self.sample_index = 0
        self.is_configured = False
        self.inner_threshold = 700.0
        self.outer_threshold = 700.0

        # Generate realistic EMG-like data (noise + occasional spikes)
        self.samples = [500 + random.randint(-50, 50) for _ in range(1000)]

        # Add some spikes for gesture simulation (every 100 samples)
        for i in range(0, 1000, 100):
            for j in range(20):
                if i + j < 1000:
                    self.samples[i+j] = 800  # Simulate muscle activation

    def configure(self, buffer_size: int):
        """Configure EMG buffer size

        Args:
            buffer_size: Size of the sample buffer
        """
        self.buffer_size = buffer_size
        self.is_configured = True

    def is_ready(self) -> bool:
        """Check if EMG is ready to read

        Returns:
            True if configured and ready
        """
        return self.is_configured

    def read_buffer(self):
        """Return next chunk of EMG samples

        Returns:
            List of sample values (simulated)
        """
        if not self.is_configured:
            raise RuntimeError("EMG not configured")

        chunk_size = 32
        chunk = self.samples[self.sample_index:self.sample_index + chunk_size]

        # Wrap around if we reach the end
        self.sample_index = (self.sample_index + chunk_size) % len(self.samples)

        return chunk

    def calibrate(self, inner_threshold: float, outer_threshold: float):
        """Calibrate EMG thresholds

        Args:
            inner_threshold: Threshold for inner muscle
            outer_threshold: Threshold for outer muscle
        """
        self.inner_threshold = inner_threshold
        self.outer_threshold = outer_threshold

    def process_data(self) -> str:
        """Process EMG data and return gesture

        Returns:
            Gesture string: "open", "close", or "idle"
        """
        if not self.is_configured:
            return "idle"

        # Simple mock logic based on recent samples
        recent = self.samples[self.sample_index:self.sample_index + 10]
        avg = sum(recent) / len(recent) if recent else 500

        if avg > self.inner_threshold:
            return "close"
        elif avg < 400:
            return "open"
        else:
            return "idle"


class MockFsr:
    """Mock FSR pressure sensors

    Simulates 8-channel force-sensitive resistor array.
    """

    def __init__(self):
        """Initialize mock FSR with default readings"""
        self.channels = 8
        self.readings = [900] * self.channels  # At rest (high resistance)
        self.at_rest_threshold = 900
        self.pressure_threshold = 500

    def configure(self, cs_pins, at_rest_threshold: int, pressure_threshold: int):
        """Configure FSR thresholds

        Args:
            cs_pins: List of CS pin numbers (ignored in mock)
            at_rest_threshold: Threshold for no pressure
            pressure_threshold: Threshold for significant pressure
        """
        self.at_rest_threshold = at_rest_threshold
        self.pressure_threshold = pressure_threshold

    def read_all(self):
        """Return FSR reading for all channels

        Returns:
            FsrReading object with channels and pressure_detected
        """
        pressure_detected = any(r < self.pressure_threshold for r in self.readings)

        return SimpleNamespace(
            channels=self.readings.copy(),
            pressure_detected=pressure_detected
        )

    def process_data(self) -> bool:
        """Process FSR data and return vibrate state

        Returns:
            True if pressure detected
        """
        reading = self.read_all()
        return reading.pressure_detected

    def simulate_pressure(self, channel: int, value: int):
        """Helper for tests: simulate pressure on a channel

        Args:
            channel: Channel to modify (0-7)
            value: Simulated pressure value
        """
        if 0 <= channel < self.channels:
            self.readings[channel] = value
        else:
            raise ValueError(f"Channel {channel} out of range (0-7)")


class MockBms:
    """Mock battery management system

    Simulates battery monitoring with realistic values.
    """

    def __init__(self):
        """Initialize mock BMS with healthy battery state"""
        self.voltage = 8.4  # Healthy 2S LiPo (fully charged)
        self.current = 0.5  # Moderate draw
        self.temperature = 25.0  # Room temperature

    def get_status(self):
        """Return BMS status

        Returns:
            BmsStatus object with voltage, current, temperature, health
        """
        # Determine health based on voltage
        is_healthy = (
            7.0 <= self.voltage <= 8.4 and
            self.current < 10.0 and
            self.temperature < 60.0
        )

        # Calculate charge percentage (assuming 2S LiPo: 6.0V empty, 8.4V full)
        charge_percentage = ((self.voltage - 6.0) / (8.4 - 6.0)) * 100.0
        charge_percentage = max(0.0, min(100.0, charge_percentage))

        return SimpleNamespace(
            voltage=self.voltage,
            current=self.current,
            temperature=self.temperature,
            is_healthy=is_healthy,
            charge_percentage=charge_percentage
        )

    def update(self):
        """Update BMS readings (mock does nothing)"""
        pass

    def simulate_low_battery(self):
        """Helper for tests: simulate low battery condition"""
        self.voltage = 6.5  # Below safe threshold

    def simulate_high_temperature(self):
        """Helper for tests: simulate overheating"""
        self.temperature = 65.0  # Above safe threshold

    def simulate_high_current(self):
        """Helper for tests: simulate overcurrent"""
        self.current = 12.0  # Above safe threshold


class MockHardwareInterface:
    """Mock version of HardwareInterface for testing

    Provides the same interface as the real HardwareInterface but
    uses mock hardware for testing without physical devices.
    """

    def __init__(self, config=None):
        """Initialize mock hardware interface

        Args:
            config: Configuration dict (optional)
        """
        self.maestro = MockMaestro()
        self.emg = MockEmg()
        self.fsr = MockFsr()
        self.bms = MockBms()
        self.config = config or {}

    def initialize(self):
        """Initialize all mock hardware with config"""
        # Configure EMG
        emg_buffer_size = self.config.get('emg_buffer_size', 256)
        self.emg.configure(emg_buffer_size)

        # Configure FSR
        if 'fsr_cs_pins' in self.config:
            self.fsr.configure(
                self.config['fsr_cs_pins'],
                self.config.get('fsr_at_rest_threshold', 900),
                self.config.get('fsr_pressure_threshold', 500)
            )

        # Calibrate EMG
        if 'emg_inner_threshold' in self.config and 'emg_outer_threshold' in self.config:
            self.emg.calibrate(
                self.config['emg_inner_threshold'],
                self.config['emg_outer_threshold']
            )

    def get_status(self) -> dict:
        """Quick health check of all mock hardware

        Returns:
            Status dict with BMS and EMG status
        """
        bms_status = self.bms.get_status()
        return {
            'bms': {
                'voltage': bms_status.voltage,
                'current': bms_status.current,
                'temperature': bms_status.temperature,
                'is_healthy': bms_status.is_healthy,
                'charge_percentage': bms_status.charge_percentage,
            },
            'emg_ready': self.emg.is_ready(),
        }

    def shutdown(self):
        """Cleanup mock hardware resources (no-op for mocks)"""
        # Move servos to rest position
        try:
            self.maestro.move_to_grip("rest")
        except Exception:
            pass  # Mock cleanup failure is not critical
