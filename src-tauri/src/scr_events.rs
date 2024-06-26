use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use serde::Serialize;
use url::Url;

use crate::{
    scr_process::{find_starcraft_api_port, find_starcraft_process},
    star_cache::StarCache,
};

static MAXIMUM_HELD_ENTRIES: usize = 30;

#[derive(Debug, PartialEq, Clone)]
pub enum ScrNetworkRequest {
    GameLoadingProfile { alias: String, gateway: u8 },
    Tooninfo { alias: String, gateway: u8 },
    LeaderboardByToon { alias: String, gateway: u8 },
    GameLoading { url: String },
    ChatPanel { url: String },
    MapPreview { hash: String },
    Unparsable { url: String },
    ToastPanel { url: String },
    Unknown { url: String },
}

/// Returns a `ScrNetworkRequest` from a given URL, which gives an id to the url as well as parses
/// relevant information from it.
impl From<String> for ScrNetworkRequest {
    fn from(url: String) -> Self {
        // Parsing URL parts
        let url_parsed = Url::parse(&url);

        return match url_parsed {
            Ok(url_parsed) => {
                let segments = url_parsed.path_segments().unwrap().collect::<Vec<&str>>();

                if url.contains("web-api/v2/aurora-profile-by-toon")
                    && url.contains("request_flags=scr_tooninfo")
                {
                    let alias = segments[3];
                    let gateway = segments[4].parse::<u8>();
                    match gateway {
                        Ok(gateway) => ScrNetworkRequest::Tooninfo {
                            alias: String::from(alias),
                            gateway,
                        },
                        Err(_) => ScrNetworkRequest::Unknown { url },
                    }
                } else if url.contains("web-api/v2/aurora-profile-by-toon")
                    && url.contains("request_flags=scr_mmgameloading")
                {
                    let alias = segments[3];
                    let gateway = segments[4].parse::<u8>();
                    match gateway {
                        Ok(gateway) => ScrNetworkRequest::GameLoadingProfile {
                            alias: String::from(alias),
                            gateway,
                        },
                        Err(_) => ScrNetworkRequest::Unknown { url },
                    }
                } else if url.contains("web-api/v1/leaderboard-rank-by-toon") {
                    let alias = segments[4];
                    let gateway = segments[5].parse::<u8>();
                    match gateway {
                        Ok(gateway) => ScrNetworkRequest::LeaderboardByToon {
                            alias: String::from(alias),
                            gateway,
                        },
                        Err(_) => ScrNetworkRequest::Unknown { url },
                    }
                } else if url.contains("MatchmakingGameLoadingPanel/MatchmakingGameLoadingPanel") {
                    ScrNetworkRequest::GameLoading { url }
                } else if url.contains("ChatPanel/ChatPanel.html") {
                    ScrNetworkRequest::ChatPanel { url }
                } else if url.contains("ToastPanel/ToastPanel.html") {
                    ScrNetworkRequest::ToastPanel { url }
                } else if url.contains("map-preview.bmp") {
                    ScrNetworkRequest::MapPreview {
                        hash: url_parsed
                            .query_pairs()
                            .find(|(k, _)| k == "hash")
                            .unwrap()
                            .1
                            .to_string(),
                    }
                } else {
                    ScrNetworkRequest::Unknown { url }
                }
            }
            Err(_) => ScrNetworkRequest::Unparsable { url },
        };
    }
}

pub struct ScrProcessEventProvider {
    _thread: std::thread::JoinHandle<()>,
}

