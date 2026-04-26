use crate::blocks::{deserialize_f64s, is_high, serialize_f64s, Block};
use crate::types::Signal;

/// Analog sum: output = sum of all inputs and parameters.
#[derive(Clone, Copy)]
pub struct Add;

impl Block for Add {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        vec![inputs.iter().chain(params.iter()).copied().sum()]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Add"
    }
}

/// Analog subtraction: first operand minus all remaining operands.
#[derive(Clone, Copy)]
pub struct Sub;

impl Block for Sub {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let mut operands = inputs.iter().chain(params.iter()).copied();
        let first = operands.next().unwrap_or(0.0);
        vec![operands.fold(first, |acc, value| acc - value)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Sub"
    }
}

/// Analog multiplication: product of all operands.
#[derive(Clone, Copy)]
pub struct Mult;

impl Block for Mult {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let operands: Vec<Signal> = inputs.iter().chain(params.iter()).copied().collect();
        if operands.is_empty() {
            vec![0.0]
        } else {
            vec![operands.into_iter().product()]
        }
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Mult"
    }
}

/// Analog division: first operand divided by the remaining operands.
#[derive(Clone, Copy)]
pub struct Div;

impl Block for Div {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let mut operands = inputs.iter().chain(params.iter()).copied();
        let first = operands.next().unwrap_or(0.0);
        let result = operands.try_fold(first, |acc, value| {
            if value.abs() <= f64::EPSILON {
                None
            } else {
                Some(acc / value)
            }
        });
        vec![result.unwrap_or(0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Div"
    }
}

/// Analog modulo: I1 % I2 (or I1 % param[0]).
#[derive(Clone, Copy)]
pub struct Mod;

impl Block for Mod {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let left = inputs.first().copied().unwrap_or(0.0);
        let right = inputs
            .get(1)
            .copied()
            .or_else(|| params.first().copied())
            .unwrap_or(0.0);
        if right.abs() <= f64::EPSILON {
            vec![0.0]
        } else {
            vec![left % right]
        }
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Mod"
    }
}

/// Floor (integer part): output = floor(input).
#[derive(Clone, Copy)]
pub struct Int;

impl Block for Int {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        vec![value.floor()]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Int"
    }
}

/// Sum of exactly 4 inputs (Loxone's Add4 block).
#[derive(Clone, Copy)]
pub struct Add4;

impl Block for Add4 {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let sum: Signal = inputs.iter().take(4).copied().sum();
        vec![sum]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Add4"
    }
}

/// Minmax: outputs [min, max] of all inputs.
#[derive(Clone, Copy)]
pub struct Minmax;

impl Block for Minmax {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        if inputs.is_empty() {
            return vec![0.0, 0.0];
        }
        let min = inputs.iter().copied().fold(f64::INFINITY, f64::min);
        let max = inputs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        vec![min, max]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Minmax"
    }
}

/// Analog min/max tracker with reset.
///
/// Inputs: I1 = value, I2 = reset (digital pulse)
/// Outputs: [current_min, current_max]
///
/// Tracks the running minimum and maximum of input values.
/// A rising edge on reset clears the tracked min/max to the current value.
///
/// Analogue Limiter (Min/Max): clamps input value between Min and Max parameters.
///
/// Connectors (from connector-map.json):
///   inputs:  Input (V)
///   params:  Min (default 0), Max (default 10)
///   outputs: AQ (clamped value)
#[derive(Clone)]
pub struct AMinmax;

impl AMinmax {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AMinmax {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AMinmax {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let min_val = params.first().copied().unwrap_or(0.0);
        let max_val = params.get(1).copied().unwrap_or(10.0);

