# LOX-SIM Engine Quick Reference

## CORE STRUCTURES

### SimEngine Fields (engine.rs:36-49)
```rust
pub struct SimEngine {
    graph: SimGraph,                                // Topology & metadata
    blocks: Vec<Box<dyn Block>>,                   // Block implementations
    signals: Vec<f64>,                             // Current connector values [cid]
    prev_signals: Vec<f64>,                        // Previous-tick values
    dirty: Vec<bool>,                              // [block_id] = needs re-eval?
    topo_order: Vec<BlockId>,                      // Evaluation order
    eval_info: Vec<BlockEvalInfo>,                 // Per-block metadata
    downstream: Vec<Vec<BlockId>>,                 // downstream[bid] → dependent blocks
    named_inputs: HashMap<String, Vec<ConnectorId>>,
    named_outputs: HashMap<String, Vec<ConnectorId>>,
    profiler: Option<SimProfiler>,
}
```

### Block Trait (blocks/mod.rs:142-165)
```rust
pub trait Block: Send + Sync + BlockClone {
    fn eval(
        &mut self,
        inputs: &[Signal],        // Current-tick values
        params: &[Signal],        // Current-tick parameters
        dt: f64,                  // Time delta
        prev_inputs: &[Signal],   // Previous-tick values (edge detection)
    ) -> Vec<Signal>;            // Outputs
    
    fn state(&self) -> Option<Vec<u8>>;    // Serialized state
    fn restore(&mut self, state: &[u8]);   // Restore state
    fn block_type(&self) -> &str;          // "PassThrough", "Add", etc.
    fn is_edge_sensitive(&self) -> bool { false }  // Uses prev_inputs?
}
```

---

## TICK FLOW (265-387)

**For each block in topological order:**
1. **Skip if not dirty** (no input change)
2. **Gather inputs** (from signals[src] or prev_signals[src] if feedback)
3. **Gather params** (from signals[param_cid])
4. **Gather prev_inputs** (from prev_signals[src])
5. **Call block.eval(inputs, params, dt, prev_inputs)** → outputs
6. **Write outputs** to signals[cid]
7. **If output changed** (by ε): mark `downstream[block_id]` blocks dirty
8. **Mark block clean**: `dirty[block_id] = false`

