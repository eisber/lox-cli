// ============================================================================
// DIFFERENTIABLE LOXONE SIMULATOR — DESIGN & REFERENCE IMPLEMENTATION
// ============================================================================
//
// This file is a compilable design sketch. It defines the core abstractions
// for a differentiable simulator of Loxone block graphs. The goal: express
// the entire Loxone PLC program as a differentiable function f(params, inputs, t)
// → outputs, so we can compute ∂outputs/∂params via automatic differentiation.
//
// Table of contents:
//   1. Value types & automatic differentiation
//   2. Smooth relaxations of discontinuous operations
//   3. Block trait & core block implementations
//   4. Computational graph & topological execution
//   5. Time-stepping & backprop through time (BPTT)
//   6. Optimization algorithms
//   7. Multi-objective optimization & Pareto fronts
//   8. Interval arithmetic for exhaustive analysis
//   9. Integration with existing Loxone XML config
//
// Build: This is a design document in .rs form. To validate syntax:
//   rustc --edition 2021 --crate-type lib docs/design/differentiable-sim.rs
//   (will fail on missing deps, but type-checks the logic)

#![allow(dead_code, unused_variables, clippy::needless_return)]

use std::collections::HashMap;
use std::f64::consts::PI;

// ============================================================================
// 1. VALUE TYPES & AUTOMATIC DIFFERENTIATION
// ============================================================================
//
// We support two AD modes:
//
// FORWARD MODE (dual numbers): Good for few parameters, many outputs.
//   Each value carries its derivative w.r.t. ONE parameter.
//   Cost: O(P) forward passes for P parameters.
//   Use case: sensitivity analysis of individual parameters.
//
// REVERSE MODE (Wengert tape): Good for many parameters, few outputs.
//   Record operations on a tape, then backpropagate.
//   Cost: O(1) backward pass for ANY number of parameters.
//   Use case: gradient-based optimization with 100+ parameters.
//
// The Block trait is generic over the value type, so the same block
// implementations work with f64 (no AD), Dual (forward), or Traced (reverse).

// ---------------------------------------------------------------------------
// 1a. Dual numbers (forward-mode AD)
// ---------------------------------------------------------------------------

/// A dual number: value + derivative. Implements forward-mode AD.
/// For a function f(x), if we set x = Dual(x₀, 1.0), then
/// f(Dual(x₀, 1.0)) = Dual(f(x₀), f'(x₀)).
#[derive(Clone, Copy, Debug)]
pub struct Dual {
    pub val: f64, // Primal value
    pub dot: f64, // Tangent (derivative w.r.t. seed parameter)
}

impl Dual {
    pub fn constant(v: f64) -> Self {
        Dual { val: v, dot: 0.0 }
    }
    pub fn variable(v: f64) -> Self {
        Dual { val: v, dot: 1.0 }
    }
}

impl std::ops::Add for Dual {
    type Output = Dual;
    fn add(self, rhs: Dual) -> Dual {
        Dual {
            val: self.val + rhs.val,
            dot: self.dot + rhs.dot,
        }
    }
}

impl std::ops::Sub for Dual {
    type Output = Dual;
    fn sub(self, rhs: Dual) -> Dual {
        Dual {
            val: self.val - rhs.val,
            dot: self.dot - rhs.dot,
        }
    }
}

impl std::ops::Mul for Dual {
    type Output = Dual;
    fn mul(self, rhs: Dual) -> Dual {
        // Product rule: (f·g)' = f'·g + f·g'
        Dual {
            val: self.val * rhs.val,
            dot: self.dot * rhs.val + self.val * rhs.dot,
        }
    }
}

impl std::ops::Div for Dual {
    type Output = Dual;
    fn div(self, rhs: Dual) -> Dual {
        // Quotient rule: (f/g)' = (f'·g - f·g') / g²
        let g2 = rhs.val * rhs.val;
        Dual {
            val: self.val / rhs.val,
            dot: (self.dot * rhs.val - self.val * rhs.dot) / g2,
        }
    }
}

// ---------------------------------------------------------------------------
// 1b. Reverse-mode AD (Wengert tape)
// ---------------------------------------------------------------------------

/// Index into the tape. Every intermediate value gets a TapeIdx.
type TapeIdx = usize;

/// One entry on the Wengert tape: records how a value was computed.
#[derive(Clone, Debug)]
struct TapeEntry {
    /// The primal (forward) value.
    val: f64,
    /// Partial derivatives w.r.t. parent values.
    /// Each (parent_idx, ∂self/∂parent) pair.
    parents: Vec<(TapeIdx, f64)>,
}

/// The computation tape. Append-only during forward pass.
/// Call `backward()` to compute all gradients via reverse accumulation.
pub struct Tape {
    entries: Vec<TapeEntry>,
}

/// A value tracked on the tape.
#[derive(Clone, Copy, Debug)]
pub struct Traced {
    pub idx: TapeIdx,
    pub val: f64,
}

impl Tape {
    pub fn new() -> Self {
        Tape {
            entries: Vec::with_capacity(4096),
        }
    }

    /// Create a leaf (input or parameter). No parents.
    pub fn leaf(&mut self, val: f64) -> Traced {
        let idx = self.entries.len();
        self.entries.push(TapeEntry {
            val,
            parents: vec![],
        });
        Traced { idx, val }
    }

    /// Record a unary operation: result = f(a), with ∂result/∂a = grad.
    pub fn unary(&mut self, a: Traced, result_val: f64, grad_a: f64) -> Traced {
        let idx = self.entries.len();
        self.entries.push(TapeEntry {
            val: result_val,
            parents: vec![(a.idx, grad_a)],
        });
        Traced {
            idx,
            val: result_val,
        }
    }

    /// Record a binary operation: result = f(a, b).
    pub fn binary(
        &mut self,
        a: Traced,
        b: Traced,
        result_val: f64,
        grad_a: f64,
        grad_b: f64,
    ) -> Traced {
        let idx = self.entries.len();
        self.entries.push(TapeEntry {
            val: result_val,
            parents: vec![(a.idx, grad_a), (b.idx, grad_b)],
        });
        Traced {
            idx,
            val: result_val,
        }
    }

    /// Reverse-mode backpropagation. Returns gradient for every tape entry.
    /// `output_idx` is the scalar loss we differentiate.
    pub fn backward(&self, output_idx: TapeIdx) -> Vec<f64> {
        let n = self.entries.len();
        let mut grads = vec![0.0f64; n];
        grads[output_idx] = 1.0; // ∂L/∂L = 1

        // Walk tape in reverse (reverse topological order by construction)
        for i in (0..n).rev() {
            let grad_i = grads[i];
            if grad_i == 0.0 {
                continue;
            }
            for &(parent_idx, local_grad) in &self.entries[i].parents {
                grads[parent_idx] += grad_i * local_grad; // Chain rule accumulation
            }
        }
        grads
    }
}

// Tape-aware arithmetic: each op records itself
impl Tape {
    pub fn add(&mut self, a: Traced, b: Traced) -> Traced {
        self.binary(a, b, a.val + b.val, 1.0, 1.0)
    }

    pub fn sub(&mut self, a: Traced, b: Traced) -> Traced {
        self.binary(a, b, a.val - b.val, 1.0, -1.0)
    }

    pub fn mul(&mut self, a: Traced, b: Traced) -> Traced {
        // ∂(a*b)/∂a = b, ∂(a*b)/∂b = a
        self.binary(a, b, a.val * b.val, b.val, a.val)
    }

    pub fn div(&mut self, a: Traced, b: Traced) -> Traced {
        // ∂(a/b)/∂a = 1/b, ∂(a/b)/∂b = -a/b²
        self.binary(a, b, a.val / b.val, 1.0 / b.val, -a.val / (b.val * b.val))
    }

    pub fn sigmoid(&mut self, x: Traced, sharpness: f64) -> Traced {
        let s = 1.0 / (1.0 + (-sharpness * x.val).exp());
        let ds = sharpness * s * (1.0 - s); // sigmoid derivative
        self.unary(x, s, ds)
    }

    pub fn clamp(&mut self, x: Traced, lo: f64, hi: f64) -> Traced {
        if x.val < lo {
            self.unary(x, lo, 0.0) // gradient killed at boundary
        } else if x.val > hi {
            self.unary(x, hi, 0.0)
        } else {
            self.unary(x, x.val, 1.0) // pass-through
        }
    }

    pub fn max(&mut self, a: Traced, b: Traced) -> Traced {
        if a.val >= b.val {
            self.binary(a, b, a.val, 1.0, 0.0)
        } else {
            self.binary(a, b, b.val, 0.0, 1.0)
        }
    }

