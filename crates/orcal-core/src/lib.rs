use std::f64::consts::{E, PI};
use thiserror::Error;

/// How deep parentheses, signs, powers and functions may nest. Every level is
/// a few stack frames, so without a cap a long run of `(` or `-` overflows the
/// stack instead of returning an error.
const MAX_DEPTH: usize = 256;

/// Largest n whose factorial fits in an f64: 171! overflows to infinity.
const MAX_FACTORIAL: f64 = 170.0;

type Function = fn(f64) -> f64;

/// Prefix functions: each applies to the operand right after it, so `sin(30)`,
/// `sin 30` and `√4` all read naturally. Trigonometry works in degrees.
const FUNCTIONS: [(&str, Function); 7] = [
    ("sqrt", f64::sqrt),
    ("√", f64::sqrt),
    ("sin", sin_degrees),
    ("cos", cos_degrees),
    ("tan", tan_degrees),
    ("ln", f64::ln),
    ("log", f64::log10),
];

const CONSTANTS: [(&str, f64); 3] = [("pi", PI), ("π", PI), ("e", E)];

/// The previous result, supplied by the caller.
const ANS: &str = "ANS";

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("empty expression")]
    EmptyInput,
    #[error("invalid token: {0}")]
    InvalidToken(char),
    #[error("incomplete expression")]
    UnexpectedEnd,
    #[error("division by zero")]
    DivisionByZero,
    #[error("expression nested too deeply")]
    TooDeep,
    #[error("factorial of a negative or fractional number")]
    FactorialDomain,
    #[error("factorial too large")]
    FactorialTooLarge,
    #[error("result is not a finite number")]
    InvalidResult,
}

impl ParseError {
    /// Stable identifier the UI turns into a translated message. The Display
    /// text is English and meant for logs, never for the screen.
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "error_empty",
            Self::InvalidToken(_) => "error_invalid_token",
            Self::UnexpectedEnd => "error_incomplete",
            Self::DivisionByZero => "error_division_by_zero",
            Self::TooDeep => "error_too_deep",
            Self::FactorialDomain => "error_factorial_domain",
            Self::FactorialTooLarge => "error_factorial_too_large",
            Self::InvalidResult => "error_invalid_result",
        }
    }
}

/// Recursive descent over this grammar, loosest binding first:
///
/// ```text
/// expression = term (("+" | "-") term)*
/// term       = unary (("*" | "/") unary | unary)*    a bare operand multiplies: 2π, 2(3), (2)(3)
/// unary      = "-" unary | power                     so -2^2 is -(2^2)
/// power      = postfix ("^" unary)?                  right-associative: 2^3^2 is 2^9
/// postfix    = primary "!"*
/// primary    = number | constant | ANS | "(" expression ")" | function primary
/// ```
struct Parser<'a> {
    input: &'a str,
    pos: usize,
    depth: usize,
    ans: f64,
}

