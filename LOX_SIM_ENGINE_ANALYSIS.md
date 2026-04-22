   - This ensures injected values propagate immediately to downstream blocks
### Workflow Example
```rust
// Setup
let mut g = SimGraph::new();
let vi = g.add_block("Sensor", Box::new(VirtualIn), &["I1"], &["Q"], &[]);
let light = g.add_block("Light", Box::new(PassThrough), &["I1"], &["Q"], &[]);
g.add_wire(g.find_connector(vi, "Q").unwrap(), g.find_connector(light, "I1").unwrap()).unwrap();
let mut engine = SimEngine::new(g);
// Injection
engine.set_input("Sensor", 1.0);  // Targets "Sensor.Q" (output) → marks Light dirty
engine.tick(0.1);
assert_eq!(engine.get_output("Light"), 1.0);  // Value propagates
```
---
## 10. BLOCK EVALUATION LOGIC
### Execution Path in tick() (lines 272-349)
**For each block in topological order:**
1. **Skip if not dirty**:
   ```rust
   if !self.dirty[block_id] { continue; }
   ```
2. **Gather inputs**:
   ```rust
   let inputs: Vec<f64> = ei.input_sources
       .iter()
       .map(|&(src, is_fb)| {
           if is_fb {
               self.prev_signals[src]    // Feedback: use prev-tick value
           } else {
               self.signals[src]         // Normal: use current value
           }
       })
       .collect();
   ```
3. **Gather parameters**:
   ```rust
   let params: Vec<f64> = ei.param_cids
       .iter()
       .map(|&cid| self.signals[cid])
       .collect();
   ```
4. **Gather previous-tick inputs**:
   ```rust
   let prev_inputs: Vec<f64> = ei.input_sources
       .iter()
       .map(|&(src, _)| self.prev_signals[src])
       .collect();
   ```
5. **Call block eval**:
   ```rust
   let outputs = self.blocks[block_id].eval(&inputs, &params, dt, &prev_inputs);
   ```
6. **Write outputs**:
   ```rust
   for (i, &cid) in ei.output_cids.iter().enumerate() {
       if let Some(&val) = outputs.get(i) {
           self.signals[cid] = val;
       }
   }
   ```
7. **Propagate dirty if output changed**:
   ```rust
   if (self.signals[cid] - val).abs() > f64::EPSILON {
       any_changed = true;
   }
   // If any_changed:
   for &ds in &self.downstream[block_id] {
       self.dirty[ds] = true;
   }
   ```
8. **Mark block clean**:
   ```rust
   self.dirty[block_id] = false;
   ```
