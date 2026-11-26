#!/usr/bin/env bash
# Build the Linux distribution artefacts from an already-built release binary:
# an install.sh tarball and a self-contained AppImage. No .deb is produced.
# Usage: scripts/package-linux.sh [OUT_DIR] [BINARY]
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT=${1:-"$ROOT/dist"}
BIN=${2:-"$ROOT/target/release/pixesh"}
VERSION=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -1)
ARCHIVE="pixesh-$VERSION-linux-x86_64"
APPIMAGE="Pixesh-$VERSION-x86_64.AppImage"

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$WORK/$ARCHIVE" "$OUT"
cp "$BIN" "$WORK/$ARCHIVE/pixesh"
cp "$ROOT/packaging/icons/pixesh.png" "$WORK/$ARCHIVE/pixesh.png"
cp "$ROOT/packaging/pixesh.desktop" "$WORK/$ARCHIVE/pixesh.desktop"
cp "$ROOT/packaging/install.sh" "$WORK/$ARCHIVE/install.sh"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$WORK/$ARCHIVE/"
chmod 755 "$WORK/$ARCHIVE/install.sh"

tar -czf "$OUT/$ARCHIVE.tar.gz" -C "$WORK" "$ARCHIVE"
bash "$ROOT/scripts/package-appimage.sh" "$OUT" "$BIN"

(cd "$OUT" && sha256sum "$ARCHIVE.tar.gz" "$APPIMAGE" > SHA256SUMS-linux.txt)
echo "Linux artefacts are in $OUT"
