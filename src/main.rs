#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod engine;
#[cfg(test)]
mod i18n;
#[cfg(target_os = "linux")]
mod linux_display;
mod migrate;
mod paths;
mod preferences;
mod window_state;

use preferences::Store;
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
            let config_dir = paths::config_dir()
                .inspect_err(|error| log::warn!("orCAL cannot keep its settings: {error}"))
                .ok();
            let legacy_dir = migrate::legacy_config_dir();
            if let (Some(config), Some(legacy)) = (&config_dir, &legacy_dir) {
                if let Err(error) = migrate::run(legacy, config) {
                    log::warn!("settings from before the Colony layout not copied: {error}");
                }
            }

            let store = Store::open(config_dir.as_deref().map(preferences::file_in));
            let mut preferences = store.get();
            // Every build before the Colony layout created its config folder on
            // first launch, so without one there is no old webview profile and
            // no theme in it to import: start on the new profile straight away.
            if !preferences.legacy_storage_imported && legacy_dir.is_some_and(|dir| !dir.exists()) {
                preferences.legacy_storage_imported = true;
                if let Err(error) = store.set(preferences.clone()) {
                    log::warn!("preferences not saved: {error}");
                }
            }
            app.manage(store);

            let window = window_state::open_main_window(app, &preferences)?;
            window_state::attach(&window, config_dir.as_deref());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::evaluate_expression,
            commands::get_app_info,
            preferences::get_preferences,
            preferences::set_preferences,
        ])
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
