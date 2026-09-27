#!/usr/bin/env bash
# ==============================================================================
# C3.AVcompressor - Universal System Uninstaller (macOS & Linux)
# Developer Team: C3net Development
# ==============================================================================

set -e

echo "🗑️  Removing C3.AVcompressor from system..."

TARGETS=(
    "/usr/local/bin/c3avcompressor"
    "$HOME/.local/bin/c3avcompressor"
    "/usr/local/share/man/man1/c3avcompressor.1"
    "$HOME/.local/share/man/man1/c3avcompressor.1"
    "/usr/local/share/zsh/site-functions/_c3avcompressor"
    "$HOME/.local/share/zsh/site-functions/_c3avcompressor"
    "/usr/local/etc/bash_completion.d/c3avcompressor"
    "$HOME/.local/share/bash-completion/completions/c3avcompressor"
    "/usr/local/share/doc/c3avcompressor"
    "$HOME/.local/share/doc/c3avcompressor"
)

for target in "${TARGETS[@]}"; do
    if [ -e "$target" ]; then
        echo "   Removing $target..."
        rm -rf "$target" 2>/dev/null || sudo rm -rf "$target" 2>/dev/null || true
    fi
done

echo "✅ C3.AVcompressor has been uninstalled."
