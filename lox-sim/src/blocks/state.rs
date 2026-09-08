use crate::blocks::{
    bool_signal, deserialize_bool, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

/// Analogue memory: stores Input on a Trigger edge, clears on Reset.
#[derive(Clone)]
pub struct AMemory {
    value: f64,
}

impl AMemory {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
}

impl Default for AMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AMemory {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let trigger = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.get(1).copied().unwrap_or(0.0);

        if is_high(reset) {
            self.value = 0.0;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            self.value = input;
        }

        vec![self.value]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.value]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.value = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "AMemory"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// SR latch. Set dominates reset when both are active.
#[derive(Clone)]
pub struct FlipFlop {
    is_on: bool,
}

impl FlipFlop {
    pub fn new() -> Self {
        Self { is_on: false }
    }
}

impl Default for FlipFlop {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for FlipFlop {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let set = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);

        if is_high(set) {
            self.is_on = true;
        } else if is_high(reset) {
            self.is_on = false;
        }

        vec![bool_signal(self.is_on)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.is_on))
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_on = deserialize_bool(state).unwrap_or(false);
    }

    fn block_type(&self) -> &str {
        "FlipFlop"
    }
}

/// RS latch where Reset dominates Set. When both R and S are active, output is 0.
#[derive(Clone)]
pub struct RSFlipFlop {
    is_on: bool,
}

impl RSFlipFlop {
    pub fn new() -> Self {
        Self { is_on: false }
    }
}

impl Default for RSFlipFlop {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for RSFlipFlop {
    /// inputs: [0] = Set, [1] = Reset
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let set = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);

        // Reset dominates: check reset first
        if is_high(reset) {
            self.is_on = false;
        } else if is_high(set) {
            self.is_on = true;
        }

        vec![bool_signal(self.is_on)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.is_on))
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_on = deserialize_bool(state).unwrap_or(false);
    }

    fn block_type(&self) -> &str {
        "RSFlipFlop"
    }
}

/// Digital memory (latch): captures a digital input on a trigger edge, clears on reset.
#[derive(Clone)]
pub struct Memory {
    value: bool,
}

impl Memory {
    pub fn new() -> Self {
        Self { value: false }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for Memory {
    /// inputs: [0] = digital input, [1] = trigger, [2] = reset
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let trigger = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.get(1).copied().unwrap_or(0.0);

        if is_high(reset) {
            self.value = false;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            self.value = is_high(input);
        }

        vec![bool_signal(self.value)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.value))
    }

    fn restore(&mut self, state: &[u8]) {
        self.value = deserialize_bool(state).unwrap_or(false);
    }

    fn block_type(&self) -> &str {
        "Memory"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Toggle switch with explicit On input and edge pulse outputs.
#[derive(Clone)]
pub struct PushButton {
    is_on: bool,
}

impl PushButton {
    pub fn new() -> Self {
        Self { is_on: false }
    }
}

impl Default for PushButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for PushButton {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let force_on = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let disable = inputs.get(3).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let previous = self.is_on;

        // WARNING: Assumed behavior — not validated against Miniserver.
        // Assumption: Reset dominates On; InputDisable gates only the trigger.
        if is_high(reset) {
            self.is_on = false;
        } else if is_high(force_on) {
            self.is_on = true;
        } else if !is_high(disable) && !is_high(prev_trigger) && is_high(trigger) {
            self.is_on = !self.is_on;
        }

        let qon = !previous && self.is_on;
        let qoff = previous && !self.is_on;
        vec![bool_signal(self.is_on), bool_signal(qoff), bool_signal(qon)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.is_on))
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_on = deserialize_bool(state).unwrap_or(false);
    }

