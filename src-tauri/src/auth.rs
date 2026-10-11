use crate::state::AppState;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const DEFAULT_GOOGLE_CLIENT_ID: &str =
    "520951322239-ats5sdthdf3se27eei9f138plug6gndi.apps.googleusercontent.com";
const YOUTUBE_READ_SCOPE: &str = "https://www.googleapis.com/auth/youtube.readonly";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    #[serde(default)]
    pub expires_in: i64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub token_type: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GoogleProfile {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
}

async fn client_id(state: &AppState) -> Result<String, String> {
    let configured = state
        .db
        .get_setting("google_oauth_client_id")
        .await
        .map_err(|e| e.to_string())?
        .filter(|value| !value.trim().is_empty());
    Ok(configured
        .or_else(|| {
            std::env::var("GOOGLE_OAUTH_CLIENT_ID")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| DEFAULT_GOOGLE_CLIENT_ID.to_owned()))
}

/// Start Google's installed-app OAuth flow in the user's default browser. The loopback listener
/// receives Google's redirect and returns the completed sign-in to the running app.
pub async fn start_google_sign_in(app: AppHandle, state: AppState) -> Result<(), String> {
    let client_id = client_id(&state).await?;
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| format!("Could not start the Google sign-in callback: {e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Could not configure the Google sign-in callback: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Could not read the Google sign-in callback address: {e}"))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/oauth2callback");

    let mut state_bytes = [0_u8; 32];
    let mut verifier_bytes = [0_u8; 48];
    rand::thread_rng().fill_bytes(&mut state_bytes);
    rand::thread_rng().fill_bytes(&mut verifier_bytes);
    let oauth_state = URL_SAFE_NO_PAD.encode(state_bytes);
    let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    let mut url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|e| format!("Could not create the Google sign-in URL: {e}"))?;
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair(
            "scope",
            &format!("openid email profile {YOUTUBE_READ_SCOPE}"),
        )
        .append_pair("access_type", "offline")
        .append_pair("include_granted_scopes", "true")
        .append_pair("prompt", "select_account consent")
        .append_pair("state", &oauth_state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");

    open_external_url(url.as_str())?;

    std::thread::Builder::new()
        .name("nova-google-oauth-callback".into())
        .spawn(move || {
            let callback = receive_oauth_callback(listener, &oauth_state);
            let (code, verifier) = match callback {
                Ok(code) => (code, verifier),
                Err(error) => {
                    let _ = app.emit("google-auth-complete", json!({"ok": false, "error": error}));
                    return;
                }
            };

            tauri::async_runtime::spawn(async move {
                match complete_sign_in(&state, &client_id, &redirect_uri, &code, &verifier).await {
                    Ok(profile) => {
                        let _ = app.emit(
                            "google-auth-complete",
                            json!({"ok": true, "name": profile.name, "email": profile.email}),
                        );
                    }
                    Err(error) => {
                        let _ =
                            app.emit("google-auth-complete", json!({"ok": false, "error": error}));
                    }
                }
            });
        })
        .map_err(|e| format!("Could not start Google sign-in: {e}"))?;

    Ok(())
}

async fn complete_sign_in(
    state: &AppState,
    client_id: &str,
    redirect_uri: &str,
    code: &str,
    verifier: &str,
) -> Result<GoogleProfile, String> {
    let tokens = exchange_code(client_id, redirect_uri, code, verifier).await?;
    let profile = google_profile(&tokens.access_token).await?;
    persist_tokens(state, &tokens).await?;
    let name = if profile.name.is_empty() {
        profile.email.clone()
    } else {
        profile.name.clone()
    };
    state
        .db
        .set_setting("account_name", &name)
        .await
        .map_err(|e| e.to_string())?;
    state
        .db
        .set_setting("account_email", &profile.email)
        .await
        .map_err(|e| e.to_string())?;
    Ok(profile)
}

async fn exchange_code(
    client_id: &str,
    redirect_uri: &str,
    code: &str,
    verifier: &str,
) -> Result<OAuthTokens, String> {
    reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("code_verifier", verifier),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| format!("Google token exchange failed: {e}"))?
        .error_for_status()
        .map_err(|e| {
            format!(
                "Google rejected the sign-in. Check the Desktop OAuth client configuration: {e}"
            )
        })?
        .json::<OAuthTokens>()
        .await
        .map_err(|e| format!("Could not read Google's sign-in response: {e}"))
}

async fn google_profile(access_token: &str) -> Result<GoogleProfile, String> {
    reqwest::Client::new()
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Could not read the signed-in Google account: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Google could not verify the signed-in account: {e}"))?
        .json::<GoogleProfile>()
        .await
        .map_err(|e| format!("Could not read the Google account profile: {e}"))
}

