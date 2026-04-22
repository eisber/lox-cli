//! Group D — Miscellaneous blocks (media, central, code, buttons, encoders, etc.).
//!
//! This module covers the remaining Loxone block types that don't fit neatly into
//! logic, math, timing, controller, energy, security, or I/O categories.
//! Many are pass-through stubs pending real Miniserver validation.

use crate::blocks::{bool_signal, deserialize_f64s, is_high, serialize_f64s, Block};
use crate::types::Signal;

// ===========================================================================
// Macro for pass-through stubs (unknown behavior)
// ===========================================================================

/// Creates a pass-through stub block with a WARNING comment.
macro_rules! stub_block {
    ($(#[$meta:meta])* $name:ident, $type_str:expr) => {
        // WARNING: Block type $type_str behavior unknown — using pass-through stub.
        // Actual Loxone behavior not documented. Will be validated later.
        $(#[$meta])*
        #[derive(Clone, Copy)]
        pub struct $name;

        impl Block for $name {
            fn eval(
                &mut self,
                inputs: &[Signal],
                _params: &[Signal],
                _dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                vec![inputs.first().copied().unwrap_or(0.0)]
            }

            fn state(&self) -> Option<Vec<u8>> {
                None
            }

            fn restore(&mut self, _state: &[u8]) {}

            fn block_type(&self) -> &str {
                $type_str
            }
        }
    };
}

/// Creates a pass-through stub that logs a warning (PicoC code blocks).
macro_rules! code_block {
    ($(#[$meta:meta])* $name:ident, $type_str:expr) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        pub struct $name;

        impl Block for $name {
            fn eval(
                &mut self,
                inputs: &[Signal],
                _params: &[Signal],
                _dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                // WARNING: PicoC code block — user scripts not simulated.
                // Passes first input to output as a no-op placeholder.
                vec![inputs.first().copied().unwrap_or(0.0)]
            }

            fn state(&self) -> Option<Vec<u8>> {
                None
            }

            fn restore(&mut self, _state: &[u8]) {}

            fn block_type(&self) -> &str {
                $type_str
            }
        }
    };
}

/// Creates an aggregator block that ORs all inputs (central controller pattern).
macro_rules! central_block {
    ($(#[$meta:meta])* $name:ident, $type_str:expr) => {
        // WARNING: Simplified model — real Loxone central block behavior may differ.
        $(#[$meta])*
        #[derive(Clone, Copy)]
        pub struct $name;

        impl Block for $name {
            /// Inputs: [sub1, sub2, ...]
            /// Outputs: [any_active, active_count]
            fn eval(
                &mut self,
                inputs: &[Signal],
                _params: &[Signal],
                _dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                let count = inputs.iter().filter(|&&v| is_high(v)).count();
                vec![bool_signal(count > 0), count as f64]
            }

            fn state(&self) -> Option<Vec<u8>> {
                None
            }

            fn restore(&mut self, _state: &[u8]) {}

            fn block_type(&self) -> &str {
                $type_str
            }
        }
    };
}

// ===========================================================================
// Math / Encoding — implemented blocks
// ===========================================================================

// ---------------------------------------------------------------------------
// Average / Avg
// ---------------------------------------------------------------------------

/// Computes the arithmetic mean of all inputs.
#[derive(Clone, Copy)]
pub struct Average;

impl Block for Average {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        if inputs.is_empty() {
            return vec![0.0];
        }
        let sum: f64 = inputs.iter().sum();
        vec![sum / inputs.len() as f64]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Average"
    }
}

/// Alias for Average — computes the arithmetic mean of all inputs.
#[derive(Clone, Copy)]
pub struct Avg;

impl Block for Avg {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        if inputs.is_empty() {
            return vec![0.0];
        }
        let sum: f64 = inputs.iter().sum();
        vec![sum / inputs.len() as f64]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Avg"
    }
}

// ---------------------------------------------------------------------------
// BinEncoder / BinDecoder
// ---------------------------------------------------------------------------

/// Binary encoder: combines digital inputs into an analog value.
/// I[0] = bit0, I[1] = bit1, ... → output = sum of 2^i for each high input.
#[derive(Clone, Copy)]
pub struct BinEncoder;

impl Block for BinEncoder {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let mut value: u64 = 0;
        for (i, &inp) in inputs.iter().enumerate() {
            if is_high(inp) && i < 64 {
                value |= 1 << i;
            }
        }
        vec![value as f64]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "BinEncoder"
    }
}

/// Binary decoder: decomposes an analog value into digital outputs.
/// Input = integer value → outputs bit0, bit1, ... (up to 32 bits).
#[derive(Clone, Copy)]
pub struct BinDecoder;

impl Block for BinDecoder {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0) as u32;
        let bits = params.first().copied().unwrap_or(8.0).clamp(1.0, 32.0) as usize;
        (0..bits)
            .map(|i| bool_signal(value & (1 << i) != 0))
            .collect()
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "BinDecoder"
    }
}

// ---------------------------------------------------------------------------
// DewPoint
// ---------------------------------------------------------------------------

