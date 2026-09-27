#!/usr/bin/env bash
# ==============================================================================
# Master Packaging Script for C3.AVcompressor
# Builds:
#   1. macOS .pkg installer (C3.AVcompressor-0.1.0-macOS.pkg)
#   2. Debian/Ubuntu .deb packages (c3avcompressor_0.1.0_amd64.deb, arm64.deb)
# Developer Team: C3net Development
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================================================"
echo "🎁 C3.AVcompressor Multiplatform Packaging Suite"
echo "   Developer Team: C3net Development"
echo "========================================================================"

mkdir -p "$PROJECT_ROOT/dist"

# Ensure binary is built
cd "$PROJECT_ROOT"
if [ ! -f "target/release/c3avcompressor" ]; then
    echo "🔨 Building release binary..."
    cargo build --release
fi

# Build macOS Package if running on Darwin
if [ "$(uname)" = "Darwin" ]; then
    echo ""
    echo "🍏 [1/2] Building macOS installer package (.pkg)..."
    chmod +x "$SCRIPT_DIR/macos/build_pkg.sh"
    "$SCRIPT_DIR/macos/build_pkg.sh"
fi

# Build Debian Packages (amd64 and arm64)
echo ""
echo "🐧 [2/2] Building Debian/Ubuntu packages (.deb)..."
chmod +x "$SCRIPT_DIR/debian/build_deb.sh"

# Build for amd64
"$SCRIPT_DIR/debian/build_deb.sh" amd64

# Build for arm64
"$SCRIPT_DIR/debian/build_deb.sh" arm64

echo ""
echo "========================================================================"
echo "🎉 All distribution packages built successfully in dist/:"
ls -lh "$PROJECT_ROOT/dist/"
echo "========================================================================"
