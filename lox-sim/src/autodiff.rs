//! Automatic differentiation primitives for differentiable simulation.
//!
//! Provides:
//! - **Forward-mode AD** via [`DualNumber`] (value + tangent pair)
//! - **Reverse-mode AD** via [`Tape`] / [`TracedValue`] (Wengert tape)
//! - **Smooth relaxations** of step / logic functions for gradient flow

use std::ops;

// ---------------------------------------------------------------------------
// Forward-mode: Dual numbers
// ---------------------------------------------------------------------------

/// Dual number for forward-mode automatic differentiation.
///
/// Carries a primal value and its derivative (tangent) with respect to a
/// single chosen parameter.  Arithmetic operators propagate derivatives via
/// the standard differentiation rules.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DualNumber {
    /// Primal (function) value.
    pub val: f64,
    /// Tangent (derivative w.r.t. the seeded parameter).
    pub dot: f64,
}

impl DualNumber {
    /// Create a constant (derivative = 0).
    pub fn constant(v: f64) -> Self {
        DualNumber { val: v, dot: 0.0 }
    }

    /// Create an independent variable (derivative = 1).
    pub fn variable(v: f64) -> Self {
        DualNumber { val: v, dot: 1.0 }
    }

    /// Smooth step (sigmoid) that approximates a hard threshold comparison.
    ///
    /// Returns σ(β·(self − threshold)).  As β→∞ this approaches the
    /// Heaviside step function.
    pub fn smooth_step(self, threshold: DualNumber, beta: f64) -> DualNumber {
        let diff = self - threshold;
        let s = 1.0 / (1.0 + (-beta * diff.val).exp());
        let ds = beta * s * (1.0 - s);
        DualNumber {
            val: s,
            dot: ds * diff.dot,
        }
    }

    /// sin(self) with derivative cos(self)·dot.
    pub fn sin(self) -> DualNumber {
        DualNumber {
            val: self.val.sin(),
            dot: self.val.cos() * self.dot,
        }
    }

    /// cos(self) with derivative −sin(self)·dot.
    pub fn cos(self) -> DualNumber {
        DualNumber {
            val: self.val.cos(),
            dot: -self.val.sin() * self.dot,
        }
    }

    /// exp(self).
    pub fn exp(self) -> DualNumber {
        let e = self.val.exp();
        DualNumber {
            val: e,
            dot: e * self.dot,
        }
    }

    /// ln(self).
    pub fn ln(self) -> DualNumber {
        DualNumber {
            val: self.val.ln(),
            dot: self.dot / self.val,
        }
    }
}

// --- Arithmetic trait impls for DualNumber ----------------------------------

impl ops::Add for DualNumber {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        DualNumber {
            val: self.val + rhs.val,
            dot: self.dot + rhs.dot,
        }
    }
}

impl ops::Sub for DualNumber {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        DualNumber {
            val: self.val - rhs.val,
            dot: self.dot - rhs.dot,
        }
    }
}

impl ops::Mul for DualNumber {
    type Output = Self;
    /// Product rule: d(f·g) = f'·g + f·g'
    fn mul(self, rhs: Self) -> Self {
        DualNumber {
            val: self.val * rhs.val,
            dot: self.dot * rhs.val + self.val * rhs.dot,
        }
    }
}

impl ops::Div for DualNumber {
    type Output = Self;
    /// Quotient rule: d(f/g) = (f'·g − f·g') / g²
    fn div(self, rhs: Self) -> Self {
        let g2 = rhs.val * rhs.val;
        DualNumber {
            val: self.val / rhs.val,
            dot: (self.dot * rhs.val - self.val * rhs.dot) / g2,
        }
    }
}

impl ops::Neg for DualNumber {
    type Output = Self;
    fn neg(self) -> Self {
        DualNumber {
            val: -self.val,
            dot: -self.dot,
        }
    }
}

// ---------------------------------------------------------------------------
// Reverse-mode: Wengert tape
// ---------------------------------------------------------------------------

/// Index into the [`Tape`] entry list.
pub type TapeIdx = usize;

