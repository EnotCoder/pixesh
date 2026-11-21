#!/bin/sh
# Install the unpacked pixesh archive into ~/.local (no root needed).
# Override PREFIX to install somewhere else, e.g. PREFIX=/usr/local ./install.sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PREFIX=${PREFIX:-"$HOME/.local"}
BIN_DIR="$PREFIX/bin"
DATA_DIR="$PREFIX/share/pixesh"
ICON_DIR="$PREFIX/share/icons/hicolor/256x256/apps"
DESKTOP_DIR="$PREFIX/share/applications"

mkdir -p "$BIN_DIR" "$DATA_DIR" "$ICON_DIR" "$DESKTOP_DIR"

cp "$HERE/pixesh" "$DATA_DIR/pixesh"
chmod 755 "$DATA_DIR/pixesh"
ln -sfn "$DATA_DIR/pixesh" "$BIN_DIR/pixesh"
cp "$HERE/pixesh.png" "$ICON_DIR/pixesh.png"
# The archive desktop file calls `pixesh`; point it at the installed binary so
# it also works when $PREFIX/bin is not in PATH.
sed "s|^Exec=pixesh |Exec=$DATA_DIR/pixesh |" "$HERE/pixesh.desktop" \
    > "$DESKTOP_DIR/pixesh.desktop"

echo "Pixesh installed to $DATA_DIR"
echo "Run it with: $BIN_DIR/pixesh (add $BIN_DIR to PATH for the pixesh command)"
