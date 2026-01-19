use bindings::exports::swift::launcher::runner::{ActionItem, Guest};
mod bindings;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::fs;
use std::path::{Path, PathBuf};

struct DirectoryScanner;

impl Guest for DirectoryScanner {
    fn get_trigger() -> String {
        "/".to_string()
    }

    fn handle(input: String) -> Vec<ActionItem> {
        // Trigger as long as we are in "path mode"
        if !input.starts_with('/') {
            return vec![];
        }

        let matcher = SkimMatcherV2::default();

        // Find the last slash to separate the directory from the search pattern
        // Example: "/usr/lo" -> base: "/usr/", pattern: "lo"
        let last_slash_idx = input.rfind('/').unwrap_or(0);
        let (base_str, pattern) = input.split_at(last_slash_idx + 1);

        // Ensure base_str is a valid path; default to "/" if empty or invalid
        let base_path = if base_str.is_empty() {
            PathBuf::from("/")
        } else {
            PathBuf::from(base_str)
        };

        let mut results = Vec::new();

        // Only scan if the base part is actually a directory
        if base_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&base_path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().into_owned();
                    let full_path = entry.path();
                    let full_path_str = full_path.to_string_lossy().to_string();

                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let display_name = full_path_str.clone();

                    // If pattern is empty (e.g. user just typed "/"), show all files
                    // Otherwise, fuzzy match the filename against the pattern
                    if pattern.is_empty() {
                        results.push((
                            1,
                            ActionItem {
                                name: display_name,
                                exec: format!("echo -n '{}' | wl-copy", full_path_str),
                                keywords: "/".into(),
                            },
                        ));
                    } else if let Some(score) = matcher.fuzzy_match(&file_name, pattern) {
                        results.push((
                            score,
                            ActionItem {
                                name: display_name,
                                exec: format!("echo -n '{}' | wl-copy", full_path_str),
                                keywords: "/".into(),
                            },
                        ));
                    }
                }
            }
        }

        // Sort by fuzzy score descending, then alphabetically
        results.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));

        results.into_iter().map(|(_, item)| item).collect()
    }
}

bindings::export!(DirectoryScanner with_types_in bindings);
