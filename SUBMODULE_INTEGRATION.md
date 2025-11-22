# Git Submodule Integration Guide

This guide explains how to integrate `gpm_original` as a git submodule to share hardware code.

## Current Status

✅ **Python bindings are ready** - All PyO3 wrapper code in `src/python_bindings/` is prepared
✅ **Python application layer complete** - All control logic in `application/` directory
⏳ **Awaiting submodule setup** - Currently using local `src/hardware/` as temporary implementation

## Prerequisites

The team working on `gpm_original` needs to:

1. ✅ Add PyO3 feature flag to their `Cargo.toml`
2. ✅ Make resources module publicly accessible
3. ✅ Ensure `Resource` trait is exported
4. ✅ Add Python bindings feature (optional, can be in this repo)

## Step 1: Add gpm_original as Submodule

From the `GPM-rework` directory:

```bash
# Add gpm_original as a submodule
git submodule add <gpm_original_repo_url> gpm_original

# Initialize and update
git submodule init
git submodule update

# Commit the submodule addition
git add .gitmodules gpm_original
git commit -m "Add gpm_original as submodule"
```

## Step 2: Update Cargo.toml

Uncomment the gpm_original dependency:

```toml
[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }

# Uncomment this line:
gpm_original = { path = "./gpm_original" }

# Remove or comment out temporary dependencies:
# anyhow = "1.0"
# tokio = { version = "1.38.0", features = ["rt", "sync"] }
# raestro = { version = "0.5.0", optional = true }
# ... etc (these will come from gpm_original)
```

## Step 3: Update Python Bindings

In each file in `src/python_bindings/`, replace the TODO comments:

### maestro.rs
```rust
// Change from:
use crate::hardware::maestro::Maestro as RustMaestro;
use crate::hardware::Resource;

// To:
use gpm_original::resources::Maestro as RustMaestro;
use gpm_original::resources::Resource;
```

### emg.rs
```rust
// Change from:
use crate::hardware::emg::Emg as RustEmg;
use crate::hardware::Resource;

// To:
use gpm_original::resources::Emg as RustEmg;
use gpm_original::resources::Resource;
```

### fsr.rs
```rust
// Change from:
use crate::hardware::fsr::{Fsr as RustFsr, FsrReading as RustFsrReading};
use crate::hardware::Resource;

// To:
use gpm_original::resources::fsr::{Fsr as RustFsr, FsrReading as RustFsrReading};
use gpm_original::resources::Resource;
```

### bms.rs
```rust
// Change from:
use crate::hardware::bms::{Bms as RustBms, BmsStatus as RustBmsStatus};
use crate::hardware::Resource;

// To:
use gpm_original::resources::bms::{Bms as RustBms, BmsStatus as RustBmsStatus};
use gpm_original::resources::Resource;
```

## Step 4: Update lib.rs

Add the external crate reference:

```rust
use pyo3::prelude::*;

// Add this to import from submodule:
mod hardware {
    pub use gpm_original::resources::*;
}

mod python_bindings;
// ... rest of file
```

Or simpler - just remove the local hardware module completely and reference gpm_original directly in the bindings.

## Step 5: Remove Local Hardware Directory

Once the submodule is integrated and working:

```bash
# Remove the temporary hardware implementation
rm -rf src/hardware/

# Commit the change
git add src/
git commit -m "Remove local hardware implementations, now using gpm_original"
```

## Step 6: Update .gitignore

Add submodule-related ignores if needed:

```gitignore
# In .gitignore
gpm_original/target/
```

## Step 7: Test the Build

```bash
# Clean build to ensure everything works
cargo clean

# Build Python extension
maturin develop

# Test it works
python -c "from gpm import Maestro; print('Success!')"

# Run demo
python main.py demo
```

## Working with Submodules

### Updating gpm_original

```bash
# Update to latest commit
cd gpm_original
git pull origin main
cd ..
git add gpm_original
git commit -m "Update gpm_original submodule"
```

### Cloning GPM-rework with Submodules

New users need to clone with submodules:

```bash
# Clone with submodules
git clone --recursive <gpm-rework-repo-url>

# Or if already cloned:
git submodule update --init --recursive
```

### Feature Flags

Pass features through to gpm_original:

```toml
[features]
pi = ["gpm_original/pi"]  # Pass pi feature to submodule
```

## Requirements for gpm_original Team

For this integration to work, `gpm_original` needs:

### 1. Public Module Exports

In `gpm_original/src/lib.rs`:

```rust
// Make resources publicly accessible
pub mod resources;

// Or re-export what's needed:
pub use resources::{
    Maestro, Emg, Fsr, Bms, BmsStatus, FsrReading,
    Resource  // The trait
};
```

### 2. Feature Flag (Optional)

In `gpm_original/Cargo.toml`:

```toml
[features]
# For standalone binary
standalone = ["hyper", "prost", "prometheus-client"]

# For Python extension use (minimal deps)
python = []

# Hardware support
pi = ["raestro", "rppal", "spidev"]

# Default for backward compatibility
default = ["standalone"]
```

### 3. Conditional Compilation

Only if they want to keep both binary and library builds:

```rust
// Keep managers/dispatchers only for standalone
#[cfg(feature = "standalone")]
pub mod managers;

#[cfg(feature = "standalone")]
pub mod dispatchers;

// Resources always available
pub mod resources;
```

## Troubleshooting

### Issue: "cannot find crate `gpm_original`"
**Solution:** Make sure you ran `git submodule update --init`

### Issue: "unresolved import `gpm_original::resources`"
**Solution:** Check that `gpm_original/src/lib.rs` has `pub mod resources;`

### Issue: Build fails with missing dependencies
**Solution:** Make sure feature flags are passed correctly in Cargo.toml

### Issue: Submodule shows modified but no changes
**Solution:** This is normal if gpm_original team is actively developing. Update with `git submodule update --remote`

## Benefits of This Approach

✅ **Single source of truth** - Hardware code only in gpm_original
✅ **Easy updates** - Bug fixes automatically available
✅ **Clear separation** - GPM-rework is purely Python bindings + application logic
✅ **Version control** - Can pin to specific gpm_original commit
✅ **Parallel development** - Teams can work independently

## Alternative: Workspace Approach

If submodules are problematic, consider a Cargo workspace:

```toml
# Create workspace Cargo.toml at root
[workspace]
members = ["gpm_original", "gpm_python"]
resolver = "2"
```

Both approaches work - submodule is simpler for separate repos, workspace is better for monorepo.
