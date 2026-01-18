use bindings::exports::shift::launcher::runner::{ActionItem, Guest};
mod bindings;

struct Calculator;

impl Guest for Calculator {
    fn get_trigger() -> String {
        "=".to_string()
    }

    fn handle(input: String) -> Vec<ActionItem> {
        let expression = input.trim_start_matches('=');
        let result = evalexpr::eval(expression)
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "Error".into());

        vec![ActionItem {
            name: format!("{} = {}", expression, result),
            exec: format!("echo -n '{}' | wl-copy", result),
            keywords: "=".into(),
        }]
    }
}

bindings::export!(Calculator with_types_in bindings);
