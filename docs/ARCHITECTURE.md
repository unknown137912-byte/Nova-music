# NOVA Music - Architecture

This document defines the system architecture for NOVA Music, a native macOS desktop application built with Tauri (Rust + SvelteKit).

---

## 1. Overview

NOVA Music is a high-performance, feature-rich YouTube Music client optimized for Apple Silicon macOS. It combines the proven YouTube Music integration from LiMusic with an original, premium interface design and Spotify-inspired discovery features.

**Key Design Principles**:
- **Performance**: libmpv backend, SQLite database, async Rust, minimal overhead
- **Reliability**: Multi-client fallback for authentication, robust error recovery
- **User Experience**: Native macOS feel, intuitive interface, responsive playback
- **Privacy**: Secure authentication storage, optional integrations
- **Maintainability**: Clear module boundaries, comprehensive testing, well-documented APIs

---

## 2. System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Backend       │    │   Native        │
│   (SvelteKit)   │◄──►│   (Rust/Tauri)  │◄──►│   Components    │
│                 │    │                 │    │                 │
│   ui/           │    │   src-tauri/    │    │   libmpv,      │
│   ┌─────┐       │    │   ┌─────┐       │    │   souvlaki,    │
│   │     │       │    │   │     │       │    │   Keychain,    │
│   │ SPA │       │    │   │ App │       │    │   etc.         │
│   │     │       │    │   │     │       │    │                 │
│   └─────┘       │    │   └─────┘       │    └─────────────────┘
└─────────────────┘    └─────────────────┘
```

### Frontend Layer (ui/)
- **Framework**: SvelteKit (TypeScript, Svelte 5 runes)
- **Routing**: File-based routing, client-side navigation
- **State Management**: Central stores with derived signals
- **Components**: Custom UI library based on shadcn-svelte + bits-ui
- **Styling**: Tailwind CSS v4 with custom design tokens
- **Architecture**: Data flow unidirectional, effects managed by Svelte

### Backend Layer (src-tauri/)
- **Runtime**: Tauri v2 (Rust core, native bridge)
- **Commands**: `src/commands.rs` - public API for UI
- **State**: `src/state.rs` - Application-wide reactive state
- **Database**: SQLite with Rust ORM (`rusqlite`)
- **Playback**: libmpv2 wrapper (`player` crate) with dual-deck crossfade
- **Authentication**: Session management, OAuth, cookie handling
- **Integrations**: External services via dedicated plugins
- **IPC**: Tauri's command boundary, type-safe bridge

### Native Layer (macOS-specific)
- **Window System**: Tauri with macOS-native window config
- **Media Keys**: `souvlaki` MPRemoteCommandCenter integration
- **Keychain**: Secure storage for authentication tokens
- **App Bundle**: .app + .dmg packaging with dylibbundler
- **Signing**: Ad-hoc signing for distribution

---

## 3. Module Structure

### Backend Modules (`src-tauri/src/`)

#### Core Systems
- **`lib.rs`** - Root module, setup, logging, platform initialization
- **`commands.rs`** - Public Tauri commands (API boundary)
- **`state.rs`** - Global application state (auth, player, queue, settings)
- **`orchestrator.rs`** - High-level command composition
- **`db.rs`** - Database schema, migrations, queries

#### Playback Engine
- **`playback.rs`** - Main player orchestration (`Player` interface)
- **`player.rs`** - libmpv wrapper (decks, crossfade, cues)
- **`audio_proxy.rs`** - Loopback HTTP server for authenticated streams
- **`video_renderer.rs`** - Video embedding, GL context management

#### Authentication & Network
- **`auth.rs`** - Authentication flow, session persistence
- **`http.rs`** - HTTP client with proxy support, cipher/PoToken
- **`session.rs`** - InnerTube session management
- **`cipher/`** - Signature cipher and player-cipher registry
- **`potoken/`** - BotGuard attestation, JavaScript runtime

#### External Integrations
- **`lastfm.rs`** - Last.fm API (scrobbling)
- **`listenbrainz.rs`** - ListenBrainz API
- **`discord.rs`** - Discord Rich Presence
- **`listentogether/`** - Listen Together protocol (WebSocket)
- **`spotify_import.rs`** - Spotify playlist import workflow

#### Local Features
- **`local_music.rs`** - Local file library, metadata extraction
- **`lyrics.rs`** - Multi-source lyrics lookup and caching
- **`hotkeys.rs`** - Global and in-app keyboard shortcuts
- **`tray.rs`** - System tray with playback controls
- **`notifications.rs`** - macOS notifications
- **`diagnostics.rs`** - System diagnostics, error reporting

#### UI Bridge
- **`mini_player.rs`** - Mini-player webview
- **`cipher_webview.rs`** - Hidden webview for JavaScript execution
- **`videoproxy.rs`** - Loopback for music videos

#### Native Integration
- **`souvlaki.rs`** - Media key and system integration
- **`keychain.rs`** - Secure credential storage

---

## 4. Data Flow

### Authentication Flow
1. UI shows sign-in options (Google OAuth, cookie paste)
2. Tauri `sign_in` command launches OAuth webview (`hotkeys.rs`)
3. OAuth redirects with code → exchange for access token
4. Extract cookies/visitorData/sessionToken via cipher
5. Store in macOS Keychain + SQLite settings table
6. UI updates to authenticated state, loads library

### Playback Flow
1. UI queue change → `play_track` command
2. Orchestrator resolves URL → headers via `http.rs`
3. Player loads URL on active deck (`playback.rs`)
4. mpv events → `PlayerEvent` → `AppState::push_event`
5. UI updates position, playing state via `State::subscribe`
6. AudioDeviceLost/TrackFailed → recovery logic

### Search Flow
1. UI search input (debounced) → `search` command
2. Orchestrator calls InnerTube search endpoint
3. Results cached in SQLite + UI search store
4. UI renders search results with preview thumbnails

### Import Flow (Spotify)
1. UI navigates to import page → `start_import` command
2. Import service fetches Spotify playlist metadata
3. Track matching → user review (confidence scores)
4. Create YouTube Music playlist → store import history
5. Report skipped/unmatched tracks

### Lyrics Flow
1. Track start → lookup in `lyrics.rs` cache
2. Multi-source fetch (Boidu, LRCLIB, YTM, fallback)
3. Cache result → UI lyrics store
4. UI displays synced lyrics, auto-scroll

---

## 5. Database Schema

### Core Tables

```sql
-- Accounts: Multiple Google accounts
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    cookie TEXT,                    -- Encrypted
    visitor_data TEXT,
    data_sync_id TEXT,
    refresh_token TEXT,             -- Encrypted
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP,
    is_default BOOLEAN DEFAULT FALSE
);

