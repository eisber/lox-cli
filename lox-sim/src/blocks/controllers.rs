//! Group C — Controller blocks (simplified simulation models).
//!
//! These blocks model complex Loxone controllers (lighting, blinds, HVAC, PID, etc.)
//! with simplified logic suitable for simulation. Each block carries a WARNING comment
//! describing assumptions and deviations from real Miniserver behaviour.

use crate::blocks::{
    bool_signal, deserialize_bool, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

fn normalize_percent(value: Signal) -> Signal {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// LightController2
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Supports direct scene selection through I1 or one-hot Sel1-Sel8
// inputs. Presence and Move edges refresh a timeout. AQ1/AQ2 mirror the active
// brightness while Scene reports the selected mood. This intentionally omits
// RGB/temperature channels and advanced alarm-clock behaviour.
// Validate against Miniserver.

/// Advanced lighting controller with scene selection, brightness, and presence timeout.
#[derive(Clone, Default)]
pub struct LightController2 {
    active_scene: f64,
    brightness: f64,
    presence_timer: f64,
}

impl LightController2 {
    pub fn new() -> Self {
        Self {
            active_scene: 0.0,
            brightness: 0.0,
            presence_timer: 0.0,
        }
    }
}

impl Block for LightController2 {
    /// Inputs: [I1, Presence, Brightness, Move, Sel1..Sel8, Reset, InputDisable]
    /// Params: [presence_timeout]
    /// Outputs: [AQ1, AQ2, Scene, PresenceActive]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let direct_scene = inputs.first().copied().unwrap_or(0.0);
        let presence = inputs.get(1).copied().unwrap_or(0.0);
        let brightness_in = inputs.get(2).copied().unwrap_or(1.0);
        let move_trigger = inputs.get(3).copied().unwrap_or(0.0);
        let timeout = params.first().copied().unwrap_or(300.0).max(0.0);
        let sel_start = 4usize;

        let mut scene = direct_scene.round().max(0.0);
        for idx in 0..8usize {
            if is_high(inputs.get(sel_start + idx).copied().unwrap_or(0.0)) {
                scene = (idx + 1) as f64;
                break;
            }
        }

        let reset = inputs.get(sel_start + 8).copied().unwrap_or(0.0);
        let disabled = inputs.get(sel_start + 9).copied().unwrap_or(0.0);
        let prev_presence = prev_inputs.get(1).copied().unwrap_or(0.0);
        let prev_move = prev_inputs.get(3).copied().unwrap_or(0.0);
        let prev_reset = prev_inputs.get(sel_start + 8).copied().unwrap_or(0.0);

        if !is_high(prev_reset) && is_high(reset) {
            self.active_scene = 0.0;
            self.brightness = 0.0;
            self.presence_timer = 0.0;
            return vec![0.0, 0.0, 0.0, 0.0];
        }

        if is_high(disabled) {
            self.presence_timer = 0.0;
            return vec![0.0, 0.0, self.active_scene, 0.0];
        }

        if scene > 0.0 && (scene - self.active_scene).abs() > f64::EPSILON {
            self.active_scene = scene;
            self.presence_timer = timeout;
        }

        if !is_high(prev_presence) && is_high(presence) {
            self.presence_timer = timeout;
        }
        if !is_high(prev_move) && is_high(move_trigger) {
            self.presence_timer = timeout;
        }

        if self.presence_timer > 0.0 {
            self.presence_timer = (self.presence_timer - dt).max(0.0);
        }

        self.brightness = normalize_percent(brightness_in);

        let presence_active = bool_signal(self.presence_timer > 0.0);
        let effective_brightness = if self.presence_timer > 0.0 {
            self.brightness
        } else {
            0.0
        };

        vec![
            effective_brightness,
            effective_brightness,
            self.active_scene,
            presence_active,
        ]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[
            self.active_scene,
            self.brightness,
            self.presence_timer,
        ]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 3) {
            self.active_scene = v[0];
            self.brightness = v[1];
            self.presence_timer = v[2];
        }
    }

    fn block_type(&self) -> &str {
        "LightController2"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// LightController
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Single scene input + brightness. No mood outputs.
// Validate against Miniserver.

/// Simple lighting controller — single scene, brightness, presence.
#[derive(Clone, Default)]
pub struct LightController {
    active_scene: f64,
    brightness: f64,
    presence_timer: f64,
}

impl LightController {
    pub fn new() -> Self {
        Self {
            active_scene: 0.0,
            brightness: 0.0,
            presence_timer: 0.0,
        }
    }
}

impl Block for LightController {
    /// Inputs: [scene_select, brightness, presence_trigger]
    /// Params: [presence_timeout]
    /// Outputs: [active_scene, brightness_out, presence_active]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let scene = inputs.first().copied().unwrap_or(0.0);
        let brightness_in = inputs.get(1).copied().unwrap_or(1.0);
        let presence = inputs.get(2).copied().unwrap_or(0.0);
        let timeout = params.first().copied().unwrap_or(300.0).max(0.0);

        let prev_scene = prev_inputs.first().copied().unwrap_or(0.0);
        if scene != prev_scene && scene > 0.0 {
            self.active_scene = scene;
            self.presence_timer = timeout;
        }

        let prev_presence = prev_inputs.get(2).copied().unwrap_or(0.0);
        if !is_high(prev_presence) && is_high(presence) {
            self.presence_timer = timeout;
        }

        if self.presence_timer > 0.0 {
            self.presence_timer = (self.presence_timer - dt).max(0.0);
        }

        self.brightness = brightness_in.clamp(0.0, 1.0);

        let effective = if self.presence_timer > 0.0 {
            self.brightness
        } else {
            0.0
        };
        vec![
            self.active_scene,
            effective,
            bool_signal(self.presence_timer > 0.0),
        ]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[
            self.active_scene,
            self.brightness,
            self.presence_timer,
        ]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 3) {
            self.active_scene = v[0];
            self.brightness = v[1];
            self.presence_timer = v[2];
        }
    }

    fn block_type(&self) -> &str {
        "LightController"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// JalousieUpDown2
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Position tracks linearly from 0.0 (open) to 1.0 (closed) based on
// travel time (TimeEnd param). TriggerUp/Down move to the end stops, InputPos
// requests an intermediate target, and InputDisable freezes motion.
// Validate against Miniserver.

/// Blind motor with position tracking.
#[derive(Clone, Default)]
pub struct JalousieUpDown2 {
    position: f64,
    direction: f64, // -1.0 = up, 0.0 = stopped, 1.0 = down
    target: f64,
}

impl JalousieUpDown2 {
    pub fn new() -> Self {
        Self {
            position: 0.0,
            direction: 0.0,
            target: 0.0,
        }
    }
}

impl Block for JalousieUpDown2 {
    /// Inputs: [InputTriggerUp, InputTriggerDown, InputPos, InputDisable]
    /// Params: [time_end (full travel seconds)]
    /// Outputs: [position, direction, moving]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let up = inputs.first().copied().unwrap_or(0.0);
        let down = inputs.get(1).copied().unwrap_or(0.0);
        let position_cmd = inputs.get(2).copied().unwrap_or(f64::NAN);
        let disable = inputs.get(3).copied().unwrap_or(0.0);
        let time_end = params.first().copied().unwrap_or(60.0).max(0.001);

        let prev_up = prev_inputs.first().copied().unwrap_or(0.0);
        let prev_down = prev_inputs.get(1).copied().unwrap_or(0.0);

        if is_high(disable) {
            self.direction = 0.0;
            self.target = self.position;
        } else {
            if !is_high(prev_up) && is_high(up) {
                self.target = 0.0;
            }
            if !is_high(prev_down) && is_high(down) {
                self.target = 1.0;
            }
            if position_cmd.is_finite() {
                self.target = normalize_percent(position_cmd);
            }
        }

        let error = self.target - self.position;
        if error.abs() <= 1e-6 {
            self.position = self.target;
            self.direction = 0.0;
        } else {
            self.direction = error.signum();
            let step = (dt / time_end).max(0.0);
            if error.abs() <= step {
                self.position = self.target;
                self.direction = 0.0;
            } else {
                self.position = (self.position + self.direction * step).clamp(0.0, 1.0);
            }
        }

        let moving = bool_signal(self.direction != 0.0);
        vec![self.position, self.direction, moving]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[
            self.position,
            self.direction,
            self.target,
        ]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 3) {
            self.position = v[0];
            self.direction = v[1];
            self.target = v[2];
        }
    }

    fn block_type(&self) -> &str {
        "JalousieUpDown2"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Jalousiemotor
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Same as JalousieUpDown2 but returned as "Jalousiemotor" type.
// Validate against Miniserver.

/// Blind motor — alias for JalousieUpDown2 with different type name.
#[derive(Clone, Default)]
pub struct Jalousiemotor {
    inner: JalousieUpDown2,
}

impl Jalousiemotor {
    pub fn new() -> Self {
        Self {
            inner: JalousieUpDown2::new(),
        }
    }
}

impl Block for Jalousiemotor {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        self.inner.eval(inputs, params, dt, prev_inputs)
    }

    fn state(&self) -> Option<Vec<u8>> {
        self.inner.state()
    }

    fn restore(&mut self, state: &[u8]) {
        self.inner.restore(state);
    }

    fn block_type(&self) -> &str {
        "Jalousiemotor"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

macro_rules! jalousie_alias_block {
    ($name:ident, $type_str:expr) => {
        #[derive(Clone, Default)]
        pub struct $name {
            inner: JalousieUpDown2,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    inner: JalousieUpDown2::new(),
                }
            }
        }

        impl Block for $name {
            fn eval(
                &mut self,
                inputs: &[Signal],
                params: &[Signal],
                dt: f64,
                prev_inputs: &[Signal],
            ) -> Vec<Signal> {
                self.inner.eval(inputs, params, dt, prev_inputs)
            }

            fn state(&self) -> Option<Vec<u8>> {
                self.inner.state()
            }

            fn restore(&mut self, state: &[u8]) {
                self.inner.restore(state);
            }

            fn block_type(&self) -> &str {
                $type_str
            }

            fn is_edge_sensitive(&self) -> bool {
                true
            }
        }
    };
}

jalousie_alias_block!(EIBJalousie, "EIBJalousie");
jalousie_alias_block!(Pergola, "Pergola");
jalousie_alias_block!(Skylight, "Skylight");

// ---------------------------------------------------------------------------
// AutoJalousie
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Output blind position = 1.0 (closed) when sun brightness exceeds threshold
// AND azimuth is within a configurable window. Otherwise 0.0 (open).
// Validate against Miniserver.

/// Sun-position shading controller.
#[derive(Clone, Copy)]
pub struct AutoJalousie;

impl Block for AutoJalousie {
    /// Inputs: [sun_azimuth, sun_altitude, brightness]
    /// Params: [azimuth_min, azimuth_max, brightness_threshold]
    /// Outputs: [blind_position]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let azimuth = inputs.first().copied().unwrap_or(0.0);
        let altitude = inputs.get(1).copied().unwrap_or(0.0);
        let brightness = inputs.get(2).copied().unwrap_or(0.0);

        let az_min = params.first().copied().unwrap_or(90.0);
        let az_max = params.get(1).copied().unwrap_or(270.0);
        let br_threshold = params.get(2).copied().unwrap_or(40000.0);

        let in_window =
            azimuth >= az_min && azimuth <= az_max && altitude > 0.0 && brightness > br_threshold;
        vec![bool_signal(in_window)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "AutoJalousie"
    }
}

// ---------------------------------------------------------------------------
// HeatIRoomController2
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone IRoomController2 has additional logic
// for occupancy detection, window contact open timeouts, weather compensation,
// scheduled setpoint profiles, and multi-zone coordination. This implementation
// models the core PI control with heating/cooling dual outputs, three operating
// modes (Comfort/Save/DeepSave), a configurable deadband between heating and
// cooling, and PI anti-windup via back-calculation.
//
// Real HVAC config values observed:
//   Comfort heating: 22.5°C, cooling: 24.5°C, hysteresis: 1.5°C
//   Kp=120 (heating), Kp=60 (cooling), time constants: 3600s
//   Window timeout: 300s (not modeled here)
//   Modes: Comfort(0), Save(-2°C), DeepSave(-4°C)
//
// Validate against Miniserver — especially mode transitions and integrator
// behavior across operating mode changes.

/// Room thermostat with PI control, heating/cooling outputs, and mode switching.
///
/// Modes: 0 = Comfort, 1 = Save (setpoint reduced by `SaveOffset`),
///        2 = DeepSave (reduced by `DeepSaveOffset`).
#[derive(Clone, Default)]
pub struct HeatIRoomController2 {
    integral_heat: f64,
    integral_cool: f64,
}

impl HeatIRoomController2 {
    pub fn new() -> Self {
        Self {
            integral_heat: 0.0,
            integral_cool: 0.0,
        }
    }
}

impl Block for HeatIRoomController2 {
    /// Inputs:  `[Temp, Setpoint, Reset, InputDisable, Mode, CoolingSetpoint]`
    /// Params:  `[Kp_heat, Ki_heat, Kp_cool, Ki_cool, Deadband, SaveOffset, DeepSaveOffset]`
    /// Outputs: `[AQh (0-100%), AQc (0-100%), EffSetpointHeat, EffSetpointCool, ActiveMode]`
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let temp = inputs.first().copied().unwrap_or(20.0);
        let base_setpoint = inputs.get(1).copied().unwrap_or(21.0);
        let reset = inputs.get(2).copied().unwrap_or(0.0);
        let disabled = inputs.get(3).copied().unwrap_or(0.0);
        let mode = inputs.get(4).copied().unwrap_or(0.0) as u8;
        let cooling_sp_input = inputs.get(5).copied();

        let kp_heat = params.first().copied().unwrap_or(10.0);
        let ki_heat = params.get(1).copied().unwrap_or(0.0).max(0.0);
        let kp_cool = params.get(2).copied().unwrap_or(10.0);
        let ki_cool = params.get(3).copied().unwrap_or(0.0).max(0.0);
        let deadband = params.get(4).copied().unwrap_or(1.5).max(0.0);
        let save_offset = params.get(5).copied().unwrap_or(2.0);
        let deep_save_offset = params.get(6).copied().unwrap_or(4.0);

        let prev_reset = prev_inputs.get(2).copied().unwrap_or(0.0);

        // Reset rising edge clears both integrals
        if !is_high(prev_reset) && is_high(reset) {
            self.integral_heat = 0.0;
            self.integral_cool = 0.0;
        }

        // Mode offset: Save and DeepSave reduce both setpoints
        let mode_offset = match mode {
            1 => save_offset,
            2 => deep_save_offset,
            _ => 0.0,
        };

        let eff_heat_sp = base_setpoint - mode_offset;
        let eff_cool_sp = cooling_sp_input
            .unwrap_or(base_setpoint + deadband)
            .max(eff_heat_sp) // cooling setpoint must be >= heating setpoint
            - mode_offset;
        let eff_cool_sp = eff_cool_sp.max(eff_heat_sp);

        // Disabled: zero outputs, clear integrals
        if is_high(disabled) {
            self.integral_heat = 0.0;
            self.integral_cool = 0.0;
            return vec![0.0, 0.0, eff_heat_sp, eff_cool_sp, mode as f64];
        }

        // --- Mutually exclusive heating/cooling with deadband ---
        // Temp below heat setpoint → heat; above cool setpoint → cool; between → neither
        let error_heat = eff_heat_sp - temp;
        let error_cool = temp - eff_cool_sp;
        let (valve_heat, valve_cool);

        if temp < eff_heat_sp {
            // HEATING zone: freeze cooling integrator
            self.integral_cool = 0.0;
            if dt > 0.0 {
                self.integral_heat += error_heat * dt;
                self.integral_heat = self.integral_heat.max(0.0);
            }
            let raw_heat = error_heat * kp_heat + self.integral_heat * ki_heat;
            valve_heat = raw_heat.clamp(0.0, 100.0);
            if ki_heat > 0.0 && raw_heat > 100.0 {
                self.integral_heat = ((100.0 - error_heat * kp_heat) / ki_heat).max(0.0);
            }
            valve_cool = 0.0;
        } else if temp > eff_cool_sp {
            // COOLING zone: freeze heating integrator
            self.integral_heat = 0.0;
            if dt > 0.0 {
                self.integral_cool += error_cool * dt;
                self.integral_cool = self.integral_cool.max(0.0);
            }
            let raw_cool = error_cool * kp_cool + self.integral_cool * ki_cool;
            valve_cool = raw_cool.clamp(0.0, 100.0);
            if ki_cool > 0.0 && raw_cool > 100.0 {
                self.integral_cool = ((100.0 - error_cool * kp_cool) / ki_cool).max(0.0);
            }
            valve_heat = 0.0;
        } else {
            // DEADBAND zone: neither heating nor cooling, decay integrals
            self.integral_heat = (self.integral_heat * 0.95).max(0.0);
            self.integral_cool = (self.integral_cool * 0.95).max(0.0);
            valve_heat = 0.0;
            valve_cool = 0.0;
        }

        vec![
            valve_heat,
            valve_cool,
            eff_heat_sp,
            eff_cool_sp,
            mode as f64,
        ]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.integral_heat, self.integral_cool]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 2) {
            self.integral_heat = v[0];
            self.integral_cool = v[1];
        } else if let Some(v) = deserialize_f64s(state, 1) {
            // Backward compat: old state format with single integral
            self.integral_heat = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "HeatIRoomController2"
    }
}

