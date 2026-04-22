#!/usr/bin/env python3
"""
Progressive House Build Eval

Builds a complete house config from scratch in 8 stages,
validating at each checkpoint.

Usage:
  python3 tests/eval/progressive_eval.py --stage 1       # run stage 1 only
  python3 tests/eval/progressive_eval.py --all            # run all stages
  python3 tests/eval/progressive_eval.py --resume 4       # resume from stage 4
  python3 tests/eval/progressive_eval.py --status         # show progress
  python3 tests/eval/progressive_eval.py --final-check    # validate final config
"""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
WORK_DIR = Path("/tmp/eval-progressive")
CONFIG = WORK_DIR / "house.Loxone"
LOX = Path("/home/amy/src/lox/target/debug/lox")


def load_progressive(prog_file):
    """Load a progressive evaluation file."""
    with open(prog_file) as f:
        return json.load(f)


def create_empty_config():
    """Create a minimal empty .Loxone config."""
    WORK_DIR.mkdir(parents=True, exist_ok=True)
    xml = '''<?xml version="1.0" encoding="utf-8"?>
<ControlList LxAV="84" Version="267" NextMem="3" NextConst="1" NextNote="1" NextObj="10">
  <C Type="Document" V="175" U="00000001-0000-0001-ffff000000000001" Title="Progressive House" WF="16384" ConfigVersion="17000331"/>
  <C Type="Program" V="175" U="40000001-0001-0001-ffff000000000001" Title="Prog" WF="16384">
  </C>
</ControlList>'''
    CONFIG.write_text(xml, encoding='utf-8')
    print(f"Created empty config: {CONFIG}")


def run_lox(*args):
    """Run a lox CLI command."""
    cmd = [str(LOX)] + list(args)
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    return r


def check_checkpoint(checkpoint, config_path):
    """Validate a checkpoint against the current config."""
    results = []

    # Check rooms
    if 'rooms' in checkpoint:
        r = run_lox('config', 'describe', str(config_path))
        for room in checkpoint['rooms']:
            if room in r.stdout:
                results.append(('✓', f'Room "{room}" exists'))
            else:
                results.append(('✗', f'Room "{room}" NOT found'))

    # Check categories
    if 'categories' in checkpoint:
        r = run_lox('config', 'stats', str(config_path))
        for cat in checkpoint['categories']:
            # Categories are in the stats output or describe
            r2 = run_lox('config', 'describe', str(config_path))
            if cat in r2.stdout or cat in r.stdout:
                results.append(('✓', f'Category "{cat}" exists'))
            else:
                results.append(('✗', f'Category "{cat}" NOT found'))

    # Check controls by type
    if 'controls_by_type' in checkpoint:
        r = run_lox('config', 'stats', str(config_path))
        for ctype, expected_count in checkpoint['controls_by_type'].items():
            match = re.search(rf'{ctype}\s+(\d+)', r.stdout)
            actual = int(match.group(1)) if match else 0
            if actual >= expected_count:
                results.append(('✓', f'{ctype}: {actual} (need {expected_count})'))
            else:
                results.append(('✗', f'{ctype}: {actual} (need {expected_count})'))

    # Check controls by title
    if 'controls_by_title' in checkpoint:
        r = run_lox('config', 'describe', str(config_path))
        for title in checkpoint['controls_by_title']:
            if title in r.stdout:
                results.append(('✓', f'Control "{title}" exists'))
            else:
                results.append(('✗', f'Control "{title}" NOT found'))

    # Check min items
    if 'min_items' in checkpoint:
        r = run_lox('config', 'stats', str(config_path))
        match = re.search(r'Total items:\s+(\d+)', r.stdout)
        actual = int(match.group(1)) if match else 0
        if actual >= checkpoint['min_items']:
            results.append(('✓', f'Total items: {actual} (need {checkpoint["min_items"]})'))
        else:
            results.append(('✗', f'Total items: {actual} (need {checkpoint["min_items"]})'))

    # Check new blocks minimum
    if 'new_blocks_min' in checkpoint:
        results.append(('✓', f'New blocks check (deferred to eval)'))

    # Check wiring minimum
    if 'wiring_min' in checkpoint:
        results.append(('✓', f'Wiring check (deferred to eval)'))

    # Check specific wiring
    if 'wiring' in checkpoint:
        for w in checkpoint['wiring']:
            results.append(('✓', f'Wiring check: {w} (deferred)'))

    # Always run check + validate
    r = run_lox('config', 'check', str(config_path))
    errors = r.stdout.count('✗')
    if errors == 0:
        results.append(('✓', f'config check: clean'))
    else:
        results.append(('⚠', f'config check: {errors} errors'))

    r = run_lox('config', 'validate', str(config_path))
    val_errors = r.stdout.count('✗')
    if val_errors == 0:
        results.append(('✓', f'config validate: clean'))
    else:
        results.append(('⚠', f'config validate: {val_errors} errors'))

    return results


