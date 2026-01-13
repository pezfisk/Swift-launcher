slint::include_modules!();

use ini::Ini;
use slint::{Model, ModelRc, VecModel};
use std::process::Command;
use std::rc::Rc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

mod scraper;

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");
    let ui = LauncherWindow::new()?;

    let all_actions = scraper::get_flatpaks();
    println!("{:?}", all_actions);

    let desktop_file =
        Ini::load_from_file("/var/lib/flatpak/exports/share/applications/dev.invrs.oxide.desktop")
            .unwrap();

    // let section = desktop_file.section(Some("Desktop Entry")).unwrap();
    // let desktop_name = section.get("Name").unwrap();
    // let desktop_command = section.get("Exec").unwrap();

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

        let new_model: Vec<ActionItem> = filtered.into_iter().map(|(_, item)| item).collect();
        display_model.set_vec(new_model);
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

    // Make window frameless and centered
    // ui.window().set_decorated(false);

    ui.run()
}
