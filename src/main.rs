slint::include_modules!();

use ini::Ini;
use slint::{Model, ModelRc, VecModel};
use std::error::Error;
use std::fs;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, LayerAnchor, LayerType, WindowConf},
    wayland_adapter::SpellWin,
};

mod plugins;
mod scraper;
mod theme;

enum queryType {
    search,
    calculator,
    directory,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let window_size = theme::get_window_info();
    println!("{:?}", window_size);

    let window_conf = WindowConf::new(
        window_size.0,
        window_size.1,
        (None, None),
        (0, 0, 0, 0),
        LayerType::Overlay,
        BoardType::Exclusive,
        None,
    );

    let waywindow = SpellWin::invoke_spell("swift-launcher", window_conf);

    let ui = LauncherWindow::new()?;
    let theme = theme::apply_theme(&ui);

    let manager = Arc::new(Mutex::new(plugins::PluginManager::new()));
    let manager_bg = Arc::clone(&manager);

    thread::spawn(move || {
        let mut mg = manager_bg.lock().unwrap();

        if let Err(e) = mg.load_all() {
            eprintln!("Failed to load plugins: {}", e);
        }
    });
    // manager.load_all()?;

    // TOOD: Need to properly handle Option<>
    let all_actions = scraper::get_programs().unwrap();
    // println!("{:?}", all_actions);

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

    let matcher = SkimMatcherV2::default();
    ui.on_search_changed(move |text: slint::SharedString| {
        let query = text.as_str().trim();

        if query.is_empty() {
            display_model.set_vec(master_list.clone());
            return;
        }
        println!("Search changed!");

        if let Some(first_char) = query.chars().next() {
            if let Ok(mg) = manager.try_lock() {
                if let Some(res) = mg.run_trigger(first_char, query) {
                    let items: Vec<ActionItem> = res
                        .into_iter()
                        .map(|item| ActionItem {
                            name: item.name.into(),
                            exec: item.exec.into(),
                            keywords: item.keywords.into(),
                        })
                        .collect();
                    display_model.set_vec(items);
                    return;
                } else {
                    let mut filtered: Vec<(i64, ActionItem)> = master_list
                        .iter()
                        .filter_map(|item| {
                            let score = matcher
                                .fuzzy_match(&item.name, &text)
                                .or_else(|| matcher.fuzzy_match(&item.keywords, &text))
                                .or_else(|| matcher.fuzzy_match(&item.exec, &text));

                            score.map(|s| (s, item.clone()))
                        })
                        .collect();

                    filtered.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

                    let new_model: Vec<ActionItem> =
                        filtered.into_iter().map(|(_, item)| item).collect();
                    display_model.set_vec(new_model);
                }
            }
        }
    });

    ui.on_linefinished(move |app| {
        let foo = Command::new("sh").arg("-c").arg(app.as_str()).spawn();
        // slint::quit_event_loop();

        // Force quit in case slint::quit_event_loop() fails
        std::process::exit(0);
    });

    ui.on_accepted({
        let ui_actions_clone = ui_actions.clone();

        move || {
            let ui = ui_handle.unwrap();
            let selected = ui.get_selected();
            if let Some(first_item) = ui_actions_clone.row_data(selected.try_into().unwrap()) {
                println!("Launching: {}", first_item.name);

                let _ = Command::new("sh").arg("-c").arg(&first_item.exec).spawn();

                slint::quit_event_loop();

                std::process::exit(0);
            }
        }
    });

    ui.on_quit(move || {
        std::process::exit(0);
    });

    // if !is_gnome() {
    println!("Using wlr-layer-shell");
    // ui.run()?;
    cast_spell(waywindow, None, None)
    // } else {
    //     // GNOME IS CURRENTLY BROKEN --- NEED TO FIX
    //     println!("Running on gnome, using standard window");
    //     ui.run()?;
    // }

    // Ok(())
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
