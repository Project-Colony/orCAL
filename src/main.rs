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

use preferences::{Preferences, Store};
use tauri::{App, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

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

            let window = open_main_window(app, &preferences)?;
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

/// Opens the main window, described in tauri.conf.json (where it is not
/// created at startup), on the layout chosen in Preferences.
fn open_main_window(app: &App, preferences: &Preferences) -> tauri::Result<WebviewWindow> {
    let mut config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .expect("tauri.conf.json describes the main window");
    let layout = preferences.startup_layout;
    config.url = WebviewUrl::App(layout.page().into());
    (config.width, config.height) = layout.size();

    let mut builder = WebviewWindowBuilder::from_config(app.handle(), &config)?;
    // The webview profile moves to Colony/orCAL only once the theme the old
    // builds kept in its localStorage has been imported (common/prefs.js):
    // until then the old profile, Tauri's default, is the only place it is.
    // macOS WKWebView has no data directory and keeps its default location.
    if preferences.legacy_storage_imported {
        match paths::data_dir() {
            Ok(data) => builder = builder.data_directory(data.join("webview")),
            Err(error) => log::warn!("the webview keeps its default profile: {error}"),
        }
    }
    let window = builder.build()?;
    // A floating window manager can map the window before the builder's size
    // reaches it (XWayland only saw a 10x10 hint) and pick its own size;
    // asking again once the window exists gives the layout's size.
    let (width, height) = layout.size();
    if let Err(error) = window.set_size(LogicalSize::new(width, height)) {
        log::warn!("the window keeps the size the window manager chose: {error}");
    }
    Ok(window)
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