### Topological Order
- Pre-computed via DFS-based topological sort in `graph.rs:164-223`
- Ensures all upstream blocks evaluate before downstream ones
- Handles feedback loops by identifying back-edges and using prev-tick values
---
## 11. EXISTING OVERRIDE/FREEZE MECHANISMS
### 1. `set_input()` — Value Injection
```rust
pub fn set_input(&mut self, name: &str, value: f64) -> bool
```
- **Effect**: Override a connector's value and mark block dirty
- **Scope**: Input connectors (or source block outputs via bare name)
### 2. `inject_output()` — Output Injection
```rust
pub fn inject_output(&mut self, name: &str, value: f64) -> bool
```
- **Effect**: Override an output connector's value
- **Scope**: Output connectors directly
- **Use case**: Simulate sensor readings without wiring a source block
### 3. `set_param()` — Parameter Setting
```rust
pub fn set_param(&mut self, block_name: &str, key: &str, value: f64) -> bool
```
- **Effect**: Override a parameter value and mark block dirty
- **Scope**: Parameter connectors only
### 4. `mark_all_dirty()` — Force Re-evaluation
```rust
pub fn mark_all_dirty(&mut self)
```
- **Effect**: Set all blocks dirty to force complete re-evaluation next tick
- **Use case**: After state restore or config change
### 5. Block State Save/Restore
```rust
pub fn block_state(&self, block_id: BlockId) -> Option<Vec<u8>>
pub fn restore_block_state(&mut self, block_id: BlockId, data: &[u8])
```
- **Effect**: Snapshot and restore stateful block internals (e.g., Counter state)
- **Side effect**: Marks block dirty after restore
### NO EXISTING "FREEZE" MECHANISM
- There is **no mechanism to prevent a block from evaluating** (other than leaving it clean)
- There is **no mechanism to hold outputs constant** while bypassing eval
- There is **no mechanism to override internal state mid-tick** (only pre-tick via restore)
---
## 12. TEST PATTERNS
### Location
`lox-sim/tests/integration.rs` and `lox-sim/src/engine.rs:634-1215`
### Basic set_input() Usage
```rust
#[test]
fn single_passthrough() {
    let mut g = SimGraph::new();
    g.add_block("P", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let mut e = SimEngine::new(g);
    e.set_input("P", 42.0);
    e.tick(0.1);
    assert!((e.get_output("P") - 42.0).abs() < f64::EPSILON);
}
```
### Chain Propagation
```rust
#[test]
fn chain_propagation() {
    // A → B → C (PassThrough chain)
    let (mut e, _a, _b, _c) = simple_chain();
    e.set_input("A", 7.0);
    e.tick(0.1);
    assert!((e.get_output("C") - 7.0).abs() < f64::EPSILON);
}
```
### Dirty Flag Optimization
```rust
#[test]
fn dirty_flag_skips_unchanged_blocks() {
    // Two blocks wired in series, tracking eval count
    let (ta, counter_a) = TrackingPassThrough::new();
    let (tb, counter_b) = TrackingPassThrough::new();
    // ... build graph ...
    
    e.set_input("A", 5.0);
    e.tick(0.1);
    assert_eq!(counter_a.load(Ordering::SeqCst), 1);
    assert_eq!(counter_b.load(Ordering::SeqCst), 1);  // Both eval'd
    // Second tick, no change → both skipped
    e.tick(0.1);
    assert_eq!(counter_a.load(Ordering::SeqCst), 1);  // Still 1 (skipped)
    assert_eq!(counter_b.load(Ordering::SeqCst), 1);  // Still 1 (skipped)
}
```
### Edge Detection (RisingEdge)
```rust
#[test]
fn rising_edge_detection() {
    let mut g = SimGraph::new();
    let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let edge = g.add_block("Edge", Box::new(RisingEdge), &["I1"], &["Q"], &[]);
    g.add_wire(g.find_connector(src, "Q").unwrap(), g.find_connector(edge, "I1").unwrap()).unwrap();
    let mut e = SimEngine::new(g);
    // Tick 1: 0→1 transition → edge fires
    e.set_input("Src", 1.0);
    e.tick(0.1);
    assert!((e.get_output("Edge") - 1.0).abs() < f64::EPSILON);
    // Tick 2: stays at 1 → no edge
    e.tick(0.1);
    assert!((e.get_output("Edge")).abs() < f64::EPSILON);
}
```
### Feedback Loops
```rust
#[test]
fn feedback_loop_uses_previous_tick() {
    // A(Add) → B(PassThrough) → A.I2 (feedback)
    let mut g = SimGraph::new();
    let a = g.add_block("A", Box::new(Add), &["I1", "I2"], &["Q"], &[]);
    let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    g.add_wire(g.find_connector(a, "Q").unwrap(), g.find_connector(b, "I1").unwrap()).unwrap();
    g.add_wire(g.find_connector(b, "Q").unwrap(), g.find_connector(a, "I2").unwrap()).unwrap();  // FEEDBACK
    let mut e = SimEngine::new(g);
    // Tick 1: A.I1=5, feedback A.I2=0 (default) → A.Q=5, B.Q=5
    e.set_input("A.I1", 5.0);
    e.tick(0.1);
    assert!((e.get_output("A") - 5.0).abs() < f64::EPSILON);
    // Tick 2: feedback=5 (from prev) → A.Q=5+5=10, B.Q=10
    e.set_input("A.I1", 5.0);
    e.tick(0.1);
    assert!((e.get_output("A") - 10.0).abs() < f64::EPSILON);
}
```
### State Save/Restore
```rust
#[test]
fn block_state_save_restore() {
    let mut e = SimEngine::new(graph_with_counter);
    
    // Count to 3
    for _ in 0..3 {
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        e.set_input("Clk", 0.0);
        e.tick(0.1);
    }
    assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);
    
    let state = e.block_state(ctr).unwrap();  // Save
    
    // Count to 5
    for _ in 0..2 {
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        e.set_input("Clk", 0.0);
        e.tick(0.1);
    }
    assert!((e.get_output("Ctr") - 5.0).abs() < f64::EPSILON);
    
    e.restore_block_state(ctr, &state);  // Restore to 3
    e.set_input("Clk", 0.0);
    e.tick(0.1);
    assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);
}
```
---
## SUMMARY: DESIGN CONSTRAINTS FOR VALUE INJECTION
1. **Signals are indexed by ConnectorId** — a connector's value is `signals[cid]`
2. **Dirty flags control evaluation** — blocks are skipped unless dirty
3. **set_input()** and **inject_output()** both work by:
   - Setting `signals[cid] = value`
   - Marking the owner block dirty
   - (For output connectors, also marking downstream blocks dirty)
4. **Source blocks** (VirtualIn, SysVar) are treated specially:
   - Bare name maps to both inputs and outputs
   - Allows `set_input("VirtualIn", 5.0)` to drive the output
5. **Feedback wires** are identified and use `prev_signals` instead of current `signals`
6. **Edge-sensitive blocks** re-examine their history context every tick
7. **No "freeze" mechanism exists** — blocks must either be evaluated or skipped
8. **Topological order is enforced** — all input blocks evaluate before output blocks
---
## PROPOSED ARCHITECTURE FOR VALUE INJECTION
Given this analysis, a value injection mechanism should:
1. **Define an "override" flag per connector** (or globally per block)
   - When set, `signals[cid]` is not updated by block eval
   - Instead, the override value is used
2. **Track override sources** (API call, simulation rule, external input, etc.)
   - Useful for debugging which override is active
3. **Support both snapshot and held overrides**:
   - Snapshot: Override lasts one tick (one-shot injection)
   - Held: Override persists until cleared (true "freeze")
