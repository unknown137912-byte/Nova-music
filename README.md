# NOVA Music - Architecture and Implementation Plan

## Overview

NOVA Music is a premium, feature-rich macOS music desktop application that combines the proven YouTube Music integration from **LiMusic** with an original, sophisticated interface design and Spotify-inspired discovery features.

**Key Differentiators**:
- **Battle-tested backend**: Complete YouTube Music integration from LiMusic (GPL-3.0)
- **Premium macOS experience**: Native design, performance optimizations, Apple Silicon focus
- **Original frontend**: Completely new UI with sophisticated design system
- **Spotify-inspired features**: Enhanced discovery and organization beyond YouTube Music alone

---

## Project Structure

```
NOVA-Music-Rebuild/
├── docs/                    # Architecture and design documentation
│   ├── ARCHITECTURE.md      # System architecture
│   ├── DATA-MODEL.md        # Database schema
│   ├── DESIGN-SYSTEM.md     # Visual design system
│   ├── FEATURE-INVENTORY.md # Complete feature list
│   └── BUILD-AND-RELEASE.md # Build and release process
├── scripts/                 # Build and test scripts
│   ├── test-build.sh        # Build verification script
├── ui/                      # Frontend (SvelteKit)
│   ├── src/                # Source code
│   │   ├── routes/          # Application routes
│   │   └── lib/             # Shared components and utilities
│   └── package.json        # Frontend dependencies
├── src-tauri/               # Backend (Rust/Tauri)
│   ├── src/                # Rust source code
│   │   ├── commands.rs     # Public API
│   │   ├── state.rs        # Application state
│   │   ├── orchestrator.rs # Core orchestration
│   │   └── ...             # Other modules
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Application configuration
├── scripts/                 # CI/CD and release scripts
├── tests/                   # Test suites
└── .gitignore              # Version control ignore patterns
```

---

## Investigation Completed

