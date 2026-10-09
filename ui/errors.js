// English text for the calculation engine's error codes (ParseError::code in
// crates/orcal-core), until the interface gets its i18n layer.
const engineErrorMessages = new Map([
  ["error_empty", "Empty expression"],
  ["error_invalid_token", "Invalid expression"],
  ["error_incomplete", "Incomplete expression"],
  ["error_division_by_zero", "Division by zero"],
  ["error_too_deep", "Too deeply nested"],
  ["error_factorial_domain", "Factorial: integer ≥ 0"],
  ["error_factorial_too_large", "Factorial too large"],
  ["error_invalid_result", "Invalid result"],
]);

// Anything else (an IPC failure, say) is not worth showing verbatim.
const engineErrorMessage = (code) =>
  engineErrorMessages.get(code) ?? "Invalid expression";
