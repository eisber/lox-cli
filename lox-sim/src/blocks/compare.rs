use crate::blocks::{
    bool_signal, deserialize_bool, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

fn operands(inputs: &[Signal], params: &[Signal]) -> (Signal, Signal) {
    let left = inputs.first().copied().unwrap_or(0.0);
    let right = inputs
        .get(1)
        .copied()
        .or_else(|| params.first().copied())
        .unwrap_or(0.0);
    (left, right)
}

#[derive(Clone, Copy)]
pub struct GreaterEqual;
#[derive(Clone, Copy)]
pub struct Less;
#[derive(Clone, Copy)]
pub struct Greater;
#[derive(Clone, Copy)]
pub struct LessEqual;

impl Block for GreaterEqual {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let (left, right) = operands(inputs, params);
        vec![bool_signal(left >= right)]
    }
    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str {
        "GreaterEqual"
    }
}

impl Block for Less {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let (left, right) = operands(inputs, params);
        vec![bool_signal(left < right)]
    }
    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str {
        "Less"
    }
}

impl Block for Greater {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let (left, right) = operands(inputs, params);
        vec![bool_signal(left > right)]
    }
    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str {
        "Greater"
    }
}

impl Block for LessEqual {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let (left, right) = operands(inputs, params);
        vec![bool_signal(left <= right)]
    }
    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str {
        "LessEqual"
    }
}

/// Threshold switch with hysteresis and edge pulse outputs.
#[derive(Clone)]
pub struct AnalogThresholdTrigger {
    is_on: bool,
}

impl AnalogThresholdTrigger {
    pub fn new() -> Self {
        Self { is_on: false }
    }
}

impl Default for AnalogThresholdTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AnalogThresholdTrigger {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let on_threshold = params.first().copied().unwrap_or(1.0);
        let off_threshold = params.get(1).copied().unwrap_or(on_threshold);
        let was_on = self.is_on;

        if input >= on_threshold {
            self.is_on = true;
        } else if input <= off_threshold {
            self.is_on = false;
        }

        let q = bool_signal(self.is_on);
        let rising = bool_signal(!was_on && self.is_on);
        let falling = bool_signal(was_on && !self.is_on);
        vec![q, rising, falling]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.is_on))
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_on = deserialize_bool(state).unwrap_or(false);
    }

    fn block_type(&self) -> &str {
        "AnalogThresholdTrigger"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Compare input against a threshold with configurable hysteresis.
///
/// Inputs: [value, threshold]
/// Params: [hysteresis]
/// Outputs: [Q (digital), QInv (inverted)]
///
/// Q goes high when value > threshold + hysteresis.
/// Q goes low when value < threshold - hysteresis.
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Symmetric hysteresis around threshold. Q and QInv are complementary.
/// // TODO: Validate against real Miniserver behavior
#[derive(Clone)]
pub struct AnalogComparator {
    is_high: bool,
}

impl AnalogComparator {
    pub fn new() -> Self {
        Self { is_high: false }
    }
}

impl Default for AnalogComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AnalogComparator {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let threshold = inputs.get(1).copied().unwrap_or(0.0);
        let hysteresis = params.first().copied().unwrap_or(0.0);

        if value > threshold + hysteresis {
            self.is_high = true;
        } else if value < threshold - hysteresis {
            self.is_high = false;
        }

        let q = bool_signal(self.is_high);
        let q_inv = bool_signal(!self.is_high);
        vec![q, q_inv]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.is_high))
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_high = deserialize_bool(state).unwrap_or(false);
    }

    fn block_type(&self) -> &str {
        "AnalogComparator"
    }
}

/// Triggers when the rate of change exceeds a threshold.
///
/// Inputs: [value]
/// Params: [rate_threshold]
/// Outputs: [Q (digital), rate_of_change (analog)]
///
/// Computes rate = |value - prev_value| / dt. If rate > rate_threshold, Q = 1.
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Rate computed as |delta_value / dt|. Trigger is non-latching
/// //   (output goes low as soon as rate drops below threshold).
/// // TODO: Validate against real Miniserver behavior
#[derive(Clone)]
pub struct AnalogDiffTrigger {
    prev_value: f64,
    initialized: bool,
}

