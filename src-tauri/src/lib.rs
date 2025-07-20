#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
mod scr_events;
mod scr_process;

use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::fs;
use std::path::Path;

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

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            init_process,
            read_settings_file,
            write_settings_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
