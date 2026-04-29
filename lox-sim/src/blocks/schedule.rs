use crate::blocks::{
    bool_signal, deserialize_bool, deserialize_f64s, is_high, serialize_bool, serialize_f64s, Block,
};
use crate::types::Signal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayTimerEntry {
    pub day_of_week: Option<u32>,
    pub to_minute: f64,
    pub value: f64,
}

/// Simple day timer driven by minutes-since-midnight and day-of-week inputs.
#[derive(Clone)]
pub struct DayTimer {
    entries: Vec<DayTimerEntry>,
    last_active: bool,
    last_value: f64,
}

impl DayTimer {
    pub fn new(entries: Vec<DayTimerEntry>) -> Self {
        Self {
            entries,
            last_active: false,
            last_value: 0.0,
        }
    }

    fn value_at(&self, minutes_since_midnight: f64, day_of_week: u32) -> (f64, f64) {
        let mut matching: Vec<DayTimerEntry> = self
            .entries
            .iter()
            .copied()
            .filter(|entry| entry.day_of_week.is_none_or(|day| day == day_of_week))
            .collect();
        matching.sort_by(|left, right| left.to_minute.total_cmp(&right.to_minute));

        let mut current_value = 0.0;
        let mut remaining_minutes = 0.0;
        for entry in matching {
            if minutes_since_midnight < entry.to_minute {
                current_value = entry.value;
                remaining_minutes = (entry.to_minute - minutes_since_midnight).max(0.0);
                break;
            }
            current_value = entry.value;
        }
        (current_value, remaining_minutes * 60.0)
    }
}

impl Default for DayTimer {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Block for DayTimer {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let minutes_since_midnight = inputs.first().copied().unwrap_or(0.0);
        let day_of_week = inputs
            .get(1)
            .copied()
            .unwrap_or(0.0)
            .round()
            .clamp(0.0, 6.0) as u32;
        let (value, remaining_seconds) = self.value_at(minutes_since_midnight, day_of_week);
        let active = value.abs() > f64::EPSILON;
        let qon = !self.last_active && active;
        let qoff = self.last_active && !active;

        self.last_active = active;
        self.last_value = value;

        vec![
            value,
            value,
            bool_signal(qon),
            bool_signal(qoff),
            remaining_seconds,
        ]
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut state = serialize_bool(self.last_active);
        state.extend_from_slice(&serialize_f64s(&[self.last_value]));
        Some(state)
    }

    fn restore(&mut self, state: &[u8]) {
        self.last_active = deserialize_bool(state).unwrap_or(false);
        if state.len() > 1 {
            if let Some(values) = deserialize_f64s(&state[1..], 1) {
                self.last_value = values[0];
            }
        }
    }

    fn block_type(&self) -> &str {
        "DayTimer"
    }
}

/// Alarm clock: triggers at a specific time-of-day with an optional preparation
/// phase that fires before the alarm.
///
/// inputs: [0] = minutes since midnight, [1] = day of week (0–6),
///         [2] = enable (default high)
/// params: [0] = alarm minute (minutes since midnight),
///         [1] = alarm duration (seconds, how long Q stays high),
///         [2] = preparation time (minutes before alarm, default 0)
///
/// Outputs: [0] = Q (alarm active), [1] = QPrepare (preparation phase active)
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Q goes high when minutes_since_midnight crosses the alarm minute,
// and stays high for `alarm_duration` seconds. QPrepare goes high `prep_time`
// minutes before the alarm. Both are suppressed when enable is low.
#[derive(Clone)]
pub struct AlarmClock {
    alarm_remaining: f64,
    last_minutes: f64,
}

impl AlarmClock {
    pub fn new() -> Self {
        Self {
            alarm_remaining: 0.0,
            last_minutes: -1.0,
        }
    }
}

impl Default for AlarmClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for AlarmClock {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        // inputs: [0]=minutes_since_midnight, [1]=day_of_week, [2]=Deactivate,
        //         [3]=Acknowledge, [4]=Snooze, [5]=TgMe, [6]=TMe (alarm minute)
        // params: [0]=AlarmTime (seconds), [1]=PrepTime (seconds), [2]=SnoozeTime
        let minutes = inputs.first().copied().unwrap_or(0.0);
        let _day_of_week = inputs.get(1).copied().unwrap_or(0.0);
        let deactivated = inputs.get(2).copied().unwrap_or(0.0);
        let tme = inputs.get(6).copied().unwrap_or(0.0);
        let alarm_minute = if tme > 0.0 { tme } else { 420.0 }; // default 7:00 AM
        let alarm_duration = params.first().copied().unwrap_or(120.0).max(0.0);
        let prep_minutes = (params.get(1).copied().unwrap_or(180.0).max(0.0)) / 60.0;

