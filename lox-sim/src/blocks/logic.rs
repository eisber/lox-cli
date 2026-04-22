use crate::blocks::{bool_signal, is_high, Block};
use crate::types::Signal;

/// Digital OR: output 1.0 if any input >= 0.5, else 0.0.
#[derive(Clone, Copy)]
pub struct Or;

impl Block for Or {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        vec![bool_signal(inputs.iter().any(|&value| is_high(value)))]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Or"
    }
}

/// Digital NOT: output 1.0 when input is low, else 0.0.
#[derive(Clone, Copy)]
pub struct Not;

impl Block for Not {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        vec![bool_signal(!is_high(value))]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Not"
    }
}

/// Digital XOR: output 1.0 when an odd number of inputs are high.
#[derive(Clone, Copy)]
pub struct Xor;

impl Block for Xor {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let high_count = inputs.iter().filter(|&&value| is_high(value)).count();
        vec![bool_signal(high_count % 2 == 1)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Xor"
    }
}

/// Digital NAND: NOT(AND) — output 0.0 only when all inputs are high.
#[derive(Clone, Copy)]
pub struct Nand;

impl Block for Nand {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let all_high = !inputs.is_empty() && inputs.iter().all(|&v| is_high(v));
        vec![bool_signal(!all_high)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Nand"
    }
}

/// Digital NOR: NOT(OR) — output 1.0 only when no input is high.
#[derive(Clone, Copy)]
pub struct Nor;

impl Block for Nor {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let any_high = inputs.iter().any(|&v| is_high(v));
        vec![bool_signal(!any_high)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Nor"
    }
}

/// Analog equality: output 1.0 when I1 == I2 (within f64 EPSILON).
#[derive(Clone, Copy)]
pub struct Equal;

impl Block for Equal {
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
        vec![bool_signal((left - right).abs() <= f64::EPSILON)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Equal"
    }
}

/// Analog inequality: output 1.0 when I1 != I2 (beyond f64 EPSILON).
#[derive(Clone, Copy)]
pub struct NotEqual;

impl Block for NotEqual {
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
        vec![bool_signal((left - right).abs() > f64::EPSILON)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "NotEqual"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn or_outputs_high_when_any_input_is_high() {
        let mut block = Or;
        assert_eq!(block.eval(&[0.0, 0.49], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[0.0, 0.5], &[], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn not_inverts_digital_signal() {
        let mut block = Not;
        assert_eq!(block.eval(&[0.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn xor_uses_odd_parity() {
        let mut block = Xor;
        assert_eq!(block.eval(&[1.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0, 1.0], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[1.0, 1.0, 1.0], &[], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn nand_is_not_and() {
        let mut block = Nand;
        assert_eq!(block.eval(&[1.0, 1.0], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[1.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[0.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        // Empty inputs: AND is false for empty, so NAND is true
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn nor_is_not_or() {
        let mut block = Nor;
        assert_eq!(block.eval(&[0.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0, 0.0], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[1.0, 1.0], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn equal_compares_two_inputs() {
        let mut block = Equal;
        assert_eq!(block.eval(&[5.0, 5.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[5.0, 6.0], &[], 0.0, &[]), vec![0.0]);
        // Falls back to param for second operand
        assert_eq!(block.eval(&[3.0], &[3.0], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[3.0], &[4.0], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn not_equal_compares_two_inputs() {
        let mut block = NotEqual;
        assert_eq!(block.eval(&[5.0, 5.0], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[5.0, 6.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[3.0], &[3.0], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[3.0], &[4.0], 0.0, &[]), vec![1.0]);
    }
}
