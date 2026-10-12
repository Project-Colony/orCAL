// The Preferences page, shared by both layouts. It replaces the calculator
// inside the window (design/settings-page.md): the orCAL name in the status bar
// opens it and closes it again, and so does its Close button.
//
// Categories, in Colony's order: General, Appearance, Accessibility, then
// About. There is no Save button: every change is saved as it is made.

import { preferences, updatePreferences } from "./prefs.js";
import { applyPreferences, contrastOn } from "./theme.js";
import { label, setLocale, t } from "./i18n.js";

// Nerd Font codepoints (colony_ui::widgets::icons).
const CHEVRON_RIGHT = "\u{f054}";
const CHEVRON_DOWN = "\u{f078}";
const CHECK = "\u{f00c}";
const CLOSE = "\u{f00d}";

const APP_FONT = "JetBrainsMono Nerd Font";
const DYSLEXIA_FONT = "OpenDyslexic";

const invoke = window.__TAURI__?.core?.invoke ?? null;

let page = null;
let brand = null;
let nav = null;
let content = null;
let toast = null;
let toastTimer = null;
let category = 0;
let catalog = null;
// Open sections, kept while orCAL runs. Every section starts closed.
const expanded = new Set();
// Each control's way of showing the current value after a change.
let syncs = [];
let nextId = 0;

/** A new element with `props` set (attributes for data-*, aria-* and role). */
const h = (tag, props = {}, ...children) => {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (key === "class") {
      node.className = value;
    } else if (/^(data-|aria-|role$)/u.test(key)) {
      node.setAttribute(key, value);
    } else {
      node[key] = value;
    }
  }
  node.append(...children.filter((child) => child !== null && child !== undefined));
  return node;
};

const uid = (prefix) => `${prefix}-${(nextId += 1)}`;

const glyph = (codepoint, className = "") =>
  h("span", { class: `glyph ${className}`.trim(), "aria-hidden": "true" }, codepoint);

/** themes.json, the vendored Colony theme and accent catalog. */
const loadCatalog = async () => {
  if (!catalog) {
    try {
      const response = await fetch(new URL("../vendor/colony/themes.json", import.meta.url));
      catalog = await response.json();
    } catch (error) {
      console.warn("orCAL could not read the theme catalog", error);
      catalog = { families: [], accents: [] };
    }
  }
  return catalog;
};

/** Save `changes`, put them into effect and show them on the page. */
const change = async (changes) => {
  const prefs = await updatePreferences(changes);
  applyPreferences(prefs);
  if ("locale" in changes) {
    // The page redraws in the new language (the "orcal:locale" event).
    await setLocale(prefs.locale);
    content.querySelector("[data-control='locale']")?.focus();
  } else {
    syncs.forEach((sync) => sync(prefs));
  }
};

const showToast = (key) => {
  toast.querySelector(".toast-message").textContent = t(key);
  toast.querySelector(".toast-dismiss").setAttribute("aria-label", t("preferences_toast_dismiss"));
  toast.hidden = false;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toast.hidden = true;
  }, 4_000);
};

// Building blocks, after colony-ui's widgets.

/** A title and its one-line description, the control on the right. */
const settingRow = (titleKey, descKey, control) => {
  const titleId = uid("pref-title");
  const descId = uid("pref-desc");
  control.setAttribute("aria-labelledby", titleId);
  control.setAttribute("aria-describedby", descId);
  return h(
    "div",
    { class: "pref-row" },
    h(
      "div",
      { class: "pref-row-text" },
      h("span", { class: "pref-row-title", id: titleId }, t(titleKey)),
      h("span", { class: "pref-row-desc", id: descId }, t(descKey)),
    ),
    control,
  );
};

/** An on/off setting: `get` reads it from the preferences, `set` builds the change. */
const toggle = (titleKey, descKey, get, set) => {
  const button = h(
    "button",
    { type: "button", class: "pref-switch", role: "switch" },
    h("span", { class: "pref-switch-knob" }),
  );
  const sync = (prefs) => button.setAttribute("aria-checked", String(Boolean(get(prefs))));
  sync(preferences());
  syncs.push(sync);
  button.addEventListener("click", () => change(set(!get(preferences()))));
  return settingRow(titleKey, descKey, button);
};

