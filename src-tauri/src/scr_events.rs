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

impl ScrProcessEventProvider {
    pub fn new(event_handler: Arc<Mutex<dyn FnMut(ScrEvent) + Send>>) -> ScrProcessEventProvider {
        ScrProcessEventProvider {
            _thread: std::thread::spawn(move || {
                let mut system = sysinfo::System::new();
                let mut last_emitted: Option<ScrEvent> = None;
                let mut down_streak: u32 = 0;

                loop {
                    let observed = find_starcraft_process(&mut system)
                        .and_then(|pid| find_starcraft_api_port(&pid))
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
