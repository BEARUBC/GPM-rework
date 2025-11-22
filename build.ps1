# Build script for GPM (Windows PowerShell)

Write-Host "Building GPM Rust extension..." -ForegroundColor Green

# Check if maturin is installed
if (!(Get-Command maturin -ErrorAction SilentlyContinue)) {
    Write-Host "Maturin not found. Installing..." -ForegroundColor Yellow
    pip install maturin
}

# Determine target
$target = $args[0]

switch ($target) {
    "pi" {
        Write-Host "Building for Raspberry Pi (with hardware features)..." -ForegroundColor Cyan
        maturin develop --features pi
    }
    "release" {
        Write-Host "Building release wheel..." -ForegroundColor Cyan
        maturin build --release
    }
    "release-pi" {
        Write-Host "Building release wheel for Pi..." -ForegroundColor Cyan
        maturin build --release --features pi
    }
    default {
        Write-Host "Building development version..." -ForegroundColor Cyan
        maturin develop
    }
}

Write-Host "`nBuild complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Usage:"
Write-Host "  python main.py demo    - Run demo mode"
Write-Host "  python main.py         - Run normal operation"
Write-Host "  python ui/cli.py status - Check system status"
