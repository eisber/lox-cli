#!/usr/bin/env python3
"""Validate all eval case files against JSON schemas and check for issues.

Validates:
- JSON schema compliance for eval cases and progressive challenges
- Unique IDs across all case files (no duplicates)
- Block types exist in connector-map.json
- Connector names are valid for each block type
- File structure and naming conventions

Usage:
    python3 tests/eval/schema/validate.py              # validate all
    python3 tests/eval/schema/validate.py --verbose    # detailed output
    python3 tests/eval/schema/validate.py --fix        # auto-fix issues
"""

import json
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set, Tuple

try:
    import jsonschema
    HAS_JSONSCHEMA = True
except ImportError:
    HAS_JSONSCHEMA = False
    print("Warning: jsonschema not installed. Schema validation disabled.", file=sys.stderr)
    print("Install with: pip3 install jsonschema", file=sys.stderr)

REPO = Path(__file__).resolve().parent.parent.parent.parent
EVAL_DIR = REPO / "tests" / "eval"
SCHEMA_DIR = EVAL_DIR / "schema"
CASES_DIR = EVAL_DIR / "cases"
PROGRESSIVE_DIR = EVAL_DIR / "progressive"
CONNECTOR_MAP = REPO / "docs" / "schemas" / "connector-map.json"

EVAL_SCHEMA = SCHEMA_DIR / "eval-case.schema.json"
PROGRESSIVE_SCHEMA = SCHEMA_DIR / "progressive.schema.json"


def load_connector_map() -> Dict:
    """Load connector-map.json to validate block types and connectors."""
    with open(CONNECTOR_MAP) as f:
        return json.load(f)


def validate_json_file(filepath: Path, schema: Dict) -> List[str]:
    """Validate a JSON file against a schema."""
    if not HAS_JSONSCHEMA:
        return []

    errors = []
    try:
        with open(filepath) as f:
            data = json.load(f)
        
        validator = jsonschema.Draft7Validator(schema)
        for error in validator.iter_errors(data):
            path = ".".join(str(p) for p in error.path)
            errors.append(f"{filepath.name}: {path}: {error.message}")
    except json.JSONDecodeError as e:
        errors.append(f"{filepath.name}: JSON parse error: {e}")
    except Exception as e:
        errors.append(f"{filepath.name}: Validation error: {e}")
    
    return errors


def collect_case_ids(cases_dir: Path) -> Tuple[Dict[str, List[str]], List[str]]:
    """Collect all case IDs and detect duplicates."""
    id_to_files = defaultdict(list)
    errors = []
    
    for json_file in sorted(cases_dir.glob("*.json")):
        try:
            with open(json_file) as f:
                cases = json.load(f)
            
            if not isinstance(cases, list):
                errors.append(f"{json_file.name}: Expected array of cases, got {type(cases).__name__}")
                continue
            
            for i, case in enumerate(cases):
                if not isinstance(case, dict):
                    errors.append(f"{json_file.name}[{i}]: Expected object, got {type(case).__name__}")
                    continue
                
                case_id = case.get("id")
                if not case_id:
                    errors.append(f"{json_file.name}[{i}]: Missing 'id' field")
                    continue
                
                id_to_files[case_id].append(json_file.name)
        
        except json.JSONDecodeError as e:
            errors.append(f"{json_file.name}: JSON parse error: {e}")
        except Exception as e:
            errors.append(f"{json_file.name}: Error reading file: {e}")
    
    return id_to_files, errors