impl Parser<'_> {
    fn parse(mut self) -> Result<f64, ParseError> {
        self.skip_whitespace();
        if self.is_eof() {
            return Err(ParseError::EmptyInput);
        }
        let value = self.parse_expression()?;
        self.skip_whitespace();
        if !self.is_eof() {
            return Err(self.unexpected());
        }
        finite(value)
    }

    fn parse_expression(&mut self) -> Result<f64, ParseError> {
        let mut value = self.parse_term()?;
        loop {
            if self.eat('+') {
                let term = self.parse_term()?;
                value = without_cancellation_noise(value + term, value, term);
            } else if self.eat('-') {
                let term = self.parse_term()?;
                value = without_cancellation_noise(value - term, value, term);
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_term(&mut self) -> Result<f64, ParseError> {
        let mut value = self.parse_unary()?;
        loop {
            if self.eat('*') {
                value *= self.parse_unary()?;
            } else if self.eat('/') {
                let divisor = self.parse_unary()?;
                if divisor == 0.0 {
                    return Err(ParseError::DivisionByZero);
                }
                value /= divisor;
            } else if self.at_implicit_operand() {
                value *= self.parse_unary()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_unary(&mut self) -> Result<f64, ParseError> {
        if self.eat('-') {
            Ok(-self.nested(Self::parse_unary)?)
        } else {
            self.parse_power()
        }
    }

    fn parse_power(&mut self) -> Result<f64, ParseError> {
        let base = self.parse_postfix()?;
        if self.eat('^') {
            // The exponent is a unary, so it may carry a sign (2^-1) and a
            // further power (2^3^2), which is what makes ^ right-associative.
            finite(base.powf(self.nested(Self::parse_unary)?))
        } else {
            Ok(base)
        }
    }

    fn parse_postfix(&mut self) -> Result<f64, ParseError> {
        let mut value = finite(self.parse_primary()?)?;
        while self.eat('!') {
            value = factorial(value)?;
        }
        Ok(value)
    }

    fn parse_primary(&mut self) -> Result<f64, ParseError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some(ch) if ch.is_ascii_digit() || ch == '.' => return self.parse_number(),
            Some('(') => {
                self.advance_char();
                let value = self.nested(Self::parse_expression)?;
                return if self.eat(')') {
                    Ok(value)
                } else {
                    Err(self.unexpected())
                };
            }
            _ => {}
        }
        // No name is a prefix of another, so the first match is the only one,
        // and `esin(30)` reads as e * sin(30).
        let rest = &self.input[self.pos..];
        if let Some((name, function)) = FUNCTIONS.iter().find(|(name, _)| rest.starts_with(name)) {
            self.pos += name.len();
            let argument = finite(self.nested(Self::parse_primary)?)?;
            return Ok(function(argument));
        }
        if let Some((name, value)) = CONSTANTS.iter().find(|(name, _)| rest.starts_with(name)) {
            self.pos += name.len();
            return Ok(*value);
        }
        if rest.starts_with(ANS) {
            self.pos += ANS.len();
            return Ok(self.ans);
        }
        Err(self.unexpected())
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.pos;
        let mut seen_dot = false;
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.advance_char();
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                self.advance_char();
            } else {
                break;
            }
        }
        // Only a lone "." fails to parse.
        self.input[start..self.pos]
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidToken('.'))
    }

    /// Whether the next character starts an operand that multiplies the one
    /// just read. A number straight after a number is a typo (`2 3`, `1.2.3`),
    /// not a product.
    fn at_implicit_operand(&mut self) -> bool {
        self.skip_whitespace();
        let is_number = |ch: char| ch.is_ascii_digit() || ch == '.';
        match self.peek_char() {
            Some(ch) if is_number(ch) => !self.input[..self.pos].trim_end().ends_with(is_number),
            Some(ch) => ch == '(' || ch == 'π' || ch == '√' || ch.is_ascii_alphabetic(),
            None => false,
        }
    }

    /// Runs one level deeper, or fails with `TooDeep` past `MAX_DEPTH`. Every
    /// recursive step goes through here.
    fn nested(
        &mut self,
        parse: fn(&mut Self) -> Result<f64, ParseError>,
    ) -> Result<f64, ParseError> {
        if self.depth == MAX_DEPTH {
            return Err(ParseError::TooDeep);
        }
        self.depth += 1;
        let value = parse(self);
        self.depth -= 1;
        value
    }

    /// The error for whatever stops the parse at the current position.
    fn unexpected(&self) -> ParseError {
        self.peek_char()
            .map_or(ParseError::UnexpectedEnd, ParseError::InvalidToken)
    }

    /// Skips whitespace, then consumes `expected` if it is next.
    fn eat(&mut self, expected: char) -> bool {
        self.skip_whitespace();
        let found = self.peek_char() == Some(expected);
        if found {
            self.advance_char();
        }
        found
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.advance_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

/// Rejects NaN and infinity as soon as an operand produces one. Checking only
/// the final result is not enough: `/` and `^` can turn them back into a finite
/// number, so 1/tan(90) would give 0 and sqrt(-1)^0 would give 1.
fn finite(value: f64) -> Result<f64, ParseError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ParseError::InvalidResult)
    }
}

/// `sum`, the sum or difference of `a` and `b`, rounded to 15 significant
/// digits of the larger operand, the precision an f64 keeps for any decimal.
/// Decimals such as 0.1 are not exact in binary, so 0.1 + 0.2 - 0.3 leaves
/// 5.6e-17 and 0.3 - 0.2999999 gives 1.00000000003e-7. When the operands
/// cancel out, that residue is most of the result, and rounding the screen to
/// 12 digits of the result cannot hide it. Rounding here gives 0 and 1e-7.
/// Whole operands add exactly, so their sum is kept as is.
fn without_cancellation_noise(sum: f64, a: f64, b: f64) -> f64 {
    let decimals = 14 - a.abs().max(b.abs()).log10().floor() as i32;
    if (a.fract() == 0.0 && b.fract() == 0.0) || decimals <= 0 {
        return sum;
    }
    format!("{sum:.prec$}", prec = decimals as usize)
        .parse()
        .unwrap_or(sum)
}

