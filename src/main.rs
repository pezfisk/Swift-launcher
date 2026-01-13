slint::include_modules!();

use ini::Ini;
use slint::{Model, ModelRc, VecModel};
use std::process::Command;
use std::rc::Rc;

mod scraper;

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");
    let ui = LauncherWindow::new()?;

    scraper::get_flatpaks();

    let desktop_file =
        Ini::load_from_file("/var/lib/flatpak/exports/share/applications/dev.invrs.oxide.desktop")
            .unwrap();

    let section = desktop_file.section(Some("Desktop Entry")).unwrap();
    let desktop_name = section.get("Name").unwrap();
    let desktop_command = section.get("Exec").unwrap();

    // Create action model
    let actions = Rc::new(VecModel::<ActionItem>::default());
    actions.push(ActionItem {
        name: "Open dotfiles".into(),
        description: "~/dotfiles".into(),
    });
    actions.push(ActionItem {
        name: "Cargo build".into(),
        description: "cargo build --release".into(),
    });
    actions.push(ActionItem {
        name: "Run tests".into(),
        description: "cargo test".into(),
    });
    actions.push(ActionItem {
        name: slint::SharedString::from(desktop_name),
        description: slint::SharedString::from(desktop_command),
    });

    ui.set_actions(ModelRc::from(actions.clone()));

    // Handle action clicks
    let actions_clone = actions.clone();
    ui.on_action_clicked(move |idx| {
        if let Some(action) = actions_clone.row_data(idx as usize) {
            println!("Executing: {} - {}", action.name, action.description);
            // TODO: Execute command here
        }
    });

    ui.on_linefinished(move |app| {
        let foo = Command::new(app).spawn().unwrap();
        slint::quit_event_loop();

        // Force quit in case slint::quit_event_loop() fails
        std::process::exit(0);
    });

    // Make window frameless and centered
    // ui.window().set_decorated(false);

    ui.run()
}
