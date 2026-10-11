# NOVA Music - Data Model (SQLite Schema)

This document describes the SQLite database used by NOVA Music (`src-tauri/src/db.rs`) and extensions needed for NOVA‑specific features.

## Table List

| # | Table | Purpose | Key Columns | Notes |
|---|-------|---------|-------------|-------|
| 1 | `accounts` | One saved Google account + per‑account metadata | `id TEXT PRIMARY KEY`, `session_cookie TEXT NOT NULL`, `data_sync_id TEXT`, `selected_identity_json TEXT`, `account_json TEXT`, `visitor_data TEXT`, `added_at INTEGER NOT NULL` | `id` is an MD5‑derived stable identifier (`ga‑…`). Sensitive cookies are **never** stored in plain text in logs; they are encrypted by the macOS Keychain wrapper before insertion. |
| 2 | `settings` | Application‑wide key/value preferences | `key TEXT PRIMARY KEY`, `value TEXT NOT NULL` | Values are JSON‑encoded where needed (`bool`, `int`, `string`). Theme, playback, shortcuts, integrations. |
| 3 | `stream_url_cache` | Latency cache for authenticated stream URLs (never source‑of‑truth) | `url TEXT PRIMARY KEY`, `itag INTEGER NOT NULL`, `expires_at INTEGER NOT NULL`, `loudness_db REAL`, `is_video BOOLEAN`, `ping_url TEXT`, `ping_client TEXT`, `client TEXT` | TTL‑based eviction. `loudness_db` and `is_video` cached for gapless replay. |
| 4 | `lyrics_cache` | Synced lyrics with source priority & timestamps | `track_id TEXT PRIMARY KEY`, `lyrics TEXT NOT NULL`, `source TEXT NOT NULL`, `last_fetched INTEGER NOT NULL`, `ttl INTEGER NOT NULL` | `source` enumerated: `boidu`, `lrclib`, `ytm`, `netease`, `qq`, `kugou`. Auto‑expire after TTL (default 7 days). |
| 5 | `plays` | Locally cached listening history (day‑bucketed) | `track_id TEXT NOT NULL`, `played_at INTEGER NOT NULL`, `source TEXT NOT NULL` | `source` = `ytm` or `local`. No personal identifiers beyond track ID; respects privacy. |
| 6 | `local_tracks` | Metadata for user‑owned audio files imported into NOVA | `file_path TEXT PRIMARY KEY`, `title TEXT`, `artist TEXT`, `album TEXT`, `duration REAL`, `last_modified INTEGER` | Uses `lofty` crate on init; rows are immutable after insertion. |
| 7 | `playlists` | NOVA‑created playlists (YTM‑synced or local) | `id TEXT PRIMARY KEY`, `name TEXT NOT NULL`, `owner_id TEXT` (YTM account id), `is_local INTEGER NOT NULL DEFAULT 0`, `created_at INTEGER NOT NULL` | `is_local` distinguishes YTM‑owned vs user‑created. |
| 8 | `playlist_tracks` | Tracks belonging to a playlist (order is implicit by row‑insertion order; explicit `order_index` can be added later) | `playlist_id TEXT NOT NULL`, `track_id TEXT NOT NULL`, `added_at INTEGER NOT NULL` | Foreign key to `playlists.id` and optionally to `local_tracks.file_path` or YTM video IDs. |
| 9 | `import_matches` | Spotify‑import session data (match confidence, user corrections) | `id INTEGER PRIMARY KEY AUTOINCREMENT`, `session_token TEXT NOT NULL`, `spotify_track_json TEXT NOT NULL`, `youtube_candidate_json TEXT`, `tier TEXT NOT NULL`, `manual INTEGER NOT NULL DEFAULT 0`, `updated_at INTEGER NOT NULL` | `tier` = `exact`, `probable`, `unmatched`. Allows user to review/fix before creation. |
| 10 | `local_playlists` | User‑created local playlists (filesystem‑backed) | `id TEXT PRIMARY KEY`, `name TEXT NOT NULL`, `description TEXT`, `created_at INTEGER NOT NULL` | Mirrors YTM playlist structure but stored locally. |
| 11 | `local_playlist_tracks` | Tracks in a local playlist | `playlist_id TEXT NOT NULL`, `track_id TEXT NOT NULL`, `order_index INTEGER NOT NULL` | Supports drag‑drop reorder; `order_index` is the position. |
| 12 | `listening_stats` | Aggregated statistics (play counts, skip rates, session duration) – **NOVA‑only extension** | `track_id TEXT NOT NULL`, `play_count INTEGER NOT NULL DEFAULT 0`, `skip_count INTEGER NOT NULL DEFAULT 0`, `total_duration REAL NOT NULL DEFAULT 0`, `last_played_at INTEGER` | Enables “Recently Added / Most Played” discovery panels. |
| 13 | `smart_playlists` | Criteria‑based saved queries for dynamic playlists – **NOVA‑only** | `id TEXT PRIMARY KEY`, `name TEXT NOT NULL`, `criteria_json TEXT NOT NULL` (SQL‑like WHERE clause), `last_generated_at INTEGER` | Users can define “Playlists > Last 30 days > Skipped > > 3 mins”. |
| 14 | `migration_versions` | Tracks SQLite schema migration version for safe upgrades | `version INTEGER PRIMARY KEY`, `applied_at INTEGER NOT NULL` | One row per upgrade; migration scripts in `src-tauri/scripts/migrations/`. |