/** A choice among `options`, [value, text] pairs. */
const dropdown = (titleKey, descKey, options, get, set, control) => {
  const select = h(
    "select",
    { class: "pref-select", "data-control": control },
    ...options.map(([value, text]) => h("option", { value }, text)),
  );
  const sync = (prefs) => {
    select.value = get(prefs);
  };
  sync(preferences());
  syncs.push(sync);
  select.addEventListener("change", () => change(set(select.value)));
  return settingRow(titleKey, descKey, select);
};

/** A collapsible section: a flat header row, then its description and controls. */
const section = (key, titleKey, descKey, ...controls) => {
  const bodyId = uid("pref-section");
  const chevron = glyph("", "pref-chevron");
  const header = h(
    "button",
    { type: "button", class: "pref-section-header", "aria-controls": bodyId },
    h("span", { class: "pref-section-title" }, t(titleKey)),
    chevron,
  );
  const body = h(
    "div",
    { class: "pref-section-body", id: bodyId },
    h("p", { class: "pref-section-desc" }, t(descKey)),
    ...controls,
  );
  const show = () => {
    const open = expanded.has(key);
    header.setAttribute("aria-expanded", String(open));
    chevron.textContent = open ? CHEVRON_DOWN : CHEVRON_RIGHT;
    body.hidden = !open;
  };
  header.addEventListener("click", () => {
    if (!expanded.delete(key)) {
      expanded.add(key);
    }
    show();
  });
  show();
  return h("section", { class: "pref-section" }, header, body);
};

const heading = (titleKey, descKey) => [
  h("h2", { class: "pref-heading" }, t(titleKey)),
  h("p", { class: "pref-heading-desc" }, t(descKey)),
];

// General

const general = () => [
  ...heading("settings_cat_general", "settings_general_desc"),
  section(
    "language",
    "preferences_section_language",
    "preferences_language_desc",
    dropdown(
      "preferences_language",
      "preferences_language_hint",
      [
        ["system", t("preferences_language_system")],
        ["en", t("preferences_language_en")],
        ["fr", t("preferences_language_fr")],
      ],
      (prefs) => prefs.locale ?? "system",
      // No value follows the system language.
      (value) => ({ locale: value === "system" ? undefined : value }),
      "locale",
    ),
  ),
  section(
    "startup",
    "preferences_section_startup",
    "preferences_startup_desc",
    dropdown(
      "preferences_startup_layout",
      "preferences_startup_layout_desc",
      [
        ["phone", t("preferences_layout_phone")],
        ["tablet", t("preferences_layout_tablet")],
      ],
      (prefs) => prefs.startup_layout,
      (value) => ({ startup_layout: value }),
      "startup_layout",
    ),
  ),
];

// Appearance

const themePicker = () => {
  const families = [...catalog.families].sort((a, b) => a.order - b.order);
  return h(
    "div",
    { class: "theme-picker" },
    ...families.map((family) => {
      const familyName = label(family.label);
      const cards = family.variants.map((variant) => {
        const variantName = label(variant.label);
        const card = h(
          "button",
          {
            type: "button",
            class: "theme-card",
            "aria-label": `${familyName} ${variantName}`,
            "data-on": contrastOn(variant.swatch.bg),
          },
          h("span", { class: "theme-card-bar" }),
          h("span", { class: "theme-card-name" }, variantName),
          glyph(CHECK, "theme-card-check"),
        );
        card.style.setProperty("--swatch-bg", variant.swatch.bg);
        card.style.setProperty("--swatch-accent", variant.swatch.accent);
        const sync = (prefs) =>
          card.setAttribute(
            "aria-pressed",
            String(prefs.selected_theme === family.key && prefs.selected_variant === variant.key),
          );
        sync(preferences());
        syncs.push(sync);
        card.addEventListener("click", async () => {
          await change({ selected_theme: family.key, selected_variant: variant.key });
          showToast("theme_applied");
        });
        return card;
      });
      return h(
        "div",
        { class: "theme-family" },
        h(
          "div",
          { class: "theme-family-name" },
          family.icon ? glyph(String.fromCodePoint(Number.parseInt(family.icon, 16))) : null,
          h("span", {}, familyName),
        ),
        h("div", { class: "theme-cards" }, ...cards),
      );
    }),
  );
};

