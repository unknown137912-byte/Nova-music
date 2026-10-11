# macOS release and updates

NOVA is a Tauri macOS desktop app distributed for Apple Silicon Macs only (arm64). A release produces one `.dmg` installer and a signed updater archive, then publishes `latest.json` with the Apple Silicon update. Intel Macs are unsupported and receive no build or download. The release workflow refuses to publish the DMG if it is 150,000,000 bytes or larger. It also bundles `libmpv` and its Homebrew dylib dependencies into the app so users do not need Homebrew installed.

## One-time setup

1. Push this repository to a **public** GitHub repository. The app's configured updater feed is generated from that repository's owner/name during release.
2. Keep the Tauri updater private key outside the repository. Add it to GitHub Actions as `TAURI_SIGNING_PRIVATE_KEY`. If you generated the key with a passphrase, also add `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Never commit or share the private key; the public key embedded in `src-tauri/tauri.conf.json` is safe to publish.
3. For macOS downloads without Gatekeeper's unidentified-developer warning, add an Apple Developer ID certificate and notarization credentials as repository Actions secrets: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `KEYCHAIN_PASSWORD`, `APPLE_API_ISSUER`, `APPLE_API_KEY`, and base64-encoded `APPLE_API_KEY_CONTENT`. Without these Apple secrets, the app is ad-hoc signed and not notarized, so macOS may show a Gatekeeper warning when downloaded.

## Publish a version

Update the shared version in `src-tauri/tauri.conf.json` and the Cargo workspace `Cargo.toml`, commit the change, then push a matching version tag, for example:

```sh
git tag v1.0.1
git push origin v1.0.1
```

GitHub Actions builds the native Apple Silicon app, adds its DMG and updater assets to a draft release, checks the DMG size, and publishes only if it is under 150 MB. Installed apps can then check GitHub in Settings → About and install the verified update.

## Build locally

On macOS with Xcode command-line tools, Rust, and pnpm installed:

```sh
brew install mpv dylibbundler
pnpm --dir ui install --frozen-lockfile
pnpm install --frozen-lockfile
pnpm exec tauri build --no-bundle
node scripts/prepare-macos-bundle.mjs
pnpm exec tauri bundle --bundles app,dmg --config target/tauri-bundle-arm64.json
```

The release DMG is written under `target/release/bundle/dmg/`. The package preparation step copies `libmpv` and all required Homebrew dylibs into the app before Tauri creates the final `.app`, updater archive, and DMG.