    fn block_type(&self) -> &str {
        "PushButton"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// PushButton with double-click detection.
///
/// Single click toggles the output. A double-click (two rising edges within
/// `double_click_time`) fires the QDoubleClick output for one tick.
///
/// Outputs: [0] = Q (toggle state), [1] = QOff pulse, [2] = QOn pulse,
///          [3] = QDoubleClick pulse
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Double-click window is measured from the first rising edge.
// If a second edge arrives within the window, QDoubleClick fires for one tick
// and the toggle state is NOT changed by the second click (it was already
// toggled by the first).
#[derive(Clone)]
pub struct PushButton2 {
    is_on: bool,
    since_last_edge: f64,
    awaiting_second: bool,
}

impl PushButton2 {
    pub fn new() -> Self {
        Self {
            is_on: false,
            since_last_edge: 0.0,
            awaiting_second: false,
        }
    }
}

impl Default for PushButton2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for PushButton2 {
    /// inputs: [0] = trigger (toggle on rising edge)
    /// params: [0] = double-click window (seconds, default 0.4)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let disable = inputs.get(3).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let dc_window = params.first().copied().unwrap_or(0.4).max(0.0);
        let previous = self.is_on;
        // WARNING: Assumed behavior — not validated against Miniserver.
        // Assumption: Reset dominates and cancels a pending double-click;
        // InputDisable gates only the trigger.
        if is_high(reset) {
            self.is_on = false;
            self.awaiting_second = false;
        }
        let rising =
            !is_high(reset) && !is_high(disable) && !is_high(prev_trigger) && is_high(trigger);
        let mut double_click = false;

        if self.awaiting_second {
            self.since_last_edge += dt;
            if rising {
                double_click = true;
                self.awaiting_second = false;
            } else if self.since_last_edge > dc_window {
                self.awaiting_second = false;
            }
        }

        if rising && !double_click {
            self.is_on = !self.is_on;
            self.awaiting_second = true;
            self.since_last_edge = 0.0;
        }

        let qon = !previous && self.is_on;
        let qoff = previous && !self.is_on;
        vec![
            bool_signal(self.is_on),
            bool_signal(qoff),
            bool_signal(qon),
            bool_signal(double_click),
        ]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.is_on);
        s.extend_from_slice(&serialize_f64s(&[self.since_last_edge]));
        s.push(u8::from(self.awaiting_second));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_on = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 1) {
                self.since_last_edge = v[0];
            }
        }
        if state.len() > 9 {
            self.awaiting_second = state[9] != 0;
        }
    }

    fn block_type(&self) -> &str {
        "PushButton2"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Counts rising edges on its first input.
#[derive(Clone)]
pub struct Counter {
    count: f64,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0.0 }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for Counter {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let current = inputs.first().copied().unwrap_or(0.0);
        let previous = prev_inputs.first().copied().unwrap_or(0.0);
        if !is_high(previous) && is_high(current) {
            self.count += 1.0;
        }
        vec![self.count]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.count]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.count = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "Counter"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Up/down counter with separate Up, Down, and Reset inputs.
///
/// Outputs: [0] = count value, [1] = Q (count > 0)
#[derive(Clone)]
pub struct UpDownCounter {
    count: f64,
    q_state: bool,
}

impl UpDownCounter {
    pub fn new() -> Self {
        Self {
            count: 0.0,
            q_state: false,
        }
    }
}

impl Default for UpDownCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for UpDownCounter {
    /// inputs: [0] = Trigger (rising edge increments/decrements),
    ///         [1] = InputDir (0=up, 1=down),
    ///         [2] = Reset (sets AQ=StartValue, Q=0)
    /// params: [0] = StartValue (default 0), [1] = OnValue (default 10),
    ///         [2] = OffValue (default 5)
    /// outputs: [0] = Q (hysteresis: ON when AQ>=OnValue, OFF when AQ<=OffValue),
    ///          [1] = AQ (current counter value)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let input_dir = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);

        let start_value = params.first().copied().unwrap_or(0.0);
        let on_value = params.get(1).copied().unwrap_or(10.0);
        let off_value = params.get(2).copied().unwrap_or(5.0);

        if is_high(reset) {
            self.count = start_value;
            self.q_state = false;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            if input_dir < 0.5 {
                self.count += 1.0;
            } else {
                self.count -= 1.0;
            }
        }

        // Hysteresis for Q output
        if self.count >= on_value {
            self.q_state = true;
        } else if self.count <= off_value {
            self.q_state = false;
        }

        vec![bool_signal(self.q_state), self.count]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.count, bool_signal(self.q_state)]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 2) {
            self.count = values[0];
            self.q_state = is_high(values[1]);
        }
    }

    fn block_type(&self) -> &str {
        "UpDownCounter"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Counts the total hours that an input has been active.
///
/// While input is high, accumulated_seconds increments by dt each tick.
/// A Reset input clears the counter.
///
/// Outputs: [0] = accumulated hours (f64), [1] = Q (currently active)
#[derive(Clone)]
pub struct HourCounter {
    accumulated_seconds: f64,
}

impl HourCounter {
    pub fn new() -> Self {
        Self {
            accumulated_seconds: 0.0,
        }
    }
}

impl Default for HourCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for HourCounter {
    /// inputs: [0] = active, [1] = reset
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let active = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);

        if is_high(reset) {
            self.accumulated_seconds = 0.0;
        } else if is_high(active) {
            self.accumulated_seconds += dt;
        }

        let hours = self.accumulated_seconds / 3600.0;
        vec![hours, bool_signal(is_high(active))]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.accumulated_seconds]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.accumulated_seconds = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "HourCounter"
    }
}

