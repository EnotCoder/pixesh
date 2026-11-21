#!/usr/bin/env bash
# Build the Windows installer from an already-built exe.
# Usage: scripts/package-windows.sh [OUT_DIR] [EXE]
#   ISCC=(iscc) by default; set WINE_ISCC=/path/to/iscc.exe to compile under wine.
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
OUT=${1:-"$ROOT/dist"}
EXE=${2:-"$ROOT/target/x86_64-pc-windows-gnu/release/pixesh.exe"}
VERSION=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -1)
ISCC=(iscc)
if [[ -n ${WINE_ISCC:-} ]]; then
    ISCC=(wine "$WINE_ISCC")
fi

# The .iss references pixesh.exe next to itself, so the script works the same
# way with a native Windows build and with a Linux cross-compiled one.
cp "$EXE" "$ROOT/packaging/windows/pixesh.exe"
trap 'rm -f "$ROOT/packaging/windows/pixesh.exe"' EXIT

mkdir -p "$OUT"
ISS="$ROOT/packaging/windows/pixesh.iss"
if [[ -n ${WINE_ISCC:-} ]]; then
    # iscc.exe only understands Windows-style paths.
    ISS=$(winepath -w "$ISS")
fi
"${ISCC[@]}" "/dMyAppVersion=$VERSION" "$ISS"

(cd "$OUT" && sha256sum "pixesh-$VERSION-setup.exe" > SHA256SUMS-windows.txt)
echo "Windows artefacts are in $OUT"
