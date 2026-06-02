use ini::Ini;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

static RE_EXEC: OnceLock<Regex> = OnceLock::new();
static RE_XDG: OnceLock<Regex> = OnceLock::new();

pub fn get_programs_raw_streaming<F>(on_item: F)
where
    F: FnMut((String, String, String)) + Send + Sync + 'static,
{
    let on_item = Arc::new(Mutex::new(on_item));
    let seen = Arc::new(Mutex::new(HashSet::<String>::new()));
    let home = env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let xdg_data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home));
    
    let default_dirs = [
        format!("{}/flatpak/exports/share", xdg_data_home),
        xdg_data_home,
        "/var/lib/flatpak/exports/share".to_string(),
        "/usr/local/share".to_string(),
        "/usr/share".to_string(),
        "/usr/share/gnome".to_string(),
        "/usr/share/plasma".to_string(),
        "/usr/share/kde5".to_string(),
        "/usr/share/kde6".to_string(),
        "/usr/share/plasma".to_string(),
        "/usr/share/plasma5".to_string(),
        "/usr/share/plasma6".to_string(),
        "/opt/share".to_string(),
        "/var/lib/snapd/desktop".to_string(),
    ];
    
    let system_dirs: Vec<String> = env::var("XDG_DATA_DIRS")
        .unwrap_or_default()
        .split(":")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    
    let dir_set: HashSet<String> = default_dirs.into_iter().chain(system_dirs).collect();
    let mut clean_dirs: Vec<String> = dir_set.into_iter().collect();
    clean_dirs.retain(|s| !s.starts_with("/nix/store/"));

    let start = Instant::now();

    clean_dirs
        .iter()
        .filter_map(|dir| {
            let path = format!("{}/applications", dir);
            fs::read_dir(&path).ok()
        })
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .par_bridge()
        .filter_map(|path| {
            fs::metadata(&path)
                .ok()
                .filter(|meta| meta.is_file())
                .and_then(|_| {
                    get_desktop_data_raw(&path)
                        .ok()
                        .map(|item| (path.clone(), item))
                })
        })
        .for_each(|(path, item)| {
            let path_key = path.to_string_lossy().to_lowercase();
            let mut seen = seen.lock().unwrap();
            if seen.insert(path_key)
                && let Ok(mut cb) = on_item.lock() {
                    cb(item);
                }
        });

    println!(
        "Finished scraping directories, took {:.2}ms",
        start.elapsed().as_millis()
    );
}

fn get_desktop_data_raw(path: &Path) -> Result<(String, String, String), Box<dyn Error>> {
    if let Ok(conf) = Ini::load_from_file(path) {
        match conf.section(Some("Desktop Entry")) {
            Some(section) => {
                let desktop_name = section.get("Name").unwrap_or("");
                let desktop_command = section.get("Exec").unwrap_or("");
                let desktop_keywords = section.get("Keywords").unwrap_or("");
                let desktop_type = section.get("Type").unwrap_or("");

                if desktop_type == "Application" {
                    let desktop_command = strip_field_codes_regex(desktop_command);

                    Ok((
                        desktop_name.to_string(),
                        desktop_command,
                        desktop_keywords.to_string(),
                    ))
                } else {
                    Err("Desktop entry doesnt have type or isnt type application".into())
                }
            }
            None => Err("Load failed".into()),
        }
    } else {
        Err("Load failed".into())
    }
}

fn strip_field_codes_regex(exec: &str) -> String {
    let re_exec = RE_EXEC.get_or_init(|| Regex::new(r"@@.*@@").unwrap());
    let result = re_exec.replace_all(exec, "");

    let re_xdg = RE_XDG.get_or_init(|| Regex::new(r"%[fFuUdDnNickvm]").unwrap());
    let result = re_xdg.replace_all(&result, "");

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

