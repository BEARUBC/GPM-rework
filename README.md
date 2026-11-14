# GPM - Grasp Primary Module (Rework)

A hybrid Rust/Python prosthetic arm controller with hardware interfaces in Rust and control logic in Python.

## Architecture

This project restructures the original GPM codebase to separate concerns:

- **Rust Layer** (PyO3 Extension): Hardware-facing drivers for low-latency sensor/actuator control
  - Maestro servo controller
  - EMG sensor reading
  - FSR (Force Sensitive Resistor) sensors
  - BMS (Battery Management System)
  
- **Python Layer**: High-level control logic and application orchestration
  - Grip controller
  - Safety monitoring
  - State machine
  - Command sequencing

## Project Structure

```
GPM-rework/
├── src/                          # Rust hardware drivers
│   ├── lib.rs                   # PyO3 module root
│   ├── hardware/                # Hardware abstractions
│   │   ├── maestro.rs
│   │   ├── emg.rs
│   │   ├── fsr.rs
│   │   ├── bms.rs
│   │   └── adc.rs
│   └── python_bindings/         # PyO3 wrappers
│       ├── maestro.rs
│       ├── emg.rs
│       ├── fsr.rs
│       └── bms.rs
├── gpm/                         # Python type stubs
│   ├── __init__.py
│   ├── maestro.pyi
│   ├── emg.pyi
│   ├── fsr.pyi
│   └── bms.pyi
├── application/                 # Python application logic
│   ├── hardware.py             # Hardware initialization
│   ├── grip_controller.py      # Grip orchestration
│   ├── safety_monitor.py       # Safety constraints
│   ├── state_machine.py        # State management
│   └── command_sequencer.py    # Command sequencing
├── config/                      # Configuration
│   ├── config.yaml
│   └── constants.py
├── main.py                      # Application entry point
├── Cargo.toml                   # Rust dependencies
└── pyproject.toml              # Python build config
```

## Building

### Prerequisites

- Rust toolchain (rustup)
- Python 3.9+
- Maturin (`pip install maturin`)
- Protocol Buffers compiler (`protoc`), required by `gpm_original`

### ⚠️ Current Status: Transition to Submodule

This project is currently in transition to use `gpm_original` as a git submodule for hardware implementations.

**Current State:**
- Using local `src/hardware/` directory (temporary implementations)
- All Python bindings have TODO comments marking transition points
- Project can build and run independently for development

**Target State:**
- `gpm_original` as git submodule providing hardware drivers
- Python bindings import from `gpm_original::resources`
- Local `src/hardware/` removed

**Documentation:**
- [`SUBMODULE_INTEGRATION.md`](SUBMODULE_INTEGRATION.md) - Step-by-step integration guide
- [`TEAM_CHECKLIST.md`](TEAM_CHECKLIST.md) - Requirements for gpm_original team
- [`ARCHITECTURE_DIAGRAMS.md`](ARCHITECTURE_DIAGRAMS.md) - Visual architecture diagrams

**Testing Current Version:**
You can build and test now with temporary hardware:
```bash
python -m venv venv
.\venv\Scripts\activate      # Windows
source venv/bin/activate     # Linux/Mac
pip install maturin
maturin develop
python main.py demo
```

### Development Build

```bash
# Install dependencies
pip install -r requirements.txt

# Build Rust extension in development mode
maturin develop

# Or for Pi with hardware features
maturin develop --features pi
```

### Release Build

```bash
# Build wheel
maturin build --release --features pi

# Install wheel
pip install target/wheels/gpm-*.whl
```

## Running

### Demo Mode (No Hardware)

```bash
python main.py demo
```

This runs a demo sequence cycling through grip positions without requiring actual hardware.

### Normal Operation (With Hardware)

```bash
python main.py
```

Starts the EMG processing loop for real-time gesture recognition and grip control.

## Configuration

Edit `config/config.yaml` to adjust:

- Hardware parameters (CS pins, thresholds, sampling rates)
- Grip positions (PWM values for each servo)
- Safety constraints (voltage, temperature, current limits)
- Application settings (loop rate, debug mode)

### Board Profiles (current Pi vs. Pi Zero 2 W)

`config/config.yaml` is the base config for the current Raspberry Pi. To target a
different board, set `GPM_CONFIG=<name>`. `config/config.<name>.yaml` is then
deep-merged on top of the base, so a profile only needs the keys that differ.

| Board | Profile file | How to select |
|---|---|---|
| Current Raspberry Pi | `config/config.yaml` | default (leave `GPM_CONFIG` unset) |
| Pi Zero 2 W (temporary) | `config/config.pi0.yaml` | `GPM_CONFIG=pi0` |

```bash
# Current Pi
python main.py

# Pi Zero 2 W
GPM_CONFIG=pi0 python main.py
GPM_CONFIG=pi0 python main.py demo
```

