# NOVA Music - Build and Release Documentation

This document covers the macOS build process, packaging, and release workflow for NOVA Music.

---

## 1. Overview

NOVA Music uses **Tauri** for desktop packaging with a Rust backend and SvelteKit frontend. The application is built for macOS (Apple Silicon) using modern Homebrew tools and a structured release process.

**Build Targets**:
- **Development**: `cargo tauri dev` - Local development with auto-rebuild
- **Release App**: `cargo tauri build` - macOS .app bundle
- **DMG**: Manual process using `hdiutil` and `dylibbundler`
- **Updater**: Tauri signer + custom packaging for auto-updates

---

## 2. Prerequisites

### Required Tools
```bash
# Command line tools
brew install git node pnpm cmake

# Rust toolchain (via rustup)
rustup install stable
rustup default stable

# Tauri CLI
npm install -g "@tauri-apps/cli@^2.11.5"

# Frontend
brew install pnpm

# macOS-specific build tools
brew install mpv dylibbundler
```

### Environment Setup
```bash
# Set linker path for libmpv
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"

# Or add to ~/.cargo/config.toml
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[build]
rustflags = ["-L", "/opt/homebrew/lib"]
EOF

# Verify installation
which cargo
which tauri
which pnpm
which mpv
which dylibbundler
```

---

## 3. Development Build

### Local Development
```bash
# Navigate to project
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild

# Install frontend dependencies
cd ui
pnpm install
cd ..

# Start development server (Tauri dev)
cargo tauri dev

# Separate frontend dev (uncomment if needed)
# pnpm --filter ui dev &
# cargo tauri dev
```

**Development Server Output**:
- Frontend: http://localhost:5183
- Tauri DevTools: http://localhost:5173/_dev
- Hot reload: Automatic on file changes

### Build for Local Testing
```bash
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild
cargo tauri build --debug

# Output location
# target/release/bundle/macos/limusic.app
```

---

## 4. Release Build (macOS)

### Automated Release Process
```bash
# This is built in GitHub Actions .github/workflows/macos-release.yml
# Manual process for testing:

export TAG="v1.0.0"  # Set your version tag
export DRY_RUN="false"  # Set to "true" for dry runs

# 1. Checkout the tag
cd /tmp
rm -rf limusic
cd limusic
cargo tauri build

# 2. Bundle libmpv and dependencies
APP=target/release/bundle/macos/limusic.app
BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw "$APP/Contents/Info.plist")"

dylibbundler -cd -of -b \
  -x "$BIN" \
  -d "$APP/Contents/Frameworks" \
  -p "@executable_path/../Frameworks" \
  -s "$(brew --prefix)/lib"

# 3. Fix duplicate rpaths
find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
  otool -l "$f" 2>/dev/null \
    | awk '/LC_RPATH/{g=1} g&&/^[[:space:]]*path /{sub(/^[[:space:]]*path /, ""); sub(/ \(offset [0-9]+\)$/, ""); print; g=0}' \
    | sort | uniq -c | while read -r n rp; do
      [ "$n" -gt 1 ] || continue
      echo "Dropping $((n - 1)) duplicate rpath(s) $rp"
      i=1
      while [ "$i" -lt "$n" ]; do
        install_name_tool -delete_rpath "$rp" "$f"
        i=$((i + 1))
      done
    done
done

# 4. Re-sign the bundle
codesign --force --deep --sign -

# 5. Build DMG
cd "$(dirname "$APP")"
DMG="limusic_${VERSION}_aarch64.dmg"
STAGE="$(mktemp -d)/Limusic"
mkdir -p "$STAGE" "$(dirname "$DMG")"
ditto "$APP" "$STAGE/limusic.app"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "Limusic" -srcfolder "$STAGE" -ov -format UDZO "$DMG"
```

### Manual Build Steps

#### Step 1: Frontend Build
```bash
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild/ui
cd pnpm install
cd pnpm build
```

#### Step 2: Backend Build
```bash
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild

# Set library path
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"

# Build with Tauri
cargo tauri build

# Output: target/release/bundle/macos/limusic.app
```

#### Step 3: Bundle Dependencies
```bash
# Install required tools if not present
brew install mpv dylibbundler

APP="target/release/bundle/macos/limusic.app"
BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw "$APP/Contents/Info.plist")"

# Create frameworks directory
mkdir -p "$APP/Contents/Frameworks"

# Bundle libmpv and all dependencies
echo "Note: dylibbundler will prompt for passwords"
dylibbundler -cd -of -b \
  -x "$BIN" \
  -d "$APP/Contents/Frameworks" \
  -p "@executable_path/../Frameworks" \
  -s "$(brew --prefix)/lib" < /dev/null
```