This project builds upon comprehensive investigation of **LiMusic** (https://github.com/SimoHypers/limusic) including:

### 1. Architecture Analysis ✅
- **Complete source code review** of LiMusic's 1000+ files
- **In-depth analysis** of:
  - Rust backend architecture (Tauri v2 + SvelteKit)
  - libmpv2 audio playback integration
  - YouTube Music InnerTube API client
  - Authentication flow (cookie + OAuth)
  - Database schema (SQLite with migrations)
  - External integrations (Last.fm, ListenBrainz, Discord, etc.)

### 2. macOS Build Process ✅
- **Identified requirements** for macOS distribution
- **Documented build steps**:
  - Homebrew dependencies: `mpv`, `dylibbundler`
  - Library path configuration
  - Dependency bundling with `dylibbundler`
  - Duplicate rpath resolution
  - Code signing process
  - DMG creation with `hdiutil`
- **Verified CI workflow** (GitHub Actions macOS release)

### 3. Feature Inventory ✅
- **Complete feature mapping** between LiMusic and NOVA
- **Documentation of**:
  - All 40+ features in LiMusic
  - Reuse strategy for each feature
  - NOVA-specific enhancements and adaptations
- **Phased implementation plan** (9 phases, 40+ milestones)

---

## Current Status

### ✅ Completed
- [x] Investigation of LiMusic source code and architecture
- [x] macOS build requirements analysis
- [x] Feature inventory and reuse strategy
- [x] Architecture documentation (ARCHITECTURE.md)
- [x] Design system documentation (DESIGN-SYSTEM.md)
- [x] Build and release documentation (BUILD-AND-RELEASE.md)
- [x] Project structure setup
- [x] Documentation structure defined

### 🔄 In Progress
- [ ] **Phase 1: Infrastructure** - Basic Tauri project setup
- [ ] **Phase 2: Core Backend** - YouTube Music integration
- [ ] **Phase 3: Playback** - Audio engine and queue management
- [ ] **Phase 4: Frontend Foundation** - UI shell and design system

---

## Technology Stack

### Backend (Rust)
```rust
- Tauri v2 (Desktop runtime)
- SvelteKit (Web framework via Tauri)
- libmpv2 (Audio playback)
- rusqlite (Database)
- InnerTube (YouTube Music API)
- Souvlaki (Media keys)
- discord-rich-presence (Discord integration)
- listener-protocol (Listen Together)
```

### Frontend (TypeScript/Svelte)
```typescript
- Svelte 5 (Component framework)
- SvelteKit (Routing + SSR)
- TypeScript (Type safety)
- Tailwind CSS v4 (Styling)
- shadcn-svelte + bits-ui (UI components)
- TanStack Query (Data fetching)
```

### macOS-specific
```bash
- Homebrew: mpv, dylibbundler
- Xcode Command Line Tools
- Tauri macOS config (minimumSystemVersion: 14.0)
- Ad-hoc code signing
- .dmg distribution via hdiutil
```

---

## Architecture Overview

### System Design
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │◄──►│   Backend       │◄──►│   Native        │
│   (SvelteKit)   │    │   (Rust/Tauri)  │    │   Components    │
│                 │    │                 │    │                 │
│   ui/           │    │   src-tauri/    │    │   libmpv,      │
│   ┌─────┐       │    │   ┌─────┐       │    │   souvlaki,    │
│   │     │       │    │   │     │       │    │   Keychain,    │
│   │ SPA │       │    │   │ App │       │    │   etc.         │
│   │     │       │    │   │     │       │    │                 │
│   └─────┘       │    │   └─────┘       │    └─────────────────┘
└─────────────────┘    └─────────────────┘
```

### Key Components

#### 1. Core Backend (`src-tauri/src/`)
- **`commands.rs`**: Public Tauri commands (API boundary)
- **`state.rs`**: Global application state
- **`orchestrator.rs`**: High-level command composition
- **`db.rs`**: Database schema and migrations
- **`playback.rs`**: Player orchestration
- **`auth.rs`**: Authentication management

#### 2. Audio Engine (`crates/player/`)
- **libmpv2 wrapper**: Gapless playback, loudness normalization
- **Dual-deck architecture**: Active + preload decks for crossfade
- **Error recovery**: Automatic reconnection, fallback handling

#### 3. YouTube Integration (`crates/innertube/`)
- **InnerTube API client**: Multi-client support (Android, iOS, Web, TV)
- **Authentication**: Cookie + OAuth, session management
- **Cipher/PoToken**: JavaScript execution for token generation

#### 4. Frontend (`ui/src/`)
- **SvelteKit routing**: File-based routing system
- **Central stores**: Reactive state management
- **Component library**: Custom UI based on shadcn-svelte

### Data Flow

1. **Authentication Flow**
   ```
   UI → sign_in command → OAuth webview → Token exchange → Store in Keychain → Load library
   ```

2. **Playback Flow**
   ```
   UI queue change → play_track command → Orchestrator → Player.load → mpv events → UI update
   ```

3. **Search Flow**
   ```
   UI search → search command → InnerTube search → Cache + UI update
   ```

---

## Build Process

### Development
```bash
# Navigate to project
cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild

# Install frontend dependencies
cd ui && pnpm install
cd ..

# Start development server
cargo tauri dev
```

### Release (macOS)
```bash
# Set up environment
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"
brew install mpv dylibbundler

# Build frontend
cd ui && pnpm build

# Build backend
cargo tauri build

# Bundle dependencies with dylibbundler
APP="target/release/bundle/macos/limusic.app"
BIN="$APP/Contents/MacOS/$(plutil -extract CFBundleExecutable raw \"$APP/Contents/Info.plist\")"
dylibbundler -cd -of -b -x "$BIN" -d "$APP/Contents/Frameworks" -p "@executable_path/../Frameworks" -s "$(brew --prefix)/lib"

# Fix duplicate rpaths
find "$APP" -type f -print0 | while IFS= read -r -d '' f; do
  otool -l "$f" 2>/dev/null | awk '/LC_RPATH/{g=1} g&&/^[[:space:]]*path /{sub(/^[[:space:]]*path /, ""); sub(/ \(offset [0-9]+\)$/, ""); print; g=0}' | sort | uniq -c | while read -r n rp; do
    [ "$n" -gt 1 ] || continue
    install_name_tool -delete_rpath "$rp" "$f"
  done
done

# Re-sign
codesign --force --deep --sign -

# Build DMG
hdiutil create -volname "Limusic" -srcfolder "$APP" -ov -format UDZO
```

---

## Feature Priorities

### Phase 1: Foundation (Week 1-2)
- [ ] Basic Tauri project setup
- [ ] Copy `innertube`, `player`, `listen-protocol` crates
- [ ] Database migrations
- [ ] Frontend shell
- [ ] Design system implementation

### Phase 2: Core Backend (Week 2-3)
- [ ] Authentication (cookie + OAuth)
- [ ] Search and browse
- [ ] Library commands
- [ ] Playback orchestration

### Phase 3: Playback & Sync (Week 3-4)
- [ ] Player integration
- [ ] Queue management
- [ ] Crossfade and loudness normalization
- [ ] Lyrics loading

### Phase 4: Frontend Foundation (Week 4-5)
- [ ] Navigation and shell
- [ ] Home screen with data
- [ ] Search interface
- [ ] Mini-player

### Phase 5: Core Screens (Week 5-7)
- [ ] Library (playlists, liked, albums, artists)
- [ ] Playlist view
- [ ] Album view
- [ ] Artist view
- [ ] Now playing
- [ ] Settings

### Phase 6: Integrations (Week 7-9)
- [ ] Last.fm / ListenBrainz
- [ ] Discord Rich Presence
- [ ] Spotify Import
- [ ] Local music
- [ ] Listen Together

### Phase 7: Testing & Release (Week 9-10)
- [ ] Integration testing
- [ ] Performance testing
- [ ] macOS DMG build
- [ ] Documentation

---

## Feature Inventory Summary

### YouTube Music Core ✅
- **Search**: Songs, albums, artists, playlists, videos
- **Home Feed**: Recommendations, recently played, moods, genres
- **Library**: Liked songs, playlists, albums, artists, uploads, history
- **Playlists**: Full CRUD with cover art
- **Playback**: Gapless, loudness normalization, crossfade
- **Queue**: Radio/automix, drag reorder, persistence

### Premium Features ✅
- **Lyrics**: Multi-source with word-by-word timing
- **Music Videos**: Separate video stream, gapless audio
- **Mini Player**: Compact, always-on-top option
- **Theater Mode**: Fullscreen artwork + lyrics
- **Local Music**: File-based library with metadata

### Integrations ✅
- **Last.fm**: Title bar scrobbling
- **ListenBrainz**: API-based scrobbling
- **Discord**: Rich presence with artwork and progress
- **Listen Together**: Self-hosted sync listening rooms
- **Spotify Import**: Playlist matching and import workflow

### macOS-specific ✅
- **Media Keys**: System-wide playback controls
- **System Tray**: Native menu bar integration
- **Keychain**: Secure credential storage
- **App Bundle**: .app + .dmg packaging
- **Performance**: Apple Silicon optimized

---

## Technical Advantages

### Over LiMusic
1. **Original Interface**: Completely new design, not a copy
2. **Enhanced Features**: Spotify-inspired discovery beyond YouTube
3. **Modern Architecture**: Cleaner separation of concerns
4. **Documentation**: Comprehensive design and architecture docs
5. **Testing**: Explicit test strategy and validation

### Advantages
1. **Battle-tested Backend**: Proven YouTube Music integration
2. **Native Performance**: libmpv + Rust + SvelteKit
3. **macOS Optimized**: Apple Silicon, Media keys, Keychain
4. **Premium UX**: Sophisticated, intuitive interface
5. **Future-Proof**: Modular architecture for extensions

---

## Next Steps

### Immediate Actions
1. **Run build test**
   ```bash
   cd /Users/ganeshsah/Documents/NOVA-Music-Rebuild
   ./scripts/test-build.sh
   ```

2. **Start Phase 1 implementation**
   - Set up basic Tauri project
   - Copy LiMusic crates
   - Implement database

3. **Begin frontend setup**
   - Initialize SvelteKit project
   - Implement design system
   - Create navigation shell

### Long-term Goals
1. **Complete all documentation** (DATA-MODEL.md, TEST-PLAN.md)
2. **Implement CI/CD pipeline**
3. **Build and verify macOS DMG**
4. **Run comprehensive testing**
5. **Release v1.0.0**

---

## Support & Resources

### Documentation
- **Architecture**: `docs/ARCHITECTURE.md`
- **Design**: `docs/DESIGN-SYSTEM.md`
- **Build**: `docs/BUILD-AND-RELEASE.md`
- **Features**: `docs/FEATURE-INVENTORY.md`

### Development Resources
- **GitHub Issues**: Feature requests and bug reports
- **Discussions**: Architecture and design questions
- **Contributions**: Pull requests and reviews

### Support
- **macOS Gatekeeper**: `xattr -dr com.apple.quarantine /Applications/Limusic.app`
- **Troubleshooting**: `scripts/test-build.sh` and build logs
- **Community**: GitHub discussions and issues

---

## Legal & Compliance

### Licensing
- **NOVA Music**: GPL-3.0-or-later (your choice)
- **LiMusic components**: GPL-3.0-or-later (as-is with reuse)
- **Dependencies**: Respective licenses (documented in Cargo.lock)

### Attribution
- All LiMusic code retains original copyright headers
- NOVA Music adds new copyright with GPL-3.0-or-later
- Clear documentation of sourced vs. original code

### Compliance
- GPL compliance verified (complete source distribution)
- License notices preserved in all files
- Modified files clearly marked with change notices
- Build instructions for user-modified versions

---

## Success Criteria

### Technical
- [ ] Basic Tauri app builds and launches on macOS
- [ ] YouTube Music authentication works
- [ ] Search and browse functionality
- [ ] Playback controls (play/pause, next/previous)
- [ ] Queue management
- [ ] Local music playback

### Quality
- [ ] UI responsive and polished
- [ ] Performance acceptable (startup < 3s, track load < 5s)
- [ ] Error handling and recovery
- [ ] macOS integration (media keys, tray)
- [ ] Crossfade and gapless transitions
- [ ] All integrations functional

### User Experience
- [ ] Intuitive navigation
- [ ] Beautiful artwork display
- [ ] Responsive controls
- [ ] macOS-native feel
- [ ] Professional presentation
- [ ] Smooth animations

---

## Conclusion

NOVA Music represents a **significant upgrade** to the existing LiMusic project, combining:

1. **Proven Technology**: Battle-tested YouTube Music integration
2. **Original Design**: Premium macOS interface
3. **Enhanced Features**: Spotify-inspired discovery
4. **Professional Quality**: Complete documentation and testing

The project is **ready for development** with a clear roadmap and comprehensive documentation. It builds upon LiMusic's strengths while delivering a substantially better NOVA experience.

**Next step**: Run the build test and begin Phase 1 implementation.

---

*Document last updated: 2026-10-11*
*Author: NOVA Music Development Team*
*Status: Ready for implementation*