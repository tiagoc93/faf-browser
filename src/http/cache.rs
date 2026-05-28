use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub url: String,
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub cached_at: u64, // unix timestamp
}

/// SHA256 hash of URL to use as cache key
pub fn cache_key(url: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(url.as_bytes());
    hex::encode(hash)
}

/// Full path to cache file
pub fn cache_path(cache_dir: &str, key: &str) -> PathBuf {
    PathBuf::from(cache_dir).join(format!("{}.json", key))
}

/// Get cached response if it exists and hasn't expired
pub fn get_cached(cache_dir: &str, url: &str, ttl: Duration) -> Option<CacheEntry> {
    let key = cache_key(url);
    let path = cache_path(cache_dir, &key);
    if !path.exists() {
        return None;
    }

    let entry: CacheEntry = serde_json::from_reader(std::fs::File::open(&path).ok()?).ok()?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if entry.cached_at + ttl.as_secs() < now {
        // Expired - remove file
        let _ = std::fs::remove_file(&path);
        return None;
    }

    Some(entry)
}

/// Save response to cache
pub fn set_cached(cache_dir: &str, url: &str, entry: &CacheEntry) -> anyhow::Result<()> {
    let key = cache_key(url);
    let path = cache_path(cache_dir, &key);

    // Create cache directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(&path)?;
    serde_json::to_writer(file, entry)?;
    Ok(())
}
