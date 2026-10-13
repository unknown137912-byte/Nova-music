use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<crate::db::Db>,
    pub app_data_dir: std::path::PathBuf,
    pub audio_cache_dir: std::path::PathBuf,
    pub innertube: Arc<innertube::InnerTube>,
    pub player: Arc<player::Player>,
    pub orchestrator: Arc<crate::orchestrator::Orchestrator>,
    pub current_track: Arc<Mutex<Option<serde_json::Value>>>,
    pub queue: Arc<Mutex<Vec<serde_json::Value>>>,
    pub download_cancellations:
        Arc<Mutex<std::collections::HashMap<String, tokio::sync::watch::Sender<bool>>>>,
    playback_events: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<player::PlayerEvent>>>>,
}

impl AppState {
    pub async fn new(
        data_dir: &std::path::Path,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let db_path = data_dir.join("novamusic.sqlite");
        let (db, quarantined_database) = crate::db::Db::open_or_quarantine(&db_path)?;
        if let Some(path) = quarantined_database {
            tracing::error!(
                backup = %path.display(),
                "The local database was damaged and has been preserved for recovery"
            );
        }
        let db = Arc::new(db);

        let saved_cookie = db.get_setting("session_cookie").await.ok().flatten();
        let saved_oauth_token = db
            .get_setting("oauth_access_token")
            .await
            .ok()
            .flatten()
            .filter(|v| !v.is_empty());
        let visitor_data = db.get_setting("visitor_data").await.ok().flatten();
        let data_sync_id = db.get_setting("data_sync_id").await.ok().flatten();

        let session = innertube::Session {
            locale: innertube::Locale::default(),
            visitor_data: visitor_data.clone(),
            data_sync_id,
            cookie: saved_cookie,
            oauth_token: saved_oauth_token,
        };
        let innertube = Arc::new(innertube::InnerTube::new(session, None)?);
        if visitor_data.is_none() {
            let it_clone = innertube.clone();
            let db_clone = db.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(vd) = it_clone.fetch_visitor_data().await {
                    it_clone.set_visitor_data(Some(vd.clone()));
                    let _ = db_clone.set_setting("visitor_data", &vd).await;
                    tracing::info!("Warmed up YouTube visitor data successfully");
                }
            });
        }

        let cache_dir = data_dir.join("audio-cache");
        std::fs::create_dir_all(&cache_dir).ok();
        let mut player = player::Player::new(cache_dir.to_str().unwrap())?;
        if let Some(volume) = db
            .get_setting("volume")
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse::<i64>().ok())
        {
            player.set_volume(volume.clamp(0, 100))?;
        } else {
            player.set_volume(85)?;
        }
        if db.get_setting("gapless").await.ok().flatten().as_deref() == Some("false") {
            player.set_gapless(false)?;
        }
        if let Some(speed) = db
            .get_setting("playback_speed")
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse::<f64>().ok())
        {
            player.set_speed(speed.clamp(0.25, 2.0))?;
        }
        let playback_events = Arc::new(Mutex::new(player.take_events()));
        let player = Arc::new(player);
        let orchestrator = Arc::new(crate::orchestrator::Orchestrator::new(
            db.clone(),
            innertube.clone(),
            player.clone(),
        )?);
        let current_track = Arc::new(Mutex::new(None));
        let queue = Arc::new(Mutex::new(Vec::new()));
        let download_cancellations = Arc::new(Mutex::new(std::collections::HashMap::new()));

        Ok(Self {
            db,
            app_data_dir: data_dir.to_path_buf(),
            audio_cache_dir: cache_dir,
            innertube,
            player,
            orchestrator,
            current_track,
            queue,
            download_cancellations,
            playback_events,
        })
    }

    /// Forward asynchronous mpv failures to the UI. Stream opening happens after `play_track`
    /// returns, so without this event path a refused stream looks like a successful click with a
    /// permanently stationary progress bar.
    pub fn start_playback_events(&self, app: tauri::AppHandle) {
        let Some(mut events) = self.playback_events.lock().unwrap().take() else {
            return;
        };
        tauri::async_runtime::spawn(async move {
            use tauri::Emitter;
            while let Some(event) = events.recv().await {
                match event {
                    player::PlayerEvent::Playing(true) => {
                        let _ = app.emit("playback-started", ());
                    }
                    player::PlayerEvent::TrackEnded => {
                        let _ = app.emit("playback-ended", ());
                    }
                    player::PlayerEvent::TrackFailed(message)
                    | player::PlayerEvent::Error(message) => {
                        let _ = app.emit("playback-error", message);
                    }
                    player::PlayerEvent::AudioDeviceLost => {
                        let _ = app.emit(
                            "playback-error",
                            "Audio output is unavailable. Check your sound device and try again.",
                        );
                    }
                    _ => {}
                }
            }
        });
    }
}

pub async fn persisted_data_sync_id(db: &crate::db::Db) -> Option<String> {
    db.get_setting("data_sync_id").await.ok().flatten()
}
