// preferences.json, read and written through the Rust side (src/preferences.rs)
// and shared by both layouts. Every change is saved as it is made.
//
// Builds before 0.4 kept the theme in the webview's localStorage. The first
// launch after upgrading still opens that old webview profile, imports the
// theme here once, and from the next launch orCAL uses a fresh profile under
// Colony/orCAL (src/main.rs). localStorage is not used for anything else.
const orcalPreferences = (() => {
  const invoke = window.__TAURI__?.core?.invoke ?? null;

  // The Rust defaults. Outside Tauri (the pages opened in a browser) changes
  // last for the page only.
  let current = {
    selected_theme: "catppuccin",
    selected_variant: "latte",
    legacy_storage_imported: true,
  };

  // Apply `changes` and save them; the saved object comes back with anything
  // Rust did not accept reset. A failed save keeps the change for this session.
  const update = async (changes) => {
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

  const load = async () => {
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
      await update({
        ...(variant && { selected_theme: "catppuccin", selected_variant: variant }),
        legacy_storage_imported: true,
      });
    }
    return current;
  };

  return {
    load,
    update,
    get current() {
      return current;
    },
  };
})();
