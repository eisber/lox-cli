# LOX-SIM Value Injection Analysis - Complete Documentation

This directory contains comprehensive analysis of the lox-sim simulator engine and a detailed design proposal for a value injection mechanism.

## Documents

### 1. **EXPLORATION_SUMMARY.txt** (START HERE)
- **Length**: 17 KB
- **Purpose**: Executive summary with key findings
- **Contains**:
  - Architecture overview
  - 11 major findings with code locations
  - Gap analysis showing limitations
  - Proposed override registry solution
  - Usage examples
  - Implementation roadmap
  - Risk assessment

**Read this first if you want the high-level picture.**

### 2. **LOX_SIM_ENGINE_ANALYSIS.md** (DETAILED REFERENCE)
- **Length**: 39 KB, 1206 lines
- **Purpose**: Complete technical breakdown of engine internals
- **Sections**:
  1. Engine struct and all fields (with code)
  2. tick() method algorithm (lines 265-387)
  3. set_input() method (lines 191-210)
  4. inject_output() method (lines 217-232)
  5. Signals array, dirty flags, downstream adjacency
  6. Block trait and eval signature
  7. Graph structure (blocks, connectors, wiring)
  8. named_inputs and named_outputs maps
  9. Source blocks (SysVar, VirtualIn)
  10. Block evaluation logic
  11. Existing override/freeze mechanisms
  12. Test patterns from codebase

**Use this as a detailed reference when implementing or debugging.**

### 3. **QUICK_REFERENCE.md** (DEVELOPER CHEAT SHEET)
- **Length**: 11 KB, 400+ lines
- **Purpose**: Quick-lookup guide for common tasks
- **Contains**:
  - Core structures (SimEngine, Block trait)
  - Tick flow diagram
  - Key concepts (dirty flags, signals array, downstream adjacency)
  - Source block special handling
  - Simple examples
  - Block implementation examples (PassThrough, RisingEdge, Counter)
  - File locations

**Use this while writing code or making quick checks.**

### 4. **VALUE_INJECTION_DESIGN.md** (IMPLEMENTATION BLUEPRINT)
- **Length**: 14 KB, 350+ lines
- **Purpose**: Design proposal for new override registry system
- **Sections**:
  - Executive summary of the gap
  - Proposed OverrideState enum and registry
  - Modified tick() flow
  - 4 new API methods with full code:
    - inject_snapshot(name, value)
    - hold_value(name, value, source)
    - release_value(name)
    - Query methods: is_overridden(), get_override()
  - Usage examples (sensor failure, test stimulus, debug, etc.)
  - Design rationale
  - Implementation checklist
  - Test strategy
  - Performance analysis
  - Future extensions
  - Summary table of changes

**Use this as the blueprint for implementing the value injection mechanism.**

## Key Architecture Points

### Signals-Based Design
- **signals[ConnectorId]** = current value (f64)
- **prev_signals[ConnectorId]** = previous-tick value
- Connector IDs are global indices into these arrays
- All values flow through the signals array

### Dirty-Flag Optimization
- **dirty[BlockId]** = true if block needs re-evaluation
- Blocks are skipped if inputs haven't changed
- Upstream output changes mark downstream blocks dirty
- Edge-sensitive blocks (RisingEdge, Counter) re-marked if prev_inputs context changes

### Topological Ordering
- Blocks evaluated in strict dependency order
- Feedback wires identified during topological sort
- Feedback uses **prev_signals[]** (not current) to prevent cycles

### Value Injection Gap
- Current: `set_input()` and `inject_output()` overwrite values one-time
- Problem: Block re-evaluation overwrites injected values
- Solution: Add override registry to prevent block output writes

## File References

### Core Engine (lox-sim/src/)
```
engine.rs          - SimEngine struct, tick(), set_input(), inject_output()
blocks/mod.rs      - Block trait, PassThrough, RisingEdge, block factory
blocks/io.rs       - VirtualIn, VirtualOut (pass-through I/O blocks)
blocks/misc.rs     - SysVar (system variable proxy)
graph.rs           - SimGraph, topological sort, wiring
types.rs           - Signal, ConnectorId, BlockId type definitions
```

### Tests
```
tests/integration.rs       - Integration tests with real configs
engine.rs:634-1215         - Unit tests in source file
```

## Implementation Roadmap

**Phase 1 (1-2 days)**: Core override registry
- OverrideState enum
- Modified tick() logic
- Snapshot cleanup

**Phase 2 (1 day)**: API methods
- inject_snapshot(), hold_value(), release_value()
- is_overridden(), get_override(), clear_all_overrides()

**Phase 3 (2-3 days)**: Testing
- Unit tests, integration tests, edge cases
- Performance validation

**Phase 4 (1 day)**: Documentation
- API docs, examples, migration guide

**Total**: 5-7 days

## Quick Start: Understanding the Engine

1. Read **EXPLORATION_SUMMARY.txt** for the big picture (30 min)
2. Study **QUICK_REFERENCE.md** sections on:
   - Core Structures
   - Tick Flow
   - Key Concepts
   (30 min)
3. Examine engine.rs code with **LOX_SIM_ENGINE_ANALYSIS.md** as guide
   - Focus on tick() method
   - Understand dirty flag propagation
   (1-2 hours)
4. Review **VALUE_INJECTION_DESIGN.md** to understand the proposed solution (1 hour)

**Total learning time**: ~2-3 hours for solid understanding

## FAQ

**Q: Why not just prevent block evaluation?**
A: Because blocks maintain internal state (e.g., Counter) that advances even if output is overridden. Skipping eval would freeze state.

**Q: Why two override modes (snapshot vs. held)?**
A: Snapshot is auto-clearing (simpler for one-offs). Held requires explicit release (safer for critical states like emergency shutdown).

**Q: How do overrides interact with dirty flags?**
A: Overridden outputs bypass block eval, so the block still gets evaluated (to advance internal state) but its output is ignored.

**Q: Is this backward compatible?**
A: Yes. Existing set_input(), inject_output(), set_param() work unchanged. New methods are opt-in extensions.

**Q: What about performance?**
A: Negligible. HashMap lookup is O(1), and we only check overrides when writing outputs (once per block eval). Typical overhead: < 1% of tick time.

## Questions or Clarifications?

Refer to the specific document:
- **Architecture questions** → LOX_SIM_ENGINE_ANALYSIS.md
- **Quick lookup** → QUICK_REFERENCE.md
- **Implementation details** → VALUE_INJECTION_DESIGN.md
- **High-level overview** → EXPLORATION_SUMMARY.txt

---

**Analysis completed**: April 22, 2024
**Total documentation**: 81 KB across 4 files
**Code examples**: 50+ snippets with line numbers
**File locations**: All referenced with exact line ranges
