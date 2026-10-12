//! The commands the web view invokes over Tauri's IPC.

use crate::engine;
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
