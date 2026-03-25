slint::include_modules!();
spell_framework::generate_widgets![LauncherWindow];

use slint::{Model, ModelRc, VecModel};
use std::error::Error;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, LayerType, WindowConf},
};

use icon_finder::find_icon;

mod config;
mod plugins;
mod scraper;
mod theme;

use std::cell::RefCell;

thread_local! {
    static UI_ACTIONS: RefCell<Option<Rc<VecModel<ActionItem>>>> = RefCell::new(None);
    static MASTER_LIST: RefCell<Vec<ActionItem>> = RefCell::new(Vec::new());
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let window_size = theme::get_window_info();
    println!("{:?}", window_size);

    let window_conf = WindowConf::builder()
        .width(window_size.0)
        .height(window_size.1)
        .board_interactivity(BoardType::Exclusive)
        .layer_type(LayerType::Overlay)
        .build()
        .unwrap();

    let ui = LauncherWindowSpell::invoke_spell("swift-launcher", window_conf);

    // let ui = LauncherWindow::new()?;
    let _theme = theme::apply_theme(&ui);

    let manager = Arc::new(Mutex::new(plugins::PluginManager::new()));
    let manager_bg = Arc::clone(&manager);

    rayon::spawn(move || {
        let mut mg = manager_bg.lock().unwrap();
        if let Err(e) = mg.load_all() {
            eprintln!("Failed to load plugins: {}", e);
        }
    });

    let ui_actions = Rc::new(VecModel::<ActionItem>::default());
    ui.set_actions(ModelRc::from(ui_actions.clone()));

    UI_ACTIONS.with(|a| *a.borrow_mut() = Some(ui_actions.clone()));

    rayon::spawn(move || {
        let mut raw_programs = scraper::get_programs_raw();
        raw_programs.extend(config::load_config_raw());

        let _ = slint::invoke_from_event_loop(move || {
            let items: Vec<ActionItem> = raw_programs
                .into_iter()
                .map(|(name, exec, keywords)| ActionItem {
                    name: name.into(),
                    exec: exec.into(),
                    keywords: keywords.into(),
                    icon: Default::default(),
                })
                .collect();

            MASTER_LIST.with(|m| *m.borrow_mut() = items.clone());

            UI_ACTIONS.with(|a| {
                if let Some(ui_actions) = a.borrow().as_ref() {
                    ui_actions.set_vec(items);
                }
            });
        });
    });

    let ui_handle = ui.as_weak();

    ui.on_action_clicked(move |idx| {
        UI_ACTIONS.with(|a| {
            if let Some(ui_actions) = a.borrow().as_ref() {
                if let Some(action) = ui_actions.row_data(idx as usize) {
                    println!("Executing: {} - {}", action.name, action.exec);
                    let _foo = Command::new("sh")
                        .arg("-c")
                        .arg(action.exec.as_str())
                        .spawn();
                }
            }
        });
    });

    let matcher = SkimMatcherV2::default();
    ui.on_search_changed(move |text: slint::SharedString| {
        let query = text.as_str().trim();

        if query.is_empty() {
            UI_ACTIONS.with(|a| {
                if let Some(ui_actions) = a.borrow().as_ref() {
                    MASTER_LIST.with(|m| {
                        ui_actions.set_vec(m.borrow().clone());
                    });
                }
            });
            return;
        }
        println!("Search changed!");

        if let Some(first_char) = query.chars().next()
            && let Ok(mg) = manager.try_lock()
        {
            if let Some(res) = mg.run_trigger(first_char, query) {
                let items: Vec<ActionItem> = res
                    .into_iter()
                    .map(|item| ActionItem {
                        name: item.name.into(),
                        exec: item.exec.into(),
                        keywords: item.keywords.into(),
                        icon: Default::default(),
                    })
                    .collect();
                UI_ACTIONS.with(|a| {
                    if let Some(ui_actions) = a.borrow().as_ref() {
                        ui_actions.set_vec(items);
                    }
                });
            } else {
                MASTER_LIST.with(|m| {
                    let master_list = m.borrow();
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

                    let new_model: Vec<ActionItem> = filtered
                        .into_iter()
                        .enumerate()
                        .map(|(i, (_, item))| {
                            let time = std::time::Instant::now();
                            let icon = if !item.icon.size().is_empty() {
                                item.icon.clone()
                            } else if i < 5 {
                                println!("loading icon {}, at index {}", item.name.as_str(), i);

                                if let Some(ico_path) =
                                    find_icon(&item.name.as_str().to_lowercase(), 256)
                                {
                                    println!("path: {:?}", ico_path);
                                    slint::Image::load_from_path(&ico_path).unwrap_or_default()
                                } else {
                                    println!("Failed to find icon");
                                    Default::default()
                                }
                            } else {
                                Default::default()
                            };

                            let elapsed = time.elapsed();
                            println!("Time took to find icons: {:.2?}", elapsed);

                            ActionItem {
                                name: item.name.into(),
                                exec: item.exec.into(),
                                keywords: item.keywords.into(),
                                icon,
                            }
                        })
                        .collect();

                    UI_ACTIONS.with(|a| {
                        if let Some(ui_actions) = a.borrow().as_ref() {
                            ui_actions.set_vec(new_model);
                        }
                    });
                });
            }
        }
    });

    ui.on_linefinished(move |app| {
        let _foo = Command::new("sh").arg("-c").arg(app.as_str()).spawn();
        // slint::quit_event_loop();

        // Force quit in case slint::quit_event_loop() fails
        std::process::exit(0);
    });

    ui.on_accepted(move || {
        let ui = ui_handle.unwrap();
        let selected = ui.get_selected();

        UI_ACTIONS.with(|a| {
            if let Some(ui_actions) = a.borrow().as_ref() {
                if let Some(first_item) = ui_actions.row_data(selected.try_into().unwrap()) {
                    println!("Launching: {}", first_item.name);

                    let _ = Command::new("sh").arg("-c").arg(&first_item.exec).spawn();

                    let _ = slint::quit_event_loop();

                    std::process::exit(0);
                }
            }
        });
    });

    ui.on_quit(move || {
        std::process::exit(0);
    });

    cast_spell!(ui)
}
