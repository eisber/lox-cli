//! Group D — Security blocks (alarm, presence, access control).
//!
//! Simplified simulation models for Loxone security-related blocks.

use crate::blocks::{
    bool_signal, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

// ---------------------------------------------------------------------------
// Alarm
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: I[0] = arm/disarm (digital), I[1..] = zone triggers.
// Armed + any zone trigger → alarm active. Validate against Miniserver.

/// Alarm block: armed/disarmed state with zone monitoring.
#[derive(Clone, Default)]
pub struct Alarm {
    armed: bool,
    triggered: bool,
}

impl Alarm {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Alarm {
    /// Inputs: [arm, zone1, zone2, ...]
    /// Outputs: [armed, triggered, alarm_active]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let arm_input = inputs.first().copied().unwrap_or(0.0);
        self.armed = is_high(arm_input);

        let any_zone = inputs.iter().skip(1).any(|&v| is_high(v));
        if self.armed && any_zone {
            self.triggered = true;
        }
        if !self.armed {
            self.triggered = false;
        }

        let active = self.armed && self.triggered;
        vec![
            bool_signal(self.armed),
            bool_signal(self.triggered),
            bool_signal(active),
        ]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut out = serialize_bool(self.armed);
        out.extend(serialize_bool(self.triggered));
        Some(out)
    }

    fn restore(&mut self, state: &[u8]) {
        if state.len() >= 2 {
            self.armed = state[0] != 0;
            self.triggered = state[1] != 0;
        }
    }

    fn block_type(&self) -> &str {
        "Alarm"
    }
}

// ---------------------------------------------------------------------------
// CentralAlarm
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: ORs all zone inputs, outputs [any_triggered, count_active].

/// Consolidates multiple alarm zones into a single status.
#[derive(Clone, Copy)]
pub struct CentralAlarm;

impl Block for CentralAlarm {
    /// Inputs: [zone1, zone2, ...]
    /// Outputs: [any_triggered, active_zone_count]
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
        "CentralAlarm"
    }
}

// ---------------------------------------------------------------------------
// AlarmChain
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Sequential alarm — output escalates from zone 0 upward.
// First triggered zone index is output. Validate against Miniserver.

/// Sequential alarm handling — outputs first active zone index.
#[derive(Clone, Copy)]
pub struct AlarmChain;

impl Block for AlarmChain {
    /// Inputs: [zone1, zone2, ...]
    /// Outputs: [first_active_index (1-based, 0 = none), any_active]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let first = inputs
            .iter()
            .position(|&v| is_high(v))
            .map(|i| (i + 1) as f64)
            .unwrap_or(0.0);
        vec![first, bool_signal(first > 0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "AlarmChain"
    }
}

// ---------------------------------------------------------------------------
// SmokeAlarm
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: ORs all smoke detector inputs. Validate against Miniserver.

/// Smoke detection consolidation — any detector triggers alarm.
#[derive(Clone, Copy)]
pub struct SmokeAlarm;

impl Block for SmokeAlarm {
    /// Inputs: [detector1, detector2, ...]
    /// Outputs: [alarm_active, active_count]
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
        "SmokeAlarm"
    }
}

// ---------------------------------------------------------------------------
// Presence
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Motion input resets a countdown timer. While timer > 0, occupied = true.
// params[0] = timeout in seconds (default 300). Validate against Miniserver.

/// Occupancy block with timeout timer.
#[derive(Clone, Default)]
pub struct Presence {
    timer: f64,
}

impl Presence {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for Presence {
    /// Inputs: [motion_trigger]
    /// Params: [timeout_seconds (default 300)]
    /// Outputs: [occupied, timer_remaining]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let motion = inputs.first().copied().unwrap_or(0.0);
        let prev_motion = prev_inputs.first().copied().unwrap_or(0.0);
        let timeout = params.first().copied().unwrap_or(300.0).max(0.0);

        // Rising edge on motion resets timer
        if !is_high(prev_motion) && is_high(motion) {
            self.timer = timeout;
        }

        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }

