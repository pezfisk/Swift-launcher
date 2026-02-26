use ini::Ini;
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs;
// use std::option::Option;
use std::path::Path;
use std::time::Instant;

use crate::ActionItem;

pub fn get_programs() -> Vec<ActionItem> {
    let data_dirs = env::var("XDG_DATA_DIRS").unwrap_or_else(|_| {
        "/var/lib/flatpak/exports/share:/usr/local/share:/usr/share:/usr/share/gnome:/usr/share/plasma:/var/lib/snapd/desktop".to_string()
    });
    let mut clean_dirs: Vec<&str> = data_dirs.split(":").collect();
    clean_dirs.retain(|&s| !s.starts_with("/nix/store/"));

    let start = Instant::now();

    // println!("{:?}", clean_dirs);

    // let mut items = Vec::new();

    // for dir in clean_dirs {
    //     // println!("Current dir: {:?}", dir);
    //     if let Ok(entries) = fs::read_dir(format!("{}/applications", dir)) {
    //         for entry in entries {
    //             if let Ok(dir_entry) = entry {
    //                 let path = dir_entry.path();
    //                 // println!("Entry: {:?} file_type: {:?}", &dir_entry, &file_type);

    //                 if let Ok(meta) = fs::metadata(&path) {
    //                     if meta.is_file() {
    //                         // println!("Found desktop file");

    //                         if let Ok(action_item) = get_desktop_data(&path) {
    //                             items.push(action_item);
    //                         }
    //                     } else if meta.is_dir() {
    //                         // println!("Skipping directory");
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

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

    let items: Vec<ActionItem> = all_app_dirs
        .into_iter()
        .filter_map(|path| {
            fs::metadata(&path)
                .ok()
                .filter(|meta| meta.is_file())
                .and_then(|_| get_desktop_data(&path).ok())
        })
        .collect();

    println!(
        "Finished scraping directories, took {:.2}ms",
        start.elapsed().as_millis()
    );

    items
}

fn get_desktop_data(path: &Path) -> Result<ActionItem, Box<dyn Error>> {
    // let desktop_file = Ini::load_from_file(&path).unwrap();
    // println!("Getting .desktop data");

    if let Ok(conf) = Ini::load_from_file(path) {
        match conf.section(Some("Desktop Entry")) {
            Some(section) => {
                let desktop_name = section.get("Name").unwrap_or("");
                let desktop_command = section.get("Exec").unwrap_or("");
                let desktop_keywords = section.get("Keywords").unwrap_or("");
                let desktop_type = section.get("Type").unwrap_or("");

                if desktop_type == "Application" {
                    // println!(
                    //     "desktop app found! Name: {} -- Exec: {}",
                    //     desktop_name, desktop_command
                    // );

                    let desktop_command = strip_field_codes_regex(desktop_command);

                    Ok(ActionItem {
                        name: desktop_name.into(),
                        exec: desktop_command.into(),
                        keywords: desktop_keywords.into(),
                        icon: Default::default(),
                    })
                } else {
                    println!("Desktop entry doesnt have type or isnt type application");
                    Err("Desktop entry doesnt have type or isnt type application".into())
                }
            }
            None => {
                println!("No Desktop entry");
                Err("Load failed".into())
            }
        }
    } else {
        Err("Load failed".into())
    }

    // let section = desktop_file.section(Some("Desktop Entry"))?;
}

fn strip_field_codes_regex(exec: &str) -> String {
    let re_exec = Regex::new(r"@@.*@@").unwrap();
    let result = re_exec.replace_all(exec, "");

    let re_xdg = Regex::new(r"%[fFuUdDnNickvm]").unwrap();
    let result = re_xdg.replace_all(&result, "");

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
