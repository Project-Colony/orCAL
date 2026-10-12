// The interface strings, in English (canonical) and French: i18n/en.json and
// i18n/fr.json, whose key sets src/i18n.rs holds equal. The strings every
// Colony program shares (the Preferences skeleton, theme and accent names) are
// not copied there: they come with the themes, in the `labels` of
// vendor/colony/themes.json, and orCAL's own strings are laid over them.
//
// Markup names its strings with data-i18n (text), data-i18n-title and
// data-i18n-aria-label; translate() fills them in.

const LOCALES = ["en", "fr"];

let locale = "en";
let english = {};
let strings = {};

const fetchJson = async (path) => {
  const response = await fetch(new URL(path, import.meta.url));
  if (!response.ok) {
    throw new Error(`${path}: ${response.status}`);
  }
  return response.json();
};

let catalog = null;

/**
 * themes.json, the vendored Colony catalog: themes, accents and the shared
 * labels. Read once; empty if it cannot be read.
 */
export const themeCatalog = () => {
  catalog ??= fetchJson("../vendor/colony/themes.json").catch((error) => {
    console.warn("orCAL could not read the theme catalog", error);
    return { families: [], accents: [], labels: {} };
  });
  return catalog;
};

/** The strings of locale `name`: Colony's shared ones, then orCAL's own. */
const fetchTable = async (name) => {
  const [{ labels }, own] = await Promise.all([themeCatalog(), fetchJson(`../i18n/${name}.json`)]);
  return { ...labels?.[name], ...own };
};

/** The locale for a preference: the saved one, else the system's. */
export const resolveLocale = (preference) => {
  if (LOCALES.includes(preference)) {
    return preference;
  }
  const system = (navigator.languages?.[0] ?? navigator.language ?? "").toLowerCase();
  return system.startsWith("fr") ? "fr" : "en";
};

export const activeLocale = () => locale;

/** The string for `key`, in English when the locale lacks it, else the key. */
export const t = (key) => strings[key] ?? english[key] ?? key;

/**
 * The key for an engine error: its code (ParseError::code in src/engine.rs),
 * or "error_invalid_token" for anything else, such as an IPC failure, which is
 * not worth showing verbatim.
 */
export const engineError = (error) =>
  typeof error === "string" && error.startsWith("error_") && error in english
    ? error
    : "error_invalid_token";

/** A label from themes.json (`{ en, fr }`) in the active locale. */
export const label = (labels) => labels?.[locale] ?? labels?.en ?? "";

/** Fill in every data-i18n* attribute under `root`. */
export const translate = (root = document) => {
  root.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  root.querySelectorAll("[data-i18n-title]").forEach((el) => {
    el.title = t(el.dataset.i18nTitle);
  });
  root.querySelectorAll("[data-i18n-aria-label]").forEach((el) => {
    el.setAttribute("aria-label", t(el.dataset.i18nAriaLabel));
  });
};

/**
 * Switch to the locale `preference` resolves to, translate the page and tell
 * the rest of the interface (the "orcal:locale" event) to redraw its text.
 */
export const setLocale = async (preference) => {
  const next = resolveLocale(preference);
  try {
    if (!Object.keys(english).length) {
      english = await fetchTable("en");
    }
    strings = next === "en" ? english : await fetchTable(next);
    locale = next;
  } catch (error) {
    console.warn("orCAL strings could not be loaded", error);
  }
  document.documentElement.lang = locale;
  translate();
  document.dispatchEvent(new Event("orcal:locale"));
};

/**
 * The locale numbers and times are formatted in: the system's own (en-GB,
 * fr-CA) when it speaks the interface language, else the interface language.
 */
export const formatLocale = () => {
  const system = navigator.languages?.[0] ?? navigator.language ?? "";
  return system.toLowerCase().startsWith(locale) ? system : locale;
};

/** The decimal separator of the active locale: "," in French. */
export const decimalSeparator = () =>
  new Intl.NumberFormat(formatLocale())
    .formatToParts(1.5)
    .find((part) => part.type === "decimal")?.value ?? ".";

/**
 * `text` with its decimal points written the locale's way. The engine's
 * results are plain decimals ("-1234.5", never an exponent), so this changes
 * the separator and nothing else: no digit is rounded away.
 */
export const localizeNumber = (text) => text.replaceAll(".", decimalSeparator());
