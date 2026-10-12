//! orCAL's `preferences.json`, in `<config>/preferences/`.
//!
//! The Appearance and Accessibility part uses the field names and values of
//! colony-ui's `StandardPreferences`, so the file reads like every other Colony
//! program's. orCAL adds its own fields after them.
//!
//! Loading never fails: a field with a bad value falls back to its default and
//! the rest of the file is kept, unknown fields are ignored, and a file that is
//! not a JSON object at all is set aside as `preferences.json.bak` rather than
//! overwritten by the next save.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::paths;

/// Appearance → Typography → Text size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontSize {
    Small,
    #[default]
    Default,
    Large,
}

/// Accessibility → Reading → Text size. Multiplies with [`FontSize`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Small,
    #[default]
    Default,
    Large,
    XLarge,
}

/// The interface language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    En,
    Fr,
}

/// The layout orCAL opens in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// The phone keypad.
    #[default]
    Phone,
    /// The scientific tablet layout.
    Tablet,
}

impl Layout {
    /// The page the window opens, relative to the frontend root.
    pub fn page(self) -> &'static str {
        match self {
            Layout::Phone => "index.html",
            Layout::Tablet => "tablet-mode/index.html",
        }
    }

    /// The window's inner size in logical pixels, as the layout switch in the
    /// interface sets it.
    pub fn size(self) -> (f64, f64) {
        match self {
            Layout::Phone => (360.0, 720.0),
            Layout::Tablet => (980.0, 560.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Preferences {
    /// Theme family key, e.g. `"catppuccin"`.
    pub selected_theme: String,
    /// Theme variant key, e.g. `"latte"`.
    pub selected_variant: String,
    /// Accent override key, e.g. `"violet"`. `None` is the theme's own accent;
    /// "auto" is never stored here, that is `auto_accent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_accent: Option<String>,
    pub auto_accent: bool,
    pub font_size: FontSize,
    pub animations: bool,
    pub high_contrast: bool,
    pub dyslexia_font: bool,
    pub reduce_motion: bool,
    pub text_size_a11y: TextSize,
    /// `None` follows the system language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<Locale>,
    pub startup_layout: Layout,
    /// Whether the theme that builds before 0.4 kept in the webview's
    /// localStorage has been read into this file. Until it has, orCAL keeps
    /// opening the old webview profile, the only place that value exists.
    pub legacy_storage_imported: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            // orCAL has always opened on Catppuccin Latte; an unknown theme
            // still falls back to Gruvbox dark, as everywhere in Colony.
            selected_theme: "catppuccin".to_string(),
            selected_variant: "latte".to_string(),
            selected_accent: None,
            auto_accent: false,
            font_size: FontSize::Default,
            animations: true,
            high_contrast: false,
            dyslexia_font: false,
            reduce_motion: false,
            text_size_a11y: TextSize::Default,
            locale: None,
            startup_layout: Layout::Phone,
            legacy_storage_imported: false,
        }
    }
}

impl Preferences {
    /// Every field of `value` that parses, laid over the defaults.
    fn from_object(object: &Map<String, Value>) -> Self {
        fn take<T: DeserializeOwned>(object: &Map<String, Value>, key: &str, field: &mut T) {
            if let Some(value) = object.get(key).and_then(|v| T::deserialize(v).ok()) {
                *field = value;
            }
        }

        let mut p = Preferences::default();
        take(object, "selected_theme", &mut p.selected_theme);
        take(object, "selected_variant", &mut p.selected_variant);
        take(object, "selected_accent", &mut p.selected_accent);
        take(object, "auto_accent", &mut p.auto_accent);
        take(object, "font_size", &mut p.font_size);
        take(object, "animations", &mut p.animations);
        take(object, "high_contrast", &mut p.high_contrast);
        take(object, "dyslexia_font", &mut p.dyslexia_font);
        take(object, "reduce_motion", &mut p.reduce_motion);
        take(object, "text_size_a11y", &mut p.text_size_a11y);
        take(object, "locale", &mut p.locale);
        take(object, "startup_layout", &mut p.startup_layout);
        take(
            object,
            "legacy_storage_imported",
            &mut p.legacy_storage_imported,
        );
        p
    }
}

/// Lenient: the interface sends the whole object back, and a stale or
/// hand-edited value must not reject the rest.
impl<'de> Deserialize<'de> for Preferences {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Value::deserialize(deserializer)? {
            Value::Object(object) => Preferences::from_object(&object),
            _ => Preferences::default(),
        })
    }
}

