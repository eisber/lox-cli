use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::types::Signal;

pub mod compare;
pub mod controllers;
pub mod energy;
pub mod io;
pub mod logic;
pub mod math;
pub mod misc;
pub mod schedule;
pub mod security;
pub mod state;
pub mod timers;

pub use compare::{
    AnalogComparator, AnalogDiffTrigger, AnalogThresholdTrigger, AnalogWatchdog, Greater,
    GreaterEqual, Less, LessEqual,
};
pub use controllers::{
    AcControl, AutoJalousie, DaylightController, Fan, Fancoil, FancoilFreshAir, HVACController,
    HeatIRoomController2, Heatcurve, Heatmixer, Heatmixer2, IRcontroller, IRoomcontrol,
    JalousieUpDown2, Jalousiemotor, LightController, LightController2, OvertempShutdown, Pi, Pid,
    PoolController, Sauna, SaunaVapor, Solarpumpcontrol, ThreePoint, ToiletFan, TwoPoint,
    VentInternorm, Ventilation, Ventilation2,
};
// Also from controllers (jalousie alias macros):
pub use controllers::{EIBJalousie, Pergola, Skylight};
pub use energy::{
    Energy, EnergyManager, EnergyManager2, Fronius, LoadShed, MeterAbsBi, MeterAbsUni, MeterDig,
    MeterPBi, MeterPUni, PVProductionForecast, Wallbox,
};
pub use io::{
    EIBPush, EIBactor, EIBextactor, EIBsensor, EibDimmer, InputRef, OutputRef, OutputRefLM,
    VirtualIn, VirtualOut, VirtualState,
};
pub use logic::{Equal, Nand, Nor, Not, NotEqual, Or, Xor};
pub use math::{
    AMinmax, Add, Add4, AnalogMultiplexer, AnalogMultiplexer2, AnalogScaler, AnalogStepper, Div,
    Formula, Int, Minmax, Mod, Mult, Sub,
};
pub use misc::{
    AutomaticScene, AutopilotRule, Average, Avg, BinDecoder, BinEncoder, BrightnessControl,
    CallGen, CarCharger, CentralFancoil, CentralGate, CentralLight, CentralMusic, CentralPresence,
    CentralRoofwindow, CentralShade, ClimateControllerUS, CmdRecognition, Code1, Code16, Code4,
    Code8, DbConE, DbConS, DeviceTablet, DewPoint, Door, Doorcontroller, HeatCentral, HvacAC,
    Intercom, Irrigation, Leaf, LightControllerH, Lightscene, LightsceneLearn, LightsceneRGB,
    LongClick, MPGroup, MailBox, MailGen, Media, MediaClient, MeterAbsSt, MeterPSt, MultiClick,
    MultiFuncSW, MusicPlayer, Nevo, NfcCodeTouch, PButtonT, Ping, Power, PowerUnit, PulseBy,
    PushDimmer, Radio, Radio2, Ramp, Rand, RandomGen, RoofWindow, Roomcontrol, SequenceController,
    Sequencer, ShadeRoof, SpotOpt, StatusMonitor, SteakThermo, StepSel, SysVar, Tablet,
    TextGenerator, TimeMinmax, TpfController, Validator, Weed, Wind, WindowsMonitor, EFM, WBEM,
};
pub use schedule::{AlarmClock, Calendar, DayTimer, DayTimerEntry, PulseAt};
pub use security::{
    AalEmergency, AalSmartAlarm, Access, Alarm, AlarmChain, CentralAlarm, JoinWindowSensor,
    Presence, PresenceController, PresenceDetector, SmokeAlarm,
};
pub use state::{
    AMemory, Counter, FlipFlop, HourCounter, Memory, PushButton, PushButton2, RSFlipFlop,
    SampleHold, Shift, State, StateV, UpDownCounter,
};
pub use timers::{
    EdgeDetection, EdgeWipingRelay, Monoflop, OffDelay, OnDelay, OnOffDelay, OnPulseDelay,
    PulseGen, RetOnDelay, StairwayLS, SwitchingTimer, PWM,
};

pub(crate) const DIGITAL_THRESHOLD: Signal = 0.5;

pub(crate) fn is_high(value: Signal) -> bool {
    value >= DIGITAL_THRESHOLD
}

