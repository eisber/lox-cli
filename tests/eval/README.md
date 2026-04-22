# Loxone Config Eval Suite

Professional evaluation suite for Loxone configuration editing capabilities. Tests AI agents and automation tools on realistic configuration tasks across 285+ test cases.

## Directory Structure

```
tests/eval/
├── README.md                          # This file
├── schema/
│   ├── eval-case.schema.json          # JSON Schema for eval cases
│   ├── progressive.schema.json        # JSON Schema for progressive evals
│   └── validate.py                    # Validation script
├── cases/
│   ├── synthetic.json                 # 20 synthetic cases
│   ├── reference.json                 # 24 reference patterns
│   ├── generated.json                 # 50 LLM-generated cases
│   ├── patterns.json                  # 25 common patterns
│   ├── rooms.json                     # 15 room-based scenarios
│   ├── extended.json                  # 15 extended scenarios
│   ├── advanced.json                  # 29 advanced patterns
│   ├── hvac.json                      # 18 HVAC scenarios
│   ├── hvac-extracted.json            # 32 extracted HVAC cases
│   ├── tier1-gaps.json                # 17 coverage gap cases
│   └── use-cases.json                 # 40 real-world use cases
├── progressive/
│   ├── challenges.json                # 11 multi-step challenges
│   ├── homeowner.json                 # Homeowner progressive scenario
│   └── expert.json                    # Expert progressive scenario
├── golden/
│   ├── use-cases.json                 # Use case metadata from loxone.com
│   ├── challenges.json                # Config Challenge reference tasks
│   └── configs/                       # Reference .Loxone files (gitignored)
├── reports/                           # Generated reports (gitignored)
├── scripts/
│   ├── run-eval.sh                    # Main eval runner
│   ├── coverage.py                    # Coverage analysis
│   ├── agent-runner.py                # LLM agent runner
│   ├── build-prompt.py                # Prompt builder
│   ├── semantic-eval.py               # Semantic evaluator
│   ├── progressive-eval.py            # Progressive evaluator
│   └── sim-check.py                   # Simulator integration
├── fixture.Loxone                     # Test fixture config
└── cases-index.json                   # Index of all case files
```

## Quick Start

### List available cases

```bash
cd tests/eval
./scripts/run-eval.sh --list
./scripts/run-eval.sh --list --filter hvac
```

### Show statistics

```bash
./scripts/run-eval.sh --stats
```

### Validate fixture

```bash
./scripts/run-eval.sh --validate-fixture
```

### Run single case

```bash
./scripts/run-eval.sh <case-id> <modified-config.Loxone>
```

Example:
```bash
./scripts/run-eval.sh s01-piano-protection result.Loxone
```

### Run batch evaluation

```bash
./scripts/run-eval.sh --batch results/
```

Results directory should contain `.Loxone` files named after case IDs.

## Evaluation Metrics

### Block Metrics

- **block_precision**: `correct_blocks / total_blocks_added` — Penalizes extra blocks
- **block_recall**: `correct_blocks / expected_blocks` — Penalizes missing blocks
- **block_f1**: Harmonic mean of precision and recall

### Wiring Metrics

- **wiring_accuracy**: `correct_wires / expected_wires` — Expected wiring coverage
- **wiring_precision**: `correct_new_wires / total_new_wires` — Penalizes extra wires
- **trace_score**: BFS reachability through wiring graph: `reachable_paths / expected_wiring_count`

### Parameter Metrics

- **param_accuracy**: `correct_params / expected_params` — Parameter setting accuracy

### Quality Metrics

- **check_score**: `lox config check` pass rate: `ok_checks / total_checks`
- **ux_score**: `blocks_with_page_and_position / total_new_blocks` — UI organization
- **xml_valid**: Result parses as valid XML
- **validation_pass**: `lox config validate` returns 0 errors

### Simulation Metrics (optional)

- **simulation_score**: `lox-sim` signal evaluation: `assertion_passes / total_assertions`
  - Only evaluated when `expected.simulation` spec exists

### Overall Score

```
overall_score = (0.20 × block_f1) 
              + (0.30 × wiring_accuracy) 
              + (0.20 × param_accuracy) 
              + (0.15 × check_score) 
              + (0.15 × trace_score)
```

**Pass threshold**: 0.80 (80%)

### Difficulty Weights

Cases are weighted by difficulty for aggregate scoring:

- **easy**: 1.0×
- **medium**: 1.5×
- **hard**: 2.0×
- **expert**: 3.0×

## Validation

### Validate all case files

```bash
python3 schema/validate.py
python3 schema/validate.py --verbose
```

Checks:
- JSON schema compliance
- Unique IDs across all files
- Valid block types (against `connector-map.json`)
- Valid connector names for each block type

### Coverage Analysis

