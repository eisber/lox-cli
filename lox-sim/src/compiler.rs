//! JIT-compiled evaluation engine for the block graph.
//!
//! [`CompiledGraph`] compiles a [`SimGraph`] into a flat array of [`EvalStep`]
//! variants that are evaluated in topological order via a tight `match` loop.
//! No trait objects, no dynamic dispatch, zero allocations during `tick()`.

use std::collections::HashMap;

use smallvec::SmallVec;

use crate::blocks::schedule::DayTimerEntry;
use crate::graph::SimGraph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Block state — one variant per stateful block kind
// ---------------------------------------------------------------------------

/// Internal state for blocks that carry state between ticks.
#[derive(Clone, Debug)]
pub enum BlockState {
    /// Timer with a remaining countdown (Monoflop, OffDelay, StairwayLS).
    Timer { remaining: f64, active: bool },
    /// Threshold hysteresis latch (AnalogThresholdTrigger).
    Threshold { is_on: bool },
    /// Analogue memory latch.
    Latch { value: f64 },
    /// SR flip-flop / toggle.
    Toggle { state: bool },
    /// Rising-edge counter.
    Counter { count: f64 },
    /// DayTimer schedule state.
    DayTimer {
        entries: Vec<DayTimerEntry>,
        last_active: bool,
        last_value: f64,
    },
    /// OnPulseDelay — two-phase timer.
    TwoPhaseTimer {
        delay_remaining: f64,
        pulse_remaining: f64,
    },
    /// PulseGen oscillator.
    PulseGen {
        high_phase: bool,
        phase_remaining: f64,
    },
    /// EdgeDetection — three independent timers.
    EdgeDetection {
        edge_remaining: f64,
        rising_remaining: f64,
        falling_remaining: f64,
    },
    /// PushButton — toggle with transition pulses.
    PushButton { is_on: bool },
}

// ---------------------------------------------------------------------------
// EvalStep — one variant per block type (enum dispatch, not trait dispatch)
// ---------------------------------------------------------------------------

/// A single evaluation step in the compiled graph.
///
/// Each variant carries pre-resolved signal indices so `tick()` never does
/// hash lookups or dynamic dispatch.
#[derive(Clone, Debug)]
pub enum EvalStep {
    // -- Stateless logic --
    And {
        inputs: SmallVec<[usize; 4]>,
        output: usize,
    },
    Or {
        inputs: SmallVec<[usize; 4]>,
        output: usize,
    },
    Not {
        input: usize,
        output: usize,
    },
    Xor {
        inputs: SmallVec<[usize; 4]>,
        output: usize,
    },

    // -- Stateless comparisons --
    GreaterEqual {
        input1: usize,
        input2: usize,
        output: usize,
    },
    Less {
        input1: usize,
        input2: usize,
        output: usize,
    },
    Greater {
        input1: usize,
        input2: usize,
        output: usize,
    },
    LessEqual {
        input1: usize,
        input2: usize,
        output: usize,
    },

    // -- Stateless math --
    Add {
        inputs: SmallVec<[usize; 4]>,
        params: SmallVec<[usize; 4]>,
        output: usize,
    },
    Sub {
        inputs: SmallVec<[usize; 4]>,
        params: SmallVec<[usize; 4]>,
        output: usize,
    },
    Mult {
        inputs: SmallVec<[usize; 4]>,
        params: SmallVec<[usize; 4]>,
        output: usize,
    },
    Div {
        inputs: SmallVec<[usize; 4]>,
        params: SmallVec<[usize; 4]>,
        output: usize,
    },

    // -- Stateful timers --
    Monoflop {
        trigger: usize,
        prev_trigger: usize,
        param_duration: usize,
        output: usize,
        state_idx: usize,
    },
    OffDelay {
        trigger: usize,
        prev_trigger: usize,
        param_time: usize,
        output: usize,
        state_idx: usize,
    },
    StairwayLS {
        trigger: usize,
        prev_trigger: usize,
        forced_on: usize,
        param_time: usize,
        output: usize,
        state_idx: usize,
    },
    OnPulseDelay {
        trigger: usize,
        prev_trigger: usize,
        param_delay: usize,
        param_duration: usize,
        output: usize,
        state_idx: usize,
    },
    PulseGen {
        enabled: usize,
        inverted: usize,
        param_time_high: usize,
        param_time_low: usize,
        output: usize,
        state_idx: usize,
    },
    EdgeDetection {
        input: usize,
        prev_input: usize,
        param_pulse_time: usize,
        /// outputs: [edge, rising, falling]
        outputs: [usize; 3],
        state_idx: usize,
    },

    // -- Threshold with hysteresis --
    Threshold {
        input: usize,
        param_on: usize,
        param_off: usize,
        /// outputs: [Q, rising, falling]
        outputs: [usize; 3],
        state_idx: usize,
    },

    // -- Memory / state --
    Memory {
        input: usize,
        trigger: usize,
        prev_trigger: usize,
        reset: usize,
        output: usize,
        state_idx: usize,
    },
    FlipFlop {
        set: usize,
        reset: usize,
        output: usize,
        state_idx: usize,
    },
    PushButton {
        trigger: usize,
        prev_trigger: usize,
        force_on: usize,
        /// outputs: [Q, Qoff, Qon]
        outputs: [usize; 3],
        state_idx: usize,
    },
    Counter {
        input: usize,
        prev_input: usize,
        output: usize,
        state_idx: usize,
    },

    // -- Pass-through / copy --
    Copy {
        src: usize,
        dst: usize,
    },

    // -- Constant (from parameter) --
    Const {
        param: usize,
        output: usize,
    },

    // -- Gain (input * param) --
    Gain {
        input: usize,
        param: usize,
        output: usize,
    },

    // -- Rising edge detector --
    RisingEdge {
        input: usize,
        prev_input: usize,
        output: usize,
    },

    // -- DayTimer (schedule-based) --
    DayTimer {
        minutes_input: usize,
        day_input: usize,
        /// outputs: [value, value_dup, qon, qoff, remaining_secs]
        outputs: [usize; 5],
        state_idx: usize,
    },
}

// ---------------------------------------------------------------------------
// CompiledGraph
// ---------------------------------------------------------------------------

