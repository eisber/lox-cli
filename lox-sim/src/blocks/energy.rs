//! Group C — Energy blocks (metering, management, solar, EV charging, load control).
//!
//! Simplified simulation models for Loxone energy-related blocks. Each complex block
//! carries a WARNING comment describing assumptions.

use crate::blocks::{
    bool_signal, deserialize_bool, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

// ---------------------------------------------------------------------------
// Energy
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Accumulates instantaneous power (W) over time to produce energy (Wh).
// energy_wh += power * dt / 3600. Validate against Miniserver.

/// Power metering accumulator — integrates watts into watt-hours.
#[derive(Clone, Default)]
pub struct Energy {
    total_wh: f64,
}

impl Energy {
    pub fn new() -> Self {
        Self { total_wh: 0.0 }
    }
}

impl Block for Energy {
    /// Inputs: [power_watts]
    /// Params: []
    /// Outputs: [total_wh, power_watts_passthrough]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let power = inputs.first().copied().unwrap_or(0.0);
        if dt > 0.0 {
            self.total_wh += power * dt / 3600.0;
        }
        vec![self.total_wh, power]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.total_wh]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.total_wh = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "Energy"
    }
}

// ---------------------------------------------------------------------------
// EnergyManager / EnergyManager2
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Computes grid_balance = production - consumption. Self-consumption
// priority: surplus output = max(0, production - consumption).
// Validate against Miniserver.

macro_rules! energy_manager_block {
    ($name:ident, $type_str:expr) => {
        #[derive(Clone, Default)]
        pub struct $name {
            total_consumed_wh: f64,
            total_produced_wh: f64,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    total_consumed_wh: 0.0,
                    total_produced_wh: 0.0,
                }
            }
        }

        impl Block for $name {
            /// Inputs: [consumption_watts, production_watts]
            /// Params: []
            /// Outputs: [grid_balance, surplus, total_consumed_wh, total_produced_wh]
            fn eval(
                &mut self,
                inputs: &[Signal],
                _params: &[Signal],
                dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                let consumption = inputs.first().copied().unwrap_or(0.0);
                let production = inputs.get(1).copied().unwrap_or(0.0);

                if dt > 0.0 {
                    self.total_consumed_wh += consumption * dt / 3600.0;
                    self.total_produced_wh += production * dt / 3600.0;
                }

                let balance = production - consumption;
                let surplus = balance.max(0.0);
                vec![
                    balance,
                    surplus,
                    self.total_consumed_wh,
                    self.total_produced_wh,
                ]
            }

            fn state(&self) -> Option<Vec<u8>> {
                Some(serialize_f64s(&[
                    self.total_consumed_wh,
                    self.total_produced_wh,
                ]))
            }

            fn restore(&mut self, state: &[u8]) {
                if let Some(v) = deserialize_f64s(state, 2) {
                    self.total_consumed_wh = v[0];
                    self.total_produced_wh = v[1];
                }
            }

            fn block_type(&self) -> &str {
                $type_str
            }
        }
    };
}

energy_manager_block!(EnergyManager, "EnergyManager");
energy_manager_block!(EnergyManager2, "EnergyManager2");

// ---------------------------------------------------------------------------
// Meter blocks
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Meters accumulate pulse counts or power values. Uni-directional meters
// only accumulate positive values; bi-directional meters accumulate signed values
// into separate positive/negative totals. Validate against Miniserver.

/// Absolute unidirectional meter — accumulates absolute value.
#[derive(Clone, Default)]
pub struct MeterAbsUni {
    total: f64,
}

impl MeterAbsUni {
    pub fn new() -> Self {
        Self { total: 0.0 }
    }
}

impl Block for MeterAbsUni {
    /// Inputs: [value]
    /// Params: [scale_factor]
    /// Outputs: [total]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let scale = params.first().copied().unwrap_or(1.0);
        self.total += value.abs() * scale;
        vec![self.total]
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
        "MeterAbsUni"
    }
}

