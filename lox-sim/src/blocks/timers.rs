use crate::blocks::{
    bool_signal, deserialize_bool, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

/// One-shot timer: a rising edge keeps Q high for Time seconds.
#[derive(Clone)]
pub struct Monoflop {
    remaining: f64,
}

impl Monoflop {
    pub fn new() -> Self {
        Self { remaining: 0.0 }
    }
}

impl Default for Monoflop {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for Monoflop {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let duration = params.first().copied().unwrap_or(1.0).max(0.0);

        if !is_high(prev_trigger) && is_high(trigger) {
            self.remaining = duration.max(dt);
        }

        let q = self.remaining > 0.0;
        if q {
            self.remaining = (self.remaining - dt).max(0.0);
        }
        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.remaining]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.remaining = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "Monoflop"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Delayed pulse: rising edge starts a delay, then emits a pulse for Time seconds.
#[derive(Clone)]
pub struct OnPulseDelay {
    delay_remaining: f64,
    pulse_remaining: f64,
}

impl OnPulseDelay {
    pub fn new() -> Self {
        Self {
            delay_remaining: 0.0,
            pulse_remaining: 0.0,
        }
    }
}

impl Default for OnPulseDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for OnPulseDelay {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let delay = params.first().copied().unwrap_or(0.0).max(0.0);
        let duration = params.get(1).copied().unwrap_or(0.5).max(0.0);

        if !is_high(prev_trigger) && is_high(trigger) {
            if delay <= 0.0 {
                // No delay — start pulse immediately
                self.pulse_remaining = duration.max(dt);
                self.delay_remaining = 0.0;
            } else {
                self.delay_remaining = delay;
                self.pulse_remaining = 0.0;
            }
        }

        let mut q = false;
        if self.pulse_remaining > 0.0 {
            q = true;
            self.pulse_remaining = (self.pulse_remaining - dt).max(0.0);
        } else if self.delay_remaining > 0.0 {
            self.delay_remaining = (self.delay_remaining - dt).max(0.0);
            if self.delay_remaining <= 0.0 {
                self.pulse_remaining = duration.max(dt);
                q = self.pulse_remaining > 0.0;
                if q {
                    self.pulse_remaining = (self.pulse_remaining - dt).max(0.0);
                }
            }
        }

        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[
            self.delay_remaining,
            self.pulse_remaining,
        ]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 2) {
            self.delay_remaining = values[0];
            self.pulse_remaining = values[1];
        }
    }

    fn block_type(&self) -> &str {
        "OnPulseDelay"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Switch-off delay: output remains high for Time seconds after the trigger falls.
#[derive(Clone)]
pub struct OffDelay {
    remaining: f64,
}

impl OffDelay {
    pub fn new() -> Self {
        Self { remaining: 0.0 }
    }
}

impl Default for OffDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for OffDelay {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let delay = params.first().copied().unwrap_or(1.0).max(0.0);

        let q = if is_high(trigger) {
            self.remaining = 0.0;
            true
        } else {
            if is_high(prev_trigger) && !is_high(trigger) {
                self.remaining = delay.max(dt);
            }
            let active = self.remaining > 0.0;
            if active {
                self.remaining = (self.remaining - dt).max(0.0);
            }
            active
        };

        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.remaining]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.remaining = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "OffDelay"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Stairwell timer: manual On forces output high, otherwise trigger starts a timed on-period.
#[derive(Clone)]
pub struct StairwayLS {
    remaining: f64,
}

impl StairwayLS {
    pub fn new() -> Self {
        Self { remaining: 0.0 }
    }
}

impl Default for StairwayLS {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for StairwayLS {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let forced_on = inputs.get(1).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let time_high = params.first().copied().unwrap_or(180.0).max(0.0);

        if !is_high(prev_trigger) && is_high(trigger) {
            self.remaining = time_high.max(dt);
        }

        let q = if is_high(forced_on) {
            true
        } else {
            let active = self.remaining > 0.0;
            if active {
                self.remaining = (self.remaining - dt).max(0.0);
            }
            active
        };

        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.remaining]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 1) {
            self.remaining = values[0];
        }
    }

    fn block_type(&self) -> &str {
        "StairwayLS"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Square-wave oscillator.
#[derive(Clone)]
pub struct PulseGen {
    high_phase: bool,
    phase_remaining: f64,
}

impl PulseGen {
    pub fn new() -> Self {
        Self {
            high_phase: true,
            phase_remaining: 0.0,
        }
    }
}

impl Default for PulseGen {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for PulseGen {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let enabled = is_high(inputs.first().copied().unwrap_or(0.0));
        let inverted = is_high(inputs.get(1).copied().unwrap_or(0.0));
        let time_high = params.first().copied().unwrap_or(1.0).max(0.0);
        let time_low = params.get(1).copied().unwrap_or(1.0).max(0.0);

        if !enabled {
            self.high_phase = true;
            self.phase_remaining = 0.0;
            return vec![0.0];
        }

        if self.phase_remaining <= 0.0 {
            self.high_phase = true;
            self.phase_remaining = time_high.max(dt);
        }

        let q = if inverted {
            !self.high_phase
        } else {
            self.high_phase
        };

        self.phase_remaining -= dt;
        while self.phase_remaining <= 0.0 {
            self.high_phase = !self.high_phase;
            let next_duration = if self.high_phase { time_high } else { time_low }.max(dt);
            self.phase_remaining += next_duration;
        }

        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut state = serialize_bool(self.high_phase);
        state.extend_from_slice(&serialize_f64s(&[self.phase_remaining]));
        Some(state)
    }

    fn restore(&mut self, state: &[u8]) {
        self.high_phase = deserialize_bool(state).unwrap_or(true);
        if state.len() > 1 {
            if let Some(values) = deserialize_f64s(&state[1..], 1) {
                self.phase_remaining = values[0];
            }
        }
    }

    fn block_type(&self) -> &str {
        "PulseGen"
    }
}

/// Edge detector with configurable pulse width.
#[derive(Clone)]
pub struct EdgeDetection {
    edge_remaining: f64,
    rising_remaining: f64,
    falling_remaining: f64,
}

impl EdgeDetection {
    pub fn new() -> Self {
        Self {
            edge_remaining: 0.0,
            rising_remaining: 0.0,
            falling_remaining: 0.0,
        }
    }
}

impl Default for EdgeDetection {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for EdgeDetection {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let current = inputs.first().copied().unwrap_or(0.0);
        let previous = prev_inputs.first().copied().unwrap_or(0.0);
        let pulse_time = params.first().copied().unwrap_or(1.0).max(dt);

        if !is_high(previous) && is_high(current) {
            self.rising_remaining = pulse_time;
            self.edge_remaining = pulse_time;
        }
        if is_high(previous) && !is_high(current) {
            self.falling_remaining = pulse_time;
            self.edge_remaining = pulse_time;
        }

        let edge = self.edge_remaining > 0.0;
        let rising = self.rising_remaining > 0.0;
        let falling = self.falling_remaining > 0.0;

        if edge {
            self.edge_remaining = (self.edge_remaining - dt).max(0.0);
        }
        if rising {
            self.rising_remaining = (self.rising_remaining - dt).max(0.0);
        }
        if falling {
            self.falling_remaining = (self.falling_remaining - dt).max(0.0);
        }

        vec![bool_signal(edge), bool_signal(rising), bool_signal(falling)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[
            self.edge_remaining,
            self.rising_remaining,
            self.falling_remaining,
        ]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 3) {
            self.edge_remaining = values[0];
            self.rising_remaining = values[1];
            self.falling_remaining = values[2];
        }
    }

    fn block_type(&self) -> &str {
        "EdgeDetection"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Switch-on delay: output goes high only after input stays high for Time seconds.
/// If the input drops before Time, the timer resets.
#[derive(Clone)]
pub struct OnDelay {
    elapsed: f64,
    output: bool,
}

impl OnDelay {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            output: false,
        }
    }
}

impl Default for OnDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for OnDelay {
    /// inputs: [0] = trigger
    /// params: [0] = delay time (seconds)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let delay = params.first().copied().unwrap_or(1.0).max(0.0);

        if is_high(trigger) {
            if !self.output {
                self.elapsed += dt;
                if self.elapsed >= delay {
                    self.output = true;
                }
            }
        } else {
            self.elapsed = 0.0;
            self.output = false;
        }

        vec![bool_signal(self.output)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.output);
        s.extend_from_slice(&serialize_f64s(&[self.elapsed]));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.output = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 1) {
                self.elapsed = v[0];
            }
        }
    }

    fn block_type(&self) -> &str {
        "OnDelay"
    }
}