        let clamped = value.max(min_val).min(max_val);
        vec![clamped]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "AMinmax"
    }
}

/// Expression evaluator: computes a formula string with I1–I4 substituted.
///
/// Params: param[0..4] correspond to I1–I4 default values.
/// The formula is set at construction time.
///
/// Supported operators: +, -, *, /, parentheses, and functions: min, max, abs, sqrt.
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Formula supports basic arithmetic (+,-,*,/) with parentheses
/// //   and functions min(), max(), abs(), sqrt(). Variables are I1-I4.
/// // TODO: Validate against real Miniserver formula syntax
#[derive(Clone)]
pub struct Formula {
    expression: String,
}

impl Formula {
    pub fn new(expression: &str) -> Self {
        Self {
            expression: expression.to_string(),
        }
    }

    fn evaluate_expr(expr: &str, vars: &[f64; 4]) -> f64 {
        let replaced = expr
            .replace("I1", &vars[0].to_string())
            .replace("I2", &vars[1].to_string())
            .replace("I3", &vars[2].to_string())
            .replace("I4", &vars[3].to_string());
        Self::parse_expression(&replaced).unwrap_or(0.0)
    }

    fn parse_expression(expr: &str) -> Option<f64> {
        let tokens = Self::tokenize(expr)?;
        let mut pos = 0;
        let result = Self::parse_additive(&tokens, &mut pos)?;
        if pos == tokens.len() {
            Some(result)
        } else {
            None
        }
    }