4. **Integrate with dirty flags**:
   - Overriding a signal should mark downstream blocks dirty
   - (But don't re-eval the overridden block itself)
5. **Priority system for multiple overrides**:
   - resolve conflicts (e.g., sensor override vs. user override)
EOF
cat /tmp/lox_sim_analysis.md
                EC=$?
                echo "___BEGIN___COMMAND_DONE_MARKER___$EC"
            }
___BEGIN___COMMAND_OUTPUT_MARKER___
# LOX-SIM Engine Architecture Analysis

## 1. ENGINE STRUCT AND FIELDS

### Location
`lox-sim/src/engine.rs:36-49` (SimEngine struct)

### Full Struct Definition
```rust
pub struct SimEngine {
    graph: SimGraph,                                    // Block topology & connectors
    blocks: Vec<Box<dyn Block>>,                       // Block implementations
    signals: Vec<f64>,                                 // Current tick signal values (f64 = Signal)
    prev_signals: Vec<f64>,                            // Previous tick values (for feedback/edge-detection)
    dirty: Vec<bool>,                                  // Per-block dirty flags (needs re-eval)
    topo_order: Vec<BlockId>,                          // Topological sort order
    eval_info: Vec<BlockEvalInfo>,                     // Pre-computed per-block eval metadata
    downstream: Vec<Vec<BlockId>>,                     // downstream[bid] = blocks depending on bid's outputs
    named_inputs: HashMap<String, Vec<ConnectorId>>,  // "BlockName" or "BlockName.Key" → input cids
    named_outputs: HashMap<String, Vec<ConnectorId>>, // "BlockName" or "BlockName.Key" → output cids
    profiler: Option<SimProfiler>,                     // Optional profiling
}
```

### Key Field Details

**BlockEvalInfo** (struct at lines 15-26):
```rust
struct BlockEvalInfo {
    input_sources: Vec<(ConnectorId, bool)>,   // (source_cid, is_feedback_wire)
    output_cids: Vec<ConnectorId>,             // Connector IDs for outputs
    param_cids: Vec<ConnectorId>,              // Connector IDs for parameters
    edge_sensitive: bool,                      // Whether block's output depends on prev_inputs
    last_prev_inputs: Vec<f64>,                // Previous-tick inputs (for dirty detection)
}
```

---

## 2. THE `tick()` METHOD - FULL BREAKDOWN

### Location
`lox-sim/src/engine.rs:265-387`

### High-Level Flow
```
tick(dt: f64):
  1. For each block in topological order:
     a. Skip if dirty flag is false (no input change)
     b. Gather inputs from signal array (or prev_signals if feedback wire)
     c. Gather parameters
     d. Gather previous-tick inputs (for edge detection)
     e. Call block.eval(inputs, params, dt, prev_inputs) → outputs
     f. Write outputs to signal array
     g. If any output changed (by ε), mark downstream blocks dirty
  
  2. Snapshot signals: prev_signals ← signals (for next tick's feedback)
  
  3. Re-mark edge-sensitive blocks dirty if their prev_inputs changed
     (ensures edge detectors re-eval when history context changes)
```

### Actual Code (Core Loop)
```rust
pub fn tick(&mut self, dt: f64) {
    // Lines 272-349: Main evaluation loop
    for idx in 0..self.topo_order.len() {
        let block_id = self.topo_order[idx];

        if !self.dirty[block_id] {
            // SKIP: No input change → no need to re-evaluate
            continue;
        }

        let ei = &self.eval_info[block_id];

        // Gather current inputs
        // Lines 285-295: Read from signals[] or prev_signals[] (if feedback)
        let inputs: Vec<f64> = ei
            .input_sources
            .iter()
            .map(|&(src, is_fb)| {
                if is_fb {
                    self.prev_signals[src]      // FEEDBACK: Use previous-tick value
                } else {
                    self.signals[src]           // NORMAL: Use current value
                }
            })
            .collect();

        // Gather parameters (always from current signals)
        let params: Vec<f64> = ei
            .param_cids.iter()
            .map(|&cid| self.signals[cid])
            .collect();

        // Gather previous-tick inputs (for edge detection like RisingEdge)
        let prev_inputs: Vec<f64> = ei
            .input_sources
            .iter()
            .map(|&(src, _)| self.prev_signals[src])
            .collect();

        // Evaluate block (Lines 309-327)
        let outputs = self.blocks[block_id].eval(&inputs, &params, dt, &prev_inputs);

        // Write outputs to signals array
        // Lines 333-347: Propagate dirty flags downstream if value changed
        let mut any_changed = false;
        for (i, &cid) in ei.output_cids.iter().enumerate() {
            if let Some(&val) = outputs.get(i) {
                if (self.signals[cid] - val).abs() > f64::EPSILON {
                    any_changed = true;
                }
                self.signals[cid] = val;
            }
        }
        
        // DIRTY PROPAGATION: Mark downstream blocks dirty
        if any_changed {
            for &ds in &self.downstream[block_id] {
                self.dirty[ds] = true;
            }
        }

        self.dirty[block_id] = false;  // Block now clean
    }

    // Lines 352-353: Snapshot signals for next tick's previous-value lookups
    self.prev_signals.copy_from_slice(&self.signals);

    // Lines 355-377: Re-mark edge-sensitive blocks dirty if prev_inputs changed
    // (ensures edge detectors trigger on input state changes)
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
}
```