/// On+Off delay: output goes high after input stays high for OnTime, and stays
/// high for OffTime after the input drops.
#[derive(Clone)]
pub struct OnOffDelay {
    on_elapsed: f64,
    off_remaining: f64,
    output: bool,
}

impl OnOffDelay {
    pub fn new() -> Self {
        Self {
            on_elapsed: 0.0,
            off_remaining: 0.0,
            output: false,
        }
    }
}

impl Default for OnOffDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for OnOffDelay {
    /// inputs: [0] = trigger
    /// params: [0] = on-delay (seconds), [1] = off-delay (seconds)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let on_delay = params.first().copied().unwrap_or(1.0).max(0.0);
        let off_delay = params.get(1).copied().unwrap_or(1.0).max(0.0);

        if is_high(trigger) {
            self.off_remaining = 0.0;
            if !self.output {
                self.on_elapsed += dt;
                if self.on_elapsed >= on_delay {
                    self.output = true;
                }
            }
        } else {
            self.on_elapsed = 0.0;
            if self.output {
                if self.off_remaining <= 0.0 {
                    self.off_remaining = off_delay;
                }
                self.off_remaining = (self.off_remaining - dt).max(0.0);
                if self.off_remaining <= 0.0 {
                    self.output = false;
                }
            }
        }

