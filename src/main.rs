#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod engine;
#[cfg(target_os = "linux")]
mod linux_display;
mod window_state;

use tauri::Manager;

fn main() {
    // Before anything else, and without opening a window: the release
    // workflow smoke-tests every build with `--version`.
    if asks_for_version(std::env::args().skip(1)) {
        println!("orCAL {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    #[cfg(target_os = "linux")]
    linux_display::configure();

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").expect("main window");
            window_state::attach(app.handle(), &window);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::evaluate_expression])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Whether the command line, without the program name, asks for the version.
fn asks_for_version(args: impl IntoIterator<Item = impl AsRef<str>>) -> bool {
    args.into_iter()
        .any(|arg| matches!(arg.as_ref(), "--version" | "-V"))
}

#[cfg(test)]
mod tests {
    use super::asks_for_version;

    #[test]
    fn version_flag_is_recognised_anywhere() {
        assert!(asks_for_version(["--version"]));
        assert!(asks_for_version(["-V"]));
        assert!(asks_for_version(["--foo", "--version"]));
    }

    #[test]
    fn other_arguments_start_the_app() {
        assert!(!asks_for_version(Vec::<String>::new()));
        assert!(!asks_for_version(["--versions", "-v", "version"]));
    }
}