### Key Insight: DIRTY FLAG STRATEGY
- **Initial state**: All blocks dirty on first tick
- **Propagation**: When a block's output changes, all downstream blocks are marked dirty
- **Edge-sensitive blocks**: Re-marked dirty if their historical input context (prev_inputs) changed
- **Skip optimization**: Clean blocks are skipped entirely

---

## 3. THE `set_input()` METHOD

### Location
`lox-sim/src/engine.rs:191-210`

### Signature & Purpose
```rust
pub fn set_input(&mut self, name: &str, value: f64) -> bool
```
Inject a value into a named input connector. Marks the owner block (and downstream) dirty.

### Full Code
```rust
pub fn set_input(&mut self, name: &str, value: f64) -> bool {
    if let Some(cids) = self.named_inputs.get(name) {
        let cids = cids.clone();
        for &cid in &cids {
            self.signals[cid] = value;
            let block_id = self.graph.connector(cid).block_id;
            self.dirty[block_id] = true;
            
            // Special case: For OUTPUT connectors (e.g., SysVar, VirtualIn),
            // also mark downstream blocks dirty so value propagates immediately
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
```

### Behavior Details
1. **Name resolution**: Looks up connector IDs via `named_inputs` map
2. **Signal update**: Sets `signals[cid] = value`
3. **Dirty marking**: 
   - Marks the block (owner of the connector) dirty
   - **If the connector is an OUTPUT** (source blocks like VirtualIn, SysVar):
     - Also marks all downstream blocks dirty
     - This ensures values injected into source blocks propagate immediately
4. **Returns**: `true` if name found, `false` otherwise

### Example Usage (from tests, line 670-672)
```rust
e.set_input("P", 42.0);  // Named "P" (first input of PassThrough block)
e.tick(0.1);
assert!((e.get_output("P") - 42.0).abs() < f64::EPSILON);
```

---

## 4. THE `inject_output()` METHOD

### Location
`lox-sim/src/engine.rs:217-232`

### Signature & Purpose
```rust
pub fn inject_output(&mut self, name: &str, value: f64) -> bool
```
Inject a value **directly into a named output connector** (e.g., PresenceDetector.OutputPresence).
Useful for simulating sensor readings or hardware inputs.

### Full Code
```rust
pub fn inject_output(&mut self, name: &str, value: f64) -> bool {
    if let Some(cids) = self.named_outputs.get(name) {
        let cids = cids.clone();
        for &cid in &cids {
            self.signals[cid] = value;
            let block_id = self.graph.connector(cid).block_id;
            self.dirty[block_id] = true;
            // Mark downstream blocks dirty so value propagates
            for &ds in &self.downstream[block_id] {
                self.dirty[ds] = true;
            }
        }
        true
    } else {
        false
    }
}
```

### How It Differs from `set_input()`
- **set_input()**: Targets input connectors (or source block outputs via bare name)
- **inject_output()**: Directly targets output connectors
- Both mark the owner block and downstream blocks dirty
- **Key difference**: `inject_output()` always marks downstream blocks (no conditional check)

### Use Case Example
```rust
// Simulate a presence detector sensing motion
engine.inject_output("PresenceDetector.OutputPresence", 1.0);
engine.tick(0.1);
// Downstream blocks that depend on this sensor now receive the injected value
```

---

## 5. SIGNALS ARRAY, DIRTY FLAGS, AND DOWNSTREAM ADJACENCY

### Signals Array
```rust
signals: Vec<f64>     // One f64 per ConnectorId (index = cid)
prev_signals: Vec<f64> // Snapshot from end of previous tick
```
- **signals[cid]** = current value on a connector
- **Indexed by ConnectorId** (a `usize` alias)
- **All connectors carry f64** (digital = 0.0/1.0, analog = arbitrary f64)

### Dirty Flags
```rust
dirty: Vec<bool>      // One bool per BlockId (index = block_id)
```
- **dirty[block_id] = true** → block will be evaluated this tick
- **dirty[block_id] = false** → block is skipped (optimization)
- **Initial**: All blocks dirty on first tick
- **Set to true when**:
  - `set_input()` or `inject_output()` called
  - Block's upstream input changes (propagated from upstream eval)
  - Edge-sensitive block's prev_inputs context changes
- **Set to false after** block evaluation (line 349)

### Downstream Adjacency
```rust
downstream: Vec<Vec<BlockId>>   // downstream[block_id] = blocks depending on block_id
```
- **Pre-computed in `SimEngine::new()` (lines 93-102)**:
  ```rust
  let mut downstream: Vec<Vec<BlockId>> = vec![Vec::new(); n_blocks];
  let mut seen_edges: HashSet<(BlockId, BlockId)> = HashSet::new();
  for &(from_cid, to_cid) in graph.wires() {
      let src = graph.connector(from_cid).block_id;
      let dst = graph.connector(to_cid).block_id;
      if src != dst && seen_edges.insert((src, dst)) {
          downstream[src].push(dst);  // Record the edge
      }
  }
  ```
- **Used for dirty propagation**: When block's output changes, all blocks in `downstream[block_id]` are marked dirty