/// Sample-and-hold: captures the analog input when the trigger fires a rising
/// edge, holds the last captured value between triggers.
///
/// Outputs: [0] = held value
#[derive(Clone)]
pub struct SampleHold {
    held: f64,
}

impl SampleHold {
    pub fn new() -> Self {
        Self { held: 0.0 }
    }
}

impl Default for SampleHold {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for SampleHold {
    /// inputs: [0] = analog input, [1] = trigger (sample on rising edge), [2] = reset
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let trigger = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.get(1).copied().unwrap_or(0.0);

        if is_high(reset) {
            self.held = 0.0;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            self.held = input;
        }

        vec![self.held]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.held]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.held = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "SampleHold"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Shift register: on each trigger rising edge, shifts all cells by one position,
/// inserting the current input at position 0. The oldest value falls off the end.
///
/// params: [0] = register length (default 4, max 32)
/// Outputs: one signal per register cell, [0] = newest, [N-1] = oldest
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Each rising edge on the trigger shifts cells right, inserting
// the input at position 0. Analog values are shifted (not just digital).
#[derive(Clone)]
pub struct Shift {
    cells: Vec<f64>,
}

impl Shift {
    pub fn new() -> Self {
        Self {
            cells: vec![0.0; 4],
        }
    }
}

impl Default for Shift {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for Shift {
    /// inputs: [0] = data input, [1] = trigger (shift on rising edge), [2] = reset
    /// params: [0] = register length (default 4)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let trigger = inputs.get(1).copied().unwrap_or(0.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.get(1).copied().unwrap_or(0.0);
        let length = (params.first().copied().unwrap_or(4.0) as usize).clamp(1, 32);

        // Resize if length changed
        if self.cells.len() != length {
            self.cells.resize(length, 0.0);
        }

        if is_high(reset) {
            self.cells.fill(0.0);
        } else if !is_high(prev_trigger) && is_high(trigger) {
            // Shift right: drop last, insert at front
            self.cells.pop();
            self.cells.insert(0, input);
        }

        self.cells.clone()
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&self.cells))
    }

    fn restore(&mut self, state: &[u8]) {
        let count = state.len() / 8;
        if count > 0 {
            if let Some(values) = deserialize_f64s(state, count) {
                self.cells = values;
            }
        }
    }

    fn block_type(&self) -> &str {
        "Shift"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Status value: outputs an integer code representing the current state of its
/// digital inputs. Each input is a named state; the output is the index (1-based)
/// of the highest-priority active input, or 0 if none are active.
///
/// Outputs: [0] = state index (0 = none, 1 = first input active, etc.)
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Priority is index-based — lower index wins when multiple inputs
// are active simultaneously.
#[derive(Clone, Copy)]
pub struct StateV;

impl Block for StateV {
    /// inputs: variable-length digital inputs
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let active_index = inputs.iter().position(|&v| is_high(v));
        let value = active_index.map(|i| (i + 1) as f64).unwrap_or(0.0);
        vec![value]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "StateV"
    }
}