        vec![bool_signal(self.timer > 0.0), self.timer]
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
        "Presence"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// PresenceController
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Like Presence but with manual override input.
// I[0] = motion, I[1] = manual override. Validate against Miniserver.

/// Presence controller with manual override capability.
#[derive(Clone, Default)]
pub struct PresenceController {
    timer: f64,
}

impl PresenceController {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Block for PresenceController {
    /// Inputs: [motion, manual_override]
    /// Params: [timeout_seconds (default 300)]
    /// Outputs: [occupied, timer_remaining]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let motion = inputs.first().copied().unwrap_or(0.0);
        let manual = inputs.get(1).copied().unwrap_or(0.0);
        let prev_motion = prev_inputs.first().copied().unwrap_or(0.0);
        let timeout = params.first().copied().unwrap_or(300.0).max(0.0);

        if is_high(manual) || (!is_high(prev_motion) && is_high(motion)) {
            self.timer = timeout;
        }

        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }

        vec![bool_signal(self.timer > 0.0), self.timer]
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
        "PresenceController"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// PresenceDetector
// ---------------------------------------------------------------------------

/// Presence detector sensor — source-only pass-through.
#[derive(Clone, Copy)]
pub struct PresenceDetector;

impl Block for PresenceDetector {
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
        "PresenceDetector"
    }
}

// ---------------------------------------------------------------------------
// Access
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: I[0] = code/credential input, params[0] = valid code.
// Outputs granted = 1.0 when input matches code. Validate against Miniserver.

/// Access control block — grants access when input matches configured code.
#[derive(Clone, Copy)]
pub struct Access;

impl Block for Access {
    /// Inputs: [code_input]
    /// Params: [valid_code]
    /// Outputs: [granted]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let code = inputs.first().copied().unwrap_or(0.0);
        let valid = params.first().copied().unwrap_or(0.0);
        vec![bool_signal(
            (code - valid).abs() <= f64::EPSILON && code != 0.0,
        )]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Access"
    }
}

// ---------------------------------------------------------------------------
// AalEmergency
// ---------------------------------------------------------------------------

// WARNING: Block type 'AalEmergency' behavior unknown — using pass-through stub.
// Actual Loxone behavior not documented. Will be validated later.

/// AAL emergency alert — pass-through stub.
#[derive(Clone, Copy)]
pub struct AalEmergency;

impl Block for AalEmergency {
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
        "AalEmergency"
    }
}

// ---------------------------------------------------------------------------
// AalSmartAlarm
// ---------------------------------------------------------------------------

// WARNING: Block type 'AalSmartAlarm' behavior unknown — using pass-through stub.
// Actual Loxone behavior not documented. Will be validated later.

/// AAL smart alarm — pass-through stub.
#[derive(Clone, Copy)]
pub struct AalSmartAlarm;

impl Block for AalSmartAlarm {
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
        "AalSmartAlarm"
    }
}

// ---------------------------------------------------------------------------
// JoinWindowSensor
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: ORs all window sensor inputs. Validate against Miniserver.

/// Window open/close detection — any sensor open → output high.
#[derive(Clone, Copy)]
pub struct JoinWindowSensor;