impl AnalogDiffTrigger {
    pub fn new() -> Self {
        Self {
            prev_value: 0.0,
            initialized: false,
        }
    }
}

impl Default for AnalogDiffTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AnalogDiffTrigger {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let rate_threshold = params.first().copied().unwrap_or(1.0);

        let rate = if !self.initialized || dt <= f64::EPSILON {
            self.initialized = true;
            self.prev_value = value;
            0.0
        } else {
            let r = (value - self.prev_value).abs() / dt;
            self.prev_value = value;
            r
        };

        let triggered = rate > rate_threshold;
        vec![bool_signal(triggered), rate]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let init_byte = if self.initialized { 1u8 } else { 0u8 };
        let mut bytes = serialize_f64s(&[self.prev_value]);
        bytes.push(init_byte);
        Some(bytes)
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(vals) = deserialize_f64s(state, 1) {
            self.prev_value = vals[0];
            self.initialized = state.get(8).copied().unwrap_or(0) != 0;
        }
    }

    fn block_type(&self) -> &str {
        "AnalogDiffTrigger"
    }
}

/// Alarm if value is out of [min, max] range for longer than a specified duration.
///
/// Inputs: [value, reset (digital)]
/// Params: [min, max, duration_seconds]
/// Outputs: [alarm (digital), out_of_range (digital)]
///
/// out_of_range goes high immediately when value < min or value > max.
/// alarm goes high only after out_of_range has been continuously true for duration_seconds.
/// A rising edge on reset clears the alarm and timer.
///
/// // WARNING: Assumed behavior — Loxone internal implementation unknown.
/// // Assumption: Duration is accumulated over consecutive out-of-range ticks.
/// //   Reset clears both alarm and accumulated time. Alarm latches until reset
/// //   or value returns to range.
/// // TODO: Validate against real Miniserver behavior
#[derive(Clone)]
pub struct AnalogWatchdog {
    accumulated_time: f64,
    alarm: bool,
}

impl AnalogWatchdog {
    pub fn new() -> Self {
        Self {
            accumulated_time: 0.0,
            alarm: false,
        }
    }
}