fn factorial(n: f64) -> Result<f64, ParseError> {
    // NaN and infinity have a NaN fractional part, so they land here too.
    if n < 0.0 || n.fract() != 0.0 {
        return Err(ParseError::FactorialDomain);
    }
    if n > MAX_FACTORIAL {
        return Err(ParseError::FactorialTooLarge);
    }
    Ok((2..=n as u32).map(f64::from).product())
}

// Exact at the multiples of 90 degrees, where going through radians leaves a
// 1e-16 residue: sin(180) is 0, and tan(90) divides by zero, so the result is
// rejected as non-finite rather than shown as 1.6e16.
fn sin_degrees(x: f64) -> f64 {
    if x % 180.0 == 0.0 {
        0.0
    } else {
        x.to_radians().sin()
    }
}

fn cos_degrees(x: f64) -> f64 {
    if (x - 90.0) % 180.0 == 0.0 {
        0.0
    } else {
        x.to_radians().cos()
    }
}

fn tan_degrees(x: f64) -> f64 {
    sin_degrees(x) / cos_degrees(x)
}

/// Evaluates `expression`, reading `ANS` as `ans`, the previous result.
pub fn evaluate_with(expression: &str, ans: f64) -> Result<f64, ParseError> {
    Parser {
        input: expression,
        pos: 0,
        depth: 0,
        ans,
    }
    .parse()
}

/// Significant digits a result is shown with. An f64 carries 15 to 17, and the
/// last ones hold the binary rounding error: 0.1 + 0.2 is 0.30000000000000004.
const RESULT_DIGITS: usize = 12;