async fn persist_tokens(state: &AppState, tokens: &OAuthTokens) -> Result<(), String> {
    let expires_at = unix_now().saturating_add(tokens.expires_in.max(0));
    state
        .db
        .set_setting("oauth_access_token", &tokens.access_token)
        .await
        .map_err(|e| e.to_string())?;
    state
        .db
        .set_setting("oauth_expires_at", &expires_at.to_string())
        .await
        .map_err(|e| e.to_string())?;
    if let Some(refresh_token) = tokens.refresh_token.as_deref().filter(|v| !v.is_empty()) {
        state
            .db
            .set_setting("oauth_refresh_token", refresh_token)
            .await
            .map_err(|e| e.to_string())?;
    }
    state
        .innertube
        .set_oauth_token(Some(tokens.access_token.clone()));
    Ok(())
}

/// Return a current access token, refreshing it when necessary and updating the shared
/// InnerTube session so subsequent YouTube Music requests use the signed-in account.
pub async fn ensure_access_token(state: &AppState) -> Result<Option<String>, String> {
    let access = state
        .db
        .get_setting("oauth_access_token")
        .await
        .map_err(|e| e.to_string())?;
    let expiry = state
        .db
        .get_setting("oauth_expires_at")
        .await
        .map_err(|e| e.to_string())?
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_default();

    if let Some(access) = access.filter(|value| !value.is_empty()) {
        if expiry > unix_now().saturating_add(60) {
            state.innertube.set_oauth_token(Some(access.clone()));
            return Ok(Some(access));
        }
    }

    let refresh = state
        .db
        .get_setting("oauth_refresh_token")
        .await
        .map_err(|e| e.to_string())?
        .filter(|value| !value.is_empty());
    let Some(refresh_token) = refresh else {
        state.innertube.set_oauth_token(None);
        return Ok(None);
    };

    let client_id = client_id(state).await?;
    let tokens = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Could not refresh Google sign-in: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Google sign-in expired. Sign in again: {e}"))?
        .json::<OAuthTokens>()
        .await
        .map_err(|e| format!("Could not read Google's refreshed sign-in: {e}"))?;
    persist_tokens(state, &tokens).await?;
    Ok(Some(tokens.access_token))
}

pub async fn youtube_data_playlists(access_token: &str) -> Result<Vec<Value>, String> {
    let client = reqwest::Client::new();
    let mut token: Option<String> = None;
    let mut playlists = Vec::new();
    for _ in 0..100 {
        let mut request = client
            .get("https://www.googleapis.com/youtube/v3/playlists")
            .bearer_auth(access_token)
            .query(&[
                ("part", "snippet,contentDetails"),
                ("mine", "true"),
                ("maxResults", "50"),
            ]);
        if let Some(page) = token.as_deref() {
            request = request.query(&[("pageToken", page)]);
        }
        let response: Value = request
            .send()
            .await
            .map_err(|e| format!("Could not load your YouTube playlists: {e}"))?
            .error_for_status()
            .map_err(|e| format!("YouTube could not load your playlists: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Could not read your YouTube playlists: {e}"))?;

        if let Some(items) = response.get("items").and_then(Value::as_array) {
            for item in items {
                let Some(id) = item.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let snippet = &item["snippet"];
                let image = ["maxres", "standard", "high", "medium", "default"]
                    .into_iter()
                    .find_map(|size| snippet["thumbnails"][size]["url"].as_str());
                playlists.push(json!({
                    "id": format!("VL{id}"),
                    "kind": "playlist",
                    "title": snippet["title"].as_str().unwrap_or("Untitled playlist"),
                    "subtitle": snippet["channelTitle"].as_str().unwrap_or("YouTube Music"),
                    "description": snippet["description"].as_str().unwrap_or(""),
                    "thumbnail": image,
                    "trackCount": item["contentDetails"]["itemCount"].as_u64().unwrap_or(0),
                }));
            }
        }
        token = response
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if token.is_none() {
            break;
        }
    }
    Ok(playlists)
}