        if is_high(deactivated) {
            self.alarm_remaining = 0.0;
            self.last_minutes = minutes;
            return vec![0.0, 0.0];
        }

        // Detect crossing the alarm minute
        let mut started_this_tick = false;
        if self.last_minutes >= 0.0
            && self.last_minutes < alarm_minute
            && minutes >= alarm_minute
            && self.alarm_remaining <= 0.0
        {
            self.alarm_remaining = alarm_duration;
            started_this_tick = self.alarm_remaining > 0.0;
        }
        self.last_minutes = minutes;

        let was_active = self.alarm_remaining > 0.0;
        if was_active {
            self.alarm_remaining = (self.alarm_remaining - dt).max(0.0);
        }
        let q = started_this_tick || was_active;

        // Preparation phase: active when within prep_minutes before alarm
        let prep_start = alarm_minute - prep_minutes;
        let q_prepare = !q && minutes >= prep_start && minutes < alarm_minute && prep_minutes > 0.0;

        vec![bool_signal(q), bool_signal(q_prepare)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.alarm_remaining, self.last_minutes]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 2) {
            self.alarm_remaining = values[0];
            self.last_minutes = values[1];
        }
    }

    fn block_type(&self) -> &str {
        "AlarmClock"
    }

    fn is_time_dependent(&self) -> bool {
        true
    }
}

/// Fire a single pulse at a configured time of day.
///
/// inputs: [0] = minutes since midnight, [1] = day of week (0–6)
/// params: [0] = trigger minute (minutes since midnight),
///         [1] = pulse duration (seconds, default 1.0)
///
/// Outputs: [0] = Q (pulse active)
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: Pulse fires when minutes_since_midnight crosses the trigger minute,
// stays high for pulse_duration seconds, then goes low until the next day.
#[derive(Clone)]
pub struct PulseAt {
    pulse_remaining: f64,
    last_minutes: f64,
}

impl PulseAt {
    pub fn new() -> Self {
        Self {
            pulse_remaining: 0.0,
            last_minutes: -1.0,
        }
    }
}

impl Default for PulseAt {
    fn default() -> Self {
        Self::new()
    }
}

impl Block for PulseAt {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let minutes = inputs.first().copied().unwrap_or(0.0);
        let trigger_minute = params.first().copied().unwrap_or(0.0);
        let pulse_duration = params.get(1).copied().unwrap_or(1.0).max(0.0);

        // Detect crossing the trigger minute (handles day wrap: last > current)
        let mut started_this_tick = false;
        if self.last_minutes >= 0.0
            && self.last_minutes < trigger_minute
            && minutes >= trigger_minute
            && self.pulse_remaining <= 0.0
        {
            self.pulse_remaining = pulse_duration.max(dt);
            started_this_tick = self.pulse_remaining > 0.0;
        }

        // Day wrap reset: if time went backwards (new day), allow re-trigger
        if minutes < self.last_minutes {
            // Day wrapped — reset so pulse can fire again tomorrow
        }

        self.last_minutes = minutes;

        let was_active = self.pulse_remaining > 0.0;
        if was_active {
            self.pulse_remaining = (self.pulse_remaining - dt).max(0.0);
        }
        let q = started_this_tick || was_active;

        vec![bool_signal(q)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.pulse_remaining, self.last_minutes]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(values) = deserialize_f64s(state, 2) {
            self.pulse_remaining = values[0];
            self.last_minutes = values[1];
        }
    }

    fn block_type(&self) -> &str {
        "PulseAt"
    }
}

/// Calendar block: detects holidays or special days.
///
/// Configured with a list of (month, day) pairs representing special days.
/// When the current date matches, the output goes high.
///
/// inputs: [0] = day of month (1–31), [1] = month (1–12)
///
/// Outputs: [0] = Q (1.0 if today is a special day, 0.0 otherwise)
// WARNING: Assumed behavior — not validated against Miniserver.
// Assumption: The Loxone Calendar block compares the current date against a
// configured list of special dates. We use (month, day) pairs. The real block
// may support more complex rules (e.g., Easter-relative, recurring patterns).
#[derive(Clone)]
pub struct Calendar {
    special_days: Vec<(u32, u32)>, // (month, day)
}

