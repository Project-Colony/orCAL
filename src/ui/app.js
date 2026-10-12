import { boot } from "./common/chrome.js";
import { decimalSeparator, engineError, localizeNumber, t } from "./common/i18n.js";
import { isPreferencesOpen } from "./common/preferences.js";

const keypad = document.querySelector(".calculator-keypad");
const expressionEl = document.querySelector("[data-expression]");
const resultEl = document.querySelector("[data-result]");
const resultWrapperEl = document.querySelector(".screen-result-wrapper");
const resultTooltipEl = document.querySelector("[data-result-tooltip]");

const tauriInvoke = window.__TAURI__?.core?.invoke ?? null;

const keys = [
  { label: "C", type: "utility", action: "clear" },
  { label: "±", type: "utility", action: "toggle-sign" },
  { label: "%", type: "utility", action: "percent" },
  { label: "⌫", type: "backspace", action: "backspace", ariaLabel: "key_backspace" },
  { label: "7", type: "number" },
  { label: "8", type: "number" },
  { label: "9", type: "number" },
  { label: "÷", type: "operator", action: "operator" },
  { label: "4", type: "number" },
  { label: "5", type: "number" },
  { label: "6", type: "number" },
  { label: "×", type: "operator", action: "operator" },
  { label: "1", type: "number" },
  { label: "2", type: "number" },
  { label: "3", type: "number" },
  { label: "−", type: "operator", action: "operator" },
  { label: "0", type: "number" },
  { label: ".", type: "number", action: "decimal" },
  { label: "=", type: "operator", action: "equals" },
  { label: "+", type: "operator", action: "operator" },
];

const state = {
  expression: "",
  // The engine's result, a plain decimal with "." whatever the locale.
  result: "0",
  // Instead of a result: the i18n key of what went wrong.
  error: null,
};

const operatorMap = {
  "÷": "/",
  "×": "*",
  "−": "-",
  "+": "+",
};

// Drawn again when the language changes: the decimal key shows the locale's
// separator and the labels are translated.
const renderKeys = () => {
  keypad.replaceChildren();
  keys.forEach((key) => {
    const button = document.createElement("button");
    button.type = "button";
    button.classList.add("key");
    button.textContent = key.action === "decimal" ? decimalSeparator() : key.label;
    button.dataset.action = key.action || "input";

    if (key.type === "operator") {
      button.classList.add("key-operator");
    }

    if (key.type === "utility") {
      button.classList.add("key-utility");
    }

    if (key.type === "backspace") {
      button.classList.add("key-backspace");
    }

    if (key.ariaLabel) {
      button.setAttribute("aria-label", t(key.ariaLabel));
    }

    keypad.appendChild(button);
  });
};

const updateScreen = () => {
  const result = state.error ? t(state.error) : localizeNumber(state.result || "0");
  expressionEl.textContent = localizeNumber(state.expression);
  resultEl.textContent = result;
  if (resultTooltipEl) {
    resultTooltipEl.textContent = result;
  }
  if (resultWrapperEl) {
    requestAnimationFrame(() => {
      const isOverflowing =
        resultEl.scrollWidth > resultEl.clientWidth;
      resultWrapperEl.classList.toggle(
        "screen-result-wrapper--tooltip",
        isOverflowing,
      );
    });
  }
};

