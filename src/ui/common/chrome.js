// The window chrome both layouts share: the status bar (clock, messages,
// layout switch, minimize, close), the orCAL name that opens Preferences, and
// the start-up sequence that puts the saved preferences into effect.

import { loadPreferences } from "./prefs.js";
import { applyPreferences } from "./theme.js";
import { formatLocale, setLocale, t } from "./i18n.js";
import { initPreferences } from "./preferences.js";

const appWindow = window.__TAURI__?.window?.getCurrentWindow() ?? null;

// Nothing in orCAL is dragged and dropped. WebKitGTK under XWayland crashes in
// gdk_drag_begin when a text or image drag starts (moving the window by its
// status bar started one), so none ever starts.
document.addEventListener("dragstart", (event) => event.preventDefault());

// Pages and window sizes, as src/preferences.rs opens them (Layout).
const LAYOUTS = {
  phone: { page: "../index.html", size: [360, 720] },
  tablet: { page: "../tablet-mode/index.html", size: [980, 560] },
};

let messageTimer = null;

/** Show the string `key` under the status bar for a few seconds. */
export const showStatusMessage = (key) => {
  const message = document.querySelector(".status-message");
  message.textContent = t(key);
  message.classList.add("status-message--visible");
  window.clearTimeout(messageTimer);
  messageTimer = window.setTimeout(() => {
    message.classList.remove("status-message--visible");
    message.textContent = "";
  }, 3_000);
};

const updateClock = () => {
  document.querySelector(".status-time").textContent = new Intl.DateTimeFormat(formatLocale(), {
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date());
};

const switchLayout = async (layout) => {
  const { page, size } = LAYOUTS[layout];
  if (appWindow) {
    if (layout === "tablet") {
      await appWindow.setResizable(true);
    }
    await appWindow.setSize(new window.__TAURI__.window.LogicalSize(...size));
    await appWindow.center();
  }
  window.location.href = new URL(page, import.meta.url).href;
};

const initStatusBar = () => {
  const icons = document.querySelector(".status-icons");
  if (!appWindow) {
    icons.querySelectorAll("[data-action='minimize'], [data-action='close']").forEach((button) => {
      button.setAttribute("aria-disabled", "true");
    });
  }
  icons.addEventListener("click", (event) => {
    const button = event.target.closest("button[data-action]");
    if (!button) {
      return;
    }
    const { action } = button.dataset;
    if (action === "switch-layout") {
      switchLayout(button.dataset.layout);
    } else if (!appWindow) {
      showStatusMessage("desktop_only");
    } else if (action === "minimize") {
      appWindow.minimize();
    } else if (action === "close") {
      appWindow.close();
    }
  });

  // A double click on the drag area would maximize the window.
  document.querySelector(".drag-region")?.addEventListener("dblclick", (event) => {
    event.preventDefault();
    event.stopPropagation();
  });

  updateClock();
  window.setInterval(updateClock, 60_000);
  document.addEventListener("orcal:locale", updateClock);
};

/**
 * Load preferences.json, put it into effect, wire the chrome and translate the
 * page. The page stays hidden until then, so it never flashes the defaults.
 */
export const boot = async () => {
  try {
    const prefs = await loadPreferences();
    applyPreferences(prefs);
    initStatusBar();
    initPreferences(
      document.querySelector(".calculator-frame"),
      document.querySelector(".status-brand"),
    );
    await setLocale(prefs.locale);
  } finally {
    delete document.documentElement.dataset.loading;
  }
};