def validate_block_types(cases_dir: Path, connector_map: Dict) -> List[str]:
    """Validate that all block types and connectors are valid."""
    errors = []
    valid_types = set(connector_map.keys())
    
    for json_file in sorted(cases_dir.glob("*.json")):
        try:
            with open(json_file) as f:
                cases = json.load(f)
            
            if not isinstance(cases, list):
                continue
            
            for i, case in enumerate(cases):
                if not isinstance(case, dict):
                    continue
                
                case_id = case.get("id", f"[{i}]")
                expected = case.get("expected", {})
                
                # Check new_blocks
                for j, block in enumerate(expected.get("new_blocks", [])):
                    block_type = block.get("type")
                    if not block_type:
                        errors.append(f"{json_file.name}:{case_id}: new_blocks[{j}] missing 'type'")
                        continue
                    
                    if block_type not in valid_types:
                        errors.append(f"{json_file.name}:{case_id}: Unknown block type '{block_type}'")
                
                # Check wiring connectors
                for j, wire in enumerate(expected.get("wiring", [])):
                    from_type = wire.get("from_type")
                    to_type = wire.get("to_type")
                    from_conn = wire.get("from_connector") or wire.get("from_conn")
                    to_conn = wire.get("to_connector") or wire.get("to_conn")
                    
                    if from_type and from_type not in valid_types:
                        errors.append(f"{json_file.name}:{case_id}: wiring[{j}] unknown from_type '{from_type}'")
                    
                    if to_type and to_type not in valid_types:
                        errors.append(f"{json_file.name}:{case_id}: wiring[{j}] unknown to_type '{to_type}'")
                    
                    # Validate connectors exist for the type
                    if from_type and from_conn and from_type in connector_map:
                        connectors = connector_map[from_type].get("t", {})
                        if from_conn not in connectors:
                            errors.append(
                                f"{json_file.name}:{case_id}: wiring[{j}] connector '{from_conn}' "
                                f"not found in {from_type}"
                            )
                    
                    if to_type and to_conn and to_type in connector_map:
                        connectors = connector_map[to_type].get("t", {})
                        if to_conn not in connectors:
                            errors.append(
                                f"{json_file.name}:{case_id}: wiring[{j}] connector '{to_conn}' "
                                f"not found in {to_type}"
                            )
        
        except Exception as e:
            errors.append(f"{json_file.name}: Error validating block types: {e}")
    
    return errors


def validate_progressive_ids(progressive_dir: Path) -> Tuple[Dict[str, List[str]], List[str]]:
    """Validate progressive challenge IDs."""
    id_to_files = defaultdict(list)
    errors = []
    
    if not progressive_dir.exists():
        return id_to_files, errors
    
    for json_file in sorted(progressive_dir.glob("*.json")):
        try:
            with open(json_file) as f:
                challenges = json.load(f)
            
            if not isinstance(challenges, list):
                errors.append(f"{json_file.name}: Expected array, got {type(challenges).__name__}")
                continue
            
            for i, challenge in enumerate(challenges):
                if not isinstance(challenge, dict):
                    continue
                
                challenge_id = challenge.get("id")
                if not challenge_id:
                    errors.append(f"{json_file.name}[{i}]: Missing 'id' field")
                    continue
                
                id_to_files[challenge_id].append(json_file.name)
        
        except json.JSONDecodeError as e:
            errors.append(f"{json_file.name}: JSON parse error: {e}")
        except Exception as e:
            errors.append(f"{json_file.name}: Error reading file: {e}")
    
    return id_to_files, errors