/// Absolute bidirectional meter — separate positive/negative totals.
#[derive(Clone, Default)]
pub struct MeterAbsBi {
    total_pos: f64,
    total_neg: f64,
}

impl MeterAbsBi {
    pub fn new() -> Self {
        Self {
            total_pos: 0.0,
            total_neg: 0.0,
        }
    }
}

impl Block for MeterAbsBi {
    /// Inputs: [value]
    /// Params: [scale_factor]
    /// Outputs: [total_positive, total_negative]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = inputs.first().copied().unwrap_or(0.0);
        let scale = params.first().copied().unwrap_or(1.0);
        let scaled = value * scale;
        if scaled >= 0.0 {
            self.total_pos += scaled;
        } else {
            self.total_neg += scaled.abs();
        }
        vec![self.total_pos, self.total_neg]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.total_pos, self.total_neg]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 2) {
            self.total_pos = v[0];
            self.total_neg = v[1];
        }
    }

    fn block_type(&self) -> &str {
        "MeterAbsBi"
    }
}

/// Power unidirectional meter — integrates power over time.
#[derive(Clone, Default)]
pub struct MeterPUni {
    total_wh: f64,
}

impl MeterPUni {
    pub fn new() -> Self {
        Self { total_wh: 0.0 }
    }
}

impl Block for MeterPUni {
    /// Inputs: [power_watts]
    /// Params: []
    /// Outputs: [total_wh, current_power]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let power = inputs.first().copied().unwrap_or(0.0).max(0.0);
        if dt > 0.0 {
            self.total_wh += power * dt / 3600.0;
        }
        vec![self.total_wh, power]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.total_wh]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.total_wh = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "MeterPUni"
    }
}

/// Power bidirectional meter — separate import/export totals.
#[derive(Clone, Default)]
pub struct MeterPBi {
    import_wh: f64,
    export_wh: f64,
}

impl MeterPBi {
    pub fn new() -> Self {
        Self {
            import_wh: 0.0,
            export_wh: 0.0,
        }
    }
}

impl Block for MeterPBi {
    /// Inputs: [power_watts (positive=import, negative=export)]
    /// Params: []
    /// Outputs: [import_wh, export_wh, current_power]
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let power = inputs.first().copied().unwrap_or(0.0);
        if dt > 0.0 {
            if power >= 0.0 {
                self.import_wh += power * dt / 3600.0;
            } else {
                self.export_wh += (-power) * dt / 3600.0;
            }
        }
        vec![self.import_wh, self.export_wh, power]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.import_wh, self.export_wh]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 2) {
            self.import_wh = v[0];
            self.export_wh = v[1];
        }
    }

    fn block_type(&self) -> &str {
        "MeterPBi"
    }
}

/// Digital pulse meter — counts rising-edge pulses.
#[derive(Clone, Default)]
pub struct MeterDig {
    count: f64,
}

impl MeterDig {
    pub fn new() -> Self {
        Self { count: 0.0 }
    }
}

impl Block for MeterDig {
    /// Inputs: [pulse_input]
    /// Params: [value_per_pulse]
    /// Outputs: [total, pulse_count]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let current = inputs.first().copied().unwrap_or(0.0);
        let previous = prev_inputs.first().copied().unwrap_or(0.0);
        let per_pulse = params.first().copied().unwrap_or(1.0);

        if !is_high(previous) && is_high(current) {
            self.count += 1.0;
        }

        vec![self.count * per_pulse, self.count]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.count]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(v) = deserialize_f64s(state, 1) {
            self.count = v[0];
        }
    }

    fn block_type(&self) -> &str {
        "MeterDig"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Fronius
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Solar inverter interface — passes through production power input.
// Validate against Miniserver.

/// Fronius solar inverter interface — pass-through.
#[derive(Clone, Copy)]
pub struct Fronius;

impl Block for Fronius {
    /// Inputs: [production_watts]
    /// Outputs: [production_watts]
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
        "Fronius"
    }
}