// ---------------------------------------------------------------------------
// AcControl
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Toggle flips on/off state. Cooling demand = max(0, temp - setpoint).
// Validate against Miniserver.

/// Air conditioning control.
#[derive(Clone, Default)]
pub struct AcControl {
    on: bool,
}

impl AcControl {
    pub fn new() -> Self {
        Self { on: false }
    }
}

impl Block for AcControl {
    /// Inputs: [toggle, inTempCurr, setpoint]
    /// Params: []
    /// Outputs: [status, cooling_demand]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let toggle = inputs.first().copied().unwrap_or(0.0);
        let temp = inputs.get(1).copied().unwrap_or(22.0);
        let setpoint = inputs.get(2).copied().unwrap_or(24.0);

        let prev_toggle = prev_inputs.first().copied().unwrap_or(0.0);
        if !is_high(prev_toggle) && is_high(toggle) {
            self.on = !self.on;
        }

        let demand = if self.on {
            (temp - setpoint).max(0.0)
        } else {
            0.0
        };

        vec![bool_signal(self.on), demand]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.on))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_bool(state) {
            self.on = v;
        }
    }

    fn block_type(&self) -> &str {
        "AcControl"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Fan speed control family
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Input maps to discrete speed level 0..max_levels. Input is rounded
// to the nearest integer level. Output = level / max_levels (normalised 0-1).
// Validate against Miniserver.

/// Generic fan speed controller used by Ventilation, Fan, Fancoil, etc.
#[derive(Clone, Copy)]
struct FanSpeedCore;

impl FanSpeedCore {
    fn eval_fan(inputs: &[Signal], params: &[Signal]) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let max_levels = params.first().copied().unwrap_or(3.0).max(1.0);
        let level = input.round().clamp(0.0, max_levels);
        vec![level, level / max_levels]
    }
}