        vec![bool_signal(self.output)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.output);
        s.extend_from_slice(&serialize_f64s(&[self.on_elapsed, self.off_remaining]));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.output = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 2) {
                self.on_elapsed = v[0];
                self.off_remaining = v[1];
            }
        }
    }

    fn block_type(&self) -> &str {
        "OnOffDelay"
    }
}

/// Retentive on-delay: like OnDelay but the elapsed time is NOT reset when the
/// input drops — it accumulates across multiple on-periods. A separate Reset
/// input clears the accumulated time.
#[derive(Clone)]
pub struct RetOnDelay {
    elapsed: f64,
    output: bool,
}

impl RetOnDelay {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            output: false,
        }
    }
}

impl Default for RetOnDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for RetOnDelay {
    /// inputs: [0] = trigger, [1] = reset
    /// params: [0] = delay time (seconds)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let reset = inputs.get(1).copied().unwrap_or(0.0);
        let delay = params.first().copied().unwrap_or(1.0).max(0.0);

        if is_high(reset) {
            self.elapsed = 0.0;
            self.output = false;
        } else if is_high(trigger) {
            self.elapsed += dt;
            if self.elapsed >= delay {
                self.output = true;
            }
        }
        // When trigger is low and not reset, elapsed is retained (retentive).

        vec![bool_signal(self.output)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.output);
        s.extend_from_slice(&serialize_f64s(&[self.elapsed]));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.output = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 1) {
                self.elapsed = v[0];
            }
        }
    }

    fn block_type(&self) -> &str {
        "RetOnDelay"
    }
}

/// Pulse-width modulator: converts an analog input (0.0–1.0 duty cycle) into
/// a digital on/off signal over a configurable period.
#[derive(Clone)]
pub struct PWM {
    phase: f64,
}

impl PWM {
    pub fn new() -> Self {
        Self { phase: 0.0 }
    }
}

impl Default for PWM {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for PWM {
    /// inputs: [0] = duty cycle (0.0–1.0)
    /// params: [0] = period (seconds, default 1.0)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let duty = inputs.first().copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let period = params.first().copied().unwrap_or(1.0).max(f64::EPSILON);

