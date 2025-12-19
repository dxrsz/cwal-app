use lru::LruCache;
use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Mutex,
};

pub struct ReplayCache {
    dir: PathBuf,
    lru: Mutex<LruCache<String, PathBuf>>, // key: url, value: cached filepath
}

impl ReplayCache {
    pub fn new(dir: PathBuf, cap: usize) -> Self {
        if let Err(e) = fs::create_dir_all(&dir) {
            println!("[replay-cache] Failed to create cache dir {dir:?}: {e}");
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

    fn hashed_path_for(&self, url: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let h = hasher.finish();
        // keep extension generic; consumers copy with their own filename
        self.dir.join(format!("{h:016x}.rep"))
    }

    pub fn get(&self, url: &str) -> Option<PathBuf> {
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
        println!("[replay-cache] MISS for {url}");
        None
    }

    pub fn put(&self, url: &str, _filename_hint: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        use std::io::Write;
        let path = self.hashed_path_for(url);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create cache dir: {e}"))?;
        }
        let mut f =
            fs::File::create(&path).map_err(|e| format!("Failed to create cache file: {e}"))?;
        f.write_all(bytes)
            .map_err(|e| format!("Failed to write cache file: {e}"))?;
        let key = Self::key(url);
        if let Ok(mut lru) = self.lru.lock() {
            lru.put(key, path.clone());
        }
        Ok(path)
    }
}

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
