use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &std::path::Path) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Db {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.initialize_tables()?;
        Ok(db)
    }

    pub fn open_or_quarantine(
        path: &std::path::Path,
    ) -> std::result::Result<(Self, Option<std::path::PathBuf>), Box<dyn std::error::Error>> {
        match Self::open(path) {
            Ok(db) => Ok((db, None)),
            Err(error) if is_database_corruption(error.as_ref()) => {
                let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
                let mut backup = path.as_os_str().to_os_string();
                backup.push(format!(".corrupt-{stamp}"));
                let backup = std::path::PathBuf::from(backup);
                std::fs::rename(path, &backup)?;
                for suffix in ["-wal", "-shm"] {
                    let mut sidecar = path.as_os_str().to_os_string();
                    sidecar.push(suffix);
                    let sidecar = std::path::PathBuf::from(sidecar);
                    if sidecar.exists() {
                        let mut sidecar_backup = backup.as_os_str().to_os_string();
                        sidecar_backup.push(suffix);
                        std::fs::rename(sidecar, sidecar_backup)?;
                    }
                }
                Ok((Self::open(path)?, Some(backup)))
            }
            Err(error) => Err(error),
        }
    }

    fn initialize_tables(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();

        // Create accounts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE,
                cookie TEXT,
                visitor_data TEXT,
                data_sync_id TEXT,
                refresh_token TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_used TIMESTAMP,
                is_default BOOLEAN DEFAULT FALSE
            )",
            [],
        )?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT,
                account_id INTEGER,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            )",
            [],
        )?;

        // Create playlists table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                account_id INTEGER,
                title TEXT NOT NULL,
                description TEXT,
                thumbnail_url TEXT,
                track_count INTEGER,
                is_local BOOLEAN DEFAULT FALSE,
                is_smart BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP,
                updated_at TIMESTAMP,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            )",
            [],
        )?;

        // Create playlist_tracks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT,
                track_id TEXT,
                position INTEGER NOT NULL,
                added_by INTEGER,
                added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_modified TIMESTAMP,
                FOREIGN KEY (playlist_id) REFERENCES playlists(id),
                FOREIGN KEY (added_by) REFERENCES accounts(id)
            )",
            [],
        )?;

        // Create history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER,
                video_id TEXT,
                title TEXT,
                artist TEXT,
                album TEXT,
                thumbnail_url TEXT,
                duration_seconds REAL,
                played_at TIMESTAMP,
                play_seconds REAL,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            )",
            [],
        )?;

        // Create cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                content_type TEXT,
                expires_at TIMESTAMP,
                accessed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create lyrics table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS lyrics (
                track_id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                content TEXT NOT NULL,
                synced BOOLEAN DEFAULT FALSE,
                word_timings TEXT,
                translations TEXT,
                fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS local_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                thumbnail_url TEXT NOT NULL DEFAULT '',
                duration_seconds REAL NOT NULL DEFAULT 0,
                play_count INTEGER NOT NULL DEFAULT 0,
                last_played TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_local_history_recent ON local_history(last_played DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_local_history_taste ON local_history(play_count DESC, last_played DESC)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS local_playlists (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS local_playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL DEFAULT '',
                thumbnail_url TEXT NOT NULL DEFAULT '',
                duration_seconds REAL NOT NULL DEFAULT 0,
                position INTEGER NOT NULL,
                added_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES local_playlists(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_local_playlist_tracks_order ON local_playlist_tracks(playlist_id, position)",
            [],
        )?;

        Ok(())
    }

    pub async fn create_local_playlist(
        &self,
        title: &str,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let id = format!(
            "NOVA_LOCAL_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        conn.execute(
            "INSERT INTO local_playlists (id, title) VALUES (?1, ?2)",
            rusqlite::params![id, title],
        )?;
        Ok(id)
    }

    pub async fn get_local_playlists(
        &self,
    ) -> std::result::Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let mut statement = conn.prepare(
            "SELECT p.id, p.title,
                (SELECT thumbnail_url FROM local_playlist_tracks t WHERE t.playlist_id = p.id ORDER BY t.position LIMIT 1),
                (SELECT COUNT(*) FROM local_playlist_tracks t WHERE t.playlist_id = p.id),
                p.created_at, p.updated_at
             FROM local_playlists p ORDER BY p.updated_at DESC, p.title COLLATE NOCASE",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "playlistId": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "thumbnail": row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                "trackCount": row.get::<_, i64>(3)?,
                "isLocal": true,
                "createdAt": row.get::<_, String>(4)?,
                "updatedAt": row.get::<_, String>(5)?,
            }))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub async fn get_local_playlist(
        &self,
        playlist_id: &str,
    ) -> std::result::Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let mut playlist = conn
            .query_row(
                "SELECT id, title, created_at, updated_at FROM local_playlists WHERE id = ?1",
                [playlist_id],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>(0)?,
                        "title": row.get::<_, String>(1)?,
                        "createdAt": row.get::<_, String>(2)?,
                        "updatedAt": row.get::<_, String>(3)?,
                    }))
                },
            )
            .optional()?;
        let Some(ref mut playlist_value) = playlist else {
            return Ok(None);
        };
        let mut statement = conn.prepare(
            "SELECT track_id, title, artist, thumbnail_url, duration_seconds, position, added_at
             FROM local_playlist_tracks WHERE playlist_id = ?1 ORDER BY position, added_at",
        )?;
        let rows = statement.query_map([playlist_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "videoId": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "artist": row.get::<_, String>(2)?,
                "thumbnail": row.get::<_, String>(3)?,
                "duration": row.get::<_, f64>(4)?,
                "position": row.get::<_, i64>(5)?,
                "addedAt": row.get::<_, String>(6)?,
            }))
        })?;
        let tracks = rows.collect::<Result<Vec<_>, _>>()?;
        playlist_value["thumbnail"] = tracks
            .first()
            .and_then(|track| track.get("thumbnail"))
            .cloned()
            .unwrap_or(serde_json::Value::String(String::new()));
        playlist_value["trackCount"] = serde_json::json!(tracks.len());
        playlist_value["items"] = serde_json::json!(tracks);
        Ok(playlist)
    }

    pub async fn add_track_to_local_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
        title: &str,
        artist: &str,
        thumbnail_url: &str,
        duration_seconds: f64,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM local_playlists WHERE id = ?1)",
            [playlist_id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Local playlist was not found",
            )
            .into());
        }
        let position: i64 = conn.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM local_playlist_tracks WHERE playlist_id = ?1",
            [playlist_id],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO local_playlist_tracks (playlist_id, track_id, title, artist, thumbnail_url, duration_seconds, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(playlist_id, track_id) DO UPDATE SET
                title = excluded.title,
                artist = excluded.artist,
                thumbnail_url = excluded.thumbnail_url,
                duration_seconds = excluded.duration_seconds",
            rusqlite::params![playlist_id, track_id, title, artist, thumbnail_url, duration_seconds, position],
        )?;
        conn.execute(
            "UPDATE local_playlists SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            [playlist_id],
        )?;
        Ok(())
    }

    pub async fn delete_local_playlist(
        &self,
        playlist_id: &str,
    ) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        Ok(conn.execute("DELETE FROM local_playlists WHERE id = ?1", [playlist_id])? > 0)
    }

    pub async fn record_local_play(
        &self,
        track_id: &str,
        title: &str,
        artist: &str,
        thumbnail_url: &str,
        duration_seconds: f64,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        conn.execute(
            "INSERT INTO local_history (track_id, title, artist, thumbnail_url, duration_seconds, play_count, last_played)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(track_id) DO UPDATE SET
                title = excluded.title,
                artist = excluded.artist,
                thumbnail_url = excluded.thumbnail_url,
                duration_seconds = excluded.duration_seconds,
                play_count = local_history.play_count + 1,
                last_played = CURRENT_TIMESTAMP",
            rusqlite::params![track_id, title, artist, thumbnail_url, duration_seconds],
        )?;
        conn.execute(
            "DELETE FROM local_history WHERE id NOT IN (SELECT id FROM local_history ORDER BY last_played DESC LIMIT 500)",
            [],
        )?;
        Ok(())
    }

    pub async fn get_local_history(
        &self,
        limit: usize,
    ) -> std::result::Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let mut statement = conn.prepare(
            "SELECT track_id, title, artist, thumbnail_url, duration_seconds, play_count, last_played
             FROM local_history ORDER BY last_played DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([limit.min(500) as i64], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "videoId": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "artist": row.get::<_, String>(2)?,
                "thumbnail": row.get::<_, String>(3)?,
                "duration": row.get::<_, f64>(4)?,
                "playCount": row.get::<_, i64>(5)?,
                "lastPlayed": row.get::<_, String>(6)?,
            }))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub async fn get_top_listened_track(
        &self,
    ) -> std::result::Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let mut statement = conn.prepare(
            "SELECT track_id, title, artist, thumbnail_url, duration_seconds, play_count
             FROM local_history ORDER BY play_count DESC, last_played DESC LIMIT 1",
        )?;
        let mut rows = statement.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "videoId": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "artist": row.get::<_, String>(2)?,
                "thumbnail": row.get::<_, String>(3)?,
                "duration": row.get::<_, f64>(4)?,
                "playCount": row.get::<_, i64>(5)?,
            })))
        } else {
            Ok(None)
        }
    }

    pub async fn optimize(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        conn.execute(
            "DELETE FROM cache WHERE CAST(expires_at AS INTEGER) > 0 AND CAST(expires_at AS INTEGER) < CAST(strftime('%s','now') AS INTEGER)",
            [],
        )?;
        conn.execute_batch("PRAGMA optimize; VACUUM;")?;
        Ok(())
    }

    // Account methods
    pub async fn create_account(
        &self,
        email: &str,
        cookie: Option<&str>,
    ) -> std::result::Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO accounts (email, cookie) VALUES (?, ?)",
            [&email, cookie.unwrap_or("")],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub async fn get_account(
        &self,
        email: &str,
    ) -> std::result::Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, cookie, visitor_data, data_sync_id, created_at, last_used, is_default 
             FROM accounts WHERE email = ?"
        )?;
        let mut rows = stmt.query(&[email])?;
        if let Some(row) = rows.next()? {
            let account = serde_json::json!({
                "id": row.get::<_, i64>("id")?,
                "email": row.get::<_, String>("email")?,
                "cookie": row.get::<_, String>("cookie")?,
                "visitor_data": row.get::<_, String>("visitor_data")?,
                "data_sync_id": row.get::<_, String>("data_sync_id")?,
                "created_at": row.get::<_, String>("created_at")?,
                "last_used": row.get::<_, String>("last_used")?,
                "is_default": row.get::<_, i64>("is_default")?
            });
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    // Settings methods
    pub async fn get_setting(
        &self,
        key: &str,
    ) -> std::result::Result<Option<String>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
        let mut rows = stmt.query(&[key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get("value")?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_setting(
        &self,
        key: &str,
        value: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            [&key, value],
        )?;
        Ok(())
    }

    pub async fn get_all_settings(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
        let mut settings = serde_json::Map::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get("key")?;
            let value: String = row.get("value")?;
            settings.insert(key, serde_json::Value::String(value));
        }
        Ok(serde_json::Value::Object(settings))
    }

    // Queue methods
    pub async fn get_queue(
        &self,
    ) -> std::result::Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT track_id, title, artist, album, thumbnail_url, duration_seconds 
             FROM playlist_tracks 
             WHERE playlist_id = (SELECT id FROM playlists WHERE is_local = 1) 
             ORDER BY position",
        )?;
        let mut queue = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let track = serde_json::json!({
                "track_id": row.get::<_, String>("track_id")?,
                "title": row.get::<_, String>("title")?,
                "artist": row.get::<_, String>("artist")?,
                "album": row.get::<_, String>("album")?,
                "thumbnail_url": row.get::<_, String>("thumbnail_url")?,
                "duration_seconds": row.get::<_, f64>("duration_seconds")?
            });
            queue.push(track);
        }
        Ok(queue)
    }

    // Search methods
    pub async fn search(
        &self,
        query: &str,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        // For now, return empty results - will be implemented with innertube integration
        Ok(serde_json::json!({"results": [], "query": query}))
    }

    // Library methods
    pub async fn get_library(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        // For now, return empty library - will be implemented with innertube integration
        Ok(serde_json::json!({
            "likedSongs": [],
            "playlists": [],
            "albums": [],
            "artists": [],
            "history": []
        }))
    }

    // Playlist methods
    pub async fn get_playlist(
        &self,
        playlist_id: &str,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let _stmt = conn.prepare(
            "SELECT p.id, p.title, p.description, p.thumbnail_url, p.track_count,
                    pt.track_id, pt.position, pt.added_by, pt.added_at
             FROM playlists p
             LEFT JOIN playlist_tracks pt ON p.id = pt.playlist_id
             WHERE p.id = ?",
        )?;
        let playlist = serde_json::json!({
            "id": playlist_id,
            "title": "",
            "description": "",
            "thumbnail_url": "",
            "track_count": 0,
            "tracks": []
        });

        // TODO: Implement proper playlist retrieval

        Ok(playlist)
    }

    // Album and artist methods
    pub async fn get_album(
        &self,
        album_id: &str,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        // For now, return empty - will be implemented with innertube integration
        Ok(serde_json::json!({
            "id": album_id,
            "title": "",
            "artist": "",
            "thumbnail_url": "",
            "tracks": [],
            "release_date": "",
            "duration_seconds": 0
        }))
    }

    pub async fn get_artist(
        &self,
        artist_id: &str,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        // For now, return empty - will be implemented with innertube integration
        Ok(serde_json::json!({
            "id": artist_id,
            "name": "",
            "thumbnail_url": "",
            "top_tracks": [],
            "albums": []
        }))
    }

    // Session management methods
    pub async fn get_visitor_data(
        &self,
    ) -> std::result::Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(self.get_setting("visitor_data").await?.map(|v| v))
    }

    pub async fn set_visitor_data(
        &self,
        data: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.set_setting("visitor_data", data).await
    }

    pub async fn get_data_sync_id(
        &self,
    ) -> std::result::Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(self.get_setting("data_sync_id").await?.map(|v| v))
    }

    pub async fn set_data_sync_id(
        &self,
        id: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.set_setting("data_sync_id", id).await
    }

    pub async fn get_session_cookie(
        &self,
    ) -> std::result::Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(self.get_setting("session_cookie").await?.map(|v| v))
    }

    pub async fn set_session_cookie(
        &self,
        cookie: &str,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.set_setting("session_cookie", cookie).await
    }

    // Cache methods
    pub async fn cache_get(
        &self,
        key: &str,
    ) -> std::result::Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM cache WHERE key = ?")?;
        let mut rows = stmt.query(&[key])?;
        if let Some(row) = rows.next()? {
            let value: Vec<u8> = row.get("value")?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub async fn cache_set(
        &self,
        key: &str,
        value: &[u8],
        content_type: &str,
        expires_at: Option<i64>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO cache (key, value, content_type, expires_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![key, value, content_type, expires_at.unwrap_or(0)],
        )?;
        Ok(())
    }

    // Helper method to execute raw SQL (for debugging)
    pub async fn execute(&self, sql: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(sql, [])?;
        Ok(())
    }

    // Query helper
    pub async fn query(
        &self,
        sql: &str,
    ) -> std::result::Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let mut rows = stmt.query([])?;
        let mut results = Vec::new();

        while let Some(row) = rows.next()? {
            let mut result = serde_json::Map::new();
            for column in &column_names {
                let value: serde_json::Value = match row.get::<_, Option<String>>(column.as_str()) {
                    Ok(Some(v)) => serde_json::Value::String(v),
                    Ok(None) => serde_json::Value::Null,
                    Err(_) => serde_json::Value::Null,
                };
                result.insert(column.clone(), value);
            }
            results.push(serde_json::Value::Object(result));
        }

        Ok(results)
    }
}