pub(crate) fn bool_signal(value: bool) -> Signal {
    if value {
        1.0
    } else {
        0.0
    }
}

pub(crate) fn serialize_bool(value: bool) -> Vec<u8> {
    vec![u8::from(value)]
}

pub(crate) fn deserialize_bool(bytes: &[u8]) -> Option<bool> {
    bytes.first().map(|byte| *byte != 0)
}

pub(crate) fn serialize_f64s(values: &[f64]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 8);
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

pub(crate) fn deserialize_f64s(bytes: &[u8], count: usize) -> Option<Vec<f64>> {
    if bytes.len() < count * 8 {
        return None;
    }
    let mut values = Vec::with_capacity(count);
    for chunk in bytes[..count * 8].chunks_exact(8) {
        values.push(f64::from_le_bytes(chunk.try_into().ok()?));
    }
    Some(values)
}

static BLOCK_WARNING_CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub(crate) fn warn_block_once(key: &str, message: &str) {
    let cache = BLOCK_WARNING_CACHE.get_or_init(|| Mutex::new(HashSet::new()));
    let mut warned = cache.lock().expect("block warning cache poisoned");
    if warned.insert(key.to_string()) {
        eprintln!("WARNING: {message}");
    }
}

pub trait BlockClone {
    fn clone_box(&self) -> Box<dyn Block>;
}

impl<T> BlockClone for T
where
    T: 'static + Block + Clone,
{
    fn clone_box(&self) -> Box<dyn Block> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Block> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Core block trait — all simulation blocks implement this.
pub trait Block: Send + Sync + BlockClone {
    /// Evaluate the block for one tick.
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal>;

    /// Serialize internal state (for snapshot/restore). Returns `None` for stateless blocks.
    fn state(&self) -> Option<Vec<u8>>;

    /// Restore internal state from serialized bytes.
    fn restore(&mut self, state: &[u8]);

    /// Block type name.
    fn block_type(&self) -> &str;

    /// Whether the block's output depends on `prev_inputs`.
    fn is_edge_sensitive(&self) -> bool {
        false
    }

    /// Whether the block has time-dependent internal state that needs
    /// re-evaluation every tick (timers, counters, delays).
    fn is_time_dependent(&self) -> bool {
        false
    }
}

/// Passes first input to output unchanged.
#[derive(Clone, Copy)]
pub struct PassThrough;

impl Block for PassThrough {
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
        "PassThrough"
    }
}

/// Digital AND: output 1.0 if all inputs are high.
#[derive(Clone, Copy)]
pub struct And;

impl Block for And {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _p: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let value = !inputs.is_empty() && inputs.iter().all(|&v| is_high(v));
        vec![bool_signal(value)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "And"
    }
}

/// Outputs the first parameter value (constant source).
#[derive(Clone, Copy)]
pub struct Constant;

impl Block for Constant {
    fn eval(
        &mut self,
        _i: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        vec![params.first().copied().unwrap_or(0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Constant"
    }
}

/// Rising-edge detector: outputs 1.0 for one tick on low-to-high transition.
#[derive(Clone, Copy)]
pub struct RisingEdge;

impl Block for RisingEdge {
    fn eval(&mut self, inputs: &[Signal], _p: &[Signal], _dt: f64, prev: &[Signal]) -> Vec<Signal> {
        let cur = inputs.first().copied().unwrap_or(0.0);
        let prv = prev.first().copied().unwrap_or(0.0);
        vec![bool_signal(!is_high(prv) && is_high(cur))]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "RisingEdge"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }
}

/// Multiplies input by a gain factor (param[0], default 1.0).
#[derive(Clone, Copy)]
pub struct Gain;

impl Block for Gain {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let input = inputs.first().copied().unwrap_or(0.0);
        let gain = params.first().copied().unwrap_or(1.0);
        vec![input * gain]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "Gain"
    }
}

/// PassThrough that tracks eval count via a shared atomic counter.
#[derive(Clone)]
pub struct TrackingPassThrough {
    counter: Arc<AtomicUsize>,
}

impl TrackingPassThrough {
    pub fn new() -> (Self, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        (
            Self {
                counter: counter.clone(),
            },
            counter,
        )
    }
}

impl Block for TrackingPassThrough {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _p: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        vec![inputs.first().copied().unwrap_or(0.0)]
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}