/// The text the interface shows for `value`: rounded to `RESULT_DIGITS`
/// significant digits, in plain notation, without trailing zeros.
pub fn format_result(value: f64) -> String {
    let rounded = if value.abs() >= 10f64.powi(RESULT_DIGITS as i32) {
        // The rounding never reaches into the whole part: past 12 integer
        // digits only the fraction goes, so 20! keeps every digit.
        value.round()
    } else {
        // Scientific notation rounds to significant digits whatever the
        // magnitude. Parsing it back and printing the f64 gives the shortest
        // plain text for it: no trailing zeros and no exponent.
        format!("{value:.prec$e}", prec = RESULT_DIGITS - 1)
            .parse()
            .unwrap_or(value)
    };
    if rounded == 0.0 {
        // Also covers -0, which would print as "-0".
        return "0".to_string();
    }
    rounded.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(expression: &str) -> Result<f64, ParseError> {
        evaluate_with(expression, 0.0)
    }

    fn approx(expression: &str, expected: f64) {
        let value = evaluate(expression).unwrap_or_else(|error| panic!("{expression}: {error}"));
        assert!(
            (value - expected).abs() < 1e-12,
            "{expression} = {value}, expected {expected}"
        );
    }

    #[test]
    fn evaluates_expressions() {
        assert_eq!(evaluate("12 * (3 + 8)"), Ok(132.0));
    }

    #[test]
    fn arithmetic_operators() {
        assert_eq!(evaluate("2+3"), Ok(5.0));
        assert_eq!(evaluate("2-3"), Ok(-1.0));
        assert_eq!(evaluate("2*3"), Ok(6.0));
        assert_eq!(evaluate("3/2"), Ok(1.5));
        assert_eq!(evaluate(".5+1."), Ok(1.5));
        assert_eq!(evaluate(" 1 + 2 "), Ok(3.0));
    }

    #[test]
    fn precedence_and_associativity() {
        assert_eq!(evaluate("2+3*4"), Ok(14.0));
        assert_eq!(evaluate("(2+3)*4"), Ok(20.0));
        assert_eq!(evaluate("10-4-3"), Ok(3.0));
        assert_eq!(evaluate("16/4/2"), Ok(2.0));
        assert_eq!(evaluate("2*3^2"), Ok(18.0));
        assert_eq!(evaluate("2^3^2"), Ok(512.0));
        assert_eq!(evaluate("2^3!"), Ok(64.0));
        assert_eq!(evaluate("(1+2)!"), Ok(6.0));
    }

    #[test]
    fn unary_minus() {
        assert_eq!(evaluate("-3"), Ok(-3.0));
        assert_eq!(evaluate("--3"), Ok(3.0));
        assert_eq!(evaluate("5*-3"), Ok(-15.0));
        assert_eq!(evaluate("-2^2"), Ok(-4.0));
        assert_eq!(evaluate("(-2)^2"), Ok(4.0));
        assert_eq!(evaluate("2^-1"), Ok(0.5));
        assert_eq!(evaluate("-3!"), Ok(-6.0));
        // What the keypad sends for 2 - 3, then ± (and x²).
        assert_eq!(evaluate("2--3"), Ok(5.0));
        assert_eq!(evaluate("2-(3)^2"), Ok(-7.0));
    }

    #[test]
    fn factorial_values() {
        assert_eq!(evaluate("0!"), Ok(1.0));
        assert_eq!(evaluate("1!"), Ok(1.0));
        assert_eq!(evaluate("5!"), Ok(120.0));
        assert_eq!(evaluate("3!!"), Ok(720.0));
        assert!(evaluate("170!").is_ok_and(f64::is_finite));
    }

    #[test]
    fn factorial_errors() {
        assert_eq!(evaluate("171!"), Err(ParseError::FactorialTooLarge));
        assert_eq!(evaluate("2.5!"), Err(ParseError::FactorialDomain));
        assert_eq!(evaluate("(-1)!"), Err(ParseError::FactorialDomain));
    }

    #[test]
    fn functions_in_degrees() {
        approx("sin(30)", 0.5);
        approx("cos(60)", 0.5);
        approx("tan(45)", 1.0);
        assert_eq!(evaluate("sin(180)"), Ok(0.0));
        assert_eq!(evaluate("cos(90)"), Ok(0.0));
        assert_eq!(evaluate("cos(270)"), Ok(0.0));
        assert_eq!(evaluate("tan(180)"), Ok(0.0));
        assert_eq!(evaluate("tan(90)"), Err(ParseError::InvalidResult));
        assert_eq!(evaluate("tan(-90)"), Err(ParseError::InvalidResult));
    }

    #[test]
    fn logarithms_and_roots() {
        approx("ln(e)", 1.0);
        approx("log(1000)", 3.0);
        assert_eq!(evaluate("sqrt(16)"), Ok(4.0));
        assert_eq!(evaluate("√16"), Ok(4.0));
        assert_eq!(evaluate("√(9+16)"), Ok(5.0));
        assert_eq!(evaluate("sin 90"), Ok(1.0));
        assert_eq!(evaluate("sqrt(16)^2"), Ok(16.0));
    }

    #[test]
    fn constants_and_ans() {
        assert_eq!(evaluate("pi"), Ok(PI));
        assert_eq!(evaluate("π"), Ok(PI));
        assert_eq!(evaluate("e"), Ok(E));
        assert_eq!(evaluate_with("ANS", 42.0), Ok(42.0));
        assert_eq!(evaluate_with("ANS*2+1", 4.0), Ok(9.0));
        assert_eq!(evaluate("ANS"), Ok(0.0));
    }

    #[test]
    fn implicit_multiplication() {
        assert_eq!(evaluate("2π"), Ok(2.0 * PI));
        assert_eq!(evaluate("2(3)"), Ok(6.0));
        assert_eq!(evaluate("(2)(3)"), Ok(6.0));
        assert_eq!(evaluate("(2)3"), Ok(6.0));
        assert_eq!(evaluate("π2"), Ok(2.0 * PI));
        assert_eq!(evaluate("2e"), Ok(2.0 * E));
        assert_eq!(evaluate("esin(90)"), Ok(E));
        assert_eq!(evaluate("2sqrt(4)"), Ok(4.0));
        assert_eq!(evaluate_with("3ANS", 2.0), Ok(6.0));
        assert_eq!(evaluate("2^2(3)"), Ok(12.0));
        assert_eq!(evaluate("2+3(4)"), Ok(14.0));
        assert_eq!(evaluate("2(3)-1"), Ok(5.0));
        approx("e(sin(30))^2", E / 4.0);
    }

    #[test]
    fn adjacent_numbers_are_not_a_product() {
        assert_eq!(evaluate("2 3"), Err(ParseError::InvalidToken('3')));
        assert_eq!(evaluate("1.2.3"), Err(ParseError::InvalidToken('.')));
    }

    #[test]
    fn division_by_zero() {
        assert_eq!(evaluate("1/0"), Err(ParseError::DivisionByZero));
        assert_eq!(evaluate("1/(2-2)"), Err(ParseError::DivisionByZero));
    }

    #[test]
    fn invalid_tokens() {
        assert_eq!(evaluate("2+$"), Err(ParseError::InvalidToken('$')));
        assert_eq!(evaluate("foo(1)"), Err(ParseError::InvalidToken('f')));
        assert_eq!(evaluate("sinh(1)"), Err(ParseError::InvalidToken('h')));
        assert_eq!(evaluate("ans"), Err(ParseError::InvalidToken('a')));
        assert_eq!(evaluate("2)"), Err(ParseError::InvalidToken(')')));
        assert_eq!(evaluate("(2]"), Err(ParseError::InvalidToken(']')));
        assert_eq!(evaluate("+2"), Err(ParseError::InvalidToken('+')));
        assert_eq!(evaluate("."), Err(ParseError::InvalidToken('.')));
        assert_eq!(evaluate("√-4"), Err(ParseError::InvalidToken('-')));
    }

    #[test]
    fn incomplete_expressions() {
        assert_eq!(evaluate(""), Err(ParseError::EmptyInput));
        assert_eq!(evaluate("   "), Err(ParseError::EmptyInput));
        assert_eq!(evaluate("2+"), Err(ParseError::UnexpectedEnd));
        assert_eq!(evaluate("(2"), Err(ParseError::UnexpectedEnd));
        assert_eq!(evaluate("sin("), Err(ParseError::UnexpectedEnd));
        assert_eq!(evaluate("sqrt"), Err(ParseError::UnexpectedEnd));
        assert_eq!(evaluate("2^"), Err(ParseError::UnexpectedEnd));
        assert_eq!(evaluate("-"), Err(ParseError::UnexpectedEnd));
    }

    #[test]
    fn non_finite_results() {
        assert_eq!(evaluate("sqrt(-1)"), Err(ParseError::InvalidResult));
        assert_eq!(evaluate("ln(0)"), Err(ParseError::InvalidResult));
        assert_eq!(evaluate("log(-1)"), Err(ParseError::InvalidResult));
        assert_eq!(evaluate("0^-1"), Err(ParseError::InvalidResult));
        assert_eq!(evaluate("10^400"), Err(ParseError::InvalidResult));
        assert_eq!(evaluate(&"9".repeat(400)), Err(ParseError::InvalidResult));
        assert_eq!(evaluate("sqrt(-1)!"), Err(ParseError::InvalidResult));
    }

    #[test]
    fn undefined_intermediate_results_are_not_hidden() {
        // Each of these turns NaN or infinity back into a finite number if only
        // the final result is checked.
        for expression in [
            "1/tan(90)",
            "1/ln(0)",
            "e^ln(0)",
            "sqrt(-1)^0",
            "1^ln(-1)",
            "sqrt(sqrt(-1))^0",
            "1/10^400",
            "2^-10^400",
            "(10^400)^0",
            "1/(10^200*10^200)",
        ] {
            assert_eq!(
                evaluate(expression),
                Err(ParseError::InvalidResult),
                "{expression}"
            );
        }
        let huge = format!("1/{}", "9".repeat(400));
        assert_eq!(evaluate(&huge), Err(ParseError::InvalidResult));
        assert_eq!(
            evaluate_with("1/ANS", f64::INFINITY),
            Err(ParseError::InvalidResult)
        );
        assert_eq!(
            evaluate_with("ANS^0", f64::NAN),
            Err(ParseError::InvalidResult)
        );
    }

    #[test]
    fn depth_cap() {
        let nested = |depth: usize| format!("{}1{}", "(".repeat(depth), ")".repeat(depth));
        assert_eq!(evaluate(&nested(MAX_DEPTH)), Ok(1.0));
        assert_eq!(evaluate(&nested(MAX_DEPTH + 1)), Err(ParseError::TooDeep));

        // Each kind of recursion stops at the cap rather than overflowing the stack.
        for hostile in [
            "(".repeat(100_000),
            format!("{}1", "-".repeat(100_000)),
            format!("{}2", "2^".repeat(100_000)),
            format!("{}4", "√".repeat(100_000)),
            format!("{}1", "sin(".repeat(100_000)),
        ] {
            assert_eq!(evaluate(&hostile), Err(ParseError::TooDeep));
        }
    }

    #[test]
    fn deepest_accepted_expression_fits_a_small_stack() {
        // Tauri runs synchronous commands on the main thread, whose stack is
        // 1 MiB on Windows. Half of that must hold the deepest valid input,
        // even in an unoptimised build.
        let nested = format!("{}1{}", "(".repeat(MAX_DEPTH), ")".repeat(MAX_DEPTH));
        let value = std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(move || evaluate(&nested))
            .expect("spawn")
            .join()
            .expect("no stack overflow");
        assert_eq!(value, Ok(1.0));
    }

    #[test]
    fn error_codes_are_stable() {
        let cases = [
            (ParseError::EmptyInput, "error_empty"),
            (ParseError::InvalidToken('x'), "error_invalid_token"),
            (ParseError::UnexpectedEnd, "error_incomplete"),
            (ParseError::DivisionByZero, "error_division_by_zero"),
            (ParseError::TooDeep, "error_too_deep"),
            (ParseError::FactorialDomain, "error_factorial_domain"),
            (ParseError::FactorialTooLarge, "error_factorial_too_large"),
            (ParseError::InvalidResult, "error_invalid_result"),
        ];
        for (error, code) in cases {
            assert_eq!(error.code(), code);
        }
    }

    #[test]
    fn formats_trimmed_result() {
        assert_eq!(format_result(42.0), "42");
        assert_eq!(format_result(2.5000), "2.5");
        assert_eq!(format_result(-0.0), "0");
        assert_eq!(format_result(-7.25), "-7.25");
    }

    #[test]
    fn formats_without_float_noise() {
        let shown = |expression: &str| format_result(evaluate(expression).unwrap());
        assert_eq!(shown("0.1+0.2"), "0.3");
        assert_eq!(shown("1/8"), "0.125");
        assert_eq!(shown("1/3"), "0.333333333333");
        assert_eq!(shown("2/3"), "0.666666666667");
        assert_eq!(shown("200*0.005"), "1");
        assert_eq!(shown("sin(30)"), "0.5");
        assert_eq!(shown("1.1*1.1"), "1.21");
    }

    #[test]
    fn cancels_rounding_residue_to_zero() {
        let shown = |expression: &str| format_result(evaluate(expression).unwrap());
        assert_eq!(shown("0.1+0.2-0.3"), "0");
        assert_eq!(shown("0.3-0.1-0.2"), "0");
        assert_eq!(shown("1.1*3-3.3"), "0");
        assert_eq!(shown("sqrt(2)^2-2"), "0");
        // Real small differences survive.
        assert_eq!(shown("1000000000000.5-1000000000000"), "0.5");
        assert_eq!(shown("1000000000000001-1000000000000000"), "1");
        assert_eq!(shown("0.3-0.2999999"), "0.0000001");
        assert_eq!(format_result(1e-20), "0.00000000000000000001");
        // The value itself is clean, so ANS carries no residue either.
        assert_eq!(evaluate("0.1+0.2").unwrap(), 0.3);
    }

    #[test]
    fn formats_extreme_magnitudes_in_plain_notation() {
        assert_eq!(format_result(1e20), "100000000000000000000");
        assert_eq!(format_result(-1e20), "-100000000000000000000");
        assert_eq!(format_result(1e-20), "0.00000000000000000001");
        assert_eq!(format_result(1.5e-7), "0.00000015");
        assert_eq!(format_result(123_456_789.123_456), "123456789.123");
        assert_eq!(format_result(999_999_999_999.999_9), "1000000000000");
        // Whole parts longer than 12 digits are never rounded, only the fraction.
        assert_eq!(format_result(1_234_567_890_123.0), "1234567890123");
        assert_eq!(format_result(1_234_567_890_123.4), "1234567890123");
        assert_eq!(
            format_result(evaluate("20!").unwrap()),
            "2432902008176640000"
        );
    }
}
