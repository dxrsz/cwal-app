use std::{
    hash::{DefaultHasher, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrome_cache_parser::{block_file::LazyBlockFileCacheEntry, CCPError, CCPResult, ChromeCache};
use chrono::{DateTime, Local, TimeDelta};
use notify::{Config, Event, Watcher};

use crate::scr_events::ScrNetworkRequest;

#[derive(Debug)]
struct CacheEntries {
    entries: Vec<CacheEntry>,
    hash: u64,
}

#[derive(Debug)]
struct CacheEntry {
    url: String,
    last_used: DateTime<Local>,
}

pub struct StarCache {
    _watcher: notify::PollWatcher,
}

const DEBOUNCE_CACHE_READ_TIME: TimeDelta = chrono::Duration::milliseconds(50);

/// Watches the StarCraft cache directory and invokes the provided event handler with the url
/// accesed by the client, as indicated by the new cache entries.
impl StarCache {
    fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap()
            .join("AppData/Local/Temp/blizzard_browser_cache")
    }

    pub fn new(event_handler: Mutex<Box<dyn FnMut(ScrNetworkRequest) + Send>>) -> Self {
        let mut watcher = Self::create_watcher(StarCache::default_path(), event_handler);
        let path = StarCache::default_path();

        // These are the cache entry files.
        for file in ["index", "data_0", "data_1", "data_2", "data_3"].iter() {
            watcher
                .watch(
                    &Path::new(&path).join(file),
                    notify::RecursiveMode::NonRecursive,
                )
                .unwrap();
        }

        StarCache { _watcher: watcher }
    }

    fn create_watcher(
        path: PathBuf,
        event_handler: Mutex<Box<dyn FnMut(ScrNetworkRequest) + Send>>,
    ) -> notify::PollWatcher {
        let last_entries_time = Arc::new(Mutex::new(Local::now()));
        let cache_path = path.clone();

        notify::PollWatcher::new(
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if !event
                        .paths
                        .iter()
                        .map(|p| p.file_name().unwrap())
                        .any(|p| p == "index" || p.to_str().unwrap().starts_with("data_"))
                    {
                        return;
                    }
                    let mut new_entries = Vec::new();
                    let mut newest_entry_time: Option<chrono::DateTime<chrono::Local>> = None;
                    let mut last_entries_time = last_entries_time.lock().unwrap();

                    let mut previous_hash: Option<u64> = None;

                    // todo maximum retries
                    let mut entries = loop {
                        let cache = ChromeCache::from_path((&cache_path).clone()).unwrap();

                        let entries = cache.entries();
                        let first_attempt_read_time = chrono::Local::now();
                        match Self::collect_entries(entries) {
                            Ok(entries) => {
                                let elapsed = chrono::Local::now() - first_attempt_read_time;
                                let remaining = DEBOUNCE_CACHE_READ_TIME - elapsed;
                                if remaining > chrono::Duration::zero() {
                                    std::thread::sleep(remaining.to_std().unwrap());
                                }
                                let CacheEntries { entries, hash } = entries;

                                if let Some(previous_hash) = previous_hash {
                                    if previous_hash == hash {
                                        break entries;
                                    }
                                }

                                previous_hash = Some(hash);
                            }
                            Err(_) => {}
                        }
                    };

                    entries.sort_by(|a, b| a.last_used.cmp(&b.last_used));

                    for entry in entries {
                        if entry.last_used > *last_entries_time {
                            //println!("entry: {:?}", &entry);
                            new_entries.push(entry.url);

                            newest_entry_time = Some(entry.last_used);
                        }
                    }

                    if let Some(t) = newest_entry_time {
                        *last_entries_time = t;
                    }

                    for entry in new_entries {
                        event_handler.lock().unwrap()(ScrNetworkRequest::from(entry));
                    }
                }
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(50)),
        )
        .unwrap()
    }

    /// Collects cache entries from the given iterator into a `CacheEntries` struct, which
    /// contains the entries and a hash of the entries. We use the hash to validate that
    /// the entries are the same as the last time we collected them (a bit of a hack to mitigate
    /// reading the cache while it's being written to and receiving corrupted data).
    fn collect_entries(
        entries: CCPResult<impl Iterator<Item = LazyBlockFileCacheEntry>>,
    ) -> CCPResult<CacheEntries> {
        // todo: new result type
        let entries: Vec<CacheEntry> = entries?
            .map(|mut entry| {
                let last_used: DateTime<Local> = entry
                    .get_rankings_node()?
                    .get()?
                    .last_used
                    .into_datetime_local()?;
                let entry = entry.get()?;

                Ok::<CacheEntry, CCPError>(CacheEntry {
                    url: entry.key.to_string(),
                    last_used,
                })
            })
            .collect::<CCPResult<Vec<CacheEntry>>>()?;

        let hash = entries
            .iter()
            .fold(DefaultHasher::new(), |mut hasher, e| {
                hasher.write(e.url.as_bytes());
                hasher.write_i64(e.last_used.timestamp());
                hasher
            })
            .finish();

        return Ok(CacheEntries { entries, hash });
    }
}