## Entity‑Relationship Summary

- **`accounts`** ‑ 1‑N **`settings`** (per‑account overrides stored as `key` prefixed with `account_<id>:`).
- **`accounts`** ‑ 1‑N **`stream_url_cache`** (each account may have its own cached streams; shared globally but logically partitioned).
- **`plays`** ‑ N‑1 **`local_tracks`** (optional – a play may reference a local file).
- **`playlists`** ‑ N‑M **`playlist_tracks`** (many‑to‑many between playlists and tracks; tracks can be local or YTM‑owned).
- **`import_matches`** ‑ 1‑N **`smart_playlists`** (a smart‑playlist may be built from resolved import matches).
- **`listening_stats`** ‑ N‑1 **`local_tracks`** (statistics per tracked track).
- **`migration_versions`** is consulted on app start; if the on‑disk version < current, the migration runner (`migrate.rs`) runs each up‑script in order.

## Migration Strategy

1. **Version 1** – Initial schema (tables 1‑9) shipped with the first NOVA release (forked from LiMusic `db.rs` version 1).
2. **Version 2** – Add `listening_stats` (table 12) and `smart_playlists` (table 13). Migration script inserts a default row with zeroed stats.
3. **Version 3** – Add `local_playlists` / `local_playlist_tracks` (tables 10‑11) to support fully offline playlists.
4. **Version 4** – Add `migration_versions` row and ensure all existing databases acquire the row.

Each migration script is a simple `ALTER TABLE … ADD COLUMN …` / `CREATE TABLE IF NOT EXISTS …` statement run inside a `rusqlite` transaction. Down‑level upgrades are **not** supported; users must re‑install the new version.

## Indexes (for performance on large libraries)

| Table | Index | Reason |
|---|---|---|
| `plays` | `idx_plays_track` (`track_id`) | Fast lookup of recent plays per track. |
| `plays` | `idx_plays_at` (`played_at DESC`) | Sorting by recency for “Recently Played”. |
| `lyrics_cache` | `idx_lyrics_source` (`source`, `last_fetched`) | Quick refresh of expired sources. |
| `import_matches` | `idx_import_session` (`session_token`) | One‑off cleanup of an import session. |
| `listening_stats` | `idx_stats_track` (`track_id`) | Aggregation queries. |
| `smart_playlists` | `idx_smart_criteria` (`criteria_json`) | GIN‑style partial scan (Postgres‑style, SQLite uses `LIKE` with generated column). |

## Caching Policy

- **Artwork** – Not stored in SQLite; cached on disk under `~/Library/Caches/NOVA-Music/Artwork/` with a 7‑day TTL and a max‑size of 200 MiB (LRU eviction).  
- **Metadata** – Search results, album/artist info stored in `stream_url_cache` with TTL; refreshed lazily when missing.  
- **Lyrics** – `lyrics_cache` TTL 7 days; background worker refreshes expired entries (once per track play).  

## Security & Privacy

- **Never store passwords** in any table.  
- **Authentication cookies** are encrypted via the macOS Keychain wrapper (`keyring` crate) before insertion into `accounts.session_cookie`. The plaintext never appears in logs.  
- **Listening stats** contain only track IDs and counts – no user‑identifiable data.  
- **SQLite file** is kept at the standard macOS application support location (`~/Library/Application Support/NOVA-Music/database.sqlite`). File‑system permissions restrict access to the app user only.  

## Extending the Model

When adding a new feature:

1. Decide if data belongs in an existing table or a new one.  
2. If new, add the table definition above and a migration entry in `migration_versions`.  
3. Add Rust struct methods in `db.rs` (or a new `db_ext.rs`) for CRUD operations, guarded by the `Db` mutex.  
4. Expose a Tauri command (`#[tauri::command]`) if the UI needs direct access, otherwise keep it internal.  
5. Update the test plan (see `TEST-PLAN.md`) with schema‑integrity checks.

---
*This model is intentionally a superset of LiMusic’s `db.rs`; NOVA will only use the columns it needs, dropping any LiMusic‑only columns that are not required.*