    pub fn min(&mut self, a: Traced, b: Traced) -> Traced {
        if a.val <= b.val {
            self.binary(a, b, a.val, 1.0, 0.0)
        } else {
            self.binary(a, b, b.val, 0.0, 1.0)
        }
    }

    pub fn abs(&mut self, x: Traced) -> Traced {
        let grad = if x.val >= 0.0 { 1.0 } else { -1.0 };
        self.unary(x, x.val.abs(), grad)
    }

    /// Smooth approximation of max(a, b) using LogSumExp.
    /// Approaches true max as sharpness → ∞.
    pub fn smooth_max(&mut self, a: Traced, b: Traced, sharpness: f64) -> Traced {
        let k = sharpness;
        let ea = (k * a.val).exp();
        let eb = (k * b.val).exp();
        let result = (ea + eb).ln() / k;
        let ga = ea / (ea + eb); // softmax weight
        let gb = eb / (ea + eb);
        self.binary(a, b, result, ga, gb)
    }
}

// ============================================================================
// 2. SMOOTH RELAXATIONS OF DISCONTINUOUS OPERATIONS
// ============================================================================
//
// The core challenge: Loxone blocks like GreaterEqual, And, Or are step
// functions with zero gradient almost everywhere. We replace them with smooth
// approximations parameterized by a "sharpness" β that controls the tradeoff
// between accuracy and differentiability.
//
// As β → ∞, the smooth version converges to the exact step function.
// During optimization, we can anneal β: start soft (β=1) for broad gradients,
// then harden (β=100) to converge to the true behavior.

/// Smooth relaxation functions. All return values in [0, 1].
pub struct SmoothOps;

impl SmoothOps {
    // -----------------------------------------------------------------------
    // Comparison operators
    // -----------------------------------------------------------------------

    /// Smooth step function: approximates (x >= threshold) ? 1.0 : 0.0
    /// Uses sigmoid: σ(β(x - threshold))
    ///
    /// Maps to: GreaterEqual, Less blocks
    /// Loxone GreaterEqual: Q = 1 if Input1 >= Input2
    pub fn greater_equal(x: f64, threshold: f64, beta: f64) -> f64 {
        1.0 / (1.0 + (-beta * (x - threshold)).exp())
    }

    /// Derivative: ∂σ/∂x = β·σ(1-σ)
    pub fn greater_equal_grad(x: f64, threshold: f64, beta: f64) -> (f64, f64, f64) {
        let s = Self::greater_equal(x, threshold, beta);
        let ds_dx = beta * s * (1.0 - s);
        let ds_dthreshold = -ds_dx; // ∂σ/∂threshold = -∂σ/∂x
        (s, ds_dx, ds_dthreshold)
    }

    // -----------------------------------------------------------------------
    // Logic gates — probabilistic relaxation
    // -----------------------------------------------------------------------
    //
    // Treat boolean values as probabilities in [0,1].
    // AND(a,b) = a·b          (both must be high)
    // OR(a,b)  = a+b - a·b    (inclusion-exclusion, = 1-(1-a)(1-b))
    // NOT(a)   = 1 - a
    // XOR(a,b) = a+b - 2·a·b  (exactly one)
    //
    // These are smooth AND exact when inputs are {0, 1}.
    // Gradient flows naturally through multiplication.

    /// Smooth AND: a·b
    /// Maps to: And block (I1, I2 → Q)
    pub fn and(a: f64, b: f64) -> f64 {
        a * b
    }

    /// Smooth OR: 1 - (1-a)(1-b)
    /// Maps to: Or block (I1, I2 → Q)
    pub fn or(a: f64, b: f64) -> f64 {
        a + b - a * b
    }

    /// Smooth NOT: 1 - a
    /// Maps to: Not block
    pub fn not(a: f64) -> f64 {
        1.0 - a
    }

    /// Smooth XOR: a + b - 2ab
    /// Maps to: Xor block
    pub fn xor(a: f64, b: f64) -> f64 {
        a + b - 2.0 * a * b
    }

    /// N-ary AND: product of all inputs
    pub fn and_n(inputs: &[f64]) -> f64 {
        inputs.iter().product()
    }

    /// N-ary OR: 1 - product(1 - xi)
    pub fn or_n(inputs: &[f64]) -> f64 {
        1.0 - inputs.iter().map(|x| 1.0 - x).product::<f64>()
    }

    // -----------------------------------------------------------------------
    // Hysteresis — smooth approximation
    // -----------------------------------------------------------------------
    //
    // AnalogThresholdTrigger has ON and OFF thresholds (ON > OFF).
    // Standard behavior:
    //   if input >= ON:  output = 1
    //   if input <= OFF: output = 0
    //   else: hold previous state
    //
    // Smooth version: blend between ON-sigmoid and OFF-sigmoid weighted
    // by the previous output (state). This creates a smooth hysteresis loop.

    /// Smooth hysteresis. prev_output should be the output from previous timestep.
    /// Maps to: AnalogThresholdTrigger (Input → Q, with On/Off params)
    pub fn hysteresis(
        input: f64,
        on_thresh: f64,
        off_thresh: f64,
        prev_output: f64,
        beta: f64,
    ) -> f64 {
        let rising = Self::greater_equal(input, on_thresh, beta);
        let falling = Self::greater_equal(input, off_thresh, beta);
        // When prev_output ≈ 0 (was OFF): need to cross ON threshold to turn on
        // When prev_output ≈ 1 (was ON): need to drop below OFF threshold to turn off
        // Blend: output = prev * falling + (1-prev) * rising
        // This creates the hysteresis gap smoothly.
        prev_output * falling + (1.0 - prev_output) * rising
    }

    // -----------------------------------------------------------------------
    // Discrete state selection — Gumbel-Softmax
    // -----------------------------------------------------------------------
    //
    // LightController2 scene selection, State block.
    // N discrete states → relax to a probability distribution using softmax.

    /// Smooth state selection: given N state trigger inputs, return soft
    /// index (as weighted sum of indices).
    /// Maps to: State block (I1..I20 → AQ)
    pub fn soft_state_select(triggers: &[f64], beta: f64) -> f64 {
        let max_t = triggers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let weights: Vec<f64> = triggers
            .iter()
            .map(|t| (beta * (t - max_t)).exp())
            .collect();
        let sum: f64 = weights.iter().sum();
        // Weighted average of state indices
        weights
            .iter()
            .enumerate()
            .map(|(i, w)| (i as f64) * w / sum)
            .sum()
    }

    // -----------------------------------------------------------------------
    // FlipFlop — differentiable latch
    // -----------------------------------------------------------------------
    //
    // FlipFlop has Set (S) and Reset (R) inputs.
    // State: Q(t) = S OR (Q(t-1) AND NOT R)
    // Smooth: q = or(s, and(prev_q, not(r)))

    /// Smooth FlipFlop (SR latch).
    /// Maps to: FlipFlop block (InputS, InputR → Q)
    pub fn flip_flop(set: f64, reset: f64, prev_q: f64) -> f64 {
        Self::or(set, Self::and(prev_q, Self::not(reset)))
    }

    // -----------------------------------------------------------------------
    // Counter — differentiable accumulator
    // -----------------------------------------------------------------------
    //
    // Counter increments on trigger, resets at EndValue.
    // Smooth: accumulate trigger probability, soft-wrap at EndValue.

    /// Smooth counter step. Returns new count value.
    /// Maps to: Counter block (Trigger → Q, AQ; EndValue param)
    pub fn counter_step(
        trigger: f64,
        prev_count: f64,
        end_value: f64,
        beta: f64,
    ) -> f64 {
        let incremented = prev_count + trigger; // trigger ∈ [0,1] acts as probability
        // Soft wrap: use modular arithmetic via sawtooth approximation
        // When incremented approaches end_value, smoothly wrap to 0
        let overflow = Self::greater_equal(incremented, end_value, beta);
        incremented * (1.0 - overflow) // smoothly zero-out when hitting end
    }

    // -----------------------------------------------------------------------
    // Scaler / linear interpolation
    // -----------------------------------------------------------------------

    /// Linear scaler: maps [in_min, in_max] → [out_min, out_max] with clamp.
    /// Maps to: Scaler block
    pub fn scaler(
        input: f64,
        in_min: f64,
        in_max: f64,
        out_min: f64,
        out_max: f64,
    ) -> f64 {
        let t = (input - in_min) / (in_max - in_min);
        let t_clamped = t.clamp(0.0, 1.0);
        out_min + t_clamped * (out_max - out_min)
    }
}

