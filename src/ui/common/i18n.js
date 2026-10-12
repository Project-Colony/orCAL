// The interface strings, in English (canonical) and French: i18n/en.json and
// i18n/fr.json, whose key sets src/i18n.rs holds equal. Theme and accent names
// are not here: they come with the themes, in vendor/colony/themes.json.
//
// Markup names its strings with data-i18n (text), data-i18n-title and
// data-i18n-aria-label; translate() fills them in.

const LOCALES = ["en", "fr"];

let locale = "en";
let english = {};
let strings = {};

const fetchTable = async (name) => {
  const response = await fetch(new URL(`../i18n/${name}.json`, import.meta.url));
  if (!response.ok) {
    throw new Error(`i18n/${name}.json: ${response.status}`);
  }
  return response.json();
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