    fn tokenize(expr: &str) -> Option<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = expr.chars().peekable();
        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' => {
                    chars.next();
                }
                '+' => {
                    tokens.push(Token::Op('+'));
                    chars.next();
                }
                '-' => {
                    // Unary minus: at start, after '(' or after operator
                    let is_unary = tokens.is_empty()
                        || matches!(
                            tokens.last(),
                            Some(Token::Op(_)) | Some(Token::LParen) | Some(Token::Comma)
                        );
                    if is_unary {
                        chars.next();
                        // Read the number
                        let mut num_str = String::from("-");
                        while let Some(&c) = chars.peek() {
                            if c.is_ascii_digit() || c == '.' {
                                num_str.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        tokens.push(Token::Num(num_str.parse().ok()?));
                    } else {
                        tokens.push(Token::Op('-'));
                        chars.next();
                    }
                }
                '*' => {
                    tokens.push(Token::Op('*'));
                    chars.next();
                }
                '/' => {
                    tokens.push(Token::Op('/'));
                    chars.next();
                }
                '%' => {
                    tokens.push(Token::Op('%'));
                    chars.next();
                }
                '(' => {
                    tokens.push(Token::LParen);
                    chars.next();
                }
                ')' => {
                    tokens.push(Token::RParen);
                    chars.next();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    chars.next();
                }
                c if c.is_ascii_digit() || c == '.' => {
                    let mut num_str = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() || c == '.' {
                            num_str.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Num(num_str.parse().ok()?));
                }
                c if c.is_ascii_alphabetic() => {
                    let mut name = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            name.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Func(name));
                }
                _ => return None,
            }
        }
        Some(tokens)
    }

    fn parse_additive(tokens: &[Token], pos: &mut usize) -> Option<f64> {
        let mut left = Self::parse_multiplicative(tokens, pos)?;
        while *pos < tokens.len() {
            match tokens.get(*pos) {
                Some(Token::Op('+')) => {
                    *pos += 1;
                    left += Self::parse_multiplicative(tokens, pos)?;
                }
                Some(Token::Op('-')) => {
                    *pos += 1;
                    left -= Self::parse_multiplicative(tokens, pos)?;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_multiplicative(tokens: &[Token], pos: &mut usize) -> Option<f64> {
        let mut left = Self::parse_primary(tokens, pos)?;
        while *pos < tokens.len() {
            match tokens.get(*pos) {
                Some(Token::Op('*')) => {
                    *pos += 1;
                    left *= Self::parse_primary(tokens, pos)?;
                }
                Some(Token::Op('/')) => {
                    *pos += 1;
                    let right = Self::parse_primary(tokens, pos)?;
                    left = if right.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        left / right
                    };
                }
                Some(Token::Op('%')) => {
                    *pos += 1;
                    let right = Self::parse_primary(tokens, pos)?;
                    left = if right.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        left % right
                    };
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_primary(tokens: &[Token], pos: &mut usize) -> Option<f64> {
        match tokens.get(*pos)? {
            Token::Num(n) => {
                let val = *n;
                *pos += 1;
                Some(val)
            }
            Token::LParen => {
                *pos += 1;
                let val = Self::parse_additive(tokens, pos)?;
                if matches!(tokens.get(*pos), Some(Token::RParen)) {
                    *pos += 1;
                }
                Some(val)
            }
            Token::Func(name) => {
                let func_name = name.clone();
                *pos += 1;
                // Expect '('
                if !matches!(tokens.get(*pos), Some(Token::LParen)) {
                    return None;
                }
                *pos += 1;
                let mut args = vec![Self::parse_additive(tokens, pos)?];
                while matches!(tokens.get(*pos), Some(Token::Comma)) {
                    *pos += 1;
                    args.push(Self::parse_additive(tokens, pos)?);
                }
                // Expect ')'
                if matches!(tokens.get(*pos), Some(Token::RParen)) {
                    *pos += 1;
                }
                match func_name.as_str() {
                    "min" => Some(args.into_iter().fold(f64::INFINITY, |a, b| a.min(b))),
                    "max" => Some(args.into_iter().fold(f64::NEG_INFINITY, |a, b| a.max(b))),
                    "abs" => Some(args.first().copied().unwrap_or(0.0).abs()),
                    "sqrt" => {
                        let v = args.first().copied().unwrap_or(0.0);
                        Some(if v < 0.0 { 0.0 } else { v.sqrt() })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum Token {
    Num(f64),
    Op(char),
    LParen,
    RParen,
    Comma,
    Func(String),
}

impl Block for Formula {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let vars = [
            inputs.first().copied().unwrap_or(0.0),
            inputs.get(1).copied().unwrap_or(0.0),
            inputs.get(2).copied().unwrap_or(0.0),
            inputs.get(3).copied().unwrap_or(0.0),
        ];
        vec![Self::evaluate_expr(&self.expression, &vars)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Formula"
    }
}

/// Linear scaler: maps input from [InMin, InMax] to [OutMin, OutMax].
///
/// Params: [InMin, InMax, OutMin, OutMax]
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Linear interpolation without clamping — output may exceed
/// //   [OutMin, OutMax] if input exceeds [InMin, InMax].
/// // TODO: Validate against real Miniserver behavior (clamping behavior)
#[derive(Clone, Copy)]
pub struct AnalogScaler;

impl Block for AnalogScaler {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let in_min = params.first().copied().unwrap_or(0.0);
        let in_max = params.get(1).copied().unwrap_or(1.0);
        let out_min = params.get(2).copied().unwrap_or(0.0);
        let out_max = params.get(3).copied().unwrap_or(1.0);

        let in_range = in_max - in_min;
        if in_range.abs() <= f64::EPSILON {
            return vec![out_min];
        }
        let normalized = (value - in_min) / in_range;
        vec![out_min + normalized * (out_max - out_min)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "AnalogScaler"
    }
}

/// Select one of N inputs by index.
///
/// Inputs: I1..IN are the selectable values. The index input is the last input.
/// Params: param[0] = number of selectable inputs (N). The index is 0-based.
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Index is 0-based, taken from the last input. param[0] = N (count
/// //   of selectable inputs). Out-of-range index clamps to valid range.
/// // TODO: Validate against real Miniserver behavior
#[derive(Clone, Copy)]
pub struct AnalogMultiplexer;

impl Block for AnalogMultiplexer {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let n = params.first().copied().unwrap_or(0.0) as usize;
        if inputs.is_empty() || n == 0 {
            return vec![0.0];
        }
        // Last input is the index selector
        let index_raw = inputs.last().copied().unwrap_or(0.0);
        let index = (index_raw.floor() as usize).min(n.saturating_sub(1));
        vec![inputs.get(index).copied().unwrap_or(0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "AnalogMultiplexer"
    }
}

/// 2-input selector: output = I1 when selector is low, I2 when selector is high.
///
/// Inputs: [I1, I2, selector]
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Digital threshold on selector input; I1 when low, I2 when high.
/// // TODO: Validate against real Miniserver behavior
#[derive(Clone, Copy)]
pub struct AnalogMultiplexer2;

impl Block for AnalogMultiplexer2 {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let i1 = inputs.first().copied().unwrap_or(0.0);
        let i2 = inputs.get(1).copied().unwrap_or(0.0);
        let selector = inputs.get(2).copied().unwrap_or(0.0);
        vec![if is_high(selector) { i2 } else { i1 }]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "AnalogMultiplexer2"
    }
}

/// Increment/decrement by step with optional min/max bounds.
///
/// Inputs: [Up (digital), Down (digital), Reset (digital)]
/// Params: [step, min, max, initial_value]
/// Outputs: [current_value]
///
/// Rising edge on Up increments by step. Rising edge on Down decrements by step.
/// Rising edge on Reset sets value to initial_value.
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Step applied on rising edges of Up/Down inputs. Value clamped to
/// //   [min, max] if both are provided (min < max). Reset sets to initial_value.
/// // TODO: Validate against real Miniserver behavior
#[derive(Clone)]
pub struct AnalogStepper {
    value: f64,
    initialized: bool,
}

impl AnalogStepper {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            initialized: false,
        }
    }
}

impl Default for AnalogStepper {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AnalogStepper {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev: &[Signal],
    ) -> Vec<Signal> {
        let step = params.first().copied().unwrap_or(1.0);
        let min_val = params.get(1).copied().unwrap_or(f64::NEG_INFINITY);
        let max_val = params.get(2).copied().unwrap_or(f64::INFINITY);
        let initial = params.get(3).copied().unwrap_or(0.0);

        if !self.initialized {
            self.value = initial;
            self.initialized = true;
        }

        let up = inputs.first().copied().unwrap_or(0.0);
        let down = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);

        let prev_up = prev.first().copied().unwrap_or(0.0);
        let prev_down = prev.get(1).copied().unwrap_or(0.0);
        let prev_reset = prev.get(2).copied().unwrap_or(0.0);

        if is_high(reset) && !is_high(prev_reset) {
            self.value = initial;
        } else {
            if is_high(up) && !is_high(prev_up) {
                self.value += step;
            }
            if is_high(down) && !is_high(prev_down) {
                self.value -= step;
            }
        }

        self.value = self.value.clamp(min_val, max_val);

        vec![self.value]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let init_byte = if self.initialized { 1u8 } else { 0u8 };
        let mut bytes = serialize_f64s(&[self.value]);
        bytes.push(init_byte);
        Some(bytes)
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(vals) = deserialize_f64s(state, 1) {
            self.value = vals[0];
            self.initialized = state.get(8).copied().unwrap_or(0) != 0;
        }
    }

    fn block_type(&self) -> &str {
        "AnalogStepper"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_supports_inputs_and_params() {
        let mut block = Add;
        assert_eq!(block.eval(&[1.0, 2.0], &[3.0], 0.0, &[]), vec![6.0]);
    }

    #[test]
    fn sub_uses_first_operand_as_base() {
        let mut block = Sub;
        assert_eq!(block.eval(&[10.0, 3.0], &[2.0], 0.0, &[]), vec![5.0]);
    }

    #[test]
    fn mult_defaults_to_zero_without_operands() {
        let mut block = Mult;
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[2.0, 3.0], &[4.0], 0.0, &[]), vec![24.0]);
    }

    #[test]
    fn div_guards_against_zero_division() {
        let mut block = Div;
        assert_eq!(block.eval(&[12.0], &[3.0], 0.0, &[]), vec![4.0]);
        assert_eq!(block.eval(&[12.0], &[0.0], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn mod_basic() {
        let mut block = Mod;
        assert_eq!(block.eval(&[10.0, 3.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[10.0], &[4.0], 0.0, &[]), vec![2.0]);
        assert_eq!(block.eval(&[10.0, 0.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn int_floors_value() {
        let mut block = Int;
        assert_eq!(block.eval(&[3.7], &[], 0.0, &[]), vec![3.0]);
        assert_eq!(block.eval(&[-2.3], &[], 0.0, &[]), vec![-3.0]);
        assert_eq!(block.eval(&[5.0], &[], 0.0, &[]), vec![5.0]);
    }

    #[test]
    fn add4_sums_four_inputs() {
        let mut block = Add4;
        assert_eq!(block.eval(&[1.0, 2.0, 3.0, 4.0], &[], 0.0, &[]), vec![10.0]);
        // Fewer than 4 inputs: sums available
        assert_eq!(block.eval(&[1.0, 2.0], &[], 0.0, &[]), vec![3.0]);
        // More than 4 inputs: only first 4
        assert_eq!(
            block.eval(&[1.0, 2.0, 3.0, 4.0, 100.0], &[], 0.0, &[]),
            vec![10.0]
        );
    }

    #[test]
    fn minmax_finds_min_and_max() {
        let mut block = Minmax;
        assert_eq!(
            block.eval(&[3.0, 1.0, 5.0, 2.0], &[], 0.0, &[]),
            vec![1.0, 5.0]
        );
        assert_eq!(block.eval(&[7.0], &[], 0.0, &[]), vec![7.0, 7.0]);
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0, 0.0]);
    }

    #[test]
    fn aminmax_clamps_value() {
        let mut block = AMinmax::new();
        // Value within bounds
        assert_eq!(block.eval(&[5.0], &[0.0, 10.0], 0.0, &[]), vec![5.0]);
        // Value below min
        assert_eq!(block.eval(&[-3.0], &[0.0, 10.0], 0.0, &[]), vec![0.0]);
        // Value above max
        assert_eq!(block.eval(&[15.0], &[0.0, 10.0], 0.0, &[]), vec![10.0]);
        // Custom bounds
        assert_eq!(block.eval(&[30.0], &[25.0, 55.0], 0.0, &[]), vec![30.0]);
        assert_eq!(block.eval(&[20.0], &[25.0, 55.0], 0.0, &[]), vec![25.0]);
        assert_eq!(block.eval(&[65.0], &[25.0, 55.0], 0.0, &[]), vec![55.0]);
    }

    #[test]
    fn formula_basic_arithmetic() {
        let mut block = Formula::new("I1 + I2 * I3");
        let result = block.eval(&[1.0, 2.0, 3.0], &[], 0.0, &[]);
        assert!((result[0] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn formula_with_functions() {
        let mut block = Formula::new("min(I1, I2)");
        assert_eq!(block.eval(&[5.0, 3.0], &[], 0.0, &[]), vec![3.0]);

        let mut block = Formula::new("max(I1, I2)");
        assert_eq!(block.eval(&[5.0, 3.0], &[], 0.0, &[]), vec![5.0]);

        let mut block = Formula::new("abs(I1)");
        assert_eq!(block.eval(&[-7.0], &[], 0.0, &[]), vec![7.0]);

        let mut block = Formula::new("sqrt(I1)");
        let result = block.eval(&[16.0], &[], 0.0, &[]);
        assert!((result[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn formula_parentheses() {
        let mut block = Formula::new("(I1 + I2) * I3");
        let result = block.eval(&[1.0, 2.0, 3.0], &[], 0.0, &[]);
        assert!((result[0] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn formula_division_by_zero() {
        let mut block = Formula::new("I1 / I2");
        assert_eq!(block.eval(&[10.0, 0.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn analog_scaler_linear_map() {
        let mut block = AnalogScaler;
        // Map [0,10] → [0,100]
        assert_eq!(
            block.eval(&[5.0], &[0.0, 10.0, 0.0, 100.0], 0.0, &[]),
            vec![50.0]
        );
        // Map [0,10] → [100,200]
        assert_eq!(
            block.eval(&[5.0], &[0.0, 10.0, 100.0, 200.0], 0.0, &[]),
            vec![150.0]
        );
        // Zero input range returns out_min
        assert_eq!(
            block.eval(&[5.0], &[5.0, 5.0, 0.0, 100.0], 0.0, &[]),
            vec![0.0]
        );
    }

    #[test]
    fn analog_multiplexer_selects_by_index() {
        let mut block = AnalogMultiplexer;
        // inputs: [val0, val1, val2, index], params: [N=3]
        assert_eq!(
            block.eval(&[10.0, 20.0, 30.0, 0.0], &[3.0], 0.0, &[]),
            vec![10.0]
        );
        assert_eq!(
            block.eval(&[10.0, 20.0, 30.0, 1.0], &[3.0], 0.0, &[]),
            vec![20.0]
        );
        assert_eq!(
            block.eval(&[10.0, 20.0, 30.0, 2.0], &[3.0], 0.0, &[]),
            vec![30.0]
        );
        // Out-of-range index clamps
        assert_eq!(
            block.eval(&[10.0, 20.0, 30.0, 99.0], &[3.0], 0.0, &[]),
            vec![30.0]
        );
    }

    #[test]
    fn analog_multiplexer2_selects_by_digital() {
        let mut block = AnalogMultiplexer2;
        assert_eq!(block.eval(&[10.0, 20.0, 0.0], &[], 0.0, &[]), vec![10.0]);
        assert_eq!(block.eval(&[10.0, 20.0, 1.0], &[], 0.0, &[]), vec![20.0]);
    }

    #[test]
    fn analog_stepper_increments_decrements() {
        let mut block = AnalogStepper::new();
        // Params: step=2, min=-10, max=10, initial=0
        // Rising edge on Up
        let result = block.eval(
            &[1.0, 0.0, 0.0],
            &[2.0, -10.0, 10.0, 0.0],
            0.0,
            &[0.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![2.0]);
        // No edge (still high) — no change
        let result = block.eval(
            &[1.0, 0.0, 0.0],
            &[2.0, -10.0, 10.0, 0.0],
            0.0,
            &[1.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![2.0]);
        // Release and press again
        let result = block.eval(
            &[0.0, 0.0, 0.0],
            &[2.0, -10.0, 10.0, 0.0],
            0.0,
            &[1.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![2.0]);
        let result = block.eval(
            &[1.0, 0.0, 0.0],
            &[2.0, -10.0, 10.0, 0.0],
            0.0,
            &[0.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![4.0]);
    }

    #[test]
    fn analog_stepper_respects_bounds() {
        let mut block = AnalogStepper::new();
        // step=5, min=0, max=10, initial=8
        let result = block.eval(
            &[1.0, 0.0, 0.0],
            &[5.0, 0.0, 10.0, 8.0],
            0.0,
            &[0.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![10.0]); // 8+5=13, clamped to 10
    }

    #[test]
    fn analog_stepper_reset() {
        let mut block = AnalogStepper::new();
        // Initial: step=1, min=-100, max=100, initial=0
        block.eval(
            &[1.0, 0.0, 0.0],
            &[1.0, -100.0, 100.0, 0.0],
            0.0,
            &[0.0, 0.0, 0.0],
        );
        // Reset rising edge
        let result = block.eval(
            &[0.0, 0.0, 1.0],
            &[1.0, -100.0, 100.0, 0.0],
            0.0,
            &[1.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![0.0]);
    }

    #[test]
    fn analog_stepper_state_roundtrip() {
        let mut block = AnalogStepper::new();
        block.eval(
            &[1.0, 0.0, 0.0],
            &[3.0, -100.0, 100.0, 0.0],
            0.0,
            &[0.0, 0.0, 0.0],
        );
        let state = block.state().unwrap();
        let mut restored = AnalogStepper::new();
        restored.restore(&state);
        // One more up
        let result = restored.eval(
            &[1.0, 0.0, 0.0],
            &[3.0, -100.0, 100.0, 0.0],
            0.0,
            &[0.0, 0.0, 0.0],
        );
        assert_eq!(result, vec![6.0]); // 3 + 3
    }
}
