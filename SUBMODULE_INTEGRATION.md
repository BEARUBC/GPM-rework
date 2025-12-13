# gpm_original Submodule Integration Guide

## Current State
- Temporary local hardware in `src/hardware/` (to be removed)
- Python bindings in `src/python_bindings/` (will stay)
- Python application in `application/` (will stay)

## Integration Steps (for Rust team)

### 1. Add gpm_original as submodule

```bash
git submodule add <repo-url> gpm_original
git submodule update --init --recursive
```

### 2. Uncomment Cargo.toml dependency

Edit `Cargo.toml` line 12:

```toml
# Before:
# gpm_original = { path = "./gpm_original", features = ["python"] }

# After:
gpm_original = { path = "./gpm_original", features = ["python"] }
```

### 3. Update src/lib.rs

Edit `src/lib.rs` line 20:

```rust
// Before:
mod hardware;

// After:
// (remove line entirely, use gpm_original instead)
```

### 4. Update python_bindings imports

Each file in `src/python_bindings/` has TODO comments marking what needs to change.

**Example** (`src/python_bindings/maestro.rs`):

```rust
// Before:
use crate::hardware::maestro::Maestro as RustMaestro;
use crate::hardware::Resource;

// After:
use gpm_original::resources::maestro::Maestro as RustMaestro;
use gpm_original::resources::Resource;
```

Apply similar changes to:
- `src/python_bindings/emg.rs`
- `src/python_bindings/fsr.rs`
- `src/python_bindings/bms.rs`

### 5. Remove src/hardware/ directory

```bash
git rm -r src/hardware/
git commit -m "Remove temporary hardware implementations - now using gpm_original"
```

### 6. Build and verify

```bash
# Build the Python extension
maturin develop

# Test that imports work
python -c "import gpm; print('Success!')"

# Test instantiation
python -c "from gpm import Maestro, Emg, Bms, Fsr; m = Maestro(); print('Hardware interfaces loaded')"
```

---

## Required Exports from gpm_original

The gpm_original submodule must expose the following modules and types for Python bindings to work:

### Resource Modules

```rust
// gpm_original/src/lib.rs or equivalent
pub mod resources {
    pub mod maestro;
    pub mod emg;
    pub mod fsr;
    pub mod bms;
}
```

### Resource Trait

```rust
pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
```

### Hardware Implementations

Each resource must implement specific methods that match the current temporary implementations:

#### Maestro
```rust
impl Maestro {
    fn set_target(&mut self, channel: u8, pwm_value: u16) -> anyhow::Result<()>;
    fn move_to_grip(&mut self, grip_type: &str) -> anyhow::Result<()>;
    fn current_pwm(&self, channel: u8) -> anyhow::Result<u16>;
}
```

#### EMG
```rust
impl Emg {
    fn configure(&mut self, buffer_size: usize);
    fn is_ready(&self) -> bool;
    fn read_buffer(&mut self) -> Vec<u16>;
    fn calibrate(&mut self, inner_threshold: f32, outer_threshold: f32);
    fn process_data(&mut self) -> String; // Returns "open", "close", or "idle"
}
```

#### FSR
```rust
impl Fsr {
    fn configure(&mut self, cs_pins: Vec<u8>, at_rest_threshold: u16, pressure_threshold: u16);
    fn read_all(&mut self) -> FsrReading;
    fn process_data(&mut self) -> bool; // Returns vibrate state
}

pub struct FsrReading {
    pub channels: Vec<u16>,
    pub pressure_detected: bool,
}
```

#### BMS
```rust
impl Bms {
    fn get_status(&self) -> BmsStatus;
    fn update(&mut self);
}

pub struct BmsStatus {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
    pub charge_percentage: f32,
}
```

---

## Manager Pattern Integration (Future)

Once gpm_original provides manager-based communication, add:

```rust
// gpm_original should export:
pub mod managers {
    pub use Manager;
    pub use ManagerChannelData;
    pub use ManagerChannelMap;
}
```

The Python team will then implement bindings in `src/python_bindings/manager.rs` to expose MPSC channel communication to Python.

See `docs/MANAGER_PATTERN.md` for architectural details.

---

## Troubleshooting

### Build fails with "unresolved import"

**Problem**: Python bindings still reference `crate::hardware`

**Solution**: Search for all occurrences of `crate::hardware` and replace with `gpm_original::resources`

```bash
grep -r "crate::hardware" src/python_bindings/
# Update each file found
```

### Python import fails

**Problem**: gpm_original doesn't export required types

**Solution**: Verify gpm_original's `lib.rs` has public exports:

```rust
// gpm_original/src/lib.rs
pub mod resources;
pub use resources::Resource;
```

### Hardware initialization fails

**Problem**: Method signatures don't match between temp implementations and gpm_original

**Solution**: Check that gpm_original methods match the expected signatures listed in "Required Exports" section above. Update Python bindings if signatures have changed.

---

## Testing Checklist

After integration, verify:

- [ ] `maturin develop` builds without errors
- [ ] `python -c "import gpm"` succeeds
- [ ] Can instantiate all hardware classes: `Maestro()`, `Emg()`, `Fsr()`, `Bms()`
- [ ] Mock mode works (build without `--features pi`)
- [ ] Pi mode builds (build with `--features pi` on Pi Zero)
- [ ] Python tests pass: `pytest tests/` (using mock fixtures)
- [ ] Application runs without errors: `python main.py`

---

## Contact

For questions or issues during integration:
- **Python team**: Focus on ensuring bindings work with new imports
- **Rust team**: Focus on completing gpm_original with required exports
- **Integration issues**: Check this guide and verify method signatures match

Last updated: 2025-12-12
