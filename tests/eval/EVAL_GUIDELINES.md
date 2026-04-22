# Eval Guidelines

Rules and conventions for writing, reviewing, and maintaining Loxone Config-as-Code eval cases.

## Eval Case Anatomy

Every eval case **must** have these fields:

```json
{
  "id": "s01-piano-protection",
  "utterance": "Lower blinds in the living room when it's sunny and above 20 degrees to protect the piano",
  "difficulty": "easy|medium|hard|expert",
  "expected": {
    "new_blocks": [...],
    "wiring": [...],
    "params": [...],
    "simulation": [...]
  }
}
```

Optional fields: `pattern` (string[]), `source` (string).

---

## Rules

### 1. Utterances

- **Always homeowner language.** No Loxone jargon. Write as if a non-technical person is describing what they want.
  - ✅ "When it's raining, close all the blinds"
  - ❌ "Wire Regen.AQ to GreaterEqual threshold 0.5 then to JalousieUpDown2.InputTriggerDown"
- **German or English.** Both are valid. German utterances should read naturally, not be machine-translated.
- **One behavior per case** for easy/medium. Expert cases may combine behaviors.
- **Include numbers when relevant.** "above 20 degrees", "after 5 minutes", "between 8am and 6pm".

### 2. IDs

- Format: `{prefix}-{kebab-case-description}`
- Prefixes by category: `s` synthetic, `k` reference, `a`-`d` generated, `kp` patterns, `kr` rooms, `kx` extended, `ec`/`rt`/`sc`/`ms`/`wp`/`ap` advanced, `hv`/`hve` hvac, `tg` tier1-gaps, `uc` use-cases
- Must be globally unique across ALL case files.

### 3. Expected Blocks (`new_blocks`)

Each block **must** have:
- `type` — exact Loxone block type from `docs/schemas/connector-map.json` (195 known types)
- `room` — which room (must exist in fixture or be created)
- `page` — which page

Optional:
- `title_contains` — substring the title should contain (for thresholds, name hints)

```json
{"type": "GreaterEqual", "title_contains": "20", "room": "Wohnzimmer", "page": "Wohnzimmer"}
```

### 4. Wiring

Every wiring spec **must** pin the connector name on new blocks. This is how we measure spec coverage.

Required on wires touching new blocks:
- `from_type` + `from_connector` — when the source is a new block
- `to_type` + `to_connector` — when the target is a new block

For wires from fixture items (sensors/actuators already in the config):
- `from_title` + `from_connector` — reference by title

```json
{
  "from_title": "Außentemperatur",
  "from_connector": "AQ",
  "to_type": "GreaterEqual",
  "to_connector": "Input1"
}
```

#### Common connector names

| Block Type | Inputs | Outputs |
|-----------|--------|---------|
| And, Or | I1, I2 | Q |
| Not | I | Q |
| GreaterEqual, Less | Input1, Input2 | Q |
| Mult, Add, Sub, Div | Input1, Input2 | AQ |
| Monoflop, OnPulseDelay, OffDelay | InputTrigger | Q |
| StairwayLS | InputTrigger, On | Q |
| DayTimer | InputTrigger | AQ, Qon, Qoff |
| FlipFlop | InputS, InputR | Q |
| Memory (AMemory) | S, R, TI | Q |
| PulseGen | InputEnable, InputInvert | Q |
| PushButton | InputTrigger, On | Q, Qon, Qoff |
| State | I1…I20 | AQ, TQ |
| Counter | Trigger | AQ, Q |

Full reference: `docs/schemas/connector-map.json`

### 5. Params

Specify params only when the case requires **non-default** values:

```json
"params": [
  {"block_type": "GreaterEqual", "title_contains": "20", "param": "Value", "value": 20.0},
  {"block_type": "OnPulseDelay", "param": "PulseTime", "value": 300}
]
```

### 6. Simulation (Input→Output)

**Every case must have simulation specs.** This is the behavioral test — does the circuit actually produce the right output?

Each simulation spec defines:
- `name` — human-readable test description
- `inputs` — sensor values (`"Title.Connector": value`)
- `ticks` — number of simulation ticks (10 for instant logic, 6000+ for timers)
- `dt` — time step in seconds (0.1 default)
- `expected_outputs` — output assertions (`"Title.Connector": {"comparator": value}`)

**Always include both positive and negative tests:**

```json
"simulation": [
  {
    "name": "should close blinds when hot and sunny",
    "inputs": {"Außentemperatur.AQ": 25.0, "Sonnenschein.AQ": 1.0},
    "ticks": 10, "dt": 0.1,
    "expected_outputs": {"Jalousie 1.InputTriggerDown": {">": 0.5}}
  },
  {
    "name": "should NOT close blinds when cold",
    "inputs": {"Außentemperatur.AQ": 15.0, "Sonnenschein.AQ": 1.0},
    "ticks": 10, "dt": 0.1,
    "expected_outputs": {"Jalousie 1.InputTriggerDown": {"<": 0.5}}
  }
]
```

