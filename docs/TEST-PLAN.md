# NOVA Music - Test Plan

## Scope

This plan covers verification of the Rust backend (Tauri), SQLite data model, authentication flow, playback, queue, search, library, integrations (Spotify import, Last.fm/ListenBrainz/Discord), and macOS packaging (.app, .dmg). Tests are a mix of unit tests (Rust), integration tests (Tauri commands), and manual macOS verification.

---

## 1. Unit Tests (Rust)

| Module | Test Cases | Purpose |
|--------|------------|---------|
| `db.rs` | - `open_database` creates file if missing<br>- `migrate_to_latest` runs all up‑scripts<br>- CRUD for each table (insert, get, update, delete)<br>- Index creation and query performance<br>- Encryption wrapper (Keychain) round‑trip | Ensure schema integrity, migration safety, data privacy. |
| `player.rs` | - `Player::new` with valid `libmpv` handle<br>- `play`, `pause`, `next`, `prev`, `seek`<br>- Gapless transition measurement (assert gap < 50 ms)<br>- Loudness normalization toggle<br>- Error recovery on `AudioDeviceLost` | Verify playback engine reliability. |
| `auth.rs` | - `sign_in` with Google OAuth URL, exchange code<br>- `sign_out` clears Keychain entry<br>- `switch_account` validates new cookie<br>- Token refresh handling | Authentication correctness and Keychain integration. |
| `lyrics.rs` | - Multi‑source fetch (Boidu first, fallback LRCLIB)<br>- Cached result TTL enforcement<br>- Word‑by‑word highlight indexing | Lyrics loading and caching. |
| `spotify_import.rs` | - Parse CSV/URI input<br>- Search YTM for each track<br>- Compute confidence tier (`exact`/`probable`/`unmatched`)<br>- Persist `import_matches` rows | Import pipeline correctness. |
| `queue.rs` | - `enqueue`, `dequeue`, `reorder` (drag‑drop simulation)<br>- Persist across restarts<br>- Radio/automix continuation | Queue management. |

Run with `cargo test --all-features` (or `--lib` for lib‑only). All must pass before proceeding.

---

## 2. Integration Tests (Tauri Commands)

| Command | Test Scenario | Success Criteria |
|---------|---------------|------------------|
| `auth::sign_in` | Launch auth webview, complete OAuth with test account, verify `Keychain` entry and `Db::account_key` | App returns signed‑in state; subsequent `auth::get_current_session` yields account ID. |
| `auth::sign_out` | Call sign_out, then `get_current_session` returns none, Keychain entry removed | Clean state, no stale cookies. |
| `search::query` | Send query "Bohemian Rhapsody", receive first 20 results with `video_id`, `title`, `artist` | Non‑empty list; each result has required fields. |
| `library::liked_songs` | Fetch liked songs page, verify items contain track metadata | Pagination works; total count matches YTM (within tolerance). |
| `library::playlists` | List user playlists, verify CRUD (create, rename, delete) | Playlist objects persisted; YTM API reflects changes. |
| `playback::load_track` | Load a track URI, start playback, verify `PlayerEvent::TrackStart` emitted | Playback begins; `current_track` updates in UI. |
| `playback::pause` / `play` | Toggle pause, verify `PlayerEvent::TrackPaused` / `TrackResumed` | State flips correctly. |
| `playback::seek` | Seek to 30 s into track, verify `PlayerEvent::Seek` and updated `time_pos` | New position close to 30 s (±2 s). |
| `queue::add_next` / `add_end` | Add track to queue, verify queue length and order | Queue reflects addition; playback proceeds to next track. |
| `lyrics::load` | Play a track with synced lyrics, open lyrics panel, verify auto‑scroll and word highlight | Lyrics panel shows timed lyrics; clicking jumps. |
| `spotify_import::start` | Provide a public Spotify playlist URL, watch import wizard, verify `import_matches` rows created with tiers | UI shows match confidence; user can accept/reject. |
| `lastfm::scrobble` | Play a track, verify scrobble POST to Last.fm (using test API key) | Scrobble endpoint returns `200`. |
| `discord::presence` | Toggle Rich Presence, verify Discord RPC updates (check Discord app) | Presence shows track, artist, progress bar. |
| `media_keys::handle` | Simulate macOS media‑key events (play/pause/next/prev) via `souvlaki`/`MPRemoteCommandCenter` | Track controls change playback. |

Integration tests use `tauri::test` harness (headless browser) where possible; otherwise they are exercised manually on a test macOS build.

---

## 3. macOS Packaging & Release Tests

