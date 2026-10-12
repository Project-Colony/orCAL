//! Remembers where the main window was, and puts it back there on the next
//! start when that spot is still on a connected monitor.
//!
//! The position is written atomically to `<config>/window-position.json`. A
//! drag sends a stream of `Moved` events, so a move is saved once the window
//! has stayed put for [`SETTLE`], and only when it differs from what is on
//! disk; closing saves the final position straight away.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, io, thread};

use serde::{Deserialize, Serialize};
use tauri::{Monitor, PhysicalPosition, Position, WebviewWindow, WindowEvent};

use crate::paths;

/// The file name, in the config folder.
pub const FILE: &str = "window-position.json";

/// How long the window must stay still before a move is saved.
const SETTLE: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
struct WindowPosition {
    x: i32,
    y: i32,
}

impl From<PhysicalPosition<i32>> for WindowPosition {
    fn from(position: PhysicalPosition<i32>) -> Self {
        WindowPosition {
            x: position.x,
            y: position.y,
        }
    }
}

/// Moves `window` to the position saved in `config_dir`, or centres it when
/// that position is off every monitor, then keeps the file up to date as the
/// window moves and closes. Without a config folder the window just opens
/// where the system puts it.
pub fn attach(window: &WebviewWindow, config_dir: Option<&Path>) {
    let Some(path) = config_dir.map(|dir| dir.join(FILE)) else {
        return;
    };
    let mut restored = load(&path);
    if let Some(saved) = restored {
        let position = PhysicalPosition::new(saved.x, saved.y);
        if position_within_available_monitors(window, position) {
            let _ = window.set_position(Position::Physical(position));
        } else {
            log::debug!("Saved window position is off every monitor; centring the window.");
            let _ = fs::remove_file(&path);
            let _ = window.center();
            restored = None;
        }
    }

    let saver = Arc::new(Saver {
        path,
        last: Mutex::new(restored),
    });
    let moves = debounced(SETTLE, {
        let saver = Arc::clone(&saver);
        move |position| saver.save(position)
    });
    let tracked = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(position) => {
            let _ = moves.send(WindowPosition::from(*position));
        }
        WindowEvent::CloseRequested { .. } => {
            if let Ok(position) = tracked.outer_position() {
                saver.save(position.into());
            }
        }
        _ => {}
    });
}

fn load(path: &Path) -> Option<WindowPosition> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Writes the position file, skipping a position already on disk. The lock
/// also keeps the settle thread and the close handler from writing at once.
struct Saver {
    path: PathBuf,
    last: Mutex<Option<WindowPosition>>,
}

impl Saver {
    fn save(&self, position: WindowPosition) {
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        if *last == Some(position) {
            return;
        }
        let written = serde_json::to_vec(&position)
            .map_err(io::Error::other)
            .and_then(|json| paths::write_atomic(&self.path, &json));
        match written {
            Ok(()) => *last = Some(position),
            Err(error) => log::warn!("window position not saved: {error}"),
        }
    }
}

/// A sender whose values reach `apply` only once none has followed for
/// `quiet`, and only the last of them. Dropping the sender applies a pending
/// value and ends the thread.
fn debounced<T: Send + 'static>(quiet: Duration, apply: impl Fn(T) + Send + 'static) -> Sender<T> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut pending = None;
        loop {
            // Nothing pending: sleep until something arrives.
            let next = match pending {
                None => receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
                Some(_) => receiver.recv_timeout(quiet),
            };
            match next {
                Ok(value) => pending = Some(value),
                Err(RecvTimeoutError::Timeout) => apply(pending.take().expect("pending")),
                Err(RecvTimeoutError::Disconnected) => {
                    if let Some(value) = pending {
                        apply(value);
                    }
                    return;
                }
            }
        }
    });
    sender
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::tests::temp_dir;

    #[test]
    fn a_burst_of_moves_applies_only_the_last() {
        let (seen, applied) = mpsc::channel();
        let moves = debounced(Duration::from_secs(60), move |value: i32| {
            seen.send(value).unwrap()
        });
        for value in 1..=5 {
            moves.send(value).unwrap();
        }
        drop(moves);
        assert_eq!(applied.iter().collect::<Vec<_>>(), vec![5]);
    }

    #[test]
    fn a_move_is_applied_once_the_window_settles() {
        let (seen, applied) = mpsc::channel();
        let moves = debounced(Duration::from_millis(10), move |value: i32| {
            seen.send(value).unwrap()
        });
        moves.send(7).unwrap();
        assert_eq!(applied.recv_timeout(Duration::from_secs(10)), Ok(7));
    }

    #[test]
    fn the_saver_writes_a_position_once() {
        let dir = temp_dir("window-saver");
        let path = dir.join(FILE);
        let saver = Saver {
            path: path.clone(),
            last: Mutex::new(None),
        };
        let position = WindowPosition { x: 40, y: -20 };
        saver.save(position);
        assert_eq!(load(&path), Some(position));

        // The same position again does not touch the disk.
        fs::remove_file(&path).unwrap();
        saver.save(position);
        assert!(!path.exists());

        saver.save(WindowPosition { x: 41, y: -20 });
        assert_eq!(load(&path), Some(WindowPosition { x: 41, y: -20 }));
        fs::remove_dir_all(dir).unwrap();
    }
}
