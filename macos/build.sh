#!/bin/sh
# Build aruaru-IME.app (macOS input method). Usage: macos/build.sh [outdir]
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${1:-$HERE/build}"
APP="$OUT/aruaru-IME.app"
rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
swiftc -O -o "$APP/Contents/MacOS/aruaru-IME" "$HERE/main.swift" -framework Cocoa -framework InputMethodKit
cp "$HERE/Info.plist" "$APP/Contents/Info.plist"
codesign --force --sign - "$APP" 2>/dev/null || true
echo "built: $APP"
