# Architecture Diagrams

## Current Architecture (Temporary)

```
┌─────────────────────────────────────────────────────────────┐
│                     GPM-rework Project                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Python Application Layer                     │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  main.py                                       │  │  │
│  │  │  - Entry point                                 │  │  │
│  │  │  - Orchestrates control loop                   │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  application/                                  │  │  │
│  │  │  - grip_controller.py                          │  │  │
│  │  │  - safety_monitor.py                           │  │  │
│  │  │  - state_machine.py                            │  │  │
│  │  │  - command_sequencer.py                        │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │ Direct calls                          │
│                     ↓                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │      Python Extension (Rust compiled to .so/.pyd)    │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  gpm module (from Python)                      │  │  │
│  │  │  - from gpm import Maestro, Emg, Fsr, Bms     │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  src/python_bindings/ (Rust + PyO3)           │  │  │
│  │  │  - maestro.rs  (#[pyclass])                    │  │  │
│  │  │  - emg.rs      (#[pyclass])                    │  │  │
│  │  │  - fsr.rs      (#[pyclass])                    │  │  │
│  │  │  - bms.rs      (#[pyclass])                    │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │ Wraps                                 │
│                     ↓                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  src/hardware/ (Rust) ← TEMPORARY                   │  │
│  │  - maestro.rs (servo control)                       │  │
│  │  - emg.rs (EMG sensors)                             │  │
│  │  - fsr.rs (force sensors)                           │  │
│  │  - bms.rs (battery)                                 │  │
│  │  - adc.rs (ADC interface)                           │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │ Hardware access                       │
│                     ↓                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Hardware (SPI, GPIO, Serial)                 │  │
│  │  - Maestro servo controller                          │  │
│  │  - EMG sensors (MCP3008)                             │  │
│  │  - FSR sensors (MCP3008)                             │  │
│  │  - Battery monitoring                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Target Architecture (After Submodule Integration)

```
┌─────────────────────────────────────────────────────────────┐
│                     GPM-rework Project                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         Python Application Layer                     │  │
│  │  (same as before - no changes needed)               │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                        │
│                     ↓                                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │      Python Extension (Rust compiled)                │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  src/python_bindings/ (PyO3 wrappers)          │  │  │
│  │  │  - maestro.rs                                   │  │  │
│  │  │  - emg.rs                                       │  │  │
│  │  │  - fsr.rs                                       │  │  │
│  │  │  - bms.rs                                       │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │ Imports from:                         │
│                     ↓                                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐  │
│  │       gpm_original/ (Git Submodule)                  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │  src/resources/ (Rust hardware drivers)        │  │  │
│  │  │  - maestro.rs                                   │  │  │
│  │  │  - emg.rs                                       │  │  │
│  │  │  - fsr.rs                                       │  │  │
│  │  │  - bms.rs                                       │  │  │
│  │  │  - common/adc.rs                                │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │                                                        │  │
│  │  (Other parts of gpm_original not used:)              │  │
│  │  - managers/ (not needed)                             │  │
│  │  - dispatchers/ (not needed)                          │  │
│  │  - exporters/ (not needed)                            │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                        │
└─────────────────────┼────────────────────────────────────────┘
                      │
                      ↓
         ┌────────────────────────┐
         │  Hardware (Raspberry Pi)│
         └────────────────────────┘
```

## Data Flow Example: Moving a Servo

```
Python Code (main.py)
    │
    │ controller.grip_controller.execute_grip(GripType.PINCH)
    ↓
Python Layer (grip_controller.py)
    │
    │ self.hardware.maestro.move_to_grip("pinch")
    ↓
──────────────────────────────────────────────
Python/Rust Boundary (PyO3)
──────────────────────────────────────────────
    ↓
PyO3 Wrapper (python_bindings/maestro.rs)
    │
    │ #[pymethods] pub fn move_to_grip(&mut self, grip_type: &str)
    │ self.inner.move_to_grip(grip_type)
    ↓
Hardware Driver (gpm_original/src/resources/maestro.rs)
    │
    │ pub fn move_to_grip(&mut self, grip_type: &str)
    │ self.set_target(0, 2000)
    │ self.set_target(1, 1800)
    │ ...
    ↓
Hardware Library (raestro crate)
    │
    │ controller.set_target(channel, pwm)
    ↓
Serial Port (/dev/ttyACM0)
    │
    │ [0x84, channel, pwm_low, pwm_high]
    ↓
Maestro Servo Controller (Hardware)
    │
    │ Moves servo to position
    ↓
