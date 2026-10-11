#!/bin/bash
set -euo pipefail

# Test build script for NOVA Music
# This script validates the basic build process on macOS

echo "=== NOVA Music Build Test ==="

# Check required tools
check_tool() {
    if ! command -v "$1" &> /dev/null; then
        echo "ERROR: $1 is not installed"
        return 1
    fi
    echo "✓ $1 found"
}

check_tool "cargo"
check_tool "pnpm"
check_tool "mpv"
check_tool "dylibbundler"

# Set library path
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"

echo ""
echo "=== Testing Frontend Build ==="
cd ui
if pnpm build; then
    echo "✓ Frontend build successful"
else
    echo "✗ Frontend build failed"
    exit 1
fi
cd ..

echo ""
echo "=== Testing Backend Build ==="
cargo build --release --features tauri --target x86_64-apple-darwin
if [ $? -eq 0 ]; then
    echo "✓ Backend build successful"
else
    echo "✗ Backend build failed"
    exit 1
fi

echo ""
echo "=== Testing Bundle Structure ==="
APP="target/release/bundle/macos/limusic.app"
if [ -d "$APP" ]; then
    echo "✓ Application bundle exists"
    
    # Check executable
    BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw "$APP/Contents/Info.plist")"
    if [ -x "$BIN" ]; then
        echo "✓ Executable is executable"
    else
        echo "✗ Executable is not executable"
        exit 1
    fi
    
    # Check for libmpv
    if find "$APP" -name "*.dylib" | grep -q libmpv; then
        echo "✓ libmpv bundled"
    else
        echo "✗ libmpv not found in bundle"
        exit 1
    fi
    
    # Check for rpaths
    if find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
        otool -l "$f" 2>/dev/null | grep -q "LC_RPATH" && echo "  - $f: has rpaths"
done; then
        echo "✓ Bundle has read paths (expected)"
    fi
else
    echo "✗ Application bundle not found"
    exit 1
fi

echo ""
echo "=== Build Test Summary ==="
echo "✓ All tests passed successfully!"
echo ""
echo "The project is ready for development and testing."
echo ""
echo "Next steps:"
echo "1. Run the app: ./target/release/bundle/macos/limusic.app/Contents/MacOS/limusic"
echo "2. Set up local development environment"
echo "3. Begin implementing core features"
echo ""
echo "For more information, see:"
echo "- docs/ARCHITECTURE.md"
echo "- docs/DESIGN-SYSTEM.md"
echo "- docs/BUILD-AND-RELEASE.md"
echo "- docs/FEATURE-INVENTORY.md"