If `GPM_CONFIG` names a profile with no matching file, startup fails with an error
instead of silently using the base config. To add a board, create
`config/config.<name>.yaml` with only the overridden keys, and set `hardware.board`
so it's clear which profile is active.

#### Pi Zero 2 W pin mapping

The Zero 2 W uses the same 40-pin header and BCM numbering as the full-size Pi,
so the pin numbers don't change; only the board setup and build target do.

| Signal | BCM GPIO | Physical pin |
|---|---|---|
| SPI0 MOSI | 10 | 19 |
| SPI0 MISO | 9 | 21 |
| SPI0 SCLK | 11 | 23 |
| EMG ADC CS (CE0) | 8 | 24 |
| FSR ADC CS (CE1) | 7 | 26 |
| Maestro UART TX → Maestro RX | 14 | 8 |
| Maestro UART RX ← Maestro TX | 15 | 10 |
| MCP3008 VDD/VREF (3.3V only) | — | 1 |
| GND | — | 6 |

Power the servos from a separate supply into the Maestro (never from the Zero's 5V
pin), and connect all grounds together.

#### Pi Zero 2 W device setup: the Maestro gets the stronger UART

The Zero 2 W has two UARTs:

- **PL011 (`ttyAMA0`)**: a full UART with its own stable clock.
- **Mini-UART (`ttyS0`)**: its baud rate comes from the GPU core clock, so it can
  drift when the clock scales or the board throttles. Drift garbles serial bytes
  and causes missed or wrong servo commands.

By default the PL011 is given to Bluetooth, and `/dev/serial0` (the port the
Maestro driver opens on GPIO 14/15) ends up on the mini-UART. **This setup
disables Bluetooth so the stronger PL011 UART goes to the Maestro.** Add to
`/boot/firmware/config.txt`:

```
dtparam=spi=on
enable_uart=1
dtoverlay=disable-bt
```

Then:

```bash
sudo systemctl disable hciuart
sudo raspi-config   # Interface Options → Serial Port: login shell NO, hardware port YES
sudo reboot

# Verify
ls -l /dev/serial0          # must point to ttyAMA0, not ttyS0
ls /dev/spidev0.*           # expect spidev0.0 and spidev0.1
```

#### Building for the Pi Zero 2 W

The Zero 2 W is aarch64 with 512 MB RAM. Compiling on the device can run out of
memory, so cross-compile from a dev machine:

```bash
rustup target add aarch64-unknown-linux-gnu
pip install ziglang
PI0_PYTHON=python3.11 ./build.sh release-pi0   # match the device's Python version
scp target/wheels/gpm-*.whl <user>@<zero-host>:~
# on the Zero, inside a venv:
pip install ~/gpm-*.whl
```

If you'd rather build on the device, raise swap to about 1 GB first
(`/etc/dphys-swapfile`).

## Key Features

### Direct Hardware Access
- No IPC overhead - Python calls Rust hardware functions directly
- Sub-millisecond latency for servo control
- Efficient sensor buffering

### Safety First
- Continuous battery monitoring
- Temperature and current limits
- Pre-execution safety checks
- Graceful error handling

### State Management
- Formal state machine with validated transitions
- Error state tracking
- Operational state checks

### Command Sequencing
- Multi-step command orchestration
- Configurable delays between steps
- Sequence registration and reuse

## Development

### Type Checking

Type stubs (`.pyi` files) provide IDE autocomplete and type checking for the Rust extension:

```python
from gpm import Maestro

maestro = Maestro()  # IDE knows the signature
maestro.set_target(0, 1500)  # IDE provides parameter hints
```

### Testing

The Python tests in `tests/` cover the state machine, grip controller, and safety
monitor using mock devices. Build the Rust library first so the application can
import the real class definitions; the tests do not require physical hardware.

```bash
# In an activated virtual environment, build without the pi feature
maturin develop --extras dev
python -m pytest tests/ -q

# With coverage
python -m pytest tests/ --cov=application --cov-report=html
```

The `gpm` Python package loads its compiled Rust library as `gpm._native`.
Application code continues to use imports such as `from gpm import Maestro`.

### Mock Hardware

The Rust layer includes mock implementations when built without the `pi` feature, allowing development and testing on non-Pi hardware.

## Performance

- EMG sampling: 1000 Hz
- Control loop: 100 Hz (configurable)
- Servo command latency: <1ms
- Hardware call overhead: <100μs

## Safety Constraints

Default limits (configurable in `config.yaml`):

- Critical voltage: 7.0V
- Max temperature: 60°C
- Max current: 10.0A
- Min charge: 10%

## Original vs Rework

**Original Architecture:**
- Monolithic Rust application
- Manager pattern with MPSC channels
- TCP dispatchers for remote control
- Protocol Buffers serialization

**Rework Architecture:**
- Hybrid Rust/Python
- Direct function calls (no channels)
- Python control logic
- Simpler, more maintainable

## License

[Your License Here]

## Contributors

UBC Bionics Team 