        let on = self.phase < duty * period;
        self.phase = (self.phase + dt) % period;
        vec![bool_signal(on)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.phase]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.phase = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "PWM"
    }
}

/// Edge-wiping relay / toggle with auto-off and warning flash.
///
/// Each rising edge toggles the output. While on, a configurable auto-off timer
/// runs. When `warning_time` seconds remain, the warning output pulses to
/// indicate imminent shutoff. A dedicated Off input forces the output low.
///
/// Outputs: [0] = Q (on/off), [1] = QWarning (flashing before auto-off)
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Warning output flashes (toggles each tick) during the warning phase.
// The exact flash pattern on real hardware is unknown.
#[derive(Clone)]
pub struct EdgeWipingRelay {
    is_on: bool,
    remaining: f64,
    warning_flash: bool,
}

impl EdgeWipingRelay {
    pub fn new() -> Self {
        Self {
            is_on: false,
            remaining: 0.0,
            warning_flash: false,
        }
    }
}

impl Default for EdgeWipingRelay {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for EdgeWipingRelay {
    /// inputs: [0] = trigger (toggle on rising edge), [1] = off (force off)
    /// params: [0] = auto-off time (seconds, 0 = no auto-off),
    ///         [1] = warning time before auto-off (seconds, default 0)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let off_input = inputs.get(1).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let auto_off_time = params.first().copied().unwrap_or(0.0).max(0.0);
        let warning_time = params.get(1).copied().unwrap_or(0.0).max(0.0);

        if is_high(off_input) {
            self.is_on = false;
            self.remaining = 0.0;
            self.warning_flash = false;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            self.is_on = !self.is_on;
            if self.is_on && auto_off_time > 0.0 {
                self.remaining = auto_off_time;
            } else if !self.is_on {
                self.remaining = 0.0;
            }
            self.warning_flash = false;
        }

        let q = self.is_on;
        let mut q_warning = false;
        if self.is_on && self.remaining > 0.0 {
            if self.remaining <= warning_time {
                // WARNING: Assumed behavior — not validated against Miniserver.
                // Assumption: Warning flashes by toggling each eval tick.
                self.warning_flash = !self.warning_flash;
                q_warning = self.warning_flash;
            }
            self.remaining = (self.remaining - dt).max(0.0);
            if self.remaining <= 0.0 {
                self.is_on = false;
                self.warning_flash = false;
            }
        }

