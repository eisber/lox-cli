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

passthrough_io_block!(
    /// Input reference — proxy that forwards a named input to the block graph.
    InputRef,
    "InputRef"
);

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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_ref_passthrough() {
        let mut block = InputRef;
        assert_eq!(block.eval(&[42.0], &[], 0.0, &[]), vec![42.0]);
        assert_eq!(block.eval(&[], &[], 0.0, &[]), vec![0.0]);
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
    fn factory_creates_all_io_types() {
        use crate::blocks::create_block;
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
        ] {
            let block = create_block(name);
            assert_eq!(block.block_type(), *name, "Factory mismatch for {name}");
        }
    }
}