// ============================================================================
// 3. BLOCK TRAIT & CORE BLOCK IMPLEMENTATIONS
// ============================================================================
//
// Each Loxone block type implements the Block trait. The trait is designed
// for tape-based reverse-mode AD: it takes a &mut Tape and Traced values.
//
// State: Blocks that have internal state (timers, flip-flops, hysteresis)
// carry it in a BlockState map. This enables BPTT: state flows forward
// through time steps, and gradients flow backward.

/// Connector values for a single block at a single timestep.
/// Keys are connector names ("I1", "Q", "AQ", "On", etc.)
pub type ConnectorMap = HashMap<String, Traced>;

/// Per-block persistent state between timesteps.
#[derive(Clone, Debug, Default)]
pub struct BlockState {
    pub values: HashMap<String, f64>,
}

/// The core block trait. Every Loxone block type implements this.
pub trait Block {
    /// Block type name (e.g., "And", "GreaterEqual", "OnPulseDelay")
    fn type_name(&self) -> &str;

    /// Execute one timestep. Reads inputs + params from `inputs`, writes
    /// outputs to the returned ConnectorMap. May read/write `state`.
    ///
    /// `tape`: the AD tape for recording operations
    /// `inputs`: connector values (both wired inputs and parameter defaults)
    /// `state`: mutable per-block state (persists across timesteps)
    /// `dt`: timestep duration in seconds
    /// `beta`: sharpness parameter for smooth relaxations
    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        state: &mut BlockState,
        dt: f64,
        beta: f64,
    ) -> ConnectorMap;

    /// List of input connector names
    fn input_names(&self) -> &[&str];
    /// List of output connector names
    fn output_names(&self) -> &[&str];
    /// List of parameter names with defaults
    fn param_defaults(&self) -> &[(&str, f64)];
}

// ---------------------------------------------------------------------------
// 3a. Logic gates
// ---------------------------------------------------------------------------

pub struct AndBlock;

impl Block for AndBlock {
    fn type_name(&self) -> &str {
        "And"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        _state: &mut BlockState,
        _dt: f64,
        _beta: f64,
    ) -> ConnectorMap {
        let i1 = inputs["I1"];
        let i2 = inputs["I2"];
        // AND = product (smooth, exact for binary inputs)
        let q = tape.mul(i1, i2);
        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["I1", "I2"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[]
    }
}

pub struct OrBlock;

impl Block for OrBlock {
    fn type_name(&self) -> &str {
        "Or"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        _state: &mut BlockState,
        _dt: f64,
        _beta: f64,
    ) -> ConnectorMap {
        let a = inputs["I1"];
        let b = inputs["I2"];
        // OR = a + b - ab = 1 - (1-a)(1-b)
        let ab = tape.mul(a, b);
        let sum = tape.add(a, b);
        let q = tape.sub(sum, ab);
        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["I1", "I2"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[]
    }
}

pub struct NotBlock;

impl Block for NotBlock {
    fn type_name(&self) -> &str {
        "Not"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        _state: &mut BlockState,
        _dt: f64,
        _beta: f64,
    ) -> ConnectorMap {
        let a = inputs["I1"];
        let one = tape.leaf(1.0);
        let q = tape.sub(one, a);
        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["I1"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[]
    }
}

// ---------------------------------------------------------------------------
// 3b. Comparison / threshold blocks
// ---------------------------------------------------------------------------

pub struct GreaterEqualBlock;

impl Block for GreaterEqualBlock {
    fn type_name(&self) -> &str {
        "GreaterEqual"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        _state: &mut BlockState,
        _dt: f64,
        beta: f64,
    ) -> ConnectorMap {
        let input1 = inputs["Input1"];
        let input2 = inputs["Input2"];
        // Q = σ(β(Input1 - Input2))
        let diff = tape.sub(input1, input2);
        let q = tape.sigmoid(diff, beta);
        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["Input1"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("Input2", 0.0)] // threshold default
    }
}

// ---------------------------------------------------------------------------
// 3c. Analog threshold trigger (hysteresis)
// ---------------------------------------------------------------------------

pub struct AnalogThresholdTriggerBlock;

impl Block for AnalogThresholdTriggerBlock {
    fn type_name(&self) -> &str {
        "AnalogThresholdTrigger"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        state: &mut BlockState,
        _dt: f64,
        beta: f64,
    ) -> ConnectorMap {
        let input = inputs["Input"];
        let on_thresh = inputs["On"];
        let off_thresh = inputs["Off"];

        // Previous output (state carries between timesteps)
        let prev_q = *state.values.get("Q").unwrap_or(&0.0);
        let prev_q_traced = tape.leaf(prev_q);

        // Smooth hysteresis: blend rising/falling edges
        let rising_diff = tape.sub(input, on_thresh);
        let rising = tape.sigmoid(rising_diff, beta);

        let falling_diff = tape.sub(input, off_thresh);
        let falling = tape.sigmoid(falling_diff, beta);

        // q = prev * falling + (1 - prev) * rising
        let hold = tape.mul(prev_q_traced, falling);
        let one = tape.leaf(1.0);
        let inv_prev = tape.sub(one, prev_q_traced);
        let activate = tape.mul(inv_prev, rising);
        let q = tape.add(hold, activate);

        // Edge detection outputs
        let prev_val = prev_q_traced;
        let one2 = tape.leaf(1.0);
        let not_prev = tape.sub(one2, prev_val);
        let rising_edge = tape.mul(q, not_prev); // q AND NOT prev
        let one3 = tape.leaf(1.0);
        let not_q = tape.sub(one3, q);
        let falling_edge = tape.mul(prev_val, not_q); // prev AND NOT q

        // Update state for next timestep
        state.values.insert("Q".to_string(), q.val);

        HashMap::from([
            ("Q".to_string(), q),
            ("RisingEdge".to_string(), rising_edge),
            ("FallingEdge".to_string(), falling_edge),
        ])
    }

    fn input_names(&self) -> &[&str] {
        &["Input"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q", "RisingEdge", "FallingEdge"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("On", 1.0), ("Off", 0.5), ("PulseTime", 0.0)]
    }
}

// ---------------------------------------------------------------------------
// 3d. Math blocks
// ---------------------------------------------------------------------------

pub struct AddBlock;

impl Block for AddBlock {
    fn type_name(&self) -> &str {
        "Add"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        _state: &mut BlockState,
        _dt: f64,
        _beta: f64,
    ) -> ConnectorMap {
        let i1 = inputs["Input1"];
        let i2 = inputs["Input2"];
        let q = tape.add(i1, i2);
        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["Input1"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("Input2", 0.0)]
    }
}

pub struct MultBlock;

impl Block for MultBlock {
    fn type_name(&self) -> &str {
        "Mult"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        _state: &mut BlockState,
        _dt: f64,
        _beta: f64,
    ) -> ConnectorMap {
        let i1 = inputs["Input1"];
        let i2 = inputs["Input2"];
        let q = tape.mul(i1, i2);
        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["Input1"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("Input2", 1.0)]
    }
}

// ---------------------------------------------------------------------------
// 3e. Timer blocks — temporal dependencies
// ---------------------------------------------------------------------------
//
// Timers are the key challenge for differentiable simulation. An OnPulseDelay
// fires its output after a configurable delay. In discrete simulation, this
// is a countdown. In differentiable simulation, we model it as a continuous
// decay/charge process.
//
// Key insight: a timer with delay T can be modeled as an exponential charge:
//   charge(t) += input * dt / T
//   output = σ(β(charge - 1.0))
//
// This makes the delay parameter T differentiable: ∂output/∂T exists
// and tells us how sensitive the system is to timing changes.

pub struct OnPulseDelayBlock;

impl Block for OnPulseDelayBlock {
    fn type_name(&self) -> &str {
        "OnPulseDelay"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        state: &mut BlockState,
        dt: f64,
        beta: f64,
    ) -> ConnectorMap {
        let trigger = inputs["InputTrigger"];
        let delay_param = inputs["Time"];

        // Internal charge state: accumulates while trigger is active
        let prev_charge = *state.values.get("charge").unwrap_or(&0.0);
        let prev_charge_traced = tape.leaf(prev_charge);

        // charge += trigger * dt / delay
        // (trigger acts as gate: only charges while input is high)
        let dt_val = tape.leaf(dt);
        let increment = tape.mul(trigger, dt_val);
        let rate = tape.div(increment, delay_param);
        let new_charge = tape.add(prev_charge_traced, rate);

        // Clamp charge to [0, 2] to prevent unbounded growth
        let charge_clamped = tape.clamp(new_charge, 0.0, 2.0);

        // Output fires when charge >= 1.0 (i.e., delay has elapsed)
        let one = tape.leaf(1.0);
        let diff = tape.sub(charge_clamped, one);
        let q = tape.sigmoid(diff, beta);

        // Reset charge when trigger goes low
        let charge_val = charge_clamped.val * trigger.val; // gate by trigger
        state.values.insert("charge".to_string(), charge_val);

        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["InputTrigger"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("Time", 1.0), ("Delay", 0.0)]
    }
}

pub struct MonoflopBlock;

impl Block for MonoflopBlock {
    fn type_name(&self) -> &str {
        "Monoflop"
    }