---

## 6. BLOCK TRAIT AND EVAL SIGNATURE

### Location
`lox-sim/src/blocks/mod.rs:141-165`

### Trait Definition
```rust
pub trait Block: Send + Sync + BlockClone {
    /// Evaluate the block for one tick.
    fn eval(
        &mut self,
        inputs: &[Signal],        // Current-tick input values
        params: &[Signal],        // Current-tick parameter values
        dt: f64,                  // Time delta (not always used)
        prev_inputs: &[Signal],   // Previous-tick input values (for edge detection)
    ) -> Vec<Signal>;            // Output values (one per output connector)

    /// Serialize internal state (for snapshot/restore). Returns None for stateless blocks.
    fn state(&self) -> Option<Vec<u8>>;

    /// Restore internal state from serialized bytes.
    fn restore(&mut self, state: &[u8]);

    /// Block type name (e.g., "PassThrough", "Add", "RisingEdge")
    fn block_type(&self) -> &str;

    /// Whether the block's output depends on prev_inputs.
    /// Edge-sensitive blocks are re-marked dirty if their prev_inputs context changed.
    fn is_edge_sensitive(&self) -> bool {
        false  // Default: stateless blocks
    }
}
```

### Key Method: eval()
**Contract**:
- Receives `inputs` (current tick) and `prev_inputs` (last tick)
- Must return one f64 per output connector
- For stateless blocks, `prev_inputs` is ignored
- For edge-sensitive blocks (RisingEdge, Counter, etc.), `prev_inputs` is used to detect transitions

### Example Implementations

**PassThrough** (lines 168-191):
```rust
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

    fn state(&self) -> Option<Vec<u8>> { None }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str { "PassThrough" }
}
```

**RisingEdge** (lines 246-270):
```rust
#[derive(Clone, Copy)]
pub struct RisingEdge;

impl Block for RisingEdge {
    fn eval(&mut self, inputs: &[Signal], _p: &[Signal], _dt: f64, prev: &[Signal]) -> Vec<Signal> {
        let cur = inputs.first().copied().unwrap_or(0.0);
        let prv = prev.first().copied().unwrap_or(0.0);
        vec![bool_signal(!is_high(prv) && is_high(cur))]  // 1.0 only on low→high transition
    }

    fn state(&self) -> Option<Vec<u8>> { None }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str { "RisingEdge" }

    fn is_edge_sensitive(&self) -> bool {
        true  // This block depends on prev_inputs for transition detection
    }
}
```

**Counter** (with state):
```rust
pub struct Counter {
    count: f64,
}

impl Block for Counter {
    fn eval(&mut self, inputs: &[Signal], _p: &[Signal], _dt: f64, prev: &[Signal]) -> Vec<Signal> {
        let cur = inputs.first().copied().unwrap_or(0.0);
        let prv = prev.first().copied().unwrap_or(0.0);
        
        // Increment on rising edge
        if !is_high(prv) && is_high(cur) {
            self.count += 1.0;
        }
        vec![self.count]
    }

    fn state(&self) -> Option<Vec<u8>> {
        Some(serialize_f64s(&[self.count]))
    }

    fn restore(&mut self, state: &[u8]) {
        if let Some(vals) = deserialize_f64s(state, 1) {
            self.count = vals[0];
        }
    }

    fn block_type(&self) -> &str { "Counter" }
    fn is_edge_sensitive(&self) -> bool { true }
}
```

---

## 7. GRAPH STRUCTURE

### Location
`lox-sim/src/graph.rs`

### SimGraph Struct (lines 76-84)
```rust
pub struct SimGraph {
    pub(crate) blocks: Vec<BlockInfo>,
    pub(crate) block_impls: Vec<Box<dyn Block>>,
    pub(crate) connectors: Vec<ConnectorInfo>,
    pub(crate) wires: Vec<(ConnectorId, ConnectorId)>,
    pub(crate) input_source: HashMap<ConnectorId, ConnectorId>,
    block_by_name: HashMap<String, BlockId>,
    blocks_by_name: HashMap<String, Vec<BlockId>>,
}
```

### BlockInfo Struct (lines 44-53)
```rust
pub struct BlockInfo {
    pub id: BlockId,                           // Unique block ID (usize)
    pub name: String,                          // Block name ("Jalousie 1", etc.)
    pub room: Option<String>,                  // Optional room name
    pub inputs: Vec<ConnectorId>,              // Input connector IDs
    pub outputs: Vec<ConnectorId>,             // Output connector IDs
    pub params: Vec<ConnectorId>,              // Parameter connector IDs
}
```

### ConnectorInfo Struct (from types.rs, lines 18-26)
```rust
pub struct ConnectorInfo {
    pub id: ConnectorId,                       // Unique connector ID (usize)
    pub key: String,                           // Name within block ("I1", "Q", "Factor", etc.)
    pub dir: ConnectorDir,                     // Input, Output, or Parameter
    pub block_id: BlockId,                     // Owning block
    pub default_value: Signal,                 // Default signal value (0.0)
}
```