// ---------------------------------------------------------------------------
// PVProductionForecast
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Estimates PV production as peak_power * irradiance_factor.
// irradiance_factor is input[0] normalised 0-1. In real systems this
// comes from weather API data. Validate against Miniserver.

/// Weather-based PV production prediction.
#[derive(Clone, Copy)]
pub struct PVProductionForecast;

impl Block for PVProductionForecast {
    /// Inputs: [irradiance_factor (0-1)]
    /// Params: [peak_power_watts]
    /// Outputs: [forecast_watts]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let irradiance = inputs.first().copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let peak = params.first().copied().unwrap_or(5000.0);
        vec![peak * irradiance]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "PVProductionForecast"
    }
}

// ---------------------------------------------------------------------------
// Wallbox
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: EV charger with state-of-charge tracking. When enabled, charges
// at charge_power_watts. SOC increases proportionally to battery_capacity_wh.
// Validate against Miniserver.

/// EV charger with SOC tracking.
#[derive(Clone, Default)]
pub struct Wallbox {
    soc: f64, // 0.0 - 1.0
    charging: bool,
}

impl Wallbox {
    pub fn new() -> Self {
        Self {
            soc: 0.0,
            charging: false,
        }
    }
}

impl Block for Wallbox {
    /// Inputs: [enable, available_power_watts]
    /// Params: [charge_power_watts, battery_capacity_wh]
    /// Outputs: [soc, charging_status, actual_power]
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let enable = inputs.first().copied().unwrap_or(0.0);
        let available = inputs.get(1).copied().unwrap_or(f64::MAX);
        let max_power = params.first().copied().unwrap_or(11000.0);
        let capacity = params.get(1).copied().unwrap_or(60000.0).max(1.0);

        self.charging = is_high(enable) && self.soc < 1.0;

        let actual_power = if self.charging {
            max_power.min(available.max(0.0))
        } else {
            0.0
        };

        if dt > 0.0 && self.charging {
            let energy = actual_power * dt / 3600.0;
            self.soc = (self.soc + energy / capacity).min(1.0);
            if self.soc >= 1.0 {
                self.charging = false;
            }
        }

        vec![self.soc, bool_signal(self.charging), actual_power]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut s = serialize_bool(self.charging);
        s.extend_from_slice(&serialize_f64s(&[self.soc]));
        Some(s)
    }

    fn restore(&mut self, state: &[u8]) {
        self.charging = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(v) = deserialize_f64s(&state[1..], 1) {
                self.soc = v[0];
            }
        }
    }

    fn block_type(&self) -> &str {
        "Wallbox"
    }
}

// ---------------------------------------------------------------------------
// LoadShed
// ---------------------------------------------------------------------------

// WARNING: Simplified model — real Loxone behavior may differ.
// Assumption: Sheds loads in priority order when total exceeds limit.
// Inputs are load power values; outputs are enable flags (1=allowed, 0=shed).
// Loads are prioritised by input index (0 = highest priority).
// Validate against Miniserver.

/// Priority-based load shedding controller.
#[derive(Clone, Copy)]
pub struct LoadShed;

impl Block for LoadShed {
    /// Inputs: [load_0_watts, load_1_watts, ..., load_N_watts]
    /// Params: [max_total_watts]
    /// Outputs: [enable_0, enable_1, ..., enable_N] (same count as inputs)
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let limit = params.first().copied().unwrap_or(10000.0);
        let mut running_total = 0.0;
        let mut enables: Vec<Signal> = Vec::with_capacity(inputs.len());

        for &load in inputs {
            if running_total + load <= limit {
                running_total += load;
                enables.push(1.0);
            } else {
                enables.push(0.0);
            }
        }

        if enables.is_empty() {
            enables.push(0.0);
        }

