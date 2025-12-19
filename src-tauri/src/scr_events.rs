use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::scr_process::{find_starcraft_api_port, find_starcraft_process};

pub struct ScrProcessEventProvider {
    _thread: std::thread::JoinHandle<()>,
}

impl ScrProcessEventProvider {
    pub fn new(event_handler: Arc<Mutex<dyn FnMut(ScrEvent) + Send>>) -> ScrProcessEventProvider {
        ScrProcessEventProvider {
            _thread: std::thread::spawn(move || loop {
                let mut system = sysinfo::System::new();
                system.refresh_all();

                std::thread::sleep(std::time::Duration::from_secs(1));

                let pid = find_starcraft_process(&mut system);

                if let Some(pid) = pid {
                    if let Some(port) = find_starcraft_api_port(&pid) {
                        event_handler.lock().unwrap()(ScrEvent::WebServerRunning { port });
                        continue;
                    }
                }

                event_handler.lock().unwrap()(ScrEvent::WebServerDown);
            }),
        }
    }
}

/// SCR events from multiple sources.
#[derive(Debug, Serialize, Clone)]
pub enum ScrEvent {
    WebServerRunning { port: u16 },
    WebServerDown,
}