### ConnectorId Explained
```rust
pub type ConnectorId = usize;   // Index into global signals[] and prev_signals[] arrays
```
- **A ConnectorId is just a usize index** into the engine's signal arrays
- Each block's inputs/outputs/parameters are assigned unique ConnectorIds
- When wiring blocks, you connect an output ConnectorId to an input ConnectorId

### Input Source Mapping
```rust
pub(crate) input_source: HashMap<ConnectorId, ConnectorId>,
```
- **input_source[input_cid] = output_cid** → the wire feeding this input
- Used during `tick()` to find where to read input values from (line 72-78 of engine.rs)

### Wiring Array
```rust
pub(crate) wires: Vec<(ConnectorId, ConnectorId)>,
```
- List of all wires: `(output_cid, input_cid)` pairs
- Used to detect feedback loops in topological sort

### Example Graph Construction
```rust
let mut g = SimGraph::new();

// Add blocks
let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
let b = g.add_block("B", Box::new(Add), &["I1", "I2"], &["Q"], &[]);

// Wire them: A.Q → B.I1
let a_q = g.find_connector(a, "Q").unwrap();   // Output ConnectorId of A
let b_i1 = g.find_connector(b, "I1").unwrap(); // Input ConnectorId of B
g.add_wire(a_q, b_i1).unwrap();                // Register the wire

let engine = SimEngine::new(g);                // Build engine (topological sort happens here)
```

---

## 8. NAMED_INPUTS AND NAMED_OUTPUTS MAPS

### Location
`lox-sim/src/engine.rs:104-168` (construction in `SimEngine::new()`)

### Maps
```rust
named_inputs: HashMap<String, Vec<ConnectorId>>,
named_outputs: HashMap<String, Vec<ConnectorId>>,
```

### Resolution Rules
The engine builds these maps from block info (lines 104-168):

1. **Dotted names** (most specific):
   - `"BlockName.ConnectorKey"` → connectors on that block
   - Example: `"A.I1"`, `"LightController.Brightness"`

2. **Room-qualified dotted names**:
   - `"BlockName [RoomName].ConnectorKey"` → connectors with room context
   - Example: `"Temperature [Kitchen].Q"`

3. **Bare block name** (coarse):
   - For **source blocks** (SysVar, VirtualIn):
     - `"BlockName"` → **all inputs AND outputs** (special case: allows injection)
   - For **other blocks**:
     - `"BlockName"` → first input connector (for `set_input`)
     - `"BlockName"` → first output connector (for `get_output`)

4. **Room-qualified bare name**:
   - `"BlockName [RoomName]"` → similar coarse resolution with room context

### Source Block Special Handling
```rust
// Line 114
let is_source = source_types.contains(&blocks[bid].block_type());

if is_source {
    // For SysVar, VirtualIn: bare name maps to ALL inputs and outputs
    for &cid in &info.inputs {
        named_inputs.entry(info.name.clone()).or_default().push(cid);
    }
    for &cid in &info.outputs {
        named_inputs.entry(info.name.clone()).or_default().push(cid);  // ALSO in inputs!
    }
}
```

This allows `set_input("VirtualIn_block", 5.0)` to drive the value directly into the output connector of a source block, which then propagates downstream.

### Usage Examples
```rust
engine.set_input("A", 5.0);              // Bare name: A's first input
engine.set_input("A.I1", 5.0);           // Dotted: A's I1 connector
engine.set_input("Temp [Kitchen]", 20.0); // Room-qualified bare
engine.set_input("Temp [Kitchen].Q", 20.0); // Room-qualified dotted

engine.get_output("B");                  // B's first output
engine.get_output("B.Q");                // B's Q connector
```

---

## 9. SOURCE BLOCKS (SysVar, VirtualIn)

### Location
- **SysVar**: `lox-sim/src/blocks/misc.rs` (stub_block! macro)
- **VirtualIn**: `lox-sim/src/blocks/io.rs` (passthrough_io_block! macro)

### Both Are Simple Pass-Through Stubs
```rust
// From io.rs, lines 14-42 (passthrough_io_block! macro)
#[derive(Clone, Copy)]
pub struct VirtualIn;

impl Block for VirtualIn {
    fn eval(&mut self, inputs: &[Signal], _params: &[Signal], _dt: f64, _prev: &[Signal]) -> Vec<Signal> {
        vec![inputs.first().copied().unwrap_or(0.0)]  // Pass first input → output
    }
    fn state(&self) -> Option<Vec<u8>> { None }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str { "VirtualIn" }
}
```

### Special Treatment in Engine
1. **Named input resolution** (line 114 of engine.rs):
   - Blocks with type "SysVar" or "VirtualIn" are flagged as source blocks
   
2. **Bare name mapping** (lines 135-141):
   - For source blocks, `named_inputs["VirtualIn"]` includes **both inputs and outputs**
   - This allows `set_input()` to directly inject into the block's output
   
3. **Downstream propagation** (lines 200-204 of set_input()):
   - When `set_input()` targets an OUTPUT connector (source blocks), it marks downstream blocks dirty
   - This ensures injected values propagate immediately to downstream blocks