```bash
python3 scripts/coverage.py              # Summary
python3 scripts/coverage.py --detail     # Per-type breakdown
python3 scripts/coverage.py --cases      # Per-case spec coverage
python3 scripts/coverage.py --uncovered  # List uncovered types
python3 scripts/coverage.py --json       # Machine-readable
```

**Aggregate coverage**: Across ALL eval cases, what % of Loxone's 195 block types and 2806 connectors are exercised?

**Per-case spec coverage**: For each eval case, what % of the new blocks' connectors are pinned by wiring specs? Low coverage = eval is too loose.

## Adding New Cases

### 1. Choose the right file

- **synthetic.json**: Hand-crafted patterns for specific testing
- **reference.json**: Basic reference patterns from docs
- **patterns.json**: Common automation patterns
- **rooms.json**: Room-based scenarios
- **hvac.json**: HVAC/climate control
- **advanced.json**: Complex multi-block patterns
- **use-cases.json**: Real-world use cases

### 2. Create the case

```json
{
  "id": "my-new-case",
  "utterance": "User request in natural language (at least 10 chars)",
  "difficulty": "medium",
  "pattern": "category-name",
  "source": "where this came from",
  "expected": {
    "new_blocks": [
      {
        "type": "LightController2",
        "title_contains": "Light",
        "room": "Living Room",
        "page": "Automation"
      }
    ],
    "wiring": [
      {
        "from_type": "MotionDetector",
        "from_title": "Motion",
        "from_connector": "AQ",
        "to_type": "LightController2",
        "to_title": "Light",
        "to_connector": "AI"
      }
    ],
    "params": [
      {
        "block_type": "LightController2",
        "block_title": "Light",
        "param_name": "FadingTime",
        "param_value": 2.0
      }
    ],
    "simulation": {
      "inputs": {
        "Motion.AQ": 1
      },
      "expected_outputs": {
        "Light.AQ": 1
      }
    }
  }
}
```

### 3. Validate

```bash
python3 schema/validate.py
```

### 4. Update cases-index.json

If adding to a new file, update the counts and difficulty breakdown in `cases-index.json`.

## Progressive Evaluation

Multi-step scenarios that build on previous steps. Located in `progressive/`:

- **challenges.json**: 11 Config Challenge tasks from Loxone
- **homeowner.json**: Homeowner persona building a smart home
- **expert.json**: Expert integrator creating complex automation

Run progressive evals:

```bash
python3 scripts/progressive-eval.py progressive/challenges.json
```

## Golden Reference Data

Located in `golden/`:

- **use-cases.json**: Metadata scraped from loxone.com/enen/kb/ articles
- **challenges.json**: Official Config Challenge task descriptions
- **configs/**: Reference `.Loxone` files (not committed)

## Reports

All generated reports go to `reports/` (gitignored). Formats:

- `coverage-report.json`: Block/connector coverage analysis
- `eval-report.json`: Single case evaluation results
- `llm-report.json`: Batch LLM agent evaluation
- `baseline-report.json`: Baseline performance metrics
- `semantic-report.json`: Semantic similarity evaluation

## Tips

### Running with the lox CLI

The eval harness auto-detects `lox` on PATH. If not found, falls back to `cargo run --quiet --`.

### Debugging failures

```bash
# Run with verbose output
./scripts/run-eval.sh s01-piano-protection result.Loxone 2>&1

# Validate the result manually
lox config validate result.Loxone
lox config check result.Loxone
lox config describe result.Loxone
```

### Creating test fixtures

```bash
# Start from the base fixture
cp fixture.Loxone my-test.Loxone

# Make changes
lox config add --type LightController2 --title "My Light" my-test.Loxone
lox config wire-connector my-test.Loxone "My Light.I1" <sensor-uuid>

# Validate
lox config validate my-test.Loxone
```

## Schema Documentation

See `schema/eval-case.schema.json` for the complete JSON Schema definition of eval cases.

See `schema/progressive.schema.json` for the progressive challenge schema.

## CI Integration

The validation script is designed for CI:

```bash
# Exit code 0 on success, 1 on validation errors
python3 schema/validate.py

# Check coverage thresholds
python3 scripts/coverage.py --json | jq '.aggregate.block_types.coverage > 0.5'
```

## Related Files

- `docs/schemas/connector-map.json`: 195 block types with 2806 connectors
- `docs/schemas/loxone-block-types-full.json`: 221 types from TechDoc
- `docs/schemas/loxone-config.xsd`: XML schema (309 types, 4892 lines)

## Contributing

1. Add cases to the appropriate file in `cases/`
2. Run `python3 schema/validate.py` to check validity
3. Run `python3 scripts/coverage.py` to verify coverage impact
4. Update this README if adding new case categories

---

**Eval suite version**: 2.0  
**Total cases**: 285  
**Block type coverage**: ~60% of 195 types  
**Connector coverage**: ~40% of 2806 connectors  
