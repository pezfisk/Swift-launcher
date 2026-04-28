slint::include_modules!();
spell_framework::generate_widgets![LauncherWindow];

use slint::{Model, ModelRc, VecModel};
use std::error::Error;
use std::process::Command;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, LayerType, WindowConf},
};

use icon_finder::find_icon;

mod cache;
mod config;
mod plugins;
mod scraper;
mod theme;

use std::cell::RefCell;

thread_local! {
    static UI_ACTIONS: RefCell<Option<Rc<VecModel<ActionItem>>>> = const { RefCell::new(None) };
    static MASTER_LIST: RefCell<Vec<ActionItem>> = const { RefCell::new(Vec::new()) };
    static FALLBACK_ICON: RefCell<Option<slint::Image>> = const { RefCell::new(None) };
}

static USAGE_CACHE: std::sync::OnceLock<Arc<Mutex<cache::UsageCache>>> = std::sync::OnceLock::new();
static MATCHER: std::sync::OnceLock<SkimMatcherV2> = std::sync::OnceLock::new();

fn main() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").expect("HOME environment variable must be set");

    let usage_cache = cache::UsageCache::load();
    USAGE_CACHE.set(Arc::new(Mutex::new(usage_cache))).ok();

    let fallback_path = format!("{}/.config/swift/icons/fallback.png", home);
    if let Ok(icon) = slint::Image::load_from_path(std::path::Path::new(&fallback_path)) {
        FALLBACK_ICON.with(|f| *f.borrow_mut() = Some(icon));
    }

    let window_size = theme::get_window_info();
    let window_conf = WindowConf::builder()
        .width(window_size.0)
        .height(window_size.1)
        .board_interactivity(BoardType::Exclusive)
        .layer_type(LayerType::Overlay)
        .build()
        .unwrap();

    let ui = LauncherWindowSpell::invoke_spell("swift-launcher", window_conf);

    let _theme = theme::apply_theme(&ui);

    let manager = Arc::new(Mutex::new(plugins::PluginManager::new()));
    let manager_bg = Arc::clone(&manager);

    rayon::spawn(move || {
        let mut mg = manager_bg.lock().unwrap();
        if let Err(e) = mg.load_all() {
            eprintln!("Failed to load plugins: {}", e);
        }
    });

    let ui_actions = std::rc::Rc::new(VecModel::<ActionItem>::default());
    ui.set_actions(ModelRc::from(ui_actions.clone()));

    UI_ACTIONS.with(|a| *a.borrow_mut() = Some(ui_actions.clone()));

    let (tx, rx) = mpsc::channel::<(String, String, String)>();

    rayon::spawn({
        let tx_scraper = tx.clone();
        let tx_config = tx.clone();
        move || {
            scraper::get_programs_raw_streaming(move |item| {
                let _ = tx_scraper.send(item);
            });

            for item in config::load_config_raw() {
                let _ = tx_config.send(item);
            }
        }
    });

    let pending = std::cell::RefCell::new(Vec::<(String, String, String)>::new());
    let item_count: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
    let icon_batch_size = 20;
    let icons_loaded: std::cell::RefCell<usize> = std::cell::RefCell::new(0);

    #[allow(clippy::arc_with_non_send_sync)]
    let timer = std::sync::Arc::new(std::sync::Mutex::new(slint::Timer::default()));
    let timer_clone = timer.clone();
    timer_clone.lock().unwrap().start(slint::TimerMode::Repeated, std::time::Duration::from_millis(50), move || {
        while let Ok(item) = rx.try_recv() {
            pending.borrow_mut().push(item.clone());
            MASTER_LIST.with(|m| {
                m.borrow_mut().push(ActionItem {
                    name: item.0.clone().into(),
                    exec: item.1.clone().into(),
                    keywords: item.2.clone().into(),
                    icon: Default::default(),
                });
            });
        }

        if !pending.borrow().is_empty() {
            let batch: Vec<(String, String, String)> = pending.borrow_mut().drain(..).collect();
            let batch_len = batch.len();
            
            let loaded = *icons_loaded.borrow();
            let batch = if loaded < icon_batch_size && batch_len > 0 {
                let priorities: Vec<u32> = batch.iter()
                    .map(|(_, exec, _)| {
                        USAGE_CACHE.get().map(|c| c.lock().unwrap().get_priority(exec)).unwrap_or(0)
                    })
                    .collect();
                let mut indices: Vec<usize> = (0..batch_len).collect();
                indices.sort_by(|&a, &b| priorities[b].cmp(&priorities[a]));
                indices.into_iter()
                    .map(|i| batch[i].clone())
                    .collect()
            } else {
                batch
            };

            *item_count.borrow_mut() += batch_len;
            
            let items: Vec<ActionItem> = batch
                .into_iter()
                .enumerate()
                .map(|(i, (name, exec, keywords))| {
                    let _global_idx = *item_count.borrow() - batch_len + i;
                    let loaded = *icons_loaded.borrow();
                    
                    let icon = if loaded < icon_batch_size {
                        *icons_loaded.borrow_mut() += 1;
                        if let Some(ico_path) = find_icon(&name.to_lowercase(), 256) {
                            slint::Image::load_from_path(&ico_path).unwrap_or_else(|_| {
                                FALLBACK_ICON.with(|f| f.borrow().clone()).unwrap_or_default()
                            })
                        } else {
                            FALLBACK_ICON.with(|f| f.borrow().clone()).unwrap_or_default()
                        }
                    } else {
                        Default::default()
                    };
                    
                    ActionItem {
                        name: name.into(),
                        exec: exec.into(),
                        keywords: keywords.into(),
                        icon,
                    }
                })
                .collect();

            UI_ACTIONS.with(|a| {
                if let Some(ui_actions) = a.borrow().as_ref() {
                    ui_actions.extend(items);
                }
            });
        }
    });

    let ui_handle = ui.as_weak();

    ui.on_action_clicked(move |idx| {
        UI_ACTIONS.with(|a| {
            if let Some(ui_actions) = a.borrow().as_ref()
                && let Some(action) = ui_actions.row_data(idx as usize) {
                    println!("Executing: {} - {}", action.name, action.exec);
                    let _foo = Command::new("sh")
                        .arg("-c")
                        .arg(action.exec.as_str())
                        .spawn();
                }
        });
    });

    let matcher = MATCHER.get_or_init(SkimMatcherV2::default);
    ui.on_search_changed(move |text: slint::SharedString| {
        let query = text.as_str().trim();

        if query.is_empty() {
            UI_ACTIONS.with(|a| {
                if let Some(ui_actions) = a.borrow().as_ref() {
                    MASTER_LIST.with(|m| {
                        let mut items: Vec<ActionItem> = m.borrow().clone().into_iter().collect();
                        items.sort_by(|a, b| {
                            let priority_a = USAGE_CACHE.get()
                                .map(|c| c.lock().unwrap().get_priority(&a.exec))
                                .unwrap_or(0);
                            let priority_b = USAGE_CACHE.get()
                                .map(|c| c.lock().unwrap().get_priority(&b.exec))
                                .unwrap_or(0);
                            priority_b.cmp(&priority_a)
                        });
                        ui_actions.set_vec(items);
                    });
                }
            });
            return;
        }

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
                    let mut filtered: Vec<(i64, u32, ActionItem)> = master_list
                        .iter()
                        .filter_map(|item| {
                            let score = matcher
                                .fuzzy_match(&item.name, &text)
                                .or_else(|| matcher.fuzzy_match(&item.keywords, &text))
                                .or_else(|| matcher.fuzzy_match(&item.exec, &text));

                            let priority = USAGE_CACHE.get()
                                .map(|c| c.lock().unwrap().get_priority(&item.exec))
                                .unwrap_or(0);

                            score.map(|s| (s, priority, item.clone()))
                        })
                        .collect();

                    filtered.sort_by(|(score_a, priority_a, _), (score_b, priority_b, _)| {
                        if priority_a != priority_b {
                            priority_b.cmp(priority_a)
                        } else {
                            score_b.cmp(score_a)
                        }
                    });

                    let new_model: Vec<ActionItem> = filtered
                        .into_iter()
                        .enumerate()
                        .map(|(i, (_, _, item))| {
                            let icon = if !item.icon.size().is_empty() {
                                item.icon.clone()
                            } else if i < 5 {
                                if let Some(ico_path) =
                                    find_icon(&item.name.as_str().to_lowercase(), 256)
                                {
                                    slint::Image::load_from_path(&ico_path).unwrap_or_else(|_| {
                                        FALLBACK_ICON.with(|f| f.borrow().clone()).unwrap_or_default()
                                    })
                                } else {
                                    FALLBACK_ICON.with(|f| f.borrow().clone()).unwrap_or_default()
                                }
                            } else {
                                FALLBACK_ICON.with(|f| f.borrow().clone()).unwrap_or_default()
                            };

                            ActionItem {
                                name: item.name,
                                exec: item.exec,
                                keywords: item.keywords,
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
            if let Some(ui_actions) = a.borrow().as_ref()
                && let Some(first_item) = ui_actions.row_data(selected.try_into().unwrap()) {
                    println!("Launching: {}", first_item.name);

                    if let Some(cache) = USAGE_CACHE.get() {
                        cache.lock().unwrap().increment(&first_item.exec);
                        cache.lock().unwrap().save();
                    }

                    let _ = Command::new("sh").arg("-c").arg(&first_item.exec).spawn();

                    let _ = slint::quit_event_loop();

                    std::process::exit(0);
                }
        });
    });

    ui.on_quit(move || {
        std::process::exit(0);
    });

    cast_spell!(ui)
}