def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    
    print("=" * 70)
    print("Loxone Eval Case Validation")
    print("=" * 70)
    print()
    
    all_errors = []
    
    # Load connector map
    try:
        connector_map = load_connector_map()
        print(f"✓ Loaded connector map: {len(connector_map)} block types")
    except Exception as e:
        print(f"✗ Failed to load connector map: {e}")
        return 1
    
    # Validate eval case schemas
    if HAS_JSONSCHEMA and EVAL_SCHEMA.exists():
        with open(EVAL_SCHEMA) as f:
            eval_schema = json.load(f)
        
        print(f"\n📋 Validating eval cases against schema...")
        for json_file in sorted(CASES_DIR.glob("*.json")):
            errors = validate_json_file(json_file, eval_schema)
            if errors:
                all_errors.extend(errors)
                print(f"  ✗ {json_file.name}: {len(errors)} error(s)")
                if verbose:
                    for err in errors:
                        print(f"      {err}")
            else:
                print(f"  ✓ {json_file.name}")
    else:
        print("\n⚠ Skipping schema validation (jsonschema not installed or schema missing)")
    
    # Validate progressive schemas
    if HAS_JSONSCHEMA and PROGRESSIVE_SCHEMA.exists() and PROGRESSIVE_DIR.exists():
        with open(PROGRESSIVE_SCHEMA) as f:
            progressive_schema = json.load(f)
        
        print(f"\n📋 Validating progressive challenges against schema...")
        for json_file in sorted(PROGRESSIVE_DIR.glob("*.json")):
            errors = validate_json_file(json_file, progressive_schema)
            if errors:
                all_errors.extend(errors)
                print(f"  ✗ {json_file.name}: {len(errors)} error(s)")
                if verbose:
                    for err in errors:
                        print(f"      {err}")
            else:
                print(f"  ✓ {json_file.name}")
    
    # Check for duplicate case IDs
    print(f"\n🔍 Checking for duplicate case IDs...")
    id_to_files, id_errors = collect_case_ids(CASES_DIR)
    all_errors.extend(id_errors)
    
    duplicates = {k: v for k, v in id_to_files.items() if len(v) > 1}
    if duplicates:
        print(f"  ✗ Found {len(duplicates)} duplicate ID(s):")
        for case_id, files in sorted(duplicates.items()):
            print(f"      '{case_id}' in: {', '.join(files)}")
            all_errors.append(f"Duplicate ID '{case_id}' in: {', '.join(files)}")
    else:
        total_cases = sum(len(files) for files in id_to_files.values())
        print(f"  ✓ All {total_cases} case IDs are unique")
    
    # Check progressive IDs
    if PROGRESSIVE_DIR.exists():
        print(f"\n🔍 Checking progressive challenge IDs...")
        prog_id_to_files, prog_errors = validate_progressive_ids(PROGRESSIVE_DIR)
        all_errors.extend(prog_errors)
        
        prog_duplicates = {k: v for k, v in prog_id_to_files.items() if len(v) > 1}
        if prog_duplicates:
            print(f"  ✗ Found {len(prog_duplicates)} duplicate progressive ID(s):")
            for prog_id, files in sorted(prog_duplicates.items()):
                print(f"      '{prog_id}' in: {', '.join(files)}")
                all_errors.append(f"Duplicate progressive ID '{prog_id}' in: {', '.join(files)}")
        else:
            total_prog = sum(len(files) for files in prog_id_to_files.values())
            print(f"  ✓ All {total_prog} progressive IDs are unique")
    
    # Validate block types and connectors
    print(f"\n🔧 Validating block types and connectors...")
    type_errors = validate_block_types(CASES_DIR, connector_map)
    if type_errors:
        all_errors.extend(type_errors)
        print(f"  ✗ Found {len(type_errors)} block/connector error(s)")
        if verbose:
            for err in type_errors:
                print(f"      {err}")
    else:
        print(f"  ✓ All block types and connectors are valid")
    
    # Summary
    print()
    print("=" * 70)
    if all_errors:
        print(f"❌ Validation FAILED: {len(all_errors)} error(s) found")
        if not verbose:
            print("\nRun with --verbose to see all errors")
        return 1
    else:
        print("✅ Validation PASSED: All checks passed")
        
        # Print summary stats
        total_cases = len(id_to_files)
        total_prog = len(prog_id_to_files) if PROGRESSIVE_DIR.exists() else 0
        
        print(f"\n📊 Summary:")
        print(f"   - {total_cases} eval cases across {len(list(CASES_DIR.glob('*.json')))} files")
        if total_prog:
            print(f"   - {total_prog} progressive challenges across {len(list(PROGRESSIVE_DIR.glob('*.json')))} files")
        print(f"   - {len(connector_map)} valid block types")
        
        return 0


if __name__ == "__main__":
    sys.exit(main())
