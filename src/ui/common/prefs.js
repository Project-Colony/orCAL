// preferences.json, read and written through the Rust side (src/preferences.rs)
// and shared by both layouts. Every change is saved as it is made.
//
// Builds before 0.4 kept the theme in the webview's localStorage. The first
// launch after upgrading still opens that old webview profile, imports the
// theme here once, and from the next launch orCAL uses a fresh profile in its
// cache folder, or in memory on macOS (src/window_state.rs). localStorage is
// not used for anything else.

const invoke = window.__TAURI__?.core?.invoke ?? null;

// The Rust defaults. Outside Tauri (the pages opened in a browser) changes
// last for the page only.
let current = {
  selected_theme: "catppuccin",
  selected_variant: "latte",
  auto_accent: false,
  font_size: "default",
  animations: true,
  high_contrast: false,
  dyslexia_font: false,
  reduce_motion: false,
  text_size_a11y: "default",
  startup_layout: "phone",
  legacy_storage_imported: true,
};

/** The preferences in effect. */
export const preferences = () => current;

/**
 * Apply `changes` and save them. The saved object comes back with anything
 * Rust did not accept reset (an unknown theme becomes Gruvbox dark). A failed
 * save keeps the change for this session. A key set to undefined is left out
 * of the JSON, which is how an option such as `locale` is cleared.
 */
export const updatePreferences = async (changes) => {
  current = { ...current, ...changes };
  if (invoke) {
    try {
      current = await invoke("set_preferences", { preferences: current });
    } catch (error) {
      console.warn("orCAL preferences could not be saved", error);
    }
  }
  return current;
};

const legacyTheme = () => {
  try {
    const value = localStorage.getItem("orcal-theme");
    return value === "latte" || value === "mocha" ? value : null;
  } catch {
    return null;
  }
};

/** Read preferences.json, importing the old localStorage theme once. */
export const loadPreferences = async () => {
  if (!invoke) {
    return current;
  }
  try {
    current = await invoke("get_preferences");
  } catch (error) {
    console.warn("orCAL preferences could not be loaded", error);
    return current;
  }
  if (!current.legacy_storage_imported) {
    // If this save fails, the flag stays unset on disk and the import runs
    // again next launch, in the same old profile.
    const variant = legacyTheme();
    await updatePreferences({
      ...(variant && { selected_theme: "catppuccin", selected_variant: variant }),
      legacy_storage_imported: true,
    });
  }
  return current;
};
