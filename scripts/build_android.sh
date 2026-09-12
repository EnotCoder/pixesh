#!/usr/bin/env bash
# Manual Android packaging pipeline (no Gradle/cargo-apk).
# Outputs a signed debug APK: pixesh-debug.apk
set -euo pipefail
cd "$(dirname "$0")/.."

# ── 1. Configuration ──────────────────────────────────
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
NDK_VER="25.2.9519653"
NDK="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/$NDK_VER}"
BUILD_TOOLS_VER="36.1.0"
PLATFORM_VER="android-33"

TOOL_BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
SDK_BIN="$ANDROID_HOME/build-tools/$BUILD_TOOLS_VER"
PLATFORM_JAR="$ANDROID_HOME/platforms/$PLATFORM_VER/android.jar"

echo "▸ 1/5 Building native library (aarch64)…"
export PATH="$TOOL_BIN:$PATH"
export CC_aarch64_linux_android="$TOOL_BIN/aarch64-linux-android26-clang"
export AR_aarch64_linux_android="$TOOL_BIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOL_BIN/aarch64-linux-android26-clang"

cargo build --target aarch64-linux-android --release --lib --no-default-features --features mobile

echo "▸ 2/5 Compiling resources…"
mkdir -p target/android/res
"$SDK_BIN/aapt2" compile --dir android/res -o target/android/compiled_res.zip

echo "▸ 3/5 Linking APK…"
"$SDK_BIN/aapt2" link -o target/android/pixesh-base.apk \
    -I "$PLATFORM_JAR" \
    --manifest android/AndroidManifest.xml \
    target/android/compiled_res.zip \
    --auto-add-overlay

echo "▸ 4/5 Injecting native library…"
# Create the required directory structure for the APK
mkdir -p target/android/lib/arm64-v8a
cp target/aarch64-linux-android/release/libpixesh.so target/android/lib/arm64-v8a/
cd target/android
zip -u pixesh-base.apk lib/arm64-v8a/libpixesh.so
cd ../..

echo "▸ 5/5 Aligning and Signing…"
"$SDK_BIN/zipalign" -f 4 target/android/pixesh-base.apk pixesh-debug-unsigned.apk
"$SDK_BIN/apksigner" sign --ks "$HOME/.android/debug.keystore" \
    --ks-pass pass:android --key-pass pass:android \
    --out pixesh-debug.apk pixesh-debug-unsigned.apk

echo "✓ Done: pixesh-debug.apk"