#### Comparators

| Comparator | Meaning |
|-----------|---------|
| `">"` | Greater than |
| `"<"` | Less than |
| `">="` | Greater or equal |
| `"<="` | Less or equal |
| `"=="` | Exactly equal |
| `"~="` | Approximate (±5%) |

#### Guidelines for simulation values

- **Threshold cases:** Test at threshold ± 5. E.g., threshold=20 → test with 25 (trigger) and 15 (no trigger).
- **Boolean logic:** Test all relevant input combinations (at minimum: all-true, one-false).
- **Timers:** Use `ticks × dt` to exceed the timer duration. E.g., 5-minute delay → `ticks: 3100, dt: 0.1` (310 seconds).
- **Analog math:** Test with specific values and verify the arithmetic. E.g., Mult with factor 0.5 → input 100, expect output ~50.

### 7. Difficulty

| Level | Blocks | Wiring | Logic |
|-------|--------|--------|-------|
| easy | 1-2 | 1-3 direct | Single condition |
| medium | 2-4 | 3-6 | Combined conditions, one timer |
| hard | 4-7 | 6-12 | Multi-stage logic, schedules |
| expert | 7+ | 12+ | Cross-room, state machines, edge cases |

---

## Quality Checklist

Before submitting new eval cases, verify:

- [ ] ID is globally unique (`python3 schema/validate.py`)
- [ ] Utterance is natural homeowner language (no block type names)
- [ ] All wiring specs have connector names on new blocks
- [ ] Simulation specs include positive AND negative tests
- [ ] Block types exist in `connector-map.json`
- [ ] Connector names are valid for the block type
- [ ] Difficulty matches complexity
- [ ] `python3 scripts/coverage.py` shows improvement

## Coverage Requirements

We track three coverage dimensions. New cases should **improve** these numbers, not regress them.

### 1. Block Type Coverage

What % of Loxone's 195 block types appear in at least one eval case.

```bash
python3 tests/eval/scripts/coverage.py
```

**Targets:**
- Tier 1 (core automation): **100%** ✅ (achieved)
- Tier 2 (common controls): **50%+** (currently 32%)
- Overall: **50%+** (currently 44%)

When adding a case, prefer uncovered block types. Run `--uncovered` to see gaps.

### 2. Per-Case Spec Coverage (Connector Tightness)

For each case: what % of the new blocks' connectors are pinned by wiring specs? A case with 0% tightness passes even if the agent wires everything wrong.

**Targets:**
- No new case at 0% — every case must have explicit connector names
- Average across all cases: **60%+** (currently 55%)
- Every wiring spec touching a new block must have `to_connector` / `from_connector`

### 3. Simulation Coverage

What % of cases have input→output behavioral tests? Structural correctness alone is insufficient — the circuit must actually produce the right output.

**Targets:**
- Every new case **must** include simulation specs (positive + negative)
- Overall: **80%+** of cases with simulation specs
- Each simulation should test at least 2 scenarios (trigger + no-trigger)

### Checking Coverage

```bash
# Full report (all 3 dimensions)
python3 tests/eval/scripts/coverage.py

# Per-case breakdown
python3 tests/eval/scripts/coverage.py --cases

# Uncovered block types
python3 tests/eval/scripts/coverage.py --uncovered

# JSON export for CI
python3 tests/eval/scripts/coverage.py --json
```

---

## Running Validation

```bash
# Schema + duplicate ID + connector validation
python3 tests/eval/schema/validate.py

# Coverage report (block types + per-case spec tightness)
python3 tests/eval/scripts/coverage.py

# Full eval run
bash tests/eval/scripts/run-eval.sh
```

## Adding a New Category

1. Create `tests/eval/cases/{category}.json` (array of cases)
2. Add entry to `tests/eval/cases-index.json`
3. Run `python3 tests/eval/schema/validate.py` to verify
4. Run `python3 tests/eval/scripts/coverage.py` to check coverage impact

## Fixture

The test fixture (`fixture.Loxone`) contains:
- **7 rooms:** Wohnzimmer, Schlafzimmer, Küche, Kinderzimmer, Büro, Bad, Garten
- **Sensors:** Außentemperatur, Sonnenschein, Regen, Wind, Luftfeuchtigkeit, Bewegungsmelder, Türkontakt, Türklingel, Pool Temperatur
- **Controls:** LightController2 per room, Jalousie per room, Poolpumpe, Klimaanlage, Lüfter Bad, Raumregler

Do not modify the fixture unless adding new sensors/rooms needed by multiple cases.