macro_rules! fan_block {
    ($name:ident, $type_str:expr) => {
        #[derive(Clone, Copy)]
        pub struct $name;

        impl Block for $name {
            /// Inputs: [speed_input]
            /// Params: [max_levels]
            /// Outputs: [level, normalised_speed]
            fn eval(
                &mut self,
                inputs: &[Signal],
                params: &[Signal],
                _dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                FanSpeedCore::eval_fan(inputs, params)
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

fan_block!(Ventilation, "Ventilation");
fan_block!(Ventilation2, "Ventilation2");
fan_block!(VentInternorm, "VentInternorm");
fan_block!(ToiletFan, "ToiletFan");
fan_block!(Fan, "Fan");
fan_block!(Fancoil, "Fancoil");
fan_block!(FancoilFreshAir, "FancoilFreshAir");

// ---------------------------------------------------------------------------
// IRoomcontrol / IRcontroller
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Pass-through for simulation — forwards first input to output.
// Validate against Miniserver.

/// IR remote control — pass-through for simulation.
#[derive(Clone, Copy)]
pub struct IRoomcontrol;

impl Block for IRoomcontrol {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _p: &[Signal],
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
        "IRoomcontrol"
    }
}

/// IR controller — pass-through for simulation.
#[derive(Clone, Copy)]
pub struct IRcontroller;

impl Block for IRcontroller {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _p: &[Signal],
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
        "IRcontroller"
    }
}

// ---------------------------------------------------------------------------
// DaylightController
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Linear mapping of brightness to blind position.
// blind_pos = clamp((brightness - min) / (max - min), 0, 1).
// Validate against Miniserver.

/// Maps ambient brightness to blind position.
#[derive(Clone, Copy)]
pub struct DaylightController;

impl Block for DaylightController {
    /// Inputs: [brightness]
    /// Params: [min_brightness, max_brightness]
    /// Outputs: [blind_position (0=open, 1=closed)]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let brightness = inputs.first().copied().unwrap_or(0.0);
        let min_br = params.first().copied().unwrap_or(5000.0);
        let max_br = params.get(1).copied().unwrap_or(60000.0);
        let range = (max_br - min_br).max(1.0);
        let pos = ((brightness - min_br) / range).clamp(0.0, 1.0);
        vec![pos]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "DaylightController"
    }
}

// ---------------------------------------------------------------------------
// Heat curve family
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Linear heat curve: supply_temp = base_temp + slope * (setpoint - outdoor_temp).
// Validate against Miniserver.

macro_rules! heat_curve_block {
    ($name:ident, $type_str:expr) => {
        #[derive(Clone, Copy)]
        pub struct $name;

        impl Block for $name {
            /// Inputs: [outdoor_temp, setpoint]
            /// Params: [base_temp, slope]
            /// Outputs: [supply_temp]
            fn eval(
                &mut self,
                inputs: &[Signal],
                params: &[Signal],
                _dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                let outdoor = inputs.first().copied().unwrap_or(5.0);
                let setpoint = inputs.get(1).copied().unwrap_or(21.0);
                let base_temp = params.first().copied().unwrap_or(20.0);
                let slope = params.get(1).copied().unwrap_or(1.5);
                let supply = base_temp + slope * (setpoint - outdoor);
                vec![supply]
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

heat_curve_block!(Heatcurve, "Heatcurve");
heat_curve_block!(Heatmixer, "Heatmixer");

// ---------------------------------------------------------------------------
// Heatmixer2 — 3-way mixing valve
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone Heatmixer2 may include additional
// logic for pump control, frost protection, and flow rate compensation.
// This models only the core 3-way mixing valve position calculation.
//
// Real HVAC config values observed:
//   Supply: 50°C, Setpoint: 40°C, Return: 32°C
//   Position = (setpoint - return) / (supply - return) = (40-32)/(50-32) = 0.44
//
// Validate against Miniserver.

/// 3-way mixing valve controller.
///
/// Computes valve position from supply, return and setpoint temperatures.
/// `position = (setpoint - return) / (supply - return)` clamped to `[0, 1]`.
#[derive(Clone, Copy)]
pub struct Heatmixer2;

impl Block for Heatmixer2 {
    /// Inputs:  `[supply_temp, return_temp, setpoint]`
    /// Params:  `[]`
    /// Outputs: `[valve_position (0-1), flow_temp]`
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let supply = inputs.first().copied().unwrap_or(50.0);
        let return_temp = inputs.get(1).copied().unwrap_or(30.0);
        let setpoint = inputs.get(2).copied().unwrap_or(40.0);

        let range = supply - return_temp;
        let position = if range.abs() < 0.01 {
            // WARNING: supply ≈ return — no mixing possible, default to closed
            0.0
        } else {
            ((setpoint - return_temp) / range).clamp(0.0, 1.0)
        };

        let flow_temp = return_temp + position * range;
        vec![position, flow_temp]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Heatmixer2"
    }
}

// ---------------------------------------------------------------------------
// Solarpumpcontrol
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Pump turns on when collector_temp - tank_temp > delta_on,
// turns off when collector_temp - tank_temp < delta_off (hysteresis).
// Validate against Miniserver.

/// Solar collector pump controller with hysteresis.
#[derive(Clone, Default)]
pub struct Solarpumpcontrol {
    pump_on: bool,
}

impl Solarpumpcontrol {
    pub fn new() -> Self {
        Self { pump_on: false }
    }
}

impl Block for Solarpumpcontrol {
    /// Inputs: [collector_temp, tank_temp]
    /// Params: [delta_on, delta_off]
    /// Outputs: [pump_state]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let collector = inputs.first().copied().unwrap_or(0.0);
        let tank = inputs.get(1).copied().unwrap_or(0.0);
        let delta_on = params.first().copied().unwrap_or(8.0);
        let delta_off = params.get(1).copied().unwrap_or(4.0);

        let diff = collector - tank;
        if diff >= delta_on {
            self.pump_on = true;
        } else if diff <= delta_off {
            self.pump_on = false;
        }

        vec![bool_signal(self.pump_on)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.pump_on))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_bool(state) {
            self.pump_on = v;
        }
    }

    fn block_type(&self) -> &str {
        "Solarpumpcontrol"
    }
}

// ---------------------------------------------------------------------------
// HVACController
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone HVACController coordinates multiple
// heat/cool sources, DHW priority, anti-cycling timers, and cascade logic.
// This implementation models single-source modulation with weather-compensated
// demand scaling and pulse-width modulation for on/off heat sources.
//
// Real HVAC config values observed:
//   Min outdoor: -22°C, Max outdoor: 28°C
//   Pulse on: 750s, Pulse off: 300s (compressor protection)
//
// Validate against Miniserver — especially cascade and DHW priority behavior.

/// Central HVAC heat source controller with PWM output.
#[derive(Clone, Default)]
pub struct HVACController {
    timer: f64,
}

impl HVACController {
    pub fn new() -> Self {
        Self { timer: 0.0 }
    }
}

impl Block for HVACController {
    /// Inputs:  `[demand (0-100), outdoor_temp]`
    /// Params:  `[min_temp, max_temp, pulse_on_time, pulse_off_time]`
    /// Outputs: `[modulation (0-100%), pulse_output (0/1)]`
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let demand = inputs.first().copied().unwrap_or(0.0).clamp(0.0, 100.0);
        let outdoor_temp = inputs.get(1).copied().unwrap_or(10.0);

        let min_temp = params.first().copied().unwrap_or(-22.0);
        let max_temp = params.get(1).copied().unwrap_or(28.0);
        let pulse_on = params.get(2).copied().unwrap_or(750.0).max(1.0);
        let pulse_off = params.get(3).copied().unwrap_or(300.0).max(0.0);

        // Weather compensation: scale demand by how cold it is outside
        let temp_range = (max_temp - min_temp).max(1.0);
        let outdoor_factor = ((max_temp - outdoor_temp) / temp_range).clamp(0.0, 1.0);
        let modulation = (demand * outdoor_factor).clamp(0.0, 100.0);

        // PWM for on/off heat sources — enforce minimum dwell times
        // pulse_on/pulse_off are minimum on/off durations (compressor protection)
        let period = pulse_on + pulse_off;
        // Scale the on-portion but enforce minimum dwell: at least pulse_off off, at least pulse_on on
        let desired_on = period * (modulation / 100.0);
        let on_duration = if desired_on > 0.0 {
            desired_on.max(pulse_on.min(period * 0.5)) // at least min(pulse_on, half period) when on
        } else {
            0.0
        };

        if dt > 0.0 {
            self.timer += dt;
            while self.timer >= period {
                self.timer -= period;
            }
        }

        let pulse = if modulation <= 0.0 {
            false
        } else if modulation >= 100.0 {
            true
        } else {
            self.timer < on_duration
        };

        vec![modulation, bool_signal(pulse)]
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
        "HVACController"
    }
}

// ---------------------------------------------------------------------------
// OvertempShutdown
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone may have configurable lockout times,
// alarm outputs, and manual reset requirements. This implements basic on/off
// protection with hysteresis: trips above threshold, resets below
// (threshold - hysteresis). Validate against Miniserver.

/// Over-temperature protection with hysteresis.
///
/// Trips when temperature exceeds `threshold`, resets when it falls below
/// `threshold - hysteresis`. Output 1.0 = tripped (shutdown active).
#[derive(Clone, Default)]
pub struct OvertempShutdown {
    tripped: bool,
}

impl OvertempShutdown {
    pub fn new() -> Self {
        Self { tripped: false }
    }
}

impl Block for OvertempShutdown {
    /// Inputs:  `[temperature]`
    /// Params:  `[threshold, hysteresis]`
    /// Outputs: `[tripped (0/1), safe (0/1)]`
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let temp = inputs.first().copied().unwrap_or(20.0);
        let threshold = params.first().copied().unwrap_or(90.0);
        let hysteresis = params.get(1).copied().unwrap_or(5.0).max(0.0);

        if temp >= threshold {
            self.tripped = true;
        } else if temp < threshold - hysteresis {
            self.tripped = false;
        }

        vec![bool_signal(self.tripped), bool_signal(!self.tripped)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.tripped))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_bool(state) {
            self.tripped = v;
        }
    }

    fn block_type(&self) -> &str {
        "OvertempShutdown"
    }
}