pub async fn youtube_data_playlist(access_token: &str, playlist_id: &str) -> Result<Value, String> {
    let playlist_id = playlist_id.strip_prefix("VL").unwrap_or(playlist_id);
    let client = reqwest::Client::new();
    let detail: Value = client
        .get("https://www.googleapis.com/youtube/v3/playlists")
        .bearer_auth(access_token)
        .query(&[("part", "snippet,contentDetails"), ("id", playlist_id)])
        .send()
        .await
        .map_err(|e| format!("Could not load the playlist: {e}"))?
        .error_for_status()
        .map_err(|e| format!("YouTube could not load the playlist: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Could not read the playlist: {e}"))?;
    let playlist = detail["items"]
        .as_array()
        .and_then(|items| items.first())
        .ok_or_else(|| "YouTube did not return that playlist.".to_string())?;
    let snippet = &playlist["snippet"];
    let thumbnail = ["maxres", "standard", "high", "medium", "default"]
        .into_iter()
        .find_map(|size| snippet["thumbnails"][size]["url"].as_str());

    let mut page_token: Option<String> = None;
    let mut tracks = Vec::new();
    for _ in 0..100 {
        let mut request = client
            .get("https://www.googleapis.com/youtube/v3/playlistItems")
            .bearer_auth(access_token)
            .query(&[
                ("part", "snippet,contentDetails"),
                ("playlistId", playlist_id),
                ("maxResults", "50"),
            ]);
        if let Some(page) = page_token.as_deref() {
            request = request.query(&[("pageToken", page)]);
        }
        let response: Value = request
            .send()
            .await
            .map_err(|e| format!("Could not load playlist tracks: {e}"))?
            .error_for_status()
            .map_err(|e| format!("YouTube could not load playlist tracks: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Could not read playlist tracks: {e}"))?;
        if let Some(items) = response["items"].as_array() {
            for item in items {
                let Some(video_id) = item["contentDetails"]["videoId"].as_str() else {
                    continue;
                };
                let track = &item["snippet"];
                let image = ["high", "medium", "default"]
                    .into_iter()
                    .find_map(|size| track["thumbnails"][size]["url"].as_str())
                    .or(thumbnail);
                tracks.push(json!({
                    "videoId": video_id,
                    "title": track["title"].as_str().unwrap_or("Unavailable video"),
                    "artists": track["videoOwnerChannelTitle"].as_str().unwrap_or("YouTube"),
                    "thumbnail": image,
                    "duration": null,
                }));
            }
        }
        page_token = response
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if page_token.is_none() {
            break;
        }
    }

    Ok(json!({
        "title": snippet["title"].as_str().unwrap_or("Untitled playlist"),
        "subtitle": snippet["channelTitle"].as_str().unwrap_or("YouTube Music"),
        "thumbnail": thumbnail,
        "description": snippet["description"].as_str().unwrap_or(""),
        "items": tracks,
        "owned": true,
    }))
}

pub async fn revoke_token(token: &str) -> Result<(), String> {
    reqwest::Client::new()
        .post("https://oauth2.googleapis.com/revoke")
        .form(&[("token", token)])
        .send()
        .await
        .map_err(|e| format!("Could not revoke Google sign-in: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Google could not revoke the saved sign-in: {e}"))?;
    Ok(())
}

fn receive_oauth_callback(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    let (mut stream, _) = loop {
        match listener.accept() {
            Ok(connection) => break connection,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Err("Google sign-in timed out. Try again.".into());
            }
            Err(error) => {
                return Err(format!(
                    "Could not receive Google's sign-in response: {error}"
                ))
            }
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok();
    let mut request = [0_u8; 8192];
    let read = stream
        .read(&mut request)
        .map_err(|e| format!("Could not read Google's sign-in response: {e}"))?;
    let request = String::from_utf8_lossy(&request[..read]);
    let first_line = request
        .lines()
        .next()
        .ok_or("Google sent an empty sign-in response")?;
    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or("Google sent an invalid sign-in response")?;
    let query = path
        .split_once('?')
        .map(|(_, query)| query)
        .unwrap_or_default();
    let mut code = None;
    let mut returned_state = None;
    let mut oauth_error = None;
    let mut oauth_error_description = None;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let key =
            urlencoding::decode(key).map_err(|_| "Google sent an invalid sign-in response")?;
        let value =
            urlencoding::decode(value).map_err(|_| "Google sent an invalid sign-in response")?;
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => returned_state = Some(value.into_owned()),
            "error" => oauth_error = Some(value.into_owned()),
            "error_description" => oauth_error_description = Some(value.into_owned()),
            _ => {}
        }
    }
    if oauth_error.is_some() {
        respond_to_browser(&mut stream, false);
        return Err(oauth_error_description.unwrap_or_else(|| {
            "Google denied sign-in. Check the app's OAuth test-user settings and try again.".into()
        }));
    }
    if returned_state.as_deref() != Some(expected_state) {
        respond_to_browser(&mut stream, false);
        return Err("Google sign-in could not be verified. Please try again.".into());
    }
    let code = code.ok_or("Google did not return an authorization code")?;
    respond_to_browser(&mut stream, true);
    Ok(code)
}

fn respond_to_browser(stream: &mut TcpStream, success: bool) {
    let (status, title, message) = if success {
        (
            "200 OK",
            "Sign-in complete",
            "You can close this browser tab and return to NOVA Music.",
        )
    } else {
        (
            "400 Bad Request",
            "Sign-in not completed",
            "Return to NOVA Music and try signing in again.",
        )
    };
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title></head><body style=\"font-family:system-ui;margin:4rem auto;max-width:36rem;line-height:1.5\"><h1>{title}</h1><p>{message}</p></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

#[cfg(target_os = "macos")]
fn open_external_url(url: &str) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open your web browser: {e}"))
}

#[cfg(target_os = "windows")]
fn open_external_url(url: &str) -> Result<(), String> {
    std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open your web browser: {e}"))
}

#[cfg(target_os = "linux")]
fn open_external_url(url: &str) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Could not open your web browser: {e}"))
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub async fn switch_account(
    _account_id: String,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    Err(
        "Google account switching is not available yet. Sign out, then connect the other account."
            .into(),
    )
}