/// State machine: selects among analog inputs based on the active digital state.
/// Rising edges on state-change inputs switch the current state.
///
/// inputs: [0] = reset, [1..N] = state trigger inputs (rising edge selects state)
/// params: [0] = number of states (default 2)
/// Outputs: [0] = current state index (0-based)
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: The Loxone "State" block outputs the index of the currently
// selected state. Lowest-index rising edge wins on simultaneous transitions.
#[derive(Clone)]
pub struct State {
    current: usize,
}

impl State {
    pub fn new() -> Self {
        Self { current: 0 }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for State {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let reset = inputs.first().copied().unwrap_or(0.0);
        let num_states = (params.first().copied().unwrap_or(2.0) as usize).max(1);

        if is_high(reset) {
            self.current = 0;
        } else {
            for i in 1..inputs.len() {
                let cur = inputs.get(i).copied().unwrap_or(0.0);
                let prev = prev_inputs.get(i).copied().unwrap_or(0.0);
                if !is_high(prev) && is_high(cur) {
                    // 1-indexed: I1 → state 1, I2 → state 2, etc.
                    self.current = ((i - 1) % num_states) + 1;
                    break;
                }
            }
        }

        vec![self.current as f64]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.current as f64]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.current = values[0] as usize;
        }
    }

    fn block_type(&self) -> &str {
        "State"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amemory_stores_value_on_trigger_edge() {
        let mut block = AMemory::new();
        assert_eq!(
            block.eval(&[7.0, 1.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]),
            vec![7.0]
        );
        assert_eq!(
            block.eval(&[9.0, 1.0, 0.0], &[], 0.0, &[7.0, 1.0, 0.0]),
            vec![7.0]
        );
        assert_eq!(
            block.eval(&[9.0, 0.0, 1.0], &[], 0.0, &[9.0, 1.0, 0.0]),
            vec![0.0]
        );
    }

    #[test]
    fn flipflop_set_dominates_reset() {
        let mut block = FlipFlop::new();
        assert_eq!(block.eval(&[1.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0, 1.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[0.0, 1.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn rs_flipflop_reset_dominates_set() {
        let mut block = RSFlipFlop::new();
        assert_eq!(block.eval(&[1.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        // Both active: reset dominates
        assert_eq!(block.eval(&[1.0, 1.0], &[], 0.0, &[]), vec![0.0]);
        assert_eq!(block.eval(&[0.0, 0.0], &[], 0.0, &[]), vec![0.0]);
        // Set again
        assert_eq!(block.eval(&[1.0, 0.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[0.0, 1.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn memory_captures_digital_input() {
        let mut block = Memory::new();
        // Trigger rising edge with input high
        assert_eq!(
            block.eval(&[1.0, 1.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]),
            vec![1.0]
        );
        // Holds even when input drops
        assert_eq!(
            block.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[1.0, 1.0, 0.0]),
            vec![1.0]
        );
        // Reset clears
        assert_eq!(
            block.eval(&[0.0, 0.0, 1.0], &[], 0.0, &[0.0, 0.0, 0.0]),
            vec![0.0]
        );
    }

    #[test]
    fn pushbutton_toggles_and_emits_transition_pulses() {
        let mut block = PushButton::new();
        assert_eq!(
            block.eval(&[1.0, 0.0], &[], 0.0, &[0.0, 0.0]),
            vec![1.0, 0.0, 1.0]
        );
        assert_eq!(
            block.eval(&[0.0, 0.0], &[], 0.0, &[1.0, 0.0]),
            vec![1.0, 0.0, 0.0]
        );
        assert_eq!(
            block.eval(&[1.0, 0.0], &[], 0.0, &[0.0, 0.0]),
            vec![0.0, 1.0, 0.0]
        );
    }

    #[test]
    fn pushbutton2_detects_double_click() {
        let mut block = PushButton2::new();
        // First click toggles on
        let out = block.eval(&[1.0], &[0.5], 0.0, &[0.0]);
        assert_eq!(out[0], 1.0); // toggled on
        assert_eq!(out[3], 0.0); // no double click yet
                                 // Second click within window → double click
        let out = block.eval(&[0.0], &[0.5], 0.1, &[1.0]);
        assert_eq!(out[3], 0.0);
        let out = block.eval(&[1.0], &[0.5], 0.1, &[0.0]);
        assert_eq!(out[3], 1.0); // double click detected
                                 // State should still be on (second click didn't toggle)
        assert_eq!(out[0], 1.0);
    }

    #[test]
    fn pushbutton2_no_double_click_after_window() {
        let mut block = PushButton2::new();
        block.eval(&[1.0], &[0.5], 0.0, &[0.0]);
        block.eval(&[0.0], &[0.5], 0.6, &[1.0]); // wait past window
        let out = block.eval(&[1.0], &[0.5], 0.1, &[0.0]);
        assert_eq!(out[3], 0.0); // no double click
        assert_eq!(out[0], 0.0); // toggled off normally
    }

    #[test]
    fn counter_counts_rising_edges() {
        let mut block = Counter::new();
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[1.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[], 0.0, &[1.0]), vec![1.0]);
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[0.0]), vec![2.0]);
    }

    #[test]
    fn up_down_counter_increments_and_decrements() {
        let mut block = UpDownCounter::new();
        // Trigger rising edge, InputDir=0 (up) → count=1
        let out = block.eval(&[1.0, 0.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out[1], 1.0); // AQ = count
        assert_eq!(out[0], 0.0); // Q=0 (1 < OnValue=3)

        // Release trigger
        block.eval(&[0.0, 0.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[1.0, 0.0, 0.0]);
        // Trigger again → count=2
        let out = block.eval(&[1.0, 0.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out[1], 2.0);

        // Release, trigger again → count=3, Q turns ON (3 >= OnValue=3)
        block.eval(&[0.0, 0.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[1.0, 0.0, 0.0]);
        let out = block.eval(&[1.0, 0.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out[1], 3.0);
        assert_eq!(out[0], 1.0); // Q=1

        // Count down: InputDir=1, trigger → count=2, Q stays ON (hysteresis: 2 > OffValue=1)
        block.eval(&[0.0, 1.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[1.0, 0.0, 0.0]);
        let out = block.eval(&[1.0, 1.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[0.0, 1.0, 0.0]);
        assert_eq!(out[1], 2.0);
        assert_eq!(out[0], 1.0); // Q still ON (hysteresis)

        // Count down again → count=1, Q turns OFF (1 <= OffValue=1)
        block.eval(&[0.0, 1.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[1.0, 1.0, 0.0]);
        let out = block.eval(&[1.0, 1.0, 0.0], &[0.0, 3.0, 1.0], 0.0, &[0.0, 1.0, 0.0]);
        assert_eq!(out[1], 1.0);
        assert_eq!(out[0], 0.0); // Q=0

        // Reset sets AQ=StartValue=0, Q=0
        let out = block.eval(&[0.0, 0.0, 1.0], &[0.0, 3.0, 1.0], 0.0, &[1.0, 1.0, 0.0]);
        assert_eq!(out[1], 0.0); // AQ = StartValue
        assert_eq!(out[0], 0.0); // Q = 0
    }

    #[test]
    fn up_down_counter_hysteresis() {
        let mut block = UpDownCounter::new();
        // Params: StartValue=0, OnValue=10, OffValue=5 (defaults)
        // Count up to 10 → Q turns ON
        for i in 0..10 {
            let prev = if i == 0 { 0.0 } else { 1.0 };
            block.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[prev, 0.0, 0.0]);
            block.eval(&[1.0, 0.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]);
        }
        let out = block.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[1.0, 0.0, 0.0]);
        assert_eq!(out[1], 10.0); // AQ=10
        assert_eq!(out[0], 1.0); // Q=1 (10 >= OnValue=10)

        // Count down to 6 → Q stays ON (6 > OffValue=5)
        for _ in 0..4 {
            block.eval(&[0.0, 1.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]);
            block.eval(&[1.0, 1.0, 0.0], &[], 0.0, &[0.0, 1.0, 0.0]);
        }
        let out = block.eval(&[0.0, 1.0, 0.0], &[], 0.0, &[1.0, 1.0, 0.0]);
        assert_eq!(out[1], 6.0);
        assert_eq!(out[0], 1.0); // Q still ON

        // Count down to 5 → Q turns OFF (5 <= OffValue=5)
        block.eval(&[1.0, 1.0, 0.0], &[], 0.0, &[0.0, 1.0, 0.0]);
        let out = block.eval(&[0.0, 1.0, 0.0], &[], 0.0, &[1.0, 1.0, 0.0]);
        assert_eq!(out[1], 5.0);
        assert_eq!(out[0], 0.0); // Q OFF
    }

    #[test]
    fn hour_counter_accumulates_time() {
        let mut block = HourCounter::new();
        // Active for 3600 seconds = 1 hour
        for _ in 0..3600 {
            block.eval(&[1.0, 0.0], &[], 1.0, &[]);
        }
        let out = block.eval(&[1.0, 0.0], &[], 0.0, &[]);
        assert!((out[0] - 1.0).abs() < 0.001);
        assert_eq!(out[1], 1.0); // currently active

        // Inactive: no accumulation
        let out = block.eval(&[0.0, 0.0], &[], 100.0, &[]);
        assert!((out[0] - 1.0).abs() < 0.001);
        assert_eq!(out[1], 0.0);
    }

    #[test]
    fn hour_counter_resets() {
        let mut block = HourCounter::new();
        block.eval(&[1.0, 0.0], &[], 7200.0, &[]);
        let out = block.eval(&[0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn sample_hold_captures_on_trigger() {
        let mut block = SampleHold::new();
        // No trigger: output stays at 0
        assert_eq!(
            block.eval(&[42.0, 0.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]),
            vec![0.0]
        );
        // Trigger edge: capture 42.0
        assert_eq!(
            block.eval(&[42.0, 1.0, 0.0], &[], 0.0, &[42.0, 0.0, 0.0]),
            vec![42.0]
        );
        // Holds value even when input changes
        assert_eq!(
            block.eval(&[99.0, 0.0, 0.0], &[], 0.0, &[42.0, 1.0, 0.0]),
            vec![42.0]
        );
        // Reset clears
        assert_eq!(
            block.eval(&[99.0, 0.0, 1.0], &[], 0.0, &[99.0, 0.0, 0.0]),
            vec![0.0]
        );
    }

    #[test]
    fn shift_register_shifts_on_trigger() {
        let mut block = Shift::new();
        // Shift in 10.0
        let out = block.eval(&[10.0, 1.0, 0.0], &[4.0], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out, vec![10.0, 0.0, 0.0, 0.0]);
        // Shift in 20.0
        let out = block.eval(&[20.0, 1.0, 0.0], &[4.0], 0.0, &[10.0, 0.0, 0.0]);
        assert_eq!(out, vec![20.0, 10.0, 0.0, 0.0]);
        // Shift in 30.0
        let out = block.eval(&[30.0, 1.0, 0.0], &[4.0], 0.0, &[20.0, 0.0, 0.0]);
        assert_eq!(out, vec![30.0, 20.0, 10.0, 0.0]);
        // Shift in 40.0
        let out = block.eval(&[40.0, 1.0, 0.0], &[4.0], 0.0, &[30.0, 0.0, 0.0]);
        assert_eq!(out, vec![40.0, 30.0, 20.0, 10.0]);
        // Shift in 50.0 — 10.0 falls off
        let out = block.eval(&[50.0, 1.0, 0.0], &[4.0], 0.0, &[40.0, 0.0, 0.0]);
        assert_eq!(out, vec![50.0, 40.0, 30.0, 20.0]);
    }

    #[test]
    fn shift_register_reset() {
        let mut block = Shift::new();
        block.eval(&[10.0, 1.0, 0.0], &[4.0], 0.0, &[0.0, 0.0, 0.0]);
        let out = block.eval(&[0.0, 0.0, 1.0], &[4.0], 0.0, &[10.0, 1.0, 0.0]);
        assert_eq!(out, vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn statev_outputs_first_active_input() {
        let mut block = StateV;
        assert_eq!(block.eval(&[0.0, 0.0, 1.0], &[], 0.0, &[]), vec![3.0]);
        assert_eq!(block.eval(&[1.0, 0.0, 1.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn state_selects_on_rising_edge() {
        let mut block = State::new();
        // Select state 2 via second state trigger (inputs[2] = I2)
        let out = block.eval(&[0.0, 0.0, 1.0], &[3.0], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out, vec![2.0]);
        // Stays in state 2 without new edges
        let out = block.eval(&[0.0, 0.0, 0.0], &[3.0], 0.0, &[0.0, 0.0, 1.0]);
        assert_eq!(out, vec![2.0]);
        // Select state 1 via first state trigger (inputs[1] = I1)
        let out = block.eval(&[0.0, 1.0, 0.0], &[3.0], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out, vec![1.0]);
        // Reset
        let out = block.eval(&[1.0, 0.0, 0.0], &[3.0], 0.0, &[0.0, 1.0, 0.0]);
        assert_eq!(out, vec![0.0]);
    }

    // --- State/Restore tests ---

    #[test]
    fn rs_flipflop_state_roundtrip() {
        let mut b = RSFlipFlop::new();
        b.eval(&[1.0, 0.0], &[], 0.0, &[]);
        let state = b.state().unwrap();
        let mut b2 = RSFlipFlop::new();
        b2.restore(&state);
        assert_eq!(b2.eval(&[0.0, 0.0], &[], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn up_down_counter_state_roundtrip() {
        let mut b = UpDownCounter::new();
        // Count up twice: Trigger edges with InputDir=0 (up)
        b.eval(&[1.0, 0.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]);
        b.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[1.0, 0.0, 0.0]);
        b.eval(&[1.0, 0.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]);
        let state = b.state().unwrap();
        let mut b2 = UpDownCounter::new();
        b2.restore(&state);
        // After restore, AQ (out[1]) should still be 2.0
        assert_eq!(
            b2.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[1.0, 0.0, 0.0])[1],
            2.0
        );
    }

    #[test]
    fn shift_state_roundtrip() {
        let mut b = Shift::new();
        b.eval(&[10.0, 1.0, 0.0], &[4.0], 0.0, &[0.0, 0.0, 0.0]);
        b.eval(&[20.0, 1.0, 0.0], &[4.0], 0.0, &[10.0, 0.0, 0.0]);
        let state = b.state().unwrap();
        let mut b2 = Shift::new();
        b2.restore(&state);
        let out = b2.eval(&[30.0, 1.0, 0.0], &[4.0], 0.0, &[20.0, 0.0, 0.0]);
        assert_eq!(out, vec![30.0, 20.0, 10.0, 0.0]);
    }

    #[test]
    fn memory_state_roundtrip() {
        let mut b = Memory::new();
        b.eval(&[1.0, 1.0, 0.0], &[], 0.0, &[0.0, 0.0, 0.0]);
        let state = b.state().unwrap();
        let mut b2 = Memory::new();
        b2.restore(&state);
        assert_eq!(
            b2.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[1.0, 1.0, 0.0]),
            vec![1.0]
        );
    }
}
