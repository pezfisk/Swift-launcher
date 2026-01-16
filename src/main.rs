slint::include_modules!();

use ini::Ini;
use slint::{Model, ModelRc, VecModel};
use std::error::Error;
use std::fs;
use std::process::Command;
use std::rc::Rc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use evalexpr::*;

use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
    wayland_adapter::SpellWin,
};

mod scraper;

enum queryType {
    search,
    calculator,
    directory,
}

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");

    let ui = LauncherWindow::new()?;

    let all_actions = scraper::get_programs();
    println!("{:?}", all_actions);

    // Create action model
    let ui_actions = Rc::new(VecModel::<ActionItem>::default());
    ui_actions.set_vec(all_actions.clone());
    ui.set_actions(ModelRc::from(ui_actions.clone()));

    let ui_handle = ui.as_weak();
    let master_list = all_actions.clone();
    let display_model = ui_actions.clone();

    // Handle action clicks
    let actions_clone = ui_actions.clone();
    ui.on_action_clicked(move |idx| {
        if let Some(action) = actions_clone.row_data(idx as usize) {
            println!("Executing: {} - {}", action.name, action.exec);
            // TODO: Execute command here
        }
    });

    ui.on_search_changed(move |text: slint::SharedString| {
        let query = text.as_str().trim();
        let matcher = SkimMatcherV2::default();

        if query.is_empty() {
            display_model.set_vec(master_list.clone());
            return;
        }

        match parse_input(query) {
            Ok(queryType::search) => {
                let mut filtered: Vec<(i64, ActionItem)> = display_model
                    .iter()
                    .filter_map(|item| {
                        let score = matcher
                            .fuzzy_match(&item.name, &text)
                            .or_else(|| matcher.fuzzy_match(&item.exec, &text))
                            .or_else(|| matcher.fuzzy_match(&item.keywords, &text));

                        score.map(|s| (s, item.clone()))
                    })
                    .collect();

                filtered.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

                let new_model: Vec<ActionItem> =
                    filtered.into_iter().map(|(_, item)| item).collect();
                display_model.set_vec(new_model);
            }
            Ok(queryType::calculator) => {
                let equation = query.split_once("=").unwrap().1;
                match eval(equation) {
                    Ok(value) => {
                        // println!("Equation has answer");
                        // println!("{}", value);

                        let new_model: Vec<ActionItem> = vec![ActionItem {
                            exec: slint::SharedString::from(""),
                            keywords: slint::SharedString::from("Test"),
                            name: slint::SharedString::from(format!("{} = {}", equation, value)),
                        }];

                        display_model.set_vec(new_model);
                    }

                    Err(_) => {
                        println!("Equation has no answer");
                    }
                }
            }
            Ok(queryType::directory) => {
                if query.ends_with("/") {
                    if let Ok(dir) = fs::read_dir(query) {
                        let mut new_model: Vec<ActionItem> = vec![];
                        for entry in dir {
                            if let Ok(dir_entry) = entry {
                                println!("{:?}", dir_entry);
                                new_model.push(ActionItem {
                                    exec: slint::SharedString::from(""),
                                    keywords: slint::SharedString::from(""),
                                    name: slint::SharedString::from(
                                        dir_entry.path().to_str().unwrap(),
                                    ),
                                });
                            }
                        }

                        display_model.set_vec(new_model);
                    }
                }
            }

            _ => {}
        }
    });

    ui.on_linefinished(move |app| {
        let foo = Command::new("sh").arg("-c").arg(app.as_str()).spawn();
        slint::quit_event_loop();

        // Force quit in case slint::quit_event_loop() fails
        std::process::exit(0);
    });

    ui.on_accepted({
        let ui_actions_clone = ui_actions.clone();

        move || {
            let ui = ui_handle.unwrap();
            if let Some(first_item) = ui_actions_clone.row_data(0) {
                println!("Launching: {}", first_item.name);

                let _ = Command::new("sh").arg("-c").arg(&first_item.exec).spawn();

                slint::quit_event_loop();
            }
        }
    });

    if !is_gnome() {
        println!("Using wlr-layer-shell");

        let window_conf = WindowConf::new(
            600,
            400,
            (Some(LayerAnchor::TOP), None),
            (10, 0, 0, 0),
            LayerType::Overlay,
            BoardType::Exclusive,
            None,
        );

        let waywindow = SpellWin::invoke_spell("launcher", window_conf);

        cast_spell(waywindow, None, None).unwrap();
    } else {
        println!("Running on gnome, using standard window");
        ui.run()?;
    }

    Ok(())
}

fn parse_input(query: &str) -> Result<queryType, Box<dyn Error>> {
    if let Some(first_char) = query.chars().next().as_ref() {
        match first_char {
            '=' => {
                println!("Calculator mode");
                Ok(queryType::calculator)
            }

            '/' => {
                println!("Directory mode");
                Ok(queryType::directory)
            }

            _ => {
                println!("Normal mode");
                Ok(queryType::search)
            }
        }
    } else {
        Err("Failed to get first character".into())
    }
}

fn is_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .map(|desktop| desktop.to_lowercase().contains("gnome"))
        .unwrap_or(false)
        || std::env::var("XDG_SESSION_DESKTOP")
            .map(|desktop| desktop.to_lowercase().contains("gnome"))
            .unwrap_or(false)
}

// fn fuzzy_search()
