// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod scr_events;
mod scr_process;
mod star_cache;

use std::sync::{atomic::AtomicBool, Arc, Mutex};

use rouille::proxy::full_proxy;
use rouille::proxy::ProxyConfig;
use rouille::Server;
use scr_events::ScrEvent;
use tauri::Window;

use crate::scr_events::AggregateScrEventsProvider;

// Esnure that the background process is only initialized once.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

struct StoppableServer {
    handle: std::thread::JoinHandle<()>,
    sender: std::sync::mpsc::Sender<()>,
    port: u16,
}

// Initialize the background processes that notify the frontend about SC:R events.
// Additionally, we create a proxy server 
#[tauri::command]
fn init_process(window: Window) {
    if INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);

    let mut server: Option<StoppableServer> = None;

    std::thread::spawn(move || {
        let window = Mutex::new(Arc::new(window));

        let mut _listen =
            AggregateScrEventsProvider::new(Arc::new(Mutex::new(move |event: ScrEvent| {
                let window = window.lock().unwrap();

                match &event {
                    ScrEvent::WebServerRunning { port } => {
                        let port = port.clone();


                        if let Some(StoppableServer {
                            port: open_port,
                            ..
                        }) = server {
                            if open_port == port {
                                return // server already open on correct port
                            }
                        }

                        // kill our old proxy if it exists
                        if let Some(StoppableServer {
                            sender,
                            handle,
                            ..
                        }) = server.take() {
                            sender.send(()).unwrap();
                            handle.join().unwrap();
                        }

                        // create new proxy
                        let proxy = Server::new("localhost:57421", move |req| {
                            full_proxy(
                                &req,
                                ProxyConfig {
                                    addr: format!("localhost:{}", port),
                                    replace_host: None,
                                },
                            )
                            .expect("could not forward to proxy")
                        })
                        .expect("Failed to start server");

                        let (handle, tx) = proxy.stoppable();
                        server = Some(StoppableServer {
                            handle,
                            sender: tx,
                            port,
                        });
                    }
                    _ => {} // ignore other events for now
                }

                window.emit("scr-event", event.clone()).unwrap();
            })));
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![init_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