const accentPicker = () =>
  h(
    "div",
    { class: "accent-picker" },
    ...catalog.accents.map((accent) => {
      const swatch = h(
        "button",
        {
          type: "button",
          class: "accent-swatch",
          "aria-label": label(accent.label),
          title: label(accent.label),
          "data-on": contrastOn(accent.color),
        },
        glyph(CHECK),
      );
      swatch.style.setProperty("--swatch", accent.color);
      const sync = (prefs) =>
        swatch.setAttribute(
          "aria-pressed",
          String(!prefs.auto_accent && prefs.selected_accent === accent.key),
        );
      sync(preferences());
      syncs.push(sync);
      swatch.addEventListener("click", () =>
        change({ selected_accent: accent.key, auto_accent: false }),
      );
      return swatch;
    }),
  );

const fontFamily = () => {
  const name = h("span", { class: "pref-value" });
  const sync = (prefs) => {
    name.textContent = prefs.dyslexia_font ? DYSLEXIA_FONT : APP_FONT;
  };
  sync(preferences());
  syncs.push(sync);
  return settingRow("settings_font_family", "settings_font_family_desc", name);
};

const sizeOptions = (sizes) => sizes.map((size) => [size, t(`settings_font_size_${size}`)]);

const preview = () => {
  const summary = h("span", { class: "preview-summary" });
  const sync = (prefs) => {
    const family = catalog.families.find((f) => f.key === prefs.selected_theme);
    const variant = family?.variants.find((v) => v.key === prefs.selected_variant);
    const theme = family ? `${label(family.label)} · ${label(variant?.label)}` : prefs.selected_theme;
    const accent = catalog.accents.find(
      (a) => !prefs.auto_accent && a.key === prefs.selected_accent,
    );
    summary.textContent = `${theme} · ${accent ? label(accent.label) : t("settings_default_accent")}`;
  };
  sync(preferences());
  syncs.push(sync);
  // A few real keys, drawn by the same rules as the keypad.
  const keys = [
    ["7", ""],
    ["C", "key-utility"],
    ["×", "key-operator"],
    ["⌫", "key-backspace"],
  ].map(([text, kind]) => h("span", { class: `key preview-key ${kind}`.trim() }, text));
  return h(
    "div",
    { class: "preview-card" },
    h("span", { class: "preview-title" }, t("settings_preview_card")),
    h("span", { class: "preview-body" }, t("settings_preview_body")),
    summary,
    h("span", { class: "preview-button" }, t("settings_preview_button")),
    h("div", { class: "preview-keys", "aria-hidden": "true" }, ...keys),
  );
};

const appearance = () => [
  ...heading("settings_cat_appearance", "settings_appearance_desc"),
  section("theme", "settings_section_theme", "settings_theme_desc", themePicker()),
  section(
    "colors",
    "settings_section_colors",
    "settings_colors_desc",
    accentPicker(),
    toggle(
      "settings_auto_accent",
      "settings_auto_accent_desc",
      (prefs) => prefs.auto_accent,
      (on) => ({ auto_accent: on }),
    ),
  ),
  section(
    "typography",
    "settings_section_typography",
    "settings_typography_desc",
    fontFamily(),
    dropdown(
      "settings_font_size",
      "settings_font_size_desc",
      sizeOptions(["small", "default", "large"]),
      (prefs) => prefs.font_size,
      (value) => ({ font_size: value }),
      "font_size",
    ),
  ),
  section(
    "effects",
    "settings_section_effects",
    "settings_effects_desc",
    toggle(
      "settings_animations",
      "settings_animations_desc",
      (prefs) => prefs.animations,
      (on) => ({ animations: on }),
    ),
  ),
  section("preview", "settings_section_preview", "settings_preview_desc", preview()),
];

// Accessibility

const accessibility = () => [
  ...heading("settings_cat_accessibility", "settings_accessibility_desc"),
  section(
    "vision",
    "settings_section_vision",
    "settings_vision_desc",
    toggle(
      "settings_high_contrast",
      "settings_high_contrast_desc",
      (prefs) => prefs.high_contrast,
      (on) => ({ high_contrast: on }),
    ),
    toggle(
      "settings_dyslexia_font",
      "settings_dyslexia_font_desc",
      (prefs) => prefs.dyslexia_font,
      (on) => ({ dyslexia_font: on }),
    ),
  ),
  section(
    "motion",
    "settings_section_motion",
    "settings_motion_desc",
    toggle(
      "settings_reduce_motion",
      "settings_reduce_motion_desc",
      (prefs) => prefs.reduce_motion,
      (on) => ({ reduce_motion: on }),
    ),
  ),
  section(
    "reading",
    "settings_section_reading",
    "settings_reading_desc",
    dropdown(
      "settings_text_size_a11y",
      "settings_text_size_a11y_desc",
      sizeOptions(["small", "default", "large", "xlarge"]),
      (prefs) => prefs.text_size_a11y,
      (value) => ({ text_size_a11y: value }),
      "text_size_a11y",
    ),
  ),
];

