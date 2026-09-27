#!/usr/bin/env bash
# ==============================================================================
# Build macOS .pkg Installer for C3.AVcompressor
# Developer Team: C3net Development
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="0.1.0"
IDENTIFIER="com.c3net.c3avcompressor"
PKG_OUTPUT="$PROJECT_ROOT/dist/C3.AVcompressor-${VERSION}-macOS.pkg"
PAYLOAD_DIR="$SCRIPT_DIR/payload"

echo "📦 Building macOS .pkg installer for C3.AVcompressor v${VERSION}..."

# Ensure release binary exists
cd "$PROJECT_ROOT"
if [ ! -f "target/release/c3avcompressor" ]; then
    echo "🔨 Building release binary..."
    cargo build --release
fi

# Clean and prepare payload structure
rm -rf "$PAYLOAD_DIR"
mkdir -p "$PAYLOAD_DIR/usr/local/bin"
mkdir -p "$PAYLOAD_DIR/usr/local/share/man/man1"
mkdir -p "$PAYLOAD_DIR/usr/local/share/zsh/site-functions"
mkdir -p "$PAYLOAD_DIR/usr/local/etc/bash_completion.d"
mkdir -p "$PAYLOAD_DIR/usr/local/share/doc/c3avcompressor"

# Copy binary
cp "$PROJECT_ROOT/target/release/c3avcompressor" "$PAYLOAD_DIR/usr/local/bin/"
chmod 755 "$PAYLOAD_DIR/usr/local/bin/c3avcompressor"

# Copy man page
cp "$PROJECT_ROOT/packaging/man/c3avcompressor.1" "$PAYLOAD_DIR/usr/local/share/man/man1/"

# Generate shell completions
"$PROJECT_ROOT/target/release/c3avcompressor" completions zsh > "$PAYLOAD_DIR/usr/local/share/zsh/site-functions/_c3avcompressor"
"$PROJECT_ROOT/target/release/c3avcompressor" completions bash > "$PAYLOAD_DIR/usr/local/etc/bash_completion.d/c3avcompressor"

# Copy documentation
cp "$PROJECT_ROOT/README.md" "$PAYLOAD_DIR/usr/local/share/doc/c3avcompressor/"
cp "$PROJECT_ROOT/docs/ARCHITECTURE.md" "$PAYLOAD_DIR/usr/local/share/doc/c3avcompressor/"
cp "$PROJECT_ROOT/docs/BENCHMARKS.md" "$PAYLOAD_DIR/usr/local/share/doc/c3avcompressor/"

# Ensure scripts permissions
chmod +x "$SCRIPT_DIR/scripts/postinstall"

# Create output directory
mkdir -p "$PROJECT_ROOT/dist"

# Build package using pkgbuild
pkgbuild \
    --root "$PAYLOAD_DIR" \
    --identifier "$IDENTIFIER" \
    --version "$VERSION" \
    --install-location "/" \
    --scripts "$SCRIPT_DIR/scripts" \
    "$PKG_OUTPUT"

rm -rf "$PAYLOAD_DIR"

echo "✅ macOS installer package created successfully:"
echo "   👉 $PKG_OUTPUT"