        enables
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }
    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "LoadShed"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Energy -------------------------------------------------------------

    #[test]
    fn energy_accumulates() {
        let mut e = Energy::new();
        // 1000W for 3600s = 1000 Wh
        let out = e.eval(&[1000.0], &[], 3600.0, &[]);
        assert!((out[0] - 1000.0).abs() < 0.01);
        assert_eq!(out[1], 1000.0);
    }

    #[test]
    fn energy_state_roundtrip() {
        let mut e = Energy::new();
        e.eval(&[500.0], &[], 3600.0, &[]);
        let state = e.state().unwrap();
        let mut e2 = Energy::new();
        e2.restore(&state);
        assert_eq!(e2.total_wh, e.total_wh);
    }

    // -- EnergyManager ------------------------------------------------------

    #[test]
    fn energy_manager_balance() {
        let mut em = EnergyManager::new();
        // consumption=1000, production=3000 → balance=2000, surplus=2000
        let out = em.eval(&[1000.0, 3000.0], &[], 1.0, &[]);
        assert!((out[0] - 2000.0).abs() < 0.01);
        assert!((out[1] - 2000.0).abs() < 0.01);
    }

    #[test]
    fn energy_manager_deficit() {
        let mut em = EnergyManager::new();
        // consumption=3000, production=1000 → balance=-2000, surplus=0
        let out = em.eval(&[3000.0, 1000.0], &[], 1.0, &[]);
        assert!((out[0] - -2000.0).abs() < 0.01);
        assert_eq!(out[1], 0.0);
    }

    #[test]
    fn energy_manager2_type_name() {
        let em = EnergyManager2::new();
        assert_eq!(em.block_type(), "EnergyManager2");
    }

    // -- MeterAbsUni --------------------------------------------------------

    #[test]
    fn meter_abs_uni_accumulates() {
        let mut m = MeterAbsUni::new();
        m.eval(&[10.0], &[1.0], 0.0, &[]);
        m.eval(&[-5.0], &[1.0], 0.0, &[]); // absolute → +5
        let out = m.eval(&[3.0], &[1.0], 0.0, &[]);
        assert!((out[0] - 18.0).abs() < 0.01); // 10 + 5 + 3
    }

    // -- MeterAbsBi ---------------------------------------------------------

    #[test]
    fn meter_abs_bi_separates() {
        let mut m = MeterAbsBi::new();
        m.eval(&[10.0], &[1.0], 0.0, &[]);
        let out = m.eval(&[-5.0], &[1.0], 0.0, &[]);
        assert!((out[0] - 10.0).abs() < 0.01); // positive total
        assert!((out[1] - 5.0).abs() < 0.01); // negative total
    }

    // -- MeterPUni ----------------------------------------------------------

    #[test]
    fn meter_p_uni_integrates() {
        let mut m = MeterPUni::new();
        // 1000W for 3600s = 1000 Wh
        let out = m.eval(&[1000.0], &[], 3600.0, &[]);
        assert!((out[0] - 1000.0).abs() < 0.01);
    }

    #[test]
    fn meter_p_uni_ignores_negative() {
        let mut m = MeterPUni::new();
        let out = m.eval(&[-1000.0], &[], 3600.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    // -- MeterPBi -----------------------------------------------------------

    #[test]
    fn meter_p_bi_import_export() {
        let mut m = MeterPBi::new();
        m.eval(&[1000.0], &[], 3600.0, &[]); // import
        let out = m.eval(&[-500.0], &[], 3600.0, &[]); // export
        assert!((out[0] - 1000.0).abs() < 0.01); // import
        assert!((out[1] - 500.0).abs() < 0.01); // export
    }

    // -- MeterDig -----------------------------------------------------------

    #[test]
    fn meter_dig_counts_pulses() {
        let mut m = MeterDig::new();
        m.eval(&[1.0], &[0.01], 0.0, &[0.0]); // pulse 1
        m.eval(&[0.0], &[0.01], 0.0, &[1.0]); // no pulse
        let out = m.eval(&[1.0], &[0.01], 0.0, &[0.0]); // pulse 2
        assert_eq!(out[1], 2.0); // count
        assert!((out[0] - 0.02).abs() < 0.001); // total (2 * 0.01)
    }

    // -- Fronius ------------------------------------------------------------

    #[test]
    fn fronius_passthrough() {
        let mut f = Fronius;
        let out = f.eval(&[4500.0], &[], 0.0, &[]);
        assert_eq!(out[0], 4500.0);
    }

    // -- PVProductionForecast -----------------------------------------------

    #[test]
    fn pv_forecast() {
        let mut pv = PVProductionForecast;
        // 80% irradiance, 5000W peak → 4000W
        let out = pv.eval(&[0.8], &[5000.0], 0.0, &[]);
        assert!((out[0] - 4000.0).abs() < 0.01);
    }

    #[test]
    fn pv_forecast_clamps_irradiance() {
        let mut pv = PVProductionForecast;
        let out = pv.eval(&[1.5], &[5000.0], 0.0, &[]);
        assert_eq!(out[0], 5000.0); // clamped to 1.0
    }

    // -- Wallbox ------------------------------------------------------------

    #[test]
    fn wallbox_charges() {
        let mut wb = Wallbox::new();
        // 11kW charger, 60kWh battery, enable=1, available=11000W
        let out = wb.eval(&[1.0, 11000.0], &[11000.0, 60000.0], 3600.0, &[]);
        assert!(out[0] > 0.0); // SOC increased
        assert_eq!(out[1], 1.0); // charging
        assert!((out[2] - 11000.0).abs() < 0.01);
    }

    #[test]
    fn wallbox_limited_power() {
        let mut wb = Wallbox::new();
        // available only 5000W of 11000W max
        let out = wb.eval(&[1.0, 5000.0], &[11000.0, 60000.0], 3600.0, &[]);
        assert!((out[2] - 5000.0).abs() < 0.01);
    }

    #[test]
    fn wallbox_stops_at_full() {
        let mut wb = Wallbox::new();
        wb.soc = 0.99;
        // Charge enough to fill
        let out = wb.eval(&[1.0, 11000.0], &[11000.0, 60000.0], 36000.0, &[]);
        assert!((out[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn wallbox_state_roundtrip() {
        let mut wb = Wallbox::new();
        wb.eval(&[1.0, 11000.0], &[11000.0, 60000.0], 1800.0, &[]);
        let state = wb.state().unwrap();
        let mut wb2 = Wallbox::new();
        wb2.restore(&state);
        assert_eq!(wb2.soc, wb.soc);
        assert_eq!(wb2.charging, wb.charging);
    }

    // -- LoadShed -----------------------------------------------------------

    #[test]
    fn load_shed_within_limit() {
        let mut ls = LoadShed;
        let out = ls.eval(&[3000.0, 2000.0, 1000.0], &[10000.0], 0.0, &[]);
        assert_eq!(out, vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn load_shed_over_limit() {
        let mut ls = LoadShed;
        // limit=5000, loads=[3000, 2000, 1500] → first two fit, third shed
        let out = ls.eval(&[3000.0, 2000.0, 1500.0], &[5000.0], 0.0, &[]);
        assert_eq!(out, vec![1.0, 1.0, 0.0]);
    }

    #[test]
    fn load_shed_empty_inputs() {
        let mut ls = LoadShed;
        let out = ls.eval(&[], &[5000.0], 0.0, &[]);
        assert_eq!(out, vec![0.0]);
    }

    // -- Factory registration -----------------------------------------------

    #[test]
    fn energy_block_types() {
        use crate::blocks::create_block;
        let types = [
            "Energy",
            "EnergyManager",
            "EnergyManager2",
            "MeterAbsUni",
            "MeterAbsBi",
            "MeterPUni",
            "MeterPBi",
            "MeterDig",
            "Fronius",
            "PVProductionForecast",
            "Wallbox",
            "LoadShed",
        ];
        for t in &types {
            let b = create_block(t);
            assert_eq!(b.block_type(), *t, "create_block({t}) returned wrong type");
        }
    }
}
