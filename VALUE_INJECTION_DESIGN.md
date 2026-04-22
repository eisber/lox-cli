# Value Injection Mechanism Design Proposal

## EXECUTIVE SUMMARY

The lox-sim engine uses a **signal-based architecture** with **dirty-flag optimization**. Current value injection (via `set_input()` and `inject_output()`) operates by:
1. Overwriting connector signal values
2. Marking blocks dirty to force re-evaluation

**Gap**: There is **no mechanism to hold a value constant** without allowing the block to override it on the next tick.

**Proposed solution**: Add an **override registry** that tracks which connectors have held values and prevents block eval from overwriting them.

---

## ARCHITECTURE OVERVIEW

### Current Flow (Every Tick)

```
set_input("MyInput", 5.0)
    ↓
signals[cid] = 5.0
dirty[block] = true
    ↓
tick(dt):
  foreach block in topo_order:
    if dirty[block]:
      outputs = block.eval(inputs, params, dt, prev_inputs)
      signals[output_cid] = outputs[i]  ← OVERWRITES any injected value!
      dirty[block] = false
```

**Problem**: If block eval runs next tick, injected value is overwritten.

---

## PROPOSED: OVERRIDE REGISTRY

### New Fields in SimEngine

```rust
pub struct SimEngine {
    // ... existing fields ...
    
    // New: Track held overrides
    overrides: HashMap<ConnectorId, OverrideState>,
}

pub enum OverrideState {
    /// One-shot: applied once, then cleared
    Snapshot {
        value: f64,
        tick_set: u64,
    },
    /// Persistent: held until explicitly cleared
    Held {
        value: f64,
        source: String,  // "api_call", "test_rule", "sensor_sim", etc.
        locked_at_tick: u64,
    },
}
```

### Modified tick() Flow

```rust
pub fn tick(&mut self, dt: f64) {
    let mut current_tick = self.tick_counter;
    self.tick_counter += 1;
    
    for block_id in &self.topo_order {
        if !self.dirty[block_id] { continue; }
        
        let inputs = gather_inputs();
        let outputs = self.blocks[block_id].eval(&inputs, params, dt, prev_inputs);
        
        // Write outputs ONLY if not overridden
        for (i, &cid) in output_cids.iter().enumerate() {
            if self.overrides.contains_key(&cid) {
                // SKIP: Value is overridden, don't update
                continue;
            }
            self.signals[cid] = outputs[i];
        }
    }
    
    // Clear one-shot snapshots
    self.overrides.retain(|cid, state| {
        match state {
            OverrideState::Snapshot { tick_set, .. } => {
                if *tick_set < current_tick {
                    return false;  // Remove expired snapshots
                }
                true
            }
            OverrideState::Held { .. } => true,
        }
    });
}
```

---

## API EXTENSIONS

### 1. Inject One-Shot Value (Snapshot)

