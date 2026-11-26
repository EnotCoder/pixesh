#!/usr/bin/env bash
# Build an AppImage from an already-built release binary.
# Usage: scripts/package-appimage.sh [OUT_DIR] [BINARY]
#   APPIMAGETOOL=/path/to/appimagetool overrides the tool lookup/download.
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT=${1:-"$ROOT/dist"}
BIN=${2:-"$ROOT/target/release/pixesh"}
VERSION=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -1)
APPIMAGE="Pixesh-$VERSION-x86_64.AppImage"

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
APPDIR="$WORK/AppDir"

mkdir -p "$APPDIR/usr/bin" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps"

cp "$BIN" "$APPDIR/usr/bin/pixesh"
cp "$ROOT/packaging/icons/pixesh.png" "$APPDIR/pixesh.png"
cp "$ROOT/packaging/icons/pixesh.png" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps/pixesh.png"
# appimagetool validates the desktop file at the AppDir root; the installed
# copy is what desktop environments read once the AppImage is mounted.
cp "$ROOT/packaging/pixesh.desktop" "$APPDIR/pixesh.desktop"
cp "$ROOT/packaging/pixesh.desktop" \
    "$APPDIR/usr/share/applications/pixesh.desktop"

cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$HERE/usr/bin/pixesh" "$@"
EOF
chmod 755 "$APPDIR/AppRun" "$APPDIR/usr/bin/pixesh"

TOOL=${APPIMAGETOOL:-}
if [[ -z "$TOOL" ]]; then
    if command -v appimagetool >/dev/null 2>&1; then
        TOOL=$(command -v appimagetool)
    else
        TOOL="$WORK/appimagetool.AppImage"
        curl -fL --retry 3 \
            -o "$TOOL" \
            https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
        chmod 755 "$TOOL"
    fi
fi

mkdir -p "$OUT"
# Extract-and-run keeps the build working on hosts without FUSE.
APPIMAGE_EXTRACT_AND_RUN=1 ARCH=x86_64 "$TOOL" "$APPDIR" "$OUT/$APPIMAGE"
echo "wrote $OUT/$APPIMAGE"