    /// Monoflop: trigger pulse → output stays high for Time seconds.
    /// Modeled as exponential decay: output = max(prev_output - dt/Time, trigger)
    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        state: &mut BlockState,
        dt: f64,
        beta: f64,
    ) -> ConnectorMap {
        let trigger = inputs["InputTrigger"];
        let time_param = inputs["Time"];

        let prev_remaining = *state.values.get("remaining").unwrap_or(&0.0);
        let prev_remaining_traced = tape.leaf(prev_remaining);

        // Decay: remaining -= dt / Time (normalized)
        let dt_val = tape.leaf(dt);
        let decay = tape.div(dt_val, time_param);
        let decayed = tape.sub(prev_remaining_traced, decay);

        // Re-trigger: remaining = max(decayed, trigger)
        // trigger > 0 restarts the monoflop
        let remaining = tape.smooth_max(decayed, trigger, beta);

        // Output is high while remaining > 0
        let q = tape.sigmoid(remaining, beta);

        state
            .values
            .insert("remaining".to_string(), remaining.val.max(0.0));

        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["InputTrigger"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("Time", 1.0)]
    }
}

// ---------------------------------------------------------------------------
// 3f. FlipFlop (SR latch with state)
// ---------------------------------------------------------------------------

pub struct FlipFlopBlock;

impl Block for FlipFlopBlock {
    fn type_name(&self) -> &str {
        "FlipFlop"
    }

    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        state: &mut BlockState,
        _dt: f64,
        _beta: f64,
    ) -> ConnectorMap {
        let set = inputs["InputS"];
        let reset = inputs["InputR"];
        let prev_q = *state.values.get("Q").unwrap_or(&0.0);
        let prev_q_traced = tape.leaf(prev_q);

        // Q = S OR (prev_Q AND NOT R)
        // Smooth: q = s + prev*(1-r) - s*prev*(1-r)
        let one = tape.leaf(1.0);
        let not_r = tape.sub(one, reset);
        let hold = tape.mul(prev_q_traced, not_r);
        let sr = tape.mul(set, hold);
        let sum = tape.add(set, hold);
        let q = tape.sub(sum, sr); // OR formula

        state.values.insert("Q".to_string(), q.val);

        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["InputS", "InputR"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[]
    }
}

// ---------------------------------------------------------------------------
// 3g. PulseGen — oscillator
// ---------------------------------------------------------------------------

pub struct PulseGenBlock;

impl Block for PulseGenBlock {
    fn type_name(&self) -> &str {
        "PulseGen"
    }

    /// PulseGen: generates square wave with configurable TimeHigh/TimeLow.
    /// Differentiable model: use a smooth sawtooth → sigmoid.
    fn forward(
        &self,
        tape: &mut Tape,
        inputs: &ConnectorMap,
        state: &mut BlockState,
        dt: f64,
        beta: f64,
    ) -> ConnectorMap {
        let enable = inputs["InputEnable"];
        let time_high = inputs["TimeHigh"];
        let time_low = inputs["TimeLow"];

        let prev_phase = *state.values.get("phase").unwrap_or(&0.0);

        // Total period
        let period = tape.add(time_high, time_low);

        // Phase advances: phase += dt / period (mod 1.0)
        let dt_val = tape.leaf(dt);
        let phase_inc = tape.div(dt_val, period);
        let prev_phase_traced = tape.leaf(prev_phase);
        let new_phase_raw = tape.add(prev_phase_traced, phase_inc);

        // Soft modulo: we track raw phase and use fract() for the primal
        let phase_val = new_phase_raw.val % 1.0;

        // Duty cycle: fraction of period that is HIGH
        let duty = tape.div(time_high, period);

        // Output: sigmoid(β(duty - phase)) — high when phase < duty
        let phase_traced = tape.leaf(phase_val);
        let diff = tape.sub(duty, phase_traced);
        let raw_q = tape.sigmoid(diff, beta);

        // Gate by enable
        let q = tape.mul(raw_q, enable);

        state.values.insert("phase".to_string(), phase_val);

        HashMap::from([("Q".to_string(), q)])
    }

    fn input_names(&self) -> &[&str] {
        &["InputEnable", "InputInvert"]
    }
    fn output_names(&self) -> &[&str] {
        &["Q"]
    }
    fn param_defaults(&self) -> &[(&str, f64)] {
        &[("TimeHigh", 1.0), ("TimeLow", 1.0)]
    }
}

// ============================================================================
// 4. COMPUTATIONAL GRAPH & TOPOLOGICAL EXECUTION
// ============================================================================
//
// The Loxone config XML defines a directed graph of blocks connected by wires.
// We compile this into an execution order (topological sort) and run it.

/// A wire connecting one block's output to another's input.
#[derive(Clone, Debug)]
pub struct Wire {
    pub src_block: usize,      // index of source block
    pub src_connector: String,  // output connector name (e.g., "Q")
    pub dst_block: usize,      // index of destination block
    pub dst_connector: String,  // input connector name (e.g., "I1")
}

/// A node in the simulation graph.
pub struct GraphNode {
    pub block: Box<dyn Block>,
    pub uuid: String,
    pub title: String,
    pub state: BlockState,
    /// Default parameter values (from XML <Co K="..." Def="..."/> elements)
    pub param_defaults: HashMap<String, f64>,
}

/// The compiled simulation graph. Blocks are in topological order.
pub struct SimGraph {
    pub nodes: Vec<GraphNode>,
    pub wires: Vec<Wire>,
    pub execution_order: Vec<usize>, // topological sort of node indices
    pub input_nodes: Vec<usize>,     // external input nodes (sensors, etc.)
    pub output_nodes: Vec<usize>,    // nodes we want to observe
}

impl SimGraph {
    /// Compile a graph from Loxone XML config.
    /// This would parse the XML (using existing loxone_xml.rs infrastructure),
    /// resolve control types to Block implementations, extract wiring, and
    /// compute a topological execution order.
    pub fn from_xml(_xml_path: &str) -> Result<Self, String> {
        // In real implementation:
        // 1. Parse XML with xmltree (existing infrastructure)
        // 2. For each <C Type="..."> element, instantiate the right Block
        // 3. For each <Co> connector, record wires from InputRef/OutputRef
        // 4. Topological sort (Kahn's algorithm)
        // 5. Detect cycles (feedback loops need special handling — see §5)
        todo!("Parse XML config into SimGraph")
    }

