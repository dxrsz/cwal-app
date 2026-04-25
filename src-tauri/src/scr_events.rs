use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::scr_process::{find_starcraft_api_port, find_starcraft_process};

pub struct ScrProcessEventProvider {
    _thread: std::thread::JoinHandle<()>,
}

// Number of consecutive "process or port not found" polls required before we
// flip the UI to WebServerDown. netstat's GetExtendedTcpTable on Windows can
// transiently fail or miss the SC listening socket for a single snapshot;
// without tolerance the indicator flaps every time that happens.
const DOWN_TOLERANCE: u32 = 3;

// Cap the diagnostic log so it doesn't grow unbounded. We rotate by truncating
// when it exceeds this size. ~64 KB is plenty to capture several startup
// sequences for diagnosis.
const LOG_MAX_BYTES: u64 = 64 * 1024;

fn log_path() -> Option<PathBuf> {
    // %APPDATA%\gg.cwal.app\scr-debug.log — same dir Tauri uses for AppData.
    let appdata = std::env::var_os("APPDATA")?;
    let mut p = PathBuf::from(appdata);
    p.push("gg.cwal.app");
    let _ = std::fs::create_dir_all(&p);
    p.push("scr-debug.log");
    Some(p)
}

fn append_log(line: &str) {
    let Some(path) = log_path() else {
        return;
    };
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > LOG_MAX_BYTES {
            let _ = std::fs::write(&path, b"");
        }
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(f, "[{now}] {line}");
    }
}

impl ScrProcessEventProvider {
    pub fn new(event_handler: Arc<Mutex<dyn FnMut(ScrEvent) + Send>>) -> ScrProcessEventProvider {
        ScrProcessEventProvider {
            _thread: std::thread::spawn(move || {
                let mut last_emitted: Option<ScrEvent> = None;
                let mut down_streak: u32 = 0;
                append_log("scr poller started");

                loop {
                    // Re-create System every iteration. Reusing it across
                    // iterations was equivalent in dev but caused detection
                    // to silently fail in release builds on some machines —
                    // probably stale internal state in sysinfo's Windows
                    // process snapshot path. Cheap to re-create at 1 Hz.
                    let mut system = sysinfo::System::new();
                    let pid = find_starcraft_process(&mut system);
                    let observed = pid
                        .as_ref()
                        .and_then(find_starcraft_api_port)
                        .map(|port| ScrEvent::WebServerRunning { port });

                    let to_emit = match observed {
                        Some(ev) => {
                            down_streak = 0;
                            Some(ev)
                        }
                        None => {
                            down_streak = down_streak.saturating_add(1);
                            if down_streak >= DOWN_TOLERANCE {
                                Some(ScrEvent::WebServerDown)
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(ev) = to_emit {
                        if last_emitted.as_ref() != Some(&ev) {
                            append_log(&format!(
                                "transition pid={:?} event={:?}",
                                pid, ev
                            ));
                            event_handler.lock().unwrap()(ev.clone());
                            last_emitted = Some(ev);
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }),
        }
    }
}

/// SCR events from multiple sources.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum ScrEvent {
    WebServerRunning { port: u16 },
    WebServerDown,
}