// WARNING: Uses the Magnus-Tetens formula (valid for −45°C to +60°C, RH 1–100%).
// Td ≈ (237.3 × α) / (17.27 − α) where α = (17.27×T)/(237.3+T) + ln(RH/100).
// This is more accurate than the simplified Td ≈ T − (100−RH)/5 approximation.
// Outside the valid temperature range, results are approximate.
// At extreme conditions (T near −237.3°C or RH near 0%), the formula has
// singularities — we guard against division by zero.

/// Dew point calculator from temperature and relative humidity.
#[derive(Clone, Copy)]
pub struct DewPoint;

impl Block for DewPoint {
    /// Inputs:  `[temperature_celsius, relative_humidity_percent]`
    /// Outputs: `[dew_point_celsius]`
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let temp = inputs.first().copied().unwrap_or(20.0);
        let rh = inputs.get(1).copied().unwrap_or(50.0).clamp(0.01, 100.0);

        // Guard against singularity when temp approaches -237.3°C
        let denom_temp = 237.3 + temp;
        if denom_temp.abs() < 0.1 {
            return vec![temp];
        }

        let alpha = (17.27 * temp) / denom_temp + (rh / 100.0).ln();
        let denom_alpha = 17.27 - alpha;
        if denom_alpha.abs() < 1e-10 {
            return vec![temp]; // Near saturation limit
        }

        let dew = (237.3 * alpha) / denom_alpha;
        vec![dew]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "DewPoint"
    }
}

// ---------------------------------------------------------------------------
// Power
// ---------------------------------------------------------------------------

/// Power block: output = base ^ exponent. I[0] = base, I[1] or params[0] = exponent.
#[derive(Clone, Copy)]
pub struct Power;

impl Block for Power {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let base = inputs.first().copied().unwrap_or(0.0);
        let exp = inputs
            .get(1)
            .copied()
            .or_else(|| params.first().copied())
            .unwrap_or(1.0);
        vec![base.powf(exp)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Power"
    }
}

// ---------------------------------------------------------------------------
// Validator
// ---------------------------------------------------------------------------

/// Range validator: output 1.0 if input is within [min, max].
/// Params: [min (default 0), max (default 100)].
#[derive(Clone, Copy)]
pub struct Validator;

impl Block for Validator {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let min = params.first().copied().unwrap_or(0.0);
        let max = params.get(1).copied().unwrap_or(100.0);
        vec![bool_signal(value >= min && value <= max)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Validator"
    }
}

// ---------------------------------------------------------------------------
// TimeMinmax
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Tracks running min/max of input. Reset input clears tracked values.

/// Tracks minimum and maximum of input signal over time.
#[derive(Clone)]
pub struct TimeMinmax {
    min_val: f64,
    max_val: f64,
    initialized: bool,
}

impl TimeMinmax {
    pub fn new() -> Self {
        Self {
            min_val: 0.0,
            max_val: 0.0,
            initialized: false,
        }
    }
}

impl Default for TimeMinmax {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for TimeMinmax {
    /// Inputs: [value, reset]
    /// Outputs: [current, min, max]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);

        if is_high(reset) || !self.initialized {
            self.min_val = value;
            self.max_val = value;
            self.initialized = true;
        } else {
            if value < self.min_val {
                self.min_val = value;
            }
            if value > self.max_val {
                self.max_val = value;
            }
        }

        vec![value, self.min_val, self.max_val]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[
            self.min_val,
            self.max_val,
            if self.initialized { 1.0 } else { 0.0 },
        ]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 3) {
            self.min_val = v[0];
            self.max_val = v[1];
            self.initialized = v[2] > 0.5;
        }
    }

    fn block_type(&self) -> &str {
        "TimeMinmax"
    }
}

// ===========================================================================
// Timing / Pulse — implemented blocks
// ===========================================================================

// ---------------------------------------------------------------------------
// Ramp
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Ramps output from current value toward target at a fixed rate per second.

/// Value ramp: smoothly transitions output toward input at a configurable rate.
#[derive(Clone, Default)]
pub struct Ramp {
    output: f64,
}

impl Ramp {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Ramp {
    /// Inputs: [target_value]
    /// Params: [rate_per_second (default 1.0)]
    /// Outputs: [ramped_value]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let target = inputs.first().copied().unwrap_or(0.0);
        let rate = params
            .first()
            .copied()
            .unwrap_or(1.0)
            .abs()
            .max(f64::EPSILON);

        if dt > 0.0 {
            let diff = target - self.output;
            let max_step = rate * dt;
            if diff.abs() <= max_step {
                self.output = target;
            } else {
                self.output += max_step * diff.signum();
            }
        } else {
            self.output = target;
        }

        vec![self.output]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.output]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.output = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "Ramp"
    }
}

// ---------------------------------------------------------------------------
// Rand / RandomGen
// ---------------------------------------------------------------------------

// WARNING: Simplified model — uses a simple LCG PRNG for deterministic simulation.
// Real Loxone may use a different random source.

/// Random value generator — outputs a pseudo-random value in [min, max].
#[derive(Clone)]
pub struct Rand {
    seed: u64,
}

impl Rand {
    pub fn new() -> Self {
        Self { seed: 12345 }
    }

    fn next_f64(&mut self) -> f64 {
        // Simple LCG for reproducibility
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.seed >> 33) as f64 / (1u64 << 31) as f64
    }
}