    /// Execute one timestep of the entire graph.
    /// Returns all connector values (for inspection / loss computation).
    pub fn step(
        &mut self,
        tape: &mut Tape,
        external_inputs: &HashMap<String, f64>, // uuid → value
        dt: f64,
        beta: f64,
    ) -> HashMap<String, ConnectorMap> {
        // Connector values indexed by block uuid
        let mut all_values: HashMap<String, ConnectorMap> = HashMap::new();

        // Set external inputs
        for &node_idx in &self.input_nodes {
            let node = &self.nodes[node_idx];
            if let Some(&val) = external_inputs.get(&node.uuid) {
                let traced = tape.leaf(val);
                let mut outputs = ConnectorMap::new();
                outputs.insert("Q".to_string(), traced);
                all_values.insert(node.uuid.clone(), outputs);
            }
        }

        // Execute blocks in topological order
        for &node_idx in self.execution_order.clone().iter() {
            // Gather inputs for this block: wired values + parameter defaults
            let mut inputs = ConnectorMap::new();

            // Load parameter defaults first
            for (name, default) in &self.nodes[node_idx].param_defaults {
                inputs.insert(name.clone(), tape.leaf(*default));
            }

            // Override with wired connections
            for wire in &self.wires {
                if wire.dst_block == node_idx {
                    if let Some(src_outputs) = all_values.get(&self.nodes[wire.src_block].uuid) {
                        if let Some(&val) = src_outputs.get(&wire.src_connector) {
                            inputs.insert(wire.dst_connector.clone(), val);
                        }
                    }
                }
            }

            // Execute block
            let node = &mut self.nodes[node_idx];
            let outputs = node.block.forward(tape, &inputs, &mut node.state, dt, beta);
            all_values.insert(node.uuid.clone(), outputs);
        }

        all_values
    }
}

// ============================================================================
// 5. TIME-STEPPING & BACKPROP THROUGH TIME (BPTT)
// ============================================================================
//
// The simulation runs over a time horizon (e.g., 24 hours at 1-minute steps).
// This is analogous to an RNN: the block graph is the "cell", state variables
// are the "hidden state", and external inputs (temperature, occupancy) are
// the input sequence.
//
// For gradient computation, we use BPTT: run the full forward pass recording
// on the tape, then call backward() once. The tape captures all operations
// across all timesteps, so gradients flow back through time automatically.
//
// Memory optimization: for long simulations, use gradient checkpointing —
// only store state at every Nth timestep, recompute intermediates during
// backward pass.

/// Time-series input data (e.g., weather, occupancy schedule).
pub struct InputTimeSeries {
    /// Timestep duration in seconds
    pub dt: f64,
    /// Number of timesteps
    pub num_steps: usize,
    /// For each timestep, a map of (block_uuid → input_value)
    pub data: Vec<HashMap<String, f64>>,
}

/// Result of a full simulation run.
pub struct SimResult {
    /// All connector values at each timestep.
    /// Index: [timestep][block_uuid][connector_name]
    pub history: Vec<HashMap<String, ConnectorMap>>,
    /// The AD tape (for calling backward())
    pub tape: Tape,
    /// Indices of parameter leaves on the tape (for reading gradients)
    pub param_indices: HashMap<String, TapeIdx>,
}

/// Run a full simulation and return the tape for gradient computation.
pub fn simulate(
    graph: &mut SimGraph,
    inputs: &InputTimeSeries,
    params: &HashMap<String, f64>, // optimizable parameters (uuid.connector → value)
    beta: f64,
) -> SimResult {
    let mut tape = Tape::new();
    let mut history = Vec::with_capacity(inputs.num_steps);
    let mut param_indices = HashMap::new();

    // Register optimizable parameters on the tape
    for (name, &val) in params {
        let traced = tape.leaf(val);
        param_indices.insert(name.clone(), traced.idx);
    }

    // Override graph parameter defaults with optimizable params
    // (In a real implementation, this would set the specific <Co> values)

    // Run simulation
    for t in 0..inputs.num_steps {
        let step_inputs = &inputs.data[t];
        let values = graph.step(&mut tape, step_inputs, inputs.dt, beta);
        history.push(values);
    }

    SimResult {
        history,
        tape,
        param_indices,
    }
}

// ============================================================================
// 6. LOSS FUNCTIONS & OPTIMIZATION
// ============================================================================
//
// The loss function maps simulation outputs to a scalar that we minimize.
// Examples:
//   - Energy cost: Σ(power_output[t] * energy_price[t]) * dt
//   - Comfort violation: Σ max(0, |temp[t] - setpoint| - tolerance)²
//   - Blind movement count: Σ |blind_pos[t] - blind_pos[t-1]|
//   - Combined: w₁·energy + w₂·comfort + w₃·wear

/// A loss function that computes a scalar from simulation history.
pub trait LossFunction {
    fn compute(&self, tape: &mut Tape, history: &[HashMap<String, ConnectorMap>]) -> Traced;
}

/// Energy cost: sum of power output × price over time.
pub struct EnergyCostLoss {
    /// Energy price per timestep ($/kWh at each step)
    pub prices: Vec<f64>,
    /// UUID of the power output block
    pub power_block_uuid: String,
    /// Connector name for power output
    pub power_connector: String,
}

impl LossFunction for EnergyCostLoss {
    fn compute(&self, tape: &mut Tape, history: &[HashMap<String, ConnectorMap>]) -> Traced {
        let mut total = tape.leaf(0.0);
        for (t, step) in history.iter().enumerate() {
            if let Some(outputs) = step.get(&self.power_block_uuid) {
                if let Some(&power) = outputs.get(&self.power_connector) {
                    let price = tape.leaf(self.prices[t]);
                    let cost = tape.mul(power, price);
                    total = tape.add(total, cost);
                }
            }
        }
        total
    }
}

/// Comfort violation: penalize temperature deviations from setpoint.
pub struct ComfortLoss {
    pub temp_block_uuid: String,
    pub temp_connector: String,
    pub setpoint: f64,
    pub tolerance: f64, // dead band (no penalty within ±tolerance)
}

impl LossFunction for ComfortLoss {
    fn compute(&self, tape: &mut Tape, history: &[HashMap<String, ConnectorMap>]) -> Traced {
        let mut total = tape.leaf(0.0);
        for step in history {
            if let Some(outputs) = step.get(&self.temp_block_uuid) {
                if let Some(&temp) = outputs.get(&self.temp_connector) {
                    let setpoint = tape.leaf(self.setpoint);
                    let diff = tape.sub(temp, setpoint);
                    let abs_diff = tape.abs(diff);
                    let tol = tape.leaf(self.tolerance);
                    let violation = tape.sub(abs_diff, tol);
                    // Smooth ReLU: softplus(x) = ln(1 + e^x)
                    let penalty_val = (violation.val.exp() + 1.0).ln();
                    let penalty_grad = 1.0 / (1.0 + (-violation.val).exp()); // sigmoid
                    let penalty = tape.unary(violation, penalty_val, penalty_grad);
                    let sq = tape.mul(penalty, penalty); // squared penalty
                    total = tape.add(total, sq);
                }
            }
        }
        total
    }
}

/// Actuator wear: penalize changes in output (blind movements, valve cycles).
pub struct ActuatorWearLoss {
    pub block_uuid: String,
    pub connector: String,
    pub weight: f64,
}

impl LossFunction for ActuatorWearLoss {
    fn compute(&self, tape: &mut Tape, history: &[HashMap<String, ConnectorMap>]) -> Traced {
        let mut total = tape.leaf(0.0);
        for t in 1..history.len() {
            let prev = history[t - 1]
                .get(&self.block_uuid)
                .and_then(|m| m.get(&self.connector));
            let curr = history[t]
                .get(&self.block_uuid)
                .and_then(|m| m.get(&self.connector));
            if let (Some(&prev_val), Some(&curr_val)) = (prev, curr) {
                let diff = tape.sub(curr_val, prev_val);
                let abs_diff = tape.abs(diff);
                let w = tape.leaf(self.weight);
                let weighted = tape.mul(abs_diff, w);
                total = tape.add(total, weighted);
            }
        }
        total
    }
}

// ---------------------------------------------------------------------------
// 6a. Optimization loop
// ---------------------------------------------------------------------------

/// Adam optimizer state for one parameter.
#[derive(Clone, Debug)]
struct AdamState {
    m: f64, // first moment (mean of gradients)
    v: f64, // second moment (mean of squared gradients)
    t: u64, // timestep
}

/// Adam optimizer (the workhorse of deep learning, works great here too).
pub struct AdamOptimizer {
    pub lr: f64,      // learning rate
    pub beta1: f64,   // first moment decay (default: 0.9)
    pub beta2: f64,   // second moment decay (default: 0.999)
    pub epsilon: f64,  // numerical stability (default: 1e-8)
    states: HashMap<String, AdamState>,
}

impl AdamOptimizer {
    pub fn new(lr: f64) -> Self {
        AdamOptimizer {
            lr,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            states: HashMap::new(),
        }
    }