// ---------------------------------------------------------------------------
// TwoPoint (2Point)
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: On/off controller with hysteresis. Output ON when
// input > setpoint + hysteresis/2, OFF when input < setpoint - hysteresis/2.
// Validate against Miniserver.

/// On/off controller with hysteresis.
#[derive(Clone, Default)]
pub struct TwoPoint {
    output: bool,
}

impl TwoPoint {
    pub fn new() -> Self {
        Self { output: false }
    }
}

impl Block for TwoPoint {
    /// Inputs: [measured_value, setpoint]
    /// Params: [hysteresis]
    /// Outputs: [Q]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let setpoint = inputs.get(1).copied().unwrap_or(0.0);
        let hyst = params.first().copied().unwrap_or(1.0).abs();

        let half = hyst / 2.0;
        if value >= setpoint + half {
            self.output = true;
        } else if value <= setpoint - half {
            self.output = false;
        }

        vec![bool_signal(self.output)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.output))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_bool(state) {
            self.output = v;
        }
    }

    fn block_type(&self) -> &str {
        "2Point"
    }
}

// ---------------------------------------------------------------------------
// ThreePoint (3Point)
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: 3-position controller. Outputs: heating=1 when value < setpoint - deadband,
// cooling=1 when value > setpoint + deadband, else both 0.
// Validate against Miniserver.

/// 3-position controller (heat/off/cool).
#[derive(Clone, Copy)]
pub struct ThreePoint;

impl Block for ThreePoint {
    /// Inputs: [measured_value, setpoint]
    /// Params: [deadband]
    /// Outputs: [heating, cooling]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let setpoint = inputs.get(1).copied().unwrap_or(0.0);
        let deadband = params.first().copied().unwrap_or(1.0).abs();

        let heating = if value < setpoint - deadband {
            1.0
        } else {
            0.0
        };
        let cooling = if value > setpoint + deadband {
            1.0
        } else {
            0.0
        };
        vec![heating, cooling]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "3Point"
    }
}

// ---------------------------------------------------------------------------
// PID
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Standard PID: output = Kp*e + Ki*∫e·dt + Kd*de/dt.
// Integral is clamped to [-1000, 1000] to prevent windup.
// Validate against Miniserver.

/// PID controller.
#[derive(Clone, Default)]
pub struct Pid {
    integral: f64,
    prev_error: f64,
}

impl Pid {
    pub fn new() -> Self {
        Self {
            integral: 0.0,
            prev_error: 0.0,
        }
    }
}

