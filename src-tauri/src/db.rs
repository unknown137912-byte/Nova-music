use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &std::path::Path) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(path)?;
        let db = Db {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.initialize_tables()?;
        Ok(db)
    }

    pub fn open_or_quarantine(
        path: &std::path::Path,
    ) -> std::result::Result<(Self, Option<std::path::PathBuf>), Box<dyn std::error::Error>> {
        // For now, just call open and return None for quarantined
        let db = Self::open(path)?;
        Ok((db, None))
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
        let value_str = std::str::from_utf8(value).unwrap_or("");
        conn.execute(
            "INSERT OR REPLACE INTO cache (key, value, content_type, expires_at) VALUES (?, ?, ?, ?)",
            [&key, value_str, content_type, &expires_at.unwrap_or(0).to_string()],
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
