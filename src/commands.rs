//! The commands the web view invokes over Tauri's IPC.

use crate::{engine, paths};
use serde::Serialize;

#[derive(Serialize)]
pub struct EvalResponse {
    /// What the screen shows, rounded to hide float noise.
    result: String,
    /// The unrounded value, kept for ANS so chained results lose nothing.
    value: f64,
}

/// Evaluates `expression`, with `ANS` standing for `ans`. Errors come back as
/// the engine's stable code (`error_division_by_zero`, ...), which the UI turns
/// into a message.
#[tauri::command]
pub fn evaluate_expression(expression: String, ans: f64) -> Result<EvalResponse, &'static str> {
    let normalized = expression.replace(',', ".");
    let value = engine::evaluate_with(&normalized, ans).map_err(|error| error.code())?;
    Ok(EvalResponse {
        result: engine::format_result(value),
        value,
    })
}

#[derive(Serialize)]
pub struct AppInfo {
    version: &'static str,
    license: &'static str,
    /// Where orCAL keeps its files, for the About page; `None` when the
    /// platform has no such folder.
    config_dir: Option<String>,
    data_dir: Option<String>,
    cache_dir: Option<String>,
}

/// What the About page shows. The folders are located, not created.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    let shown =
        |dir: std::io::Result<std::path::PathBuf>| dir.ok().map(|d| d.display().to_string());
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        license: env!("CARGO_PKG_LICENSE"),
        config_dir: shown(paths::locate::config_dir()),
        data_dir: shown(paths::locate::data_dir()),
        cache_dir: shown(paths::locate::cache_dir()),
    }
}
