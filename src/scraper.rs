use ini::Ini;
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs;
// use std::option::Option;
use std::path::Path;
use std::time::Instant;
use rayon::prelude::*;

pub fn get_programs_raw() -> Vec<(String, String, String)> {
    let data_dirs = env::var("XDG_DATA_DIRS").unwrap_or_else(|_| {
        "/var/lib/flatpak/exports/share:/usr/local/share:/usr/share:/usr/share/gnome:/usr/share/plasma:/var/lib/snapd/desktop".to_string()
    });
    let mut clean_dirs: Vec<&str> = data_dirs.split(":").collect();
    clean_dirs.retain(|&s| !s.starts_with("/nix/store/"));

    let start = Instant::now();

    let all_app_dirs: Vec<_> = clean_dirs
        .iter()
        .filter_map(|dir| {
            let path = format!("{}/applications", dir);
            fs::read_dir(&path).ok()
        })
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    let raw_items: Vec<(String, String, String)> = all_app_dirs
        .into_par_iter()
        .filter_map(|path| {
            fs::metadata(&path)
                .ok()
                .filter(|meta| meta.is_file())
                .and_then(|_| get_desktop_data_raw(&path).ok())
        })
        .collect();

    println!(
        "Finished scraping directories, took {:.2}ms",
        start.elapsed().as_millis()
    );

    raw_items
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
    let re_exec = Regex::new(r"@@.*@@").unwrap();
    let result = re_exec.replace_all(exec, "");

    let re_xdg = Regex::new(r"%[fFuUdDnNickvm]").unwrap();
    let result = re_xdg.replace_all(&result, "");

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
