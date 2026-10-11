//! NOVA Music Tauri app - Lib

pub mod audioproxy;
pub mod auth;
pub mod commands;
pub mod db;
pub mod http;
pub mod orchestrator;
pub mod state;

// Re-export the workspace crates
pub use innertube;
pub use player;

use db::Db;
use state::AppState;
use tauri::Manager;

fn init_logging(dir: &std::path::Path) {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::Layer;

    let filter = || {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,app_lib=debug".into())
    };
    let path = dir.join("novamusic.log");
    let _ = std::fs::rename(&path, dir.join("novamusic.log.1"));
    let file = std::fs::File::create(&path).ok().map(std::sync::Mutex::new);
    let file_layer = file.map(|f| {
        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(f)
            .with_filter(filter())
    });
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter()))
        .with(file_layer)
        .init();
}

fn fatal(what: &str, detail: &str) -> ! {
    tracing::error!("fatal at startup: {what}: {detail}");
    std::process::exit(1)
}

pub fn run() {
    let builder = tauri::Builder::default();

    builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .with_filter(|label| label == "main")
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, _shortcut, _event| {})
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_home,
            commands::get_playlist,
            commands::get_album,
            commands::get_artist,
            commands::get_library,
            commands::get_history,
            commands::get_account_info,
            commands::search,
            commands::search_suggestions,
            commands::play_track,
            commands::download_current_track,
            commands::download_track,
            commands::enqueue_track,
            commands::dequeue_next_track,
            commands::pause_playback,
            commands::skip_track,
            commands::previous_track,
            commands::seek,
            commands::set_volume,
            commands::rate_track,
            commands::add_track_to_playlist,
            commands::get_queue,
            commands::get_now_playing,
            commands::get_lyrics,
            commands::get_playback_state,
            commands::show_song_notification,
            commands::get_settings,
            commands::update_settings,
            commands::start_google_sign_in,
            commands::sign_in,
            commands::sign_out,
            commands::switch_account,
        ])
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir());
            std::fs::create_dir_all(&data_dir).ok();
            init_logging(&data_dir);
            let cache_dir = data_dir.join("audio-cache");
            std::fs::create_dir_all(&cache_dir).ok();

            let _db = match Db::open(&data_dir.join("novamusic.sqlite")) {
                Ok(v) => v,
                Err(e) => fatal(
                    "NOVA Music could not open or create its database",
                    &format!("{}: {e}", data_dir.display()),
                ),
            };

            // Start loopback HTTP audio proxy for high-speed gapless streaming
            audioproxy::start();

            // Initialize AppState
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            let app_state = rt
                .block_on(async { AppState::new(&data_dir).await })
                .expect("Failed to initialize AppState");
            app_state.start_playback_events(app.handle().clone());
            app.manage(app_state);

            println!("[NOVA] AppState initialized, configuring windows...");
            for (label, w) in app.webview_windows() {
                println!(
                    "[NOVA] Window '{}': is_visible={:?}, position={:?}, size={:?}",
                    label,
                    w.is_visible(),
                    w.outer_position(),
                    w.outer_size()
                );
                let _ = w.set_size(tauri::LogicalSize::new(1200.0, 800.0));
                let _ = w.center();
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }

            #[cfg(target_os = "macos")]
            unsafe {
                use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
                let ns_app = NSApp();
                ns_app.setActivationPolicy_(
                    NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular,
                );
                ns_app.activateIgnoringOtherApps_(true);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