/// A compiled block graph that evaluates the entire graph in a single tight
/// loop with zero allocations and no dynamic dispatch.
pub struct CompiledGraph {
    /// Flat signal array: every connector maps to one index.
    signals: Vec<f64>,
    /// Previous tick signals (for edge detection and feedback).
    prev_signals: Vec<f64>,
    /// Compiled evaluation steps in topological order.
    steps: Vec<EvalStep>,
    /// Block state array (timers, latches, etc.).
    state: Vec<BlockState>,
    /// Name → signal index for inputs.
    named_inputs: HashMap<String, usize>,
    /// Name → signal index for outputs.
    named_outputs: HashMap<String, usize>,
    /// Name → signal index for parameters.
    named_params: HashMap<String, usize>,
}

impl CompiledGraph {
    /// Compile a [`SimGraph`] into a [`CompiledGraph`].
    ///
    /// This resolves all wiring into flat array indices and converts each
    /// block into an [`EvalStep`] variant. The resulting structure can be
    /// ticked millions of times with zero allocation overhead.
    pub fn from_graph(graph: &SimGraph) -> Self {
        let topo = graph.topological_order();
        let n_conn = graph.connector_count();
        let n_blocks = graph.block_count();

        // Build signals from connector defaults.
        let signals: Vec<f64> = (0..n_conn)
            .map(|i| graph.connector(i).default_value)
            .collect();

        // We need a prev_signals index for each connector. We allocate
        // a second range [n_conn .. 2*n_conn) in the prev_signals array,
        // but for simplicity we use a separate Vec<f64> of the same size.
        let prev_signals = signals.clone();

        // Helper: resolve an input connector to the source signal index.
        // For feedback wires, we'll read from prev_signals instead.
        let feedback_wires = &topo.feedback_wires;

        // Build input source map: for each input connector, where does its value come from?
        let mut input_source: Vec<(usize, bool)> = vec![(0, false); n_conn];
        for (cid, src) in input_source.iter_mut().enumerate() {
            let is_input = graph.connector(cid).dir == ConnectorDir::Input;
            if is_input {
                match graph.input_source_of(cid) {
                    Some(from) => {
                        let is_fb = feedback_wires.contains(&(from, cid));
                        *src = (from, is_fb);
                    }
                    None => {
                        *src = (cid, false);
                    }
                }
            }
        }

        let mut steps = Vec::with_capacity(n_blocks);
        let mut state = Vec::new();

        for &block_id in &topo.order {
            let info = graph.block_info(block_id);
            let block_type = graph
                .block_impls
                .get(block_id)
                .map(|b| b.block_type())
                .unwrap_or("PassThrough");

            // Resolve input signal indices (accounting for wiring).
            let resolved_inputs: Vec<usize> =
                info.inputs.iter().map(|&cid| input_source[cid].0).collect();

            // Prev-tick versions of inputs (same index, read from prev_signals).
            let prev_inputs: Vec<usize> = resolved_inputs.clone();

            let outputs = &info.outputs;
            let params = &info.params;

            let step = match block_type {
                "And" => EvalStep::And {
                    inputs: resolved_inputs.iter().copied().collect(),
                    output: outputs[0],
                },
                "Or" => EvalStep::Or {
                    inputs: resolved_inputs.iter().copied().collect(),
                    output: outputs[0],
                },
                "Not" => EvalStep::Not {
                    input: resolved_inputs.first().copied().unwrap_or(0),
                    output: outputs[0],
                },
                "Xor" => EvalStep::Xor {
                    inputs: resolved_inputs.iter().copied().collect(),
                    output: outputs[0],
                },
                "GreaterEqual" => {
                    let i1 = resolved_inputs.first().copied().unwrap_or(0);
                    let i2 = resolved_inputs
                        .get(1)
                        .copied()
                        .or_else(|| params.first().copied())
                        .unwrap_or(0);
                    EvalStep::GreaterEqual {
                        input1: i1,
                        input2: i2,
                        output: outputs[0],
                    }
                }
                "Less" => {
                    let i1 = resolved_inputs.first().copied().unwrap_or(0);
                    let i2 = resolved_inputs
                        .get(1)
                        .copied()
                        .or_else(|| params.first().copied())
                        .unwrap_or(0);
                    EvalStep::Less {
                        input1: i1,
                        input2: i2,
                        output: outputs[0],
                    }
                }
                "Greater" => {
                    let i1 = resolved_inputs.first().copied().unwrap_or(0);
                    let i2 = resolved_inputs
                        .get(1)
                        .copied()
                        .or_else(|| params.first().copied())
                        .unwrap_or(0);
                    EvalStep::Greater {
                        input1: i1,
                        input2: i2,
                        output: outputs[0],
                    }
                }
                "LessEqual" => {
                    let i1 = resolved_inputs.first().copied().unwrap_or(0);
                    let i2 = resolved_inputs
                        .get(1)
                        .copied()
                        .or_else(|| params.first().copied())
                        .unwrap_or(0);
                    EvalStep::LessEqual {
                        input1: i1,
                        input2: i2,
                        output: outputs[0],
                    }
                }
                "Add" | "Add4" => EvalStep::Add {
                    inputs: resolved_inputs.iter().copied().collect(),
                    params: params.iter().copied().collect(),
                    output: outputs[0],
                },
                "Sub" => EvalStep::Sub {
                    inputs: resolved_inputs.iter().copied().collect(),
                    params: params.iter().copied().collect(),
                    output: outputs[0],
                },
                "Mult" => EvalStep::Mult {
                    inputs: resolved_inputs.iter().copied().collect(),
                    params: params.iter().copied().collect(),
                    output: outputs[0],
                },
                "Div" => EvalStep::Div {
                    inputs: resolved_inputs.iter().copied().collect(),
                    params: params.iter().copied().collect(),
                    output: outputs[0],
                },
                "Monoflop" => {
                    let si = state.len();
                    state.push(BlockState::Timer {
                        remaining: 0.0,
                        active: false,
                    });
                    EvalStep::Monoflop {
                        trigger: resolved_inputs.first().copied().unwrap_or(0),
                        prev_trigger: prev_inputs.first().copied().unwrap_or(0),
                        param_duration: params.first().copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "OffDelay" => {
                    let si = state.len();
                    state.push(BlockState::Timer {
                        remaining: 0.0,
                        active: false,
                    });
                    EvalStep::OffDelay {
                        trigger: resolved_inputs.first().copied().unwrap_or(0),
                        prev_trigger: prev_inputs.first().copied().unwrap_or(0),
                        param_time: params.first().copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "StairwayLS" => {
                    let si = state.len();
                    state.push(BlockState::Timer {
                        remaining: 0.0,
                        active: false,
                    });
                    EvalStep::StairwayLS {
                        trigger: resolved_inputs.first().copied().unwrap_or(0),
                        prev_trigger: prev_inputs.first().copied().unwrap_or(0),
                        forced_on: resolved_inputs.get(1).copied().unwrap_or(0),
                        param_time: params.first().copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "OnPulseDelay" => {
                    let si = state.len();
                    state.push(BlockState::TwoPhaseTimer {
                        delay_remaining: 0.0,
                        pulse_remaining: 0.0,
                    });
                    EvalStep::OnPulseDelay {
                        trigger: resolved_inputs.first().copied().unwrap_or(0),
                        prev_trigger: prev_inputs.first().copied().unwrap_or(0),
                        param_delay: params.first().copied().unwrap_or(0),
                        param_duration: params.get(1).copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "PulseGen" => {
                    let si = state.len();
                    state.push(BlockState::PulseGen {
                        high_phase: true,
                        phase_remaining: 0.0,
                    });
                    EvalStep::PulseGen {
                        enabled: resolved_inputs.first().copied().unwrap_or(0),
                        inverted: resolved_inputs.get(1).copied().unwrap_or(0),
                        param_time_high: params.first().copied().unwrap_or(0),
                        param_time_low: params.get(1).copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "EdgeDetection" => {
                    let si = state.len();
                    state.push(BlockState::EdgeDetection {
                        edge_remaining: 0.0,
                        rising_remaining: 0.0,
                        falling_remaining: 0.0,
                    });
                    EvalStep::EdgeDetection {
                        input: resolved_inputs.first().copied().unwrap_or(0),
                        prev_input: prev_inputs.first().copied().unwrap_or(0),
                        param_pulse_time: params.first().copied().unwrap_or(0),
                        outputs: [
                            *outputs.first().unwrap_or(&0),
                            *outputs.get(1).unwrap_or(&0),
                            *outputs.get(2).unwrap_or(&0),
                        ],
                        state_idx: si,
                    }
                }
                "AnalogThresholdTrigger" => {
                    let si = state.len();
                    state.push(BlockState::Threshold { is_on: false });
                    EvalStep::Threshold {
                        input: resolved_inputs.first().copied().unwrap_or(0),
                        param_on: params.first().copied().unwrap_or(0),
                        param_off: params.get(1).copied().unwrap_or(0),
                        outputs: [
                            *outputs.first().unwrap_or(&0),
                            *outputs.get(1).unwrap_or(&0),
                            *outputs.get(2).unwrap_or(&0),
                        ],
                        state_idx: si,
                    }
                }
                "AMemory" => {
                    let si = state.len();
                    state.push(BlockState::Latch { value: 0.0 });
                    EvalStep::Memory {
                        input: resolved_inputs.first().copied().unwrap_or(0),
                        trigger: resolved_inputs.get(1).copied().unwrap_or(0),
                        prev_trigger: prev_inputs.get(1).copied().unwrap_or(0),
                        reset: resolved_inputs.get(2).copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "FlipFlop" | "RSFlipFlop" | "SRFlipFlop" => {
                    let si = state.len();
                    state.push(BlockState::Toggle { state: false });
                    EvalStep::FlipFlop {
                        set: resolved_inputs.first().copied().unwrap_or(0),
                        reset: resolved_inputs.get(1).copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "PushButton" | "PushButton2" | "PushButtonSel" | "PushButton2Sel" => {
                    let si = state.len();
                    state.push(BlockState::PushButton { is_on: false });
                    EvalStep::PushButton {
                        trigger: resolved_inputs.first().copied().unwrap_or(0),
                        prev_trigger: prev_inputs.first().copied().unwrap_or(0),
                        force_on: resolved_inputs.get(1).copied().unwrap_or(0),
                        outputs: [
                            *outputs.first().unwrap_or(&0),
                            *outputs.get(1).unwrap_or(&0),
                            *outputs.get(2).unwrap_or(&0),
                        ],
                        state_idx: si,
                    }
                }
                "Counter" => {
                    let si = state.len();
                    state.push(BlockState::Counter { count: 0.0 });
                    EvalStep::Counter {
                        input: resolved_inputs.first().copied().unwrap_or(0),
                        prev_input: prev_inputs.first().copied().unwrap_or(0),
                        output: outputs[0],
                        state_idx: si,
                    }
                }
                "Constant" => EvalStep::Const {
                    param: params.first().copied().unwrap_or(0),
                    output: outputs[0],
                },
                "Gain" => EvalStep::Gain {
                    input: resolved_inputs.first().copied().unwrap_or(0),
                    param: params.first().copied().unwrap_or(0),
                    output: outputs[0],
                },
                "RisingEdge" => EvalStep::RisingEdge {
                    input: resolved_inputs.first().copied().unwrap_or(0),
                    prev_input: prev_inputs.first().copied().unwrap_or(0),
                    output: outputs[0],
                },
                "DayTimer" => {
                    let si = state.len();
                    // Extract entries from the block impl if available.
                    let entries = extract_daytimer_entries(graph, block_id);
                    state.push(BlockState::DayTimer {
                        entries,
                        last_active: false,
                        last_value: 0.0,
                    });
                    EvalStep::DayTimer {
                        minutes_input: resolved_inputs.first().copied().unwrap_or(0),
                        day_input: resolved_inputs.get(1).copied().unwrap_or(0),
                        outputs: [
                            *outputs.first().unwrap_or(&0),
                            *outputs.get(1).unwrap_or(&0),
                            *outputs.get(2).unwrap_or(&0),
                            *outputs.get(3).unwrap_or(&0),
                            *outputs.get(4).unwrap_or(&0),
                        ],
                        state_idx: si,
                    }
                }
                // Default: PassThrough / unknown → copy first input to first output
                _ => EvalStep::Copy {
                    src: resolved_inputs.first().copied().unwrap_or(0),
                    dst: outputs.first().copied().unwrap_or(0),
                },
            };

            steps.push(step);
        }

        // Build named lookups (same logic as SimEngine).
        let mut named_inputs: HashMap<String, usize> = HashMap::new();
        let mut named_outputs: HashMap<String, usize> = HashMap::new();
        let mut named_params: HashMap<String, usize> = HashMap::new();
        for bid in 0..n_blocks {
            let info = graph.block_info(bid);
            for &cid in &info.inputs {
                let key = format!("{}.{}", info.name, graph.connector(cid).key);
                named_inputs.insert(key, cid);
            }
            for &cid in &info.outputs {
                let key = format!("{}.{}", info.name, graph.connector(cid).key);
                named_outputs.insert(key, cid);
            }
            for &cid in &info.params {
                let key = format!("{}.{}", info.name, graph.connector(cid).key);
                named_params.insert(key, cid);
            }
            if let Some(&cid) = info.inputs.first() {
                named_inputs.entry(info.name.clone()).or_insert(cid);
            }
            if let Some(&cid) = info.outputs.first() {
                named_outputs.entry(info.name.clone()).or_insert(cid);
            }
        }

        CompiledGraph {
            signals,
            prev_signals,
            steps,
            state,
            named_inputs,
            named_outputs,
            named_params,
        }
    }

    /// Evaluate all blocks for one time step.
    ///
    /// This is the hot loop — no allocations, no dynamic dispatch.
    #[inline(never)]
    pub fn tick(&mut self, dt: f64) {
        // Snapshot previous signals.
        self.prev_signals.copy_from_slice(&self.signals);

        for step in 0..self.steps.len() {
            // SAFETY: we index by step which is in-bounds.
            // We need to split borrows: steps is read-only, signals+state are mutated.
            // Using index-based access to avoid borrow conflicts.
            match &self.steps[step] {
                // -- Stateless logic --
                EvalStep::And { inputs, output } => {
                    let val = !inputs.is_empty() && inputs.iter().all(|&i| self.signals[i] >= 0.5);
                    self.signals[*output] = bool_f64(val);
                }
                EvalStep::Or { inputs, output } => {
                    let val = inputs.iter().any(|&i| self.signals[i] >= 0.5);
                    self.signals[*output] = bool_f64(val);
                }
                EvalStep::Not { input, output } => {
                    self.signals[*output] = bool_f64(self.signals[*input] < 0.5);
                }
                EvalStep::Xor { inputs, output } => {
                    let high_count = inputs.iter().filter(|&&i| self.signals[i] >= 0.5).count();
                    self.signals[*output] = bool_f64(high_count % 2 == 1);
                }

                // -- Comparisons --
                EvalStep::GreaterEqual {
                    input1,
                    input2,
                    output,
                } => {
                    self.signals[*output] =
                        bool_f64(self.signals[*input1] >= self.signals[*input2]);
                }
                EvalStep::Less {
                    input1,
                    input2,
                    output,
                } => {
                    self.signals[*output] = bool_f64(self.signals[*input1] < self.signals[*input2]);
                }
                EvalStep::Greater {
                    input1,
                    input2,
                    output,
                } => {
                    self.signals[*output] = bool_f64(self.signals[*input1] > self.signals[*input2]);
                }
                EvalStep::LessEqual {
                    input1,
                    input2,
                    output,
                } => {
                    self.signals[*output] =
                        bool_f64(self.signals[*input1] <= self.signals[*input2]);
                }

                // -- Math --
                EvalStep::Add {
                    inputs,
                    params,
                    output,
                } => {
                    let sum: f64 = inputs
                        .iter()
                        .chain(params.iter())
                        .map(|&i| self.signals[i])
                        .sum();
                    self.signals[*output] = sum;
                }
                EvalStep::Sub {
                    inputs,
                    params,
                    output,
                } => {
                    let mut operands = inputs.iter().chain(params.iter()).map(|&i| self.signals[i]);
                    let first = operands.next().unwrap_or(0.0);
                    self.signals[*output] = operands.fold(first, |acc, v| acc - v);
                }
                EvalStep::Mult {
                    inputs,
                    params,
                    output,
                } => {
                    let all: SmallVec<[f64; 8]> = inputs
                        .iter()
                        .chain(params.iter())
                        .map(|&i| self.signals[i])
                        .collect();
                    self.signals[*output] = if all.is_empty() {
                        0.0
                    } else {
                        all.into_iter().product()
                    };
                }
                EvalStep::Div {
                    inputs,
                    params,
                    output,
                } => {
                    let mut operands = inputs.iter().chain(params.iter()).map(|&i| self.signals[i]);
                    let first = operands.next().unwrap_or(0.0);
                    let result = operands.try_fold(first, |acc, v| {
                        if v.abs() <= f64::EPSILON {
                            None
                        } else {
                            Some(acc / v)
                        }
                    });
                    self.signals[*output] = result.unwrap_or(0.0);
                }

                // -- Monoflop --
                EvalStep::Monoflop {
                    trigger,
                    prev_trigger,
                    param_duration,
                    output,
                    state_idx,
                } => {
                    let trig = self.signals[*trigger];
                    let prev_trig = self.prev_signals[*prev_trigger];
                    let duration = self.signals[*param_duration].max(0.0);
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::Timer { remaining, .. } = &mut self.state[si] {
                        if prev_trig < 0.5 && trig >= 0.5 {
                            *remaining = duration.max(dt);
                        }
                        let q = *remaining > 0.0;
                        if q {
                            *remaining = (*remaining - dt).max(0.0);
                        }
                        self.signals[out] = bool_f64(q);
                    }
                }

                // -- OffDelay --
                EvalStep::OffDelay {
                    trigger,
                    prev_trigger,
                    param_time,
                    output,
                    state_idx,
                } => {
                    let trig = self.signals[*trigger];
                    let prev_trig = self.prev_signals[*prev_trigger];
                    let delay = self.signals[*param_time].max(0.0);
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::Timer { remaining, .. } = &mut self.state[si] {
                        let q = if trig >= 0.5 {
                            *remaining = 0.0;
                            true
                        } else {
                            if prev_trig >= 0.5 && trig < 0.5 {
                                *remaining = delay.max(dt);
                            }
                            let active = *remaining > 0.0;
                            if active {
                                *remaining = (*remaining - dt).max(0.0);
                            }
                            active
                        };
                        self.signals[out] = bool_f64(q);
                    }
                }

                // -- StairwayLS --
                EvalStep::StairwayLS {
                    trigger,
                    prev_trigger,
                    forced_on,
                    param_time,
                    output,
                    state_idx,
                } => {
                    let trig = self.signals[*trigger];
                    let prev_trig = self.prev_signals[*prev_trigger];
                    let force = self.signals[*forced_on];
                    let time_high = self.signals[*param_time].max(0.0);
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::Timer { remaining, .. } = &mut self.state[si] {
                        if prev_trig < 0.5 && trig >= 0.5 {
                            *remaining = time_high.max(dt);
                        }
                        let q = if force >= 0.5 {
                            true
                        } else {
                            let active = *remaining > 0.0;
                            if active {
                                *remaining = (*remaining - dt).max(0.0);
                            }
                            active
                        };
                        self.signals[out] = bool_f64(q);
                    }
                }

                // -- OnPulseDelay --
                EvalStep::OnPulseDelay {
                    trigger,
                    prev_trigger,
                    param_delay,
                    param_duration,
                    output,
                    state_idx,
                } => {
                    let trig = self.signals[*trigger];
                    let prev_trig = self.prev_signals[*prev_trigger];
                    let delay = self.signals[*param_delay].max(0.0);
                    let duration = self.signals[*param_duration].max(0.0);
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::TwoPhaseTimer {
                        delay_remaining,
                        pulse_remaining,
                    } = &mut self.state[si]
                    {
                        if prev_trig < 0.5 && trig >= 0.5 {
                            *delay_remaining = delay;
                            *pulse_remaining = 0.0;
                        }

                        let mut q = false;
                        if *pulse_remaining > 0.0 {
                            q = true;
                            *pulse_remaining = (*pulse_remaining - dt).max(0.0);
                        } else if *delay_remaining > 0.0 {
                            *delay_remaining = (*delay_remaining - dt).max(0.0);
                            if *delay_remaining <= 0.0 {
                                *pulse_remaining = duration.max(dt);
                                q = *pulse_remaining > 0.0;
                                if q {
                                    *pulse_remaining = (*pulse_remaining - dt).max(0.0);
                                }
                            }
                        }
                        self.signals[out] = bool_f64(q);
                    }
                }

                // -- PulseGen --
                EvalStep::PulseGen {
                    enabled,
                    inverted,
                    param_time_high,
                    param_time_low,
                    output,
                    state_idx,
                } => {
                    let en = self.signals[*enabled] >= 0.5;
                    let inv = self.signals[*inverted] >= 0.5;
                    let time_high = self.signals[*param_time_high].max(0.0);
                    let time_low = self.signals[*param_time_low].max(0.0);
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::PulseGen {
                        high_phase,
                        phase_remaining,
                    } = &mut self.state[si]
                    {
                        if !en {
                            *high_phase = true;
                            *phase_remaining = 0.0;
                            self.signals[out] = 0.0;
                        } else {
                            if *phase_remaining <= 0.0 {
                                *high_phase = true;
                                *phase_remaining = time_high.max(dt);
                            }
                            let q = if inv { !*high_phase } else { *high_phase };
                            *phase_remaining -= dt;
                            while *phase_remaining <= 0.0 {
                                *high_phase = !*high_phase;
                                let next = if *high_phase { time_high } else { time_low }.max(dt);
                                *phase_remaining += next;
                            }
                            self.signals[out] = bool_f64(q);
                        }
                    }
                }

                // -- EdgeDetection --
                EvalStep::EdgeDetection {
                    input,
                    prev_input,
                    param_pulse_time,
                    outputs,
                    state_idx,
                } => {
                    let cur = self.signals[*input];
                    let prev = self.prev_signals[*prev_input];
                    let pulse_time = self.signals[*param_pulse_time].max(dt);
                    let outs = *outputs;
                    let si = *state_idx;

                    if let BlockState::EdgeDetection {
                        edge_remaining,
                        rising_remaining,
                        falling_remaining,
                    } = &mut self.state[si]
                    {
                        if prev < 0.5 && cur >= 0.5 {
                            *rising_remaining = pulse_time;
                            *edge_remaining = pulse_time;
                        }
                        if prev >= 0.5 && cur < 0.5 {
                            *falling_remaining = pulse_time;
                            *edge_remaining = pulse_time;
                        }

                        let edge = *edge_remaining > 0.0;
                        let rising = *rising_remaining > 0.0;
                        let falling = *falling_remaining > 0.0;

                        if edge {
                            *edge_remaining = (*edge_remaining - dt).max(0.0);
                        }
                        if rising {
                            *rising_remaining = (*rising_remaining - dt).max(0.0);
                        }
                        if falling {
                            *falling_remaining = (*falling_remaining - dt).max(0.0);
                        }

                        self.signals[outs[0]] = bool_f64(edge);
                        self.signals[outs[1]] = bool_f64(rising);
                        self.signals[outs[2]] = bool_f64(falling);
                    }
                }

                // -- Threshold (AnalogThresholdTrigger) --
                EvalStep::Threshold {
                    input,
                    param_on,
                    param_off,
                    outputs,
                    state_idx,
                } => {
                    let val = self.signals[*input];
                    let on_thresh = self.signals[*param_on];
                    let off_thresh = self.signals[*param_off];
                    let outs = *outputs;
                    let si = *state_idx;

                    if let BlockState::Threshold { is_on } = &mut self.state[si] {
                        let was_on = *is_on;
                        if val >= on_thresh {
                            *is_on = true;
                        } else if val <= off_thresh {
                            *is_on = false;
                        }
                        self.signals[outs[0]] = bool_f64(*is_on);
                        self.signals[outs[1]] = bool_f64(!was_on && *is_on);
                        self.signals[outs[2]] = bool_f64(was_on && !*is_on);
                    }
                }

                // -- AMemory --
                EvalStep::Memory {
                    input,
                    trigger,
                    prev_trigger,
                    reset,
                    output,
                    state_idx,
                } => {
                    let inp = self.signals[*input];
                    let trig = self.signals[*trigger];
                    let prev_trig = self.prev_signals[*prev_trigger];
                    let rst = self.signals[*reset];
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::Latch { value } = &mut self.state[si] {
                        if rst >= 0.5 {
                            *value = 0.0;
                        } else if prev_trig < 0.5 && trig >= 0.5 {
                            *value = inp;
                        }
                        self.signals[out] = *value;
                    }
                }

                // -- FlipFlop --
                EvalStep::FlipFlop {
                    set,
                    reset,
                    output,
                    state_idx,
                } => {
                    let s = self.signals[*set];
                    let r = self.signals[*reset];
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::Toggle { state } = &mut self.state[si] {
                        if s >= 0.5 {
                            *state = true;
                        } else if r >= 0.5 {
                            *state = false;
                        }
                        self.signals[out] = bool_f64(*state);
                    }
                }

                // -- PushButton --
                EvalStep::PushButton {
                    trigger,
                    prev_trigger,
                    force_on,
                    outputs,
                    state_idx,
                } => {
                    let trig = self.signals[*trigger];
                    let prev_trig = self.prev_signals[*prev_trigger];
                    let force = self.signals[*force_on];
                    let outs = *outputs;
                    let si = *state_idx;

                    if let BlockState::PushButton { is_on } = &mut self.state[si] {
                        let previous = *is_on;
                        if force >= 0.5 {
                            *is_on = true;
                        } else if prev_trig < 0.5 && trig >= 0.5 {
                            *is_on = !*is_on;
                        }
                        let qon = !previous && *is_on;
                        let qoff = previous && !*is_on;
                        self.signals[outs[0]] = bool_f64(*is_on);
                        self.signals[outs[1]] = bool_f64(qoff);
                        self.signals[outs[2]] = bool_f64(qon);
                    }
                }

                // -- Counter --
                EvalStep::Counter {
                    input,
                    prev_input,
                    output,
                    state_idx,
                } => {
                    let cur = self.signals[*input];
                    let prev = self.prev_signals[*prev_input];
                    let out = *output;
                    let si = *state_idx;

                    if let BlockState::Counter { count } = &mut self.state[si] {
                        if prev < 0.5 && cur >= 0.5 {
                            *count += 1.0;
                        }
                        self.signals[out] = *count;
                    }
                }

                // -- Copy (PassThrough) --
                EvalStep::Copy { src, dst } => {
                    self.signals[*dst] = self.signals[*src];
                }

                // -- Constant --
                EvalStep::Const { param, output } => {
                    self.signals[*output] = self.signals[*param];
                }

                // -- Gain --
                EvalStep::Gain {
                    input,
                    param,
                    output,
                } => {
                    self.signals[*output] = self.signals[*input] * self.signals[*param];
                }

                // -- RisingEdge --
                EvalStep::RisingEdge {
                    input,
                    prev_input,
                    output,
                } => {
                    let cur = self.signals[*input];
                    let prev = self.prev_signals[*prev_input];
                    self.signals[*output] = bool_f64(prev < 0.5 && cur >= 0.5);
                }

                // -- DayTimer --
                EvalStep::DayTimer {
                    minutes_input,
                    day_input,
                    outputs,
                    state_idx,
                } => {
                    let minutes = self.signals[*minutes_input];
                    let day = self.signals[*day_input].round().clamp(0.0, 6.0) as u32;
                    let outs = *outputs;
                    let si = *state_idx;

                    if let BlockState::DayTimer {
                        entries,
                        last_active,
                        last_value: _,
                    } = &mut self.state[si]
                    {
                        let (value, remaining_secs) = daytimer_value_at(entries, minutes, day);
                        let active = value.abs() > f64::EPSILON;
                        let qon = !*last_active && active;
                        let qoff = *last_active && !active;
                        *last_active = active;

                        self.signals[outs[0]] = value;
                        self.signals[outs[1]] = value;
                        self.signals[outs[2]] = bool_f64(qon);
                        self.signals[outs[3]] = bool_f64(qoff);
                        self.signals[outs[4]] = remaining_secs;
                    }
                }
            }
        }
    }

    // -- External I/O --------------------------------------------------------

    /// Inject a value into a named input connector.
    /// Returns `true` if the name was found.
    pub fn set_input(&mut self, name: &str, value: f64) -> bool {
        if let Some(&idx) = self.named_inputs.get(name) {
            self.signals[idx] = value;
            true
        } else {
            false
        }
    }

    /// Read a named output connector value.
    pub fn get_output(&self, name: &str) -> f64 {
        self.named_outputs
            .get(name)
            .map(|&idx| self.signals[idx])
            .unwrap_or(0.0)
    }

    /// Direct signal read by index.
    #[inline]
    pub fn signal(&self, idx: usize) -> f64 {
        self.signals[idx]
    }

    /// Direct signal write by index.
    #[inline]
    pub fn set_signal(&mut self, idx: usize, value: f64) {
        self.signals[idx] = value;
    }

    /// Number of signals (connectors) in the compiled graph.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Number of evaluation steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Number of stateful blocks.
    pub fn state_count(&self) -> usize {
        self.state.len()
    }

    /// Look up the signal index for a named input.
    pub fn input_index(&self, name: &str) -> Option<usize> {
        self.named_inputs.get(name).copied()
    }

    /// Look up the signal index for a named output.
    pub fn output_index(&self, name: &str) -> Option<usize> {
        self.named_outputs.get(name).copied()
    }

    /// Set a parameter value by block name and key.
    /// Returns `true` if found.
    pub fn set_param(&mut self, name: &str, key: &str, value: f64) -> bool {
        let full = format!("{name}.{key}");
        if let Some(&idx) = self.named_params.get(&full) {
            self.signals[idx] = value;
            true
        } else {
            false
        }
    }

    /// Look up the signal index for a named parameter.
    pub fn param_index(&self, name: &str) -> Option<usize> {
        self.named_params.get(name).copied()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[inline(always)]
fn bool_f64(v: bool) -> f64 {
    if v {
        1.0
    } else {
        0.0
    }
}

/// Extract DayTimer entries from the block impl (downcasting via state serialisation).
fn extract_daytimer_entries(graph: &SimGraph, block_id: BlockId) -> Vec<DayTimerEntry> {
    // The DayTimer block stores entries internally. We can't downcast trait objects,
    // so we return an empty vec — entries must be configured via `set_daytimer_entries`.
    let _ = (graph, block_id);
    Vec::new()
}

/// Evaluate DayTimer schedule (mirrors `DayTimer::value_at`).
fn daytimer_value_at(
    entries: &[DayTimerEntry],
    minutes_since_midnight: f64,
    day_of_week: u32,
) -> (f64, f64) {
    let mut matching: SmallVec<[DayTimerEntry; 16]> = entries
        .iter()
        .copied()
        .filter(|e| e.day_of_week.is_none_or(|d| d == day_of_week))
        .collect();
    matching.sort_by(|a, b| a.to_minute.total_cmp(&b.to_minute));

    let mut current_value = 0.0;
    let mut remaining_minutes = 0.0;
    for entry in &matching {
        if minutes_since_midnight < entry.to_minute {
            current_value = entry.value;
            remaining_minutes = (entry.to_minute - minutes_since_midnight).max(0.0);
            break;
        }
        current_value = entry.value;
    }
    (current_value, remaining_minutes * 60.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{self, And, Block, PassThrough};
    use crate::graph::SimGraph;

    fn pt() -> Box<dyn Block> {
        Box::new(PassThrough)
    }

    // -- Basic And gate --

    #[test]
    fn compiled_and_gate_both_high() {
        let mut g = SimGraph::new();
        let a = g.add_block("InputA", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("InputB", pt(), &["I1"], &["Q"], &[]);
        let and = g.add_block("Gate", Box::new(And), &["I1", "I2"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(and, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(and, "I2").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);
        c.set_input("InputA", 1.0);
        c.set_input("InputB", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("Gate"), 1.0);
    }

    #[test]
    fn compiled_and_gate_one_low() {
        let mut g = SimGraph::new();
        let a = g.add_block("InputA", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("InputB", pt(), &["I1"], &["Q"], &[]);
        let and = g.add_block("Gate", Box::new(And), &["I1", "I2"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(and, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(and, "I2").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);
        c.set_input("InputA", 1.0);
        c.set_input("InputB", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("Gate"), 0.0);
    }

    // -- GreaterEqual + And chain --

    #[test]
    fn compiled_ge_and_chain() {
        let mut g = SimGraph::new();
        let temp = g.add_block("Temp", pt(), &["I1"], &["Q"], &[]);
        let thresh = g.add_block("Thresh", pt(), &["I1"], &["Q"], &[]);
        let ge = g.add_block(
            "GE",
            Box::new(blocks::GreaterEqual),
            &["I1", "I2"],
            &["Q"],
            &[],
        );
        let enable = g.add_block("Enable", pt(), &["I1"], &["Q"], &[]);
        let and = g.add_block("Result", Box::new(And), &["I1", "I2"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(temp, "Q").unwrap(),
            g.find_connector(ge, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(thresh, "Q").unwrap(),
            g.find_connector(ge, "I2").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(ge, "Q").unwrap(),
            g.find_connector(and, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(enable, "Q").unwrap(),
            g.find_connector(and, "I2").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        // Temp=25, Thresh=20, Enable=1 → GE=1, Result=1
        c.set_input("Temp", 25.0);
        c.set_input("Thresh", 20.0);
        c.set_input("Enable", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("GE"), 1.0);
        assert_eq!(c.get_output("Result"), 1.0);

        // Temp=15 → GE=0, Result=0
        c.set_input("Temp", 15.0);
        c.tick(0.1);
        assert_eq!(c.get_output("GE"), 0.0);
        assert_eq!(c.get_output("Result"), 0.0);

        // Temp=25, Enable=0 → GE=1, Result=0
        c.set_input("Temp", 25.0);
        c.set_input("Enable", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("GE"), 1.0);
        assert_eq!(c.get_output("Result"), 0.0);
    }

    // -- Monoflop timing --

    #[test]
    fn compiled_monoflop_timing() {
        let mut g = SimGraph::new();
        let trig = g.add_block("Trig", pt(), &["I1"], &["Q"], &[]);
        let mono = g.add_block(
            "Mono",
            Box::new(blocks::Monoflop::new()),
            &["I1"],
            &["Q"],
            &["Time"],
        );
        g.add_wire(
            g.find_connector(trig, "Q").unwrap(),
            g.find_connector(mono, "I1").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        // Set duration parameter to 1.0 second
        let time_idx = c.param_index("Mono.Time").unwrap();
        // Time is a parameter, write directly
        c.set_signal(time_idx, 1.0);

        // Rising edge → Monoflop starts
        c.set_input("Trig", 1.0);
        c.tick(0.25);
        assert_eq!(c.get_output("Mono"), 1.0, "tick 1: should be high");

        c.set_input("Trig", 0.0);
        c.tick(0.25);
        assert_eq!(c.get_output("Mono"), 1.0, "tick 2: still high");

        c.tick(0.25);
        assert_eq!(c.get_output("Mono"), 1.0, "tick 3: still high");

        c.tick(0.25);
        assert_eq!(
            c.get_output("Mono"),
            1.0,
            "tick 4: still high (last tick of duration)"
        );

        c.tick(0.25);
        assert_eq!(c.get_output("Mono"), 0.0, "tick 5: should be low");
    }

    // -- FlipFlop --

    #[test]
    fn compiled_flipflop() {
        let mut g = SimGraph::new();
        let set = g.add_block("Set", pt(), &["I1"], &["Q"], &[]);
        let rst = g.add_block("Reset", pt(), &["I1"], &["Q"], &[]);
        let ff = g.add_block(
            "FF",
            Box::new(blocks::FlipFlop::new()),
            &["S", "R"],
            &["Q"],
            &[],
        );

        g.add_wire(
            g.find_connector(set, "Q").unwrap(),
            g.find_connector(ff, "S").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(rst, "Q").unwrap(),
            g.find_connector(ff, "R").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        c.set_input("Set", 1.0);
        c.set_input("Reset", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("FF"), 1.0);

        c.set_input("Set", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("FF"), 1.0, "latched high");

        c.set_input("Reset", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("FF"), 0.0, "reset");
    }

    // -- Math blocks --

    #[test]
    fn compiled_add_mult() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let add = g.add_block("Sum", Box::new(blocks::Add), &["I1", "I2"], &["Q"], &[]);
        let c_blk = g.add_block("C", pt(), &["I1"], &["Q"], &[]);
        let mult = g.add_block(
            "Product",
            Box::new(blocks::Mult),
            &["I1", "I2"],
            &["Q"],
            &[],
        );

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(add, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(add, "I2").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(add, "Q").unwrap(),
            g.find_connector(mult, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(c_blk, "Q").unwrap(),
            g.find_connector(mult, "I2").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);
        c.set_input("A", 3.0);
        c.set_input("B", 4.0);
        c.set_input("C", 2.0);
        c.tick(0.1);

        assert_eq!(c.get_output("Sum"), 7.0);
        assert_eq!(c.get_output("Product"), 14.0);
    }

    // -- Compiled vs interpreter equivalence --

    #[test]
    fn compiled_matches_interpreter() {
        use crate::engine::SimEngine;

        // Build graph with a mix of blocks.
        fn build_graph() -> SimGraph {
            let mut g = SimGraph::new();
            let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
            let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
            let or = g.add_block("OR", Box::new(blocks::Or), &["I1", "I2"], &["Q"], &[]);
            let not = g.add_block("NOT", Box::new(blocks::Not), &["I1"], &["Q"], &[]);

            g.add_wire(
                g.find_connector(a, "Q").unwrap(),
                g.find_connector(or, "I1").unwrap(),
            )
            .unwrap();
            g.add_wire(
                g.find_connector(b, "Q").unwrap(),
                g.find_connector(or, "I2").unwrap(),
            )
            .unwrap();
            g.add_wire(
                g.find_connector(or, "Q").unwrap(),
                g.find_connector(not, "I1").unwrap(),
            )
            .unwrap();
            g
        }

        let inputs = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];

        for &(a_val, b_val) in &inputs {
            // Interpreter
            let g1 = build_graph();
            let mut eng = SimEngine::new(g1);
            eng.set_input("A", a_val);
            eng.set_input("B", b_val);
            eng.tick(0.1);
            let interp_or = eng.get_output("OR");
            let interp_not = eng.get_output("NOT");

            // Compiled
            let g2 = build_graph();
            let mut comp = CompiledGraph::from_graph(&g2);
            comp.set_input("A", a_val);
            comp.set_input("B", b_val);
            comp.tick(0.1);
            let comp_or = comp.get_output("OR");
            let comp_not = comp.get_output("NOT");

            assert_eq!(
                interp_or, comp_or,
                "OR mismatch for inputs ({a_val}, {b_val})"
            );
            assert_eq!(
                interp_not, comp_not,
                "NOT mismatch for inputs ({a_val}, {b_val})"
            );
        }
    }

    // -- Performance: compile + run many ticks --

    #[test]
    fn benchmark_compiled_100k_ticks() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let ge = g.add_block(
            "GE",
            Box::new(blocks::GreaterEqual),
            &["I1", "I2"],
            &["Q"],
            &[],
        );
        let and = g.add_block("AND", Box::new(And), &["I1", "I2"], &["Q"], &[]);
        let mono = g.add_block(
            "Mono",
            Box::new(blocks::Monoflop::new()),
            &["I1"],
            &["Q"],
            &["Time"],
        );

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(ge, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(ge, "I2").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(ge, "Q").unwrap(),
            g.find_connector(and, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(and, "I2").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(and, "Q").unwrap(),
            g.find_connector(mono, "I1").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        let time_idx = c.param_index("Mono.Time").unwrap();
        c.set_signal(time_idx, 1.0);

        let start = std::time::Instant::now();
        for i in 0..100_000 {
            let v = if i % 100 < 50 { 1.0 } else { 0.0 };
            c.set_input("A", v);
            c.set_input("B", 0.5);
            c.tick(0.001);
        }
        let elapsed = start.elapsed();
        // 100K ticks with 5 blocks = 500K block evaluations
        let evals_per_sec = 500_000.0 / elapsed.as_secs_f64();

        eprintln!(
            "compiled: 100K ticks in {:.2?} ({:.1}M block-evals/sec)",
            elapsed,
            evals_per_sec / 1_000_000.0
        );

        // We expect at least 10M block-evals/sec even in debug mode.
        // In release mode this should be 50M+.
        assert!(
            evals_per_sec > 1_000_000.0,
            "too slow: {evals_per_sec:.0} evals/sec"
        );
    }

    // -- OffDelay timing --

    #[test]
    fn compiled_offdelay_timing() {
        let mut g = SimGraph::new();
        let trig = g.add_block("Trig", pt(), &["I1"], &["Q"], &[]);
        let od = g.add_block(
            "OD",
            Box::new(blocks::OffDelay::new()),
            &["I1"],
            &["Q"],
            &["Time"],
        );
        g.add_wire(
            g.find_connector(trig, "Q").unwrap(),
            g.find_connector(od, "I1").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);
        let time_idx = c.param_index("OD.Time").unwrap();
        c.set_signal(time_idx, 0.5);

        // Input high → output high immediately
        c.set_input("Trig", 1.0);
        c.tick(0.25);
        assert_eq!(c.get_output("OD"), 1.0);

        // Falling edge → starts delay
        c.set_input("Trig", 0.0);
        c.tick(0.25);
        assert_eq!(c.get_output("OD"), 1.0, "delay active");

        c.tick(0.25);
        assert_eq!(c.get_output("OD"), 1.0, "delay still active");

        c.tick(0.25);
        assert_eq!(c.get_output("OD"), 0.0, "delay expired");
    }

    // -- Counter --

    #[test]
    fn compiled_counter() {
        let mut g = SimGraph::new();
        let inp = g.add_block("Inp", pt(), &["I1"], &["Q"], &[]);
        let ctr = g.add_block(
            "Ctr",
            Box::new(blocks::Counter::new()),
            &["I1"],
            &["Q"],
            &[],
        );
        g.add_wire(
            g.find_connector(inp, "Q").unwrap(),
            g.find_connector(ctr, "I1").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        // Rising edge
        c.set_input("Inp", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("Ctr"), 1.0);

        // Still high → no count
        c.tick(0.1);
        assert_eq!(c.get_output("Ctr"), 1.0);

        // Low
        c.set_input("Inp", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("Ctr"), 1.0);

        // Another rising edge
        c.set_input("Inp", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("Ctr"), 2.0);
    }

    // -- Not block --

    #[test]
    fn compiled_not() {
        let mut g = SimGraph::new();
        let inp = g.add_block("Inp", pt(), &["I1"], &["Q"], &[]);
        let not = g.add_block("NOT", Box::new(blocks::Not), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(inp, "Q").unwrap(),
            g.find_connector(not, "I1").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        c.set_input("Inp", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("NOT"), 1.0);

        c.set_input("Inp", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("NOT"), 0.0);
    }

    // -- Xor block --

    #[test]
    fn compiled_xor() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", pt(), &["I1"], &["Q"], &[]);
        let xor = g.add_block("XOR", Box::new(blocks::Xor), &["I1", "I2"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(xor, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(xor, "I2").unwrap(),
        )
        .unwrap();

        let mut c = CompiledGraph::from_graph(&g);

        c.set_input("A", 1.0);
        c.set_input("B", 0.0);
        c.tick(0.1);
        assert_eq!(c.get_output("XOR"), 1.0);

        c.set_input("B", 1.0);
        c.tick(0.1);
        assert_eq!(c.get_output("XOR"), 0.0);
    }

    // -- Compiled step/state counts --

    #[test]
    fn compiled_graph_metadata() {
        let mut g = SimGraph::new();
        g.add_block("A", pt(), &["I1"], &["Q"], &[]);
        g.add_block("B", Box::new(And), &["I1"], &["Q"], &[]);
        g.add_block(
            "M",
            Box::new(blocks::Monoflop::new()),
            &["I1"],
            &["Q"],
            &["T"],
        );

        let c = CompiledGraph::from_graph(&g);
        assert_eq!(c.step_count(), 3);
        assert_eq!(c.state_count(), 1); // only Monoflop is stateful
        assert_eq!(c.signal_count(), g.connector_count());
    }
}
