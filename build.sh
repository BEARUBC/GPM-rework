#!/bin/bash
# Build script for GPM

set -e

echo "Building GPM Rust extension..."

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    echo "Maturin not found. Installing..."
    pip install maturin
fi

# Determine target
if [[ "$1" == "pi" ]]; then
    echo "Building for Raspberry Pi (with hardware features)..."
    maturin develop --features pi
elif [[ "$1" == "release" ]]; then
    echo "Building release wheel..."
    maturin build --release
elif [[ "$1" == "release-pi" ]]; then
    echo "Building release wheel for Pi..."
    maturin build --release --features pi
elif [[ "$1" == "release-pi0" ]]; then
    # Cross-compile for Raspberry Pi Zero 2 W (aarch64, 64-bit Raspberry Pi OS).
    # Requires: rustup target add aarch64-unknown-linux-gnu && pip install ziglang
    # PI0_PYTHON must match the Python version on the device (Bookworm: 3.11, Trixie: 3.13).
    PI0_PYTHON="${PI0_PYTHON:-python3.11}"
    echo "Cross-compiling release wheel for Pi Zero 2 W ($PI0_PYTHON)..."
    maturin build --release --features pi \
        --target aarch64-unknown-linux-gnu --zig -i "$PI0_PYTHON"
else
    echo "Building development version..."
    maturin develop
fi

echo "Build complete!"
echo ""
echo "Usage:"
echo "  python main.py demo    - Run demo mode"
echo "  python main.py         - Run normal operation"
echo "  GPM_CONFIG=pi0 python main.py - Run with the Pi Zero 2 W board profile"
echo "  python ui/cli.py status - Check system status"