        vec![bool_signal(q), bool_signal(q_warning)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.is_on);
        s.extend_from_slice(&serialize_f64s(&[self.remaining]));
        s.push(u8::from(self.warning_flash));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.is_on = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 1) {
                self.remaining = v[0];
            }
        }
        if state.len() > 9 {
            self.warning_flash = state[9] != 0;
        }
    }

    fn block_type(&self) -> &str {
        "EdgeWipingRelay"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Generic switching timer: rising edge turns output on for a configurable
/// duration, with an optional retrigger capability and an Off input.
///
/// Unlike Monoflop, this has an explicit Off input and retrigger behavior:
/// a new rising edge while active restarts the timer from the full duration.
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Retrigger always resets the full timer. Off input has priority.
#[derive(Clone)]
pub struct SwitchingTimer {
    remaining: f64,
}

impl SwitchingTimer {
    pub fn new() -> Self {
        Self { remaining: 0.0 }
    }
}

impl Default for SwitchingTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for SwitchingTimer {
    /// inputs: [0] = trigger (start/retrigger on rising edge), [1] = off (force off)
    /// params: [0] = duration (seconds)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let trigger = inputs.first().copied().unwrap_or(0.0);
        let off_input = inputs.get(1).copied().unwrap_or(0.0);
        let prev_trigger = prev_inputs.first().copied().unwrap_or(0.0);
        let duration = params.first().copied().unwrap_or(1.0).max(0.0);

        if is_high(off_input) {
            self.remaining = 0.0;
        } else if !is_high(prev_trigger) && is_high(trigger) {
            self.remaining = duration.max(dt);
        }

        let q = self.remaining >= dt && self.remaining > 0.0;
        if q {
            self.remaining = (self.remaining - dt).max(0.0);
        }

        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.remaining]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.remaining = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "SwitchingTimer"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monoflop_holds_output_for_duration() {
        let mut block = Monoflop::new();
        assert_eq!(block.eval(&[1.0], &[1.0], 0.25, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[1.0], 0.25, &[1.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[1.0], 0.25, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[1.0], 0.25, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[1.0], 0.25, &[0.0]), vec![0.0]);
    }

    #[test]
    fn on_pulse_delay_emits_delayed_pulse() {
        let mut block = OnPulseDelay::new();
        assert_eq!(block.eval(&[1.0], &[0.5, 0.5], 0.25, &[0.0]), vec![0.0]);
        assert_eq!(block.eval(&[0.0], &[0.5, 0.5], 0.25, &[1.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[0.5, 0.5], 0.25, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[0.5, 0.5], 0.25, &[0.0]), vec![0.0]);
    }

    #[test]
    fn off_delay_keeps_output_high_after_falling_edge() {
        let mut block = OffDelay::new();
        assert_eq!(block.eval(&[1.0], &[0.5], 0.25, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[0.5], 0.25, &[1.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[0.5], 0.25, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[0.5], 0.25, &[0.0]), vec![0.0]);
    }

    #[test]
    fn stairway_manual_on_overrides_timer() {
        let mut block = StairwayLS::new();
        assert_eq!(block.eval(&[0.0, 1.0], &[1.0], 0.5, &[0.0, 0.0]), vec![1.0]);
    }

    #[test]
    fn pulsegen_oscillates_when_enabled() {
        let mut block = PulseGen::new();
        assert_eq!(block.eval(&[1.0, 0.0], &[0.5, 0.5], 0.25, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0, 0.0], &[0.5, 0.5], 0.25, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0, 0.0], &[0.5, 0.5], 0.25, &[]), vec![0.0]);
    }

    #[test]
    fn edge_detection_emits_pulses_for_each_edge() {
        let mut block = EdgeDetection::new();
        assert_eq!(
            block.eval(&[1.0], &[0.5], 0.25, &[0.0]),
            vec![1.0, 1.0, 0.0]
        );
        assert_eq!(
            block.eval(&[1.0], &[0.5], 0.25, &[1.0]),
            vec![1.0, 1.0, 0.0]
        );
        assert_eq!(
            block.eval(&[0.0], &[0.5], 0.25, &[1.0]),
            vec![1.0, 0.0, 1.0]
        );
    }

    // --- OnDelay ---

    #[test]
    fn on_delay_waits_before_turning_on() {
        let mut b = OnDelay::new();
        // Input goes high, delay = 1.0s
        assert_eq!(b.eval(&[1.0], &[1.0], 0.25, &[0.0]), vec![0.0]);
        assert_eq!(b.eval(&[1.0], &[1.0], 0.25, &[1.0]), vec![0.0]);
        assert_eq!(b.eval(&[1.0], &[1.0], 0.25, &[1.0]), vec![0.0]);
        assert_eq!(b.eval(&[1.0], &[1.0], 0.25, &[1.0]), vec![1.0]); // 1.0s elapsed
    }

    #[test]
    fn on_delay_resets_when_input_drops() {
        let mut b = OnDelay::new();
        assert_eq!(b.eval(&[1.0], &[1.0], 0.4, &[0.0]), vec![0.0]);
        // Input drops before delay expires
        assert_eq!(b.eval(&[0.0], &[1.0], 0.25, &[1.0]), vec![0.0]);
        // Re-trigger: must wait full delay again
        assert_eq!(b.eval(&[1.0], &[1.0], 0.5, &[0.0]), vec![0.0]);
        assert_eq!(b.eval(&[1.0], &[1.0], 0.5, &[1.0]), vec![1.0]);
    }

    #[test]
    fn on_delay_turns_off_immediately_when_input_drops() {
        let mut b = OnDelay::new();
        for _ in 0..4 {
            b.eval(&[1.0], &[1.0], 0.25, &[1.0]);
        }
        assert_eq!(b.eval(&[1.0], &[1.0], 0.01, &[1.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0], &[1.0], 0.01, &[1.0]), vec![0.0]);
    }

    // --- OnOffDelay ---

    #[test]
    fn on_off_delay_delays_both_edges() {
        let mut b = OnOffDelay::new();
        // On-delay = 0.5s, off-delay = 0.5s
        assert_eq!(b.eval(&[1.0], &[0.5, 0.5], 0.25, &[0.0]), vec![0.0]);
        assert_eq!(b.eval(&[1.0], &[0.5, 0.5], 0.25, &[1.0]), vec![1.0]); // on after 0.5s
                                                                          // Now drop input — off-delay starts
        assert_eq!(b.eval(&[0.0], &[0.5, 0.5], 0.25, &[1.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0], &[0.5, 0.5], 0.25, &[0.0]), vec![0.0]); // off after 0.5s
    }

    // --- RetOnDelay ---

    #[test]
    fn ret_on_delay_accumulates_across_pulses() {
        let mut b = RetOnDelay::new();
        // delay = 1.0s, each pulse adds 0.3s
        assert_eq!(b.eval(&[1.0], &[1.0], 0.3, &[0.0]), vec![0.0]);
        assert_eq!(b.eval(&[0.0], &[1.0], 0.3, &[1.0]), vec![0.0]); // off, retains 0.3
        assert_eq!(b.eval(&[1.0], &[1.0], 0.3, &[0.0]), vec![0.0]); // 0.6
        assert_eq!(b.eval(&[0.0], &[1.0], 0.3, &[1.0]), vec![0.0]); // off, retains 0.6
        assert_eq!(b.eval(&[1.0], &[1.0], 0.5, &[0.0]), vec![1.0]); // 1.1 >= 1.0
    }

    #[test]
    fn ret_on_delay_reset_clears() {
        let mut b = RetOnDelay::new();
        b.eval(&[1.0], &[1.0], 0.8, &[0.0]);
        // Reset clears accumulated time
        assert_eq!(b.eval(&[1.0, 1.0], &[1.0], 0.01, &[1.0, 0.0]), vec![0.0]);
        assert_eq!(b.eval(&[1.0, 0.0], &[1.0], 0.5, &[1.0, 1.0]), vec![0.0]);
    }

    // --- PWM ---

    #[test]
    fn pwm_generates_duty_cycle() {
        let mut b = PWM::new();
        // duty=0.5, period=1.0 → on for first 0.5s, off for next 0.5s
        let mut on_count = 0;
        for _ in 0..10 {
            let out = b.eval(&[0.5], &[1.0], 0.1, &[]);
            if out[0] > 0.5 {
                on_count += 1;
            }
        }
        assert_eq!(on_count, 5);
    }

    #[test]
    fn pwm_zero_duty_always_off() {
        let mut b = PWM::new();
        for _ in 0..5 {
            assert_eq!(b.eval(&[0.0], &[1.0], 0.1, &[]), vec![0.0]);
        }
    }

    #[test]
    fn pwm_full_duty_always_on() {
        let mut b = PWM::new();
        for _ in 0..5 {
            assert_eq!(b.eval(&[1.0], &[1.0], 0.1, &[]), vec![1.0]);
        }
    }

    // --- EdgeWipingRelay ---

    #[test]
    fn edge_wiping_relay_toggles() {
        let mut b = EdgeWipingRelay::new();
        let out = b.eval(&[1.0, 0.0], &[0.0, 0.0], 0.1, &[0.0, 0.0]);
        assert_eq!(out[0], 1.0); // on
        let out = b.eval(&[0.0, 0.0], &[0.0, 0.0], 0.1, &[1.0, 0.0]);
        assert_eq!(out[0], 1.0); // stays on
        let out = b.eval(&[1.0, 0.0], &[0.0, 0.0], 0.1, &[0.0, 0.0]);
        assert_eq!(out[0], 0.0); // toggled off
    }

    #[test]
    fn edge_wiping_relay_auto_off() {
        let mut b = EdgeWipingRelay::new();
        // Toggle on, auto-off = 0.5s
        b.eval(&[1.0, 0.0], &[0.5, 0.0], 0.1, &[0.0, 0.0]);
        assert_eq!(b.eval(&[0.0, 0.0], &[0.5, 0.0], 0.2, &[1.0, 0.0])[0], 1.0);
        assert_eq!(b.eval(&[0.0, 0.0], &[0.5, 0.0], 0.2, &[0.0, 0.0])[0], 1.0);
        // After 0.5s total => auto-off
        assert_eq!(b.eval(&[0.0, 0.0], &[0.5, 0.0], 0.1, &[0.0, 0.0])[0], 0.0);
    }

    #[test]
    fn edge_wiping_relay_off_input() {
        let mut b = EdgeWipingRelay::new();
        b.eval(&[1.0, 0.0], &[5.0, 0.0], 0.1, &[0.0, 0.0]);
        assert_eq!(b.eval(&[0.0, 1.0], &[5.0, 0.0], 0.1, &[1.0, 0.0])[0], 0.0);
    }

    // --- SwitchingTimer ---

    #[test]
    fn switching_timer_on_for_duration() {
        let mut b = SwitchingTimer::new();
        assert_eq!(b.eval(&[1.0, 0.0], &[1.0], 0.25, &[0.0, 0.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0, 0.0], &[1.0], 0.25, &[1.0, 0.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0, 0.0], &[1.0], 0.25, &[0.0, 0.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0, 0.0], &[1.0], 0.25, &[0.0, 0.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0, 0.0], &[1.0], 0.25, &[0.0, 0.0]), vec![0.0]);
    }

    #[test]
    fn switching_timer_retrigger_resets() {
        let mut b = SwitchingTimer::new();
        b.eval(&[1.0, 0.0], &[1.0], 0.5, &[0.0, 0.0]);
        // Retrigger at 0.5s → timer resets to 1.0
        b.eval(&[0.0, 0.0], &[1.0], 0.25, &[1.0, 0.0]);
        b.eval(&[1.0, 0.0], &[1.0], 0.25, &[0.0, 0.0]);
        // Should still be on for ~1.0s from retrigger
        assert_eq!(b.eval(&[0.0, 0.0], &[1.0], 0.5, &[1.0, 0.0]), vec![1.0]);
        assert_eq!(b.eval(&[0.0, 0.0], &[1.0], 0.5, &[0.0, 0.0]), vec![0.0]);
    }

    #[test]
    fn switching_timer_off_input() {
        let mut b = SwitchingTimer::new();
        b.eval(&[1.0, 0.0], &[5.0], 0.1, &[0.0, 0.0]);
        assert_eq!(b.eval(&[0.0, 1.0], &[5.0], 0.1, &[1.0, 0.0]), vec![0.0]);
    }

    // --- State/Restore tests ---

    #[test]
    fn on_delay_state_roundtrip() {
        let mut b = OnDelay::new();
        b.eval(&[1.0], &[2.0], 0.5, &[0.0]);
        let state = b.state().unwrap();
        let mut b2 = OnDelay::new();
        b2.restore(&state);
        assert_eq!(
            b2.eval(&[1.0], &[2.0], 0.5, &[1.0]),
            b.eval(&[1.0], &[2.0], 0.5, &[1.0])
        );
    }

    #[test]
    fn pwm_state_roundtrip() {
        let mut b = PWM::new();
        b.eval(&[0.5], &[1.0], 0.3, &[]);
        let state = b.state().unwrap();
        let mut b2 = PWM::new();
        b2.restore(&state);
        assert_eq!(
            b2.eval(&[0.5], &[1.0], 0.1, &[]),
            b.eval(&[0.5], &[1.0], 0.1, &[])
        );
    }
}