impl ScrProcessEventProvider {
    fn new(event_handler: Arc<Mutex<dyn FnMut(ScrEvent) + Send>>) -> ScrProcessEventProvider {
        ScrProcessEventProvider {
            _thread: std::thread::spawn(move || loop {
                let mut system = sysinfo::System::new();
                system.refresh_all();

                std::thread::sleep(std::time::Duration::from_secs(1));

                let pid = find_starcraft_process(&mut system);

                if let Some(pid) = pid {
                    if let Some(port) = find_starcraft_api_port(&pid) {
                        event_handler.lock().unwrap()(ScrEvent::WebServerRunning { port: port });
                        continue;
                    }
                }

                event_handler.lock().unwrap()(ScrEvent::WebServerDown);
            }),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Clone)]
pub struct Player {
    alias: String,
    gateway: u8,
}

/// SCR events from multiple sources.
#[derive(Debug, Serialize, Clone)]
pub enum ScrEvent {
    // tooninfo -> leaderboardbytoon -> chatpanel
    ProfileSelect {
        alias: String,
        gateway: u8,
    },
    MatchFound {
        player1: Player,
        player2: Player,
        map: String,
    },
    GameEnded,
    WebServerRunning {
        port: u16,
    },
    WebServerDown
}

/// Watches the SCR network cache and invokes the provided callback to notify
/// when those events have indicated an SCR event. For example, the cache might indicate
/// url fetches for two player profiles and a map preview, which would be used to
/// derive a `MatchFound` event.
pub struct ScrNetworkEventsReceiver {
    _thread: std::thread::JoinHandle<()>,
}

impl ScrNetworkEventsReceiver {
    fn new(event_handler: Arc<Mutex<dyn FnMut(ScrEvent) + Send>>) -> Self {
        let thread = std::thread::spawn(move || {
            let mut events = VecDeque::new();

            let _network_requests_receiver = StarCache::new(Mutex::new(Box::new(move |entry| {
                let request = ScrNetworkRequest::from(entry);

                let mut derived_event = None;

                match request {
                    ScrNetworkRequest::GameLoadingProfile { ref alias, gateway } => {
                        let map = events.iter().rev().take(30).find(|event| match event {
                            ScrNetworkRequest::MapPreview { .. } => true,
                            _ => false,
                        });

                        let other_player = events.iter().rev().take(30).find(|event| match event {
                            ScrNetworkRequest::GameLoadingProfile {
                                alias: other_alias,
                                gateway: other_gateway,
                            } => other_alias != alias || other_gateway != &gateway,
                            _ => false,
                        });

                        if let Some(ScrNetworkRequest::MapPreview { hash }) = map {
                            if let Some(ScrNetworkRequest::GameLoadingProfile {
                                alias: other_alias,
                                gateway: other_gateway,
                            }) = other_player
                            {
                                derived_event = Some(ScrEvent::MatchFound {
                                    player1: Player {
                                        alias: alias.clone(),
                                        gateway: gateway,
                                    },
                                    player2: Player {
                                        alias: other_alias.clone(),
                                        gateway: *other_gateway,
                                    },
                                    map: hash.clone(),
                                });
                            }
                        }
                    }
                    ScrNetworkRequest::ChatPanel { .. } => {
                        // try to find the most recent preceding leaderboard event by toon event, it should
                        // be in the most recent 5 or so events
                        let leaderboard_event =
                            events.iter().rev().take(5).find(|event| match event {
                                ScrNetworkRequest::LeaderboardByToon { .. } => true,
                                _ => false,
                            });

                        // if we found a leaderboard event, we can derive a profile select event
                        if let Some(leaderboard_event) = leaderboard_event {
                            match leaderboard_event {
                                ScrNetworkRequest::LeaderboardByToon { alias, gateway } => {
                                    derived_event = Some(ScrEvent::ProfileSelect {
                                        alias: alias.clone(),
                                        gateway: *gateway,
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                    ScrNetworkRequest::ToastPanel { .. } => {
                        derived_event = Some(ScrEvent::GameEnded);
                    }
                    _ => {}
                }

                if let Some(received) = derived_event {
                    event_handler.lock().unwrap()(received);
                    events.clear();
                }

                events.push_back(request);

                if events.len() > MAXIMUM_HELD_ENTRIES {
                    events.pop_front();
                }
            })));
        });

        ScrNetworkEventsReceiver { _thread: thread }
    }
}

/// A provider of SCR events, which can be either from a process or network source.
#[allow(dead_code)] // just used to aggregate
enum ScrEventProvider {
    ScrProcessEventProvider(ScrProcessEventProvider),
    ScrNetworkEventsReceiver(ScrNetworkEventsReceiver),
}

/// An aggregate provider of SCR events, which can be used to listen to multiple sources of
/// SCR events.
pub struct AggregateScrEventsProvider {
    _providers: Vec<ScrEventProvider>,
}

impl AggregateScrEventsProvider {
    pub fn new(
        event_handler: Arc<Mutex<dyn FnMut(ScrEvent) + Send>>,
    ) -> AggregateScrEventsProvider {
        let process_provider = ScrProcessEventProvider::new(event_handler.clone());
        let network_provider = ScrNetworkEventsReceiver::new(event_handler.clone());

        AggregateScrEventsProvider {
            _providers: vec![
                ScrEventProvider::ScrProcessEventProvider(process_provider),
                ScrEventProvider::ScrNetworkEventsReceiver(network_provider),
            ],
        }
    }
}
