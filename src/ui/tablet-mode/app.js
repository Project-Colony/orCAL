import { boot } from "../common/chrome.js";
import { decimalSeparator, engineError, localizeNumber, t } from "../common/i18n.js";
import { isPreferencesOpen } from "../common/preferences.js";

const keypad = document.querySelector(".calculator-keypad");
const expressionEl = document.querySelector("[data-expression]");
const resultEl = document.querySelector("[data-result]");
const resultWrapperEl = document.querySelector(".screen-result-wrapper");
const resultTooltipEl = document.querySelector("[data-result-tooltip]");

const tauriInvoke = window.__TAURI__?.core?.invoke ?? null;

// Scientific keypad (7 columns x 5 rows).
const keysScientific = [
  { label: "(", type: "muted", action: "paren" },
  { label: ")", type: "muted", action: "paren" },
  { label: "π", type: "muted", action: "const-pi", ariaLabel: "key_pi" },
  { label: "C", type: "utility", action: "clear" },
  { label: "±", type: "utility", action: "toggle-sign" },
  { label: "%", type: "utility", action: "percent" },
  { label: "⌫", type: "backspace", action: "backspace", ariaLabel: "key_backspace" },

  { label: "sin", type: "muted", action: "func" },
  { label: "cos", type: "muted", action: "func" },
  { label: "tan", type: "muted", action: "func" },
  { label: "7", type: "number" },
  { label: "8", type: "number" },
  { label: "9", type: "number" },
  { label: "÷", type: "operator", action: "operator" },

  { label: "ln", type: "muted", action: "func" },
  { label: "log", type: "muted", action: "func" },
  { label: "√", type: "muted", action: "sqrt", ariaLabel: "key_sqrt" },
  { label: "4", type: "number" },
  { label: "5", type: "number" },
  { label: "6", type: "number" },
  { label: "×", type: "operator", action: "operator" },

  { label: "x²", type: "muted", action: "square" },
  { label: "xʸ", type: "muted", action: "power" },
  { label: "1/x", type: "muted", action: "reciprocal" },
  { label: "1", type: "number" },
  { label: "2", type: "number" },
  { label: "3", type: "number" },
  { label: "−", type: "operator", action: "operator" },

  { label: "e", type: "muted", action: "const-e", ariaLabel: "key_e" },
  { label: "!", type: "muted", action: "factorial" },
  { label: "ANS", type: "muted", action: "ans", ariaLabel: "key_ans" },
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

// The last result, sent back to the engine for the ANS key.
const memory = {
  ans: 0,
};

const operatorMap = {
  "÷": "/",
  "×": "*",
  "−": "-",
  "+": "+",
  "^": "^",
};

// Drawn again when the language changes: the decimal key shows the locale's
// separator and the labels are translated.
const renderKeys = () => {
  keypad.replaceChildren();
  keysScientific.forEach((key) => {
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

    if (key.type === "muted") {
      button.classList.add("key-muted");
    }

    if (key.label.length >= 3) {
      button.dataset.long = "";
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

const extractLastAtom = (expr = state.expression) => {
  if (!expr) {
    return null;
  }

  // A factorial belongs to the atom it follows: x² on "5!" squares 5!.
  if (expr.endsWith("!")) {
    const atom = extractLastAtom(expr.slice(0, -1));
    return atom && { start: atom.start, end: expr.length, text: expr.slice(atom.start) };
  }

  if (expr.endsWith("ANS")) {
    return { start: expr.length - 3, end: expr.length, text: "ANS" };
  }

  const lastChar = expr.at(-1);
  if (lastChar === "π" || lastChar === "e") {
    return { start: expr.length - 1, end: expr.length, text: lastChar };
  }

  if (lastChar === ")") {
    let depth = 0;
    let start = -1;
    for (let i = expr.length - 1; i >= 0; i -= 1) {
      if (expr[i] === ")") {
        depth += 1;
      } else if (expr[i] === "(") {
        depth -= 1;
        if (depth === 0) {
          start = i;
          break;
        }
      }
    }
    if (start === -1) {
      return null;
    }

    // Include the function name right before the parentheses (sin, cos, ln, ...),
    // and only that: in "esin(30)" the e is a separate factor.
    const name = expr.slice(0, start).match(/(?:sin|cos|tan|ln|log|sqrt)$/u);
    const fnStart = name ? start - name[0].length : start;

    return { start: fnStart, end: expr.length, text: expr.slice(fnStart) };
  }

  const number = lastNumber(expr);
  if (number) {
    return {
      start: expr.length - number.length,
      end: expr.length,
      text: number,
    };
  }

  return null;
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
  if (/[+\-*/^]$/u.test(state.expression)) {
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

const insertFunction = (name) => {
  // Always insert an opening parenthesis for a natural scientific flow.
  state.expression = `${state.expression}${name}(`;
};

const insertConstant = (symbol) => {
  state.expression = `${state.expression}${symbol}`;
};

const applySquare = () => {
  const atom = extractLastAtom();
  if (!atom) {
    return;
  }
  const before = state.expression.slice(0, atom.start);
  const target = atom.text;
  state.expression = `${before}(${target})^2`;
};

const applyReciprocal = () => {
  const atom = extractLastAtom();
  if (!atom) {
    return;
  }
  const before = state.expression.slice(0, atom.start);
  const target = atom.text;
  state.expression = `${before}1/(${target})`;
};

const applyFactorial = () => {
  // The engine evaluates the postfix "!" along with the rest on "=".
  if (extractLastAtom()) {
    state.expression = `${state.expression}!`;
  }
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
    const response = await tauriInvoke("evaluate_expression", {
      expression: toBackendExpression(),
      ans: memory.ans,
    });
    // Already rounded by the engine: shown as is. ANS keeps the full value.
    state.result = response.result;
    state.error = null;
    memory.ans = response.value;
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
    case "paren":
      appendInput(label);
      break;
    case "const-pi":
      insertConstant("π");
      break;
    case "const-e":
      insertConstant("e");
      break;
    case "ans":
      appendInput("ANS");
      break;
    case "func":
      insertFunction(label);
      break;
    case "sqrt":
      insertFunction("sqrt");
      break;
    case "square":
      applySquare();
      break;
    case "power":
      appendOperator("^");
      break;
    case "reciprocal":
      applyReciprocal();
      break;
    case "factorial":
      applyFactorial();
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
