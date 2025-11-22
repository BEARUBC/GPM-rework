# GPM Quick Start Guide

## Installation

### 1. Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dependencies
pip install -r requirements.txt
```

### 2. Build the Project

**For development (mock hardware):**
```bash
# Windows
.\build.ps1

# Linux/Mac
./build.sh
```

**For Raspberry Pi (with hardware):**
```bash
# Windows
.\build.ps1 pi

# Linux/Mac
./build.sh pi
```

## Running

### Demo Mode (No Hardware Required)

```bash
python main.py demo
```

This will:
1. Initialize the system (with mock hardware if not on Pi)
2. Run through a sequence of grip positions:
   - Open hand
   - Pinch grip
   - Power grip
   - Rest position
3. Print status for each step

### Normal Operation (EMG Control)

```bash
python main.py
```

This starts the main control loop which:
1. Reads EMG sensor data
2. Classifies gestures (open/close)
3. Executes corresponding grips
4. Monitors safety constraints

Press `Ctrl+C` to stop gracefully.

### CLI Tools

**Check system status:**
```bash
python ui/cli.py status
```

**Run demo via CLI:**
```bash
python ui/cli.py demo
```

**Start normal operation:**
```bash
python ui/cli.py run
```

### Web Dashboard (Optional)

```bash
# Install Flask first
pip install flask

# Start dashboard
python ui/web_dashboard.py
```

Then open http://localhost:5000 in your browser.

## Configuration

Edit `config/config.yaml` to customize:

### Hardware Settings

```yaml
hardware:
  emg:
    buffer_size: 256
    cs_pin: 8
    inner_threshold: 450.0
    outer_threshold: 450.0
  
  fsr:
    cs_pins: [7]
    at_rest_threshold: 900
    pressure_threshold: 500
```

### Grip Positions

```yaml
grip_positions:
  pinch:
    - 2000  # Channel 0 PWM
    - 1800  # Channel 1 PWM
    # ... etc
```

### Safety Limits

```yaml
safety:
  critical_voltage: 7.0
  max_temperature: 60.0
  max_current: 10.0
```

## Troubleshooting

### Build Errors

**"maturin not found":**
```bash
pip install maturin
```

**Rust compilation errors:**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
maturin develop
```

### Runtime Errors

**Import error: "No module named 'gpm'":**
```bash
# Make sure you've built the extension
maturin develop
```

**"yaml" import error:**
```bash
pip install pyyaml
```

**BMS/EMG read errors (on Pi):**
- Check SPI is enabled: `sudo raspi-config` → Interface Options → SPI
- Check pin connections
- Verify CS pins in config.yaml match hardware

### Performance Issues

**Slow control loop:**
- Check `control_loop_rate` in config.yaml
- Reduce EMG `buffer_size` if needed
- Profile with debug mode: edit config.yaml → `debug_mode: true`

## Development

### Project Structure

```
GPM-rework/
├── src/              # Rust hardware drivers
├── application/      # Python control logic
├── config/          # Configuration files
├── gpm/             # Python type stubs
├── ui/              # User interfaces
└── main.py          # Entry point
```

### Adding New Grip Types

1. Update `config/config.yaml`:
```yaml
grip_positions:
  my_grip:
    - 1800
    - 1900
    # ... PWM values for all channels
```

2. Update `application/grip_controller.py`:
```python
class GripType(Enum):
    # ... existing
    MY_GRIP = "my_grip"
```

3. The Rust layer automatically supports it via the `move_to_grip()` function.

### Running Tests

```bash
# Install test dependencies
pip install pytest pytest-cov

# Run tests
pytest

# With coverage
pytest --cov=application
```

### Type Checking

The project includes `.pyi` stub files for IDE support:

```python
from gpm import Maestro  # IDE provides autocomplete
maestro = Maestro()      # Type hints available
```

## Next Steps

1. **Calibrate EMG thresholds** for your specific sensors
2. **Tune grip positions** for your servo configuration
3. **Adjust safety limits** based on your battery specifications
4. **Test on actual hardware** progressively (start with just servos, then add sensors)

## Support

- See [README.md](README.md) for architecture details
- Check [UBC Bionics Embedded GPM Rework Doc.txt](UBC%20Bionics%20Embedded%20GPM%20Rework%20Doc.txt) for design decisions
- Review original code in `gpm_original/` for reference

## Safety Notes

⚠️ **Always** test with low battery voltage first
⚠️ Verify servo limits before full power operation
⚠️ Keep emergency stop accessible
⚠️ Monitor temperature during extended use