    /// Update parameters given gradients. Returns new parameter values.
    pub fn step(
        &mut self,
        params: &HashMap<String, f64>,
        grads: &HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        let mut new_params = HashMap::new();

        for (name, &val) in params {
            let grad = grads.get(name).copied().unwrap_or(0.0);

            let state = self.states.entry(name.clone()).or_insert(AdamState {
                m: 0.0,
                v: 0.0,
                t: 0,
            });

            state.t += 1;
            state.m = self.beta1 * state.m + (1.0 - self.beta1) * grad;
            state.v = self.beta2 * state.v + (1.0 - self.beta2) * grad * grad;

            // Bias correction
            let m_hat = state.m / (1.0 - self.beta1.powi(state.t as i32));
            let v_hat = state.v / (1.0 - self.beta2.powi(state.t as i32));

            let new_val = val - self.lr * m_hat / (v_hat.sqrt() + self.epsilon);
            new_params.insert(name.clone(), new_val);
        }

        new_params
    }
}

/// Full optimization loop with beta annealing.
pub fn optimize(
    graph: &mut SimGraph,
    inputs: &InputTimeSeries,
    initial_params: HashMap<String, f64>,
    param_bounds: &HashMap<String, (f64, f64)>, // min, max per param
    losses: &[Box<dyn LossFunction>],
    loss_weights: &[f64],
    num_iterations: usize,
) -> HashMap<String, f64> {
    let mut optimizer = AdamOptimizer::new(0.01);
    let mut params = initial_params;

    for iter in 0..num_iterations {
        // Beta annealing: start soft, harden over iterations.
        // Soft beta gives broad gradients early on,
        // hard beta gives precise behavior near convergence.
        let beta = 1.0 + (iter as f64 / num_iterations as f64) * 99.0; // 1 → 100

        // Forward pass: simulate with current parameters
        let result = simulate(graph, inputs, &params, beta);
        let mut tape = result.tape;

        // Compute weighted loss
        let mut total_loss = tape.leaf(0.0);
        for (loss_fn, &weight) in losses.iter().zip(loss_weights.iter()) {
            let l = loss_fn.compute(&mut tape, &result.history);
            let w = tape.leaf(weight);
            let weighted = tape.mul(l, w);
            total_loss = tape.add(total_loss, weighted);
        }

        // Backward pass: compute gradients
        let grads = tape.backward(total_loss.idx);

        // Extract parameter gradients
        let mut param_grads = HashMap::new();
        for (name, &tape_idx) in &result.param_indices {
            param_grads.insert(name.clone(), grads[tape_idx]);
        }

        // Adam update
        params = optimizer.step(&params, &param_grads);

        // Project onto bounds (box constraints)
        for (name, &(lo, hi)) in param_bounds {
            if let Some(val) = params.get_mut(name) {
                *val = val.clamp(lo, hi);
            }
        }

        if iter % 50 == 0 {
            eprintln!(
                "iter {}: loss = {:.4}, beta = {:.1}",
                iter, total_loss.val, beta
            );
        }
    }

    params
}

// ============================================================================
// 7. MULTI-OBJECTIVE OPTIMIZATION & PARETO FRONTS
// ============================================================================
//
// Home automation inherently involves trade-offs:
//   - Energy cost vs thermal comfort
//   - Privacy (blinds down) vs natural light
//   - Actuator lifetime vs responsiveness
//
// We find the Pareto front: the set of parameter configs where you can't
// improve one objective without worsening another.

/// A point on the Pareto front.
#[derive(Clone, Debug)]
pub struct ParetoPoint {
    pub params: HashMap<String, f64>,
    pub objectives: Vec<f64>, // one value per objective
}

/// Multi-objective optimization via weighted scalarization.
/// Samples weight vectors uniformly and finds optimal params for each.
/// Returns an approximation of the Pareto front.
pub fn pareto_front(
    graph: &mut SimGraph,
    inputs: &InputTimeSeries,
    initial_params: HashMap<String, f64>,
    param_bounds: &HashMap<String, (f64, f64)>,
    losses: &[Box<dyn LossFunction>],
    num_weight_samples: usize,
    iters_per_sample: usize,
) -> Vec<ParetoPoint> {
    let n_obj = losses.len();
    let mut front = Vec::new();

    for sample in 0..num_weight_samples {
        // Generate weight vector on simplex
        // For 2 objectives: w = [t, 1-t] for t in linspace(0, 1, N)
        let t = sample as f64 / (num_weight_samples - 1).max(1) as f64;
        let weights = if n_obj == 2 {
            vec![t, 1.0 - t]
        } else {
            vec![1.0 / n_obj as f64; n_obj]
        };

        let optimized_params = optimize(
            graph,
            inputs,
            initial_params.clone(),
            param_bounds,
            losses,
            &weights,
            iters_per_sample,
        );

        // Evaluate each objective at the optimized params
        let result = simulate(graph, inputs, &optimized_params, 100.0);
        let mut tape = result.tape;
        let objectives: Vec<f64> = losses
            .iter()
            .map(|loss| loss.compute(&mut tape, &result.history).val)
            .collect();

        front.push(ParetoPoint {
            params: optimized_params,
            objectives,
        });
    }

    // Filter dominated points
    pareto_filter(&mut front);
    front
}

/// Remove dominated points from candidate set.
fn pareto_filter(points: &mut Vec<ParetoPoint>) {
    let snapshot: Vec<Vec<f64>> = points.iter().map(|p| p.objectives.clone()).collect();
    points.retain(|p| {
        !snapshot.iter().any(|q_obj| {
            // q dominates p if q <= in all objectives and < in at least one
            let dominated = q_obj.iter().zip(&p.objectives).all(|(qi, pi)| qi <= pi)
                && q_obj.iter().zip(&p.objectives).any(|(qi, pi)| qi < pi);
            dominated
        })
    });
}

// ============================================================================
// 8. INTERVAL ARITHMETIC — FINDING ALL BREAKING POINTS
// ============================================================================
//
// Gradient-based optimization finds LOCAL optima. For safety analysis, we
// need to find ALL points where behavior changes (e.g., "at what temperature
// does the heating turn off?"). Interval arithmetic gives guaranteed bounds.
//
// An interval [lo, hi] represents all possible values. We propagate intervals
// through the block graph. Where an interval crosses a threshold, we've found
// a breaking point.

/// An interval [lo, hi] representing a range of possible values.
#[derive(Clone, Copy, Debug)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    pub fn new(lo: f64, hi: f64) -> Self {
        Interval { lo, hi }
    }
    pub fn point(v: f64) -> Self {
        Interval { lo: v, hi: v }
    }
    pub fn contains(&self, v: f64) -> bool {
        self.lo <= v && v <= self.hi
    }
    pub fn width(&self) -> f64 {
        self.hi - self.lo
    }
}

impl std::ops::Add for Interval {
    type Output = Interval;
    fn add(self, rhs: Interval) -> Interval {
        Interval {
            lo: self.lo + rhs.lo,
            hi: self.hi + rhs.hi,
        }
    }
}

impl std::ops::Sub for Interval {
    type Output = Interval;
    fn sub(self, rhs: Interval) -> Interval {
        Interval {
            lo: self.lo - rhs.hi,
            hi: self.hi - rhs.lo,
        }
    }
}

impl std::ops::Mul for Interval {
    type Output = Interval;
    fn mul(self, rhs: Interval) -> Interval {
        let products = [
            self.lo * rhs.lo,
            self.lo * rhs.hi,
            self.hi * rhs.lo,
            self.hi * rhs.hi,
        ];
        Interval {
            lo: products.iter().cloned().fold(f64::INFINITY, f64::min),
            hi: products.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }
}

/// Interval version of GreaterEqual: does the interval cross the threshold?
pub fn interval_greater_equal(input: Interval, threshold: Interval) -> IntervalBool {
    let diff = input - threshold;
    if diff.lo >= 0.0 {
        IntervalBool::AlwaysTrue
    } else if diff.hi < 0.0 {
        IntervalBool::AlwaysFalse
    } else {
        // The interval CROSSES the threshold — this is a breaking point!
        IntervalBool::Crosses {
            critical_value: threshold.lo, // approximate crossing point
        }
    }
}

/// Result of an interval boolean operation.
#[derive(Debug)]
pub enum IntervalBool {
    AlwaysTrue,
    AlwaysFalse,
    /// The interval crosses a discontinuity. The critical_value is where.
    Crosses { critical_value: f64 },
}

/// Find all breaking points by bisection on input intervals.
/// Returns a list of parameter values where behavior changes discontinuously.
pub fn find_breaking_points(
    graph: &mut SimGraph,
    param_name: &str,
    param_range: Interval,
    inputs: &InputTimeSeries,
    max_depth: u32,
    tolerance: f64,
) -> Vec<f64> {
    let mut breaking_points = Vec::new();
    find_bp_recursive(
        graph,
        param_name,
        param_range,
        inputs,
        max_depth,
        tolerance,
        &mut breaking_points,
    );
    breaking_points.sort_by(|a, b| a.partial_cmp(b).unwrap());
    breaking_points.dedup_by(|a, b| (*a - *b).abs() < tolerance);
    breaking_points
}

fn find_bp_recursive(
    graph: &mut SimGraph,
    param_name: &str,
    range: Interval,
    inputs: &InputTimeSeries,
    depth: u32,
    tolerance: f64,
    results: &mut Vec<f64>,
) {
    if range.width() < tolerance || depth == 0 {
        results.push((range.lo + range.hi) / 2.0);
        return;
    }

    // Evaluate at endpoints
    let lo_behavior = evaluate_at(graph, param_name, range.lo, inputs);
    let hi_behavior = evaluate_at(graph, param_name, range.hi, inputs);

    // If behavior differs, there's a breaking point in this interval
    if behavior_differs(&lo_behavior, &hi_behavior) {
        let mid = (range.lo + range.hi) / 2.0;
        find_bp_recursive(
            graph,
            param_name,
            Interval::new(range.lo, mid),
            inputs,
            depth - 1,
            tolerance,
            results,
        );
        find_bp_recursive(
            graph,
            param_name,
            Interval::new(mid, range.hi),
            inputs,
            depth - 1,
            tolerance,
            results,
        );
    }
}

/// Behavioral fingerprint: vector of output values at final timestep.
type Fingerprint = Vec<(String, f64)>;

fn evaluate_at(
    graph: &mut SimGraph,
    param_name: &str,
    param_value: f64,
    inputs: &InputTimeSeries,
) -> Fingerprint {
    let mut params = HashMap::new();
    params.insert(param_name.to_string(), param_value);
    let result = simulate(graph, inputs, &params, 1000.0); // sharp beta for exact eval
    // Extract output values at last timestep
    if let Some(last_step) = result.history.last() {
        let mut fp = Vec::new();
        for (uuid, connectors) in last_step {
            for (name, traced) in connectors {
                // Discretize to detect behavioral changes (not just numeric drift)
                let discrete = (traced.val * 100.0).round();
                fp.push((format!("{}.{}", uuid, name), discrete));
            }
        }
        fp.sort_by(|a, b| a.0.cmp(&b.0));
        fp
    } else {
        vec![]
    }
}

fn behavior_differs(a: &Fingerprint, b: &Fingerprint) -> bool {
    if a.len() != b.len() {
        return true;
    }
    a.iter().zip(b.iter()).any(|((_, av), (_, bv))| (av - bv).abs() > f64::EPSILON)
}

// ============================================================================
// 9. SENSITIVITY ANALYSIS — FORWARD-MODE SHORTCUTS
// ============================================================================
//
// For answering "how sensitive is output X to parameter Y?", forward-mode
// AD with dual numbers is more efficient than reverse mode (one forward
// pass per parameter, vs. building the entire tape).

/// Compute sensitivity of all outputs to a single parameter.
/// Uses forward-mode AD (dual numbers).
pub fn sensitivity_analysis(
    // In a real implementation, this would use a Dual-number version
    // of the block graph. The Block trait would be generic:
    //   trait Block<V: Value> { fn forward(..., inputs: &Map<V>) -> Map<V>; }
    // and we'd instantiate it with V=Dual.
    _graph: &SimGraph,
    _param_name: &str,
    _param_value: f64,
    _inputs: &InputTimeSeries,
) -> HashMap<String, f64> {
    // For each output connector, returns ∂output/∂param
    todo!("Forward-mode sensitivity analysis")
}

// ============================================================================
// 10. PUTTING IT ALL TOGETHER — EXAMPLE USAGE
// ============================================================================

/// Example: optimize a heating system with AnalogThresholdTrigger controlling
/// a heater based on temperature, with comfort and energy objectives.
pub fn example_heating_optimization() {
    // The setup (in real code, parsed from XML):
    //
    //   [TempSensor] --Input--> [AnalogThresholdTrigger] --Q--> [HeaterRelay]
    //                            On=21.0  Off=19.5
    //
    // External inputs: outdoor temperature over 24h (hourly)
    // Optimizable params: On threshold, Off threshold
    // Objectives: minimize energy, maximize comfort (temp near 21°C)

    let outdoor_temps: Vec<f64> = (0..24)
        .map(|h| {
            // Sinusoidal outdoor temp: min 2°C at 4am, max 12°C at 2pm
            7.0 + 5.0 * (2.0 * PI * (h as f64 - 4.0) / 24.0).sin()
        })
        .collect();

    // In a real scenario: parse XML, build graph, set up losses, run optimize()
    eprintln!("Outdoor temperatures: {:?}", outdoor_temps);
    eprintln!("Would optimize On/Off thresholds to minimize energy + comfort loss");
    eprintln!("Gradient ∂energy/∂On tells us: raising On threshold by 1°C saves X kWh");
    eprintln!("Gradient ∂comfort/∂Off tells us: lowering Off threshold causes Y°C·h discomfort");
}

// ============================================================================
// 11. GRADIENT CHECKPOINTING FOR LONG SIMULATIONS
// ============================================================================
//
// A 24h simulation at 1s resolution = 86,400 timesteps. Each timestep adds
// ~100 entries to the tape (for a medium graph). That's 8.6M tape entries,
// each ~40 bytes = 344MB just for the tape.
//
// Gradient checkpointing trades compute for memory: store only every Nth
// state, recompute intermediate states during backward pass.
//
// With √T checkpoints, memory = O(√T) instead of O(T), at 2× compute cost.

pub struct CheckpointConfig {
    /// Store a checkpoint every N timesteps
    pub interval: usize,
    /// Maximum memory for tape entries (soft limit)
    pub max_tape_entries: usize,
}

impl CheckpointConfig {
    /// Auto-configure based on simulation length and available memory.
    pub fn auto(num_timesteps: usize, memory_budget_mb: usize) -> Self {
        let entries_per_step = 200; // estimate
        let bytes_per_entry = 48;
        let max_entries = memory_budget_mb * 1_000_000 / bytes_per_entry;
        let interval = (num_timesteps * entries_per_step / max_entries).max(1);
        CheckpointConfig {
            interval,
            max_tape_entries: max_entries,
        }
    }
}

// ============================================================================
// 12. BLOCK REGISTRY — MAPPING LOXONE TYPES TO DIFFERENTIABLE IMPLEMENTATIONS
// ============================================================================

/// Registry mapping Loxone block type names to differentiable implementations.
pub struct BlockRegistry {
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn Block>>>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut reg = BlockRegistry {
            factories: HashMap::new(),
        };

