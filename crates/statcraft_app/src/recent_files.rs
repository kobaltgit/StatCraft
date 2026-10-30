use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RecentFiles {
    pub paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub const MAX_ENTRIES: usize = 10;
    pub const STORAGE_KEY: &'static str = "statcraft_recent_files";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, path: PathBuf) {
        self.paths.retain(|p| p != &path);
        self.paths.insert(0, path);
        if self.paths.len() > Self::MAX_ENTRIES {
            self.paths.truncate(Self::MAX_ENTRIES);
        }
    }

    pub fn clear(&mut self) {
        self.paths.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn storage_file_path() -> PathBuf {
        PathBuf::from(".statcraft").join("recent_files.json")
    }

    pub fn load_default() -> Self {
        let path = Self::storage_file_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(rf) = serde_json::from_str::<RecentFiles>(&content) {
                    return rf;
                }
            }
        }
        Self::default()
    }

    pub fn save_default(&self) {
        let path = Self::storage_file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json_str) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json_str);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_files_add_and_deduplicate() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("a.csv"));
        rf.add(PathBuf::from("b.csv"));
        rf.add(PathBuf::from("a.csv"));

        assert_eq!(rf.len(), 2);
        assert_eq!(rf.paths[0], PathBuf::from("a.csv"));
        assert_eq!(rf.paths[1], PathBuf::from("b.csv"));
    }

    #[test]
    fn test_recent_files_max_limit_truncation() {
        let mut rf = RecentFiles::new();
        for i in 0..15 {
            rf.add(PathBuf::from(format!("file_{}.csv", i)));
        }

        assert_eq!(rf.len(), RecentFiles::MAX_ENTRIES);
        assert_eq!(rf.paths[0], PathBuf::from("file_14.csv"));
        assert_eq!(rf.paths[9], PathBuf::from("file_5.csv"));
    }

    #[test]
    fn test_recent_files_clear() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("file.csv"));
        assert!(!rf.is_empty());
        rf.clear();
        assert!(rf.is_empty());
    }

    #[test]
    fn test_recent_files_serialization() {
        let mut rf = RecentFiles::new();
        rf.add(PathBuf::from("sample.csv"));
        let json_str = serde_json::to_string(&rf).unwrap();
        let decoded: RecentFiles = serde_json::from_str(&json_str).unwrap();
        assert_eq!(rf, decoded);
    }
}