impl Default for Rand {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for Rand {
    /// Inputs: [trigger]
    /// Params: [min (default 0), max (default 1)]
    /// Outputs: [random_value]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev = prev_inputs.first().copied().unwrap_or(0.0);
        let min = params.first().copied().unwrap_or(0.0);
        let max = params.get(1).copied().unwrap_or(1.0);

        if !is_high(prev) && is_high(trigger) {
            let r = self.next_f64();
            vec![min + r * (max - min)]
        } else {
            vec![min]
        }
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(self.seed.to_le_bytes().to_vec())
    }

    fn restore(&mut self, state: &[u8]) {
        if state.len() >= 8 {
            self.seed = u64::from_le_bytes(state[..8].try_into().unwrap_or_default());
        }
    }

    fn block_type(&self) -> &str {
        "Rand"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Alias for Rand — pseudo-random value generator.
#[derive(Clone)]
pub struct RandomGen {
    inner: Rand,
}

impl RandomGen {
    pub fn new() -> Self {
        Self { inner: Rand::new() }
    }
}

impl Default for RandomGen {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for RandomGen {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev: &[Signal],
    ) -> Vec<Signal> {
        self.inner.eval(inputs, params, dt, prev)
    }

    fn state(&self) -> Option<Vec<u8>> {
        self.inner.state()
    }

    fn restore(&mut self, state: &[u8]) {
        self.inner.restore(state);
    }

    fn block_type(&self) -> &str {
        "RandomGen"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// PulseBy
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Outputs a pulse for a duration specified by params[0].
// Rising edge on input starts the pulse.

/// Pulse for a specified duration on trigger (like Monoflop).
#[derive(Clone, Default)]
pub struct PulseBy {
    timer: f64,
}

impl PulseBy {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for PulseBy {
    /// Inputs: [trigger]
    /// Params: [duration_seconds (default 1.0)]
    /// Outputs: [pulse_active]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev = prev_inputs.first().copied().unwrap_or(0.0);
        let duration = params.first().copied().unwrap_or(1.0).max(0.0);

        if !is_high(prev) && is_high(trigger) {
            self.timer = duration;
        }

        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }

        vec![bool_signal(self.timer > 0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.timer]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.timer = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "PulseBy"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ===========================================================================
// Button variants — implemented blocks
// ===========================================================================

// ---------------------------------------------------------------------------
// LongClick
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Outputs 1.0 after input has been held high for threshold duration.

/// Long-click detector: outputs high after input held for threshold.
#[derive(Clone, Default)]
pub struct LongClick {
    hold_time: f64,
    fired: bool,
}

impl LongClick {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for LongClick {
    /// Inputs: [button]
    /// Params: [threshold_seconds (default 1.0)]
    /// Outputs: [long_click_detected]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let pressed = is_high(inputs.first().copied().unwrap_or(0.0));
        let threshold = params.first().copied().unwrap_or(1.0).max(0.0);

        if pressed {
            self.hold_time += dt;
            if self.hold_time >= threshold && !self.fired {
                self.fired = true;
                return vec![1.0];
            }
        } else {
            self.hold_time = 0.0;
            self.fired = false;
        }

        vec![0.0]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.hold_time]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.hold_time = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "LongClick"
    }
}

// ---------------------------------------------------------------------------
// MultiClick
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Counts rising edges within a timeout window. After timeout, outputs click count.

/// Multi-click detector: counts clicks within a timeout window.
#[derive(Clone, Default)]
pub struct MultiClick {
    click_count: f64,
    timer: f64,
    output: f64,
}

impl MultiClick {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for MultiClick {
    /// Inputs: [button]
    /// Params: [timeout_seconds (default 0.5)]
    /// Outputs: [click_count (0 while counting, N after timeout)]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let cur = inputs.first().copied().unwrap_or(0.0);
        let prev = prev_inputs.first().copied().unwrap_or(0.0);
        let timeout = params.first().copied().unwrap_or(0.5).max(0.01);

        // Rising edge → increment count, reset timer
        if !is_high(prev) && is_high(cur) {
            self.click_count += 1.0;
            self.timer = timeout;
        }

        self.output = 0.0;
        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
            if self.timer <= 0.0 && self.click_count > 0.0 {
                self.output = self.click_count;
                self.click_count = 0.0;
            }
        }

        vec![self.output]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.click_count, self.timer]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 2) {
            self.click_count = v[0];
            self.timer = v[1];
        }
    }

    fn block_type(&self) -> &str {
        "MultiClick"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// PushDimmer
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Holding button ramps brightness up/down, toggling direction on release.

/// Push-to-dim: holding button ramps value, direction toggles on release.
#[derive(Clone)]
pub struct PushDimmer {
    value: f64,
    direction: f64, // 1.0 = up, -1.0 = down
}

impl PushDimmer {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            direction: 1.0,
        }
    }
}

impl Default for PushDimmer {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for PushDimmer {
    /// Inputs: [button]
    /// Params: [rate_per_second (default 0.5)]
    /// Outputs: [dim_level (0–1)]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let pressed = is_high(inputs.first().copied().unwrap_or(0.0));
        let was_pressed = is_high(prev_inputs.first().copied().unwrap_or(0.0));
        let rate = params
            .first()
            .copied()
            .unwrap_or(0.5)
            .abs()
            .max(f64::EPSILON);

        if pressed && dt > 0.0 {
            self.value = (self.value + self.direction * rate * dt).clamp(0.0, 1.0);
            // Reverse at limits
            if self.value >= 1.0 || self.value <= 0.0 {
                self.direction = -self.direction;
            }
        }

        // Toggle direction on release
        if was_pressed && !pressed {
            self.direction = -self.direction;
        }

        vec![self.value]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.value, self.direction]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 2) {
            self.value = v[0];
            self.direction = v[1];
        }
    }

