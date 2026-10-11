# NOVA Music - Feature Inventory

**Source**: LiMusic (https://github.com/SimoHypers/limusic) - GPL-3.0-or-later
**Investigation Date**: 2026-10-11
**Status**: Complete investigation of LiMusic source code, architecture, and macOS build process

---

## 1. YouTube Music Integration (Core)

| Capability | LiMusic Implementation | Reuse Strategy | NOVA Implementation Notes |
|------------|----------------------|----------------|---------------------------|
| **InnerTube API Client** | `crates/innertube` - Pure Rust client impersonating official clients (Android, iOS, Web, TV) | **Full reuse** - Copy crate as-is, update branding | Multi-client fallback, context-aware requests, locale support |
| **Authentication** | Cookie-based + Google OAuth webview (`session.rs`, `commands.rs`) | **Full reuse** - Adapt for NOVA branding | Secure cookie storage in macOS Keychain, session persistence |
| **Search** | `browse.rs` endpoint - Songs, albums, artists, playlists, videos, community playlists | **Full reuse** | Debounced search, category filters, responsive results |
| **Home Feed** | `browse.rs` - Personalized shelves (recommendations, recently played, moods, genres) | **Full reuse** | Rich discovery page with real service data |
| **Library** | `library.rs` - Liked songs, playlists, albums, artists, uploads, history | **Full reuse** | All library categories with real actions (like, save, manage) |
| **Playlists** | Full CRUD + cover art upload, collaborative editing where supported | **Full reuse** | Elegant playlist view with drag-drop reorder |
| **Playback URLs** | Signature cipher + n-parameter + PoToken (BotGuard) via hidden webview | **Full reuse** - Critical path | `cipher/` + `potoken/` modules, `webview.rs` harness |
| **Music Videos** | Separate video stream, played in webview alongside audio | **Full reuse** | Optional, gapless audio continues |
| **Lyrics** | Multi-source (Boidu, LRCLIB, YTM, Netease, QQ, Kugou) with word-by-word timing | **Full reuse** | Synced lyrics panel, auto-scroll, translations |
| **Radio/Automix** | `next` endpoint for automatic continuation | **Full reuse** | Queue radio mode, seamless transitions |

---

## 2. Playback Engine

| Capability | LiMusic Implementation | Reuse Strategy | NOVA Implementation Notes |
|------------|----------------------|----------------|---------------------------|
| **Core Player** | `crates/player` - libmpv2 wrapper, gapless, loudness normalization | **Full reuse** | Single `Player` struct with dual-deck crossfade support |
| **Gapless Playback** | mpv's `gapless-audio=yes` + `prefetch-playlist=yes` + 1-track lookahead | **Full reuse** | Zero-gap transitions, buffered next track |
| **Loudness Normalization** | YouTube's loudness metadata → mpv `volume` + `replaygain` filters | **Full reuse** | Per-track attenuation, user-adjustable target |
| **Crossfade** | Dual mpv instances, volume ramp on fade_gen | **Full reuse** | Optional, configurable duration |
| **Seeking** | `time-pos` property, instant cached seeks | **Full reuse** | Responsive scrubber, keyboard shortcuts |
| **Queue Management** | `orchestrator.rs` - Central queue state, persistence, radio | **Full reuse** | Drag-drop reorder, play next, add to queue |
| **Audio Proxy** | Loopback HTTP server for authenticated googlevideo streams | **Full reuse** | Required for cookie-authenticated streams |
| **Music Video Sync** | Separate video mpv instance, frame-accurate sync | **Full reuse** | Video in artwork area, toggleable |
| **Error Recovery** | TrackFailed → re-resolve URL → retry, AudioDeviceLost → reinit | **Full reuse** | User-facing toasts, automatic retry |

---

## 3. Local Database (SQLite)

| Capability | LiMusic Implementation (`db.rs`) | Reuse Strategy | NOVA Implementation Notes |
|------------|----------------------------------|----------------|---------------------------|
| **Schema** | Tables: accounts, settings, playlists, playlist_tracks, history, cache, lyrics, imports | **Full reuse with migration** | Add NOVA-specific tables (custom collections, smart playlists) |
| **Migrations** | Versioned SQL migrations, quarantine on corruption | **Full reuse** | Safe, testable migrations |
| **Accounts** | Multiple Google accounts, cookie/dataSyncId/visitorData per account | **Full reuse** | Secure storage in macOS Keychain |
| **Settings** | Key-value with typed getters (bool, int, string) | **Full reuse** | Theme, playback, integrations, shortcuts |
| **Cache** | Artwork, search results, metadata with TTL | **Full reuse** | LRU eviction, disk quota |
| **Lyrics Cache** | Cached by track ID + length, source priority | **Full reuse** | Instant replay, background refresh |
| **Spotify Import History** | Import sessions with match confidence, user corrections | **Full reuse** | Reviewable import workflow |
| **Listening Stats** | Local play counts, skip rates, session durations | **Extend** | Enhanced statistics dashboard |

---

## 4. External Integrations

| Integration | LiMusic Implementation | Reuse Strategy | NOVA Implementation Notes |
|-------------|----------------------|----------------|---------------------------|
| **Last.fm Scrobbling** | `lastfm.rs` - API sig (MD5), halfway rule, title bar connect button | **Full reuse** | Optional, own API keys required |
| **ListenBrainz** | `listenbrainz.rs` - User token, playing_now + single listen | **Full reuse** | Settings page connect flow |
| **Discord Rich Presence** | `discord.rs` - Artwork, progress bar, toggle in title bar | **Full reuse** | Optional, green dot indicator |
| **Listen Together** | `listentogether/` - WebSocket relay, join codes, host approval | **Full reuse** | Self-hosted relay option |
| **Spotify Import** | `spotify.rs` + `import.rs` - Link/CSV/export, match confidence, review UI | **Full reuse** | Dedicated import workflow screen |
| **Local Music** | `local.rs` - Folder picker, lofty metadata, integrated in library | **Full reuse** | Separate "Local Files" library section |
| **Keyboard Shortcuts** | `hotkeys.rs` + `shortcuts.ts` - Global + in-app, customizable | **Full reuse** | macOS media keys, custom shortcuts |
| **System Tray** | `tray.rs` - Play/pause/skip, start minimized, close to tray | **Full reuse** | Native macOS menu bar item |

---

## 5. Frontend (SvelteKit + TypeScript)

| Capability | LiMusic Implementation | Reuse Strategy | NOVA Implementation Notes |
|------------|----------------------|----------------|---------------------------|
| **Routing** | File-based: `/`, `/search`, `/library`, `/playlist/[id]`, `/album/[id]`, `/artist/[id]`, `/history`, `/list` | **Restructure** | New IA: Home, Search, Library, Playlist, Album, Artist, Now Playing, Settings |
| **State Management** | Svelte 5 runes (`$state`, `$derived`, `$effect`) in `.svelte.ts` stores | **Full reuse pattern** | Consistent stores: player, queue, library, search, settings |
| **Components** | shadcn-svelte + bits-ui + custom (rows, selection, queue, lyrics, video) | **Redesign** | New design system, consistent components |
| **Theming** | `theme.svelte.ts` - Accent palettes, custom colors, adaptive (artwork colors) | **Extend** | NOVA brand colors, sophisticated dark theme |
| **Internationalization** | 18 languages, Weblate, JSON catalogs | **Reuse infrastructure** | English first, i18n ready |
| **Mini Player** | `mini.rs` + `mini.svelte` - Separate webview, fixed size | **Full reuse** | Compact, always-on-top option |
| **Theater Mode** | Fullscreen artwork + lyrics side-by-side | **Redesign** | Immersive now-playing experience |
| **Accessibility** | Keyboard nav, focus management, reduced motion | **Maintain** | WCAG AA compliance |

---

## 6. macOS-Specific Features

| Capability | LiMusic Implementation | Reuse Strategy | NOVA Implementation Notes |
|------------|----------------------|----------------|---------------------------|
| **App Bundle** | Tauri `app` + `dmg` targets, `tauri.macos.conf.json` | **Full reuse** | NOVA branding, custom icons |
| **Window Config** | Overlay titlebar, traffic lights, hidden title | **Full reuse** | Native macOS feel |
| **Media Keys** | `souvlaki` with `MPNowPlayingInfoCenter` / `MPRemoteCommandCenter` | **Full reuse** | Play/pause/next/prev, scrubber |
| **Menu Bar** | Tray icon with playback controls | **Full reuse** | Native NSStatusItem |
| **Keychain** | Not yet implemented (cookies in SQLite) | **New** | Store cookies/tokens in Keychain |
| **Notifications** | `notify-rust` - NSUserNotification | **Full reuse** | Track change, import complete |
| **Signing/Notarization** | Ad-hoc CI, manual notarization for releases | **Document** | Clear instructions for distribution |
| **macOS support** | Apple Silicon ARM64 only | **ARM64 only** | Intel Macs are unsupported |

---

## 7. Build & Release

| Capability | LiMusic Implementation | Reuse Strategy | NOVA Implementation Notes |
|------------|----------------------|----------------|---------------------------|
| **CI/CD** | GitHub Actions: linux-release, windows-release, macos-release | **Adapt** | NOVA-specific workflows, versioning |
| **macOS Build** | `dylibbundler` for libmpv + deps, re-sign, DMG via `hdiutil` | **Full reuse** | Documented in BUILD-AND-RELEASE.md |
| **Updater** | Tauri updater + custom signing, `latest.json` | **Full reuse** | Self-updating DMG |
| **Versioning** | Workspace version in `Cargo.toml` | **Full reuse** | Semantic versioning |

---

## 8. GPL-3.0 Compliance

| Requirement | LiMusic Approach | NOVA Obligation |
|-------------|------------------|-----------------|
| **Source Distribution** | All source on GitHub | Must provide complete corresponding source |
| **License Notices** | GPL-3.0 headers in all files | Preserve all copyright/license headers |
| **Modified Files** | N/A (original) | Mark modified files with change notices |
| **Installation Info** | Build docs in repo | Provide build instructions for user-modified versions |
| **Copyleft** | Entire work GPL-3.0 | NOVA must be GPL-3.0-or-later |

**Strategy**: NOVA Music will be licensed GPL-3.0-or-later. All LiMusic code reused will retain original copyright headers. New NOVA code will carry NOVA copyright with GPL-3.0-or-later license.

---

## 9. Implementation Priority (Phased)

### Phase 1: Foundation (Week 1-2)
- [ ] Project structure, Cargo workspace, Tauri config
- [ ] Copy `innertube`, `player`, `listen-protocol` crates
- [ ] Basic Tauri app builds on macOS (ARM64)
- [ ] macOS build script with `dylibbundler`
- [ ] SQLite database with migrations

### Phase 2: Core Backend (Week 2-3)
- [ ] Authentication flow (cookie + OAuth)
- [ ] Search, browse, home feed commands
- [ ] Library commands (liked, playlists, albums, artists)
- [ ] Playback orchestration (load, enqueue, queue)
- [ ] Audio proxy for authenticated streams

### Phase 3: Playback & Sync (Week 3-4)
- [ ] Player integration (PlayerEvent → frontend)
- [ ] Gapless, loudness, crossfade
- [ ] Queue persistence, radio
- [ ] Lyrics loading + caching
- [ ] Music video support

### Phase 4: Frontend Foundation (Week 4-5)
- [ ] Design system (tokens, typography, components)
- [ ] Navigation shell (sidebar, mini-player, queue panel)
- [ ] Home screen with real data
- [ ] Search screen with debounced results

### Phase 5: Core Screens (Week 5-7)
- [ ] Library (playlists, liked, albums, artists)
- [ ] Playlist view (header, tracks, actions)
- [ ] Album view (artwork, metadata, tracks)
- [ ] Artist view (top songs, albums, related)
- [ ] Now Playing (full player, lyrics, queue)
- [ ] Mini-player

### Phase 6: Integrations & Polish (Week 7-9)
- [ ] Settings (playback, appearance, integrations, shortcuts)
- [ ] Last.fm / ListenBrainz / Discord
- [ ] Spotify Import workflow
- [ ] Local music library
- [ ] Listen Together
- [ ] Keyboard shortcuts, media keys

### Phase 7: Testing & Release (Week 9-10)
- [ ] Integration tests (auth, playback, search, import)
- [ ] Performance testing (large libraries, long queues)
- [ ] macOS DMG build verification
- [ ] Gatekeeper/ad-hoc signing documentation
- [ ] Release checklist

---

## 10. Technical Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| YouTube API changes break InnerTube | Medium | High | Multiple client fallbacks, runtime cipher registry update |
| libmpv build/link issues on macOS | Medium | High | Documented dylibbundler process, CI verification |
| GPL compliance complexity | Low | High | Clear license headers, build-from-source docs |
| Large library performance | Medium | Medium | Virtualized lists, indexed DB, pagination |
| macOS Gatekeeper quarantine | Certain | Medium | Document `xattr -dr com.apple.quarantine` for users |
| Cipher/PoToken maintenance | Ongoing | Medium | Bake registry, runtime fetch fallback |
| Webview memory (Linux) | Low (macOS) | N/A | WebKit tuning not needed on macOS |

---

## 11. File Mapping: LiMusic → NOVA

```
LiMusic                          → NOVA Music
├── crates/innertube/            → src-tauri/crates/innertube/ (copy)
├── crates/player/               → src-tauri/crates/player/ (copy)
├── crates/listen-protocol/      → src-tauri/crates/listen-protocol/ (copy)
├── crates/sync-server/          → src-tauri/crates/sync-server/ (copy)
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs              → src-tauri/src/lib.rs (adapt)
│   │   ├── main.rs             → src-tauri/src/main.rs (adapt)
│   │   ├── commands.rs         → src-tauri/src/commands.rs (adapt)
│   │   ├── state.rs            → src-tauri/src/state.rs (adapt)
│   │   ├── orchestrator.rs     → src-tauri/src/orchestrator.rs (adapt)
│   │   ├── db.rs               → src-tauri/src/db.rs (extend)
│   │   ├── session.rs          → src-tauri/src/auth.rs (rename, adapt)
│   │   ├── media.rs            → src-tauri/src/playback.rs (rename)
│   │   ├── lyrics.rs           → src-tauri/src/lyrics.rs (adapt)
│   │   ├── import.rs           → src-tauri/src/spotify_import.rs (rename)
│   │   ├── local.rs            → src-tauri/src/local_music.rs (rename)
│   │   ├── lastfm.rs           → src-tauri/src/lastfm.rs (copy)
│   │   ├── listenbrainz.rs     → src-tauri/src/listenbrainz.rs (copy)
│   │   ├── discord.rs          → src-tauri/src/discord.rs (copy)
│   │   ├── listentogether/     → src-tauri/src/listen_together/ (copy)
│   │   ├── hotkeys.rs          → src-tauri/src/hotkeys.rs (adapt)
│   │   ├── tray.rs             → src-tauri/src/tray.rs (adapt)
│   │   ├── notify.rs           → src-tauri/src/notifications.rs (rename)
│   │   ├── cipher/             → src-tauri/src/cipher/ (copy)
│   │   ├── potoken/            → src-tauri/src/potoken/ (copy)
│   │   ├── http.rs             → src-tauri/src/http.rs (copy)
│   │   ├── audioproxy.rs       → src-tauri/src/audio_proxy.rs (rename)
│   │   ├── videoproxy.rs       → src-tauri/src/video_proxy.rs (rename)
│   │   ├── webview.rs          → src-tauri/src/cipher_webview.rs (rename)
│   │   ├── diagnostics.rs      → src-tauri/src/diagnostics.rs (copy)
│   │   ├── romanize.rs         → src-tauri/src/romanize.rs (copy)
│   │   ├── blocked.rs          → src-tauri/src/blocked.rs (copy)
│   │   ├── appicon.rs          → src-tauri/src/app_icon.rs (rename)
│   │   ├── mini.rs             → src-tauri/src/mini_player.rs (rename)
│   │   └── nativevideo*.rs     → src-tauri/src/video_renderer.rs (adapt)
│   ├── Cargo.toml              → src-tauri/Cargo.toml (adapt)
│   ├── tauri.conf.json         → src-tauri/tauri.conf.json (NOVA branding)
│   ├── tauri.macos.conf.json   → src-tauri/tauri.macos.conf.json (adapt)
│   └── build.rs                → src-tauri/build.rs (adapt)
├── ui/
│   ├── src/
│   │   ├── routes/             → ui/src/routes/ (redesign IA)
│   │   ├── lib/                → ui/src/lib/ (new design system)
│   │   └── app.d.ts            → ui/src/app.d.ts (adapt)
│   ├── package.json            → ui/package.json (NOVA deps)
│   ├── vite.config.ts          → ui/vite.config.ts (adapt)
│   └── tailwind.config.*       → ui/tailwind.config.* (NOVA design tokens)
└── docs/                       → docs/ (new documentation)
```

---

## 12. Next Steps

1. **Create ARCHITECTURE.md** - System design, module boundaries, data flow
2. **Create DATA-MODEL.md** - Database schema, entity relationships, migration plan
3. **Create DESIGN-SYSTEM.md** - Visual tokens, components, interaction principles
4. **Create BUILD-AND-RELEASE.md** - macOS build process, dylibbundler, signing, DMG
5. **Create TEST-PLAN.md** - Test cases, automation strategy, verification checklist
6. **Initialize project** - Cargo workspace, Tauri config, SvelteKit setup
7. **Copy and adapt LiMusic crates** - innertube, player, listen-protocol
8. **Implement macOS build script** - Verify libmpv linking and bundling
