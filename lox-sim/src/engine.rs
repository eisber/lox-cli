use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::autodiff::DualNumber;
use crate::blocks::Block;
use crate::graph::SimGraph;
use crate::profiler::{ProfileReport, SimProfiler};
use crate::trace::{self, TraceResult};
use crate::types::*;

// ---------------------------------------------------------------------------
// Pre-computed per-block evaluation info (avoids borrowing graph during tick)
// ---------------------------------------------------------------------------

struct BlockEvalInfo {
    /// `(source_cid, is_feedback)` for each input connector.
    input_sources: Vec<(ConnectorId, bool)>,
    /// Connector IDs for outputs.
    output_cids: Vec<ConnectorId>,
    /// Connector IDs for parameters.
    param_cids: Vec<ConnectorId>,
    /// Whether the block uses prev_inputs for edge detection.
    edge_sensitive: bool,
    /// Previous-tick inputs as seen during last evaluation (for dirty detection).
    last_prev_inputs: Vec<f64>,
}

// ---------------------------------------------------------------------------
// SimEngine
// ---------------------------------------------------------------------------

/// Tick-based evaluation engine.
///
/// Owns the simulation graph, block implementations, and all signal state.
/// Call [`tick`] to advance the simulation by one time step.
pub struct SimEngine {
    graph: SimGraph,
    blocks: Vec<Box<dyn Block>>,
    signals: Vec<f64>,
    prev_signals: Vec<f64>,
    dirty: Vec<bool>,
    topo_order: Vec<BlockId>,
    eval_info: Vec<BlockEvalInfo>,
    /// `downstream[block_id]` → blocks whose inputs depend on this block's outputs.
    downstream: Vec<Vec<BlockId>>,
    named_inputs: HashMap<String, Vec<ConnectorId>>,
    named_outputs: HashMap<String, Vec<ConnectorId>>,
    /// Persistent output overrides: after block eval, these replace computed values.
    output_overrides: HashMap<ConnectorId, f64>,
    profiler: Option<SimProfiler>,
}

impl SimEngine {
    /// Build an engine from a fully-wired graph.
    pub fn new(mut graph: SimGraph) -> Self {
        let topo = graph.topological_order();
        let blocks = graph.take_block_impls();
        let n_conn = graph.connector_count();
        let n_blocks = graph.block_count();

        // Initialise signals with connector defaults.
        let signals: Vec<f64> = (0..n_conn)
            .map(|i| graph.connector(i).default_value)
            .collect();

        // Pre-compute per-block eval info.
        let eval_info: Vec<BlockEvalInfo> = (0..n_blocks)
            .map(|bid| {
                let info = graph.block_info(bid);
                let input_sources: Vec<(ConnectorId, bool)> = info
                    .inputs
                    .iter()
                    .map(|&cid| {
                        match graph.input_source_of(cid) {
                            Some(src) => {
                                let is_fb = topo.feedback_wires.contains(&(src, cid));
                                (src, is_fb)
                            }
                            None => (cid, false), // unwired — read local connector value
                        }
                    })
                    .collect();
                let n_inputs = input_sources.len();
                let edge_sensitive = blocks[bid].is_edge_sensitive();
                BlockEvalInfo {
                    input_sources,
                    output_cids: info.outputs.clone(),
                    param_cids: info.params.clone(),
                    edge_sensitive,
                    last_prev_inputs: vec![0.0; n_inputs],
                }
            })
            .collect();

        // Pre-compute downstream map.
        let mut downstream: Vec<Vec<BlockId>> = vec![Vec::new(); n_blocks];
        let mut seen_edges: HashSet<(BlockId, BlockId)> = HashSet::new();
        for &(from_cid, to_cid) in graph.wires() {
            let src = graph.connector(from_cid).block_id;
            let dst = graph.connector(to_cid).block_id;
            if src != dst && seen_edges.insert((src, dst)) {
                downstream[src].push(dst);
            }
        }

        // Build named input / output lookup tables.
        // Entries: "BlockName" → connectors, "BlockName.Key" → specific connector.
        // For source blocks (SysVar, VirtualIn), the bare name maps to
        // output connectors so set_input drives the value out directly.
        let mut named_inputs: HashMap<String, Vec<ConnectorId>> = HashMap::new();
        let mut named_outputs: HashMap<String, Vec<ConnectorId>> = HashMap::new();
        let source_types = ["SysVar", "VirtualIn"];
        #[allow(clippy::needless_range_loop)]
        for bid in 0..n_blocks {
            let info = graph.block_info(bid);
            let is_source = source_types.contains(&blocks[bid].block_type());
            for &cid in &info.inputs {
                let key = format!("{}.{}", info.name, graph.connector(cid).key);
                named_inputs.entry(key.clone()).or_default().push(cid);
                // Room-qualified: "Name [Room].Connector"
                if let Some(ref room) = info.room {
                    let rkey = format!("{} [{}].{}", info.name, room, graph.connector(cid).key);
                    named_inputs.entry(rkey).or_default().push(cid);
                }
            }
            for &cid in &info.outputs {
                let key = format!("{}.{}", info.name, graph.connector(cid).key);
                named_outputs.entry(key.clone()).or_default().push(cid);
                if let Some(ref room) = info.room {
                    let rkey = format!("{} [{}].{}", info.name, room, graph.connector(cid).key);
                    named_outputs.entry(rkey).or_default().push(cid);
                }
            }
            // Bare block name + outputs as settable inputs
            // Register "Block.OutputConnector" as settable (for simulation injection)
            // but do NOT override bare block name mapping for non-source blocks
            if is_source {
                for &cid in &info.inputs {
                    named_inputs.entry(info.name.clone()).or_default().push(cid);
                }
                for &cid in &info.outputs {
                    named_inputs.entry(info.name.clone()).or_default().push(cid);
                }
            } else if let Some(&cid) = info.inputs.first() {
                named_inputs.entry(info.name.clone()).or_default().push(cid);
            }
            if let Some(&cid) = info.outputs.first() {
                named_outputs
                    .entry(info.name.clone())
                    .or_default()
                    .push(cid);
            }
            // Room-qualified bare name: "Name [Room]"
            if let Some(ref room) = info.room {
                let rname = format!("{} [{}]", info.name, room);
                if is_source {
                    for &cid in &info.inputs {
                        named_inputs.entry(rname.clone()).or_default().push(cid);
                    }
                    for &cid in &info.outputs {
                        named_inputs.entry(rname.clone()).or_default().push(cid);
                    }
                } else if let Some(&cid) = info.inputs.first() {
                    named_inputs.entry(rname.clone()).or_default().push(cid);
                }
                if let Some(&cid) = info.outputs.first() {
                    named_outputs.entry(rname).or_default().push(cid);
                }
            }
        }

        SimEngine {
            graph,
            blocks,
            signals: signals.clone(),
            prev_signals: signals,
            dirty: vec![true; n_blocks], // first tick evaluates everything
            topo_order: topo.order,
            eval_info,
            downstream,
            named_inputs,
            named_outputs,
            output_overrides: HashMap::new(),
            profiler: None,
        }
    }

