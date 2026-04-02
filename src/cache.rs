use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageCache {
    counts: HashMap<String, u32>,
}

impl Default for UsageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageCache {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    fn cache_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap()));
        let cache_dir = format!("{}/swift", config_dir);
        fs::create_dir_all(&cache_dir).ok();
        PathBuf::from(format!("{}/usage_cache.json", cache_dir))
    }

    pub fn load() -> Self {
        let path = Self::cache_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    pub fn save(&self) {
        let path = Self::cache_path();
        if let Ok(contents) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, contents);
        }
    }

    pub fn increment(&mut self, exec: &str) {
        let key = exec.split_whitespace().next().unwrap_or("").to_lowercase();
        if !key.is_empty() {
            *self.counts.entry(key).or_insert(0) += 1;
        }
    }

    pub fn get_priority(&self, exec: &str) -> u32 {
        let key = exec.split_whitespace().next().unwrap_or("").to_lowercase();
        self.counts.get(&key).copied().unwrap_or(0)
    }
}