impl Block for Pid {
    /// Inputs: [measured_value, setpoint]
    /// Params: [Kp, Ki, Kd, output_min, output_max]
    /// Outputs: [control_output]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let setpoint = inputs.get(1).copied().unwrap_or(0.0);
        let kp = params.first().copied().unwrap_or(1.0);
        let ki = params.get(1).copied().unwrap_or(0.0);
        let kd = params.get(2).copied().unwrap_or(0.0);
        let out_min = params.get(3).copied().unwrap_or(-100.0);
        let out_max = params.get(4).copied().unwrap_or(100.0);

        let error = setpoint - value;

        if dt > 0.0 {
            self.integral = (self.integral + error * dt).clamp(-1000.0, 1000.0);
        }

        let derivative = if dt > 0.0 {
            (error - self.prev_error) / dt
        } else {
            0.0
        };
        self.prev_error = error;

        let output = (kp * error + ki * self.integral + kd * derivative).clamp(out_min, out_max);
        vec![output]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.integral, self.prev_error]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 2) {
            self.integral = v[0];
            self.prev_error = v[1];
        }
    }

    fn block_type(&self) -> &str {
        "PID"
    }
}

// ---------------------------------------------------------------------------
// PI
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: PI controller (PID without derivative term).
// Validate against Miniserver.

/// PI controller (PID without derivative).
#[derive(Clone, Default)]
pub struct Pi {
    integral: f64,
}

impl Pi {
    pub fn new() -> Self {
        Self { integral: 0.0 }
    }
}

impl Block for Pi {
    /// Inputs: [measured_value, setpoint]
    /// Params: [Kp, Ki, output_min, output_max]
    /// Outputs: [control_output]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let setpoint = inputs.get(1).copied().unwrap_or(0.0);
        let kp = params.first().copied().unwrap_or(1.0);
        let ki = params.get(1).copied().unwrap_or(0.0);
        let out_min = params.get(2).copied().unwrap_or(-100.0);
        let out_max = params.get(3).copied().unwrap_or(100.0);

        let error = setpoint - value;

        if dt > 0.0 {
            self.integral = (self.integral + error * dt).clamp(-1000.0, 1000.0);
        }

        let output = (kp * error + ki * self.integral).clamp(out_min, out_max);
        vec![output]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.integral]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.integral = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "PI"
    }
}

// ---------------------------------------------------------------------------
// PoolController
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Turns on heating when pool_temp < setpoint - hysteresis,
// off when pool_temp >= setpoint. Also outputs pump state based on filter schedule input.
// Validate against Miniserver.

/// Pool temperature controller.
#[derive(Clone, Default)]
pub struct PoolController {
    heating: bool,
}

impl PoolController {
    pub fn new() -> Self {
        Self { heating: false }
    }
}

impl Block for PoolController {
    /// Inputs: [pool_temp, setpoint, filter_pump_input]
    /// Params: [hysteresis]
    /// Outputs: [heating, pump]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let temp = inputs.first().copied().unwrap_or(20.0);
        let setpoint = inputs.get(1).copied().unwrap_or(28.0);
        let pump_in = inputs.get(2).copied().unwrap_or(0.0);
        let hyst = params.first().copied().unwrap_or(1.0).abs();

        if temp < setpoint - hyst {
            self.heating = true;
        } else if temp >= setpoint {
            self.heating = false;
        }

        vec![bool_signal(self.heating), bool_signal(is_high(pump_in))]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_bool(self.heating))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_bool(state) {
            self.heating = v;
        }
    }

    fn block_type(&self) -> &str {
        "PoolController"
    }
}

// ---------------------------------------------------------------------------
// Sauna
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Heater turns on until target temp is reached. Simple on/off with hysteresis.
// Temperature output simulates linear ramp when heating.
// Validate against Miniserver.

/// Sauna temperature profile controller.
#[derive(Clone)]
pub struct Sauna {
    current_temp: f64,
    heater_on: bool,
}

impl Default for Sauna {
    fn default() -> Self {
        Self::new()
    }
}

impl Sauna {
    pub fn new() -> Self {
        Self {
            current_temp: 20.0,
            heater_on: false,
        }
    }
}

impl Block for Sauna {
    /// Inputs: [enable, actual_temp, target_temp]
    /// Params: [heat_rate_per_sec, hysteresis]
    /// Outputs: [heater, current_temp]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let enable = inputs.first().copied().unwrap_or(0.0);
        let actual = inputs.get(1).copied().unwrap_or(self.current_temp);
        let target = inputs.get(2).copied().unwrap_or(80.0);
        let heat_rate = params.first().copied().unwrap_or(0.1);
        let hyst = params.get(1).copied().unwrap_or(2.0).abs();

        self.current_temp = actual;

        if !is_high(enable) {
            self.heater_on = false;
        } else if actual < target - hyst {
            self.heater_on = true;
        } else if actual >= target {
            self.heater_on = false;
        }

        if self.heater_on {
            self.current_temp += heat_rate * dt;
        }

        vec![bool_signal(self.heater_on), self.current_temp]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.heater_on);
        s.extend_from_slice(&serialize_f64s(&[self.current_temp]));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.heater_on = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 1) {
                self.current_temp = v[0];
            }
        }
    }

    fn block_type(&self) -> &str {
        "Sauna"
    }
}

// ---------------------------------------------------------------------------
// SaunaVapor
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Steam generator variant — same structure as Sauna but targets
// lower temperature with steam output. Type name differs.
// Validate against Miniserver.

/// Steam sauna controller.
#[derive(Clone, Default)]
pub struct SaunaVapor {
    inner: Sauna,
}

impl SaunaVapor {
    pub fn new() -> Self {
        Self {
            inner: Sauna::new(),
        }
    }
}