#### Step 4: Fix Duplicate Rpaths
```bash
# Homebrew libmpv ships with duplicate rpaths (Xcode + system paths)
APP="target/release/bundle/macos/limusic.app"

find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
  echo "Checking $f"
  otool -l "$f" 2>/dev/null \
    | awk '/LC_RPATH/{g=1} g&&/^[[:space:]]*path /{sub(/^[[:space:]]*path /, ""); sub(/ \(offset [0-9]+\)$/, ""); print; g=0}' \
    | sort | uniq -c | while read -r count path; do
      [ "$count" -gt 1 ] || continue
      echo "  Dropping duplicate rpath: $path"
      install_name_tool -delete_rpath "$path" "$f"
    done
done

echo "Rpath cleanup complete"
```

#### Step 5: Re-sign the Bundle
```bash
APP="target/release/bundle/macos/limusic.app"

# Ad-hoc signing (for distribution)
codesign --force --deep --sign - "$APP"

# Verify signature
codesign --verify --deep --strict "$APP"

# Check entitlements
codesign --display --type=archive "$APP"
```

#### Step 6: Build DMG
```bash
APP="target/release/bundle/macos/limusic.app"
VERSION="$(jq -r .version src-tauri/tauri.conf.json)"
DMG="target/release/bundle/dmg/limusic_${VERSION}_aarch64.dmg"

STAGE="$(mktemp -d)/Limusic"
mkdir -p "$STAGE" "$(dirname "$DMG")"

# Copy app to stage directory
ditto "$APP" "$STAGE/limusic.app"

# Create Applications symlink (for drag-to-install)
ln -s /Applications "$STAGE/Applications"

# Create DMG
hdiutil create -volname "Limusic" -srcfolder "$STAGE" -ov -format UDZO "$DMG"

echo "DMG created: $DMG"
```

#### Step 7: Verify Bundle
```bash
APP="target/release/bundle/macos/limusic.app"
VERSION="$(jq -r .version src-tauri/tauri.conf.json)"

# 1. Check no Homebrew absolute paths remain
find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
  otool -L "$f" 2>/dev/null | tail -n +2 \
    | grep "/opt/homebrew" && echo "ERROR: $f still links to Homebrew"
done

# 2. Test launch
BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw "$APP/Contents/Info.plist")"
if [ -x "$BIN" ]; then
  echo "Executable is executable"
  # Optional: launch in background and quit
  # "$BIN" &
  # sleep 2
  # kill $!
else
  echo "ERROR: Binary is not executable"
  exit 1
fi

# 3. Check signature
codesign --verify --deep --strict "$APP" && echo "Signature valid"
```

---

## 5. Update Process

### Signing Updates
```bash
# Required secrets (GitHub Actions)
export TAURI_SIGNING_PRIVATE_KEY="..."
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="..."

# Sign the DMG (after build)
APP="target/release/bundle/macos/limusic.app"
DMG="target/release/bundle/dmg/limusic_${VERSION}_aarch64.dmg"

# Create updater artifact (Tauri signer)
tauri signer sign "$DMG"

# The .sig file is used by the updater for verification
```

### Update Workflow
1. **New build**: Build and test new DMG
2. **Sign**: Generate signed DMG + .sig file
3. **Release**: Attach both DMG and .sig to GitHub release
4. **Update test**: Install DMG on test Mac
5. **Publish**: Mark release as stable

---

## 6. Troubleshooting

### Common Issues

#### libmpv not found
```bash
# Verify libmpv is installed
brew list mpv
ls "$(brew --prefix)/lib" | grep libmpv

# Set LIBRARY_PATH
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"

# Or add to Cargo config
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[build]
rustflags = ["-L", "/opt/homebrew/lib"]
EOF
```

#### dylibbundler failures
```bash
# Ensure permissions are correct
chmod +x "$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw ...)"

# Check for prompt issues
# dylibbundler will prompt for sudo password when resolving absolute paths
# Use: dylibbundler -cd -of -b -x "binary" -d "dest" -p "@executable_path" -s "search_path" < /dev/null
```