// About

const aboutRow = (titleKey, value) =>
  h(
    "div",
    { class: "pref-row" },
    h(
      "div",
      { class: "pref-row-text" },
      h("span", { class: "pref-row-title" }, t(titleKey)),
      h("span", { class: "pref-row-desc pref-path" }, value),
    ),
  );

const about = () => {
  const rows = h("div", { class: "about-rows" });
  const fill = (info) => {
    const unknown = t("desktop_only");
    rows.replaceChildren(
      aboutRow("preferences_about_version", info?.version ?? unknown),
      aboutRow("preferences_about_license", info?.license ?? unknown),
      aboutRow("preferences_about_config", info?.config_dir ?? unknown),
      aboutRow("preferences_about_data", info?.data_dir ?? unknown),
      aboutRow("preferences_about_cache", info?.cache_dir ?? unknown),
      aboutRow("preferences_about_fonts", t("preferences_about_fonts_desc")),
    );
  };
  fill(null);
  invoke?.("get_app_info").then(fill, (error) => console.warn("orCAL app info", error));
  return [
    h("h2", { class: "pref-heading" }, "orCAL"),
    h("p", { class: "pref-heading-desc" }, t("preferences_about_desc")),
    rows,
  ];
};

const CATEGORIES = [
  ["settings_cat_general", general],
  ["settings_cat_appearance", appearance],
  ["settings_cat_accessibility", accessibility],
  ["settings_cat_about", about],
];

/** Draw the category list and the selected category. */
const render = () => {
  syncs = [];
  nav.replaceChildren(
    ...CATEGORIES.map(([key], index) => {
      const button = h("button", { type: "button", class: "pref-category" }, t(key));
      if (index === category) {
        button.setAttribute("aria-current", "page");
      }
      button.addEventListener("click", () => {
        category = index;
        render();
        nav.children[index].focus();
      });
      return button;
    }),
  );
  content.replaceChildren(...CATEGORIES[category][1]());
  content.scrollTop = 0;
};

export const isPreferencesOpen = () => Boolean(page && !page.hidden);

/** Open Preferences in place of the calculator, or close them. */
export const togglePreferences = async () => {
  const open = !isPreferencesOpen();
  if (open) {
    await loadCatalog();
    category = 0;
    render();
  }
  page.hidden = !open;
  document.querySelectorAll("[data-calculator]").forEach((el) => {
    el.hidden = open;
  });
  brand.setAttribute("aria-expanded", String(open));
  (open ? nav.children[0] : brand).focus();
};

/** Build the page inside `frame`, closed, with `brandButton` as its toggle. */
export const initPreferences = (frame, brandButton) => {
  brand = brandButton;
  nav = h("nav", { class: "preferences-nav", "data-i18n-aria-label": "settings_title" });
  content = h("div", { class: "preferences-content" });
  const close = h("button", {
    type: "button",
    class: "preferences-close",
    "data-i18n": "settings_close",
  });
  close.addEventListener("click", togglePreferences);
  page = h(
    "section",
    { class: "preferences", id: "preferences", hidden: true, "aria-labelledby": "preferences-title" },
    h(
      "header",
      { class: "preferences-header" },
      h("h1", { class: "preferences-title", id: "preferences-title", "data-i18n": "settings_title" }),
      close,
    ),
    h("div", { class: "preferences-body" }, nav, content),
  );
  toast = h(
    "div",
    { class: "toast", role: "status", hidden: true },
    h("span", { class: "toast-message" }),
    h("button", { type: "button", class: "toast-dismiss glyph" }, CLOSE),
  );
  toast.querySelector(".toast-dismiss").addEventListener("click", () => {
    toast.hidden = true;
  });
  frame.append(page, toast);
  brand.setAttribute("aria-controls", page.id);
  brand.setAttribute("aria-expanded", "false");
  brand.addEventListener("click", togglePreferences);

  // Escape leaves Preferences, before the calculator sees it as Clear.
  document.addEventListener(
    "keydown",
    (event) => {
      if (event.key === "Escape" && isPreferencesOpen()) {
        event.preventDefault();
        event.stopPropagation();
        togglePreferences();
      }
    },
    { capture: true },
  );
  document.addEventListener("orcal:locale", () => {
    if (isPreferencesOpen()) {
      render();
    }
  });
};
