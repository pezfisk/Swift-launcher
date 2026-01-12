slint::include_modules!();

use slint::{Model, ModelRc, VecModel};
use std::process::Command;
use std::rc::Rc;

fn main() -> Result<(), slint::PlatformError> {
    println!("Hello, world!");
    let ui = LauncherWindow::new()?;

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
        let foo = Command::new(app).output().unwrap();
        std::process::exit(0);

        println!("FOO: {}", String::from_utf8_lossy(&foo.stdout));
    });

    // Make window frameless and centered
    // ui.window().set_decorated(false);

    ui.run()
}
