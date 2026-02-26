use ini::Ini;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
// use std::path::{self, Path};

use crate::ActionItem;

pub fn load_config() -> Vec<ActionItem> {
    let mut actions = Vec::new();
    let conf = Ini::load_from_file(get_config_file().unwrap()).unwrap_or_default();

    let mut vars = HashMap::new();
    if let Some(section) = conf.section(Some("variables")) {
        for (k, v) in section.iter() {
            vars.insert(format!("${}", k), v);
        }
    }
    for (sec, prop) in &conf {
        let section_name = sec.unwrap_or("");

        if section_name.starts_with("action:") {
            let mut name = prop.get("name").unwrap_or("").to_string();
            let mut exec = prop.get("exec").unwrap_or("").to_string();
            let mut keywords = prop.get("keywords").unwrap_or("").to_string();

            // Replace custom variables ($editor, etc.)
            for (k, v) in &vars {
                name = name.replace(k, v);
                exec = exec.replace(k, v);
                keywords = keywords.replace(k, v);
            }

            actions.push(ActionItem {
                name: name.into(),
                exec: exec.into(),
                keywords: keywords.into(),
                icon: Default::default(),
            });
        }
    }
    actions
}

fn get_config_file() -> Result<PathBuf, Box<dyn Error>> {
    let config_dir = std::env::var("HOME")?;
    let file = PathBuf::from(format!("{}/.config/swift/config.conf", config_dir));
    Ok(file)
}
