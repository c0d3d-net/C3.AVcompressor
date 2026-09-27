#!/usr/bin/env bash
# ==============================================================================
# Build Debian/Ubuntu .deb Package for C3.AVcompressor
# Developer Team: C3net Development
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="0.1.0"
ARCH="${1:-amd64}"
DEB_NAME="c3avcompressor_${VERSION}_${ARCH}.deb"
PKG_DIR="$SCRIPT_DIR/build_deb_pkg"
DIST_DIR="$PROJECT_ROOT/dist"

echo "📦 Building Debian/Ubuntu .deb package for C3.AVcompressor v${VERSION} (${ARCH})..."

mkdir -p "$DIST_DIR"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/man/man1"
mkdir -p "$PKG_DIR/usr/share/bash-completion/completions"
mkdir -p "$PKG_DIR/usr/share/zsh/vendor-completions"
mkdir -p "$PKG_DIR/usr/share/doc/c3avcompressor"

# Prepare control file with correct architecture
sed "s/Architecture: .*/Architecture: ${ARCH}/" "$SCRIPT_DIR/DEBIAN/control" > "$PKG_DIR/DEBIAN/control"
cp "$SCRIPT_DIR/DEBIAN/postinst" "$PKG_DIR/DEBIAN/"
cp "$SCRIPT_DIR/DEBIAN/prerm" "$PKG_DIR/DEBIAN/"
chmod 755 "$PKG_DIR/DEBIAN/postinst" "$PKG_DIR/DEBIAN/prerm"

# Copy binary
if [ -f "$PROJECT_ROOT/target/release/c3avcompressor" ]; then
    cp "$PROJECT_ROOT/target/release/c3avcompressor" "$PKG_DIR/usr/bin/"
    chmod 755 "$PKG_DIR/usr/bin/c3avcompressor"
    
    # Generate completions
    "$PROJECT_ROOT/target/release/c3avcompressor" completions bash > "$PKG_DIR/usr/share/bash-completion/completions/c3avcompressor"
    "$PROJECT_ROOT/target/release/c3avcompressor" completions zsh > "$PKG_DIR/usr/share/zsh/vendor-completions/_c3avcompressor"
fi

# Copy man page (compressed with gzip)
gzip -c -9 "$PROJECT_ROOT/packaging/man/c3avcompressor.1" > "$PKG_DIR/usr/share/man/man1/c3avcompressor.1.gz"

# Copy docs
cp "$PROJECT_ROOT/README.md" "$PKG_DIR/usr/share/doc/c3avcompressor/"
cp "$PROJECT_ROOT/docs/ARCHITECTURE.md" "$PKG_DIR/usr/share/doc/c3avcompressor/"
cp "$PROJECT_ROOT/docs/BENCHMARKS.md" "$PKG_DIR/usr/share/doc/c3avcompressor/"

# Build package
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "$PKG_DIR" "$DIST_DIR/$DEB_NAME"
else
    echo "ℹ️  dpkg-deb not found; using standard tar+ar packaging engine..."
    WORK_TMP="$SCRIPT_DIR/.deb_tmp"
    rm -rf "$WORK_TMP"
    mkdir -p "$WORK_TMP"
    
    echo "2.0" > "$WORK_TMP/debian-binary"
    
    (cd "$PKG_DIR/DEBIAN" && tar -czf "$WORK_TMP/control.tar.gz" .)
    (cd "$PKG_DIR" && tar -czf "$WORK_TMP/data.tar.gz" ./usr)
    
    python3 "$SCRIPT_DIR/create_deb_archive.py" "$DIST_DIR/$DEB_NAME" \
        "$WORK_TMP/debian-binary" \
        "$WORK_TMP/control.tar.gz" \
        "$WORK_TMP/data.tar.gz"
    rm -rf "$WORK_TMP"
fi

rm -rf "$PKG_DIR"

echo "✅ Debian package created successfully:"
echo "   👉 $DIST_DIR/$DEB_NAME"