// The number the expression ends with. A "-" in front of it is its sign only at
// the start or after an operator or "(": the number is -3 in "2*-3" but 3 in
// "2-3", where the "-" is a subtraction.
const lastNumber = (expr = state.expression) => {
  const match = expr.match(/(?:^|[+\-*/^(])(-?\d+(?:[.,]\d+)?)$|(\d+(?:[.,]\d+)?)$/u);
  return match ? match[1] ?? match[2] : null;
};

const toggleSign = () => {
  const number = lastNumber();
  if (!number) {
    return;
  }
  const start = state.expression.slice(0, -number.length);
  const updated = number.startsWith("-")
    ? number.slice(1)
    : `-${number}`;
  state.expression = `${start}${updated}`;
};

const applyPercent = () => {
  const number = lastNumber();
  if (!number) {
    return;
  }
  const normalized = number.replace(",", ".");
  const value = Number.parseFloat(normalized);
  if (Number.isNaN(value)) {
    return;
  }
  // The full value, not a display rounding: 200*0.5% must give 1. An f64
  // round-trips any decimal of up to 15 significant digits, so 1.1% gives 0.011
  // rather than 0.011000000000000001, and toLocaleString never writes an
  // exponent, which the engine would read as the constant e (1e-7 is 1*e-7).
  const percent = (value / 100).toLocaleString("en-US", {
    useGrouping: false,
    maximumSignificantDigits: 15,
  });
  const start = state.expression.slice(0, -number.length);
  state.expression = `${start}${percent}`;
};

const appendOperator = (operator) => {
  if (!state.expression) {
    return;
  }
  if (/[+\-*/]$/u.test(state.expression)) {
    state.expression = `${state.expression.slice(0, -1)}${operator}`;
    return;
  }
  state.expression = `${state.expression}${operator}`;
};

const appendInput = (value) => {
  if (value === "." || value === ",") {
    const number = lastNumber();
    if (number && /[.,]/u.test(number)) {
      return;
    }
  }
  const normalizedValue = value === "," ? "." : value;
  state.expression = `${state.expression}${normalizedValue}`;
};

const toBackendExpression = () =>
  state.expression
    .split("")
    .map((char) => operatorMap[char] || char)
    .join("")
    .replace(/,/gu, ".");

const evaluateExpression = async () => {
  if (!state.expression) {
    return;
  }
  if (!tauriInvoke) {
    state.error = "desktop_only";
    return;
  }
  try {
    // This layout has no ANS key.
    const response = await tauriInvoke("evaluate_expression", {
      expression: toBackendExpression(),
      ans: 0,
    });
    // Already rounded by the engine: shown as is.
    state.result = response.result;
    state.error = null;
  } catch (error) {
    state.error = engineError(error);
  }
};

const handleInputAction = async ({ action, label }) => {
  switch (action) {
    case "clear":
      state.expression = "";
      state.result = "0";
      state.error = null;
      break;
    case "backspace":
      state.expression = state.expression.slice(0, -1);
      break;
    case "toggle-sign":
      toggleSign();
      break;
    case "percent":
      applyPercent();
      break;
    case "operator":
      appendOperator(operatorMap[label] || label);
      break;
    case "equals":
      await evaluateExpression();
      break;
    default:
      appendInput(label);
  }

  updateScreen();
};

keypad.addEventListener("click", async (event) => {
  const button = event.target.closest("button");
  if (!button) {
    return;
  }

  await handleInputAction({
    action: button.dataset.action,
    label: button.textContent,
  });
});

document.addEventListener("keydown", async (event) => {
  // Keys typed in Preferences are theirs, and Enter on a window control or on
  // the orCAL name presses it rather than "=".
  if (
    event.altKey ||
    event.ctrlKey ||
    event.metaKey ||
    isPreferencesOpen() ||
    (event.key === "Enter" && event.target.closest?.(".calculator-status"))
  ) {
    return;
  }

  const { key, code } = event;
  let action = null;
  let label = null;

  if (/^\d$/u.test(key)) {
    action = "input";
    label = key;
  } else if (key === "," || key === ".") {
    action = "decimal";
    label = ".";
  } else if (["+", "-", "*", "/"].includes(key)) {
    action = "operator";
    label = key;
  } else if (key === "%") {
    action = "percent";
  } else if (key === "Enter" || code === "NumpadEnter") {
    action = "equals";
  } else if (key === "Backspace") {
    action = "backspace";
  } else if (key === "Delete" || key === "Escape") {
    action = "clear";
  }

  if (!action) {
    return;
  }

  event.preventDefault();
  await handleInputAction({ action, label });
});

document.addEventListener("orcal:locale", () => {
  renderKeys();
  updateScreen();
});

renderKeys();
updateScreen();
boot();