```rust
pub fn inject_snapshot(&mut self, name: &str, value: f64) -> bool {
    if let Some(cids) = self.named_outputs.get(name) {
        for &cid in cids {
            self.signals[cid] = value;
            self.overrides.insert(cid, OverrideState::Snapshot {
                value,
                tick_set: self.tick_counter,
            });
            let block_id = self.graph.connector(cid).block_id;
            self.dirty[block_id] = true;
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

**Behavior**:
- Value applied for 1 tick
- Automatically cleared before next tick
- Useful for one-off sensor readings

### 2. Hold Value Indefinitely

```rust
pub fn hold_value(&mut self, name: &str, value: f64, source: &str) -> bool {
    if let Some(cids) = self.named_outputs.get(name) {
        for &cid in cids {
            self.signals[cid] = value;
            self.overrides.insert(cid, OverrideState::Held {
                value,
                source: source.to_string(),
                locked_at_tick: self.tick_counter,
            });
            // Mark downstream dirty so held value propagates
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
```

**Behavior**:
- Value persists across ticks
- Block's output is ignored
- Useful for simulating faulty sensors or frozen hardware

### 3. Release Held Value

```rust
pub fn release_value(&mut self, name: &str) -> bool {
    if let Some(cids) = self.named_outputs.get(name) {
        for &cid in cids {
            self.overrides.remove(&cid);
            // Mark block dirty to re-evaluate with new (non-overridden) value
            let block_id = self.graph.connector(cid).block_id;
            self.dirty[block_id] = true;
        }
        true
    } else {
        false
    }
}
```

### 4. Query Overrides

```rust
pub fn is_overridden(&self, name: &str) -> bool {
    self.named_outputs
        .get(name)
        .map(|cids| cids.iter().any(|&cid| self.overrides.contains_key(&cid)))
        .unwrap_or(false)
}

pub fn get_override(&self, name: &str) -> Option<(&str, f64)> {
    self.named_outputs
        .get(name)
        .and_then(|cids| cids.first())
        .and_then(|&cid| {
            self.overrides.get(&cid).map(|state| {
                let source = match state {
                    OverrideState::Snapshot { .. } => "snapshot",
                    OverrideState::Held { source, .. } => source,
                };
                let value = match state {
                    OverrideState::Snapshot { value, .. } => *value,
                    OverrideState::Held { value, .. } => *value,
                };
                (source, value)
            })
        })
}

pub fn clear_all_overrides(&mut self) {
    self.overrides.clear();
    self.mark_all_dirty();
}
```

---

## INTEGRATION WITH EXISTING API

### Backwards Compatibility

**Current behavior unchanged**:
- `set_input()` — Still works (injects and marks dirty)
- `inject_output()` — Still works (injects and marks dirty)
- `set_param()` — Still works (injects and marks dirty)

**New functions extend, don't replace**:
- `inject_snapshot()` — Explicit one-tick override
- `hold_value()` — New persistent override
- `release_value()` — Clear overrides
- `is_overridden()` — Query state
- `get_override()` — Inspect active override

### Interaction with dirty flags

**Overridden blocks should NOT re-evaluate**:
```rust
if self.overrides.iter().any(|cid| output_cids.contains(&cid)) {
    self.dirty[block_id] = false;  // Don't re-eval if output is overridden
}
```

Better: Skip eval entirely if all outputs are overridden.

---

## USAGE EXAMPLES

### Example 1: Simulate Sensor Failure

```rust
// Sensor stuck at temperature 20.0
engine.hold_value("Room1.Temperature", 20.0, "sensor_fault");
for tick in 0..100 {
    engine.tick(0.01);
    // Downstream blocks see constant 20.0
}

// Resume normal operation
engine.release_value("Room1.Temperature");
engine.tick(0.01);  // Block re-evaluates
```

### Example 2: One-Off Test Stimulus

```rust
// Inject motion detection for exactly one tick
engine.inject_snapshot("PresenceDetector.OutputPresence", 1.0);
engine.tick(0.01);  // Motion processed
// Snapshot automatically cleared

engine.tick(0.01);  // No motion signal
assert!(!engine.is_overridden("PresenceDetector.OutputPresence"));
```

### Example 3: Debugging with Value Freezing

```rust
let mut issues = Vec::new();
for tick in 0..1000 {
    engine.tick(0.01);
    
    if engine.get_output("House.Temperature") > 30.0 {
        // Freeze heating output to investigate
        engine.hold_value("HeatController.Output", 0.0, "debug_freeze");
        issues.push(("overtemp", tick));
        break;
    }
}

// ... inspect state, then release
engine.release_value("HeatController.Output");
```

### Example 4: Multi-Override Conflict Resolution

```rust
// Safety override: force emergency shutdown
engine.hold_value("System.MainRelay", 0.0, "emergency_shutdown");

// Later: attempt to restore (fails if emergency active)
if engine.get_override("System.MainRelay") == Some(("emergency_shutdown", 0.0)) {
    eprintln!("Cannot restore: emergency shutdown active");
}
```

---

## DESIGN RATIONALE

### Why Signal-Level, Not Block-Level?

**Alternative 1: Block-level freeze** (e.g., `freeze_block_eval(block_id)`)
- ❌ Doesn't specify which output values to freeze
- ❌ Freezes ALL outputs even if only some need override
- ❌ Can't hold one output while computing another

**Alternative 2: Connector-level overrides** (proposed)
- ✓ Precise: target individual connectors
- ✓ Flexible: freeze any output, not tied to blocks
- ✓ Compatible: works with any block type
- ✓ Debuggable: can inspect what's overridden

### Why HashMap, Not Vec?

**Option 1: Vec<Option<OverrideState>>[cid]**
- ✓ O(1) lookup
- ❌ Memory overhead for engines with 10k+ connectors but few overrides

**Option 2: HashMap<ConnectorId, OverrideState>** (proposed)
- ✓ O(1) amortized lookup
- ✓ Memory-efficient (only stores active overrides)
- ✓ Easier iteration for clearing/inspection

### Snapshot vs. Held

**Why two modes?**

1. **Snapshot** (one-tick):
   - Use case: Inject stimulus, observe response
   - Auto-clearing prevents stale data
   - Less manual cleanup needed

2. **Held** (persistent):
   - Use case: Simulate broken sensor or frozen hardware
   - Explicit release prevents accidental behavior changes
   - Requires intentional management (safer)

---

## IMPLEMENTATION CHECKLIST

- [ ] Add `overrides: HashMap<ConnectorId, OverrideState>` to SimEngine
- [ ] Add `tick_counter: u64` to track tick number (for snapshot expiry)
- [ ] Modify `tick()` to skip writing outputs for overridden connectors
- [ ] Implement `inject_snapshot(name, value)`
- [ ] Implement `hold_value(name, value, source)`
- [ ] Implement `release_value(name)`
- [ ] Implement `is_overridden(name)` and `get_override(name)`
- [ ] Implement `clear_all_overrides()`
- [ ] Add tests for one-shot snapshots
- [ ] Add tests for held values across multiple ticks
- [ ] Add tests for release behavior
- [ ] Add tests for interaction with dirty flags
- [ ] Document in API guide

---

## TESTING STRATEGY

### Unit Tests

```rust
#[test]
fn snapshot_expires_after_one_tick() {
    let mut e = create_engine();
    e.inject_snapshot("Output", 42.0);
    assert_eq!(e.signal(output_cid), 42.0);
    
    e.tick(0.01);  // Snapshot applied
    assert_eq!(e.signal(output_cid), 42.0);
    
    e.tick(0.01);  // Snapshot cleared
    assert!(e.signal(output_cid) != 42.0);  // Back to computed value
}

#[test]
fn held_value_persists() {
    let mut e = create_engine();
    e.hold_value("Output", 42.0, "test");
    assert!(e.is_overridden("Output"));
    
    for _ in 0..10 {
        e.tick(0.01);
        assert_eq!(e.signal(output_cid), 42.0);
    }
    
    e.release_value("Output");
    assert!(!e.is_overridden("Output"));
}

#[test]
fn override_blocks_eval() {
    // Create block with side effect (state increment)
    let mut e = create_engine_with_counter();
    
    // Hold output constant
    e.hold_value("Counter.Output", 99.0, "test");
    e.tick(0.01);
    assert_eq!(e.signal(counter_output), 99.0);
    
    // Block's internal state still advances even though output is frozen
    let state = e.block_state(counter_block_id).unwrap();
    
    e.release_value("Counter.Output");
    e.tick(0.01);
    // Output now reflects actual state
}
```

### Integration Tests

```rust
#[test]
fn sensor_failure_simulation() {
    // Normal operation
    let mut engine = parse_engine("climate.Loxone");
    for _ in 0..10 {
        engine.set_input("Thermostat.SetPoint", 20.0);
        engine.tick(0.01);
    }
    let normal_heating = engine.get_output("Heater.Output");
    
    // Simulate stuck sensor
    engine.hold_value("TempSensor.Reading", 15.0, "stuck");
    for _ in 0..10 {
        engine.tick(0.01);
    }
    let heating_max = engine.get_output("Heater.Output");
    assert!(heating_max > normal_heating);
    
    // Resume
    engine.release_value("TempSensor.Reading");
    engine.tick(0.01);
    assert!(engine.get_output("Heater.Output") < heating_max);
}
```

---

## PERFORMANCE IMPLICATIONS

**Memory**:
- HashMap: O(number of overridden connectors)
- Typical: < 1KB for most simulations

**CPU**:
- Lookup: O(1) per output connector
- Insertion/removal: O(1) amortized
- Negligible impact on tick time

---

## FUTURE EXTENSIONS

1. **Priority-based conflicts**:
   ```rust
   pub enum OverrideSource {
       UserAPI(priority: u8),
       TestRule { priority: u8 },
       EmergencyShutdown,  // Always wins
   }
   ```

2. **Time-limited holds**:
   ```rust
   pub fn hold_value_until(&mut self, name: &str, value: f64, 
                           until_tick: u64, source: &str) -> bool
   ```

3. **Conditional overrides**:
   ```rust
   pub fn hold_if(&mut self, name: &str, condition: Box<dyn Fn(&SimEngine) -> bool>,
                  value: f64, source: &str) -> bool
   ```

4. **Audit trail**:
   ```rust
   pub fn override_history(&self) -> Vec<(u64, ConnectorId, &str, f64)>
   ```

---

## SUMMARY TABLE

| Feature | Current API | Proposed |
|---------|-------------|----------|
| Inject one-shot value | `set_input()` | `inject_snapshot()` |
| Hold value indefinitely | ❌ Not possible | `hold_value()` |
| Release override | ❌ Not possible | `release_value()` |
| Query override state | ❌ Not possible | `is_overridden()`, `get_override()` |
| Clear all overrides | ❌ Not possible | `clear_all_overrides()` |
| Prevent block eval | ❌ Not directly | ✓ Via override registry |
| Track override source | ❌ No | ✓ String annotation |