### Workflow Example
```rust
// Setup
let mut g = SimGraph::new();
let vi = g.add_block("Sensor", Box::new(VirtualIn), &["I1"], &["Q"], &[]);
let light = g.add_block("Light", Box::new(PassThrough), &["I1"], &["Q"], &[]);
g.add_wire(g.find_connector(vi, "Q").unwrap(), g.find_connector(light, "I1").unwrap()).unwrap();

let mut engine = SimEngine::new(g);

// Injection
engine.set_input("Sensor", 1.0);  // Targets "Sensor.Q" (output) → marks Light dirty
engine.tick(0.1);
assert_eq!(engine.get_output("Light"), 1.0);  // Value propagates
```

---

## 10. BLOCK EVALUATION LOGIC

### Execution Path in tick() (lines 272-349)

**For each block in topological order:**

1. **Skip if not dirty**:
   ```rust
   if !self.dirty[block_id] { continue; }
   ```

2. **Gather inputs**:
   ```rust
   let inputs: Vec<f64> = ei.input_sources
       .iter()
       .map(|&(src, is_fb)| {
           if is_fb {
               self.prev_signals[src]    // Feedback: use prev-tick value
           } else {
               self.signals[src]         // Normal: use current value
           }
       })
       .collect();
   ```

3. **Gather parameters**:
   ```rust
   let params: Vec<f64> = ei.param_cids
       .iter()
       .map(|&cid| self.signals[cid])
       .collect();
   ```

4. **Gather previous-tick inputs**:
   ```rust
   let prev_inputs: Vec<f64> = ei.input_sources
       .iter()
       .map(|&(src, _)| self.prev_signals[src])
       .collect();
   ```

5. **Call block eval**:
   ```rust
   let outputs = self.blocks[block_id].eval(&inputs, &params, dt, &prev_inputs);
   ```

6. **Write outputs**:
   ```rust
   for (i, &cid) in ei.output_cids.iter().enumerate() {
       if let Some(&val) = outputs.get(i) {
           self.signals[cid] = val;
       }
   }
   ```

7. **Propagate dirty if output changed**:
   ```rust
   if (self.signals[cid] - val).abs() > f64::EPSILON {
       any_changed = true;
   }
   // If any_changed:
   for &ds in &self.downstream[block_id] {
       self.dirty[ds] = true;
   }
   ```

8. **Mark block clean**:
   ```rust
   self.dirty[block_id] = false;
   ```

### Topological Order
- Pre-computed via DFS-based topological sort in `graph.rs:164-223`
- Ensures all upstream blocks evaluate before downstream ones
- Handles feedback loops by identifying back-edges and using prev-tick values

---

## 11. EXISTING OVERRIDE/FREEZE MECHANISMS

### 1. `set_input()` — Value Injection
```rust
pub fn set_input(&mut self, name: &str, value: f64) -> bool
```
- **Effect**: Override a connector's value and mark block dirty
- **Scope**: Input connectors (or source block outputs via bare name)

### 2. `inject_output()` — Output Injection
```rust
pub fn inject_output(&mut self, name: &str, value: f64) -> bool
```
- **Effect**: Override an output connector's value
- **Scope**: Output connectors directly
- **Use case**: Simulate sensor readings without wiring a source block

### 3. `set_param()` — Parameter Setting
```rust
pub fn set_param(&mut self, block_name: &str, key: &str, value: f64) -> bool
```
- **Effect**: Override a parameter value and mark block dirty
- **Scope**: Parameter connectors only

### 4. `mark_all_dirty()` — Force Re-evaluation
```rust
pub fn mark_all_dirty(&mut self)
```
- **Effect**: Set all blocks dirty to force complete re-evaluation next tick
- **Use case**: After state restore or config change

### 5. Block State Save/Restore
```rust
pub fn block_state(&self, block_id: BlockId) -> Option<Vec<u8>>
pub fn restore_block_state(&mut self, block_id: BlockId, data: &[u8])
```
- **Effect**: Snapshot and restore stateful block internals (e.g., Counter state)
- **Side effect**: Marks block dirty after restore

### NO EXISTING "FREEZE" MECHANISM
- There is **no mechanism to prevent a block from evaluating** (other than leaving it clean)
- There is **no mechanism to hold outputs constant** while bypassing eval
- There is **no mechanism to override internal state mid-tick** (only pre-tick via restore)

---

## 12. TEST PATTERNS

### Location
`lox-sim/tests/integration.rs` and `lox-sim/src/engine.rs:634-1215`

### Basic set_input() Usage
```rust
#[test]
fn single_passthrough() {
    let mut g = SimGraph::new();
    g.add_block("P", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let mut e = SimEngine::new(g);

    e.set_input("P", 42.0);
    e.tick(0.1);
    assert!((e.get_output("P") - 42.0).abs() < f64::EPSILON);
}
```

### Chain Propagation
```rust
#[test]
fn chain_propagation() {
    // A → B → C (PassThrough chain)
    let (mut e, _a, _b, _c) = simple_chain();
    e.set_input("A", 7.0);
    e.tick(0.1);
    assert!((e.get_output("C") - 7.0).abs() < f64::EPSILON);
}
```

