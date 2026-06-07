//! Group D — I/O blocks (references, KNX/EIB, virtual I/O).
//!
//! These are proxy/adapter blocks that bridge between the Loxone internal
//! wiring model and external bus systems (KNX/EIB) or virtual endpoints.
//! Most are pass-through by nature — they relay values without transformation.

use crate::blocks::Block;
use crate::types::Signal;

// ---------------------------------------------------------------------------
// Macro for simple pass-through I/O blocks
// ---------------------------------------------------------------------------

macro_rules! passthrough_io_block {
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

// ---------------------------------------------------------------------------
// Input/Output references — wire proxies
// ---------------------------------------------------------------------------

/// Input reference — proxy that forwards named inputs to the block graph.
/// I→Q (digital) and AI→AQ (analog).
#[derive(Clone, Copy)]
pub struct InputRef;

impl Block for InputRef {
    fn eval(
        &mut self,
        inputs: &[Signal],
        _params: &[Signal],
        _dt: f64,
        _prev: &[Signal],
    ) -> Vec<Signal> {
        let i = inputs.first().copied().unwrap_or(0.0);
        let ai = inputs.get(1).copied().unwrap_or(0.0);
        vec![i, ai]
    }

    fn block_type(&self) -> &str {
        "InputRef"
    }

    fn state(&self) -> Option<Vec<u8>> {
        None
    }

    fn restore(&mut self, _state: &[u8]) {}
}

passthrough_io_block!(
    /// Output reference — proxy that forwards a block output to a named output.
    OutputRef,
    "OutputRef"
);

passthrough_io_block!(
    /// Output reference (Loxone Miniserver) — pass-through proxy.
    OutputRefLM,
    "OutputRefLM"
);

// ---------------------------------------------------------------------------
// KNX/EIB blocks (EIBJalousie is in controllers.rs with full jalousie logic)
// ---------------------------------------------------------------------------

passthrough_io_block!(
    /// KNX push button — pass-through digital signal.
    EIBPush,
    "EIBPush"
);

passthrough_io_block!(
    /// KNX sensor — pass-through analog/digital value.
    EIBsensor,
    "EIBsensor"
);

passthrough_io_block!(
    /// KNX actor — pass-through output command.
    EIBactor,
    "EIBactor"
);

passthrough_io_block!(
    /// KNX extended actor — pass-through output command.
    EIBextactor,
    "EIBextactor"
);

passthrough_io_block!(
    /// KNX dimmer — pass analog dim level (0.0–1.0).
    EibDimmer,
    "EibDimmer"
);

// ---------------------------------------------------------------------------
// Virtual I/O
// ---------------------------------------------------------------------------

passthrough_io_block!(
    /// Virtual input — external value injection point (pass-through).
    VirtualIn,
    "VirtualIn"
);

passthrough_io_block!(
    /// Virtual output — external value export point (pass-through).
    VirtualOut,
    "VirtualOut"
);

passthrough_io_block!(
    /// Virtual state — external state variable (pass-through).
    VirtualState,
    "VirtualState"
);

// ---------------------------------------------------------------------------
// Tree / Air hardware I/O
// ---------------------------------------------------------------------------

passthrough_io_block!(
    /// Tree digital sensor — wired digital input (pass-through).
    TreeSensor,
    "TreeSensor"
);

passthrough_io_block!(
    /// Tree analog sensor — wired analog input (pass-through).
    TreeAsensor,
    "TreeAsensor"
);

passthrough_io_block!(
    /// Air digital sensor — wireless digital input (pass-through).
    LoxAIRsensor,
    "LoxAIRsensor"
);

passthrough_io_block!(
    /// Air analog sensor — wireless analog input (pass-through).
    LoxAIRAsensor,
    "LoxAIRAsensor"
);

// ---------------------------------------------------------------------------
// Tree / Air hardware actors (device outputs)
//
// Actors are terminal: in real configs their `I` input carries the value
// delivered to the physical device and they expose no output. To let a sim
// assert on what the actor would emit, these mirror the input onto a
// synthesized output (`AQ` for the analog `A`-actors, `Q` for digital actors —
// see `block_signature`). They carry a real `block_type()` so they classify as
// Simulated (not an unreliable PassThrough fallback).
// ---------------------------------------------------------------------------

passthrough_io_block!(
    /// Air analog actor — wireless analog output (input mirrored to AQ).
    LoxAIRAactor,
    "LoxAIRAactor"
);

passthrough_io_block!(
    /// Air digital actor — wireless digital output (input mirrored to Q).
    LoxAIRactor,
    "LoxAIRactor"
);

passthrough_io_block!(
    /// Tree analog actor — wired analog output (input mirrored to AQ).
    TreeAactor,
    "TreeAactor"
);

passthrough_io_block!(
    /// Tree digital actor — wired digital output (input mirrored to Q).
    TreeActor,
    "TreeActor"
);

// ---------------------------------------------------------------------------
// Weather / generic (MQTT) sources and sinks
//
// `WeatherData` (Loxone weather service) and `GenTSensor` (generic MQTT
// subscription) are value *sources*: each instance represents one quantity
// (wind speed, temperature, sunshine, …) on its analog output. A sim injects a
// reading with `set_input("<name>.AQ", value)`; the override persists across
// ticks because the real `block_type()` is not in the source-passthrough
// exclusion list. `GenTActor` (generic MQTT publish) is a *sink* that mirrors
// its `Text` input onto `AQ` so the published value can be asserted.
// ---------------------------------------------------------------------------

passthrough_io_block!(
    /// Loxone weather-service data point — analog source on `AQ`.
    WeatherData,
    "WeatherData"
);

passthrough_io_block!(
    /// Generic MQTT sensor subscription — analog source on `AQ`.
    GenTSensor,
    "GenTSensor"
);

passthrough_io_block!(
    /// Generic MQTT publish actor — mirrors `Text` input onto `AQ`.
    GenTActor,
    "GenTActor"
);

// ---------------------------------------------------------------------------
// Hardware analog/digital inputs
// ---------------------------------------------------------------------------

passthrough_io_block!(
    /// Analog voltage input — hardware analog reading (pass-through).
    VoltageIn,
    "VoltageIn"
);

passthrough_io_block!(
    /// Digital input — hardware digital state (pass-through).
    DigitalIn,
    "DigitalIn"
);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::create_block;
    #[test]
    fn input_ref_passthrough() {
        let mut block = InputRef;
        // I=42, AI=0 → Q=42, AQ=0
        assert_eq!(block.eval(&[42.0], &[], 0.0, &[]), vec![42.0, 0.0]);
        // I=0, AI=99 → Q=0, AQ=99
        assert_eq!(block.eval(&[0.0, 99.0], &[], 0.0, &[]), vec![0.0, 99.0]);
        // empty → Q=0, AQ=0
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0, 0.0]);
    }

    #[test]
    fn output_ref_passthrough() {
        let mut block = OutputRef;
        assert_eq!(block.eval(&[7.5], &[], 0.0, &[]), vec![7.5]);
    }

    #[test]
    fn eib_dimmer_passthrough() {
        let mut block = EibDimmer;
        assert_eq!(block.eval(&[0.75], &[], 0.0, &[]), vec![0.75]);
    }

    #[test]
    fn virtual_in_out_state_passthrough() {
        let mut vi = VirtualIn;
        let mut vo = VirtualOut;
        let mut vs = VirtualState;
        assert_eq!(vi.eval(&[1.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(vo.eval(&[2.0], &[], 0.0, &[]), vec![2.0]);
        assert_eq!(vs.eval(&[3.0], &[], 0.0, &[]), vec![3.0]);
    }

    #[test]
    fn voltage_in_passthrough() {
        let mut block = VoltageIn;
        assert_eq!(block.eval(&[3.3], &[], 0.0, &[]), vec![3.3]);
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn digital_in_passthrough() {
        let mut block = DigitalIn;
        assert_eq!(block.eval(&[1.0], &[], 0.0, &[]), vec![1.0]);
        assert_eq!(block.eval(&[0.0], &[], 0.0, &[]), vec![0.0]);
    }

    #[test]
    fn factory_creates_all_io_types() {
        for name in &[
            "InputRef",
            "OutputRef",
            "OutputRefLM",
            "EIBPush",
            "EIBsensor",
            "EIBactor",
            "EIBextactor",
            "EibDimmer",
            "VirtualIn",
            "VirtualOut",
            "VirtualState",
            "VoltageIn",
            "DigitalIn",
            "LoxAIRAactor",
            "LoxAIRactor",
            "TreeAactor",
            "TreeActor",
            "WeatherData",
            "GenTSensor",
            "GenTActor",
        ] {
            let block = create_block(name);
            assert_eq!(block.block_type(), *name, "Factory mismatch for {name}");
        }
    }

    #[test]
    fn device_actors_mirror_input_to_output() {
        // Actors are terminal sinks; the `I` input value is mirrored onto the
        // synthesized output so a sim can assert what the device would emit.
        for name in &["LoxAIRAactor", "LoxAIRactor", "TreeAactor", "TreeActor"] {
            let mut block = create_block(name);
            assert_eq!(block.eval(&[42.0], &[], 0.0, &[]), vec![42.0], "{name}");
            assert_eq!(block.eval(&[0.0], &[], 0.0, &[]), vec![0.0], "{name}");
        }
    }

    #[test]
    fn weather_and_generic_sources_are_simulated() {
        use crate::blocks::{block_support, BlockSupport};
        // Sources/sinks must classify as Simulated (not unreliable PassThrough)
        // so injected output overrides persist and `sim check` does not flag
        // them.
        for name in &["WeatherData", "GenTSensor", "GenTActor"] {
            assert_eq!(create_block(name).block_type(), *name);
            assert_eq!(
                block_support(name),
                BlockSupport::Simulated,
                "{name} should be Simulated"
            );
        }
    }
}
