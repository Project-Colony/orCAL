//! Remembers where the main window was, and puts it back there on the next
//! start when that spot is still on a connected monitor.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager, Monitor, PhysicalPosition, Position, WebviewWindow, WindowEvent};

#[derive(Deserialize, Serialize)]
struct WindowPosition {
    x: i32,
    y: i32,
}

/// Moves `window` to its saved position, or centres it when that position is
/// off every monitor, then saves the position whenever the window moves or
/// closes.
pub fn attach(app: &AppHandle, window: &WebviewWindow) {
    if let Some(position) = load_window_position(app) {
        if position_within_available_monitors(window, position) {
            let _ = window.set_position(Position::Physical(position));
        } else {
            reset_invalid_window_position(app, window);
        }
    }
    let app = app.clone();
    let tracked = window.clone();
    window.on_window_event(move |event| {
        if matches!(
            event,
            WindowEvent::CloseRequested { .. } | WindowEvent::Moved(_)
        ) {
            save_window_position(&app, &tracked);
        }
    });
}

fn window_position_path(app: &AppHandle) -> Option<PathBuf> {
    let mut dir = app.path().app_config_dir().ok()?;
    dir.push("orcal");
    let _ = fs::create_dir_all(&dir);
    dir.push("window-position.json");
    Some(dir)
}

fn load_window_position(app: &AppHandle) -> Option<PhysicalPosition<i32>> {
    let path = window_position_path(app)?;
    let contents = fs::read_to_string(path).ok()?;
    let saved: WindowPosition = serde_json::from_str(&contents).ok()?;
    Some(PhysicalPosition::new(saved.x, saved.y))
}

fn save_window_position(app: &AppHandle, window: &WebviewWindow) {
    let Some(path) = window_position_path(app) else {
        return;
    };
    let Ok(position) = window.outer_position() else {
        return;
    };
    let saved = WindowPosition {
        x: position.x,
        y: position.y,
    };
    let Ok(payload) = serde_json::to_string(&saved) else {
        return;
    };
    let _ = fs::write(path, payload);
}

fn position_within_monitor(position: PhysicalPosition<i32>, monitor: &Monitor) -> bool {
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let max_x = monitor_position.x + monitor_size.width as i32;
    let max_y = monitor_position.y + monitor_size.height as i32;

    position.x >= monitor_position.x
        && position.x < max_x
        && position.y >= monitor_position.y
        && position.y < max_y
}

fn position_within_available_monitors(
    window: &WebviewWindow,
    position: PhysicalPosition<i32>,
) -> bool {
    if let Ok(Some(monitor)) = window.current_monitor() {
        if position_within_monitor(position, &monitor) {
            return true;
        }
    }

    if let Ok(monitors) = window.available_monitors() {
        return monitors
            .iter()
            .any(|monitor| position_within_monitor(position, monitor));
    }

    false
}

fn reset_invalid_window_position(app: &AppHandle, window: &WebviewWindow) {
    log::debug!("Saved window position is invalid; resetting to centered.");
    if let Some(path) = window_position_path(app) {
        let _ = fs::remove_file(path);
    }
    let _ = window.center();
}
