use bindings::exports::swift::launcher::runner::{ActionItem, Guest};
mod bindings;

use std::collections::HashMap;
use std::env;
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
        let mut engine_map = HashMap::new();
        engine_map.insert("g", "google.com/search");
        engine_map.insert("d", "duckduckgo.com/");

        if let Some((engine, query)) = cleaned.split_once(' ') {
            if let Some(engine_url) = engine_map.get(engine) {
                let exec = query.replace(" ", "+");
                println!("domain: {:?} search: {:?}", engine, exec);

                let search_url = format!("xdg-open https:/{}?q={}", engine_url, exec);
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

fn show_search_engines(map: &HashMap<&str, &str>) -> Vec<ActionItem> {
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