Servo Motor (Physical Movement)
```

## Comparison: Original vs Rework

### Original Architecture (gpm_original)
```
┌─────────────────────────────────┐
│    Rust Binary Application      │
│                                  │
│  ┌────────────────────────────┐ │
│  │  main.rs                   │ │
│  │  - Tokio runtime           │ │
│  │  - Async task spawning     │ │
│  └────────────────────────────┘ │
│            │                     │
│            ↓                     │
│  ┌────────────────────────────┐ │
│  │  Managers (async)          │ │
│  │  - maestro_manager         │ │
│  │  - emg_manager             │ │
│  │  - fsr_manager             │ │
│  │  (MPSC channels between)   │ │
│  └────────────────────────────┘ │
│            │                     │
│            ↓                     │
│  ┌────────────────────────────┐ │
│  │  Dispatchers               │ │
│  │  - TCP dispatcher          │ │
│  │  - GPIO dispatcher         │ │
│  │  - Bio-signal dispatcher   │ │
│  └────────────────────────────┘ │
│            │                     │
│            ↓                     │
│  ┌────────────────────────────┐ │
│  │  Resources (hardware)      │ │
│  │  - maestro.rs              │ │
│  │  - emg.rs                  │ │
│  │  - fsr.rs                  │ │
│  └────────────────────────────┘ │
└─────────────────────────────────┘

Latency: ~1-10ms (channel overhead)
Control Logic: Hard-coded in Rust
Flexibility: Low (recompile for changes)
```

### Rework Architecture (GPM-rework)
```
┌─────────────────────────────────┐
│   Python Application             │
│                                  │
│  ┌────────────────────────────┐ │
│  │  main.py                   │ │
│  │  - Simple event loop       │ │
│  └────────────────────────────┘ │
│            │                     │
│            ↓                     │
│  ┌────────────────────────────┐ │
│  │  Application Layer         │ │
│  │  - grip_controller.py      │ │
│  │  - safety_monitor.py       │ │
│  │  - state_machine.py        │ │
│  │  (Direct function calls)   │ │
│  └────────────────────────────┘ │
│            │                     │
│            ↓                     │
├─────────────────────────────────┤
│  Rust Extension (.so)           │
│  ┌────────────────────────────┐ │
│  │  Python Bindings (PyO3)    │ │
│  │  - #[pyclass] wrappers     │ │
│  └────────────────────────────┘ │
│            │                     │
│            ↓                     │
│  ┌────────────────────────────┐ │
│  │  Hardware Drivers          │ │
│  │  (from gpm_original)       │ │
│  └────────────────────────────┘ │
└─────────────────────────────────┘

Latency: <100μs (direct calls)
Control Logic: Flexible Python
Flexibility: High (no recompile)
```

## Module Dependencies

```
Python Packages
    ↓
┌─────────────────────┐
│  gpm (Python import)│  ← Compiled from Rust
└─────────────────────┘
    ↓ depends on
┌─────────────────────┐
│  gpm_original       │  ← Git submodule or path dependency
│  (Rust crate)       │
└─────────────────────┘
    ↓ depends on
┌─────────────────────┐
│  raestro            │  ← Servo library from crates.io
│  rppal              │  ← Raspberry Pi GPIO library
│  spidev             │  ← SPI interface
└─────────────────────┘
```

## File Structure Comparison

### Before Integration
```
GPM-rework/
├── src/
│   ├── lib.rs
│   ├── hardware/          ← Will be removed
│   │   ├── mod.rs
│   │   ├── maestro.rs
│   │   ├── emg.rs
│   │   ├── fsr.rs
│   │   ├── bms.rs
│   │   └── adc.rs
│   └── python_bindings/   ← Stays
│       ├── mod.rs
│       ├── maestro.rs
│       ├── emg.rs
│       ├── fsr.rs
│       └── bms.rs
└── application/           ← Stays
    └── *.py
```

### After Integration
```
GPM-rework/
├── gpm_original/          ← New submodule
│   └── src/
│       └── resources/
│           ├── maestro.rs
│           ├── emg.rs
│           ├── fsr.rs
│           └── bms.rs
├── src/
│   ├── lib.rs
│   └── python_bindings/
│       ├── mod.rs
│       ├── maestro.rs     ← Updated imports
│       ├── emg.rs         ← Updated imports
│       ├── fsr.rs         ← Updated imports
│       └── bms.rs         ← Updated imports
└── application/
    └── *.py
```

## Key Insight

**The Python bindings are just thin wrappers:**

```rust
// python_bindings/maestro.rs
#[pyclass]
pub struct Maestro {
    inner: RustMaestro,  // ← The actual hardware code
}

#[pymethods]
impl Maestro {
    pub fn set_target(&mut self, ch: u8, pwm: u16) -> PyResult<()> {
        self.inner.set_target(ch, pwm)  // ← Just forwards the call
            .map_err(|e| /* error conversion */)
    }
}
```

**All the real work happens in the hardware drivers from gpm_original!**
