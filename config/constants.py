"""Configuration constants for GPM application"""
import os
import yaml
from pathlib import Path

# Get config file path
CONFIG_DIR = Path(__file__).parent
CONFIG_FILE = CONFIG_DIR / "config.yaml"

# Board profile: GPM_CONFIG=<name> overlays config/config.<name>.yaml on top of
# config.yaml (e.g. GPM_CONFIG=pi0 -> config.pi0.yaml). Unset = base config only.
CONFIG_PROFILE = os.environ.get('GPM_CONFIG', '').strip() or None


def _deep_merge(base, override):
    """Recursively merge override into base; override values win."""
    merged = dict(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(merged.get(key), dict):
            merged[key] = _deep_merge(merged[key], value)
        else:
            merged[key] = value
    return merged


def _read_yaml(path):
    with open(path, 'r') as f:
        return yaml.safe_load(f) or {}


# Load configuration
def load_config(profile=CONFIG_PROFILE):
    """Load config.yaml, then overlay config.<profile>.yaml if a profile is set"""
    if CONFIG_FILE.exists():
        config = _read_yaml(CONFIG_FILE)
    else:
        print(f"Warning: Config file not found at {CONFIG_FILE}, using defaults")
        config = {}

    if profile:
        profile_file = CONFIG_DIR / f"config.{profile}.yaml"
        if not profile_file.exists():
            raise FileNotFoundError(
                f"GPM_CONFIG={profile} but profile file not found: {profile_file}"
            )
        config = _deep_merge(config, _read_yaml(profile_file))

    return config

CONFIG = load_config()

# Board the active config targets (informational, shown in logs)
BOARD = CONFIG.get('hardware', {}).get('board', 'raspberry-pi')

# Hardware Configuration
HARDWARE_CONFIG = {
    'emg_buffer_size': CONFIG.get('hardware', {}).get('emg', {}).get('buffer_size', 256),
    'emg_cs_pin': CONFIG.get('hardware', {}).get('emg', {}).get('pins', {}).get('cs'),
    'emg_clock_speed': CONFIG.get('hardware', {}).get('emg', {}).get('clock_speed', 1350000),
    'emg_inner_threshold': CONFIG.get('hardware', {}).get('emg', {}).get('inner_threshold', 450.0),
    'emg_outer_threshold': CONFIG.get('hardware', {}).get('emg', {}).get('outer_threshold', 450.0),
    'emg_sample_rate': CONFIG.get('hardware', {}).get('emg', {}).get('sample_rate', 1000),
    'emg_spi': CONFIG.get('definitions', {}).get('spi', {}).get(CONFIG.get('hardware', {}).get('emg', {}).get('spi'), {}).get('bus'),
    
    'fsr_cs_pin': CONFIG.get('hardware', {}).get('fsr', {}).get('pins', {}).get('cs'),
    'fsr_at_rest_threshold': CONFIG.get('hardware', {}).get('fsr', {}).get('at_rest_threshold', 900),
    'fsr_pressure_threshold': CONFIG.get('hardware', {}).get('fsr', {}).get('pressure_threshold', 500),
    'fsr_clock_speed': CONFIG.get('hardware', {}).get('fsr', {}).get('clock_speed', 1350000),
    'fsr_spi': CONFIG.get('definitions', {}).get('spi', {}).get(CONFIG.get('hardware', {}).get('fsr', {}).get('spi'), {}).get('bus'),
    
    'maestro_baudrate': CONFIG.get('hardware', {}).get('maestro', {}).get('baudrate', 115200),
    'maestro_num_channels': CONFIG.get('hardware', {}).get('maestro', {}).get('num_channels', 6),
    
    'bms_update_interval': CONFIG.get('hardware', {}).get('bms', {}).get('update_interval', 1.0),
}

# Grip Positions
GRIP_POSITIONS = CONFIG.get('grip_positions', {
    'rest': [1500] * 6,
    'pinch': [2000, 1800, 1500, 1500, 1500, 1500],
    'power': [2200, 2200, 2200, 2000, 2000, 2000],
    'open': [1000] * 6,
})

# Safety Constraints
SAFETY_CONFIG = {
    'critical_voltage': CONFIG.get('safety', {}).get('critical_voltage', 7.0),
    'max_temperature': CONFIG.get('safety', {}).get('max_temperature', 60.0),
    'max_current': CONFIG.get('safety', {}).get('max_current', 10.0),
    'min_charge_percentage': CONFIG.get('safety', {}).get('min_charge_percentage', 10.0),
}

# Application Settings
APP_CONFIG = {
    'control_loop_rate': CONFIG.get('application', {}).get('control_loop_rate', 100),
    'gesture_hold_time': CONFIG.get('application', {}).get('gesture_hold_time', 5),
    'debug_mode': CONFIG.get('application', {}).get('debug_mode', False),
    'log_level': CONFIG.get('application', {}).get('log_level', 'INFO'),
}

# Derived constants
CONTROL_LOOP_PERIOD = 1.0 / APP_CONFIG['control_loop_rate']  # seconds

# Gesture types
GESTURE_OPEN = 1
GESTURE_CLOSE = 0
GESTURE_HOLD = -1