/// `preferences.json` inside the config directory.
pub fn file_in(config_dir: &Path) -> PathBuf {
    config_dir.join("preferences").join("preferences.json")
}

/// Read `path`: the defaults when it does not exist, and when it cannot be
/// read as a JSON object, after moving it aside to `preferences.json.bak`.
pub fn load(path: &Path) -> Preferences {
    let object = match fs::read_to_string(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Preferences::default(),
        Err(error) => Err(error.to_string()),
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(object)) => Ok(object),
            Ok(_) => Err("not a JSON object".to_string()),
            Err(error) => Err(error.to_string()),
        },
    };
    match object {
        Ok(object) => Preferences::from_object(&object),
        Err(reason) => {
            let backup = path.with_extension("json.bak");
            match fs::rename(path, &backup) {
                Ok(()) => log::warn!(
                    "{} is unreadable ({reason}); kept as {} and started from the defaults",
                    path.display(),
                    backup.display()
                ),
                Err(error) => log::warn!(
                    "{} is unreadable ({reason}) and could not be set aside: {error}",
                    path.display()
                ),
            }
            Preferences::default()
        }
    }
}

/// Write `preferences` to `path` atomically, creating its directory.
pub fn save(path: &Path, preferences: &Preferences) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(preferences).map_err(io::Error::other)?;
    paths::write_atomic(path, &json)
}

/// The preferences in effect, shared by the setup code and the commands.
pub struct Store {
    /// `None` when the platform has no config directory: changes then last
    /// for the session only.
    path: Option<PathBuf>,
    current: Mutex<Preferences>,
}

impl Store {
    pub fn open(path: Option<PathBuf>) -> Self {
        let current = path.as_deref().map(load).unwrap_or_default();
        Store {
            path,
            current: Mutex::new(current),
        }
    }

