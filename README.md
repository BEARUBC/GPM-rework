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

```bash
# Run tests
pytest

# With coverage
pytest --cov=application --cov-report=html
```

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