    fn block_type(&self) -> &str {
        "TrackingPassThrough"
    }
}

/// Create a block implementation for a Loxone block type.
pub fn create_block(block_type: &str) -> Box<dyn Block> {
    match block_type {
        // Schedule
        "AlarmClock" => Box::new(AlarmClock::new()),
        // State
        "AMemory" => Box::new(AMemory::new()),
        // Math
        "AMinmax" => Box::new(AMinmax::new()),
        "Add" => Box::new(Add),
        "Add4" => Box::new(Add4),
        // Compare
        "AnalogComparator" => Box::new(AnalogComparator::new()),
        "AnalogDiffTrigger" => Box::new(AnalogDiffTrigger::new()),
        "AnalogMultiplexer" => Box::new(AnalogMultiplexer),
        "AnalogMultiplexer2" => Box::new(AnalogMultiplexer2),
        "AnalogScaler" => Box::new(AnalogScaler),
        "AnalogStepper" => Box::new(AnalogStepper::new()),
        "AnalogThresholdTrigger" => Box::new(AnalogThresholdTrigger::new()),
        "AnalogWatchdog" => Box::new(AnalogWatchdog::new()),
        // Logic
        "And" => Box::new(And),
        // Schedule
        "Calendar" => Box::new(Calendar::default()),
        // Constant
        "Constant" => Box::new(Constant),
        // State
        "Counter" => Box::new(Counter::new()),
        // Schedule
        "DayTimer" => Box::new(DayTimer::default()),
        // Math
        "Div" => Box::new(Div),
        // Timers
        "EdgeDetection" => Box::new(EdgeDetection::new()),
        "EdgeWipingRelay" => Box::new(EdgeWipingRelay::new()),
        "Equal" => Box::new(Equal),
        "FlipFlop" | "SRFlipFlop" => Box::new(FlipFlop::new()),
        "Formula" => Box::new(Formula::new("I1")),
        "Gain" => Box::new(Gain),
        // Compare
        "Greater" => Box::new(Greater),
        "GreaterEqual" => Box::new(GreaterEqual),
        // State
        "HourCounter" => Box::new(HourCounter::new()),
        "Int" => Box::new(Int),
        "Less" => Box::new(Less),
        "LessEqual" => Box::new(LessEqual),
        // State
        "Memory" => Box::new(Memory::new()),
        "Minmax" => Box::new(Minmax),
        "Mod" => Box::new(Mod),
        // Timers
        "Monoflop" => Box::new(Monoflop::new()),
        // Math
        "Mult" => Box::new(Mult),
        // Logic
        "Nand" => Box::new(Nand),
        "Nor" => Box::new(Nor),
        "Not" => Box::new(Not),
        "NotEqual" => Box::new(NotEqual),
        // Timers
        "OffDelay" => Box::new(OffDelay::new()),
        "OnDelay" => Box::new(OnDelay::new()),
        "OnOffDelay" => Box::new(OnOffDelay::new()),
        "OnPulseDelay" => Box::new(OnPulseDelay::new()),
        // Logic
        "Or" => Box::new(Or),
        "PassThrough" => Box::new(PassThrough),
        // Timers
        "PulseGen" => Box::new(PulseGen::new()),
        // Schedule
        "PulseAt" => Box::new(PulseAt::new()),
        // State
        "PushButton" | "PushButtonSel" => Box::new(PushButton::new()),
        "PushButton2" | "PushButton2Sel" => Box::new(PushButton2::new()),
        // Timers
        "PWM" => Box::new(PWM::new()),
        // Timers
        "RetOnDelay" => Box::new(RetOnDelay::new()),
        "RisingEdge" => Box::new(RisingEdge),
        // State
        "RSFlipFlop" => Box::new(RSFlipFlop::new()),
        // State
        "SampleHold" => Box::new(SampleHold::new()),
        "Shift" => Box::new(Shift::new()),
        "State" => Box::new(State::new()),
        "StateV" => Box::new(StateV),
        // Timers
        "StairwayLS" => Box::new(StairwayLS::new()),
        "SwitchingTimer" => Box::new(SwitchingTimer::new()),
        // Math
        "Sub" => Box::new(Sub),
        "TrackingPassThrough" => Box::new(TrackingPassThrough::new().0),
        // State
        "UpDownCounter" => Box::new(UpDownCounter::new()),
        // Logic
        "Xor" => Box::new(Xor),

        // Group C — Controllers
        "LightController2" => Box::new(LightController2::new()),
        "LightController" => Box::new(LightController::new()),
        "JalousieUpDown2" => Box::new(JalousieUpDown2::new()),
        "Jalousiemotor" => Box::new(Jalousiemotor::new()),
        "EIBJalousie" => Box::new(EIBJalousie::new()),
        "Pergola" => Box::new(Pergola::new()),
        "Skylight" => Box::new(Skylight::new()),
        "AutoJalousie" => Box::new(AutoJalousie),
        "HeatIRoomController2" => Box::new(HeatIRoomController2::new()),
        "AcControl" => Box::new(AcControl::new()),
        "HVACController" => Box::new(HVACController::new()),
        "OvertempShutdown" => Box::new(OvertempShutdown::new()),
        "Ventilation" => Box::new(Ventilation),
        "Ventilation2" => Box::new(Ventilation2),
        "VentInternorm" => Box::new(VentInternorm),
        "ToiletFan" => Box::new(ToiletFan),
        "Fan" => Box::new(Fan),
        "Fancoil" => Box::new(Fancoil),
        "FancoilFreshAir" => Box::new(FancoilFreshAir),
        "IRoomcontrol" => Box::new(IRoomcontrol),
        "IRcontroller" => Box::new(IRcontroller),
        "DaylightController" => Box::new(DaylightController),
        "Heatcurve" => Box::new(Heatcurve),
        "Heatmixer" => Box::new(Heatmixer),
        "Heatmixer2" => Box::new(Heatmixer2),
        "Solarpumpcontrol" => Box::new(Solarpumpcontrol::new()),
        "2Point" => Box::new(TwoPoint::new()),
        "3Point" => Box::new(ThreePoint),
        "PID" => Box::new(Pid::new()),
        "PI" => Box::new(Pi::new()),
        "PoolController" => Box::new(PoolController::new()),
        "Sauna" => Box::new(Sauna::new()),
        "SaunaVapor" => Box::new(SaunaVapor::new()),

        // Group C — Energy
        "Energy" => Box::new(Energy::new()),
        "EnergyManager" => Box::new(EnergyManager::new()),
        "EnergyManager2" => Box::new(EnergyManager2::new()),
        "MeterAbsUni" => Box::new(MeterAbsUni::new()),
        "MeterAbsBi" => Box::new(MeterAbsBi::new()),
        "MeterPUni" => Box::new(MeterPUni::new()),
        "MeterPBi" => Box::new(MeterPBi::new()),
        "MeterDig" => Box::new(MeterDig::new()),
        "Fronius" => Box::new(Fronius),
        "PVProductionForecast" => Box::new(PVProductionForecast),
        "Wallbox" => Box::new(Wallbox::new()),
        "LoadShed" => Box::new(LoadShed),

        // Group D — Security
        "Alarm" => Box::new(Alarm::new()),
        "CentralAlarm" => Box::new(CentralAlarm),
        "AlarmChain" => Box::new(AlarmChain),
        "SmokeAlarm" => Box::new(SmokeAlarm),
        "Presence" => Box::new(Presence::new()),
        "PresenceController" => Box::new(PresenceController::new()),
        "PresenceDetector" => Box::new(PresenceDetector),
        "Access" => Box::new(Access),
        "AalEmergency" => Box::new(AalEmergency),
        "AalSmartAlarm" => Box::new(AalSmartAlarm),
        "JoinWindowSensor" => Box::new(JoinWindowSensor),

        // Group D — I/O
        "InputRef" => Box::new(InputRef),
        "OutputRef" => Box::new(OutputRef),
        "OutputRefLM" => Box::new(OutputRefLM),
        "EIBPush" => Box::new(EIBPush),
        "EIBsensor" => Box::new(EIBsensor),
        "EIBactor" => Box::new(EIBactor),
        "EIBextactor" => Box::new(EIBextactor),
        "EibDimmer" => Box::new(EibDimmer),
        "VirtualIn" => Box::new(VirtualIn),
        "VirtualOut" => Box::new(VirtualOut),
        "VirtualState" => Box::new(VirtualState),

        // Group D — Misc: Math/Encoding
        "Average" => Box::new(Average),
        "Avg" => Box::new(Avg),
        "BinEncoder" => Box::new(BinEncoder),
        "BinDecoder" => Box::new(BinDecoder),
        "DewPoint" => Box::new(DewPoint),
        "Power" => Box::new(Power),
        "Validator" => Box::new(Validator),
        "TimeMinmax" => Box::new(TimeMinmax::new()),

        // Group D — Misc: Timing/Pulse
        "Ramp" => Box::new(Ramp::new()),
        "Rand" => Box::new(Rand::new()),
        "RandomGen" => Box::new(RandomGen::new()),
        "PulseBy" => Box::new(PulseBy::new()),

        // Group D — Misc: Buttons
        "LongClick" => Box::new(LongClick::new()),
        "MultiClick" => Box::new(MultiClick::new()),
        "PushDimmer" => Box::new(PushDimmer::new()),
        "StepSel" => Box::new(StepSel::new()),
        "PButtonT" => Box::new(PButtonT),
        "MultiFuncSW" => Box::new(MultiFuncSW),

        // Group D — Misc: Sequencer
        "Sequencer" => Box::new(Sequencer::new()),
        "SequenceController" => Box::new(SequenceController),

        // Group D — Misc: Central controllers
        "CentralLight" => Box::new(CentralLight),
        "CentralMusic" => Box::new(CentralMusic),
        "CentralGate" => Box::new(CentralGate),
        "CentralPresence" => Box::new(CentralPresence),
        "CentralFancoil" => Box::new(CentralFancoil),
        "CentralRoofwindow" => Box::new(CentralRoofwindow),
        "CentralShade" => Box::new(CentralShade),

        // Group D — Misc: PicoC code blocks
        "Code1" => {
            warn_block_once(
                "Code1",
                "PicoC block 'Code1' is not simulated; using pass-through stub",
            );
            Box::new(Code1)
        }
        "Code4" => {
            warn_block_once(
                "Code4",
                "PicoC block 'Code4' is not simulated; using pass-through stub",
            );
            Box::new(Code4)
        }
        "Code8" => {
            warn_block_once(
                "Code8",
                "PicoC block 'Code8' is not simulated; using pass-through stub",
            );
            Box::new(Code8)
        }
        "Code16" => {
            warn_block_once(
                "Code16",
                "PicoC block 'Code16' is not simulated; using pass-through stub",
            );
            Box::new(Code16)
        }

        // Group D — Misc: Meters
        "MeterAbsSt" => Box::new(MeterAbsSt::new()),
        "MeterPSt" => Box::new(MeterPSt::new()),

        // Group D — Misc: Lighting scenes
        "Lightscene" => Box::new(Lightscene),
        "LightsceneLearn" => Box::new(LightsceneLearn),
        "LightsceneRGB" => Box::new(LightsceneRGB),
        "LightControllerH" => Box::new(LightControllerH),

        // Group D — Misc: HVAC stubs (HVACController moved to Group C — Controllers)
        "HvacAC" => Box::new(HvacAC),
        "HeatCentral" => Box::new(HeatCentral),
        "ClimateControllerUS" => Box::new(ClimateControllerUS),

        // Group D — Misc: Media/device stubs
        "Media" => Box::new(Media),
        "MediaClient" => Box::new(MediaClient),
        "MusicPlayer" => Box::new(MusicPlayer),
        "Radio" => Box::new(Radio),
        "Radio2" => Box::new(Radio2),
        "Intercom" => Box::new(Intercom),
        "Tablet" => Box::new(Tablet),
        "Device Tablet" => Box::new(DeviceTablet),
        "Nevo" => Box::new(Nevo),
        "NfcCodeTouch" => Box::new(NfcCodeTouch),
        "MPGroup" => Box::new(MPGroup),

        // Group D — Misc: Automation stubs
        "AutomaticScene" => Box::new(AutomaticScene),
        "AutopilotRule" => Box::new(AutopilotRule),
        "BrightnessControl" => Box::new(BrightnessControl),
        "CmdRecognition" => Box::new(CmdRecognition),

        // Group D — Misc: Communication stubs
        "CallGen" => Box::new(CallGen),
        "MailBox" => Box::new(MailBox),
        "MailGen" => Box::new(MailGen),
        "SysVar" => Box::new(SysVar),
        "TextGenerator" => Box::new(TextGenerator),

        // Group D — Misc: Infrastructure stubs
        "Ping" => Box::new(Ping),
        "PowerUnit" => Box::new(PowerUnit),
        "CarCharger" => Box::new(CarCharger),
        "StatusMonitor" => Box::new(StatusMonitor),
        "SteakThermo" => Box::new(SteakThermo),
        "SpotOpt" => Box::new(SpotOpt),

        // Group D — Misc: Access/door stubs
        "Door" => Box::new(Door),
        "Doorcontroller" => Box::new(Doorcontroller),

        // Group D — Misc: Shade/window stubs
        "RoofWindow" => Box::new(RoofWindow),
        "ShadeRoof" => Box::new(ShadeRoof),

        // Group D — Misc: Outdoor stubs
        "Irrigation" => Box::new(Irrigation),
        "Leaf" => Box::new(Leaf),
        "Weed" => Box::new(Weed),
        "Wind" => Box::new(Wind),

        // Group D — Misc: Database/connectivity stubs
        "DbConE" => Box::new(DbConE),
        "DbConS" => Box::new(DbConS),
        "WBEM" => Box::new(WBEM),
        "EFM" => Box::new(EFM),

        // Group D — Misc: Controller stubs
        "Roomcontrol" => Box::new(Roomcontrol),
        "WindowsMonitor" => Box::new(WindowsMonitor),
        "TpfController" => Box::new(TpfController),

        _ => {
            warn_block_once(
                &format!("unknown:{block_type}"),
                &format!(
                    "Unknown block type '{block_type}' is not implemented; falling back to PassThrough"
                ),
            );
            Box::new(PassThrough)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_through_forwards_input() {
        let mut block = PassThrough;
        assert_eq!(block.eval(&[5.0], &[], 0.1, &[0.0]), vec![5.0]);
        assert_eq!(block.eval(&[], &[], 0.1, &[5.0]), vec![0.0]);
    }

    #[test]
    fn and_all_high() {
        let mut block = And;
        assert_eq!(block.eval(&[1.0, 1.0, 1.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[1.0, 0.0, 1.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn constant_outputs_param() {
        let mut block = Constant;
        assert_eq!(block.eval(&[], &[42.0], 0.0, &[]), vec![42.0]);
    }

    #[test]
    fn rising_edge_detects_transition() {
        let mut block = RisingEdge;
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[0.0]), vec![1.0]);
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[1.0]), vec![0.0]);
    }

    #[test]
    fn gain_multiplies() {
        let mut block = Gain;
        assert_eq!(block.eval(&[5.0], &[3.0], 0.0, &[]), vec![15.0]);
    }

    #[test]
    fn tracking_counts_evals() {
        let (mut block, counter) = TrackingPassThrough::new();
        block.eval(&[1.0], &[], 0.0, &[]);
        block.eval(&[2.0], &[], 0.0, &[]);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn factory_maps_known_and_unknown_types() {
        assert_eq!(create_block("And").block_type(), "And");
        assert_eq!(create_block("PulseGen").block_type(), "PulseGen");
        assert_eq!(create_block("NoSuchBlock").block_type(), "PassThrough");
    }

    #[test]
    fn factory_maps_all_group_a_logic_types() {
        assert_eq!(create_block("And").block_type(), "And");
        assert_eq!(create_block("Or").block_type(), "Or");
        assert_eq!(create_block("Not").block_type(), "Not");
        assert_eq!(create_block("Xor").block_type(), "Xor");
        assert_eq!(create_block("Nand").block_type(), "Nand");
        assert_eq!(create_block("Nor").block_type(), "Nor");
        assert_eq!(create_block("Equal").block_type(), "Equal");
        assert_eq!(create_block("NotEqual").block_type(), "NotEqual");
    }

    #[test]
    fn factory_maps_all_group_a_math_types() {
        assert_eq!(create_block("Add").block_type(), "Add");
        assert_eq!(create_block("Sub").block_type(), "Sub");
        assert_eq!(create_block("Mult").block_type(), "Mult");
        assert_eq!(create_block("Div").block_type(), "Div");
        assert_eq!(create_block("Mod").block_type(), "Mod");
        assert_eq!(create_block("Int").block_type(), "Int");
        assert_eq!(create_block("Add4").block_type(), "Add4");
        assert_eq!(create_block("Minmax").block_type(), "Minmax");
        assert_eq!(create_block("AMinmax").block_type(), "AMinmax");
        assert_eq!(create_block("Formula").block_type(), "Formula");
        assert_eq!(create_block("AnalogScaler").block_type(), "AnalogScaler");
        assert_eq!(
            create_block("AnalogMultiplexer").block_type(),
            "AnalogMultiplexer"
        );
        assert_eq!(
            create_block("AnalogMultiplexer2").block_type(),
            "AnalogMultiplexer2"
        );
        assert_eq!(create_block("AnalogStepper").block_type(), "AnalogStepper");
    }

    #[test]
    fn factory_maps_all_group_a_compare_types() {
        assert_eq!(create_block("Greater").block_type(), "Greater");
        assert_eq!(create_block("GreaterEqual").block_type(), "GreaterEqual");
        assert_eq!(create_block("Less").block_type(), "Less");
        assert_eq!(create_block("LessEqual").block_type(), "LessEqual");
        assert_eq!(
            create_block("AnalogComparator").block_type(),
            "AnalogComparator"
        );
        assert_eq!(
            create_block("AnalogDiffTrigger").block_type(),
            "AnalogDiffTrigger"
        );
        assert_eq!(
            create_block("AnalogThresholdTrigger").block_type(),
            "AnalogThresholdTrigger"
        );
        assert_eq!(
            create_block("AnalogWatchdog").block_type(),
            "AnalogWatchdog"
        );
    }

    #[test]
    fn factory_maps_all_group_b_timer_types() {
        assert_eq!(create_block("Monoflop").block_type(), "Monoflop");
        assert_eq!(create_block("OnPulseDelay").block_type(), "OnPulseDelay");
        assert_eq!(create_block("OffDelay").block_type(), "OffDelay");
        assert_eq!(create_block("StairwayLS").block_type(), "StairwayLS");
        assert_eq!(create_block("PulseGen").block_type(), "PulseGen");
        assert_eq!(create_block("EdgeDetection").block_type(), "EdgeDetection");
        assert_eq!(create_block("OnDelay").block_type(), "OnDelay");
        assert_eq!(create_block("OnOffDelay").block_type(), "OnOffDelay");
        assert_eq!(create_block("RetOnDelay").block_type(), "RetOnDelay");
        assert_eq!(create_block("PWM").block_type(), "PWM");
        assert_eq!(
            create_block("EdgeWipingRelay").block_type(),
            "EdgeWipingRelay"
        );
        assert_eq!(
            create_block("SwitchingTimer").block_type(),
            "SwitchingTimer"
        );
    }

    #[test]
    fn factory_maps_all_group_b_state_types() {
        assert_eq!(create_block("Memory").block_type(), "Memory");
        assert_eq!(create_block("FlipFlop").block_type(), "FlipFlop");
        assert_eq!(create_block("RSFlipFlop").block_type(), "RSFlipFlop");
        assert_eq!(create_block("AMemory").block_type(), "AMemory");
        assert_eq!(create_block("PushButton").block_type(), "PushButton");
        assert_eq!(create_block("PushButton2").block_type(), "PushButton2");
        assert_eq!(create_block("Counter").block_type(), "Counter");
        assert_eq!(create_block("UpDownCounter").block_type(), "UpDownCounter");
        assert_eq!(create_block("HourCounter").block_type(), "HourCounter");
        assert_eq!(create_block("SampleHold").block_type(), "SampleHold");
        assert_eq!(create_block("Shift").block_type(), "Shift");
        assert_eq!(create_block("StateV").block_type(), "StateV");
        assert_eq!(create_block("State").block_type(), "State");
    }

    #[test]
    fn factory_maps_all_group_b_schedule_types() {
        assert_eq!(create_block("DayTimer").block_type(), "DayTimer");
        assert_eq!(create_block("AlarmClock").block_type(), "AlarmClock");
        assert_eq!(create_block("PulseAt").block_type(), "PulseAt");
        assert_eq!(create_block("Calendar").block_type(), "Calendar");
    }
}