    pub fn get(&self) -> Preferences {
        self.current
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Save `preferences` and make them current. On a write error nothing
    /// changes, so what is in effect is always what the file says.
    pub fn set(&self, preferences: Preferences) -> io::Result<Preferences> {
        let mut current = self.current.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(path) = &self.path {
            save(path, &preferences)?;
        }
        *current = preferences.clone();
        Ok(preferences)
    }
}

#[tauri::command]
pub fn get_preferences(store: tauri::State<'_, Store>) -> Preferences {
    store.get()
}

/// Saves the whole object and returns it as stored, with every value the
/// interface sent that orCAL does not know replaced by its default.
#[tauri::command]
pub fn set_preferences(
    store: tauri::State<'_, Store>,
    preferences: Preferences,
) -> Result<Preferences, String> {
    store.set(preferences).map_err(|error| {
        log::warn!("preferences not saved: {error}");
        error.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::tests::temp_dir;

    fn parse(json: &str) -> Preferences {
        serde_json::from_str(json).expect("lenient parsing never fails on a value")
    }

    #[test]
    fn writes_the_colony_field_names_and_values() {
        let p = Preferences {
            font_size: FontSize::Large,
            text_size_a11y: TextSize::XLarge,
            locale: Some(Locale::Fr),
            startup_layout: Layout::Tablet,
            ..Preferences::default()
        };
        let json: Value = serde_json::to_value(&p).unwrap();
        assert_eq!(json["selected_theme"], "catppuccin");
        assert_eq!(json["selected_variant"], "latte");
        assert_eq!(json["font_size"], "large");
        assert_eq!(json["text_size_a11y"], "xlarge");
        assert_eq!(json["locale"], "fr");
        assert_eq!(json["startup_layout"], "tablet");
        // Unset options are left out, not written as null.
        assert!(json.get("selected_accent").is_none());
        assert!(serde_json::to_value(Preferences::default())
            .unwrap()
            .get("locale")
            .is_none());
        assert_eq!(parse(&json.to_string()), p);
    }

    #[test]
    fn reads_what_colony_ui_writes() {
        let p = parse(
            r#"{
                "selected_theme": "nord", "selected_variant": "dark",
                "selected_accent": "violet", "auto_accent": true,
                "font_size": "small", "animations": false, "high_contrast": true,
                "dyslexia_font": true, "reduce_motion": true, "text_size_a11y": "large",
                "keyboard_nav": true
            }"#,
        );
        assert_eq!(
            (p.selected_theme.as_str(), p.selected_variant.as_str()),
            ("nord", "dark")
        );
        assert_eq!(p.selected_accent.as_deref(), Some("violet"));
        assert!(p.auto_accent && p.high_contrast && p.dyslexia_font && p.reduce_motion);
        assert!(!p.animations);
        assert_eq!(p.font_size, FontSize::Small);
        assert_eq!(p.text_size_a11y, TextSize::Large);
    }

    #[test]
    fn a_bad_value_falls_back_and_keeps_the_rest() {
        let p = parse(
            r#"{
                "selected_theme": 42, "selected_accent": ["violet"], "font_size": "enormous",
                "text_size_a11y": null, "locale": "de", "startup_layout": "watch",
                "legacy_storage_imported": "yes", "animations": false, "reduce_motion": true
            }"#,
        );
        let d = Preferences::default();
        assert_eq!(p.selected_theme, d.selected_theme);
        assert_eq!(p.selected_accent, None);
        assert_eq!(p.font_size, FontSize::Default);
        assert_eq!(p.text_size_a11y, TextSize::Default);
        assert_eq!(p.locale, None);
        assert_eq!(p.startup_layout, Layout::Phone);
        assert!(!p.legacy_storage_imported);
        assert!(!p.animations && p.reduce_motion);
        assert_eq!(parse("[1, 2]"), d);
    }

    #[test]
    fn a_missing_file_is_the_defaults() {
        let dir = temp_dir("prefs-missing");
        assert_eq!(load(&file_in(&dir)), Preferences::default());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn saves_atomically_and_loads_back() {
        let dir = temp_dir("prefs-roundtrip");
        let path = file_in(&dir);
        let p = Preferences {
            selected_variant: "mocha".to_string(),
            legacy_storage_imported: true,
            ..Preferences::default()
        };
        save(&path, &p).unwrap();
        assert_eq!(load(&path), p);
        assert!(!path.with_extension("json.tmp").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_corrupt_file_is_kept_aside_not_overwritten() {
        let dir = temp_dir("prefs-corrupt");
        let path = file_in(&dir);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{\"selected_theme\": \"nor").unwrap();
        assert_eq!(load(&path), Preferences::default());
        assert!(!path.exists());
        let backup = path.with_extension("json.bak");
        assert_eq!(
            fs::read_to_string(backup).unwrap(),
            "{\"selected_theme\": \"nor"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_store_only_changes_once_the_file_is_written() {
        let dir = temp_dir("prefs-store");
        let path = file_in(&dir);
        let store = Store::open(Some(path.clone()));
        let mut p = store.get();
        p.high_contrast = true;
        assert_eq!(store.set(p.clone()).unwrap(), p);
        assert_eq!(load(&path), p);

        // A file where the preferences directory should be: the write fails,
        // nothing changes.
        let blocked = dir.join("blocked");
        fs::create_dir_all(&blocked).unwrap();
        fs::write(blocked.join("preferences"), "").unwrap();
        let store = Store::open(Some(file_in(&blocked)));
        let before = store.get();
        assert!(store.set(p).is_err());
        assert_eq!(store.get(), before);
        fs::remove_dir_all(dir).unwrap();
    }
}