**After all blocks:**
9. **Snapshot signals** → prev_signals (for next tick's feedback/edge detection)
10. **Re-mark edge-sensitive blocks dirty** if their prev_inputs context changed

---

## VALUE INJECTION METHODS

### set_input() (191-210)
```rust
pub fn set_input(&mut self, name: &str, value: f64) -> bool
```
- **Targets**: Input connectors (or source block outputs via bare name)
- **Effect**: `signals[cid] = value`; mark owner block dirty
- **Special**: If cid is OUTPUT, also mark downstream blocks dirty
- **Names**: "BlockName" or "BlockName.Key" or "Block [Room].Key"

### inject_output() (217-232)
```rust
pub fn inject_output(&mut self, name: &str, value: f64) -> bool
```
- **Targets**: Output connectors directly
- **Effect**: `signals[cid] = value`; mark owner block AND downstream dirty
- **Use case**: Simulate sensor readings (PresenceDetector.OutputPresence)

### set_param() (424-435)
```rust
pub fn set_param(&mut self, block_name: &str, key: &str, value: f64) -> bool
```
- **Targets**: Parameter connectors only
- **Effect**: `signals[cid] = value`; mark owner block dirty

---

## KEY CONCEPTS

### Dirty Flags
- **dirty[block_id] = true** → Block will be evaluated this tick
- **dirty[block_id] = false** → Block skipped (optimization)
- **Set to true by**: `set_input()`, `inject_output()`, upstream output change
- **Set to false after**: Block evaluation completes

### Signals Array
```rust
signals[ConnectorId] = f64  // Current value
prev_signals[ConnectorId] = f64  // Value from end of previous tick
```
- Used to read inputs: `signals[src]` or `prev_signals[src]` if feedback wire
- Used to read params: `signals[param_cid]`
- Used for edge detection: `prev_inputs[i] = prev_signals[src]`

### Downstream Adjacency
```rust
downstream[block_id] → Vec<BlockId>  // Blocks depending on block_id
```
- Built once in `SimEngine::new()` from wiring
- Used to propagate dirty flags after output changes

### Feedback Wires
```rust
eval_info[block_id].input_sources[i] = (src_cid, is_feedback)
```
- If `is_feedback=true`: read `prev_signals[src]` (not `signals[src]`)
- Prevents combinatorial loops (uses previous-tick value)

### Edge-Sensitive Blocks
- `is_edge_sensitive() = true` → block depends on `prev_inputs`
- Examples: RisingEdge, Counter, OffDelay
- Re-marked dirty each tick if prev_inputs context changed

---

## SOURCE BLOCKS (SysVar, VirtualIn)

**Special Handling in Engine:**

1. **Type check** (line 114): `source_types = ["SysVar", "VirtualIn"]`

2. **Bare name maps to both inputs AND outputs**:
   ```rust
   for &cid in &info.inputs {
       named_inputs.entry(info.name.clone()).or_default().push(cid);
   }
   for &cid in &info.outputs {
       named_inputs.entry(info.name.clone()).or_default().push(cid);  // Also HERE!
   }
   ```

3. **set_input() to source block output marks downstream dirty**:
   ```rust
   if self.graph.connector(cid).dir == ConnectorDir::Output {
       for &ds in &self.downstream[block_id] {
           self.dirty[ds] = true;
       }
   }
   ```

**Result**: `set_input("VirtualIn", 5.0)` drives value through the block's output immediately.

---

## EXAMPLE: SIMPLE CHAIN

```rust
let mut g = SimGraph::new();
let a = g.add_block("A", Box::new(PassThrough), &["I1"], &["Q"], &[]);
let b = g.add_block("B", Box::new(PassThrough), &["I1"], &["Q"], &[]);
g.add_wire(
    g.find_connector(a, "Q").unwrap(),
    g.find_connector(b, "I1").unwrap(),
).unwrap();

let mut e = SimEngine::new(g);

// Inject value at A
e.set_input("A", 42.0);
// → signals[A.I1_cid] = 42.0
// → dirty[A] = true
// → downstream[A] = [B], so dirty[B] = true

e.tick(0.1);
// 1. A is dirty → eval: PassThrough → inputs=[42.0] → outputs=[42.0]
//    → signals[A.Q_cid] = 42.0 (already 42, so no change, don't mark downstream)
// 2. B is dirty → eval: PassThrough → inputs=[signals[A.Q_cid]] → outputs=[42.0]
//    → signals[B.Q_cid] = 42.0
// 3. Snapshot: prev_signals = signals

assert_eq!(e.get_output("B"), 42.0);
// → named_outputs["B"] = [B.Q_cid]
// → signals[B.Q_cid] = 42.0 ✓
```

---

## NO EXISTING FREEZE MECHANISM

**What exists:**
- `set_input()`, `inject_output()` — override connector values (one-shot)
- `block_state()` / `restore_block_state()` — save/restore internal state
- Dirty flags — control whether blocks re-evaluate

**What does NOT exist:**
- ❌ No "freeze output" to prevent re-evaluation while holding a value
- ❌ No "override flag" to distinguish held overrides from computed values
- ❌ No "frozen block" mode to skip eval but keep output fixed
- ❌ No tracking of where an override came from (debug info)

**Gap for Value Injection System:** A value injection mechanism needs to:
1. Override connector values **without** triggering block evaluation
2. Track **which overrides are active** (for debugging)
3. Support both **snapshot** (one-tick) and **held** (persistent) modes
4. Resolve **conflicts** when multiple overrides target same connector

---

## TOPOLOGICAL SORT

From `graph.rs:164-223`:
- **DFS-based** topological sort
- **Back-edge detection** → feedback wires (stored in `TopologicalResult.feedback_wires`)
- **Self-wires** (block → itself) are ignored
- **Result**: `topo_order[block]` is the evaluation sequence

Example: A → B → C
```
Edges: A→B, B→C
DFS visits: A, B, C (postorder reversal)
topo_order: [A, B, C]
```

With feedback: A → B → C → A (feedback)
```
DFS detects back-edge C→A
feedback_wires: {C→A}
topo_order: [A, B, C]  (still valid, C→A uses prev_signals)
```

---

## CONNECTOR ID SYSTEM

```rust
pub type ConnectorId = usize;  // Just a usize index!
```

**Global ID space:**
- Engine has one flat `signals: Vec<f64>` array
- **signals[cid]** = value of connector cid
- Each connector gets a unique cid at construction time
- Blocks don't know about cids; they just receive input/param values

**Lookup:**
```rust
let cid = graph.find_connector(block_id, "I1").unwrap();
let value = engine.signal(cid);
```

---

## CIRCULAR REFERENCE PREVENTION

**Feedback wires are explicitly identified:**
```rust
input_sources: Vec<(ConnectorId, is_feedback)>
```

**During eval:**
- Normal wire: `inputs[i] = signals[src]` (current value)
- Feedback wire: `inputs[i] = prev_signals[src]` (previous-tick value)

**This prevents:**
- A → B → A (same-tick cycle)
- A → B → C → A (longer cycle)

**Instead, the feedback value is delayed by one tick**, allowing feedback loops to stabilize.

---

## TEST STRATEGY

See `engine.rs:634-1215` and `tests/integration.rs`:

1. **Basic propagation**: Set input, tick, check output
2. **Chain propagation**: Multi-block chains
3. **Dirty flag skipping**: Track eval count with `TrackingPassThrough`
4. **Diamond merging**: Multiple paths merging at Add block
5. **Edge detection**: RisingEdge with 0→1 transitions
6. **Feedback loops**: State accumulation with previous-tick values
7. **Parameter setting**: `set_param()` with Gain block
8. **State save/restore**: Counter state serialization

---

## APPENDIX: BLOCK EXAMPLES

### PassThrough (simplest)
```rust
pub struct PassThrough;
impl Block for PassThrough {
    fn eval(&mut self, inputs: &[Signal], _p: &[Signal], _dt: f64, _prev: &[Signal]) 
        -> Vec<Signal> {
        vec![inputs.first().copied().unwrap_or(0.0)]
    }
    fn state(&self) -> Option<Vec<u8>> { None }
    fn restore(&mut self, _state: &[u8]) {}
    fn block_type(&self) -> &str { "PassThrough" }
}
```

### RisingEdge (edge-sensitive)
```rust
pub struct RisingEdge;
impl Block for RisingEdge {
    fn eval(&mut self, inputs: &[Signal], _p: &[Signal], _dt: f64, prev: &[Signal]) 
        -> Vec<Signal> {
        let cur = inputs[0];
        let prv = prev[0];
        vec![bool_signal(!is_high(prv) && is_high(cur))]  // Only true on 0→1
    }
    // ...
    fn is_edge_sensitive(&self) -> bool { true }
}
```

### Counter (stateful + edge-sensitive)
```rust
pub struct Counter { count: f64 }
impl Block for Counter {
    fn eval(&mut self, inputs: &[Signal], _p: &[Signal], _dt: f64, prev: &[Signal]) 
        -> Vec<Signal> {
        if !is_high(prev[0]) && is_high(inputs[0]) {
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
    fn is_edge_sensitive(&self) -> bool { true }
}
```

---

## FILE LOCATIONS

- **Engine**: `lox-sim/src/engine.rs` (880 lines)
- **Blocks trait**: `lox-sim/src/blocks/mod.rs` (500+ lines)
- **Graph**: `lox-sim/src/graph.rs` (400+ lines)
- **Types**: `lox-sim/src/types.rs` (97 lines)
- **Tests**: `lox-sim/tests/integration.rs`, `lox-sim/src/engine.rs:634+`