impl Block for SaunaVapor {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        self.inner.eval(inputs, params, dt, prev_inputs)
    }

    fn state(&self) -> Option<Vec<u8>> {
        self.inner.state()
    }

    fn restore(&mut self, state: &[u8]) {
        self.inner.restore(state);
    }

    fn block_type(&self) -> &str {
        "SaunaVapor"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- LightController2 --------------------------------------------------

    #[test]
    fn lc2_scene_selection() {
        let mut lc2 = LightController2::new();
        // I1=1 → scene 1, brightness=80%
        let out = lc2.eval(&[1.0, 0.0, 0.8, 0.0], &[10.0], 0.1, &[0.0; 4]);
        assert!((out[0] - 0.8).abs() < 0.001); // AQ1 brightness
        assert_eq!(out[2], 1.0); // scene
        assert!(out[3] > 0.0); // presence active
    }

    #[test]
    fn lc2_presence_timeout() {
        let mut lc2 = LightController2::new();
        // Trigger scene
        lc2.eval(&[1.0, 0.0, 1.0, 0.0], &[1.0], 0.5, &[0.0; 4]);
        // Advance past timeout
        let out = lc2.eval(&[0.0; 4], &[1.0], 1.5, &[1.0, 0.0, 1.0, 0.0]);
        assert_eq!(out[3], 0.0); // presence expired
        assert_eq!(out[0], 0.0); // brightness zero
    }

    #[test]
    fn lc2_state_roundtrip() {
        let mut lc2 = LightController2::new();
        lc2.eval(&[2.0, 1.0, 0.8, 0.0], &[60.0], 1.0, &[0.0; 4]);

        let state = lc2.state().unwrap();
        let mut lc2b = LightController2::new();
        lc2b.restore(&state);
        assert_eq!(lc2b.active_scene, lc2.active_scene);
        assert_eq!(lc2b.brightness, lc2.brightness);
    }

    // -- LightController ----------------------------------------------------

    #[test]
    fn lc_scene_change() {
        let mut lc = LightController::new();
        let out = lc.eval(&[3.0, 0.9, 0.0], &[10.0], 0.1, &[0.0, 0.9, 0.0]);
        assert_eq!(out[0], 3.0);
        assert!(out[1] > 0.0);
    }

    // -- JalousieUpDown2 ----------------------------------------------------

    #[test]
    fn jalousie_moves_down() {
        let mut j = JalousieUpDown2::new();
        // Trigger down
        j.eval(
            &[0.0, 1.0, f64::NAN, 0.0],
            &[10.0],
            0.0,
            &[0.0, 0.0, 0.0, 0.0],
        );
        // Advance 5s (half travel time)
        let out = j.eval(
            &[0.0, 0.0, f64::NAN, 0.0],
            &[10.0],
            5.0,
            &[0.0, 1.0, 0.0, 0.0],
        );
        assert!((out[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn jalousie_disable_stops() {
        let mut j = JalousieUpDown2::new();
        j.eval(
            &[0.0, 1.0, f64::NAN, 0.0],
            &[10.0],
            0.0,
            &[0.0, 0.0, 0.0, 0.0],
        );
        j.eval(
            &[0.0, 0.0, f64::NAN, 1.0],
            &[10.0],
            2.0,
            &[0.0, 1.0, 0.0, 0.0],
        ); // disable
        let out = j.eval(
            &[0.0, 0.0, f64::NAN, 1.0],
            &[10.0],
            5.0,
            &[0.0, 0.0, 0.0, 1.0],
        );
        // Should not have moved further
        assert!(out[0] < 0.5);
    }

    #[test]
    fn jalousie_state_roundtrip() {
        let mut j = JalousieUpDown2::new();
        j.eval(
            &[0.0, 1.0, f64::NAN, 0.0],
            &[10.0],
            0.0,
            &[0.0, 0.0, 0.0, 0.0],
        );
        j.eval(
            &[0.0, 0.0, f64::NAN, 0.0],
            &[10.0],
            3.0,
            &[0.0, 1.0, 0.0, 0.0],
        );

        let state = j.state().unwrap();
        let mut j2 = JalousieUpDown2::new();
        j2.restore(&state);
        assert_eq!(j2.position, j.position);
    }

    // -- Jalousiemotor ------------------------------------------------------

    #[test]
    fn jalousiemotor_type_name() {
        let j = Jalousiemotor::new();
        assert_eq!(j.block_type(), "Jalousiemotor");
    }

    // -- AutoJalousie -------------------------------------------------------

    #[test]
    fn auto_jalousie_shade() {
        let mut aj = AutoJalousie;
        // Sun in window, bright
        let out = aj.eval(&[180.0, 45.0, 50000.0], &[90.0, 270.0, 40000.0], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        // Sun outside window
        let out = aj.eval(&[30.0, 45.0, 50000.0], &[90.0, 270.0, 40000.0], 0.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    // -- HeatIRoomController2 -----------------------------------------------

    #[test]
    fn heat_room_controller() {
        let mut hrc = HeatIRoomController2::new();
        // temp=18, setpoint=21, Kp=10 → valve = (21-18)*10 = 30
        let out = hrc.eval(&[18.0, 21.0], &[10.0], 0.0, &[]);
        assert!((out[0] - 30.0).abs() < 0.01);
    }

    #[test]
    fn heat_room_controller_clamps() {
        let mut hrc = HeatIRoomController2::new();
        // temp=10, setpoint=21, Kp=20 → (21-10)*20 = 220 → clamped to 100
        let out = hrc.eval(&[10.0, 21.0], &[20.0], 0.0, &[]);
        assert_eq!(out[0], 100.0);
    }

    #[test]
    fn heat_room_controller_reset_and_disable() {
        let mut hrc = HeatIRoomController2::new();
        hrc.eval(&[18.0, 21.0], &[2.0, 1.0], 10.0, &[0.0; 4]);
        let out = hrc.eval(
            &[18.0, 21.0, 1.0, 0.0],
            &[2.0, 1.0],
            0.0,
            &[18.0, 21.0, 0.0, 0.0],
        );
        assert!((out[0] - 6.0).abs() < 0.01);
        let out = hrc.eval(
            &[18.0, 21.0, 0.0, 1.0],
            &[2.0, 1.0],
            1.0,
            &[18.0, 21.0, 1.0, 0.0],
        );
        assert_eq!(out[0], 0.0);
    }

    // -- AcControl ----------------------------------------------------------

    #[test]
    fn ac_toggle_and_demand() {
        let mut ac = AcControl::new();
        // Toggle on
        let out = ac.eval(&[1.0, 26.0, 24.0], &[], 0.0, &[0.0, 0.0, 0.0]);
        assert_eq!(out[0], 1.0); // on
        assert!((out[1] - 2.0).abs() < 0.01); // demand = 26 - 24
    }

    // -- Fan blocks ---------------------------------------------------------

    #[test]
    fn ventilation_speed_levels() {
        let mut v = Ventilation;
        let out = v.eval(&[2.0], &[3.0], 0.0, &[]);
        assert_eq!(out[0], 2.0);
        assert!((out[1] - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn fan_block_clamps() {
        let mut f = Fan;
        let out = f.eval(&[5.0], &[3.0], 0.0, &[]);
        assert_eq!(out[0], 3.0); // clamped to max
    }

    // -- DaylightController -------------------------------------------------

    #[test]
    fn daylight_mapping() {
        let mut dc = DaylightController;
        let out = dc.eval(&[32500.0], &[5000.0, 60000.0], 0.0, &[]);
        assert!((out[0] - 0.5).abs() < 0.01);
    }

    // -- Heat curve ---------------------------------------------------------

    #[test]
    fn heatcurve_output() {
        let mut hc = Heatcurve;
        // outdoor=0, setpoint=21, base=20, slope=1.5 → 20 + 1.5*(21-0) = 51.5
        let out = hc.eval(&[0.0, 21.0], &[20.0, 1.5], 0.0, &[]);
        assert!((out[0] - 51.5).abs() < 0.01);
    }

    // -- Solarpumpcontrol ---------------------------------------------------

    #[test]
    fn solar_pump_hysteresis() {
        let mut sp = Solarpumpcontrol::new();
        // collector=60, tank=50 → diff=10 > delta_on=8 → on
        let out = sp.eval(&[60.0, 50.0], &[8.0, 4.0], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        // collector=54, tank=50 → diff=4 = delta_off → off
        let out = sp.eval(&[54.0, 50.0], &[8.0, 4.0], 0.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    // -- 2Point -------------------------------------------------------------

    #[test]
    fn two_point_hysteresis() {
        let mut tp = TwoPoint::new();
        // value=11, setpoint=10, hyst=2 → 11 > 10+1 → on
        let out = tp.eval(&[11.0, 10.0], &[2.0], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        // value=9.5, still within dead band → stays on
        let out = tp.eval(&[9.5, 10.0], &[2.0], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        // value=8.5 < 10-1 → off
        let out = tp.eval(&[8.5, 10.0], &[2.0], 0.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn two_point_state_roundtrip() {
        let mut tp = TwoPoint::new();
        tp.eval(&[11.0, 10.0], &[2.0], 0.0, &[]);
        let state = tp.state().unwrap();
        let mut tp2 = TwoPoint::new();
        tp2.restore(&state);
        assert_eq!(tp2.output, tp.output);
    }

    // -- 3Point -------------------------------------------------------------

    #[test]
    fn three_point_zones() {
        let mut tp = ThreePoint;
        // Heating zone: value=18, setpoint=21, deadband=1 → 18 < 21-1
        let out = tp.eval(&[18.0, 21.0], &[1.0], 0.0, &[]);
        assert_eq!(out[0], 1.0); // heating
        assert_eq!(out[1], 0.0); // no cooling
                                 // Cooling zone: value=24
        let out = tp.eval(&[24.0, 21.0], &[1.0], 0.0, &[]);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 1.0);
        // Dead band: value=21
        let out = tp.eval(&[21.0, 21.0], &[1.0], 0.0, &[]);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[1], 0.0);
    }

    // -- PID ----------------------------------------------------------------

    #[test]
    fn pid_proportional() {
        let mut pid = Pid::new();
        // value=18, setpoint=21, Kp=2, Ki=0, Kd=0 → 2*3 = 6
        let out = pid.eval(&[18.0, 21.0], &[2.0, 0.0, 0.0], 0.1, &[]);
        assert!((out[0] - 6.0).abs() < 0.01);
    }

    #[test]
    fn pid_integral_accumulates() {
        let mut pid = Pid::new();
        // error=3, Ki=1, dt=1 → integral after 2 steps = 3+3 = 6
        pid.eval(&[18.0, 21.0], &[0.0, 1.0, 0.0], 1.0, &[]);
        let out = pid.eval(&[18.0, 21.0], &[0.0, 1.0, 0.0], 1.0, &[]);
        assert!((out[0] - 6.0).abs() < 0.01);
    }

    #[test]
    fn pid_state_roundtrip() {
        let mut pid = Pid::new();
        pid.eval(&[18.0, 21.0], &[1.0, 0.5, 0.1], 1.0, &[]);
        let state = pid.state().unwrap();
        let mut pid2 = Pid::new();
        pid2.restore(&state);
        assert_eq!(pid2.integral, pid.integral);
        assert_eq!(pid2.prev_error, pid.prev_error);
    }

    // -- PI -----------------------------------------------------------------

    #[test]
    fn pi_control() {
        let mut pi = Pi::new();
        let out = pi.eval(&[18.0, 21.0], &[2.0, 0.5], 1.0, &[]);
        // Kp*3 + Ki*3*1 = 6 + 1.5 = 7.5
        assert!((out[0] - 7.5).abs() < 0.01);
    }

    // -- PoolController -----------------------------------------------------

    #[test]
    fn pool_heating() {
        let mut pc = PoolController::new();
        let out = pc.eval(&[25.0, 28.0, 1.0], &[1.0], 0.0, &[]);
        assert_eq!(out[0], 1.0); // heating on (25 < 28-1)
        assert_eq!(out[1], 1.0); // pump on
    }

    // -- Sauna --------------------------------------------------------------

    #[test]
    fn sauna_heats_up() {
        let mut s = Sauna::new();
        let out = s.eval(&[1.0, 20.0, 80.0], &[10.0, 2.0], 1.0, &[]);
        assert_eq!(out[0], 1.0); // heater on
        assert!(out[1] > 20.0); // temp increased
    }

    #[test]
    fn sauna_turns_off_at_target() {
        let mut s = Sauna::new();
        let out = s.eval(&[1.0, 80.0, 80.0], &[10.0, 2.0], 1.0, &[]);
        assert_eq!(out[0], 0.0); // heater off (at target)
    }

    // -- SaunaVapor ---------------------------------------------------------

    #[test]
    fn sauna_vapor_type_name() {
        let sv = SaunaVapor::new();
        assert_eq!(sv.block_type(), "SaunaVapor");
    }

    // -- Factory registration -----------------------------------------------

    #[test]
    fn controller_block_types() {
        use crate::blocks::create_block;
        let types = [
            "LightController2",
            "LightController",
            "JalousieUpDown2",
            "Jalousiemotor",
            "EIBJalousie",
            "Pergola",
            "RoofWindow",
            "ShadeRoof",
            "Skylight",
            "AutoJalousie",
            "HeatIRoomController2",
            "ClimateControllerUS",
            "HVACController",
            "AcControl",
            "Ventilation",
            "Ventilation2",
            "VentInternorm",
            "ToiletFan",
            "Fan",
            "Fancoil",
            "FancoilFreshAir",
            "IRoomcontrol",
            "IRcontroller",
            "DaylightController",
            "Heatcurve",
            "Heatmixer",
            "Heatmixer2",
            "Solarpumpcontrol",
            "OvertempShutdown",
            "2Point",
            "3Point",
            "PID",
            "PI",
            "PoolController",
            "Sauna",
            "SaunaVapor",
        ];
        for t in &types {
            let b = create_block(t);
            assert_eq!(b.block_type(), *t, "create_block({t}) returned wrong type");
        }
    }

    // ========================================================================
    // HeatIRoomController2 — extended tests
    // ========================================================================

    #[test]
    fn hrc2_cooling_output() {
        let mut hrc = HeatIRoomController2::new();
        // temp=26, setpoint=22, deadband=2 → cool_sp=24, error_cool=26-24=2
        // Kp_cool=10 (param[2]) → valve_cool = 20
        let out = hrc.eval(&[26.0, 22.0], &[10.0, 0.0, 10.0, 0.0, 2.0], 0.0, &[]);
        assert_eq!(out[0], 0.0); // no heating (temp > setpoint)
        assert!((out[1] - 20.0).abs() < 0.01); // cooling = (26-24)*10
    }

    #[test]
    fn hrc2_deadband_no_output() {
        let mut hrc = HeatIRoomController2::new();
        // temp=22, heat_sp=21, cool_sp=21+2=23 → temp in deadband
        let out = hrc.eval(&[22.0, 21.0], &[10.0, 0.0, 10.0, 0.0, 2.0], 0.0, &[]);
        assert_eq!(out[0], 0.0); // no heating
        assert_eq!(out[1], 0.0); // no cooling
    }

    #[test]
    fn hrc2_mode_save() {
        let mut hrc = HeatIRoomController2::new();
        // Mode=1 (Save), save_offset=2 → eff_heat_sp = 22 - 2 = 20
        let out = hrc.eval(
            &[19.0, 22.0, 0.0, 0.0, 1.0],
            &[10.0, 0.0, 10.0, 0.0, 1.5, 2.0, 4.0],
            0.0,
            &[],
        );
        // error_heat = 20 - 19 = 1, valve = 1 * 10 = 10
        assert!((out[0] - 10.0).abs() < 0.01);
        assert!((out[2] - 20.0).abs() < 0.01); // effective heat setpoint
        assert_eq!(out[4], 1.0); // mode
    }

    #[test]
    fn hrc2_mode_deepsave() {
        let mut hrc = HeatIRoomController2::new();
        // Mode=2 (DeepSave), deep_offset=4 → eff_heat_sp = 22 - 4 = 18
        let out = hrc.eval(
            &[17.0, 22.0, 0.0, 0.0, 2.0],
            &[10.0, 0.0, 10.0, 0.0, 1.5, 2.0, 4.0],
            0.0,
            &[],
        );
        // error_heat = 18 - 17 = 1, valve = 10
        assert!((out[0] - 10.0).abs() < 0.01);
        assert!((out[2] - 18.0).abs() < 0.01); // effective heat setpoint
        assert_eq!(out[4], 2.0); // mode
    }

    #[test]
    fn hrc2_anti_windup_heating() {
        let mut hrc = HeatIRoomController2::new();
        // Large error with integral: should not overshoot 100%
        // Kp=50, Ki=1, error=3 → proportional alone = 150 (saturated)
        hrc.eval(&[18.0, 21.0], &[50.0, 1.0], 10.0, &[]);
        let out = hrc.eval(&[18.0, 21.0], &[50.0, 1.0], 10.0, &[]);
        assert_eq!(out[0], 100.0); // clamped at 100
                                   // After reaching setpoint, integral should have been limited
        let out = hrc.eval(&[21.0, 21.0], &[50.0, 1.0], 0.0, &[]);
        // error=0, output = integral * Ki, integral should be <= 100/Ki = 100
        assert!(out[0] <= 100.0);
    }

    #[test]
    fn hrc2_negative_temp() {
        let mut hrc = HeatIRoomController2::new();
        // Outdoor winter: temp=-10, setpoint=20
        let out = hrc.eval(&[-10.0, 20.0], &[2.0], 0.0, &[]);
        // error = 20 - (-10) = 30, valve = 30*2 = 60
        assert!((out[0] - 60.0).abs() < 0.01);
    }

    #[test]
    fn hrc2_state_roundtrip() {
        let mut hrc = HeatIRoomController2::new();
        // Accumulate some heating integral
        hrc.eval(&[18.0, 21.0], &[2.0, 0.5], 5.0, &[]);
        // Accumulate some cooling integral
        hrc.eval(
            &[26.0, 21.0],
            &[2.0, 0.5, 2.0, 0.5, 2.0],
            5.0,
            &[18.0, 21.0],
        );

        let state = hrc.state().unwrap();
        let mut hrc2 = HeatIRoomController2::new();
        hrc2.restore(&state);
        assert_eq!(hrc2.integral_heat, hrc.integral_heat);
        assert_eq!(hrc2.integral_cool, hrc.integral_cool);
    }

    #[test]
    fn hrc2_explicit_cooling_setpoint() {
        let mut hrc = HeatIRoomController2::new();
        // Explicit cooling setpoint = 25 (via input[5])
        let out = hrc.eval(
            &[26.0, 21.0, 0.0, 0.0, 0.0, 25.0],
            &[10.0, 0.0, 5.0],
            0.0,
            &[],
        );
        // error_cool = 26 - 25 = 1, Kp_cool=5 → valve = 5
        assert!((out[1] - 5.0).abs() < 0.01);
        assert!((out[3] - 25.0).abs() < 0.01); // effective cool setpoint
    }

    // ========================================================================
    // Heatmixer2 — mixing valve tests
    // ========================================================================

    #[test]
    fn heatmixer2_basic_position() {
        let mut hm = Heatmixer2;
        // supply=50, return=30, setpoint=40 → pos = (40-30)/(50-30) = 0.5
        let out = hm.eval(&[50.0, 30.0, 40.0], &[], 0.0, &[]);
        assert!((out[0] - 0.5).abs() < 0.01);
        assert!((out[1] - 40.0).abs() < 0.01); // flow = return + 0.5*20 = 40
    }

    #[test]
    fn heatmixer2_real_hvac_values() {
        let mut hm = Heatmixer2;
        // Real config: supply=50, return=32, setpoint=40
        let out = hm.eval(&[50.0, 32.0, 40.0], &[], 0.0, &[]);
        let expected_pos = (40.0 - 32.0) / (50.0 - 32.0); // 0.444...
        assert!((out[0] - expected_pos).abs() < 0.01);
        assert!((out[1] - 40.0).abs() < 0.5);
    }

    #[test]
    fn heatmixer2_fully_open() {
        let mut hm = Heatmixer2;
        // setpoint >= supply → fully open (pos=1)
        let out = hm.eval(&[50.0, 30.0, 55.0], &[], 0.0, &[]);
        assert_eq!(out[0], 1.0);
        assert!((out[1] - 50.0).abs() < 0.01); // flow = supply
    }

    #[test]
    fn heatmixer2_fully_closed() {
        let mut hm = Heatmixer2;
        // setpoint <= return → fully closed (pos=0)
        let out = hm.eval(&[50.0, 30.0, 25.0], &[], 0.0, &[]);
        assert_eq!(out[0], 0.0);
        assert!((out[1] - 30.0).abs() < 0.01); // flow = return
    }

    #[test]
    fn heatmixer2_equal_temps() {
        let mut hm = Heatmixer2;
        // supply == return → no range, position = 0
        let out = hm.eval(&[40.0, 40.0, 40.0], &[], 0.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    // ========================================================================
    // HVACController — heat source modulation tests
    // ========================================================================

    #[test]
    fn hvac_ctrl_full_demand_cold() {
        let mut hc = HVACController::new();
        // demand=100, outdoor=-20, min=-22, max=28
        // outdoor_factor = (28-(-20))/(28-(-22)) = 48/50 = 0.96
        let out = hc.eval(&[100.0, -20.0], &[-22.0, 28.0, 750.0, 300.0], 0.0, &[]);
        assert!((out[0] - 96.0).abs() < 0.1); // modulation ~96%
    }

    #[test]
    fn hvac_ctrl_no_demand() {
        let mut hc = HVACController::new();
        let out = hc.eval(&[0.0, 10.0], &[-22.0, 28.0], 0.0, &[]);
        assert_eq!(out[0], 0.0); // no modulation
        assert_eq!(out[1], 0.0); // pulse off
    }

    #[test]
    fn hvac_ctrl_warm_outdoor() {
        let mut hc = HVACController::new();
        // outdoor >= max_temp → factor=0 → modulation=0
        let out = hc.eval(&[80.0, 30.0], &[-22.0, 28.0], 0.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn hvac_ctrl_pwm_cycles() {
        let mut hc = HVACController::new();
        let params = &[-22.0, 28.0, 750.0, 300.0];
        // demand=50, outdoor at min=-22 → factor=1 → modulation=50
        // period=1050, desired_on=525, but min dwell=525 (min(750, 525))
        // on_duration = max(525, 525) = 525 → off_duration = 1050-525 = 525
        // (At 50% mod, desired_on == min_dwell, so no clamp change)
        // At t=0: timer=0 < 525 → pulse on
        let out = hc.eval(&[50.0, -22.0], params, 0.0, &[]);
        assert!((out[0] - 50.0).abs() < 0.1);
        assert_eq!(out[1], 1.0); // pulse on at start

        // Advance to t=600 → timer=600 > 525 → pulse off
        let out = hc.eval(&[50.0, -22.0], params, 600.0, &[]);
        assert_eq!(out[1], 0.0); // pulse off

        // Advance to t=1100 → timer wraps to 50 < 525 → pulse on
        let out = hc.eval(&[50.0, -22.0], params, 500.0, &[]);
        assert_eq!(out[1], 1.0); // pulse on again
    }

    #[test]
    fn hvac_ctrl_state_roundtrip() {
        let mut hc = HVACController::new();
        hc.eval(&[50.0, -10.0], &[-22.0, 28.0, 750.0, 300.0], 400.0, &[]);
        let state = hc.state().unwrap();
        let mut hc2 = HVACController::new();
        hc2.restore(&state);
        assert!((hc2.timer - hc.timer).abs() < 0.001);
    }

    // ========================================================================
    // OvertempShutdown — protection logic tests
    // ========================================================================

    #[test]
    fn overtemp_trips_above_threshold() {
        let mut ot = OvertempShutdown::new();
        // threshold=90, hysteresis=5
        let out = ot.eval(&[91.0], &[90.0, 5.0], 0.0, &[]);
        assert_eq!(out[0], 1.0); // tripped
        assert_eq!(out[1], 0.0); // not safe
    }

    #[test]
    fn overtemp_stays_tripped_in_hysteresis() {
        let mut ot = OvertempShutdown::new();
        // Trip
        ot.eval(&[92.0], &[90.0, 5.0], 0.0, &[]);
        // Cool to 87 — still within hysteresis (90-5=85)
        let out = ot.eval(&[87.0], &[90.0, 5.0], 0.0, &[]);
        assert_eq!(out[0], 1.0); // still tripped
    }

    #[test]
    fn overtemp_resets_below_hysteresis() {
        let mut ot = OvertempShutdown::new();
        ot.eval(&[92.0], &[90.0, 5.0], 0.0, &[]);
        // Cool below threshold - hysteresis
        let out = ot.eval(&[84.0], &[90.0, 5.0], 0.0, &[]);
        assert_eq!(out[0], 0.0); // reset
        assert_eq!(out[1], 1.0); // safe
    }

    #[test]
    fn overtemp_negative_temps() {
        let mut ot = OvertempShutdown::new();
        // threshold=-10 (e.g. freeze protection inverted)
        let out = ot.eval(&[-5.0], &[-10.0, 3.0], 0.0, &[]);
        assert_eq!(out[0], 1.0); // tripped (-5 >= -10)
        let out = ot.eval(&[-14.0], &[-10.0, 3.0], 0.0, &[]);
        assert_eq!(out[0], 0.0); // reset (-14 < -10-3 = -13)
    }

    #[test]
    fn overtemp_state_roundtrip() {
        let mut ot = OvertempShutdown::new();
        ot.eval(&[95.0], &[90.0, 5.0], 0.0, &[]);
        assert!(ot.tripped);
        let state = ot.state().unwrap();
        let mut ot2 = OvertempShutdown::new();
        ot2.restore(&state);
        assert_eq!(ot2.tripped, ot.tripped);
    }

    #[test]
    fn overtemp_exact_threshold() {
        let mut ot = OvertempShutdown::new();
        // At exactly threshold → trips (>=)
        let out = ot.eval(&[90.0], &[90.0, 5.0], 0.0, &[]);
        assert_eq!(out[0], 1.0);
    }
}