    fn block_type(&self) -> &str {
        "PushDimmer"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// StepSel
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Cycles through discrete values on each rising edge.

/// Step selector: cycles through values 0..N on each trigger.
#[derive(Clone)]
pub struct StepSel {
    current: f64,
}

impl StepSel {
    pub fn new() -> Self {
        Self { current: 0.0 }
    }
}

impl Default for StepSel {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for StepSel {
    /// Inputs: [trigger, reset]
    /// Params: [num_steps (default 4)]
    /// Outputs: [current_step]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);
        let steps = params.first().copied().unwrap_or(4.0).max(1.0) as u32;

        if is_high(reset) {
            self.current = 0.0;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            self.current = ((self.current as u32 + 1) % steps) as f64;
        }

        vec![self.current]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.current]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.current = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "StepSel"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ===========================================================================
// Sequencer blocks
// ===========================================================================

// ---------------------------------------------------------------------------
// Sequencer
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Steps through stored values on each trigger. Values set via params.

/// Sequencer: outputs stored values in sequence on each trigger.
#[derive(Clone)]
pub struct Sequencer {
    step: usize,
}

impl Sequencer {
    pub fn new() -> Self {
        Self { step: 0 }
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for Sequencer {
    /// Inputs: [trigger, reset]
    /// Params: [val0, val1, val2, ...] — sequence values
    /// Outputs: [current_value, step_index]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);

        if is_high(reset) {
            self.step = 0;
        } else if !is_high(prev_trigger) && is_high(trigger) && !params.is_empty() {
            self.step = (self.step + 1) % params.len();
        }

        let value = if params.is_empty() {
            0.0
        } else {
            params[self.step % params.len()]
        };

        vec![value, self.step as f64]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.step as f64]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.step = v[0] as usize;
        }
    }

    fn block_type(&self) -> &str {
        "Sequencer"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// WARNING: Block type 'SequenceController' behavior unknown — using pass-through stub.
// Actual Loxone behavior not documented. Will be validated later.
stub_block!(
    /// Sequence controller — pass-through stub.
    SequenceController,
    "SequenceController"
);

// ===========================================================================
// Meter variants (stat-based)
// ===========================================================================

// WARNING: Simplified models — real Loxone meter behavior may differ.
// MeterAbsSt/MeterPSt are statistical meter variants. Validate against Miniserver.

/// Absolute meter with statistical tracking — accumulates input.
#[derive(Clone, Default)]
pub struct MeterAbsSt {
    total: f64,
}

impl MeterAbsSt {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for MeterAbsSt {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        if dt > 0.0 {
            self.total += value * dt;
        }
        vec![self.total, value]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.total]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.total = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "MeterAbsSt"
    }
}

/// Power meter with statistical tracking — accumulates input.
#[derive(Clone, Default)]
pub struct MeterPSt {
    total: f64,
}

impl MeterPSt {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for MeterPSt {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        if dt > 0.0 {
            self.total += value * dt / 3600.0;
        }
        vec![self.total, value]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.total]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.total = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "MeterPSt"
    }
}

// ===========================================================================
// Central controller blocks (aggregate patterns)
// ===========================================================================

central_block!(
    /// Central light control — aggregates light zone states.
    CentralLight,
    "CentralLight"
);

central_block!(
    /// Central music control — aggregates music zone states.
    CentralMusic,
    "CentralMusic"
);

central_block!(
    /// Central gate control — aggregates gate states.
    CentralGate,
    "CentralGate"
);

central_block!(
    /// Central presence — aggregates presence zone states.
    CentralPresence,
    "CentralPresence"
);

central_block!(
    /// Central fancoil — aggregates fancoil states.
    CentralFancoil,
    "CentralFancoil"
);

central_block!(
    /// Central roof window — aggregates roof window states.
    CentralRoofwindow,
    "CentralRoofwindow"
);

central_block!(
    /// Central shade — aggregates shade/blind states.
    CentralShade,
    "CentralShade"
);

// ===========================================================================
// PicoC code blocks (not simulated)
// ===========================================================================

code_block!(
    /// PicoC code block with 1 I/O pair — user scripts not simulated.
    Code1,
    "Code1"
);

code_block!(
    /// PicoC code block with 4 I/O pairs — user scripts not simulated.
    Code4,
    "Code4"
);

code_block!(
    /// PicoC code block with 8 I/O pairs — user scripts not simulated.
    Code8,
    "Code8"
);

code_block!(
    /// PicoC code block with 16 I/O pairs — user scripts not simulated.
    Code16,
    "Code16"
);

// ---------------------------------------------------------------------------
// Lighting scene variants
// ---------------------------------------------------------------------------

stub_block!(
    /// Lightscene — scene snapshot/recall.
    Lightscene,
    "Lightscene"
);

stub_block!(
    /// Lightscene with learn capability.
    LightsceneLearn,
    "LightsceneLearn"
);

stub_block!(
    /// RGB lightscene — scene with color data.
    LightsceneRGB,
    "LightsceneRGB"
);

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: LightControllerH is a simplified LightController variant.
stub_block!(
    /// Lighting controller (hotel variant) — pass-through stub.
    LightControllerH,
    "LightControllerH"
);

// ---------------------------------------------------------------------------
// HVAC / climate stubs
// ---------------------------------------------------------------------------

// WARNING: Block type behavior unknown for these HVAC blocks — using pass-through stubs.
// Actual Loxone behavior not documented. Will be validated later.
// Note: HVACController moved to controllers.rs with real implementation.

stub_block!(
    /// HVAC AC control — pass-through stub.
    HvacAC,
    "HvacAC"
);

stub_block!(
    /// Heat central — pass-through stub.
    HeatCentral,
    "HeatCentral"
);

stub_block!(
    /// Climate controller (US variant) — pass-through stub.
    ClimateControllerUS,
    "ClimateControllerUS"
);

// ===========================================================================
// Media / device stubs
// ===========================================================================

stub_block!(
    /// Media player — pass-through stub.
    Media,
    "Media"
);

stub_block!(
    /// Media client — pass-through stub.
    MediaClient,
    "MediaClient"
);

stub_block!(
    /// Music player — pass-through stub.
    MusicPlayer,
    "MusicPlayer"
);

stub_block!(
    /// Radio (v1) — pass-through stub.
    Radio,
    "Radio"
);

stub_block!(
    /// Radio (v2) — pass-through stub.
    Radio2,
    "Radio2"
);

stub_block!(
    /// Intercom — pass-through stub.
    Intercom,
    "Intercom"
);

stub_block!(
    /// Tablet device — pass-through stub.
    Tablet,
    "Tablet"
);

/// Device Tablet (name has space in Loxone XML) — pass-through stub.
// WARNING: Block type 'Device Tablet' behavior unknown — using pass-through stub.
// Actual Loxone behavior not documented. Will be validated later.
#[derive(Clone, Copy)]
pub struct DeviceTablet;

impl Block for DeviceTablet {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        vec![inputs.first().copied().unwrap_or(0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Device Tablet"
    }
}

stub_block!(
    /// Nevo remote control — pass-through stub.
    Nevo,
    "Nevo"
);

stub_block!(
    /// NFC code touch — pass-through stub.
    NfcCodeTouch,
    "NfcCodeTouch"
);

stub_block!(
    /// Music player group — pass-through stub.
    MPGroup,
    "MPGroup"
);

// ===========================================================================
// Button / switch stubs
// ===========================================================================

stub_block!(
    /// Push button with timer — pass-through stub.
    PButtonT,
    "PButtonT"
);

stub_block!(
    /// Multi-function switch — pass-through stub.
    MultiFuncSW,
    "MultiFuncSW"
);

// ===========================================================================
// Automation stubs
// ===========================================================================

stub_block!(
    /// Automatic scene — pass-through stub.
    AutomaticScene,
    "AutomaticScene"
);

stub_block!(
    /// Autopilot rule — pass-through stub.
    AutopilotRule,
    "AutopilotRule"
);

stub_block!(
    /// Brightness control — pass-through stub.
    BrightnessControl,
    "BrightnessControl"
);

stub_block!(
    /// Command recognition — pass-through stub.
    CmdRecognition,
    "CmdRecognition"
);

// ===========================================================================
// Communication stubs
// ===========================================================================

stub_block!(
    /// Call generator — pass-through stub.
    CallGen,
    "CallGen"
);

stub_block!(
    /// Mail box — pass-through stub.
    MailBox,
    "MailBox"
);

stub_block!(
    /// Mail generator — pass-through stub.
    MailGen,
    "MailGen"
);

stub_block!(
    /// System variable proxy — pass-through stub.
    SysVar,
    "SysVar"
);

stub_block!(
    /// Text generator — pass-through stub.
    TextGenerator,
    "TextGenerator"
);

// ===========================================================================
// Infrastructure stubs
// ===========================================================================

stub_block!(
    /// Ping monitor — pass-through stub.
    Ping,
    "Ping"
);

stub_block!(
    /// Power unit — pass-through stub.
    PowerUnit,
    "PowerUnit"
);

stub_block!(
    /// Car charger — pass-through stub.
    CarCharger,
    "CarCharger"
);

stub_block!(
    /// Status monitor — pass-through stub.
    StatusMonitor,
    "StatusMonitor"
);

stub_block!(
    /// Steak thermometer — pass-through stub.
    SteakThermo,
    "SteakThermo"
);

stub_block!(
    /// Spot optimization — pass-through stub.
    SpotOpt,
    "SpotOpt"
);

// ===========================================================================
// Access / door stubs
// ===========================================================================

stub_block!(
    /// Door control — pass-through stub.
    Door,
    "Door"
);

stub_block!(
    /// Door controller — pass-through stub.
    Doorcontroller,
    "Doorcontroller"
);

// ---------------------------------------------------------------------------
// Shade / window stubs
// ---------------------------------------------------------------------------

stub_block!(
    /// Roof window control — pass-through stub.
    RoofWindow,
    "RoofWindow"
);

stub_block!(
    /// Shade/roof hybrid — pass-through stub.
    ShadeRoof,
    "ShadeRoof"
);

// ---------------------------------------------------------------------------
// Outdoor / garden stubs
// ---------------------------------------------------------------------------

stub_block!(
    /// Irrigation control — pass-through stub.
    Irrigation,
    "Irrigation"
);

stub_block!(
    /// Leaf wetness sensor — pass-through stub.
    Leaf,
    "Leaf"
);

stub_block!(
    /// Weed sensor — pass-through stub.
    Weed,
    "Weed"
);

stub_block!(
    /// Wind sensor — pass-through stub.
    Wind,
    "Wind"
);

// ===========================================================================
// Database / connectivity stubs
// ===========================================================================

stub_block!(
    /// Database connector (event) — pass-through stub.
    DbConE,
    "DbConE"
);

stub_block!(
    /// Database connector (state) — pass-through stub.
    DbConS,
    "DbConS"
);

stub_block!(
    /// WBEM monitoring — pass-through stub.
    WBEM,
    "WBEM"
);

stub_block!(
    /// EFM (extended function module) — pass-through stub.
    EFM,
    "EFM"
);

// ===========================================================================
// Misc controller stubs
// ===========================================================================

stub_block!(
    /// Room controller — pass-through stub.
    Roomcontrol,
    "Roomcontrol"
);

stub_block!(
    /// Windows monitor — pass-through stub.
    WindowsMonitor,
    "WindowsMonitor"
);

stub_block!(
    /// TPF controller — pass-through stub.
    TpfController,
    "TpfController"
);

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Average / Avg ---

    #[test]
    fn average_computes_mean() {
        let mut block = Average;
        assert_eq!(block.eval(&[2.0, 4.0, 6.0], &[], 0.0, &[]), vec![4.0]);
        assert_eq!(block.eval(&[10.0], &[], 0.0, &[]), vec![10.0]);
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn avg_computes_mean() {
        let mut block = Avg;
        assert_eq!(block.eval(&[3.0, 6.0, 9.0], &[], 0.0, &[]), vec![6.0]);
    }

    // --- BinEncoder / BinDecoder ---

    #[test]
    fn bin_encoder_combines_bits() {
        let mut block = BinEncoder;
        // bit0=1, bit1=0, bit2=1 → 5
        assert_eq!(block.eval(&[1.0, 0.0, 1.0], &[], 0.0, &[]), vec![5.0]);
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn bin_decoder_splits_bits() {
        let mut block = BinDecoder;
        // 5 = 101 in binary → [1, 0, 1, 0] (4 bits default? no, 8 bits default)
        let out = block.eval(&[5.0], &[4.0], 0.0, &[]);
        assert_eq!(out, vec![1.0, 0.0, 1.0, 0.0]);
    }

    // --- DewPoint ---

    #[test]
    fn dew_point_calculation() {
        let mut block = DewPoint;
        // At 20°C and 50% RH, dew point ≈ 9.3°C
        let out = block.eval(&[20.0, 50.0], &[], 0.0, &[]);
        assert!((out[0] - 9.3).abs() < 0.5);
    }

    #[test]
    fn dew_point_at_saturation() {
        let mut block = DewPoint;
        // At 100% RH, dew point ≈ air temperature
        let out = block.eval(&[25.0, 100.0], &[], 0.0, &[]);
        assert!((out[0] - 25.0).abs() < 0.1);
    }

    #[test]
    fn dew_point_very_low_humidity() {
        let mut block = DewPoint;
        // At very low humidity, dew point is far below air temp
        let out = block.eval(&[20.0, 1.0], &[], 0.0, &[]);
        assert!(out[0] < -20.0);
    }

    #[test]
    fn dew_point_negative_temp() {
        let mut block = DewPoint;
        // At -10°C, 80% RH → dew point ≈ -12.8°C
        let out = block.eval(&[-10.0, 80.0], &[], 0.0, &[]);
        assert!(out[0] < -10.0);
        assert!(out[0] > -20.0);
    }

    #[test]
    fn dew_point_hot_humid() {
        let mut block = DewPoint;
        // At 35°C, 90% RH → dew point ≈ 33°C
        let out = block.eval(&[35.0, 90.0], &[], 0.0, &[]);
        assert!((out[0] - 33.2).abs() < 1.0);
    }

    // --- Power ---

    #[test]
    fn power_exponentiation() {
        let mut block = Power;
        assert_eq!(block.eval(&[2.0, 3.0], &[], 0.0, &[]), vec![8.0]);
        assert_eq!(block.eval(&[3.0], &[2.0], 0.0, &[]), vec![9.0]);
    }

    // --- Validator ---

    #[test]
    fn validator_range_check() {
        let mut block = Validator;
        assert_eq!(block.eval(&[50.0], &[0.0, 100.0], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[150.0], &[0.0, 100.0], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[-1.0], &[0.0, 100.0], 0.0, &[]), vec![0.0]);
    }

    // --- TimeMinmax ---

    #[test]
    fn time_minmax_tracks_range() {
        let mut block = TimeMinmax::new();
        let out = block.eval(&[5.0, 0.0], &[], 0.0, &[]);
        assert_eq!(out, vec![5.0, 5.0, 5.0]); // initial

        let out = block.eval(&[3.0, 0.0], &[], 0.1, &[]);
        assert_eq!(out, vec![3.0, 3.0, 5.0]);

        let out = block.eval(&[7.0, 0.0], &[], 0.1, &[]);
        assert_eq!(out, vec![7.0, 3.0, 7.0]);

        // Reset
        let out = block.eval(&[10.0, 1.0], &[], 0.1, &[]);
        assert_eq!(out, vec![10.0, 10.0, 10.0]);
    }

    #[test]
    fn time_minmax_state_roundtrip() {
        let mut block = TimeMinmax::new();
        block.eval(&[5.0, 0.0], &[], 0.0, &[]);
        block.eval(&[3.0, 0.0], &[], 0.1, &[]);
        let state = block.state().unwrap();
        let mut restored = TimeMinmax::new();
        restored.restore(&state);
        assert_eq!(restored.min_val, block.min_val);
        assert_eq!(restored.max_val, block.max_val);
    }

    // --- Ramp ---

    #[test]
    fn ramp_approaches_target() {
        let mut block = Ramp::new();
        // Target=10, rate=5/s, dt=1s → output should be 5
        let out = block.eval(&[10.0], &[5.0], 1.0, &[]);
        assert!((out[0] - 5.0).abs() < f64::EPSILON);

        // Another second → 10
        let out = block.eval(&[10.0], &[5.0], 1.0, &[]);
        assert!((out[0] - 10.0).abs() < f64::EPSILON);

        // Overshoot protection — stays at target
        let out = block.eval(&[10.0], &[5.0], 1.0, &[]);
        assert!((out[0] - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ramp_state_roundtrip() {
        let mut block = Ramp::new();
        block.eval(&[10.0], &[5.0], 1.0, &[]);
        let state = block.state().unwrap();
        let mut restored = Ramp::new();
        restored.restore(&state);
        assert!((restored.output - 5.0).abs() < f64::EPSILON);
    }

    // --- Rand ---

    #[test]
    fn rand_generates_on_trigger() {
        let mut block = Rand::new();
        // No trigger → min value
        let out = block.eval(&[0.0], &[0.0, 100.0], 0.0, &[0.0]);
        assert_eq!(out[0], 0.0);

        // Rising edge → random value in [0, 100]
        let out = block.eval(&[1.0], &[0.0, 100.0], 0.0, &[0.0]);
        assert!(out[0] >= 0.0 && out[0] <= 100.0);
    }

    // --- PulseBy ---

    #[test]
    fn pulse_by_fires_on_trigger() {
        let mut block = PulseBy::new();
        // Rising edge starts pulse
        let out = block.eval(&[1.0], &[2.0], 0.0, &[0.0]);
        assert_eq!(out[0], 1.0);

        // Count down
        let out = block.eval(&[0.0], &[2.0], 1.0, &[1.0]);
        assert_eq!(out[0], 1.0); // still active

        let out = block.eval(&[0.0], &[2.0], 1.0, &[0.0]);
        assert_eq!(out[0], 0.0); // expired
    }

    // --- LongClick ---

    #[test]
    fn long_click_detects_hold() {
        let mut block = LongClick::new();
        // Press and hold
        let out = block.eval(&[1.0], &[1.0], 0.5, &[]);
        assert_eq!(out[0], 0.0); // not yet

        let out = block.eval(&[1.0], &[1.0], 0.5, &[]);
        assert_eq!(out[0], 1.0); // threshold reached

        let out = block.eval(&[1.0], &[1.0], 0.5, &[]);
        assert_eq!(out[0], 0.0); // already fired

        // Release and re-press
        block.eval(&[0.0], &[1.0], 0.0, &[]);
        let out = block.eval(&[1.0], &[1.0], 1.0, &[]);
        assert_eq!(out[0], 1.0); // fires again
    }

    // --- MultiClick ---

    #[test]
    fn multi_click_counts_clicks() {
        let mut block = MultiClick::new();
        // First click
        block.eval(&[1.0], &[1.0], 0.0, &[0.0]);
        block.eval(&[0.0], &[1.0], 0.1, &[1.0]);
        // Second click
        block.eval(&[1.0], &[1.0], 0.0, &[0.0]);
        block.eval(&[0.0], &[1.0], 0.1, &[1.0]);
        // Wait for timeout
        let out = block.eval(&[0.0], &[1.0], 1.0, &[0.0]);
        assert_eq!(out[0], 2.0); // two clicks
    }

    // --- PushDimmer ---

    #[test]
    fn push_dimmer_ramps() {
        let mut block = PushDimmer::new();
        // Hold for 1 second at rate 0.5/s → value = 0.5
        let out = block.eval(&[1.0], &[0.5], 1.0, &[0.0]);
        assert!((out[0] - 0.5).abs() < 0.01);
    }

    // --- StepSel ---

    #[test]
    fn step_sel_cycles() {
        let mut block = StepSel::new();
        assert_eq!(block.eval(&[1.0, 0.0], &[3.0], 0.0, &[0.0])[0], 1.0);
        assert_eq!(block.eval(&[1.0, 0.0], &[3.0], 0.0, &[0.0])[0], 2.0);
        assert_eq!(block.eval(&[1.0, 0.0], &[3.0], 0.0, &[0.0])[0], 0.0); // wrap
    }

    // --- Sequencer ---

    #[test]
    fn sequencer_steps_through_values() {
        let mut block = Sequencer::new();
        let out = block.eval(&[1.0, 0.0], &[10.0, 20.0, 30.0], 0.0, &[0.0]);
        assert_eq!(out[0], 20.0);
        assert_eq!(out[1], 1.0);

        let out = block.eval(&[1.0, 0.0], &[10.0, 20.0, 30.0], 0.0, &[0.0]);
        assert_eq!(out[0], 30.0);
        assert_eq!(out[1], 2.0);

        let out = block.eval(&[1.0, 0.0], &[10.0, 20.0, 30.0], 0.0, &[0.0]);
        assert_eq!(out[0], 10.0); // wraps
        assert_eq!(out[1], 0.0);
    }

    // --- Central blocks ---

    #[test]
    fn central_light_aggregates() {
        let mut block = CentralLight;
        let out = block.eval(&[0.0, 1.0, 0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        assert_eq!(out[1], 2.0);
    }

    // --- Meter variants ---

    #[test]
    fn meter_abs_st_accumulates() {
        let mut block = MeterAbsSt::new();
        block.eval(&[100.0], &[], 1.0, &[]); // 100 * 1 = 100
        let out = block.eval(&[200.0], &[], 1.0, &[]); // 200 * 1 = 200, total = 300
        assert!((out[0] - 300.0).abs() < f64::EPSILON);
    }

    // --- Code blocks ---

    #[test]
    fn code_blocks_pass_through() {
        let mut c1 = Code1;
        let mut c4 = Code4;
        let mut c8 = Code8;
        let mut c16 = Code16;
        assert_eq!(c1.eval(&[5.0], &[], 0.0, &[]), vec![5.0]);
        assert_eq!(c4.eval(&[5.0], &[], 0.0, &[]), vec![5.0]);
        assert_eq!(c8.eval(&[5.0], &[], 0.0, &[]), vec![5.0]);
        assert_eq!(c16.eval(&[5.0], &[], 0.0, &[]), vec![5.0]);
    }

    // --- Stub blocks have correct type names ---

    #[test]
    fn stub_blocks_return_correct_type_names() {
        assert_eq!(Media.block_type(), "Media");
        assert_eq!(Tablet.block_type(), "Tablet");
        assert_eq!(DeviceTablet.block_type(), "Device Tablet");
        assert_eq!(Door.block_type(), "Door");
        assert_eq!(Doorcontroller.block_type(), "Doorcontroller");
        assert_eq!(Irrigation.block_type(), "Irrigation");
        assert_eq!(WindowsMonitor.block_type(), "WindowsMonitor");
    }

    // --- Factory test ---

    #[test]
    fn factory_creates_all_misc_types() {
        use crate::blocks::create_block;
        let types = [
            "Average",
            "Avg",
            "BinDecoder",
            "BinEncoder",
            "BrightnessControl",
            "CallGen",
            "CarCharger",
            "CentralFancoil",
            "CentralGate",
            "CentralLight",
            "CentralMusic",
            "CentralPresence",
            "CentralRoofwindow",
            "CentralShade",
            "CmdRecognition",
            "Code1",
            "Code4",
            "Code8",
            "Code16",
            "DbConE",
            "DbConS",
            "Device Tablet",
            "DewPoint",
            "Door",
            "Doorcontroller",
            "EFM",
            "HeatCentral",
            "HvacAC",
            "Intercom",
            "Irrigation",
            "Leaf",
            "LightControllerH",
            "Lightscene",
            "LightsceneLearn",
            "LightsceneRGB",
            "LongClick",
            "MPGroup",
            "MailBox",
            "MailGen",
            "Media",
            "MediaClient",
            "MeterAbsSt",
            "MeterPSt",
            "MultiClick",
            "MultiFuncSW",
            "MusicPlayer",
            "Nevo",
            "NfcCodeTouch",
            "PButtonT",
            "Ping",
            "Power",
            "PowerUnit",
            "PulseBy",
            "PushDimmer",
            "Radio",
            "Radio2",
            "Ramp",
            "Rand",
            "RandomGen",
            "Roomcontrol",
            "SequenceController",
            "Sequencer",
            "SpotOpt",
            "StatusMonitor",
            "SteakThermo",
            "StepSel",
            "SysVar",
            "Tablet",
            "TextGenerator",
            "TimeMinmax",
            "TpfController",
            "Validator",
            "WBEM",
            "Weed",
            "Wind",
            "WindowsMonitor",
            "AutomaticScene",
            "AutopilotRule",
        ];
        for name in &types {
            let block = create_block(name);
            assert_eq!(block.block_type(), *name, "Factory mismatch for {name}");
        }
    }
}