#### Codesign errors
```bash
# Check keychain access
certsync --import
security find-identity -v -s codesign

# Reset signing (if needed)
rm -rf "target/release/bundle/macos/limusic.app"
cargo tauri build
codesign --force --deep --sign - "target/release/bundle/macos/limusic.app"
```

#### DMG creation issues
```bash
# Check disk space
df -h /Users/ganeshsah

# Clean up temporary files
rm -rf "/tmp/Limusic" "$(mktemp -d)"

# Ensure proper permissions
chmod -R a+rX "$APP/Contents"
```

### Debugging Launch Issues
```bash
# Run the app directly for debugging
APP="target/release/bundle/macos/limusic.app"
BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw "$APP/Contents/Info.plist")"

# With environment variables
DYLD_PRINT_INITIALIZED_PATHS=1 "$BIN"

# Or with debugging
DYLD_INSERT_LIBRARIES=/usr/lib/libdyld.dylib DYLD_DEBUG=0x01 "$BIN"
```

---

## 7. CI/CD Integration

### GitHub Actions Workflow
```yaml
name: macOS Release
on:
  workflow_dispatch:
    inputs:
      tag:
        description: Release tag to build for
        required: true
        default: v1.0.0
      dry_run:
        description: Build but don't release
        required: false
        default: false

jobs:
  build:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.tag }}

      - name: Set up environment
        run: |
          export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"
          brew install mpv dylibbundler

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: ui/pnpm-lock.yaml

      - name: Install dependencies
        run: cd ui && pnpm install --frozen-lockfile

      - name: Build frontend
        run: cd ui && pnpm build

      - name: Build backend
        run: cargo tauri build

      - name: Bundle dependencies
        run: |
          APP="target/release/bundle/macos/limusic.app"
          BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw \"$APP/Contents/Info.plist\")"
          dylibbundler -cd -of -b \
            -x "$BIN" \
            -d "$APP/Contents/Frameworks" \
            -p "@executable_path/../Frameworks" \
            -s "$(brew --prefix)/lib" < /dev/null

      - name: Fix rpaths
        run: |
          APP="target/release/bundle/macos/limusic.app"
          find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
            otool -l "$f" 2>/dev/null \
              | awk '/LC_RPATH/{g=1} g&&/^[[:space:]]*path /{sub(/^[[:space:]]*path /, ""); sub(/ \(offset [0-9]+\)$/, ""); print; g=0}' \
              | sort | uniq -c | while read -r n rp; do
                [ "$n" -gt 1 ] || continue
                install_name_tool -delete_rpath "$rp" "$f"
              done
          done

      - name: Re-sign
        run: codesign --force --deep --sign -

      - name: Build DMG
        run: |
          APP="target/release/bundle/macos/limusic.app"
          VERSION="$(jq -r .version src-tauri/tauri.conf.json)"
          DMG="target/release/bundle/dmg/limusic_${VERSION}_aarch64.dmg"
          STAGE="$(mktemp -d)/Limusic"
          mkdir -p "$STAGE" "$(dirname "$DMG")"
          ditto "$APP" "$STAGE/limusic.app"
          ln -s /Applications "$STAGE/Applications"
          hdiutil create -volname "Limusic" -srcfolder "$STAGE" -ov -format UDZO "$DMG"

      - name: Verify bundle
        run: |
          APP="target/release/bundle/macos/limusic.app"
          find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
            otool -L "$f" 2>/dev/null | tail -n +2 \
              | grep "/opt/homebrew" && echo "ERROR: $f still links to Homebrew"
          done
          codesign --verify --deep --strict "$APP"

      - name: Upload DMG
        if: ${{ !inputs.dry_run }}
        uses: actions/upload-artifact@v4
        with:
          name: limusic-macos-${{ inputs.tag }}
          path: "target/release/bundle/dmg/*.dmg"
          retention-days: 7
```

---

## 8. Performance Optimization

### Build Time Optimization
```bash
# Use rust-cache for incremental compilation
cargo install cargo-husky
cargo husky init

# Enable caching for npm/pnpm
cache-pnpm() {
  export PNPM_HOME="${XDG_CACHE_HOME:-$HOME/.cache/pnpm}"
  pnpm install --frozen-lockfile
}

# Parallel builds
cargo build --release -j$(nproc)
```

### Runtime Optimization
```bash
# Set performance parameters
cat > ~/.taurc << 'EOF'
[tauri]
# Performance settings
development = {
  strict = false,
  beforeDevCommand = "pnpm --filter ui dev",
  beforeBuildCommand = "pnpm --filter ui build"
}
EOF
```