fn is_database_corruption(error: &(dyn std::error::Error + 'static)) -> bool {
    let Some(rusqlite::Error::SqliteFailure(failure, _)) = error.downcast_ref::<rusqlite::Error>()
    else {
        return false;
    };
    matches!(
        failure.code,
        rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
    )
}

#[cfg(test)]
mod tests {
    use super::Db;

    fn temporary_db_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "nova-music-{label}-{}-{}.sqlite",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ))
    }

    #[tokio::test]
    async fn local_history_updates_play_count_and_keeps_latest_track_metadata() {
        let path = temporary_db_path("history");
        let db = Db::open(&path).expect("open test database");
        db.record_local_play("track-1", "First title", "Artist", "cover", 120.0)
            .await
            .expect("record first play");
        db.record_local_play("track-1", "Updated title", "Artist", "cover-2", 135.0)
            .await
            .expect("record repeat play");

        let history = db.get_local_history(10).await.expect("read history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0]["playCount"], 2);
        assert_eq!(history[0]["title"], "Updated title");
        assert_eq!(history[0]["duration"], 135.0);

        drop(db);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }

    #[tokio::test]
    async fn cache_round_trips_arbitrary_binary_data() {
        let path = temporary_db_path("cache");
        let db = Db::open(&path).expect("open test database");
        let bytes = [0, 255, 17, 128, 1];
        db.cache_set("binary", &bytes, "application/octet-stream", Some(123))
            .await
            .expect("store bytes");
        assert_eq!(
            db.cache_get("binary").await.expect("read bytes"),
            Some(bytes.to_vec())
        );

        drop(db);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }

    #[tokio::test]
    async fn local_playlists_create_add_list_and_delete_tracks() {
        let path = temporary_db_path("playlists");
        let db = Db::open(&path).expect("open test database");
        let playlist_id = db
            .create_local_playlist("Road trip")
            .await
            .expect("create playlist");
        db.add_track_to_local_playlist(&playlist_id, "track-1", "First", "Artist", "cover", 95.0)
            .await
            .expect("add track");
        db.add_track_to_local_playlist(
            &playlist_id,
            "track-1",
            "Updated",
            "Artist",
            "cover-2",
            100.0,
        )
        .await
        .expect("update duplicate track");

        let playlists = db.get_local_playlists().await.expect("list playlists");
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0]["id"], playlist_id);
        assert_eq!(playlists[0]["trackCount"], 1);
        assert_eq!(playlists[0]["thumbnail"], "cover-2");

        let playlist = db
            .get_local_playlist(&playlist_id)
            .await
            .expect("load playlist")
            .expect("playlist exists");
        assert_eq!(playlist["items"].as_array().expect("track list").len(), 1);
        assert_eq!(playlist["items"][0]["title"], "Updated");
        assert!(db
            .delete_local_playlist(&playlist_id)
            .await
            .expect("delete playlist"));
        assert!(db
            .get_local_playlist(&playlist_id)
            .await
            .expect("load deleted playlist")
            .is_none());

        drop(db);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }
}