| Test | Procedure | Expected Outcome |
|------|-----------|------------------|
| `build::debug` | `cargo tauri build --debug` on Apple Silicon MacBook Air M4 | Build succeeds; binary runs; no linker `library 'mpv' not found` error. |
| `build::release` | `cargo tauri build --release` | Produces `target/release/bundle/app.nova.app`. |
| `dmg::generate` | Run `cargo tauri generate-app-dmg` (or custom script) | `.dmg` contains `NOVA Music.app`;.app launches after drag‑to‑Applications. |
| `gatekeeper::verify` | Right‑click → Open (first launch) or `xattr -dr com.apple.quarantine` then launch | Application launches without “unable to verify developer” dialog (if signed/notarized). |
| `notarization::check` | If codesign + notarization keys available, verify `codesign --verify --deep` and `altool --notification` | No notarization warnings. |
| `startup::performance` | Measure time from launch to first `PlayerEvent::TrackStart` (target < 2 s on M4) | Startup within budget. |
| `memory::profile` | Run instrumentary `instruments --allocations` for 5 min of playback with 5 k‑track library | Memory growth < 150 MiB; no leaks. |
| `ui:: regression` | Take screenshots of each core screen (Home, Search, Library, Now Playing, Mini‑player) and compare against baseline (pixel diff < 5 %) | Visual consistency. |

---

## 4. Manual Verification Checklist (run on a fresh M4 macOS install)

- [ ] Application opens from .dmg, window respects system dark mode.
- [ ] Sign‑in via Google OAuth works; account can be switched.
- [ ] Search returns real YTM results; debounce visible.
- [ ] Liked songs, playlists, albums, artists load with real data.
- [ ] Playback: play/pause/next/prev/seeking all functional; gapless transition audible.
- [ ] Queue can be reordered; persistence across restarts.
- [ ] Lyrics panel appears for tracks with synchronized lyrics; auto‑scroll works.
- [ ] Mini‑player stays on top, controls functional.
- [ ] Media keys (Fn‑arrow, F7/F8/F9) control playback.
- [ ] System tray/minimise‑to‑tray works; click restores window.
- [ ] Last.fm/ListenBrainz scrobbling enabled after API key entry.
- [ ] Discord Rich Presence shows current track; toggles on/off.
- [ ] Spotify import wizard: paste playlist link, matches displayed, user can correct, final playlist created in YTM.
- [ ] Listen Together: start relay, two clients join, playback sync.
- [ ] Settings: theme, playback, integrations, shortcuts saved and restored on restart.
- [ ] Application quit and relaunch retains state (database, login, queue).

---

## 5. Test Automation Strategy

1. **CI pipeline** (GitHub Actions) runs `cargo test --all-features` on Linux, Windows, and macOS (ARM64) nightly.
2. **Unit test coverage target** >= 80 % (measured by `cargo tarpaulin` or `cargo nextest coverage`).
3. **Integration test matrix** runs headless Tauri tests via `tauri test` on each OS; failures block merge.
4. **Nightly performance bot** builds debug, runs a synthetic “play 100 tracks, seek, queue changes” scenario and records time/memory metrics; fails if thresholds exceeded.
5. **Release sign‑off** manual step: generate .dmg, verify Gatekeeper, notarize (if keys present), upload to GitHub Releases.

---

## 6. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| YouTube API changes break InnerTube cipher/PoToken | Medium | High | Maintain fallback client versions; runtime cipher registry; unit tests with recorded JSON fixtures. |
| libmpv linking issues on macOS (new OS version) | Medium | High | Document `dylibbundler` process; CI builds on latest macOS; keep `libmpv` version pinned. |
| GPL‑3.0 compliance overhead | Low | High | Clear source‑distribution docs; license headers automated in CI. |
| Large library (≥ 100 k tracks) UI freeze | Medium | Medium | Virtualized lists in Svelte; pagination; indexed DB for stats. |
| macOS Gatekeeper/quarantine on first launch | Certain | Medium | Provide `xattr` instruction in release notes; optional codesign for early adopters. |
| Cipher/PoToken maintenance burden | Ongoing | Medium | Bake registry; accept upstream PRs from LiMusic; keep a private fork with patches. |

---

## 7. Checklist for “Feature Complete”

- [ ] All unit tests pass (>80 % coverage).
- [ ] All integration Tauri commands return expected data.
- [ ] macOS debug build runs without linker errors.
- [ ] Release .dmg generated and app launches.
- [ ] Core screens (Home, Search, Library, Now Playing, Mini‑player) render with real data.
- [ ] Playback (play/pause/next/prev/seek) works end‑to‑end.
- [ ] Queue operations (add, reorder, persist) functional.
- [ ] Lyrics loading and sync operational.
- [ ] Spotify import wizard functional (match confidence, review, create).
- [ ] Last.fm/ListenBrainz scrobbling works (test mode).
- [ ] Discord Rich Presence toggles correctly.
- [ ] Media key handling functional.
- [ ] Settings persist across restarts.
- [ ] Authentication stored in Keychain, not in logs.
- [ ] Release notes include Gatekeeper/quarantine guidance.

*When all items are checked, the project may be considered ready for public beta.*

---
*Maintain this document alongside `docs/ARCHITECTURE.md`, `docs/FEATURE-INVENTORY.md`, `docs/DESIGN-SYSTEM.md`, and `docs/BUILD-AND-RELEASE.md`. Update whenever new commands, tables, or UI flows are added.*