        // Logic gates
        reg.register("And", || Box::new(AndBlock));
        reg.register("Or", || Box::new(OrBlock));
        reg.register("Not", || Box::new(NotBlock));

        // Comparators
        reg.register("GreaterEqual", || Box::new(GreaterEqualBlock));

        // Math
        reg.register("Add", || Box::new(AddBlock));
        reg.register("Mult", || Box::new(MultBlock));

        // Threshold
        reg.register("AnalogThresholdTrigger", || {
            Box::new(AnalogThresholdTriggerBlock)
        });

        // Timers
        reg.register("OnPulseDelay", || Box::new(OnPulseDelayBlock));
        reg.register("Monoflop", || Box::new(MonoflopBlock));

        // State
        reg.register("FlipFlop", || Box::new(FlipFlopBlock));

        // Oscillators
        reg.register("PulseGen", || Box::new(PulseGenBlock));

        // ... 180+ more block types would be registered here
        // Priority order for implementation:
        //   1. Logic (And, Or, Not, Xor) — foundation
        //   2. Math (Add, Sub, Mult, Div, Scaler) — always differentiable
        //   3. Comparators (GreaterEqual, Less) — need relaxation
        //   4. Thresholds (AnalogThresholdTrigger) — hysteresis
        //   5. Timers (OnPulseDelay, OffDelay, Monoflop) — temporal
        //   6. State (FlipFlop, AMemory, Counter, State) — memory
        //   7. Complex (LightController2, Thermostat, AutoJalousie) — compound

        reg
    }

    fn register<F: Fn() -> Box<dyn Block> + 'static>(&mut self, name: &str, factory: F) {
        self.factories
            .insert(name.to_string(), Box::new(factory));
    }

    pub fn create(&self, type_name: &str) -> Option<Box<dyn Block>> {
        self.factories.get(type_name).map(|f| f())
    }
}

