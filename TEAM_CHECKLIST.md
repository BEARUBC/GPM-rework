# Checklist for gpm_original Team

This checklist outlines what the team refactoring `gpm_original` needs to do to make it compatible with the GPM-rework Python bindings.

## ✅ Required Changes to gpm_original

### 1. Update lib.rs to Export Resources

**File:** `gpm_original/src/lib.rs`

**Action:** Make the resources module publicly accessible

```rust
// Add or ensure this exists:
pub mod resources;

// Optionally re-export for convenience:
pub use resources::{
    Maestro,
    Emg,
    Fsr,
    Bms,
    Resource,  // The trait
};
```

**Why:** The Python bindings need to import these types

---

### 2. Ensure Resource Trait is Public

**File:** `gpm_original/src/resources.rs`

**Current:**
```rust
pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
```

**Ensure:** The trait and its methods are `pub`

**Why:** Python bindings call `Resource::init()` to create instances

---

### 3. Make Resource Implementations Accessible

**Files:**
- `gpm_original/src/resources/maestro.rs`
- `gpm_original/src/resources/emg.rs`
- `gpm_original/src/resources/fsr.rs`
- `gpm_original/src/resources/bms.rs`

**Action:** Ensure these are re-exported in `resources` module

```rust
// In src/resources.rs
pub mod maestro;
pub mod emg;
pub mod fsr;
pub mod bms;
pub mod common;

pub use maestro::Maestro;
pub use emg::Emg;
pub use fsr::Fsr;
pub use bms::{Bms, BmsStatus};
```

**Why:** Python bindings import like: `use gpm_original::resources::Maestro`

---

### 4. Add Feature Flags (Recommended)

**File:** `gpm_original/Cargo.toml`

**Add:**
```toml
[features]
# For building standalone binary
standalone = [
    "dep:hyper",
    "dep:http-body-util",
    "dep:prost",
    "dep:prost-types",
    "dep:prometheus-client",
]

# For Python extension use (minimal deps)
python = []

# Hardware support
pi = ["dep:raestro", "dep:rppal", "dep:spidev"]

# Backward compatibility
default = ["standalone"]
```

**Make dependencies optional:**
```toml
[dependencies]
# Core deps (always needed)
anyhow = "1.0"
log = "0.4"
chrono = "0.4.41"
tokio = { version = "1.38.0", features = ["rt", "sync"] }

# Standalone binary deps (optional)
hyper = { version = "1", features = ["full"], optional = true }
prost = { version = "0.12", optional = true }
prometheus-client = { version = "0.22.3", optional = true }

# Hardware deps (optional)
raestro = { version = "0.5.0", optional = true }
rppal = { version = "0.19.0", optional = true }
spidev = { version = "0.7.0", optional = true }
```

**Why:** Allows building library without unnecessary dependencies

---

### 5. Conditional Compilation for Binary-Only Code

**Files:** Various in `src/`

**Action:** Guard binary-specific code with feature flags

```rust
// Managers only needed for standalone
#[cfg(feature = "standalone")]
pub mod managers;

// Dispatchers only needed for standalone
#[cfg(feature = "standalone")]
pub mod dispatchers;

// Exporters only needed for standalone
#[cfg(feature = "standalone")]
pub mod exporters;

// Resources always available
pub mod resources;
```

**Why:** Python bindings don't need managers, dispatchers, or exporters

---

### 6. Update main.rs (if keeping binary)

**File:** `gpm_original/src/main.rs`

**Action:** Add feature requirement for binary

In `Cargo.toml`:
```toml
[[bin]]
name = "gpm"
path = "src/main.rs"
required-features = ["standalone"]
```

**Why:** Binary won't build if feature dependencies aren't enabled

---

### 7. Update Library Crate Type

**File:** `gpm_original/Cargo.toml`

**Current:**
```toml
[lib]
name = "gpm"
```

**Update to:**
```toml
[lib]
name = "gpm_original"  # Or keep as "gpm" if preferred
crate-type = ["lib", "rlib"]  # Library for other Rust crates
```

**Why:** Makes it usable as a dependency in GPM-rework

---

## 🔍 Verification Checklist

After making changes, verify:

- [ ] `cargo build --no-default-features` works (library only)
- [ ] `cargo build --features standalone,pi` works (binary with hardware)
- [ ] Resources are accessible: `use gpm_original::resources::Maestro;` compiles
- [ ] Resource trait is accessible: `use gpm_original::resources::Resource;` compiles
- [ ] Can create instances: `Maestro::init()` works
- [ ] Binary still works: `cargo run --features standalone,pi` runs
- [ ] Tests pass: `cargo test --all-features`

---

## 🚫 Optional: What NOT to Change

You **don't need** to:

- ❌ Add PyO3 to gpm_original (that's in GPM-rework)
- ❌ Create Python bindings in gpm_original (already in GPM-rework)
- ❌ Remove managers/dispatchers (just make them conditional)
- ❌ Break existing functionality (feature flags maintain compatibility)

---

## 📋 Minimal Changes Example

If you want to make **minimal changes** for now:

### Just add these 3 things:

1. **In `src/lib.rs`:**
```rust
pub mod resources;
```

2. **In `src/resources.rs`:**
```rust
pub mod maestro;
pub mod emg;
pub mod fsr;
pub mod bms;
pub mod common;

pub use maestro::Maestro;
pub use emg::Emg;
pub use fsr::Fsr;
pub use bms::{Bms, BmsStatus};

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
```

3. **In `Cargo.toml`:**
```toml
[lib]
name = "gpm_original"
crate-type = ["lib", "rlib"]
```

That's it! Feature flags can come later.

---

## 🤝 Coordination Points

### Questions for GPM-rework team?

1. What should the library name be? (`gpm` vs `gpm_original` vs `gpm_core`)
2. Do you need access to any internal modules beyond resources?
3. Should we use a git submodule or Cargo workspace?
4. What version/commit should we pin to initially?

### Questions for your team?

1. When will the PyO3 refactor be ready for integration?
2. Are there breaking changes we should know about?
3. Should we keep the binary build or deprecate it?
4. Testing strategy for both builds?

---

## 📚 Testing Integration

Once you've made changes, GPM-rework team can test with:

```bash
# In GPM-rework directory, add dependency:
# In Cargo.toml:
# gpm_original = { path = "../gpm_original" }

# Then build:
maturin develop

# Test Python import:
python -c "from gpm import Maestro; m = Maestro(); print('Success!')"
```

---

## 🎯 Success Criteria

Integration is successful when:

✅ GPM-rework can import and use all hardware resources
✅ gpm_original binary still builds and runs (if keeping it)
✅ All existing tests pass
✅ No circular dependencies
✅ Clean separation of concerns

---

## Need Help?

- See `SUBMODULE_INTEGRATION.md` for GPM-rework team's setup
- Check `UBC Bionics Embedded GPM Rework Doc.txt` for architecture details
- Reference this checklist during code review
