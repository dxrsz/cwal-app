#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
mod cache;
mod replay_parser;
mod scr_events;
mod scr_process;

use std::fs;
use std::path::Path;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

use cache::ReplayCache;
use replay_parser::ReplayParser;
use scr_events::ScrProcessEventProvider;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tauri::State;
use tauri::Window;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn init_process(window: Window) {
    if INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);

    std::thread::spawn(move || {
        let window = Mutex::new(Arc::new(window));
        let mut _listen = ScrProcessEventProvider::new(Arc::new(Mutex::new(move |event| {
            println!("event: {event:?}");
            let window = window.lock().unwrap();
            window.emit("scr-event", event).unwrap();
        })));
    });
}

#[tauri::command]
fn read_settings_file(path: String) -> Result<String, String> {
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(String::new())
            } else {
                Err(format!("Failed to read settings file: {e}"))
            }
        }
    }
}

#[tauri::command]
fn write_settings_file(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {e}"));
        }
    }

    match fs::write(&path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write settings file: {e}")),
    }
}

#[tauri::command]
async fn download_file(
    url: String,
    destination_path: String,
    filename: String,
    cache: State<'_, Arc<ReplayCache>>,
) -> Result<String, String> {
    use tauri_plugin_http::reqwest;

    let full_path = Path::new(&destination_path).join(&filename);
    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {e}"));
        }
    }

    if let Some(cached) = cache.get(&url) {
        println!(
            "[replay-cache] Using cached file for {} -> {}",
            url,
            cached.display()
        );
        fs::copy(&cached, &full_path).map_err(|e| format!("Failed to copy from cache: {e}"))?;
        return Ok(full_path.to_string_lossy().to_string());
    } else {
        println!("[replay-cache] No cache for {url}, downloading");
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    fs::write(&full_path, &bytes).map_err(|e| format!("Failed to write file: {e}"))?;
    let _ = cache.put(&url, &filename, &bytes);
    Ok(full_path.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
struct ParsedChatMessage {
    sender_name: String,
    message: String,
    frame_number: u32,
    sender_id: u8,
    timestamp_ms: u32,
}

#[derive(serde::Serialize)]
struct DownloadAndParseReplayResponse {
    duration_ms: u32,
    start_time_ms: u64,
    chat_messages: Vec<ParsedChatMessage>,
    cached: bool,
}

fn parse_replay_bytes(bytes: &[u8]) -> Result<(u32, u64, Vec<ParsedChatMessage>), String> {
    let parser = ReplayParser::new(bytes);
    let parsed = parser
        .parse()
        .map_err(|e| format!("Failed to parse replay: {e:?}"))?;

    let duration_ms = parsed.duration_ms();
    let start_time_ms = parsed
        .game_info
        .start_time
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();

    let chat_messages = parsed
        .chat_messages()
        .into_iter()
        .map(|m| ParsedChatMessage {
            sender_name: m.sender_name,
            message: m.message,
            frame_number: m.frame_number,
            sender_id: m.sender_id,
            timestamp_ms: m.frame_number * 42,
        })
        .collect();

    Ok((duration_ms, start_time_ms, chat_messages))
}

#[tauri::command]
async fn download_and_parse_replay(
    url: String,
    filename: String,
    cache: State<'_, Arc<ReplayCache>>,
) -> Result<DownloadAndParseReplayResponse, String> {
    use tauri_plugin_http::reqwest;

    // Acquire bytes from cache or network
    let cached_path = cache.get(&url);
    let cached = cached_path.is_some();
    let bytes: Vec<u8> = if let Some(ref cached_path) = cached_path {
        println!(
            "[replay-cache] Parse using cached file for {} -> {}",
            url,
            cached_path.display()
        );
        fs::read(cached_path).map_err(|e| format!("Failed to read cached file: {e}"))?
    } else {
        println!("[replay-cache] No cache for {url}, downloading for parse");
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to download file: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }
        let b = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;
        let vec = b.to_vec();
        let _ = cache.put(&url, &filename, &vec);
        vec
    };

    let (duration_ms, start_time_ms, chat_messages) = parse_replay_bytes(&bytes)?;
    Ok(DownloadAndParseReplayResponse {
        duration_ms,
        start_time_ms,
        chat_messages,
        cached,
    })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Establish cache in AppData/cwal-app/replay-cache
            let app_handle = app.handle();
            let cache_dir = app_handle
                .path()
                .resolve("replay-cache", BaseDirectory::AppData)
                .unwrap_or_else(|_| {
                    let mut p = std::env::temp_dir();
                    p.push("cwal-app-replay-cache");
                    p
                });
            let cache = Arc::new(ReplayCache::new(cache_dir, 1000));
            app.manage(cache);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_process,
            read_settings_file,
            write_settings_file,
            download_file,
            download_and_parse_replay,
            reveal_in_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn reveal_in_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    showfile::show_path_in_file_manager(p);
    Ok(())
}
