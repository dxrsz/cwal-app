#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
mod replay_parser;
mod scr_events;
mod scr_process;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicBool, Arc, Mutex};

use replay_parser::ReplayParser;
use scr_events::ScrProcessEventProvider;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tauri::State;
use tauri::Window;

use lru::LruCache;
use std::num::NonZeroUsize;

struct ReplayCache {
    dir: PathBuf,
    lru: Mutex<LruCache<String, PathBuf>>, // key: url, value: cached filepath
}

impl ReplayCache {
    fn new(dir: PathBuf, cap: usize) -> Self {
        if let Err(e) = fs::create_dir_all(&dir) {
            println!("[replay-cache] Failed to create cache dir {:?}: {}", dir, e);
        }
        Self {
            dir,
            lru: Mutex::new(LruCache::new(
                NonZeroUsize::new(cap).unwrap_or(NonZeroUsize::new(1000).unwrap()),
            )),
        }
    }

    fn key(url: &str) -> String {
        // Basic key; consider hashing if URLs are long
        url.to_string()
    }

    fn get(&self, url: &str) -> Option<PathBuf> {
        let key = Self::key(url);
        let mut lru = self.lru.lock().ok()?;
        let hit = lru.get(&key).cloned();
        if let Some(p) = hit {
            if p.exists() {
                println!("[replay-cache] HIT for {} -> {}", url, p.display());
                return Some(p.clone());
            } else {
                // stale entry
                println!("[replay-cache] STALE entry for {} -> {}", url, p.display());
                lru.pop(&key);
            }
        }
        println!("[replay-cache] MISS for {}", url);
        None
    }

    fn put(&self, url: &str, filename_hint: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        use std::io::Write;
        let sanitized = filename_hint
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
                _ => c,
            })
            .collect::<String>();
        let path = self.dir.join(sanitized);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create cache dir: {}", e))?;
        }
        let mut f =
            fs::File::create(&path).map_err(|e| format!("Failed to create cache file: {}", e))?;
        f.write_all(bytes)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;
        let key = Self::key(url);
        if let Ok(mut lru) = self.lru.lock() {
            lru.put(key, path.clone());
        }
        Ok(path)
    }
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl Drop for ReplayCache {
    fn drop(&mut self) {
        if let Ok(mut lru) = self.lru.lock() {
            for (_k, p) in lru.iter() {
                if p.exists() {
                    let _ = fs::remove_file(p);
                }
            }
            lru.clear();
        }
        println!("[replay-cache] Cache cleared on drop");
    }
}

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
    cache: State<'_, Arc<ReplayCache>>,
) -> Result<String, String> {
    use tauri_plugin_http::reqwest;

    let full_path = Path::new(&destination_path).join(&filename);
    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    if let Some(cached) = cache.get(&url) {
        println!(
            "[replay-cache] Using cached file for {} -> {}",
            url,
            cached.display()
        );
        fs::copy(&cached, &full_path).map_err(|e| format!("Failed to copy from cache: {}", e))?;
        return Ok(full_path.to_string_lossy().to_string());
    } else {
        println!("[replay-cache] No cache for {}, downloading", url);
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

    fs::write(&full_path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
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
    saved_path: String,
    duration_ms: u32,
    start_time_ms: u64,
    chat_messages: Vec<ParsedChatMessage>,
}

fn parse_replay_bytes(bytes: &[u8]) -> Result<(u32, u64, Vec<ParsedChatMessage>), String> {
    let parser = ReplayParser::new(bytes);
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
            timestamp_ms: m.frame_number * 42,
        })
        .collect();

    Ok((duration_ms, start_time_ms, chat_messages))
}

#[tauri::command]
async fn download_and_parse_replay(
    url: String,
    destination_path: String,
    filename: String,
    cache: State<'_, Arc<ReplayCache>>,
) -> Result<DownloadAndParseReplayResponse, String> {
    use tauri_plugin_http::reqwest;

    let full_path = Path::new(&destination_path).join(&filename);
    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    // Acquire bytes from cache or network
    let bytes: Vec<u8> = if let Some(cached) = cache.get(&url) {
        println!(
            "[replay-cache] Parse using cached file for {} -> {}",
            url,
            cached.display()
        );
        fs::read(&cached).map_err(|e| format!("Failed to read cached file: {}", e))?
    } else {
        println!("[replay-cache] No cache for {}, downloading for parse", url);
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
        let b = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        let vec = b.to_vec();
        let _ = cache.put(&url, &filename, &vec);
        vec
    };

    // Save to destination for visibility/use
    fs::write(&full_path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;

    let (duration_ms, start_time_ms, chat_messages) = parse_replay_bytes(&bytes)?;
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
            download_and_parse_replay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
