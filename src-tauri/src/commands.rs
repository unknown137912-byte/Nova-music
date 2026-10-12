use crate::state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::AsyncWriteExt;

fn metadata_client() -> Result<innertube::clients::YouTubeClient, String> {
    let clients = innertube::clients::Clients::bundled();
    clients
        .get(innertube::clients::METADATA_CLIENT)
        .cloned()
        .ok_or_else(|| "METADATA_CLIENT not found".to_string())
}

#[tauri::command]
pub async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Welcome to NOVA Music, {}!", name))
}

#[tauri::command]
pub async fn get_home(
    params: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    match state.innertube.home(&client, params.as_deref()).await {
        Ok(home_page) => serde_json::to_value(&home_page).map_err(|e| e.to_string()),
        Err(e) => {
            tracing::error!("get_home error: {e}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_home_continuation(
    continuation: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    let page = state
        .innertube
        .home_continuation(&client, &continuation)
        .await
        .map_err(|error| format!("Could not load more recommendations: {error}"))?;
    serde_json::to_value(page).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    if playlist_id.starts_with("NOVA_LOCAL_") {
        return state
            .db
            .get_local_playlist(&playlist_id)
            .await
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "This local playlist no longer exists.".to_string());
    }
    let access_token = crate::auth::ensure_access_token(&state)
        .await
        .ok()
        .flatten();
    let client = metadata_client()?;
    match state.innertube.playlist(&client, &playlist_id, None).await {
        Ok(page) => serde_json::to_value(&page).map_err(|e| e.to_string()),
        Err(e) => {
            tracing::error!("get_playlist error: {e}");
            if let Some(token) = access_token {
                crate::auth::youtube_data_playlist(&token, &playlist_id).await
            } else {
                Err(e.to_string())
            }
        }
    }
}

#[tauri::command]
pub async fn get_album(
    album_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    match state.innertube.album(&client, &album_id).await {
        Ok(album) => serde_json::to_value(&album).map_err(|e| e.to_string()),
        Err(e) => {
            tracing::error!("get_album error: {e}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_artist(
    artist_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    match state.innertube.artist(&client, &artist_id).await {
        Ok(artist) => serde_json::to_value(&artist).map_err(|e| e.to_string()),
        Err(e) => {
            tracing::error!("get_artist error: {e}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_library(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let access_token = crate::auth::ensure_access_token(&state)
        .await
        .ok()
        .flatten();
    let local_playlists = state
        .db
        .get_local_playlists()
        .await
        .map_err(|error| error.to_string())?;
    let liked_songs = state
        .db
        .get_liked_tracks()
        .await
        .map_err(|error| error.to_string())?;
    let client = metadata_client()?;
    if state.innertube.is_logged_in() {
        let (playlists, albums, artists) = tokio::join!(
            state.innertube.library_playlists(&client),
            state.innertube.library_albums(&client),
            state.innertube.library_artists(&client),
        );
        if playlists.is_ok() || albums.is_ok() || artists.is_ok() {
            let mut playlists: Vec<serde_json::Value> = match playlists {
                Ok(items) => items
                    .into_iter()
                    .filter_map(|item| serde_json::to_value(item).ok())
                    .collect(),
                Err(_) => match access_token.as_ref() {
                    Some(token) => crate::auth::youtube_data_playlists(token)
                        .await
                        .unwrap_or_default(),
                    None => Vec::new(),
                },
            };
            playlists.extend(local_playlists);
            return Ok(serde_json::json!({
                "playlists": playlists,
                "albums": albums.unwrap_or_default(),
                "artists": artists.unwrap_or_default(),
                "songs": liked_songs,
            }));
        }
    }
    if let Some(token) = access_token {
        let mut playlists = crate::auth::youtube_data_playlists(&token).await?;
        playlists.extend(local_playlists);
        return Ok(
            serde_json::json!({"playlists": playlists, "songs": liked_songs, "albums": [], "artists": []}),
        );
    }
    match state.db.get_library().await {
        Ok(mut library) => {
            if let Some(playlists) = library
                .get_mut("playlists")
                .and_then(serde_json::Value::as_array_mut)
            {
                playlists.extend(local_playlists);
            }
            library["songs"] = serde_json::Value::Array(liked_songs);
            Ok(library)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn create_local_playlist(
    title: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 80 {
        return Err("Playlist names must contain 1 to 80 characters.".into());
    }
    let id = state
        .db
        .create_local_playlist(title)
        .await
        .map_err(|error| error.to_string())?;
    Ok(
        serde_json::json!({ "id": id, "playlistId": id, "title": title, "trackCount": 0, "isLocal": true }),
    )
}

#[tauri::command]
pub async fn add_track_to_local_playlist(
    playlist_id: String,
    track: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !playlist_id.starts_with("NOVA_LOCAL_") {
        return Err("Choose a local NOVA playlist.".into());
    }
    let track_id = track
        .get("videoId")
        .or_else(|| track.get("video_id"))
        .or_else(|| track.get("trackId"))
        .or_else(|| track.get("track_id"))
        .or_else(|| track.get("id"))
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| "This song is missing its YouTube track ID.".to_string())?;
    let title = track
        .get("title")
        .or_else(|| track.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown track");
    let artist = track
        .get("artist")
        .or_else(|| track.get("artists"))
        .or_else(|| track.get("subtitle"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("YouTube Music");
    let thumbnail = track
        .get("thumbnail")
        .or_else(|| track.get("thumbnail_url"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let duration = track
        .get("duration_seconds")
        .or_else(|| track.get("duration"))
        .and_then(|value| {
            value.as_f64().or_else(|| {
                value.as_str().and_then(|text| {
                    let parts = text
                        .split(':')
                        .map(str::parse::<f64>)
                        .collect::<Result<Vec<_>, _>>()
                        .ok()?;
                    match parts.as_slice() {
                        [minutes, seconds] => Some(minutes * 60.0 + seconds),
                        [hours, minutes, seconds] => {
                            Some(hours * 3600.0 + minutes * 60.0 + seconds)
                        }
                        _ => None,
                    }
                })
            })
        })
        .unwrap_or(0.0);
    state
        .db
        .add_track_to_local_playlist(&playlist_id, track_id, title, artist, thumbnail, duration)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_local_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !playlist_id.starts_with("NOVA_LOCAL_") {
        return Err("Only NOVA-created playlists can be deleted here.".into());
    }
    if state
        .db
        .delete_local_playlist(&playlist_id)
        .await
        .map_err(|error| error.to_string())?
    {
        Ok(())
    } else {
        Err("This local playlist no longer exists.".into())
    }
}

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    if state
        .db
        .get_setting("history_enabled")
        .await
        .ok()
        .flatten()
        .as_deref()
        == Some("false")
    {
        return Ok(serde_json::json!([]));
    }
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    if state.innertube.is_logged_in() {
        match state.innertube.history(&client).await {
            Ok(hist) if !hist.is_empty() => {
                let local = state
                    .db
                    .get_local_history(100)
                    .await
                    .map_err(|error| error.to_string())?;
                if local.is_empty() {
                    return serde_json::to_value(&hist).map_err(|e| e.to_string());
                }
                let mut groups = vec![serde_json::json!({
                    "header": "Recently played in NOVA",
                    "items": local,
                })];
                groups.extend(
                    serde_json::to_value(hist)
                        .map_err(|error| error.to_string())?
                        .as_array()
                        .cloned()
                        .unwrap_or_default(),
                );
                return Ok(serde_json::json!(groups));
            }
            Err(e) => tracing::warn!("history error: {e}"),
            _ => {}
        }
    }
    state
        .db
        .get_local_history(100)
        .await
        .map(|history| serde_json::json!(history))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_liked_tracks(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    state
        .db
        .get_liked_tracks()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_listening_recommendations(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    if state
        .db
        .get_setting("history_enabled")
        .await
        .ok()
        .flatten()
        .as_deref()
        == Some("false")
    {
        return Ok(serde_json::json!([]));
    }
    let seed = state
        .db
        .get_top_listened_track()
        .await
        .map_err(|e| e.to_string())?;
    let Some(seed) = seed else {
        return Ok(serde_json::json!([]));
    };
    let Some(video_id) = seed.get("videoId").and_then(serde_json::Value::as_str) else {
        return Ok(serde_json::json!([]));
    };
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    let page = match state.innertube.next(&client, Some(video_id), None).await {
        Ok(page) => page,
        Err(error) => {
            tracing::warn!("listening recommendations error: {error}");
            return Ok(serde_json::json!({ "seed": seed, "items": [] }));
        }
    };
    let items = if page.items.len() <= 1 {
        if let Some(radio) = page.automix_playlist_id.as_deref() {
            match state
                .innertube
                .next(&client, Some(video_id), Some(radio))
                .await
            {
                Ok(continued) => continued.items,
                Err(_) => page.items,
            }
        } else {
            page.items
        }
    } else {
        page.items
    };
    let tracks = items
        .into_iter()
        .filter(|item| item.video_id != video_id)
        .take(18)
        .map(|item| serde_json::to_value(item).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(serde_json::json!({
        "seed": seed,
        "items": tracks,
    }))
}

#[tauri::command]
pub async fn search(
    query: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let _ = crate::auth::ensure_access_token(&state).await;
    let client = metadata_client()?;
    let results = state
        .innertube
        .search_all(&client, &query, false)
        .await
        .map_err(|e| e.to_string())?;

    let songs: Vec<serde_json::Value> = results
        .songs
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "artist": item.subtitle.unwrap_or_default(),
                "thumbnail": item.thumbnail,
                "duration": item.duration,
                "kind": "song",
            })
        })
        .collect();

    let albums: Vec<serde_json::Value> = results
        .albums
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "artist": item.subtitle.unwrap_or_default(),
                "thumbnail": item.thumbnail,
                "kind": "album",
            })
        })
        .collect();

    let artists: Vec<serde_json::Value> = results
        .artists
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "artist": item.subtitle.unwrap_or_default(),
                "thumbnail": item.thumbnail,
                "kind": "artist",
            })
        })
        .collect();

    let playlists: Vec<serde_json::Value> = results
        .playlists
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "artist": item.subtitle.unwrap_or_default(),
                "thumbnail": item.thumbnail,
                "kind": "playlist",
            })
        })
        .collect();

    Ok(serde_json::json!({
        "items": songs,
        "songs": songs,
        "albums": albums,
        "artists": artists,
        "playlists": playlists,
    }))
}

#[tauri::command]
pub async fn search_suggestions(
    query: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let client = metadata_client()?;
    match state.innertube.search_suggestions(&client, &query).await {
        Ok(sug) => serde_json::to_value(&sug).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

struct ResolvedAudio {
    url: String,
    headers: std::collections::HashMap<String, String>,
    loudness_db: Option<f64>,
    title: Option<String>,
    duration: Option<f64>,
}

fn audio_quality(value: Option<String>) -> innertube::AudioQuality {
    match value.as_deref() {
        Some("low") => innertube::AudioQuality::Low,
        Some("medium") => innertube::AudioQuality::Medium,
        Some("auto") => innertube::AudioQuality::Auto,
        _ => innertube::AudioQuality::High,
    }
}

async fn resolve_audio(
    state: &AppState,
    track_id: &str,
    quality: innertube::AudioQuality,
) -> Result<ResolvedAudio, String> {
    let _ = crate::auth::ensure_access_token(state).await;
    if state.innertube.visitor_data().is_none() {
        if let Ok(vd) = state.innertube.fetch_visitor_data().await {
            state.innertube.set_visitor_data(Some(vd.clone()));
            let _ = state.db.set_setting("visitor_data", &vd).await;
        }
    }

    let clients = innertube::Clients::bundled();
    for client_key in innertube::STREAM_FALLBACK_ORDER {
        let Some(client) = clients.get(client_key) else {
            continue;
        };
        if let Ok(resp) = state
            .innertube
            .player(client, track_id, None, None, None)
            .await
        {
            if !resp.playability_status.is_ok() {
                continue;
            }
            if let Some(sd) = resp.streaming_data {
                if let Some(format) = innertube::find_format(&sd, quality) {
                    if let Some(url) = format.direct_url() {
                        return Ok(ResolvedAudio {
                            url: url.to_owned(),
                            headers: [("User-Agent".into(), client.user_agent.clone())].into(),
                            loudness_db: format.loudness_db.map(f64::from),
                            title: resp.video_details.as_ref().and_then(|v| v.title.clone()),
                            duration: resp
                                .video_details
                                .as_ref()
                                .and_then(|v| v.length_seconds.as_ref())
                                .and_then(|s| s.parse().ok()),
                        });
                    }
                }
            }
        }
    }

    let stream = innertube::rustypipe_fallback::resolve_quality(track_id, quality)
        .await
        .map_err(|e| format!("Could not resolve a playable stream for track {track_id}: {e}"))?;
    let ua = if stream.url.contains("c=IOS") {
        "com.google.ios.youtubemusic/7.01.05 (iPhone16,2; U; CPU iOS 18_2 like Mac OS X;)"
    } else if stream.url.contains("c=ANDROID") {
        "com.google.android.apps.youtube.vr.oculus/1.43.32 (Linux; U; Android 12; en_US; Quest 3; Build/SQ3A.220605.009.A1; Cronet/107.0.5284.2)"
    } else {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15"
    };
    Ok(ResolvedAudio {
        url: stream.url,
        headers: [("User-Agent".into(), ua.into())].into(),
        loudness_db: stream.loudness_db.map(f64::from),
        title: stream.title,
        duration: stream.duration_secs.map(f64::from),
    })
}

#[tauri::command]
pub async fn play_track(
    track_id: String,
    title: Option<String>,
    artist: Option<String>,
    thumbnail: Option<String>,
    duration: Option<f64>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    if track_id.trim().is_empty() {
        return Err(
            "This song has no YouTube track ID. Refresh the playlist and try again.".into(),
        );
    }
    println!("[NOVA] Resolving audio stream for videoId: {track_id}");
    let quality = audio_quality(state.db.get_setting("quality").await.ok().flatten());
    let normalize = state
        .db
        .get_setting("replay_gain")
        .await
        .ok()
        .flatten()
        .as_deref()
        != Some("false");
    let speed = state
        .db
        .get_setting("playback_speed")
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(1.0)
        .clamp(0.25, 2.0);
    state.player.set_speed(speed).map_err(|e| e.to_string())?;
    let resolved = resolve_audio(&state, &track_id, quality).await?;
    let play_url = crate::audioproxy::register(&resolved.url, &resolved.headers)
        .unwrap_or_else(|| resolved.url.clone());
    state
        .player
        .load(
            &play_url,
            &resolved.headers,
            if normalize {
                resolved.loudness_db
            } else {
                None
            },
            None,
        )
        .map_err(|e| format!("Player load error: {e}"))?;
    state
        .player
        .play()
        .map_err(|e| format!("Player play error: {e}"))?;

    let track_info = serde_json::json!({
        "id": track_id.clone(),
        "title": title.or(resolved.title).unwrap_or_else(|| "Playing Track".into()),
        "artist": artist.unwrap_or_else(|| "YouTube Music".into()),
        "thumbnail": thumbnail.unwrap_or_else(|| "/placeholder.jpg".into()),
        "duration": duration.or(resolved.duration).unwrap_or(180.0),
        "url": play_url,
    });
    if state
        .db
        .get_setting("history_enabled")
        .await
        .ok()
        .flatten()
        .as_deref()
        != Some("false")
    {
        let _ = state
            .db
            .record_local_play(
                &track_id,
                track_info["title"].as_str().unwrap_or("Playing Track"),
                track_info["artist"].as_str().unwrap_or("YouTube Music"),
                track_info["thumbnail"].as_str().unwrap_or(""),
                track_info["duration"].as_f64().unwrap_or(0.0),
            )
            .await;
    }
    *state.current_track.lock().unwrap() = Some(track_info.clone());
    Ok(track_info)
}

#[tauri::command]
pub async fn download_current_track(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let track = state
        .current_track
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "Play a song first, then you can download it here.".to_string())?;
    let track_id = track["id"]
        .as_str()
        .ok_or("The current song has no track ID.")?;
    let title = track["title"].as_str().unwrap_or("NOVA Music download");
    download_audio(&state, &app, track_id, title).await
}

#[tauri::command]
pub async fn download_track(
    app: AppHandle,
    track_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if track_id.trim().is_empty() {
        return Err("This song has no YouTube track ID, so NOVA cannot download it.".into());
    }
    download_audio(&state, &app, &track_id, &title).await
}

fn emit_download_update(
    app: &AppHandle,
    id: &str,
    track_id: &str,
    title: &str,
    status: &str,
    progress: Option<f64>,
    bytes_downloaded: u64,
    total_bytes: Option<u64>,
    path: Option<&str>,
    error: Option<&str>,
) {
    let _ = app.emit(
        "download-updated",
        serde_json::json!({
            "id": id, "trackId": track_id, "title": title, "status": status,
            "progress": progress, "bytesDownloaded": bytes_downloaded,
            "totalBytes": total_bytes, "path": path, "error": error,
        }),
    );
}

async fn download_audio(
    state: &AppState,
    app: &AppHandle,
    track_id: &str,
    title: &str,
) -> Result<String, String> {
    let id = format!(
        "{}-{}",
        chrono::Utc::now().timestamp_millis(),
        rand::random::<u32>()
    );
    emit_download_update(
        app, &id, track_id, title, "starting", None, 0, None, None, None,
    );
    let result = download_audio_inner(state, app, &id, track_id, title).await;
    match &result {
        Ok(path) => emit_download_update(
            app,
            &id,
            track_id,
            title,
            "completed",
            Some(100.0),
            0,
            None,
            Some(path),
            None,
        ),
        Err(error) => emit_download_update(
            app,
            &id,
            track_id,
            title,
            "failed",
            None,
            0,
            None,
            None,
            Some(error),
        ),
    }
    result
}

async fn download_audio_inner(
    state: &AppState,
    app: &AppHandle,
    job_id: &str,
    track_id: &str,
    title: &str,
) -> Result<String, String> {
    let quality = audio_quality(
        state
            .db
            .get_setting("download_quality")
            .await
            .ok()
            .flatten()
            .or(state.db.get_setting("quality").await.ok().flatten()),
    );
    let resolved = resolve_audio(state, track_id, quality).await?;
    let format = state
        .db
        .get_setting("download_format")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "original".into());

    let directory = state
        .db
        .get_setting("download_directory")
        .await
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("Music")
                .join("NOVA Music Downloads")
        });
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|e| format!("Could not create the download folder: {e}"))?;
    let client = reqwest::Client::new();
    let mut request = client.get(&resolved.url);
    for (name, value) in &resolved.headers {
        let key =
            reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|e| e.to_string())?;
        let value = reqwest::header::HeaderValue::from_str(value).map_err(|e| e.to_string())?;
        request = request.header(key, value);
    }
    let mut response = request
        .send()
        .await
        .map_err(|e| format!("Download request failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("The audio server refused the download: {e}"))?;
    let total_bytes = response.content_length();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let source_extension = if content_type.contains("mp4") {
        "m4a"
    } else if content_type.contains("mpeg") || content_type.contains("mp3") {
        "mp3"
    } else {
        "webm"
    };

    let safe_title: String = title
        .chars()
        .take(150)
        .map(|c| {
            if c.is_control() || "/\\:*?\"<>|".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let safe_title = safe_title.trim().trim_matches('.');
    let safe_title = if safe_title.is_empty() {
        "NOVA Music download"
    } else {
        safe_title
    };
    let extension = match format.as_str() {
        "mp3" => "mp3",
        "opus" => "opus",
        "flac" => "flac",
        _ => source_extension,
    };
    let mut index = 1;
    let destination = loop {
        let suffix = if index == 1 {
            String::new()
        } else {
            format!(" ({index})")
        };
        let path = directory.join(format!("{safe_title}{suffix}.{extension}"));
        if tokio::fs::metadata(&path).await.is_err() {
            break path;
        }
        index += 1;
    };
    let partial = directory.join(format!(
        ".{safe_title}.{}.{}.part",
        job_id, source_extension
    ));
    let mut file = tokio::fs::File::create(&partial)
        .await
        .map_err(|e| format!("Could not create download file: {e}"))?;
    let mut bytes_downloaded = 0u64;
    let mut last_progress_update = std::time::Instant::now();
    loop {
        let chunk = match response.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                drop(file);
                let _ = tokio::fs::remove_file(&partial).await;
                return Err(format!("Audio download stopped: {error}"));
            }
        };
        if let Err(error) = file.write_all(&chunk).await {
            drop(file);
            let _ = tokio::fs::remove_file(&partial).await;
            return Err(format!("Could not write audio download: {error}"));
        }
        bytes_downloaded += chunk.len() as u64;
        let progress = total_bytes
            .map(|total| (bytes_downloaded as f64 / total.max(1) as f64 * 100.0).min(99.0));
        if last_progress_update.elapsed() >= std::time::Duration::from_millis(250)
            || progress.is_some_and(|value| value >= 99.0)
        {
            emit_download_update(
                app,
                job_id,
                track_id,
                title,
                "downloading",
                progress,
                bytes_downloaded,
                total_bytes,
                None,
                None,
            );
            last_progress_update = std::time::Instant::now();
        }
    }
    if let Err(error) = file.flush().await {
        drop(file);
        let _ = tokio::fs::remove_file(&partial).await;
        return Err(format!("Could not finish audio download: {error}"));
    }
    drop(file);
    if format == "original" || !matches!(format.as_str(), "mp3" | "opus" | "flac") {
        if let Err(error) = tokio::fs::rename(&partial, &destination).await {
            let _ = tokio::fs::remove_file(&partial).await;
            return Err(format!("Could not finish the downloaded file: {error}"));
        }
    } else {
        emit_download_update(
            app,
            job_id,
            track_id,
            title,
            "converting",
            None,
            bytes_downloaded,
            total_bytes,
            None,
            None,
        );
        let converted = directory.join(format!(".{safe_title}.{}.converted.{extension}", job_id));
        let ffmpeg = app
            .path()
            .resolve("ffmpeg", tauri::path::BaseDirectory::Resource)
            .ok()
            .filter(|path| path.is_file())
            .or_else(|| {
                ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"]
                    .into_iter()
                    .map(std::path::PathBuf::from)
                    .find(|path| path.is_file())
            });
        let mut command = tokio::process::Command::new(
            ffmpeg.unwrap_or_else(|| std::path::PathBuf::from("ffmpeg")),
        );
        command
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-i")
            .arg(&partial)
            .arg("-vn");
        match format.as_str() {
            "mp3" => {
                command.args(["-c:a", "libmp3lame", "-q:a", "2"]);
            }
            "opus" => {
                command.args(["-c:a", "libopus", "-b:a", "160k"]);
            }
            "flac" => {
                command.args(["-c:a", "flac"]);
            }
            _ => {}
        }
        command.arg(&converted);
        let status = match command.status().await {
            Ok(status) => status,
            Err(error) => {
                let _ = tokio::fs::remove_file(&partial).await;
                let _ = tokio::fs::remove_file(&converted).await;
                return Err(format!(
                    "Audio conversion needs FFmpeg installed and available on PATH: {error}"
                ));
            }
        };
        if !status.success() {
            let _ = tokio::fs::remove_file(&converted).await;
            let _ = tokio::fs::remove_file(&partial).await;
            return Err(
                "FFmpeg could not convert this audio stream to the selected format.".into(),
            );
        }
        if let Err(error) = tokio::fs::rename(&converted, &destination).await {
            let _ = tokio::fs::remove_file(&partial).await;
            let _ = tokio::fs::remove_file(&converted).await;
            return Err(format!("Could not save converted audio: {error}"));
        }
        let _ = tokio::fs::remove_file(&partial).await;
    }
    Ok(destination.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn enqueue_track(
    mut track: serde_json::Value,
    play_next: bool,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    if !track.is_object() {
        return Err("This song has invalid track details.".into());
    }
    let track_id = track
        .get("videoId")
        .or_else(|| track.get("video_id"))
        .or_else(|| track.get("trackId"))
        .or_else(|| track.get("track_id"))
        .or_else(|| track.get("id"))
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| "This song is missing its YouTube track ID.".to_string())?
        .to_string();
    let title = track
        .get("title")
        .or_else(|| track.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown track")
        .to_string();
    let artist = track
        .get("artist")
        .or_else(|| track.get("artists"))
        .or_else(|| track.get("subtitle"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("YouTube Music")
        .to_string();
    let thumbnail = track
        .get("thumbnail")
        .or_else(|| track.get("thumbnail_url"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    let duration = track
        .get("duration_seconds")
        .or_else(|| track.get("duration"))
        .and_then(|value| {
            value.as_f64().or_else(|| {
                value.as_str().and_then(|text| {
                    let parts = text
                        .split(':')
                        .map(str::parse::<f64>)
                        .collect::<Result<Vec<_>, _>>()
                        .ok()?;
                    match parts.as_slice() {
                        [minutes, seconds] => Some(minutes * 60.0 + seconds),
                        [hours, minutes, seconds] => {
                            Some(hours * 3600.0 + minutes * 60.0 + seconds)
                        }
                        _ => None,
                    }
                })
            })
        });
    track["id"] = serde_json::Value::String(track_id.clone());
    track["track_id"] = serde_json::Value::String(track_id);
    track["title"] = serde_json::Value::String(title);
    track["artist"] = serde_json::Value::String(artist);
    track["thumbnail"] = serde_json::Value::String(thumbnail);
    if let Some(duration) = duration {
        track["duration"] = serde_json::json!(duration);
        track["duration_seconds"] = serde_json::json!(duration);
    }

    let mut queue = state.queue.lock().map_err(|e| e.to_string())?;
    if play_next {
        queue.insert(0, track);
    } else {
        queue.push(track);
    }
    Ok(queue.len())
}

#[tauri::command]
pub async fn dequeue_next_track(
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, String> {
    let mut queue = state.queue.lock().map_err(|e| e.to_string())?;
    if queue.is_empty() {
        Ok(None)
    } else {
        Ok(Some(queue.remove(0)))
    }
}

#[tauri::command]
pub async fn get_autoplay_track(
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, String> {
    {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        if !queue.is_empty() {
            return Ok(Some(queue.remove(0)));
        }
    }
    let current = state
        .current_track
        .lock()
        .map_err(|error| error.to_string())?
        .clone();
    let Some(current) = current else {
        return Ok(None);
    };
    let Some(video_id) = current.get("id").and_then(serde_json::Value::as_str) else {
        return Ok(None);
    };
    let _ = crate::auth::ensure_access_token(&state).await;
    let metadata = metadata_client()?;
    let page = state
        .innertube
        .next(&metadata, Some(video_id), None)
        .await
        .map_err(|error| format!("Could not load YouTube Music autoplay: {error}"))?;
    let items = if page.items.len() <= 1 {
        if let Some(radio) = page.automix_playlist_id.as_deref() {
            match state
                .innertube
                .next(&metadata, Some(video_id), Some(radio))
                .await
            {
                Ok(continued) => continued.items,
                Err(_) => page.items,
            }
        } else {
            page.items
        }
    } else {
        page.items
    };
    let mut recommendations = items
        .into_iter()
        .filter(|item| item.video_id != video_id)
        .map(|item| serde_json::to_value(item).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if recommendations.is_empty() {
        return Ok(None);
    }

    // If a song was queued while the network request was in flight, honor that user choice.
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    if !queue.is_empty() {
        return Ok(Some(queue.remove(0)));
    }
    let next = recommendations.remove(0);
    queue.extend(recommendations);
    Ok(Some(next))
}

#[tauri::command]
pub async fn pause_playback(state: State<'_, AppState>) -> Result<bool, String> {
    state.player.toggle().map_err(|e| e.to_string())?;
    Ok(state.player.is_paused())
}

#[tauri::command]
pub async fn skip_track(state: State<'_, AppState>) -> Result<(), String> {
    state.player.next().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn previous_track(state: State<'_, AppState>) -> Result<(), String> {
    state.player.prev().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn seek(position: f64, state: State<'_, AppState>) -> Result<(), String> {
    state.player.seek(position).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_volume(volume: i64, state: State<'_, AppState>) -> Result<(), String> {
    state.player.set_volume(volume).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rate_track(
    track_id: String,
    liked: bool,
    track: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let track = track
        .filter(serde_json::Value::is_object)
        .or_else(|| {
            state
                .current_track
                .lock()
                .ok()
                .and_then(|current| current.clone())
                .filter(|current| {
                    current.get("id").and_then(serde_json::Value::as_str) == Some(track_id.as_str())
                })
        })
        .unwrap_or_else(|| serde_json::json!({}));
    let title = track
        .get("title")
        .or_else(|| track.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown track");
    let artist = track
        .get("artist")
        .or_else(|| track.get("artists"))
        .or_else(|| track.get("subtitle"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("YouTube Music");
    let album = track
        .get("album")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let thumbnail = track
        .get("thumbnail")
        .or_else(|| track.get("thumbnail_url"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let duration = track
        .get("duration_seconds")
        .or_else(|| track.get("duration"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let artist_id = track
        .get("artistId")
        .or_else(|| track.get("artist_id"))
        .and_then(serde_json::Value::as_str);
    let album_id = track
        .get("albumId")
        .or_else(|| track.get("album_id"))
        .and_then(serde_json::Value::as_str);
    state
        .db
        .set_liked_track(
            &track_id, title, artist, album, thumbnail, duration, artist_id, album_id, liked,
        )
        .await
        .map_err(|error| error.to_string())?;
    if let Some(track) = state.current_track.lock().unwrap().as_mut() {
        if track.get("id").and_then(serde_json::Value::as_str) == Some(track_id.as_str()) {
            track["liked"] = serde_json::Value::Bool(liked);
            track["rating"] =
                serde_json::Value::String(if liked { "like" } else { "indifferent" }.into());
        }
    }

    // The local library is authoritative for the UI. Sync to YouTube in the background so a
    // slow or rejected Innertube request cannot make the like button appear to hang or lose data.
    let sync_state = state.inner().clone();
    let sync_track_id = track_id.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::auth::ensure_access_token(&sync_state).await;
        if !sync_state.innertube.is_logged_in() {
            return;
        }
        let Ok(client) = metadata_client() else {
            return;
        };
        let rating = if liked {
            innertube::Rating::Like
        } else {
            innertube::Rating::Indifferent
        };
        if let Err(error) = sync_state
            .innertube
            .rate(&client, &sync_track_id, rating)
            .await
        {
            tracing::warn!(track_id = %sync_track_id, error = %error, "Could not sync local like to YouTube Music");
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn add_track_to_playlist(
    track_id: String,
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if track_id.trim().is_empty() || playlist_id.trim().is_empty() {
        return Err("Choose a song and playlist first.".into());
    }
    let _ = crate::auth::ensure_access_token(&state).await;
    if !state.innertube.is_logged_in() {
        return Err("Sign in to your YouTube Music account to edit playlists.".into());
    }
    let client = metadata_client()?;
    match state
        .innertube
        .playlist_add(&client, &playlist_id, &track_id, false)
        .await
        .map_err(|e| e.to_string())?
    {
        true => Ok(()),
        false => Err("That song is already in this playlist.".into()),
    }
}

#[tauri::command]
pub async fn get_queue(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let queue = state.queue.lock().map_err(|e| e.to_string())?.clone();
    serde_json::to_value(queue).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_now_playing(
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, String> {
    let cur = state.current_track.lock().unwrap().clone();
    Ok(cur)
}

#[tauri::command]
pub async fn get_lyrics(
    video_id: String,
    title: String,
    artist: String,
    album: Option<String>,
    duration: Option<f64>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let metadata = metadata_client()?;
    let lyrics_clients = innertube::clients::Clients::bundled();

    // YouTube Music exposes the lyrics browse id through `next`; a video id alone is not a
    // lyrics browse id. Prefer its own timed lyrics, then its plain lyrics, before using LRCLIB.
    let browse_id = state
        .innertube
        .next(&metadata, Some(&video_id), None)
        .await
        .ok()
        .and_then(|next| next.lyrics_browse_id);

    if let Some(browse_id) = browse_id {
        if let Some(mobile) = lyrics_clients.get(innertube::clients::LYRICS_TIMED_CLIENT) {
            if let Ok(lines) = state.innertube.lyrics_timed(mobile, &browse_id).await {
                if !lines.is_empty() {
                    return Ok(serde_json::json!({
                        "lines": lines.into_iter().map(|line| serde_json::json!({
                            "time_ms": line.time_ms,
                            "text": line.text,
                        })).collect::<Vec<_>>(),
                        "plain_text": null,
                        "attribution": null,
                        "source": "YouTube Music",
                    }));
                }
            }
        }
        if let Ok(Some(plain)) = state.innertube.lyrics_plain(&metadata, &browse_id).await {
            if !plain.text.trim().is_empty() {
                return Ok(serde_json::json!({
                    "lines": [],
                    "plain_text": plain.text,
                    "attribution": plain.footer,
                    "source": "YouTube Music",
                }));
            }
        }
    }

    if let Some(result) = fetch_lrclib_lyrics(&title, &artist, album.as_deref(), duration).await {
        return Ok(result);
    }

    Ok(serde_json::json!({
        "lines": [],
        "plain_text": null,
        "attribution": null,
        "source": null,
    }))
}

async fn fetch_lrclib_lyrics(
    title: &str,
    artist: &str,
    album: Option<&str>,
    duration: Option<f64>,
) -> Option<serde_json::Value> {
    let title = title.trim();
    let artist = artist.trim();
    if title.is_empty() || artist.is_empty() {
        return None;
    }

    let mut params = vec![("track_name", title), ("artist_name", artist)];
    if let Some(album) = album.map(str::trim).filter(|value| !value.is_empty()) {
        params.push(("album_name", album));
    }
    let duration_text = duration
        .filter(|value| value.is_finite() && *value > 0.0)
        .map(|value| value.to_string());
    if let Some(value) = duration_text.as_deref() {
        params.push(("duration", value));
    }

    let mut response = crate::http::client()
        .get("https://lrclib.net/api/get")
        .header(
            reqwest::header::USER_AGENT,
            "NOVA-Music/1.0 (desktop lyrics)",
        )
        .timeout(std::time::Duration::from_secs(8))
        .query(&params)
        .send()
        .await
        .ok()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND
        && params.iter().any(|(key, _)| *key == "album_name")
    {
        params.retain(|(key, _)| *key != "album_name");
        response = crate::http::client()
            .get("https://lrclib.net/api/get")
            .header(
                reqwest::header::USER_AGENT,
                "NOVA-Music/1.0 (desktop lyrics)",
            )
            .timeout(std::time::Duration::from_secs(8))
            .query(&params)
            .send()
            .await
            .ok()?;
    }
    if !response.status().is_success() {
        return None;
    }
    let data: serde_json::Value = response.json().await.ok()?;
    let plain = data
        .get("plainLyrics")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let synced = data
        .get("syncedLyrics")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty());

    let lines = synced.map(parse_lrc_lines).unwrap_or_default();
    if lines.is_empty() && plain.is_none() {
        return None;
    }
    Some(serde_json::json!({
        "lines": lines,
        "plain_text": plain,
        "attribution": "LRCLIB",
        "source": "LRCLIB",
    }))
}

fn parse_lrc_lines(lrc: &str) -> Vec<serde_json::Value> {
    let mut lines = Vec::new();
    for row in lrc.lines() {
        let mut rest = row.trim();
        let mut timestamps = Vec::new();
        while let Some(after_open) = rest.strip_prefix('[') {
            let Some((tag, after_tag)) = after_open.split_once(']') else {
                break;
            };
            if let Some(time_ms) = parse_lrc_timestamp(tag) {
                timestamps.push(time_ms);
            }
            rest = after_tag;
        }
        let text = rest.trim();
        if text.is_empty() {
            continue;
        }
        for time_ms in timestamps {
            lines.push(serde_json::json!({ "time_ms": time_ms, "text": text }));
        }
    }
    lines.sort_by_key(|line| {
        line.get("time_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    });
    lines
}

fn parse_lrc_timestamp(value: &str) -> Option<u64> {
    let parts: Vec<_> = value.split(':').collect();
    let (minutes, seconds) = match parts.as_slice() {
        [minutes, seconds] => (minutes.parse::<u64>().ok()?, seconds.parse::<f64>().ok()?),
        [hours, minutes, seconds] => (
            hours
                .parse::<u64>()
                .ok()?
                .checked_mul(60)?
                .checked_add(minutes.parse::<u64>().ok()?)?,
            seconds.parse::<f64>().ok()?,
        ),
        _ => return None,
    };
    if !seconds.is_finite() || !(0.0..60.0).contains(&seconds) {
        return None;
    }
    minutes
        .checked_mul(60)?
        .checked_add(seconds as u64)?
        .checked_mul(1000)
        .and_then(|whole| whole.checked_add((seconds.fract() * 1000.0).round() as u64))
}

#[tauri::command]
pub async fn get_playback_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(state.player.get_playback_state())
}

#[tauri::command]
pub async fn show_song_notification(title: String, artist: String) -> Result<(), String> {
    notify_rust::Notification::new()
        .summary(&title)
        .body(&artist)
        .appname("NOVA Music")
        .show()
        .map(|_| ())
        .map_err(|error| format!("Could not show song notification: {error}"))
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    match state.db.get_all_settings().await {
        Ok(settings) => Ok(settings),
        Err(e) => Err(e.to_string()),
    }
}

fn directory_size(path: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let Ok(file_type) = entry.file_type() else {
                return 0;
            };
            if file_type.is_dir() {
                directory_size(&entry.path())
            } else if file_type.is_file() {
                entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
            } else {
                0
            }
        })
        .sum()
}

#[tauri::command]
pub async fn get_storage_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let database_path = state.app_data_dir.join("novamusic.sqlite");
    let database_bytes = std::fs::metadata(&database_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let cache_bytes = directory_size(&state.audio_cache_dir);
    let download_directory = state
        .db
        .get_setting("download_directory")
        .await
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("Music")
                .join("NOVA Music Downloads")
        });
    Ok(serde_json::json!({
        "databaseBytes": database_bytes,
        "playbackCacheBytes": cache_bytes,
        "downloadsBytes": directory_size(&download_directory),
        "databasePath": database_path.to_string_lossy(),
        "downloadDirectory": download_directory.to_string_lossy(),
    }))
}

#[tauri::command]
pub async fn optimize_database(state: State<'_, AppState>) -> Result<(), String> {
    state.db.optimize().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_settings(
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    match key.as_str() {
        "quality" | "download_quality"
            if !matches!(value.as_str(), "high" | "medium" | "low" | "auto") =>
        {
            return Err("Choose high, medium, low, or auto audio quality.".into());
        }
        "download_format" if !matches!(value.as_str(), "original" | "mp3" | "opus" | "flac") => {
            return Err("Choose original, MP3, Opus, or FLAC download format.".into());
        }
        "appearance_backdrop" if !matches!(value.as_str(), "artwork" | "gradient" | "flat") => {
            return Err("Choose artwork, gradient, or minimal dark background.".into());
        }
        "motion_preference" if !matches!(value.as_str(), "full" | "reduced") => {
            return Err("Choose full or reduced motion.".into());
        }
        "accent_color"
            if value.len() != 7
                || !value.starts_with('#')
                || !value[1..]
                    .chars()
                    .all(|character| character.is_ascii_hexdigit()) =>
        {
            return Err("Accent color must be a six-digit hex color.".into());
        }
        "replay_gain" | "gapless" | "auto_play_next" | "history_enabled"
            if !matches!(value.as_str(), "true" | "false") =>
        {
            return Err("This setting must be true or false.".into());
        }
        "google_oauth_client_id"
            if !value.trim().is_empty()
                && !value.trim().ends_with(".apps.googleusercontent.com") =>
        {
            return Err(
                "Enter a Google OAuth client ID ending in .apps.googleusercontent.com.".into(),
            );
        }
        "volume" => {
            let volume = value
                .parse::<i64>()
                .map_err(|_| "Volume must be between 0 and 100.")?
                .clamp(0, 100);
            state.player.set_volume(volume).map_err(|e| e.to_string())?;
            state
                .db
                .set_setting(&key, &volume.to_string())
                .await
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        "playback_speed" => {
            let speed = value
                .parse::<f64>()
                .map_err(|_| "Choose a playback speed between 0.25 and 2.")?
                .clamp(0.25, 2.0);
            state.player.set_speed(speed).map_err(|e| e.to_string())?;
            state
                .db
                .set_setting(&key, &speed.to_string())
                .await
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
        "gapless" => state
            .player
            .set_gapless(value == "true")
            .map_err(|e| e.to_string())?,
        "download_directory" if !value.trim().is_empty() => {
            let path = std::path::Path::new(&value);
            if !path.is_absolute() {
                return Err("Choose an absolute download folder path.".into());
            }
            tokio::fs::create_dir_all(path)
                .await
                .map_err(|e| format!("Could not use that download folder: {e}"))?;
        }
        _ => {}
    }
    match state.db.set_setting(&key, &value).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn start_google_sign_in(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    crate::auth::start_google_sign_in(app, state.inner().clone()).await
}

#[tauri::command]
pub async fn sign_in(
    cookie: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let cookie = match cookie {
        Some(c) if !c.trim().is_empty() => c.trim().to_string(),
        _ => {
            return Err("Use the Google sign-in button to connect your account.".into());
        }
    };

    if innertube::cookie_sapisid(&cookie).is_none() {
        return Err("Invalid cookie: must contain 'SAPISID'. To get your YouTube Music session cookie, visit music.youtube.com in your browser, open Developer Tools (Cmd+Option+I) -> Application -> Cookies -> https://music.youtube.com, and copy the full Cookie string. (If you have an OAuth code, paste it in the Verify Code box above instead.)".into());
    }

    // 1. Immediately update the live InnerTube session
    state.innertube.set_cookie(Some(cookie.clone()));

    // 2. Verify session against YouTube Music account menu without 90s healing hang
    let client = metadata_client()?;
    let verification = innertube::without_healing(state.innertube.account_menu(&client)).await;

    match verification {
        Ok(account) => {
            let _ = state.db.set_session_cookie(&cookie).await;
            if let Some(vd) = account.visitor_data {
                state.innertube.set_visitor_data(Some(vd.clone()));
                let _ = state.db.set_visitor_data(&vd).await;
            }
            let name = account.name.unwrap_or_else(|| "YouTube Music User".into());
            let email = account.email.unwrap_or_default();
            let _ = state.db.set_setting("account_name", &name).await;
            let _ = state.db.set_setting("account_email", &email).await;

            Ok(serde_json::json!({
                "status": "logged_in",
                "name": name,
                "email": email,
            }))
        }
        Err(e) => {
            state.innertube.set_cookie(None);
            Err(format!(
                "Cookie verification failed with YouTube Music: {e}"
            ))
        }
    }
}

#[tauri::command]
pub async fn get_account_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let oauth_token = crate::auth::ensure_access_token(&state)
        .await
        .ok()
        .flatten();
    let is_logged_in = state.innertube.is_logged_in() || oauth_token.is_some();
    let name = state.db.get_setting("account_name").await.ok().flatten();
    let email = state.db.get_setting("account_email").await.ok().flatten();
    Ok(serde_json::json!({
        "is_logged_in": is_logged_in,
        "name": name,
        "email": email,
    }))
}

#[tauri::command]
pub async fn sign_out(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(token) = state
        .db
        .get_setting("oauth_access_token")
        .await
        .ok()
        .flatten()
        .filter(|v| !v.is_empty())
    {
        let _ = crate::auth::revoke_token(&token).await;
    }
    state.innertube.set_cookie(None);
    state.innertube.set_oauth_token(None);
    state.innertube.set_visitor_data(None);
    state.innertube.set_data_sync_id(None);
    let _ = state.db.set_setting("session_cookie", "").await;
    let _ = state.db.set_setting("visitor_data", "").await;
    let _ = state.db.set_setting("account_name", "").await;
    let _ = state.db.set_setting("account_email", "").await;
    let _ = state.db.set_setting("oauth_access_token", "").await;
    let _ = state.db.set_setting("oauth_refresh_token", "").await;
    let _ = state.db.set_setting("oauth_expires_at", "").await;
    Ok(())
}

#[tauri::command]
pub async fn switch_account(account_id: String, state: State<'_, AppState>) -> Result<(), String> {
    crate::auth::switch_account(account_id, state).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_playback_pipeline() {
        use std::collections::HashMap;
        use std::time::Duration;

        let tmp = std::env::temp_dir().join("nova-test-pipeline");
        std::fs::create_dir_all(&tmp).ok();
        let state = AppState::new(&tmp).await.expect("AppState");

        // 1. Warm visitor data
        let vd = state.innertube.fetch_visitor_data().await.expect("vd");
        state.innertube.set_visitor_data(Some(vd));

        // 2. Query player via VISIONOS
        let clients = innertube::Clients::bundled();
        let client = clients.get("VISIONOS").unwrap();
        let resp = state
            .innertube
            .player(client, "hLuhfSP8Odc", None, None, None)
            .await
            .expect("player");
        let sd = resp.streaming_data.expect("sd");
        let fmt = innertube::find_format(&sd, innertube::AudioQuality::High).expect("fmt");
        let direct_url = fmt.direct_url().expect("url");

        // 3. Register with audioproxy
        crate::audioproxy::start();
        let mut headers = HashMap::new();
        headers.insert("User-Agent".into(), client.user_agent.clone());
        let play_url = crate::audioproxy::register(&direct_url, &headers).expect("register");
        println!("Loopback proxy URL: {}", play_url);

        // 4. Load into player
        state
            .player
            .load(&play_url, &headers, fmt.loudness_db.map(|l| l as f64), None)
            .expect("load");
        state.player.play().expect("play");

        // 5. Wait 2 seconds and observe position advancing
        tokio::time::sleep(Duration::from_millis(2000)).await;
        let pos1 = state.player.position();
        println!("Observed playback position after 2s: {:?}", pos1);
        assert!(pos1 > 0.0, "Position must advance: pos={pos1}");

        // 6. Test Pause
        state.player.pause().expect("pause");
        assert!(state.player.is_paused(), "Player must be paused");
        tokio::time::sleep(Duration::from_millis(500)).await;
        let pos_paused = state.player.position();
        assert!(
            pos_paused - pos1 < 1.0,
            "Position should stop advancing while paused: before={pos1}, paused={pos_paused}"
        );

        // 7. Test Resume
        state.player.play().expect("resume");
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(!state.player.is_paused(), "Player must be playing");

        // 8. Test Seek
        state.player.seek(45.0).expect("seek");
        tokio::time::sleep(Duration::from_millis(600)).await;
        let pos_after_seek = state.player.position();
        println!("Position after seek to 45.0s: {:?}", pos_after_seek);
        assert!(
            pos_after_seek >= 44.0,
            "Position should be near seek target"
        );

        // 9. Test Volume
        state.player.set_volume(60).expect("set_volume");
        assert_eq!(state.player.volume(), 60, "Volume should be 60");

        // 10. Test Switching Tracks
        let resp2 = state
            .innertube
            .player(client, "VOvXf07g0H0", None, None, None)
            .await
            .expect("player track 2");
        let sd2 = resp2.streaming_data.expect("sd2");
        let fmt2 = innertube::find_format(&sd2, innertube::AudioQuality::High).expect("fmt2");
        let direct_url2 = fmt2.direct_url().expect("url2");
        let play_url2 = crate::audioproxy::register(&direct_url2, &headers).expect("register 2");
        state
            .player
            .load(
                &play_url2,
                &headers,
                fmt2.loudness_db.map(|l| l as f64),
                None,
            )
            .expect("load track 2");
        state.player.play().expect("play track 2");
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let pos_track2 = state.player.position();
        println!("Track 2 playback position after switch: {:?}", pos_track2);
        assert!(
            pos_track2 > 0.0,
            "Track 2 position must advance after switch"
        );

        println!("ALL PLAYBACK FEATURES (play, pause, resume, seek, volume, track switch) VERIFIED 100%!");
    }
}