### Dirty Flag Optimization
```rust
#[test]
fn dirty_flag_skips_unchanged_blocks() {
    // Two blocks wired in series, tracking eval count
    let (ta, counter_a) = TrackingPassThrough::new();
    let (tb, counter_b) = TrackingPassThrough::new();
    // ... build graph ...
    
    e.set_input("A", 5.0);
    e.tick(0.1);
    assert_eq!(counter_a.load(Ordering::SeqCst), 1);
    assert_eq!(counter_b.load(Ordering::SeqCst), 1);  // Both eval'd

    // Second tick, no change → both skipped
    e.tick(0.1);
    assert_eq!(counter_a.load(Ordering::SeqCst), 1);  // Still 1 (skipped)
    assert_eq!(counter_b.load(Ordering::SeqCst), 1);  // Still 1 (skipped)
}
```

### Edge Detection (RisingEdge)
```rust
#[test]
fn rising_edge_detection() {
    let mut g = SimGraph::new();
    let src = g.add_block("Src", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    let edge = g.add_block("Edge", Box::new(RisingEdge), &["I1"], &["Q"], &[]);
    g.add_wire(g.find_connector(src, "Q").unwrap(), g.find_connector(edge, "I1").unwrap()).unwrap();

    let mut e = SimEngine::new(g);

    // Tick 1: 0→1 transition → edge fires
    e.set_input("Src", 1.0);
    e.tick(0.1);
    assert!((e.get_output("Edge") - 1.0).abs() < f64::EPSILON);

    // Tick 2: stays at 1 → no edge
    e.tick(0.1);
    assert!((e.get_output("Edge")).abs() < f64::EPSILON);
}
```

### Feedback Loops
```rust
#[test]
fn feedback_loop_uses_previous_tick() {
    // A(Add) → B(PassThrough) → A.I2 (feedback)
    let mut g = SimGraph::new();
    let a = g.add_block("A", Box::new(Add), &["I1", "I2"], &["Q"], &[]);
    let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
    g.add_wire(g.find_connector(a, "Q").unwrap(), g.find_connector(b, "I1").unwrap()).unwrap();
    g.add_wire(g.find_connector(b, "Q").unwrap(), g.find_connector(a, "I2").unwrap()).unwrap();  // FEEDBACK

    let mut e = SimEngine::new(g);

    // Tick 1: A.I1=5, feedback A.I2=0 (default) → A.Q=5, B.Q=5
    e.set_input("A.I1", 5.0);
    e.tick(0.1);
    assert!((e.get_output("A") - 5.0).abs() < f64::EPSILON);

    // Tick 2: feedback=5 (from prev) → A.Q=5+5=10, B.Q=10
    e.set_input("A.I1", 5.0);
    e.tick(0.1);
    assert!((e.get_output("A") - 10.0).abs() < f64::EPSILON);
}
```

### State Save/Restore
```rust
#[test]
fn block_state_save_restore() {
    let mut e = SimEngine::new(graph_with_counter);
    
    // Count to 3
    for _ in 0..3 {
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        e.set_input("Clk", 0.0);
        e.tick(0.1);
    }
    assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);
    
    let state = e.block_state(ctr).unwrap();  // Save
    
    // Count to 5
    for _ in 0..2 {
        e.set_input("Clk", 1.0);
        e.tick(0.1);
        e.set_input("Clk", 0.0);
        e.tick(0.1);
    }
    assert!((e.get_output("Ctr") - 5.0).abs() < f64::EPSILON);
    
    e.restore_block_state(ctr, &state);  // Restore to 3
    e.set_input("Clk", 0.0);
    e.tick(0.1);
    assert!((e.get_output("Ctr") - 3.0).abs() < f64::EPSILON);
}
```

---

## SUMMARY: DESIGN CONSTRAINTS FOR VALUE INJECTION

1. **Signals are indexed by ConnectorId** — a connector's value is `signals[cid]`

2. **Dirty flags control evaluation** — blocks are skipped unless dirty

3. **set_input()** and **inject_output()** both work by:
   - Setting `signals[cid] = value`
   - Marking the owner block dirty
   - (For output connectors, also marking downstream blocks dirty)

4. **Source blocks** (VirtualIn, SysVar) are treated specially:
   - Bare name maps to both inputs and outputs
   - Allows `set_input("VirtualIn", 5.0)` to drive the output

5. **Feedback wires** are identified and use `prev_signals` instead of current `signals`

6. **Edge-sensitive blocks** re-examine their history context every tick

7. **No "freeze" mechanism exists** — blocks must either be evaluated or skipped

8. **Topological order is enforced** — all input blocks evaluate before output blocks

---

## PROPOSED ARCHITECTURE FOR VALUE INJECTION

Given this analysis, a value injection mechanism should:

1. **Define an "override" flag per connector** (or globally per block)
   - When set, `signals[cid]` is not updated by block eval
   - Instead, the override value is used

2. **Track override sources** (API call, simulation rule, external input, etc.)
   - Useful for debugging which override is active

3. **Support both snapshot and held overrides**:
   - Snapshot: Override lasts one tick (one-shot injection)
   - Held: Override persists until cleared (true "freeze")

4. **Integrate with dirty flags**:
   - Overriding a signal should mark downstream blocks dirty
   - (But don't re-eval the overridden block itself)

5. **Priority system for multiple overrides**:
   - resolve conflicts (e.g., sensor override vs. user override)

___BEGIN___COMMAND_DONE_MARKER___0
