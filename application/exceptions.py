"""Custom exceptions for GPM application

This module defines a hierarchy of exceptions for fail-fast error handling.
All GPM-specific exceptions inherit from GpmError for easy catching.
"""


class GpmError(Exception):
    """Base exception for all GPM errors

    All GPM-specific exceptions should inherit from this class.
    This allows catching all GPM errors with a single except clause.
    """
    pass


class HardwareError(GpmError):
    """Hardware initialization or communication failure

    Raised when:
    - Hardware device fails to initialize
    - Communication with hardware fails
    - Hardware operation times out
    - Hardware returns unexpected values
    """
    pass


class SensorError(HardwareError):
    """Sensor read/write failure

    Specific subclass of HardwareError for sensor-related issues.

    Raised when:
    - EMG sensor read fails
    - FSR sensor read fails
    - BMS sensor read fails
    - Sensor calibration fails
    """
    pass


class SafetyError(GpmError):
    """Safety constraint violation

    Raised when safety checks fail and operation must be aborted.
    Contains list of specific violations for logging and telemetry.

    Attributes:
        violations (list[str]): List of safety violations that occurred
    """

    def __init__(self, violations: list[str]):
        """Initialize SafetyError with list of violations

        Args:
            violations: List of safety violation messages
        """
        self.violations = violations
        super().__init__(f"Safety violations: {', '.join(violations)}")


class StateTransitionError(GpmError):
    """Invalid state machine transition

    Raised when attempting an invalid state transition.

    Example:
        Trying to transition from ERROR state to ACTIVE without going through IDLE
    """
    pass


class ConfigurationError(GpmError):
    """Invalid configuration

    Raised when:
    - Config file is malformed
    - Required configuration keys are missing
    - Configuration values are out of valid range
    - Configuration file cannot be read
    """
    pass


class CalibrationError(GpmError):
    """Calibration failure

    Raised when:
    - EMG calibration fails (insufficient signal)
    - FSR calibration fails
    - Calibration is interrupted
    """
    pass


class RateLimitError(GpmError):
    """Rate limit exceeded

    Raised when operations are performed too rapidly,
    potentially causing hardware damage or instability.

    Example:
        Grip changes occurring faster than MIN_GRIP_INTERVAL
    """
    pass