---

## 9. Versioning

### Semantic Versioning
```
major: BREAKING changes
minor: new features, backward compatible
patch: bug fixes, backward compatible
```

### Build Numbering
```
NOVA Music follows: M.m.p.BUILD
Example: 1.2.3.456 (v1.2.3 with build 456)
```

### Tagging
```bash
# Release tag
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild
git tag v1.0.0 -m "First release"
git push origin v1.0.0
```

---

## 10. User Documentation

### Installation Instructions
```bash
# Download .dmg from GitHub releases
cd /Users/ganeshsah/Downloads
# Double-click to install
# Move Limusic.app to Applications

# First launch (Gatekeeper warning)
open /Applications/Limusic.app
# macOS will warn about "unidentified developer"
# Click "Open" and confirm with password

# Alternative: xattr bypass
xattr -dr com.apple.quarantine /Applications/Limusic.app
open /Applications/Limusic.app
```

### Update Instructions
```bash
# Standard macOS update process:
# Download new .dmg
# Drag new Limusic.app to Applications (overwrites old)

# Or using built-in updater:
# Nova Music has auto-update capability
# Check Settings -> General -> Update
```

---

## 11. Support & Troubleshooting

### Common User Issues

#### App crashes on launch
```bash
# Check for missing libraries
otool -L "$(find . -name 'Limusic' -type f | head -1)" | grep "not loaded"

# Re-sign if needed
codesign --force --deep --sign - "target/release/bundle/macos/Limusic.app"
```

#### GUI not showing
```bash
# Check display
ls -la /dev/tty*

# Try with different display
cargo tauri build --target aarch64-apple-darwin
```

#### Performance issues
```bash
# Check system resources
top -o cpu -n 10

# Free up disk space
du -sh /Users/ganeshsah
```

### Support Channels
- **GitHub Issues**: Feature requests and bug reports
- **macOS Console**: Logs in ~/Library/Logs
- **App logs**: Limited in production; use `codesign --verbose=2`

---

## 12. Final Verification Checklist

### Before Release
```
✅ Frontend builds successfully
✅ Backend builds successfully
✅ libmpv bundled correctly
✅ Duplicate rpaths removed
✅ Bundle re-signed
✅ DMG created successfully
✅ No Homebrew absolute paths remain
✅ Signature verification passes
✅ App launches and runs
✅ Basic functionality works (search, play, queue)
✅ UI tests pass
✅ Performance acceptable
```

### Release Assets
```
target/release/bundle/dmg/limusic_1.0.0_aarch64.dmg      # macOS .dmg
# Optionally: .sig file for updater verification
target/release/bundle/macos/limusic.app                    # For debugging
```

---

## 13. Security & Privacy

### Data Handling
- **No telemetry**: No usage analytics collected
- **Secure storage**: Authentication tokens stored in macOS Keychain
- **No crash logs**: Minified in production to prevent info disclosure
- **Sign-app guidelines**: Follow App Store security best practices

### Code Signing
- **Ad-hoc signing**: For direct distribution
- **Notarization**: Required for App Store distribution (paid Developer account)
- **Notarization process**: Upload .dmg to Apple, wait for approval
- **Retry policy**: Automate retry on rejection

---

## 14. Project Cleanup

### Temporary Files
```bash
# Clean build artifacts
cargo clean
rm -rf ui/node_modules
rm -rf target/release/bundle/dmg/*.dmg
rm -rf "$(mktemp -d)"

# Clean logs
rm -f ~/Library/Logs/limusic.log*
```

### Environment Configuration
```bash
# Save build configuration
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild
cp .cargo/config.toml .cargo/config.toml.bak
cp Cargo.lock Cargo.lock.bak
```

---

## 15. Acknowledgements

Special thanks to:
- **LiMusic contributors**: Battle-tested YouTube Music integration
- **Tauri team**: Excellent desktop framework
- **libmpv developers**: High-performance audio playback
- **SvelteKit team**: Modern frontend framework
- **Homebrew**: Dependency management on macOS

---

## 16. Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-10-11 | 1.0.0 | Initial release build documentation |
| | | macOS build process documentation |
| | | Performance optimization guide |
| | | Troubleshooting section |

---

*Document maintained by NOVA Music development team*
*Last updated: 2026-10-11*
*For questions: contact the development team*
