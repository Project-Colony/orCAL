//! Bringing over what orCAL kept before the Colony layout.
//!
//! Builds before 0.4 kept `window-position.json` in Tauri's app config folder,
//! `<config_dir>/com.orcal.app/orcal/` (Roaming on Windows), and the theme in
//! the webview's localStorage. The window position is copied here; the theme is
//! imported by the interface (`common/prefs.js`), the only code that can read
//! that storage.
//!
//! Per Project-Colony-Resources `design/filesystem.md`: the old folder is never
//! deleted, and a failed copy leaves orCAL on its defaults, never stuck.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{paths, window_state};

/// Tauri's bundle identifier, the folder name of the pre-Colony layout.
const LEGACY_IDENTIFIER: &str = "com.orcal.app";

/// Written to the config folder once the copy is done or found unnecessary.
const MARKER: &str = ".migrated";

/// Where the pre-Colony builds kept their config: `dirs::config_dir()`, as
/// Tauri's `app_config_dir` resolved it.
pub fn legacy_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(LEGACY_IDENTIFIER))
}

/// Copy the window position from `legacy` to `config` unless it was done
/// before or `config` already has one, then mark the migration done.
pub fn run(legacy: &Path, config: &Path) -> io::Result<()> {
    let marker = config.join(MARKER);
    if marker.exists() {
        return Ok(());
    }
    let old = legacy.join("orcal").join(window_state::FILE);
    let new = config.join(window_state::FILE);
    if old.is_file() && !new.exists() {
        paths::write_atomic(&new, &fs::read(&old)?)?;
        log::info!("copied {} to {}", old.display(), new.display());
    }
    fs::write(
        marker,
        format!(
            "Settings from before the Colony layout were looked for in {}.\n",
            legacy.display()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::tests::temp_dir;

    fn legacy_with_position(root: &Path, contents: &str) -> PathBuf {
        let legacy = root.join(LEGACY_IDENTIFIER);
        fs::create_dir_all(legacy.join("orcal")).unwrap();
        fs::write(legacy.join("orcal").join(window_state::FILE), contents).unwrap();
        legacy
    }

    #[test]
    fn copies_the_window_position_once_and_keeps_the_old_file() {
        let root = temp_dir("migrate-copy");
        let legacy = legacy_with_position(&root, r#"{"x":10,"y":20}"#);
        let config = root.join("Colony").join("orCAL");
        fs::create_dir_all(&config).unwrap();

        run(&legacy, &config).unwrap();
        let new = config.join(window_state::FILE);
        assert_eq!(fs::read_to_string(&new).unwrap(), r#"{"x":10,"y":20}"#);
        assert!(legacy.join("orcal").join(window_state::FILE).exists());
        assert!(config.join(MARKER).exists());

        // The marker skips it from then on, even if the old file changes.
        fs::write(
            legacy.join("orcal").join(window_state::FILE),
            r#"{"x":0,"y":0}"#,
        )
        .unwrap();
        fs::remove_file(&new).unwrap();
        run(&legacy, &config).unwrap();
        assert!(!new.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn never_overwrites_a_position_already_in_the_new_folder() {
        let root = temp_dir("migrate-existing");
        let legacy = legacy_with_position(&root, r#"{"x":10,"y":20}"#);
        let config = root.join("new");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join(window_state::FILE), r#"{"x":5,"y":5}"#).unwrap();

        run(&legacy, &config).unwrap();
        assert_eq!(
            fs::read_to_string(config.join(window_state::FILE)).unwrap(),
            r#"{"x":5,"y":5}"#
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_fresh_install_just_marks_it_done() {
        let root = temp_dir("migrate-fresh");
        let config = root.join("new");
        fs::create_dir_all(&config).unwrap();

        run(&root.join(LEGACY_IDENTIFIER), &config).unwrap();
        assert!(config.join(MARKER).exists());
        assert!(!config.join(window_state::FILE).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_failed_copy_reports_and_does_not_mark() {
        let root = temp_dir("migrate-fail");
        let legacy = legacy_with_position(&root, "{}");
        // No config folder: the copy cannot be written.
        let config = root.join("missing");

        assert!(run(&legacy, &config).is_err());
        assert!(!config.join(MARKER).exists());
        assert!(legacy.join("orcal").join(window_state::FILE).exists());
        fs::remove_dir_all(root).unwrap();
    }
}
