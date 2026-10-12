//! Checks on the interface strings in `src/ui/i18n/`. The strings themselves
//! are read by the web view (src/ui/common/i18n.js), on top of the labels every
//! Colony program shares, which come with the vendored `themes.json`; this
//! module only holds them to Colony's rules, so a missing translation fails
//! `cargo test`.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

const EN: &str = include_str!("ui/i18n/en.json");
const FR: &str = include_str!("ui/i18n/fr.json");
const CATALOG: &str = include_str!("ui/vendor/colony/themes.json");

/// Every file that names a string by its key.
const SOURCES: [(&str, &str); 6] = [
    ("index.html", include_str!("ui/index.html")),
    (
        "tablet-mode/index.html",
        include_str!("ui/tablet-mode/index.html"),
    ),
    ("app.js", include_str!("ui/app.js")),
    ("tablet-mode/app.js", include_str!("ui/tablet-mode/app.js")),
    ("common/chrome.js", include_str!("ui/common/chrome.js")),
    (
        "common/preferences.js",
        include_str!("ui/common/preferences.js"),
    ),
];

fn table(json: &str) -> Map<String, Value> {
    match serde_json::from_str(json) {
        Ok(Value::Object(table)) => table,
        other => panic!("a locale file is a JSON object of strings, got {other:?}"),
    }
}

/// The shared English labels of themes.json, which common/i18n.js lays
/// orCAL's own strings over.
fn shared_english() -> Map<String, Value> {
    let catalog: Value = serde_json::from_str(CATALOG).expect("themes.json is JSON");
    match &catalog["labels"]["en"] {
        Value::Object(labels) if !labels.is_empty() => labels.clone(),
        other => panic!("themes.json has no English labels: {other:?}"),
    }
}

/// The keys `source` uses: data-i18n* attributes, the `t("…")`,
/// `showStatusMessage("…")` and `showToast("…")` calls, `ariaLabel: "…"` and
/// `state.error = "…"`, and every quoted `"settings_…"` or `"preferences_…"`
/// literal, which covers the tables keys are looked up from.
fn used_keys(source: &str) -> BTreeSet<String> {
    // (what precedes the key, the part of it that belongs to the key)
    const OPENERS: [(&str, &str); 10] = [
        ("data-i18n=\"", ""),
        ("data-i18n-title=\"", ""),
        ("data-i18n-aria-label=\"", ""),
        ("t(\"", ""),
        ("showStatusMessage(\"", ""),
        ("showToast(\"", ""),
        ("ariaLabel: \"", ""),
        ("state.error = \"", ""),
        ("\"settings_", "settings_"),
        ("\"preferences_", "preferences_"),
    ];
    let mut keys = BTreeSet::new();
    for (opener, kept) in OPENERS {
        for (start, _) in source.match_indices(opener) {
            // `t("` must be the function t, not the end of `Event("`.
            let glued = source[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || "_$.".contains(c));
            if glued {
                continue;
            }
            let rest = &source[start + opener.len()..];
            let end = rest.find('"').expect("a quoted key ends");
            keys.insert(format!("{kept}{}", &rest[..end]));
        }
    }
    keys
}

#[test]
fn english_and_french_have_identical_key_sets() {
    let en: BTreeSet<_> = table(EN).keys().cloned().collect();
    let fr: BTreeSet<_> = table(FR).keys().cloned().collect();
    assert_eq!(
        en.difference(&fr).collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "in en.json only"
    );
    assert_eq!(
        fr.difference(&en).collect::<Vec<_>>(),
        Vec::<&String>::new(),
        "in fr.json only"
    );
}

#[test]
fn no_string_is_empty() {
    for (name, json) in [("en", EN), ("fr", FR)] {
        for (key, value) in table(json) {
            assert!(
                value.as_str().is_some_and(|s| !s.trim().is_empty()),
                "{name}.json: {key} is empty or not a string"
            );
        }
    }
}

#[test]
fn every_key_the_interface_uses_exists() {
    let mut en = shared_english();
    en.extend(table(EN));
    for (file, source) in SOURCES {
        for key in used_keys(source) {
            assert!(
                en.contains_key(&key),
                "{file} uses {key}, in neither en.json nor themes.json"
            );
        }
    }
    // Built from parts in common/preferences.js.
    for size in ["small", "default", "large", "xlarge"] {
        let key = format!("settings_font_size_{size}");
        assert!(en.contains_key(&key), "{key} is missing");
    }
}

#[test]
fn every_engine_error_has_a_string() {
    use crate::engine::ParseError::*;
    let en = table(EN);
    // A new variant fails to compile here until it is listed.
    for error in [
        EmptyInput,
        InvalidToken('$'),
        UnexpectedEnd,
        DivisionByZero,
        TooDeep,
        FactorialDomain,
        FactorialTooLarge,
        InvalidResult,
    ] {
        let listed = match error {
            EmptyInput | InvalidToken(_) | UnexpectedEnd | DivisionByZero | TooDeep
            | FactorialDomain | FactorialTooLarge | InvalidResult => error.code(),
        };
        assert!(en.contains_key(listed), "{listed} is missing from en.json");
    }
}

/// The shared strings come from the vendored themes.json, in both languages: a
/// copy here would be a second source to drift from it.
#[test]
fn shared_labels_are_not_redefined() {
    let shared = shared_english();
    for (name, json) in [("en", EN), ("fr", FR)] {
        for key in table(json).keys() {
            assert!(
                !shared.contains_key(key),
                "{name}.json redefines {key}, which belongs to themes.json"
            );
        }
    }
}