impl Calendar {
    pub fn new(special_days: Vec<(u32, u32)>) -> Self {
        Self { special_days }
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Block for Calendar {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        let day = inputs.first().copied().unwrap_or(0.0).round() as u32;
        let month = inputs.get(1).copied().unwrap_or(0.0).round() as u32;

        let is_special = self
            .special_days
            .iter()
            .any(|&(m, d)| m == month && d == day);

        vec![bool_signal(is_special)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Calendar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daytimer_uses_schedule_entries() {
        let entries = vec![
            DayTimerEntry {
                day_of_week: None,
                to_minute: 480.0,
                value: 0.0,
            },
            DayTimerEntry {
                day_of_week: None,
                to_minute: 1020.0,
                value: 1.0,
            },
            DayTimerEntry {
                day_of_week: None,
                to_minute: 1440.0,
                value: 0.0,
            },
        ];
        let mut block = DayTimer::new(entries);
        assert_eq!(
            block.eval(&[300.0, 0.0], &[], 0.0, &[]),
            vec![0.0, 0.0, 0.0, 0.0, 10800.0]
        );
        assert_eq!(
            block.eval(&[600.0, 0.0], &[], 0.0, &[]),
            vec![1.0, 1.0, 1.0, 0.0, 25200.0]
        );
        assert_eq!(
            block.eval(&[1200.0, 0.0], &[], 0.0, &[]),
            vec![0.0, 0.0, 0.0, 1.0, 14400.0]
        );
    }

    #[test]
    fn daytimer_can_filter_by_day() {
        let entries = vec![
            DayTimerEntry {
                day_of_week: Some(0),
                to_minute: 1440.0,
                value: 1.0,
            },
            DayTimerEntry {
                day_of_week: Some(1),
                to_minute: 1440.0,
                value: 0.0,
            },
        ];
        let mut block = DayTimer::new(entries);
        assert_eq!(block.eval(&[100.0, 0.0], &[], 0.0, &[])[0], 1.0);
        assert_eq!(block.eval(&[100.0, 1.0], &[], 0.0, &[])[0], 0.0);
    }

    // --- AlarmClock ---

    #[test]
    fn alarm_clock_triggers_at_configured_time() {
        let mut block = AlarmClock::new();
        // alarm at minute 60 (1:00 AM) via TMe input[6], duration 30s via param[0]
        let inputs = |min: f64, deact: f64| vec![min, 0.0, deact, 0.0, 0.0, 0.0, 60.0];
        let params = [30.0, 0.0]; // AlarmTime=30s, PrepTime=0s
        assert_eq!(
            block.eval(&inputs(50.0, 0.0), &params, 1.0, &[]),
            vec![0.0, 0.0]
        );
        // Cross the alarm minute (remaining starts at 30, -1 → 29)
        let out = block.eval(&inputs(60.0, 0.0), &params, 1.0, &[]);
        assert_eq!(out[0], 1.0); // alarm active
                                 // Still active after 10s (29-10 = 19 remaining)
        let out = block.eval(&inputs(60.5, 0.0), &params, 10.0, &[]);
        assert_eq!(out[0], 1.0);
        // Drain remaining 19s (output still high this tick, goes to 0)
        let out = block.eval(&inputs(61.0, 0.0), &params, 19.0, &[]);
        assert_eq!(out[0], 1.0);
        // Now expired
        let out = block.eval(&inputs(62.0, 0.0), &params, 1.0, &[]);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn alarm_clock_preparation_phase() {
        let mut block = AlarmClock::new();
        // alarm at 60, prep 600 seconds (10 minutes) before
        let inputs = |min: f64| vec![min, 0.0, 0.0, 0.0, 0.0, 0.0, 60.0];
        let params = [30.0, 600.0]; // AlarmTime=30s, PrepTime=600s (10min)
        let out = block.eval(&inputs(50.0), &params, 1.0, &[]);
        assert_eq!(out[0], 0.0); // alarm not yet
        assert_eq!(out[1], 1.0); // preparation active (50 >= 50 && 50 < 60)

        let out = block.eval(&inputs(45.0), &params, 1.0, &[]);
        assert_eq!(out[1], 0.0); // too early for preparation
    }

    #[test]
    fn alarm_clock_disabled() {
        let mut block = AlarmClock::new();
        let params = [30.0, 0.0]; // AlarmTime=30s, PrepTime=0s
        block.eval(&[50.0, 0.0, 0.0, 0.0, 0.0, 0.0, 60.0], &params, 1.0, &[]);
        // Deactivate=1.0 → disabled
        let out = block.eval(&[60.0, 0.0, 1.0, 0.0, 0.0, 0.0, 60.0], &params, 1.0, &[]);
        assert_eq!(out[0], 0.0); // disabled
    }

    // --- PulseAt ---

    #[test]
    fn pulse_at_fires_at_configured_time() {
        let mut block = PulseAt::new();
        // Pulse at minute 120, duration 2s
        assert_eq!(
            block.eval(&[100.0, 0.0], &[120.0, 2.0], 1.0, &[]),
            vec![0.0]
        );
        // Cross trigger minute (remaining starts at 2, -0.5 → 1.5)
        let out = block.eval(&[120.0, 0.0], &[120.0, 2.0], 0.5, &[]);
        assert_eq!(out, vec![1.0]);
        // Still active (1.5 - 0.5 = 1.0)
        let out = block.eval(&[121.0, 0.0], &[120.0, 2.0], 0.5, &[]);
        assert_eq!(out, vec![1.0]);
        // Still active (1.0 - 0.5 = 0.5)
        let out = block.eval(&[121.5, 0.0], &[120.0, 2.0], 0.5, &[]);
        assert_eq!(out, vec![1.0]);
        // Drains to 0 this tick (still high)
        let out = block.eval(&[122.0, 0.0], &[120.0, 2.0], 0.5, &[]);
        assert_eq!(out, vec![1.0]);
        // Now expired
        let out = block.eval(&[122.5, 0.0], &[120.0, 2.0], 0.5, &[]);
        assert_eq!(out, vec![0.0]);
    }

    #[test]
    fn pulse_at_does_not_retrigger_same_day() {
        let mut block = PulseAt::new();
        block.eval(&[100.0, 0.0], &[120.0, 1.0], 1.0, &[]);
        block.eval(&[120.0, 0.0], &[120.0, 1.0], 0.5, &[]);
        block.eval(&[121.0, 0.0], &[120.0, 1.0], 0.5, &[]);
        // Pulse expired. Passing minute 120 again should not retrigger.
        let out = block.eval(&[125.0, 0.0], &[120.0, 1.0], 1.0, &[]);
        assert_eq!(out, vec![0.0]);
    }

    #[test]
    fn pulse_at_retriggers_after_day_wrap() {
        let mut block = PulseAt::new();
        block.eval(&[100.0, 0.0], &[120.0, 1.0], 1.0, &[]);
        block.eval(&[120.0, 0.0], &[120.0, 1.0], 0.5, &[]);
        block.eval(&[121.0, 0.0], &[120.0, 1.0], 1.0, &[]); // pulse done
                                                            // Day wraps (time goes to 0)
        block.eval(&[0.0, 0.0], &[120.0, 1.0], 1.0, &[]);
        // Next day, approach and cross trigger again
        block.eval(&[100.0, 0.0], &[120.0, 1.0], 1.0, &[]);
        let out = block.eval(&[120.0, 0.0], &[120.0, 1.0], 0.5, &[]);
        assert_eq!(out, vec![1.0]);
    }

    // --- Calendar ---

    #[test]
    fn calendar_detects_special_day() {
        let mut block = Calendar::new(vec![(12, 25), (1, 1), (7, 4)]);
        // Christmas
        assert_eq!(block.eval(&[25.0, 12.0], &[], 0.0, &[]), vec![1.0]);
        // Regular day
        assert_eq!(block.eval(&[15.0, 3.0], &[], 0.0, &[]), vec![0.0]);
        // New Year
        assert_eq!(block.eval(&[1.0, 1.0], &[], 0.0, &[]), vec![1.0]);
    }

    #[test]
    fn calendar_empty_no_special_days() {
        let mut block = Calendar::default();
        assert_eq!(block.eval(&[25.0, 12.0], &[], 0.0, &[]), vec![0.0]);
    }

    // --- State/Restore tests ---

    #[test]
    fn alarm_clock_state_roundtrip() {
        let mut b = AlarmClock::new();
        let inputs = |min: f64| vec![min, 0.0, 0.0, 0.0, 0.0, 0.0, 60.0];
        let params = [60.0, 0.0]; // AlarmTime=60s, PrepTime=0s
        b.eval(&inputs(50.0), &params, 1.0, &[]);
        b.eval(&inputs(60.0), &params, 1.0, &[]);
        let state = b.state().unwrap();
        let mut b2 = AlarmClock::new();
        b2.restore(&state);
        // Should still be in alarm state
        let out = b2.eval(&inputs(61.0), &params, 1.0, &[]);
        assert_eq!(out[0], 1.0);
    }

    #[test]
    fn pulse_at_state_roundtrip() {
        let mut b = PulseAt::new();
        b.eval(&[100.0, 0.0], &[120.0, 5.0], 1.0, &[]);
        b.eval(&[120.0, 0.0], &[120.0, 5.0], 1.0, &[]);
        let state = b.state().unwrap();
        let mut b2 = PulseAt::new();
        b2.restore(&state);
        let out = b2.eval(&[121.0, 0.0], &[120.0, 5.0], 1.0, &[]);
        assert_eq!(out, vec![1.0]); // still pulsing
    }
}