    // --- External I/O -------------------------------------------------------

    /// Inject a value into a named input connector.
    ///
    /// Names are resolved as `"BlockName"` (first input) or `"BlockName.Key"`.
    /// Returns `true` if the name was found.
    pub fn set_input(&mut self, name: &str, value: f64) -> bool {
        if let Some(cids) = self.named_inputs.get(name) {
            let cids = cids.clone();
            for &cid in &cids {
                self.signals[cid] = value;
                let block_id = self.graph.connector(cid).block_id;
                self.dirty[block_id] = true;
                // When setting an output connector directly (source blocks),
                // also mark downstream blocks dirty so the value propagates.
                if self.graph.connector(cid).dir == ConnectorDir::Output {
                    for &ds in &self.downstream[block_id] {
                        self.dirty[ds] = true;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Inject a value directly into a named output connector.
    ///
    /// Unlike `set_input`, this targets output connectors — useful for
    /// simulating sensor readings on blocks like PresenceDetector where
    /// the output (e.g. `OutputPresence`) is what downstream blocks read.
    pub fn inject_output(&mut self, name: &str, value: f64) -> bool {
        if let Some(cids) = self.named_outputs.get(name) {
            let cids = cids.clone();
            for &cid in &cids {
                self.output_overrides.insert(cid, value);
                self.signals[cid] = value;
                let block_id = self.graph.connector(cid).block_id;
                for &ds in &self.downstream[block_id] {
                    self.dirty[ds] = true;
                }
            }
            true
        } else {
            false
        }
    }

    /// Remove a persistent output override for a named output connector.
    ///
    /// Returns `true` if the name was found (regardless of whether an override existed).
    pub fn clear_override(&mut self, name: &str) -> bool {
        if let Some(cids) = self.named_outputs.get(name) {
            let cids = cids.clone();
            for &cid in &cids {
                self.output_overrides.remove(&cid);
                let block_id = self.graph.connector(cid).block_id;
                self.dirty[block_id] = true;
            }
            true
        } else {
            false
        }
    }

    /// Remove all persistent output overrides.
    pub fn clear_all_overrides(&mut self) {
        for &cid in self.output_overrides.keys().collect::<Vec<_>>() {
            let block_id = self.graph.connector(cid).block_id;
            self.dirty[block_id] = true;
        }
        self.output_overrides.clear();
    }

    /// Read a named output connector value.
    ///
    /// Names are resolved as `"BlockName"` (first output) or `"BlockName.Key"`.
    pub fn get_output(&self, name: &str) -> f64 {
        self.named_outputs
            .get(name)
            .map(|cids| {
                cids.iter()
                    .map(|&cid| self.signals[cid])
                    .reduce(|a, b| if b.abs() > a.abs() { b } else { a })
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0)
    }

    /// Look up connector IDs for a named input (for reading signals arriving at actuators).
    pub fn named_input_cids(&self, name: &str) -> Option<&Vec<ConnectorId>> {
        self.named_inputs.get(name)
    }

    /// Force all blocks to be evaluated on the next tick.
    pub fn mark_all_dirty(&mut self) {
        self.dirty.fill(true);
    }

    // --- Tick ---------------------------------------------------------------

    /// Advance the simulation by one time step.
    ///
    /// Evaluates blocks in topological order, skipping clean blocks whose
    /// inputs have not changed (dirty-flag optimisation).
    pub fn tick(&mut self, dt: f64) {
        let profiling_enabled = self.profiler.is_some();
        let tick_start = profiling_enabled.then(Instant::now);
        let mut blocks_evaluated = 0usize;
        let mut blocks_skipped = 0usize;
        let mut signal_changes = 0u64;

        for idx in 0..self.topo_order.len() {
            let block_id = self.topo_order[idx];

            if !self.dirty[block_id] {
                if profiling_enabled {
                    blocks_skipped += 1;
                }
                continue;
            }

            let ei = &self.eval_info[block_id];

            // Gather current inputs.
            let inputs: Vec<f64> = ei
                .input_sources
                .iter()
                .map(|&(src, is_fb)| {
                    if is_fb {
                        self.prev_signals[src]
                    } else {
                        self.signals[src]
                    }
                })
                .collect();

            // Gather params.
            let params: Vec<f64> = ei.param_cids.iter().map(|&cid| self.signals[cid]).collect();

            // Gather previous-tick inputs (for edge detection).
            let prev_inputs: Vec<f64> = ei
                .input_sources
                .iter()
                .map(|&(src, _)| self.prev_signals[src])
                .collect();

            // Evaluate.
            let output_cids = ei.output_cids.clone();
            let (outputs, block_time_ns) = if profiling_enabled {
                let block_start = Instant::now();
                let outputs = self.blocks[block_id].eval(&inputs, &params, dt, &prev_inputs);
                (
                    outputs,
                    Some(block_start.elapsed().as_nanos().min(u64::MAX as u128) as u64),
                )
            } else {
                (
                    self.blocks[block_id].eval(&inputs, &params, dt, &prev_inputs),
                    None,
                )
            };
            if let Some(time_ns) = block_time_ns {
                blocks_evaluated += 1;
                if let Some(profiler) = self.profiler.as_mut() {
                    profiler.record_block(block_id, time_ns);
                }
            }

            // Save the prev_inputs this block saw (for next-tick dirty detection).
            self.eval_info[block_id].last_prev_inputs = prev_inputs;

            // Write outputs, propagate dirty if any output changed.
            // Persistent overrides take precedence over computed values.
            let mut any_changed = false;
            for (i, &cid) in output_cids.iter().enumerate() {
                let val = if let Some(&ov) = self.output_overrides.get(&cid) {
                    ov
                } else {
                    outputs.get(i).copied().unwrap_or(self.signals[cid])
                };
                if (self.signals[cid] - val).abs() > f64::EPSILON {
                    any_changed = true;
                    signal_changes += 1;
                }
                self.signals[cid] = val;
            }
            if any_changed {
                for &ds in &self.downstream[block_id] {
                    self.dirty[ds] = true;
                }
            }

            self.dirty[block_id] = false;
        }

        // Snapshot signals for next tick's previous-value lookups.
        self.prev_signals.copy_from_slice(&self.signals);

        // Mark edge-sensitive blocks dirty whose prev_inputs will differ on
        // the next tick compared to what they saw during this tick's evaluation.
        // This ensures edge-detection blocks (RisingEdge, Counter, etc.)
        // re-evaluate when their historical context changes.
        for &block_id in &self.topo_order {
            if self.dirty[block_id] {
                continue;
            }
            let ei = &self.eval_info[block_id];
            if !ei.edge_sensitive {
                continue;
            }
            let mut changed = false;
            for (i, &(src, _)) in ei.input_sources.iter().enumerate() {
                if (self.prev_signals[src] - ei.last_prev_inputs[i]).abs() > f64::EPSILON {
                    changed = true;
                    break;
                }
            }
            if changed {
                self.dirty[block_id] = true;
            }
        }

        if let (Some(profiler), Some(tick_start)) = (self.profiler.as_mut(), tick_start) {
            profiler.record_tick(
                tick_start.elapsed().as_nanos().min(u64::MAX as u128) as u64,
                blocks_evaluated,
                blocks_skipped,
            );
            profiler.record_signal_changes(signal_changes);
        }
    }

    // --- Accessors ----------------------------------------------------------

    /// Direct signal read by connector ID.
    pub fn signal(&self, cid: ConnectorId) -> f64 {
        self.signals[cid]
    }

    /// Previous-tick signal by connector ID.
    pub fn prev_signal(&self, cid: ConnectorId) -> f64 {
        self.prev_signals[cid]
    }

    /// Reference to the underlying graph.
    pub fn graph(&self) -> &SimGraph {
        &self.graph
    }

    /// Check whether a block is currently marked dirty.
    pub fn is_dirty(&self, block_id: BlockId) -> bool {
        self.dirty[block_id]
    }

    /// Read a block's serialised state.
    pub fn block_state(&self, block_id: BlockId) -> Option<Vec<u8>> {
        self.blocks[block_id].state()
    }

    /// Restore a block's state from serialised bytes. Marks the block dirty
    /// so the next tick re-evaluates it with the restored state.
    pub fn restore_block_state(&mut self, block_id: BlockId, data: &[u8]) {
        self.blocks[block_id].restore(data);
        self.dirty[block_id] = true;
    }

    /// Set a parameter connector value by block name and key.
    pub fn set_param(&mut self, block_name: &str, key: &str, value: f64) -> bool {
        if let Some(bid) = self.graph.find_block_by_name(block_name) {
            if let Some(cid) = self.graph.find_connector(bid, key) {
                if self.graph.connector(cid).dir == ConnectorDir::Parameter {
                    self.signals[cid] = value;
                    self.dirty[bid] = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn enable_profiling(&mut self) {
        let mut profiler = SimProfiler::new(self.blocks.len());
        profiler.set_block_types(
            self.blocks
                .iter()
                .map(|block| block.block_type().to_string())
                .collect(),
        );
        self.profiler = Some(profiler);
    }

    pub fn profile_report(&self) -> Option<ProfileReport> {
        self.profiler.as_ref().map(SimProfiler::report)
    }

    // --- Trace mode ---------------------------------------------------------

    /// Trace signal reachability from `source` to `dest` through the wiring graph.
    ///
    /// Returns a [`TraceResult`] with the path and hop count.  The trace is
    /// purely structural — it does not execute blocks.
    pub fn trace(&self, source: &str, dest: &str) -> TraceResult {
        trace::trace(&self.graph, source, dest, None)
    }

    // --- Forward-mode AD evaluation -----------------------------------------

    /// Evaluate one tick using dual numbers for forward-mode AD.
    ///
    /// `inputs` maps named input connectors to [`DualNumber`] values (each
    /// carrying a derivative seed).  The method runs the normal tick logic
    /// but lifts every value into dual-number arithmetic so derivatives
    /// propagate through the block graph.
    ///
    /// Returns a map of named outputs → `DualNumber`.  The `.dot` field of
    /// each output is the derivative of that output w.r.t. whichever
    /// parameter was seeded with `dot = 1.0`.
    ///
    /// **Note**: This is a simplified forward-mode pass that treats each
    /// block as a linear passthrough of its dual inputs.  For blocks with
    /// non-linear behaviour (comparisons, edge detectors), the smooth
    /// relaxation functions in [`crate::autodiff`] should be used to build
    /// differentiable block variants.
    pub fn eval_with_dual(
        &mut self,
        inputs: &HashMap<String, DualNumber>,
    ) -> HashMap<String, DualNumber> {
        // Inject primal values as regular inputs.
        for (name, dual) in inputs {
            self.set_input(name, dual.val);
        }

        // Run a normal tick to get correct primal outputs.
        self.tick(0.1);

        // Build a dual-number signal array seeded from inputs.
        let mut dual_signals: Vec<DualNumber> = self
            .signals
            .iter()
            .map(|&v| DualNumber::constant(v))
            .collect();

        // Seed the input connectors with the provided derivatives.
        for (name, dual) in inputs {
            if let Some(cids) = self.named_inputs.get(name.as_str()) {
                for &cid in cids {
                    dual_signals[cid] = *dual;
                }
            }
        }

        // Propagate duals in topological order.
        for &block_id in &self.topo_order {
            let ei = &self.eval_info[block_id];

            let in_duals: Vec<DualNumber> = ei
                .input_sources
                .iter()
                .map(|&(src, is_fb)| {
                    if is_fb {
                        // Feedback wires use prev-tick: treat as constant.
                        DualNumber::constant(self.prev_signals[src])
                    } else {
                        dual_signals[src]
                    }
                })
                .collect();

            let param_duals: Vec<DualNumber> =
                ei.param_cids.iter().map(|&cid| dual_signals[cid]).collect();

            // Compute dual outputs using the block type's semantics.
            let out_duals =
                eval_block_dual(self.blocks[block_id].block_type(), &in_duals, &param_duals);

            for (i, &cid) in ei.output_cids.iter().enumerate() {
                if i < out_duals.len() {
                    dual_signals[cid] = out_duals[i];
                }
            }
        }

        // Collect named outputs.
        let mut result = HashMap::new();
        for (name, cids) in &self.named_outputs {
            // Use the connector with the largest absolute dual value.
            if let Some(&cid) = cids.first() {
                let best = cids
                    .iter()
                    .map(|&c| dual_signals[c])
                    .reduce(|a, b| if b.val.abs() > a.val.abs() { b } else { a })
                    .unwrap_or(dual_signals[cid]);
                result.insert(name.clone(), best);
            }
        }
        result
    }
}

/// Evaluate a block in dual-number mode based on its type name.
///
/// This provides forward-mode AD semantics for common block types.
/// Unknown types fall back to linear passthrough (sum of inputs).
fn eval_block_dual(
    block_type: &str,
    inputs: &[DualNumber],
    params: &[DualNumber],
) -> Vec<DualNumber> {
    match block_type {
        "PassThrough" => {
            vec![inputs.first().copied().unwrap_or(DualNumber::constant(0.0))]
        }
        "Add" => {
            let sum = inputs
                .iter()
                .chain(params.iter())
                .copied()
                .fold(DualNumber::constant(0.0), |a, b| a + b);
            vec![sum]
        }
        "Sub" => {
            if inputs.is_empty() {
                vec![DualNumber::constant(0.0)]
            } else {
                let mut result = inputs[0];
                for d in &inputs[1..] {
                    result = result - *d;
                }
                for d in params {
                    result = result - *d;
                }
                vec![result]
            }
        }
        "Mult" => {
            let product = inputs
                .iter()
                .chain(params.iter())
                .copied()
                .fold(DualNumber::constant(1.0), |a, b| a * b);
            vec![product]
        }
        "Div" => {
            if inputs.len() < 2 {
                vec![inputs.first().copied().unwrap_or(DualNumber::constant(0.0))]
            } else {
                let mut result = inputs[0];
                for d in &inputs[1..] {
                    if d.val.abs() > f64::EPSILON {
                        result = result / *d;
                    }
                }
                vec![result]
            }
        }
        "Gain" => {
            let input = inputs.first().copied().unwrap_or(DualNumber::constant(0.0));
            let factor = params.first().copied().unwrap_or(DualNumber::constant(1.0));
            vec![input * factor]
        }
        "Constant" => {
            vec![params.first().copied().unwrap_or(DualNumber::constant(0.0))]
        }
        // Default: linear passthrough (sum), preserves gradient flow.
        _ => {
            let sum = inputs
                .iter()
                .copied()
                .fold(DualNumber::constant(0.0), |a, b| a + b);
            vec![sum]
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::*;
    use crate::graph::SimGraph;
    use std::sync::atomic::Ordering;

    /// Helper: build a graph, return engine + block IDs.
    fn simple_chain() -> (SimEngine, BlockId, BlockId, BlockId) {
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let c = g.add_block("C", Box::new(PassThrough), &["I1"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(c, "I1").unwrap(),
        )
        .unwrap();

        (SimEngine::new(g), a, b, c)
    }

    // -- basic signal propagation --------------------------------------------

    #[test]
    fn single_passthrough() {
        let mut g = SimGraph::new();
        g.add_block("P", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let mut e = SimEngine::new(g);

        e.set_input("P", 42.0);
        e.tick(0.1);
        assert!((e.get_output("P") - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn chain_propagation() {
        let (mut e, _a, _b, _c) = simple_chain();
        e.set_input("A", 7.0);
        e.tick(0.1);
        assert!((e.get_output("C") - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn chain_multiple_ticks() {
        let (mut e, _, _, _) = simple_chain();

        e.set_input("A", 1.0);
        e.tick(0.1);
        assert!((e.get_output("C") - 1.0).abs() < f64::EPSILON);

        e.set_input("A", 2.0);
        e.tick(0.1);
        assert!((e.get_output("C") - 2.0).abs() < f64::EPSILON);

        e.set_input("A", 0.0);
        e.tick(0.1);
        assert!((e.get_output("C") - 0.0).abs() < f64::EPSILON);
    }

    // -- named I/O with dotted keys ------------------------------------------

    #[test]
    fn named_io_dotted() {
        let mut g = SimGraph::new();
        g.add_block("Blk", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let mut e = SimEngine::new(g);

        e.set_input("Blk.I1", 99.0);
        e.tick(0.1);
        assert!((e.get_output("Blk.Q") - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_input_unknown_name_returns_false() {
        let mut g = SimGraph::new();
        g.add_block("X", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let mut e = SimEngine::new(g);
        assert!(!e.set_input("NOPE", 1.0));
    }

    #[test]
    fn get_output_unknown_name_returns_zero() {
        let mut g = SimGraph::new();
        g.add_block("X", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let e = SimEngine::new(g);
        assert!((e.get_output("NOPE")).abs() < f64::EPSILON);
    }

    // -- diamond merging signals ---------------------------------------------

    #[test]
    fn diamond_add_merge() {
        let mut g = SimGraph::new();
        let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let left = g.add_block("Left", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let right = g.add_block("Right", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let merge = g.add_block("Merge", Box::new(Add), &["I1", "I2"], &["Q"], &[]);

        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(left, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(right, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(left, "Q").unwrap(),
            g.find_connector(merge, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(right, "Q").unwrap(),
            g.find_connector(merge, "I2").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.set_input("Src", 5.0);
        e.tick(0.1);
        // Both paths carry 5.0, Add produces 10.0.
        assert!((e.get_output("Merge") - 10.0).abs() < f64::EPSILON);
    }

    // -- dirty-flag optimisation ---------------------------------------------

    #[test]
    fn dirty_flag_skips_unchanged_blocks() {
        let (ta, counter_a) = TrackingPassThrough::new();
        let (tb, counter_b) = TrackingPassThrough::new();

        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(ta), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(tb), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);

        // First tick: everything is dirty → both blocks evaluated.
        e.set_input("A", 5.0);
        e.tick(0.1);
        assert_eq!(counter_a.load(Ordering::SeqCst), 1);
        assert_eq!(counter_b.load(Ordering::SeqCst), 1);
        assert!((e.get_output("B") - 5.0).abs() < f64::EPSILON);

        // Second tick, no input change → neither block evaluated again.
        e.tick(0.1);
        assert_eq!(counter_a.load(Ordering::SeqCst), 1);
        assert_eq!(counter_b.load(Ordering::SeqCst), 1);

        // Change input → both evaluated.
        e.set_input("A", 10.0);
        e.tick(0.1);
        assert_eq!(counter_a.load(Ordering::SeqCst), 2);
        assert_eq!(counter_b.load(Ordering::SeqCst), 2);
        assert!((e.get_output("B") - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dirty_flag_same_value_no_propagation() {
        let (ta, counter_a) = TrackingPassThrough::new();
        let (tb, counter_b) = TrackingPassThrough::new();

        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(ta), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(tb), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.set_input("A", 5.0);
        e.tick(0.1);
        assert_eq!(counter_a.load(Ordering::SeqCst), 1);
        assert_eq!(counter_b.load(Ordering::SeqCst), 1);

        // Set same value → A is re-evaluated (dirty from set_input),
        // but output doesn't change so B is NOT re-evaluated.
        e.set_input("A", 5.0);
        e.tick(0.1);
        assert_eq!(counter_a.load(Ordering::SeqCst), 2); // A runs
        assert_eq!(counter_b.load(Ordering::SeqCst), 1); // B skipped
    }

    // -- edge detection (previous tick values) --------------------------------

    #[test]
    fn rising_edge_detection() {
        let mut g = SimGraph::new();
        let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let edge = g.add_block("Edge", Box::new(RisingEdge), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(edge, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);

        // Tick 1: input goes from 0→1 → rising edge detected.
        e.set_input("Src", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Edge") - 1.0).abs() < f64::EPSILON);

        // Tick 2: input stays at 1 → no edge.
        e.tick(0.1);
        assert!((e.get_output("Edge")).abs() < f64::EPSILON);

        // Tick 3: input goes to 0.
        e.set_input("Src", 0.0);
        e.tick(0.1);
        assert!((e.get_output("Edge")).abs() < f64::EPSILON);

        // Tick 4: input goes back to 1 → rising edge again.
        e.set_input("Src", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Edge") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn counter_via_engine() {
        let mut g = SimGraph::new();
        let src = g.add_block("Clk", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let ctr = g.add_block("Ctr", Box::new(Counter::new()), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(ctr, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);

        // Pulse 1: 0→1
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Ctr") - 1.0).abs() < f64::EPSILON);

        // Hold high → no increment.
        e.tick(0.1);
        assert!((e.get_output("Ctr") - 1.0).abs() < f64::EPSILON);

        // Go low.
        e.set_input("Clk", 0.0);
        e.tick(0.1);
        assert!((e.get_output("Ctr") - 1.0).abs() < f64::EPSILON);

        // Pulse 2: 0→1
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Ctr") - 2.0).abs() < f64::EPSILON);

        // Pulse 3: low→high
        e.set_input("Clk", 0.0);
        e.tick(0.1);
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);
    }

    // -- previous-tick buffer ------------------------------------------------

    #[test]
    fn prev_signal_reflects_last_tick() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let a_q = g.find_connector(a, "Q").unwrap();

        let mut e = SimEngine::new(g);
        e.set_input("A", 10.0);
        e.tick(0.1);
        assert!((e.signal(a_q) - 10.0).abs() < f64::EPSILON);

        e.set_input("A", 20.0);
        e.tick(0.1);
        assert!((e.signal(a_q) - 20.0).abs() < f64::EPSILON);
        // prev_signal = end-of-tick snapshot = current values after tick
        assert!((e.prev_signal(a_q) - 20.0).abs() < f64::EPSILON);
    }

    // -- feedback loop -------------------------------------------------------

    #[test]
    fn feedback_loop_uses_previous_tick() {
        // A(Add) → B(PassThrough) → A.I2 (feedback).
        // A.Q = A.I1 + A.I2. Feedback wire from B.Q to A.I2.
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(Add), &["I1", "I2"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);

        // A.Q → B.I1 (forward)
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();
        // B.Q → A.I2 (feedback)
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(a, "I2").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);

        // Tick 1: A.I1=5, feedback A.I2=0 (prev default). A→5, B→5.
        e.set_input("A.I1", 5.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 5.0).abs() < f64::EPSILON);
        assert!((e.get_output("B") - 5.0).abs() < f64::EPSILON);

        // Tick 2: feedback=5 → A=5+5=10, B=10.
        e.set_input("A.I1", 5.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 10.0).abs() < f64::EPSILON);
        assert!((e.get_output("B") - 10.0).abs() < f64::EPSILON);

        // Tick 3: feedback=10 → A=5+10=15.
        e.set_input("A.I1", 5.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 15.0).abs() < f64::EPSILON);
    }

    // -- parameter setting ---------------------------------------------------

    #[test]
    fn set_param_gain_block() {
        let mut g = SimGraph::new();
        let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let gain = g.add_block("G", Box::new(Gain), &["I1"], &["Q"], &["Factor"]);
        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(gain, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.set_param("G", "Factor", 3.0);
        e.set_input("Src", 4.0);
        e.tick(0.1);
        assert!((e.get_output("G") - 12.0).abs() < f64::EPSILON);

        // Change gain.
        e.set_param("G", "Factor", 0.5);
        e.tick(0.1);
        assert!((e.get_output("G") - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_param_unknown_returns_false() {
        let mut g = SimGraph::new();
        g.add_block("X", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let mut e = SimEngine::new(g);
        assert!(!e.set_param("X", "NOPE", 1.0));
        assert!(!e.set_param("MISSING", "P", 1.0));
    }

    #[test]
    fn profiling_is_disabled_by_default() {
        let mut g = SimGraph::new();
        g.add_block("P", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let e = SimEngine::new(g);
        assert!(e.profile_report().is_none());
    }

    #[test]
    fn profile_report_has_valid_numbers() {
        let (mut e, _, _, _) = simple_chain();
        e.enable_profiling();
        e.set_input("A", 1.0);
        for _ in 0..1000 {
            e.tick(0.01);
        }

        let report = e.profile_report().expect("profiling enabled");
        assert_eq!(report.total_ticks, 1000);
        assert!(report.total_wall_time_ms >= 0.0);
        assert!(report.ticks_per_second >= 0.0);
        assert!(report.block_evals_per_second >= 0.0);
        assert!(report.avg_blocks_per_tick > 0.0);
        assert!((0.0..=1.0).contains(&report.dirty_skip_ratio));
        assert!(!report.hottest_blocks.is_empty());
        assert!(report.hottest_blocks[0].2 > 0);
        assert!(report.hottest_blocks[0].3 >= 0.0);
    }

    // -- constant block via parameter ----------------------------------------

    #[test]
    fn constant_block_via_param() {
        let mut g = SimGraph::new();
        g.add_block("C", Box::new(Constant), &[], &["Q"], &["Value"]);
        let mut e = SimEngine::new(g);

        e.set_param("C", "Value", 99.0);
        e.tick(0.1);
        assert!((e.get_output("C") - 99.0).abs() < f64::EPSILON);
    }

    // -- state save/restore --------------------------------------------------

    #[test]
    fn block_state_save_restore() {
        let mut g = SimGraph::new();
        let clk = g.add_block("Clk", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let ctr = g.add_block("Ctr", Box::new(Counter::new()), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(clk, "Q").unwrap(),
            g.find_connector(ctr, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);

        // Count to 3.
        for _ in 0..3 {
            e.set_input("Clk", 1.0);
            e.tick(0.1);
            e.set_input("Clk", 0.0);
            e.tick(0.1);
        }
        assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);

        // Save state.
        let state = e.block_state(ctr).unwrap();

        // Count to 5.
        for _ in 0..2 {
            e.set_input("Clk", 1.0);
            e.tick(0.1);
            e.set_input("Clk", 0.0);
            e.tick(0.1);
        }
        assert!((e.get_output("Ctr") - 5.0).abs() < f64::EPSILON);

        // Restore to 3.
        e.restore_block_state(ctr, &state);
        // Next eval without rising edge should show 3.
        e.set_input("Clk", 0.0);
        e.tick(0.1);
        assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);
    }

    // -- evaluation order correctness ----------------------------------------

    #[test]
    fn topo_order_respected_in_single_tick() {
        // Chain: A(×2) → B(×3). Input=4 → A outputs 8 → B outputs 24.
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(Gain), &["I1"], &["Q"], &["F"]);
        let b = g.add_block("B", Box::new(Gain), &["I1"], &["Q"], &["F"]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.set_param("A", "F", 2.0);
        e.set_param("B", "F", 3.0);
        e.set_input("A", 4.0);
        e.tick(0.1);

        assert!((e.get_output("A") - 8.0).abs() < f64::EPSILON);
        assert!((e.get_output("B") - 24.0).abs() < f64::EPSILON);
    }

    // -- empty engine --------------------------------------------------------

    #[test]
    fn empty_engine_ticks_without_panic() {
        let g = SimGraph::new();
        let mut e = SimEngine::new(g);
        e.tick(0.1);
        e.tick(0.1);
    }

    // -- And block in engine -------------------------------------------------

    #[test]
    fn and_block_in_engine() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let gate = g.add_block("Gate", Box::new(And), &["I1", "I2"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(gate, "I1").unwrap(),
        )
        .unwrap();
        g.add_wire(
            g.find_connector(b, "Q").unwrap(),
            g.find_connector(gate, "I2").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);

        // Both low.
        e.tick(0.1);
        assert!((e.get_output("Gate")).abs() < f64::EPSILON);

        // One high.
        e.set_input("A", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Gate")).abs() < f64::EPSILON);

        // Both high.
        e.set_input("B", 1.0);
        e.tick(0.1);
        assert!((e.get_output("Gate") - 1.0).abs() < f64::EPSILON);

        // One goes low.
        e.set_input("A", 0.0);
        e.tick(0.1);
        assert!((e.get_output("Gate")).abs() < f64::EPSILON);
    }

    // -- multi-output block --------------------------------------------------

    #[test]
    fn multi_output_block() {
        #[derive(Clone)]
        struct DoubleTriple;
        impl Block for DoubleTriple {
            fn eval(
                &mut self,
                inputs: &[Signal],
                _p: &[Signal],
                _dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                let v = inputs.first().copied().unwrap_or(0.0);
                vec![v * 2.0, v * 3.0]
            }
            fn state(&self) -> Option<Vec<u8>> {
                None
            }
            fn restore(&mut self, _: &[u8]) {}
            fn block_type(&self) -> &str {
                "DoubleTriple"
            }
        }

        let mut g = SimGraph::new();
        let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let dt = g.add_block("DT", Box::new(DoubleTriple), &["I1"], &["Q1", "Q2"], &[]);
        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(dt, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.set_input("Src", 5.0);
        e.tick(0.1);
        assert!((e.get_output("DT.Q1") - 10.0).abs() < f64::EPSILON);
        assert!((e.get_output("DT.Q2") - 15.0).abs() < f64::EPSILON);
    }

    // -- dirty flag after construction (all dirty) ---------------------------

    #[test]
    fn all_blocks_dirty_initially() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();

        let e = SimEngine::new(g);
        assert!(e.is_dirty(a));
        assert!(e.is_dirty(b));
    }

    #[test]
    fn blocks_clean_after_tick() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        g.add_wire(
            g.find_connector(a, "Q").unwrap(),
            g.find_connector(b, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.tick(0.1);
        assert!(!e.is_dirty(a));
        assert!(!e.is_dirty(b));
    }

    // -- dt passed to blocks -------------------------------------------------

    #[test]
    fn dt_passed_to_eval() {
        #[derive(Clone)]
        struct DtBlock;
        impl Block for DtBlock {
            fn eval(
                &mut self,
                _i: &[Signal],
                _p: &[Signal],
                dt: f64,
                _prev: &[Signal],
            ) -> Vec<Signal> {
                vec![dt]
            }
            fn state(&self) -> Option<Vec<u8>> {
                None
            }
            fn restore(&mut self, _: &[u8]) {}
            fn block_type(&self) -> &str {
                "DtBlock"
            }
        }

        let mut g = SimGraph::new();
        g.add_block("D", Box::new(DtBlock), &[], &["Q"], &[]);
        let mut e = SimEngine::new(g);

        e.tick(0.05);
        assert!((e.get_output("D") - 0.05).abs() < f64::EPSILON);
    }

    // -- trace mode ----------------------------------------------------------

    #[test]
    fn engine_trace_linear_chain() {
        let (e, a, b, c) = simple_chain();
        let result = e.trace("A", "C");
        assert!(result.found);
        assert_eq!(result.hops, 2);
        assert_eq!(result.path, vec![a, b, c]);
    }

    #[test]
    fn engine_trace_no_path() {
        let mut g = SimGraph::new();
        g.add_block("X", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        g.add_block("Y", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let e = SimEngine::new(g);
        let result = e.trace("X", "Y");
        assert!(!result.found);
    }

    #[test]
    fn engine_trace_self() {
        let mut g = SimGraph::new();
        let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let e = SimEngine::new(g);
        let result = e.trace("A", "A");
        assert!(result.found);
        assert_eq!(result.hops, 0);
        assert_eq!(result.path, vec![a]);
    }

    // -- forward-mode AD evaluation ------------------------------------------

    #[test]
    fn eval_with_dual_passthrough() {
        let mut g = SimGraph::new();
        g.add_block("P", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let mut e = SimEngine::new(g);

        let mut inputs = HashMap::new();
        inputs.insert("P".to_string(), crate::autodiff::DualNumber::variable(5.0));
        let outputs = e.eval_with_dual(&inputs);

        let out = outputs.get("P").or_else(|| outputs.get("P.Q")).unwrap();
        assert!((out.val - 5.0).abs() < f64::EPSILON);
        assert!((out.dot - 1.0).abs() < f64::EPSILON); // d(passthrough)/d(input) = 1
    }

    #[test]
    fn eval_with_dual_gain() {
        // f(x) = 3*x, df/dx = 3
        let mut g = SimGraph::new();
        let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let gain = g.add_block("G", Box::new(Gain), &["I1"], &["Q"], &["Factor"]);
        g.add_wire(
            g.find_connector(src, "Q").unwrap(),
            g.find_connector(gain, "I1").unwrap(),
        )
        .unwrap();

        let mut e = SimEngine::new(g);
        e.set_param("G", "Factor", 3.0);

        let mut inputs = HashMap::new();
        inputs.insert(
            "Src".to_string(),
            crate::autodiff::DualNumber::variable(4.0),
        );
        let outputs = e.eval_with_dual(&inputs);

        let out = outputs.get("G").or_else(|| outputs.get("G.Q")).unwrap();
        assert!((out.val - 12.0).abs() < f64::EPSILON);
        assert!((out.dot - 3.0).abs() < f64::EPSILON); // d(3x)/dx = 3
    }

    #[test]
    fn eval_with_dual_add_chain() {
        // f(x) = x + x = 2x, df/dx = 2
        let mut g = SimGraph::new();
        let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
        let adder = g.add_block("Sum", Box::new(Add), &["I1", "I2"], &["Q"], &[]);

        let src_q = g.find_connector(src, "Q").unwrap();
        g.add_wire(src_q, g.find_connector(adder, "I1").unwrap())
            .unwrap();
        g.add_wire(src_q, g.find_connector(adder, "I2").unwrap())
            .unwrap();

        let mut e = SimEngine::new(g);
        let mut inputs = HashMap::new();
        inputs.insert(
            "Src".to_string(),
            crate::autodiff::DualNumber::variable(7.0),
        );
        let outputs = e.eval_with_dual(&inputs);

        let out = outputs.get("Sum").or_else(|| outputs.get("Sum.Q")).unwrap();
        assert!((out.val - 14.0).abs() < f64::EPSILON);
        assert!((out.dot - 2.0).abs() < f64::EPSILON); // d(2x)/dx = 2
    }

    // -- output override persistence -----------------------------------------

    #[test]
    fn inject_output_persists_across_ticks() {
        // A → B → C. Inject A's output, verify B and C see it after multiple ticks.
        let (mut e, _a, _b, _c) = simple_chain();
        e.inject_output("A", 99.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 99.0).abs() < f64::EPSILON);
        assert!((e.get_output("C") - 99.0).abs() < f64::EPSILON);

        // Second tick — override must persist even though block re-evaluates.
        e.set_input("A", 0.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 99.0).abs() < f64::EPSILON);
        assert!((e.get_output("C") - 99.0).abs() < f64::EPSILON);

        // Third tick — still persists.
        e.tick(0.1);
        assert!((e.get_output("A") - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clear_override_restores_natural_output() {
        let (mut e, _a, _b, _c) = simple_chain();
        e.set_input("A", 5.0);
        e.inject_output("A", 99.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 99.0).abs() < f64::EPSILON);

        // Clear override, tick again — natural value should flow.
        e.clear_override("A");
        e.tick(0.1);
        assert!((e.get_output("A") - 5.0).abs() < f64::EPSILON);
        assert!((e.get_output("C") - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn inject_propagates_to_downstream() {
        // A → B → C. Inject B's output, verify C sees it.
        let (mut e, _a, _b, _c) = simple_chain();
        e.set_input("A", 1.0);
        e.tick(0.1);
        assert!((e.get_output("C") - 1.0).abs() < f64::EPSILON);

        e.inject_output("B", 42.0);
        e.tick(0.1);
        assert!((e.get_output("B") - 42.0).abs() < f64::EPSILON);
        assert!((e.get_output("C") - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clear_all_overrides() {
        let (mut e, _a, _b, _c) = simple_chain();
        e.set_input("A", 3.0);
        e.inject_output("A", 10.0);
        e.inject_output("B", 20.0);
        e.tick(0.1);
        assert!((e.get_output("A") - 10.0).abs() < f64::EPSILON);
        assert!((e.get_output("B") - 20.0).abs() < f64::EPSILON);

        e.clear_all_overrides();
        e.tick(0.1);
        // A's natural output: passthrough of input 3.0
        assert!((e.get_output("A") - 3.0).abs() < f64::EPSILON);
    }
}