-- Application settings (key-value)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT,
    account_id INTEGER,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- Playlists (user-created and imported)
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,             -- YouTube Music playlist ID
    account_id INTEGER,
    title TEXT NOT NULL,
    description TEXT,
    thumbnail_url TEXT,
    track_count INTEGER,
    is_local BOOLEAN DEFAULT FALSE,   -- For custom collections
    is_smart BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- Playlist tracks (ordered)
CREATE TABLE playlist_tracks (
    playlist_id TEXT,
    track_id TEXT,
    position INTEGER NOT NULL,
    added_by INTEGER,                 -- User ID, NULL for imported
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_modified TIMESTAMP,
    FOREIGN KEY (playlist_id) REFERENCES playlists(id),
    FOREIGN KEY (added_by) REFERENCES accounts(id)
);

-- History (YouTube Music account)
CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER,
    video_id TEXT,
    title TEXT,
    artist TEXT,
    album TEXT,
    thumbnail_url TEXT,
    duration_seconds REAL,
    played_at TIMESTAMP,
    play_seconds REAL,               -- For midpoint scrobbling
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- Cached metadata (artwork, search results, lyrics)
CREATE TABLE cache (
    key TEXT PRIMARY KEY,            -- Hash of request URL/params
    value BLOB NOT NULL,             -- Compressed JSON or raw data
    content_type TEXT,
    expires_at TIMESTAMP,
    accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Lyrics cache
CREATE TABLE lyrics (
    track_id TEXT PRIMARY KEY,        -- videoId + length hash
    source TEXT NOT NULL,
    content TEXT NOT NULL,
    synced BOOLEAN DEFAULT FALSE,
    word_timings TEXT,                -- JSON array of {time, word}
    translations TEXT,                -- JSON object {lang: text}
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Import history (Spotify, etc.)
CREATE TABLE import_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,             -- spotify, youtube_playlist, etc.
    source_id TEXT,                   -- External ID
    source_title TEXT,
    account_id INTEGER,
    status TEXT NOT NULL,             -- pending, in_progress, completed, failed
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

-- Import mappings (source track → target track)
CREATE TABLE import_mappings (
    session_id INTEGER,
    source_track_id TEXT,
    source_title TEXT,
    source_artist TEXT,
    source_album TEXT,
    confidence REAL,                  -- 0.0-1.0 match score
    status TEXT NOT NULL,             -- matched, skipped, user_reviewed, created
    target_playlist_id TEXT,           -- YouTube Music playlist ID
    target_track_id TEXT,              -- Matched track ID (if any)
    FOREIGN KEY (session_id) REFERENCES import_sessions(id)
);
```

---

## 6. Playback Architecture

### libmpv Integration
- **Why libmpv**: Industry standard for audio playback, gapless transitions, loudness normalization, cross-platform
- **Why not web audio**: Better audio quality, lower CPU, proper pause/resume semantics
- **MacOS Optimization**: Apple Silicon acceleration, Audio Unit integration

### Player Architecture
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   UI Player     │◄──►│   Orchestrator  │◄──►│    Player       │
│   (Svelte)      │    │  (playback.rs) │    │  (crates/player)│
│                 │    │                 │    │                 │
│   Queue +      │    │   - Load track  │    │   - Deck A (active)│
│   Controls     │    │   - Enqueue     │    │   - Deck B (preload)│
│                 │    │   - Seek        │    │   - mpv core      │
│   Play/Pause   │    │   - Crossfade   │    │   - Event loop    │
│   Volume       │    │   - Skip/Queue  │    │   - Properties   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Deck Architecture
- **Deck A**: Active playback, visible to UI
- **Deck B**: Preloaded for crossfade, hidden from UI
- **Crossfade**: Volume ramp from Deck A to Deck B over N seconds
- **Queue**: Gapless transitions use mpv's own playlist (`loadfile append`)
- **Overlap**: Only active Deck receives UI commands

### Error Recovery
- **Network**: Retry with exponential backoff, fallback clients
- **Audio Device**: `AudioDeviceLost` → reinitialize output device
- **TrackFailed**: Attempt new resolution, skip if 403/5xx
- **LookaheadFailed**: Evict cached URL, try next track
- **VideoFailed**: Fallback to audio-only, optional video stream

---

## 7. Frontend Architecture

### SvelteKit Structure
```
ui/
├── src/
│   ├── routes/
│   │   ├── +layout.svelte           - Shell, navigation, providers
│   │   ├── +layout.ts               - Layout data loading
│   │   ├── +page.svelte             - Home page
│   │   ├── search/+page.svelte      - Search results
│   │   ├── library/+page.svelte     - Library root
│   │   ├── playlists/               - Playlist subroutes
│   │   ├── albums/                  - Album routes
│   │   ├── artists/                 - Artist routes
│   │   ├── player/+layout.svelte    - Full player layout
│   │   ├── player/+page.svelte      - Now playing
│   │   ├── settings/+page.svelte    - User settings
│   │   └── import/+page.svelte      - Import workflow
│   ├── lib/
│   │   ├── stores.ts              - Central stores (player, queue, ui)
│   │   ├── components/            - UI component library
│   │   ├── api/                    - Tauri command wrappers
│   │   ├── utils/                  - Shared utilities
│   │   └── themes/                - Design system tokens
│   ├── static/                    - Static assets
│   └── app.d.ts                    - Component types
├── package.json
├── vite.config.ts                  - Tauri + SvelteKit integration
└── tailwind.config.*              - Design tokens, variants
```

### Design System
- **Colors**: Sophisticated dark theme, accent palette, adaptive artwork
- **Typography**: System font stack, hierarchical sizing, accessibility
- **Spacing**: 4px grid, consistent margins/padding
- **Components**: Glassmorphism selectively, shadows, borders, hover states
- **Motion**: Reduced motion support, smooth transitions, micro-interactions

---

## 8. Native macOS Integration

### Media Keys
- **API**: `souvlaki` (MPNowPlayingInfoCenter + MPRemoteCommandCenter)
- **Features**: Play/pause/next/previous, volume, scrubber, now playing widget
- **Integration**: Works even when app is minimized

### System Tray
- **Menu**: Play/pause, next, previous, queue, settings, quit
- **Behavior**: Click to show/hide window, close to minimize
- **Icon**: Dynamic based on playback state, artwork when available

### App Bundle
- **Structure**: `.app/Contents/{MacOS,Resources,Frameworks}`
- **Frameworks**: libmpv and dependencies via `dylibbundler`
- **Signing**: Ad-hoc (`codesign --sign -`) for distribution
- **Quarantine**: User must `xattr -dr com.apple.quarantine` on first launch

### Keychain Integration
- **Storage**: macOS Keychain for sensitive data (cookies, tokens)
- **Encryption**: System-provided encryption
- **Access**: App-specific keychain access groups

---

## 9. Error Handling

### Architecture
- **Centralized**: Single error type hierarchy in backend
- **User-facing**: Toast notifications for recoverable errors
- **Logging**: Structured logs to `limusic.log` (truncated per launch)
- **Graceful degradation**: Fallback when external services unavailable

### Error Categories
- **Authentication**: Invalid credentials, expired tokens
- **Network**: 403/5xx from YouTube, timeouts, proxy failures
- **Playback**: Audio device lost, unsupported format, corrupted streams
- **UI**: Rendering errors, component state mismatches
- **System**: Permission denied, disk full, out of memory

### Recovery Strategies
- **Retry**: Exponential backoff with circuit breaker
- **Fallback**: Previous client identity, cached responses
- **User action**: Retry with new credentials, change network, contact support
- **Graceful degradation**: Disable features, show informative messages

---

## 10. Testing Strategy

### Unit Tests
- **Rust**: `libmpv2` bindings, database queries, cipher operations
- **Coverage**: Minimum 80% for core modules
- **Property testing**: Random inputs for edge cases

### Integration Tests
- **Authentication**: Cookie paste, OAuth flow simulation
- **Playback**: Real URL loading, error recovery scenarios
- **Database**: Migration tests, concurrent access
- **UI**: Component rendering, store behavior

### E2E Tests
- **Critical paths**: Search → play → queue → import
- **macOS specific**: Media keys, system tray, app bundle
- **Performance**: Large library loading, many concurrent tracks

---

## 11. Build & Release Process

### macOS Build Steps
1. **Dependencies**: `brew install mpv dylibbundler`
2. **Linker**: Set `LIBRARY_PATH=$(brew --prefix)/lib`
3. **Frontend**: `cd ui && pnpm build`
4. **Backend**: `cargo tauri build`
5. **Bundle**: `dylibbundler` walks dependency graph
6. **Fix rpaths**: Remove duplicate `@executable_path/../Frameworks`
7. **Sign**: `codesign --force --deep --sign -`
8. **Verify**: Check no Homebrew references remain
9. **DMG**: `hdiutil create -volname "Limusic" -srcfolder bundle -ov -format UDZO`

### Release Checklist
- [ ] All tests pass (unit, integration, e2e)
- [ ] Build on macOS (arm64)
- [ ] Bundle verification (no absolute paths)
- [ ] Code signing (ad-hoc)
- [ ] Gatekeeper documentation (`xattr -dr com.apple.quarantine`)
- [ ] DMG test install and launch
- [ ] Updater bundle generation
- [ ] Final artifact upload to GitHub Releases

---

## 12. Performance Optimization

### Database
- **Indexing**: Track ID, playlist position, cache key
- **Query batching**: Bulk operations for imports
- **Cache eviction**: LRU on cache table, TTL on artwork
- **Connection pooling**: One persistent SQLite connection

### Playback
- **Prefetch**: 1-track lookahead, crossfade preload
- **Buffering**: mpv's demuxer cache (64 MiB forward/back)
- **Gapless**: Single mpv instance per deck, internal transitions
- **Loudness**: Per-track attenuation, user-adjustable target

### Network
- **Connection reuse**: HTTP client pooling
- **Compression**: Brotli/gzip for API responses
- **Caching**: Cache API responses with ETag validation
- **Rate limiting**: Respect YouTube API limits

### UI
- **Virtual lists**: Only render visible tracks
- **Debounced search**: 300ms delay, cancel previous requests
- **Lazy loading**: Artwork on demand, thumbnails sized appropriately
- **WebSocket**: Real-time queue updates (for Listen Together)

---

## 13. Security

### Authentication
- **Cookies**: Stored in macOS Keychain, encrypted
- **Tokens**: Short-lived with refresh, secure storage
- **OAuth**: Redirect URIs validated, PKCE where supported
- **Session**: Expiration and renewal

### Data Protection
- **Input validation**: All external inputs sanitized
- **Output encoding**: XSS prevention in webviews
- **Access control**: Account isolation, per-user permissions
- **Audit logging**: Security-relevant actions logged

### Network
- **HTTPS**: All external communications TLS encrypted
- **Certificate pinning**: Against certificate authorities
- **Proxy support**: Corporate environments
- **Rate limiting**: Protect against abuse

### Storage
- **SQLite**: Encrypted database for sensitive data
- **Cache**: No sensitive data cached
- **Backups**: Optional user-controlled backups
- **Cleanup**: Secure deletion on uninstall

---

## 14. Migration from OLD NOVA to NOVA

### Data Migration
1. **Authentication**: Import cookies/tokens from old SQLite database
2. **Library**: Migrate playlists, liked songs, history
3. **Settings**: Copy playback, appearance, integration preferences
4. **Cache**: Preserve artwork, lyrics if possible

### Schema Changes
- **New tables**: Custom collections, smart playlists
- **Enhanced fields**: Better metadata, import tracking
- **Migrations**: Versioned SQL scripts, quarantine on error

### Feature Parity
- **Core features**: Search, playback, library, playlists (full parity)
- **Integrations**: Last.fm, ListenBrainz, Discord, Listen Together
- **Import**: Spotify (plus potential future sources)
- **Local music**: Enhanced with better metadata extraction

### UI Migration
- **Branding**: NOVA design system, original interface
- **Navigation**: Sidebar, mini-player, queue panel
- **Components**: New design language, consistent theming
- **Interactions**: Native macOS feel, refined gestures

---

## 15. Development Workflow

### Code Review
- **Branches**: Feature branches, PRs for every change
- **Peer review**: Minimum 2 reviewers for production code
- **Tests**: New code requires tests, existing tests must pass
- **Documentation**: Update docs, comments, and examples

### Build Process
```bash
# Development
cd ui && pnpm install && pnpm dev
cargo tauri dev

# Testing
cargo test --workspace
cargo clippy -- -D warnings
pnpm test

# Build
cd ui && pnpm build
cargo tauri build

# macOS specific (if needed)
export LIBRARY_PATH="$(brew --prefix)/lib:$LIBRARY_PATH"
brew install dylibbundler
cargo tauri build
# dylibbundler steps...
codesign --force --deep --sign -
```

### Environment Setup
```bash
# Required tools
rustup install stable
brew install mpv dylibbundler  # macOS
pnpm install  # UI deps

# Configuration
mkdir -p ~/.config/novamusic
# Edit settings via UI first launch

# Development
cargo tauri dev
```

---

## 16. Future Extensibility

### Modular Architecture
- **Plugins**: Pluggable architecture for new integrations
- **Services**: Separation of concerns, testability
- **Adapters**: External service clients
- **Commands**: Extensible command set

### Feature Expansion
- **New music sources**: Apple Music, Tidal, Bandcamp
- **Advanced features**: Radio algorithms, smart playlists
- **Collaboration**: Family sharing, multi-room playback
- **Analytics**: Detailed listening statistics, insights

### Platform Expansion
- **Windows**: SMTC media keys, native Windows packaging
- **Linux**: MPRIS media keys, AppImage, system tray
- **Mobile**: Consider React Native or Flutter (future)

---

## 17. Documentation

### Internal
- **ARCHITECTURE.md** (this document)
- **DATA-MODEL.md** (database schema, migrations)
- **DESIGN-SYSTEM.md** (visual design, components)
- **BUILD-AND-RELEASE.md** (development, packaging)
- **TEST-PLAN.md** (test cases, coverage, CI)

### User
- **README.md** (getting started, basic usage)
- **INSTALL.md** (platform-specific installation)
- **SETTINGS.md** (configuration guide)
- **SUPPORT.md** (troubleshooting, FAQs)
- **CHANGELOG.md** (version history)
- **LICENSE.md** (GPL-3.0, credits)

---

## 18. Project Health Metrics

### Code Quality
- **Lines of code**: ~500k (Rust) + ~200k (TypeScript)
- **Test coverage**: >80% for core modules
- **Cyclomatic complexity**: Keep under 20 for public APIs
- **Documentation**: All public APIs documented

### Performance
- **Startup time**: < 3 seconds
- **Track load**: < 2 seconds (cached), < 5 seconds (fresh)
- **Memory**: < 512 MiB for typical 10k-track library
- **CPU**: < 10% on idle, < 40% during playback

### Reliability
- **Uptime**: > 99.5% (production monitoring)
- **Recovery**: Automatic retry, graceful degradation
- **Error rates**: < 0.1% for user actions
- **Database**: ACID compliant, MVCC

---

## Conclusion

NOVA Music combines the proven YouTube Music integration from LiMusic with an original, premium macOS interface and Spotify-inspired features. The architecture emphasizes performance, reliability, and user experience while maintaining code quality and security.

**Key advantages**:
- Reuse of battle-tested YouTube Music backend
- Native macOS integration with modern Swift/Rust ecosystem
- Modular architecture for future expansion
- Comprehensive testing and error recovery
- GPL-3.0 licensing with clear compliance path

The project is ready for development following the phased implementation plan outlined in FEATURE-INVENTORY.md, leveraging the existing LiMusic codebase as a foundation while building a substantially better NOVA experience.