/// One recorded operation on the Wengert tape.
#[derive(Clone, Debug)]
struct TapeEntry {
    /// Value computed at this node (used during forward pass, stored for debugging).
    #[allow(dead_code)]
    val: f64,
    /// Parent nodes and the local partial derivative ∂self/∂parent.
    parents: Vec<(TapeIdx, f64)>,
}

/// Wengert tape for reverse-mode automatic differentiation.
///
/// Record a computation graph by calling [`leaf`], [`unary`], [`binary`],
/// or the convenience arithmetic helpers.  Then call [`backward`] to
/// compute gradients of one output w.r.t. all recorded values.
pub struct Tape {
    entries: Vec<TapeEntry>,
}

/// A value recorded on a [`Tape`], carrying its tape index and cached value.
#[derive(Clone, Copy, Debug)]
pub struct TracedValue {
    pub idx: TapeIdx,
    pub val: f64,
}

impl Tape {
    /// Create an empty tape pre-allocated for ~4K entries.
    pub fn new() -> Self {
        Tape {
            entries: Vec::with_capacity(4096),
        }
    }

    /// Number of recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the tape is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Record a leaf value (input or parameter — no parents).
    pub fn leaf(&mut self, val: f64) -> TracedValue {
        let idx = self.entries.len();
        self.entries.push(TapeEntry {
            val,
            parents: Vec::new(),
        });
        TracedValue { idx, val }
    }

    /// Record a unary operation:  result = f(a), with ∂result/∂a = grad_a.
    pub fn unary(&mut self, a: TracedValue, result_val: f64, grad_a: f64) -> TracedValue {
        let idx = self.entries.len();
        self.entries.push(TapeEntry {
            val: result_val,
            parents: vec![(a.idx, grad_a)],
        });
        TracedValue {
            idx,
            val: result_val,
        }
    }

    /// Record a binary operation:  result = f(a, b).
    pub fn binary(
        &mut self,
        a: TracedValue,
        b: TracedValue,
        result_val: f64,
        grad_a: f64,
        grad_b: f64,
    ) -> TracedValue {
        let idx = self.entries.len();
        self.entries.push(TapeEntry {
            val: result_val,
            parents: vec![(a.idx, grad_a), (b.idx, grad_b)],
        });
        TracedValue {
            idx,
            val: result_val,
        }
    }

    // --- Convenience arithmetic on traced values ----------------------------

    pub fn add(&mut self, a: TracedValue, b: TracedValue) -> TracedValue {
        self.binary(a, b, a.val + b.val, 1.0, 1.0)
    }

    pub fn sub(&mut self, a: TracedValue, b: TracedValue) -> TracedValue {
        self.binary(a, b, a.val - b.val, 1.0, -1.0)
    }

    pub fn mul(&mut self, a: TracedValue, b: TracedValue) -> TracedValue {
        // ∂(a*b)/∂a = b,  ∂(a*b)/∂b = a
        self.binary(a, b, a.val * b.val, b.val, a.val)
    }

    pub fn div(&mut self, a: TracedValue, b: TracedValue) -> TracedValue {
        // ∂(a/b)/∂a = 1/b,  ∂(a/b)/∂b = −a/b²
        let val = a.val / b.val;
        let ga = 1.0 / b.val;
        let gb = -a.val / (b.val * b.val);
        self.binary(a, b, val, ga, gb)
    }

    /// Sigmoid:  σ(β·x) with correct local gradient β·σ·(1−σ).
    pub fn sigmoid(&mut self, x: TracedValue, beta: f64) -> TracedValue {
        let s = 1.0 / (1.0 + (-beta * x.val).exp());
        let ds = beta * s * (1.0 - s);
        self.unary(x, s, ds)
    }

    // --- Reverse pass -------------------------------------------------------

    /// Compute gradients of `output_idx` w.r.t. every tape entry.
    ///
    /// Returns a `Vec<f64>` where `grads[i]` = ∂output / ∂entry_i.
    pub fn backward(&self, output_idx: TapeIdx) -> Vec<f64> {
        let n = self.entries.len();
        let mut grads = vec![0.0; n];
        grads[output_idx] = 1.0; // ∂L/∂L = 1

        for i in (0..n).rev() {
            let grad_i = grads[i];
            if grad_i == 0.0 {
                continue;
            }
            for &(parent_idx, local_grad) in &self.entries[i].parents {
                grads[parent_idx] += grad_i * local_grad; // chain rule
            }
        }
        grads
    }
}

