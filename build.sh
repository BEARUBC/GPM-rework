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
else
    echo "Building development version..."
    maturin develop
fi

echo "Build complete!"
echo ""
echo "Usage:"
echo "  python main.py demo    - Run demo mode"
echo "  python main.py         - Run normal operation"
echo "  python ui/cli.py status - Check system status"
