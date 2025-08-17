#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
mod replay_parser;
mod scr_events;
mod scr_process;

use std::fs;
use std::path::Path;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

use replay_parser::ReplayParser;
use scr_events::ScrProcessEventProvider;
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
            println!("event: {:?}", event);
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
                Err(format!("Failed to read settings file: {}", e))
            }
        }
    }
}

#[tauri::command]
fn write_settings_file(path: String, content: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    match fs::write(&path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write settings file: {}", e)),
    }
}

#[tauri::command]
async fn download_file(
    url: String,
    destination_path: String,
    filename: String,
) -> Result<String, String> {
    use std::io::Write;
    use tauri_plugin_http::reqwest;

    let full_path = Path::new(&destination_path).join(&filename);

    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let mut file =
        fs::File::create(&full_path).map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

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
    saved_path: String,
    duration_ms: u32,
    start_time_ms: u64,
    chat_messages: Vec<ParsedChatMessage>,
}

#[tauri::command]
async fn download_and_parse_replay(
    url: String,
    destination_path: String,
    filename: String,
) -> Result<DownloadAndParseReplayResponse, String> {
    use std::io::Write;
    use tauri_plugin_http::reqwest;

    let full_path = Path::new(&destination_path).join(&filename);

    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Save file to disk
    //{
    //    let mut file =
    //        fs::File::create(&full_path).map_err(|e| format!("Failed to create file: {}", e))?;
    //    file.write_all(&bytes)
    //        .map_err(|e| format!("Failed to write file: {}", e))?;
    //}

    // Parse replay from in-memory bytes
    let parser = ReplayParser::new(&bytes);
    let parsed = parser
        .parse()
        .map_err(|e| format!("Failed to parse replay: {:?}", e))?;

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
            timestamp_ms: m.frame_number * 42, // 42 ms per frame
        })
        .collect();

    Ok(DownloadAndParseReplayResponse {
        saved_path: full_path.to_string_lossy().to_string(),
        duration_ms,
        start_time_ms,
        chat_messages,
    })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            init_process,
            read_settings_file,
            write_settings_file,
            download_file,
            download_and_parse_replay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
