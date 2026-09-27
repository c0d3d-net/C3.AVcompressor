#!/usr/bin/env bash
# ==============================================================================
# C3.AVcompressor - Universal System Installer (macOS & Linux)
# Developer Team: C3net Development
# ==============================================================================

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

if [ -f "C3.AVcompressor-Header.sh" ]; then
    source "C3.AVcompressor-Header.sh"
    show_banner
fi

echo "========================================================================"
echo "🚀 C3.AVcompressor System Installer"
echo "   Developer Team: C3net Development"
echo "   Platform:       $(uname -sm)"
echo "========================================================================"

# Check for --user flag
INSTALL_MODE="system"
if [ "$1" = "--user" ] || [ "$(id -u)" -ne 0 ] && [ "$EUID" -ne 0 ]; then
    if [ "$1" != "--user" ]; then
        echo "ℹ️  Non-root user detected. Installing to user directory (~/.local/bin)."
        echo "   (To install system-wide, run with: sudo ./install.sh)"
    fi
    INSTALL_MODE="user"
    BIN_DIR="$HOME/.local/bin"
    MAN_DIR="$HOME/.local/share/man/man1"
    ZSH_COMP_DIR="$HOME/.local/share/zsh/site-functions"
    BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
    DOC_DIR="$HOME/.local/share/doc/c3avcompressor"
else
    BIN_DIR="/usr/local/bin"
    MAN_DIR="/usr/local/share/man/man1"
    ZSH_COMP_DIR="/usr/local/share/zsh/site-functions"
    BASH_COMP_DIR="/usr/local/etc/bash_completion.d"
    DOC_DIR="/usr/local/share/doc/c3avcompressor"
fi

echo ""
echo "📁 Installation targets:"
echo "   Binary:      $BIN_DIR/c3avcompressor"
echo "   Man page:    $MAN_DIR/c3avcompressor.1"
echo "   Completions: $ZSH_COMP_DIR, $BASH_COMP_DIR"
echo ""

# Ensure release binary exists
if [ ! -f "target/release/c3avcompressor" ]; then
    echo "🔨 Building release binary..."
    cargo build --release
fi

# Create directories
mkdir -p "$BIN_DIR"
mkdir -p "$MAN_DIR"
mkdir -p "$ZSH_COMP_DIR"
mkdir -p "$BASH_COMP_DIR"
mkdir -p "$DOC_DIR"

# Install binary
echo "⚙️ Installing binary..."
cp -f "target/release/c3avcompressor" "$BIN_DIR/c3avcompressor"
chmod 755 "$BIN_DIR/c3avcompressor"

# Install manpage
if [ -f "packaging/man/c3avcompressor.1" ]; then
    echo "📄 Installing manual page..."
    cp -f "packaging/man/c3avcompressor.1" "$MAN_DIR/"
    chmod 644 "$MAN_DIR/c3avcompressor.1"
    if command -v mandb >/dev/null 2>&1; then
        mandb -q 2>/dev/null || true
    fi
fi

# Install completions
echo "⚡ Generating & installing shell auto-completions..."
"$BIN_DIR/c3avcompressor" completions zsh > "$ZSH_COMP_DIR/_c3avcompressor" 2>/dev/null || true
"$BIN_DIR/c3avcompressor" completions bash > "$BASH_COMP_DIR/c3avcompressor" 2>/dev/null || true

# Install docs
cp -f README.md "$DOC_DIR/" 2>/dev/null || true
cp -f docs/ARCHITECTURE.md "$DOC_DIR/" 2>/dev/null || true
cp -f docs/BENCHMARKS.md "$DOC_DIR/" 2>/dev/null || true

echo ""
echo "========================================================================"
echo "🎉 C3.AVcompressor installed successfully!"
echo ""
# Test execution
"$BIN_DIR/c3avcompressor" npu-status

echo ""
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "⚠️  NOTE: $BIN_DIR is not in your PATH."
    echo "   Add it by running:"
    echo "   export PATH=\"$BIN_DIR:\$PATH\""
    echo ""
fi
echo "👉 Run 'c3avcompressor --help' to get started."
echo "👉 Run './uninstall.sh' if you ever wish to remove it."
echo "========================================================================"
