use ini::Ini;
use slint::{Color, ComponentHandle};
use std::error::Error;

use crate::{LauncherWindow, Theme};

pub fn get_window_info() -> (u32, u32) {
    let home = std::env::var("HOME").unwrap();
    let config_path = format!("{}/.config/swift/theme.conf", home);

    if let Ok(conf) = Ini::load_from_file(config_path) {
        if let Some(section) = conf.section(Some("Window")) {
            let width = section.get("width").unwrap_or("").parse::<u32>().unwrap();
            let height = section.get("height").unwrap_or("").parse::<u32>().unwrap();

            (width, height)
        } else {
            (600, 400)
        }
    } else {
        (600, 400)
    }
}

pub fn apply_theme(ui: &LauncherWindow) -> Result<(), Box<dyn Error>> {
    println!("Applying theme");
    let home = std::env::var("HOME").unwrap();
    let config_path = format!("{}/.config/swift/theme.conf", home);
    let theme = ui.global::<Theme>();

    if let Ok(conf) = Ini::load_from_file(config_path) {
        if let Some(section) = conf.section(Some("Window")) {
            let set_if_present = |key: &str, setter: &dyn Fn(f32)| {
                if let Some(val) = section.get(key).and_then(|v| v.parse::<f32>().ok()) {
                    setter(val);
                }
            };

            set_if_present("width", &|v| theme.set_width(v));
            set_if_present("height", &|v| theme.set_height(v));
            set_if_present("border-radius", &|v| theme.set_border_radius(v));
            set_if_present("border-width", &|v| theme.set_border_width(v));

            if let Some(color_str) = section
                .get("background-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_background_color(color_str);
            }

            if let Some(color_str) = section
                .get("border-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_border_color(color_str);
            }
        }

        if let Some(section) = conf.section(Some("Action")) {
            let set_if_present = |key: &str, setter: &dyn Fn(f32)| {
                if let Some(val) = section.get(key).and_then(|v| v.parse::<f32>().ok()) {
                    setter(val);
                }
            };

            set_if_present("max-height", &|v| theme.set_max_height(v));
            set_if_present("option-border-radius", &|v| {
                theme.set_option_border_radius(v)
            });
            set_if_present("name-font-size", &|v| theme.set_name_font_size(v));
            set_if_present("exec-font-size", &|v| theme.set_exec_font_size(v));

            // Parse colors (hex)
            if let Some(color_str) = section
                .get("option-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_option_color(color_str);
            }
            if let Some(color_str) = section
                .get("option-color-selected")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_option_color_selected(color_str);
            }
            if let Some(color_str) = section
                .get("name-font-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_name_font_color(color_str);
            }
            if let Some(color_str) = section
                .get("name-font-color-selected")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_name_font_color_selected(color_str);
            }
            if let Some(color_str) = section
                .get("exec-font-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_exec_font_color(color_str);
            }
            if let Some(color_str) = section
                .get("exec-font-color-selected")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_exec_font_color_selected(color_str);
            }

            // Parse booleans
            if let Some(show) = section
                .get("exec-show")
                .and_then(|v| v.parse::<bool>().ok())
            {
                theme.set_exec_show(show);
            }
        }

        if let Some(section) = conf.section(Some("Runner")) {
            let set_if_present = |key: &str, setter: &dyn Fn(f32)| {
                if let Some(val) = section.get(key).and_then(|v| v.parse::<f32>().ok()) {
                    setter(val);
                }
            };

            set_if_present("font-size", &|v| theme.set_runner_font_size(v));
            set_if_present("border-width", &|v| theme.set_runner_border_width(v));
            set_if_present("border-radius", &|v| theme.set_runner_border_radius(v));
            set_if_present("height", &|v| theme.set_runner_height(v));

            if let Some(color_str) = section
                .get("background-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_runner_background_color(color_str);
            }
            if let Some(color_str) = section
                .get("border-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_runner_border_color(color_str);
            }
            if let Some(color_str) = section
                .get("font-color")
                .and_then(|c| parse_hex_color(c).ok())
            {
                theme.set_runner_color(color_str);
            }
        }
    }

    Ok(())
}

fn parse_hex_color(hex: &str) -> Result<Color, Box<dyn Error>> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err("Invalid hex length".into());
    }
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;
    Ok(Color::from_rgb_u8(r, g, b))
}
