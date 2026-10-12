// Puts the Appearance and Accessibility preferences into effect, as colony-ui's
// StandardPreferences::apply() does for the Rust programs. Everything lands on
// <body>, where common/base.css reads it.

/** Appearance > Typography: small 0.85, default 1.0, large 1.2. */
export const FONT_SIZES = { small: 0.85, default: 1, large: 1.2 };
/** Accessibility > Reading: multiplies with the font size. */
export const TEXT_SIZES = { small: 0.85, default: 1, large: 1.2, xlarge: 1.4 };

/** Colony's fallback for a theme it does not know (THM-3). */
const FALLBACK_THEME = "gruvbox-dark";

const systemReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

let applied = null;

/** WCAG relative luminance of an sRGB colour given as [r, g, b] in 0..1. */
const luminance = (rgb) => {
  const channel = (v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  const [r, g, b] = rgb.map(channel);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};

const contrast = (a, b) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};

/** "#rrggbb", as the vendored palettes and themes.json write colours. */
const parseHex = (text) => {
  const hex = text.trim().match(/^#([0-9a-f]{6})$/iu)?.[1];
  return hex ? [0, 2, 4].map((i) => Number.parseInt(hex.slice(i, i + 2), 16) / 255) : null;
};

/** colony-ui's contrast_on() ends, which colony-accents.css sets on :root. */
const contrastEnds = () => {
  const root = getComputedStyle(document.documentElement);
  return ["--colony-ink", "--colony-paper"].map((name) => parseHex(root.getPropertyValue(name)));
};

/**
 * "ink" or "paper": whichever of colony-ui's two label colours contrasts more
 * with `color`, as contrast_on() picks it. Paper for an unreadable colour.
 */
export const contrastOn = (color) => {
  const rgb = parseHex(color);
  const [ink, paper] = contrastEnds();
  return rgb && ink && paper && contrast(ink, rgb) >= contrast(paper, rgb) ? "ink" : "paper";
};

/** Whether `color` is a light surface, by YIQ luminance as colony-ui reads it. */
const isLight = (color) => {
  const rgb = parseHex(color);
  return Boolean(rgb) && 0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2] > 0.5;
};

/** The effects are off: Effects unticked, Reduce motion on, or the system asks. */
const motionOff = (prefs) =>
  !prefs.animations || prefs.reduce_motion || systemReducedMotion.matches;

/** Put `prefs` into effect. */
export const applyPreferences = (prefs) => {
  applied = prefs;
  const { body } = document;
  // The slug colony-themes.css selects on: "stellar_blade" + "eve" is
  // "stellar-blade-eve" (each variant's slug in themes.json).
  body.dataset.colonyTheme = `${prefs.selected_theme.replaceAll("_", "-")}-${prefs.selected_variant}`;
  let style = getComputedStyle(body);
  if (!style.getPropertyValue("--colony-bg-primary").trim()) {
    body.dataset.colonyTheme = FALLBACK_THEME;
    style = getComputedStyle(body);
  }

  const accent = prefs.auto_accent ? null : prefs.selected_accent;
  if (accent) {
    body.dataset.accent = accent;
  } else {
    delete body.dataset.accent;
  }
  body.toggleAttribute("data-high-contrast", Boolean(prefs.high_contrast));
  body.toggleAttribute("data-dyslexia", Boolean(prefs.dyslexia_font));
  body.dataset.motion = motionOff(prefs) ? "off" : "on";
  const scale =
    (FONT_SIZES[prefs.font_size] ?? 1) * (TEXT_SIZES[prefs.text_size_a11y] ?? 1);
  body.style.setProperty("--font-scale", String(scale));

  // What base.css cannot compute itself: whether the theme is light, and the
  // label colour for the accent and warning fills.
  body.dataset.mode = isLight(style.getPropertyValue("--colony-bg-primary")) ? "light" : "dark";
  body.dataset.onAccent = contrastOn(style.getPropertyValue("--accent"));
  body.dataset.onWarning = contrastOn(style.getPropertyValue("--colony-warning"));
};

// The system setting can change while orCAL is open.
systemReducedMotion.addEventListener("change", () => {
  if (applied) {
    applyPreferences(applied);
  }
});