impl Block for JoinWindowSensor {
    /// Inputs: [sensor1, sensor2, ...]
    /// Outputs: [any_open, open_count]
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
        "JoinWindowSensor"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarm_arm_and_trigger() {
        let mut alarm = Alarm::new();
        // Not armed, zone active → no alarm
        let out = alarm.eval(&[0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[0], 0.0); // not armed
        assert_eq!(out[2], 0.0); // not active

        // Arm it
        let out = alarm.eval(&[1.0, 0.0], &[], 0.0, &[]);
        assert_eq!(out[0], 1.0); // armed
        assert_eq!(out[1], 0.0); // not triggered

        // Trigger zone
        let out = alarm.eval(&[1.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[1], 1.0); // triggered
        assert_eq!(out[2], 1.0); // active

        // Disarm clears trigger
        let out = alarm.eval(&[0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[1], 0.0); // cleared
        assert_eq!(out[2], 0.0); // inactive
    }

    #[test]
    fn alarm_state_roundtrip() {
        let mut alarm = Alarm::new();
        alarm.eval(&[1.0, 1.0], &[], 0.0, &[]);
        let state = alarm.state().unwrap();
        let mut restored = Alarm::new();
        restored.restore(&state);
        assert_eq!(restored.armed, alarm.armed);
        assert_eq!(restored.triggered, alarm.triggered);
    }

    #[test]
    fn central_alarm_consolidates_zones() {
        let mut block = CentralAlarm;
        let out = block.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[]);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 0.0);

        let out = block.eval(&[1.0, 0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        assert_eq!(out[1], 2.0);
    }

    #[test]
    fn alarm_chain_first_active() {
        let mut block = AlarmChain;
        let out = block.eval(&[0.0, 0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[0], 3.0); // third zone (1-based)
        assert_eq!(out[1], 1.0); // active

        let out = block.eval(&[0.0, 0.0, 0.0], &[], 0.0, &[]);
        assert_eq!(out[0], 0.0); // none
    }

    #[test]
    fn smoke_alarm_any_detector() {
        let mut block = SmokeAlarm;
        assert_eq!(block.eval(&[0.0, 0.0], &[], 0.0, &[])[0], 0.0);
        assert_eq!(block.eval(&[0.0, 1.0], &[], 0.0, &[])[0], 1.0);
    }

    #[test]
    fn presence_timeout() {
        let mut p = Presence::new();
        // Rising edge triggers timer
        let out = p.eval(&[1.0], &[10.0], 0.0, &[0.0]);
        assert_eq!(out[0], 1.0); // occupied
        assert_eq!(out[1], 10.0); // timer

        // Count down
        let out = p.eval(&[0.0], &[10.0], 5.0, &[1.0]);
        assert_eq!(out[0], 1.0); // still occupied
        assert_eq!(out[1], 5.0);

        // Expire
        let out = p.eval(&[0.0], &[10.0], 5.0, &[0.0]);
        assert_eq!(out[0], 0.0); // expired
    }

    #[test]
    fn presence_state_roundtrip() {
        let mut p = Presence::new();
        p.eval(&[1.0], &[10.0], 0.0, &[0.0]);
        let state = p.state().unwrap();
        let mut restored = Presence::new();
        restored.restore(&state);
        assert_eq!(restored.timer, p.timer);
    }

    #[test]
    fn presence_controller_manual_override() {
        let mut pc = PresenceController::new();
        // Manual override keeps it occupied
        let out = pc.eval(&[0.0, 1.0], &[10.0], 0.0, &[0.0, 0.0]);
        assert_eq!(out[0], 1.0); // occupied via manual
    }

    #[test]
    fn access_grants_on_matching_code() {
        let mut block = Access;
        assert_eq!(block.eval(&[1234.0], &[1234.0], 0.0, &[])[0], 1.0);
        assert_eq!(block.eval(&[1234.0], &[5678.0], 0.0, &[])[0], 0.0);
        assert_eq!(block.eval(&[0.0], &[0.0], 0.0, &[])[0], 0.0); // zero code → deny
    }

    #[test]
    fn join_window_sensor_detects_open() {
        let mut block = JoinWindowSensor;
        let out = block.eval(&[0.0, 0.0, 1.0], &[], 0.0, &[]);
        assert_eq!(out[0], 1.0); // any open
        assert_eq!(out[1], 1.0); // one open
    }

    #[test]
    fn factory_creates_all_security_types() {
        use crate::blocks::create_block;
        for name in &[
            "Alarm",
            "CentralAlarm",
            "AlarmChain",
            "SmokeAlarm",
            "Presence",
            "PresenceController",
            "PresenceDetector",
            "Access",
            "AalEmergency",
            "AalSmartAlarm",
            "JoinWindowSensor",
        ] {
            let block = create_block(name);
            assert_eq!(block.block_type(), *name, "Factory mismatch for {name}");
        }
    }
}