impl Default for Tape {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Smooth relaxation functions (plain f64)
// ---------------------------------------------------------------------------

/// Smooth approximation of the Heaviside step function using a sigmoid.
///
/// `smooth_step(x, threshold, beta)` ≈ 1 if x > threshold, ≈ 0 otherwise.
/// `beta` controls sharpness: β=1 is gentle, β=100 is nearly binary.
pub fn smooth_step(x: f64, threshold: f64, beta: f64) -> f64 {
    1.0 / (1.0 + (-beta * (x - threshold)).exp())
}

/// Smooth AND: probabilistic `a * b`.
///
/// Exact for {0,1} inputs (0·0=0, 0·1=0, 1·0=0, 1·1=1).
/// For intermediate values, returns a smoothly varying product.
pub fn smooth_and(a: f64, b: f64) -> f64 {
    a * b
}

/// Smooth OR: probabilistic `a + b − a·b`.
///
/// Exact for {0,1} inputs.  Equivalent to `1 − (1−a)(1−b)`.
pub fn smooth_or(a: f64, b: f64) -> f64 {
    a + b - a * b
}

/// Smooth NOT: `1 − a`.
///
/// Exact for {0,1}, smoothly interpolates for intermediate values.
pub fn smooth_not(a: f64) -> f64 {
    1.0 - a
}

/// Smooth XOR: `a + b − 2·a·b`.
pub fn smooth_xor(a: f64, b: f64) -> f64 {
    a + b - 2.0 * a * b
}

/// N-ary smooth AND: product of all inputs.
pub fn smooth_and_n(inputs: &[f64]) -> f64 {
    inputs.iter().product()
}

/// N-ary smooth OR: `1 − Π(1 − xᵢ)`.
pub fn smooth_or_n(inputs: &[f64]) -> f64 {
    1.0 - inputs.iter().map(|x| 1.0 - x).product::<f64>()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- DualNumber basic arithmetic -----------------------------------------

    #[test]
    fn dual_add() {
        let a = DualNumber::variable(3.0);
        let b = DualNumber::constant(5.0);
        let c = a + b;
        assert!((c.val - 8.0).abs() < f64::EPSILON);
        assert!((c.dot - 1.0).abs() < f64::EPSILON); // d/dx (x+5) = 1
    }

    #[test]
    fn dual_sub() {
        let a = DualNumber::variable(10.0);
        let b = DualNumber::constant(3.0);
        let c = a - b;
        assert!((c.val - 7.0).abs() < f64::EPSILON);
        assert!((c.dot - 1.0).abs() < f64::EPSILON); // d/dx (x-3) = 1
    }

    #[test]
    fn dual_mul_product_rule() {
        // f(x) = x * x at x=4 → val=16, dot=2*4=8
        let x = DualNumber::variable(4.0);
        let c = x * x;
        assert!((c.val - 16.0).abs() < f64::EPSILON);
        assert!((c.dot - 8.0).abs() < f64::EPSILON); // d/dx x² = 2x = 8
    }

    #[test]
    fn dual_div_quotient_rule() {
        // f(x) = x / 2 at x=6 → val=3, dot=0.5
        let x = DualNumber::variable(6.0);
        let two = DualNumber::constant(2.0);
        let c = x / two;
        assert!((c.val - 3.0).abs() < f64::EPSILON);
        assert!((c.dot - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn dual_chain_rule() {
        // f(x) = (x + 1) * (x + 1) at x=2 → val=9, dot=2*(x+1)=6
        let x = DualNumber::variable(2.0);
        let one = DualNumber::constant(1.0);
        let xp1 = x + one;
        let c = xp1 * xp1;
        assert!((c.val - 9.0).abs() < f64::EPSILON);
        assert!((c.dot - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dual_neg() {
        let x = DualNumber::variable(3.0);
        let c = -x;
        assert!((c.val - -3.0).abs() < f64::EPSILON);
        assert!((c.dot - -1.0).abs() < f64::EPSILON);
    }

    // -- Smooth step --------------------------------------------------------

    #[test]
    fn smooth_step_approaches_hard_step() {
        // At x=5, threshold=3: well above → should be near 1.0
        assert!(smooth_step(5.0, 3.0, 1.0) > 0.8);
        assert!(smooth_step(5.0, 3.0, 10.0) > 0.99);
        assert!(smooth_step(5.0, 3.0, 100.0) > 0.999);

        // At x=1, threshold=3: well below → should be near 0.0
        assert!(smooth_step(1.0, 3.0, 1.0) < 0.2);
        assert!(smooth_step(1.0, 3.0, 10.0) < 0.01);
        assert!(smooth_step(1.0, 3.0, 100.0) < 0.001);

        // At x=threshold: always exactly 0.5
        assert!((smooth_step(3.0, 3.0, 1.0) - 0.5).abs() < f64::EPSILON);
        assert!((smooth_step(3.0, 3.0, 100.0) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn smooth_step_is_monotonic() {
        let beta = 10.0;
        let threshold = 5.0;
        let mut prev = smooth_step(-10.0, threshold, beta);
        for i in -99..100 {
            let x = i as f64 * 0.1;
            let cur = smooth_step(x, threshold, beta);
            assert!(cur >= prev - f64::EPSILON, "not monotonic at x={x}");
            prev = cur;
        }
    }

    #[test]
    fn dual_smooth_step_derivative() {
        // The derivative of σ(β(x−t)) w.r.t. x at x=t is β/4
        let x = DualNumber::variable(5.0);
        let t = DualNumber::constant(5.0);
        let beta = 10.0;
        let s = x.smooth_step(t, beta);
        assert!((s.val - 0.5).abs() < f64::EPSILON);
        assert!((s.dot - beta / 4.0).abs() < 1e-10);
    }

    // -- Smooth logic -------------------------------------------------------

    #[test]
    fn smooth_logic_exact_for_binary() {
        // AND truth table
        assert!((smooth_and(0.0, 0.0)).abs() < f64::EPSILON);
        assert!((smooth_and(0.0, 1.0)).abs() < f64::EPSILON);
        assert!((smooth_and(1.0, 0.0)).abs() < f64::EPSILON);
        assert!((smooth_and(1.0, 1.0) - 1.0).abs() < f64::EPSILON);

        // OR truth table
        assert!((smooth_or(0.0, 0.0)).abs() < f64::EPSILON);
        assert!((smooth_or(0.0, 1.0) - 1.0).abs() < f64::EPSILON);
        assert!((smooth_or(1.0, 0.0) - 1.0).abs() < f64::EPSILON);
        assert!((smooth_or(1.0, 1.0) - 1.0).abs() < f64::EPSILON);

        // NOT
        assert!((smooth_not(0.0) - 1.0).abs() < f64::EPSILON);
        assert!((smooth_not(1.0)).abs() < f64::EPSILON);

        // XOR
        assert!((smooth_xor(0.0, 0.0)).abs() < f64::EPSILON);
        assert!((smooth_xor(0.0, 1.0) - 1.0).abs() < f64::EPSILON);
        assert!((smooth_xor(1.0, 0.0) - 1.0).abs() < f64::EPSILON);
        assert!((smooth_xor(1.0, 1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn smooth_logic_intermediate_values() {
        // Intermediate values should be in [0, 1]
        let a = 0.7;
        let b = 0.4;
        let and_val = smooth_and(a, b);
        let or_val = smooth_or(a, b);
        assert!(and_val >= 0.0 && and_val <= 1.0);
        assert!(or_val >= 0.0 && or_val <= 1.0);
        // OR ≥ AND always
        assert!(or_val >= and_val);
    }

    #[test]
    fn smooth_nary_logic() {
        let inputs = vec![1.0, 1.0, 1.0];
        assert!((smooth_and_n(&inputs) - 1.0).abs() < f64::EPSILON);
        assert!((smooth_or_n(&inputs) - 1.0).abs() < f64::EPSILON);

        let inputs2 = vec![0.0, 0.0, 0.0];
        assert!((smooth_and_n(&inputs2)).abs() < f64::EPSILON);
        assert!((smooth_or_n(&inputs2)).abs() < f64::EPSILON);

        // One high in OR → 1.0
        let inputs3 = vec![0.0, 1.0, 0.0];
        assert!((smooth_or_n(&inputs3) - 1.0).abs() < f64::EPSILON);
        // One low in AND → 0.0
        assert!((smooth_and_n(&inputs3)).abs() < f64::EPSILON);
    }

    // -- Tape / reverse mode ------------------------------------------------

    #[test]
    fn tape_leaf_and_backward() {
        let mut tape = Tape::new();
        let x = tape.leaf(3.0);
        let grads = tape.backward(x.idx);
        assert!((grads[x.idx] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tape_add_backward() {
        let mut tape = Tape::new();
        let x = tape.leaf(2.0);
        let y = tape.leaf(3.0);
        let z = tape.add(x, y); // z = x + y
        let grads = tape.backward(z.idx);
        assert!((grads[x.idx] - 1.0).abs() < f64::EPSILON); // ∂z/∂x = 1
        assert!((grads[y.idx] - 1.0).abs() < f64::EPSILON); // ∂z/∂y = 1
    }

    #[test]
    fn tape_mul_backward() {
        let mut tape = Tape::new();
        let x = tape.leaf(3.0);
        let y = tape.leaf(4.0);
        let z = tape.mul(x, y); // z = x * y = 12
        assert!((z.val - 12.0).abs() < f64::EPSILON);
        let grads = tape.backward(z.idx);
        assert!((grads[x.idx] - 4.0).abs() < f64::EPSILON); // ∂z/∂x = y = 4
        assert!((grads[y.idx] - 3.0).abs() < f64::EPSILON); // ∂z/∂y = x = 3
    }

    #[test]
    fn tape_compound_expression() {
        // f(x, y) = x*y + x  at (x=2, y=3) → f=8
        // ∂f/∂x = y + 1 = 4,  ∂f/∂y = x = 2
        let mut tape = Tape::new();
        let x = tape.leaf(2.0);
        let y = tape.leaf(3.0);
        let xy = tape.mul(x, y);
        let f = tape.add(xy, x);
        assert!((f.val - 8.0).abs() < f64::EPSILON);
        let grads = tape.backward(f.idx);
        assert!((grads[x.idx] - 4.0).abs() < f64::EPSILON);
        assert!((grads[y.idx] - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tape_div_backward() {
        // f = x / y at (x=6, y=3) → f=2
        // ∂f/∂x = 1/y = 1/3,  ∂f/∂y = -x/y² = -6/9 = -2/3
        let mut tape = Tape::new();
        let x = tape.leaf(6.0);
        let y = tape.leaf(3.0);
        let f = tape.div(x, y);
        assert!((f.val - 2.0).abs() < f64::EPSILON);
        let grads = tape.backward(f.idx);
        assert!((grads[x.idx] - 1.0 / 3.0).abs() < 1e-10);
        assert!((grads[y.idx] - (-2.0 / 3.0)).abs() < 1e-10);
    }

    #[test]
    fn tape_sigmoid_backward() {
        // At x=0, σ(β·x)=0.5, gradient = β/4
        let mut tape = Tape::new();
        let x = tape.leaf(0.0);
        let beta = 10.0;
        let s = tape.sigmoid(x, beta);
        assert!((s.val - 0.5).abs() < f64::EPSILON);
        let grads = tape.backward(s.idx);
        assert!((grads[x.idx] - beta / 4.0).abs() < 1e-10);
    }

    #[test]
    fn tape_sub_backward() {
        let mut tape = Tape::new();
        let x = tape.leaf(5.0);
        let y = tape.leaf(2.0);
        let z = tape.sub(x, y); // z = x - y = 3
        assert!((z.val - 3.0).abs() < f64::EPSILON);
        let grads = tape.backward(z.idx);
        assert!((grads[x.idx] - 1.0).abs() < f64::EPSILON);
        assert!((grads[y.idx] - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn tape_is_empty() {
        let tape = Tape::new();
        assert!(tape.is_empty());
        assert_eq!(tape.len(), 0);
    }

    #[test]
    fn tape_len_grows() {
        let mut tape = Tape::new();
        let a = tape.leaf(1.0);
        assert_eq!(tape.len(), 1);
        let b = tape.leaf(2.0);
        assert_eq!(tape.len(), 2);
        let _ = tape.add(a, b);
        assert_eq!(tape.len(), 3);
    }
}