impl Default for AnalogWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AnalogWatchdog {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);
        let prev_reset = prev.get(1).copied().unwrap_or(0.0);

        let min_val = params.first().copied().unwrap_or(0.0);
        let max_val = params.get(1).copied().unwrap_or(100.0);
        let duration = params.get(2).copied().unwrap_or(0.0);

        // Reset on rising edge
        if is_high(reset) && !is_high(prev_reset) {
            self.accumulated_time = 0.0;
            self.alarm = false;
        }

        let out_of_range = value < min_val || value > max_val;

        if out_of_range {
            self.accumulated_time += dt;
            if self.accumulated_time >= duration {
                self.alarm = true;
            }
        } else {
            self.accumulated_time = 0.0;
            self.alarm = false;
        }

        vec![bool_signal(self.alarm), bool_signal(out_of_range)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let alarm_byte = if self.alarm { 1u8 } else { 0u8 };
        let mut bytes = serialize_f64s(&[self.accumulated_time]);
        bytes.push(alarm_byte);
        Some(bytes)
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(vals) = deserialize_f64s(state, 1) {
            self.accumulated_time = vals[0];
            self.alarm = state.get(8).copied().unwrap_or(0) != 0;
        }
    }

    fn block_type(&self) -> &str {
        "AnalogWatchdog"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::is_high;

    #[test]
    fn compare_blocks_use_inputs_or_params() {
        let mut ge = GreaterEqual;
        let mut lt = Less;
        let mut gt = Greater;
        let mut le = LessEqual;

        assert_eq!(ge.eval(&[5.0], &[5.0], 0.0, &[]), vec![1.0]);
        assert_eq!(lt.eval(&[4.0], &[5.0], 0.0, &[]), vec![1.0]);
        assert_eq!(gt.eval(&[6.0], &[5.0], 0.0, &[]), vec![1.0]);
        assert_eq!(le.eval(&[5.0], &[5.0], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn analog_threshold_trigger_holds_state_between_thresholds() {
        let mut block = AnalogThresholdTrigger::new();
        assert_eq!(
            block.eval(&[0.0], &[10.0, 8.0], 0.0, &[]),
            vec![0.0, 0.0, 0.0]
        );
        assert_eq!(
            block.eval(&[10.0], &[10.0, 8.0], 0.0, &[]),
            vec![1.0, 1.0, 0.0]
        );
        assert_eq!(
            block.eval(&[9.0], &[10.0, 8.0], 0.0, &[]),
            vec![1.0, 0.0, 0.0]
        );
        assert_eq!(
            block.eval(&[8.0], &[10.0, 8.0], 0.0, &[]),
            vec![0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn analog_threshold_trigger_state_roundtrip() {
        let mut block = AnalogThresholdTrigger::new();
        block.eval(&[1.0], &[1.0, 0.0], 0.0, &[]);
        let state = block.state().unwrap();
        let mut restored = AnalogThresholdTrigger::new();
        restored.restore(&state);
        assert_eq!(
            restored.eval(&[0.5], &[1.0, 0.0], 0.0, &[1.0]),
            vec![1.0, 0.0, 0.0]
        );
        assert!(is_high(restored.eval(&[0.0], &[1.0, 0.0], 0.0, &[0.5])[2]));
    }

    #[test]
    fn analog_comparator_with_hysteresis() {
        let mut block = AnalogComparator::new();
        // threshold=10, hysteresis=2 → on at >12, off at <8
        assert_eq!(block.eval(&[9.0, 10.0], &[2.0], 0.0, &[]), vec![0.0, 1.0]);
        assert_eq!(block.eval(&[13.0, 10.0], &[2.0], 0.0, &[]), vec![1.0, 0.0]);
        // In hysteresis band — stays high
        assert_eq!(block.eval(&[9.0, 10.0], &[2.0], 0.0, &[]), vec![1.0, 0.0]);
        // Drop below threshold - hysteresis
        assert_eq!(block.eval(&[7.0, 10.0], &[2.0], 0.0, &[]), vec![0.0, 1.0]);
    }

    #[test]
    fn analog_comparator_no_hysteresis() {
        let mut block = AnalogComparator::new();
        assert_eq!(block.eval(&[5.0, 3.0], &[0.0], 0.0, &[]), vec![1.0, 0.0]);
        assert_eq!(block.eval(&[2.0, 3.0], &[0.0], 0.0, &[]), vec![0.0, 1.0]);
    }

    #[test]
    fn analog_comparator_state_roundtrip() {
        let mut block = AnalogComparator::new();
        block.eval(&[15.0, 10.0], &[2.0], 0.0, &[]);
        let state = block.state().unwrap();
        let mut restored = AnalogComparator::new();
        restored.restore(&state);
        // Should remain high (in hysteresis band)
        assert_eq!(
            restored.eval(&[9.0, 10.0], &[2.0], 0.0, &[]),
            vec![1.0, 0.0]
        );
    }

    #[test]
    fn analog_diff_trigger_detects_fast_change() {
        let mut block = AnalogDiffTrigger::new();
        // First eval: initializes, rate = 0
        let result = block.eval(&[0.0], &[5.0], 1.0, &[]);
        assert_eq!(result[0], 0.0); // not triggered
        assert!((result[1] - 0.0).abs() < 1e-10); // rate = 0

        // Slow change: delta=2 in 1s → rate=2, threshold=5 → not triggered
        let result = block.eval(&[2.0], &[5.0], 1.0, &[]);
        assert_eq!(result[0], 0.0);
        assert!((result[1] - 2.0).abs() < 1e-10);

        // Fast change: delta=10 in 1s → rate=10, threshold=5 → triggered
        let result = block.eval(&[12.0], &[5.0], 1.0, &[]);
        assert_eq!(result[0], 1.0);
        assert!((result[1] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn analog_diff_trigger_zero_dt() {
        let mut block = AnalogDiffTrigger::new();
        block.eval(&[5.0], &[1.0], 1.0, &[]);
        // dt = 0 → rate = 0 (avoid division by zero)
        let result = block.eval(&[100.0], &[1.0], 0.0, &[]);
        assert_eq!(result[0], 0.0);
    }

    #[test]
    fn analog_diff_trigger_state_roundtrip() {
        let mut block = AnalogDiffTrigger::new();
        block.eval(&[10.0], &[5.0], 1.0, &[]);
        let state = block.state().unwrap();
        let mut restored = AnalogDiffTrigger::new();
        restored.restore(&state);
        // Next eval should use prev_value = 10.0
        let result = restored.eval(&[20.0], &[5.0], 1.0, &[]);
        assert_eq!(result[0], 1.0); // rate = 10 > threshold 5
    }

    #[test]
    fn analog_watchdog_triggers_after_duration() {
        let mut block = AnalogWatchdog::new();
        // params: min=5, max=15, duration=3.0
        // In range: no alarm
        let result = block.eval(&[10.0, 0.0], &[5.0, 15.0, 3.0], 1.0, &[0.0, 0.0]);
        assert_eq!(result, vec![0.0, 0.0]);

        // Out of range for 1s
        let result = block.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 1.0, &[10.0, 0.0]);
        assert_eq!(result, vec![0.0, 1.0]); // out_of_range but no alarm yet

        // Out of range for 2s more (total 3)
        let result = block.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 2.0, &[20.0, 0.0]);
        assert_eq!(result, vec![1.0, 1.0]); // alarm!
    }

    #[test]
    fn analog_watchdog_resets_on_return_to_range() {
        let mut block = AnalogWatchdog::new();
        block.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 2.0, &[0.0, 0.0]);
        // Return to range
        let result = block.eval(&[10.0, 0.0], &[5.0, 15.0, 3.0], 1.0, &[20.0, 0.0]);
        assert_eq!(result, vec![0.0, 0.0]);
    }

    #[test]
    fn analog_watchdog_reset_input() {
        let mut block = AnalogWatchdog::new();
        // Accumulate time out of range
        block.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 4.0, &[0.0, 0.0]);
        // Alarm should be on
        let result = block.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 0.0, &[20.0, 0.0]);
        assert_eq!(result[0], 1.0);
        // Reset rising edge
        let result = block.eval(&[20.0, 1.0], &[5.0, 15.0, 3.0], 0.0, &[20.0, 0.0]);
        // After reset, accumulated time is 0, still out of range but no alarm
        assert_eq!(result[0], 0.0);
        assert_eq!(result[1], 1.0);
    }

    #[test]
    fn analog_watchdog_state_roundtrip() {
        let mut block = AnalogWatchdog::new();
        block.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 2.0, &[0.0, 0.0]);
        let state = block.state().unwrap();
        let mut restored = AnalogWatchdog::new();
        restored.restore(&state);
        // Should continue accumulating from 2.0
        let result = restored.eval(&[20.0, 0.0], &[5.0, 15.0, 3.0], 1.5, &[20.0, 0.0]);
        assert_eq!(result, vec![1.0, 1.0]); // 2.0 + 1.5 >= 3.0
    }

    #[test]
    fn analog_watchdog_below_min() {
        let mut block = AnalogWatchdog::new();
        // Value below min
        let result = block.eval(&[2.0, 0.0], &[5.0, 15.0, 0.0], 0.1, &[0.0, 0.0]);
        assert_eq!(result, vec![1.0, 1.0]); // duration=0, instant alarm
    }
}