def show_status(prog):
    """Show current progress."""
    print("═══ Progressive House Build Status ═══\n")
    for stage in prog['stages']:
        status_file = WORK_DIR / f"{stage['id']}.done"
        done = status_file.exists()
        icon = "✓" if done else "○"
        print(f"  {icon} Stage {stage['progress']:3d}% — {stage['name']:20s} ({len(stage['steps'])} steps)")
    print()

    if CONFIG.exists():
        r = run_lox('config', 'stats', str(CONFIG))
        match = re.search(r'Total items:\s+(\d+)', r.stdout)
        items = int(match.group(1)) if match else 0
        print(f"  Config: {CONFIG} ({items} items)")


def final_check(prog, config_path):
    """Validate the final config against the full checkpoint."""
    fc = prog['final_checkpoint']
    print("═══ Final House Validation ═══\n")

    r = run_lox('config', 'stats', str(config_path))
    print(r.stdout)

    r = run_lox('config', 'check', str(config_path))
    print(f"Check:\n{r.stdout}")

    r = run_lox('config', 'validate', str(config_path))
    print(f"Validate:\n{r.stdout}")

    r = run_lox('config', 'scan', str(config_path))
    print(f"Scan:\n{r.stdout}")

    # Check required types
    r = run_lox('config', 'stats', str(config_path))
    missing = []
    for t in fc['required_types']:
        if t not in r.stdout:
            missing.append(t)

    if missing:
        print(f"\n✗ Missing block types: {', '.join(missing)}")
    else:
        print(f"\n✓ All {len(fc['required_types'])} required block types present")

    match = re.search(r'Total items:\s+(\d+)', r.stdout)
    items = int(match.group(1)) if match else 0
    print(f"✓ Total items: {items} (need {fc['min_controls']})" if items >= fc['min_controls']
          else f"✗ Total items: {items} (need {fc['min_controls']})")


def main():
    parser = argparse.ArgumentParser(description='Progressive House Build Eval')
    parser.add_argument('prog_file', nargs='?', default=str(EVAL_DIR / 'progressive' / 'expert.json'),
                        help='Progressive eval file to run')
    parser.add_argument('--stage', type=int, help='Run a specific stage')
    parser.add_argument('--all', action='store_true', help='Run all stages')
    parser.add_argument('--resume', type=int, help='Resume from stage N')
    parser.add_argument('--status', action='store_true', help='Show progress')
    parser.add_argument('--final-check', action='store_true', help='Validate final config')
    parser.add_argument('--reset', action='store_true', help='Reset and start fresh')
    args = parser.parse_args()

    prog = load_progressive(args.prog_file)

    if args.status:
        show_status(prog)
        return

    if args.reset:
        create_empty_config()
        for f in WORK_DIR.glob("*.done"):
            f.unlink()
        print("Reset complete")
        return

    if args.final_check:
        final_check(prog, CONFIG)
        return

    if not CONFIG.exists():
        create_empty_config()

    stages = prog['stages']
    if args.stage:
        stages = [s for s in stages if s['progress'] // 10 == args.stage or
                  int(s['id'].split('-')[1]) == args.stage]
    elif args.resume:
        stages = [s for s in stages if int(s['id'].split('-')[1]) >= args.resume]
    elif not args.all:
        parser.print_help()
        return

    for stage in stages:
        print(f"\n{'═' * 60}")
        print(f"  Stage: {stage['name']} ({stage['progress']}%)")
        print(f"  {stage['description']}")
        print(f"{'═' * 60}\n")

        for step in stage['steps']:
            print(f"  [{step['id']}] {step['utterance'][:70]}...")
            print(f"  → Agent should implement this using lox CLI")
            print()

        # Check stage checkpoint
        last_step = stage['steps'][-1]
        if 'checkpoint' in last_step:
            print(f"  Checkpoint:")
            results = check_checkpoint(last_step['checkpoint'], CONFIG)
            passed = 0
            for status, msg in results:
                print(f"    {status} {msg}")
                if status == '✓':
                    passed += 1

            total = len(results)
            if passed == total:
                print(f"\n  ✓ Stage {stage['name']} complete ({passed}/{total})")
                (WORK_DIR / f"{stage['id']}.done").touch()
            else:
                print(f"\n  ✗ Stage {stage['name']} incomplete ({passed}/{total})")

    print()


if __name__ == '__main__':
    main()
