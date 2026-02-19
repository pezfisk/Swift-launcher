use bindings::exports::swift::launcher::runner::{ActionItem, Guest};
mod bindings;

use ini::Ini;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

struct SearchPlugin;

impl Guest for SearchPlugin {
    fn get_trigger() -> String {
        "@".to_string()
    }

    // TODO: NEED TO ADD PARSING FROM CONFIG FILE
    fn handle(input: String) -> Vec<ActionItem> {
        println!("internet_mode");
        let cleaned = input.trim_start_matches('@').trim();
        let mut action = Vec::new();
        let mut engine_map: HashMap<String, String> = HashMap::new();
        engine_map.insert("g".to_string(), "google.com/search".to_string());
        engine_map.insert("d".to_string(), "duckduckgo.com/".to_string());

        let mut config_path = PathBuf::from("/config/search.conf");

        if let Ok(conf) = Ini::load_from_file(&config_path) {
            if let Some(section) = conf.section(None::<String>) {
                for (key, value) in section.iter() {
                    engine_map.insert(key.to_string(), value.to_string());
                }
            }
        } else {
            println!("Failed to get config");
        }

        if let Some((engine, query)) = cleaned.split_once(' ') {
            if let Some(engine_url) = engine_map.get(engine) {
                let exec = query.replace(" ", "+");
                println!("domain: {:?} search: {:?}", engine, exec);

                let search_url = format!("xdg-open https://{}?q={}", engine_url, exec);
                action.push(ActionItem {
                    name: format!("hey"),
                    exec: search_url.into(),
                    keywords: "".into(),
                });
            } else {
                action.push(ActionItem {
                    name: format!("Using {}", engine),
                    exec: "".into(),
                    keywords: "".into(),
                });
            }
        } else {
            action.extend(show_search_engines(&engine_map));
        }

        action
    }
}

fn show_search_engines(map: &HashMap<String, String>) -> Vec<ActionItem> {
    let mut engines = Vec::new();
    for (key, value) in map {
        println!("{}{}", key, value);

        engines.push(ActionItem {
            name: value.to_string(),
            exec: key.to_string(),
            keywords: "".into(),
        })
    }
    engines
}

bindings::export!(SearchPlugin with_types_in bindings);