// ============================================================================
// 13. DESIGN NOTES & FUTURE DIRECTIONS
// ============================================================================
//
// FEEDBACK LOOPS (cycles in the graph):
//   Loxone configs can have feedback loops (e.g., thermostat output → heater →
//   temperature → thermostat input). These create cycles in the DAG.
//   Solution: break cycles at one edge, use the PREVIOUS timestep's value.
//   This is exactly how RNNs handle recurrence — the feedback becomes a
//   temporal dependency, and BPTT handles the gradients.
//
// MIXED CONTINUOUS/DISCRETE OPTIMIZATION:
//   Some parameters are inherently discrete (scene number, counter limit).
//   Approach: relax to continuous during optimization, then round to nearest
//   valid discrete value. For critical discrete choices, use Gumbel-Softmax
//   or straight-through estimator.
//
// STRAIGHT-THROUGH ESTIMATOR (STE):
//   For blocks where smooth relaxation isn't accurate enough:
//   Forward pass: use the exact discrete operation (step function)
//   Backward pass: pretend it was a smooth operation (pass gradient through)
//   This is widely used in quantized neural networks and works surprisingly well.
//
//   fn ste_step(x: f64, threshold: f64) -> (f64, f64) {
//       let forward = if x >= threshold { 1.0 } else { 0.0 };
//       let backward_grad = 1.0; // pretend identity in backward pass
//       (forward, backward_grad)
//   }
//
// BAYESIAN OPTIMIZATION FALLBACK:
//   When the landscape is too rugged for gradients (many discrete switches),
//   use Bayesian optimization as a fallback: model the loss surface with a
//   Gaussian process, use expected improvement to select next evaluation point.
//   Gradients from AD inform the GP kernel lengthscales.
//
// SYMBOLIC EXECUTION:
//   For proving safety properties ("the heater NEVER runs when window is open"),
//   use symbolic execution: replace f64 with symbolic expressions, propagate
//   through blocks, check satisfiability with an SMT solver (e.g., Z3).
//   The block graph is small enough (~100-3000 blocks) that this is feasible.
//
// ABSTRACT INTERPRETATION:
//   For range analysis ("what's the maximum possible heater runtime?"),
//   use abstract interpretation with interval domains. Propagate intervals
//   through the graph to get guaranteed bounds on all outputs.
//   This is implemented in §8 above.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_arithmetic() {
        // f(x) = x² + 2x + 1 at x=3
        // f'(x) = 2x + 2 = 8
        let x = Dual::variable(3.0);
        let two = Dual::constant(2.0);
        let one = Dual::constant(1.0);
        let x_sq = x * x;
        let two_x = two * x;
        let result = x_sq + two_x + one;
        assert!((result.val - 16.0).abs() < 1e-10); // 9 + 6 + 1
        assert!((result.dot - 8.0).abs() < 1e-10); // 2*3 + 2
    }

    #[test]
    fn test_reverse_mode_add_mul() {
        // f(a, b) = a * b + a at a=2, b=3
        // ∂f/∂a = b + 1 = 4, ∂f/∂b = a = 2
        let mut tape = Tape::new();
        let a = tape.leaf(2.0);
        let b = tape.leaf(3.0);
        let ab = tape.mul(a, b);
        let result = tape.add(ab, a);

        assert!((result.val - 8.0).abs() < 1e-10); // 2*3 + 2
        let grads = tape.backward(result.idx);
        assert!((grads[a.idx] - 4.0).abs() < 1e-10); // ∂f/∂a = b + 1
        assert!((grads[b.idx] - 2.0).abs() < 1e-10); // ∂f/∂b = a
    }

    #[test]
    fn test_sigmoid_relaxation() {
        // With high beta, sigmoid should approximate step function
        let step_at_0 = SmoothOps::greater_equal(0.0, 0.0, 100.0);
        assert!((step_at_0 - 0.5).abs() < 0.01); // exactly at threshold

        let step_above = SmoothOps::greater_equal(1.0, 0.0, 100.0);
        assert!(step_above > 0.999); // well above threshold

        let step_below = SmoothOps::greater_equal(-1.0, 0.0, 100.0);
        assert!(step_below < 0.001); // well below threshold
    }

    #[test]
    fn test_smooth_logic_gates() {
        // With binary inputs, smooth gates should be exact
        assert!((SmoothOps::and(1.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((SmoothOps::and(1.0, 0.0) - 0.0).abs() < 1e-10);
        assert!((SmoothOps::and(0.0, 1.0) - 0.0).abs() < 1e-10);
        assert!((SmoothOps::and(0.0, 0.0) - 0.0).abs() < 1e-10);

        assert!((SmoothOps::or(0.0, 0.0) - 0.0).abs() < 1e-10);
        assert!((SmoothOps::or(1.0, 0.0) - 1.0).abs() < 1e-10);
        assert!((SmoothOps::or(0.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((SmoothOps::or(1.0, 1.0) - 1.0).abs() < 1e-10);

        assert!((SmoothOps::not(0.0) - 1.0).abs() < 1e-10);
        assert!((SmoothOps::not(1.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_smooth_hysteresis() {
        let beta = 50.0;
        // Starting OFF, input above ON threshold → should turn ON
        let q = SmoothOps::hysteresis(22.0, 21.0, 19.0, 0.0, beta);
        assert!(q > 0.99, "Should be ON: {}", q);

        // Staying ON, input between ON and OFF → should stay ON
        let q = SmoothOps::hysteresis(20.0, 21.0, 19.0, 1.0, beta);
        assert!(q > 0.99, "Should stay ON: {}", q);

        // ON, input below OFF threshold → should turn OFF
        let q = SmoothOps::hysteresis(18.0, 21.0, 19.0, 1.0, beta);
        assert!(q < 0.01, "Should be OFF: {}", q);
    }

    #[test]
    fn test_smooth_flipflop() {
        // Set=1, Reset=0, prev=0 → should SET to 1
        assert!(SmoothOps::flip_flop(1.0, 0.0, 0.0) > 0.99);
        // Set=0, Reset=0, prev=1 → should HOLD at 1
        assert!(SmoothOps::flip_flop(0.0, 0.0, 1.0) > 0.99);
        // Set=0, Reset=1, prev=1 → should RESET to 0
        assert!(SmoothOps::flip_flop(0.0, 1.0, 1.0) < 0.01);
    }

    #[test]
    fn test_and_block_gradient() {
        // AND gate with tape: verify gradient flows
        let mut tape = Tape::new();
        let i1 = tape.leaf(0.8);
        let i2 = tape.leaf(0.6);
        let q = tape.mul(i1, i2); // AND = multiply

        // ∂(AND)/∂i1 = i2 = 0.6
        // ∂(AND)/∂i2 = i1 = 0.8
        let grads = tape.backward(q.idx);
        assert!((grads[i1.idx] - 0.6).abs() < 1e-10);
        assert!((grads[i2.idx] - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_greater_equal_gradient() {
        // Gradient of smooth GreaterEqual w.r.t. threshold
        let mut tape = Tape::new();
        let input = tape.leaf(20.5); // temperature
        let threshold = tape.leaf(21.0); // setpoint
        let diff = tape.sub(input, threshold);
        let q = tape.sigmoid(diff, 10.0); // beta=10

        let grads = tape.backward(q.idx);
        // ∂Q/∂input should be positive (higher temp → more likely >= threshold)
        assert!(grads[input.idx] > 0.0);
        // ∂Q/∂threshold should be negative (higher threshold → less likely >=)
        assert!(grads[threshold.idx] < 0.0);
        // They should be equal magnitude, opposite sign
        assert!((grads[input.idx] + grads[threshold.idx]).abs() < 1e-10);
    }

    #[test]
    fn test_interval_arithmetic() {
        // Temperature range [18, 24] vs threshold [20, 22]
        let temp = Interval::new(18.0, 24.0);
        let threshold = Interval::new(21.0, 21.0); // fixed threshold

        match interval_greater_equal(temp, threshold) {
            IntervalBool::Crosses { .. } => {} // expected: interval spans threshold
            other => panic!("Expected Crosses, got {:?}", other),
        }

        // Temperature clearly above
        let hot = Interval::new(25.0, 30.0);
        match interval_greater_equal(hot, threshold) {
            IntervalBool::AlwaysTrue => {}
            other => panic!("Expected AlwaysTrue, got {:?}", other),
        }
    }

    #[test]
    fn test_adam_optimizer() {
        // Minimize f(x) = (x - 3)²
        let mut opt = AdamOptimizer::new(0.1);
        let mut params = HashMap::from([("x".to_string(), 0.0)]);

        for _ in 0..200 {
            let x = params["x"];
            let grad = 2.0 * (x - 3.0); // ∂f/∂x = 2(x-3)
            let grads = HashMap::from([("x".to_string(), grad)]);
            params = opt.step(&params, &grads);
        }

        assert!(
            (params["x"] - 3.0).abs() < 0.1,
            "Should converge near 3.0, got {}",
            params["x"]
        );
    }
}
