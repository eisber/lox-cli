#!/usr/bin/env python3
"""Validate element schemas by creating blocks in config XML.

For each schema type:
1. Add a minimal <C Type="..."> to the config XML
2. Open in Loxone Config (which generates <Co> connectors)
3. Re-parse XML to extract connector keys
4. Compare with KB schema abbreviations
5. Update crosswalk.json

Usage:
    # Step 1: Generate config with validation blocks
    python3 scripts/validate-schemas.py generate --tier 1

    # Step 2: After opening/saving in Loxone Config, extract connectors
    python3 scripts/validate-schemas.py extract

    # Step 3: Build crosswalk from extracted data
    python3 scripts/validate-schemas.py crosswalk
"""

import argparse
import json
import os
import re
import sys
import uuid
import xml.etree.ElementTree as ET
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
REPO_DIR = SCRIPT_DIR.parent
SCHEMA_DIR = REPO_DIR / 'docs' / 'schemas'
CONFIG_PATH = Path('/mnt/c/Users/AmyCozowicz/AppData/Local/Temp/Lauf1FinalExport.Loxone')
BACKUP_PATH = CONFIG_PATH.with_suffix('.bak')

# Tier assignments
TIER_SLUGS = {
    1: [  # Logic Simple - 32 types
        'and', 'or', 'not', 'exclusive-or', 'add-2-way', 'add-4-way',
        'subtract', 'multiply', 'divide', 'modulo', 'formula', 'average',
        'integer', 'binary-decoder', 'binary-decoder-2', 'binary-encoder',
        'analogue-multiplexer-2-way', 'analogue-multiplexer-4-way',
        'analogue-watchdog', 'status', 'pulse-by', 'pulse-at',
        'pulse-generator', 'pulse-width-modulation', 'delayed-pulse',
        'analogue-memory', 'dewpoint-calculator', 'random-number-generator',
        'command-recognition', 'memory-flags', 'ir-control',
        'touch-and-grill-air',
    ],
    2: [  # Logic Medium - 34 types
        'analogue-min-max-limiter', 'custom-script-programming',
        'edge-triggered-wiping-relay', 'minmax', 'monoflop',
        'saving-switch-on-delay', 'switch-on-delay', 'comparator',
        'counter', 'edge-detection', 'flipflop-rs', 'flipflop-sr',
        'minmax-since-reset', 'ping-function-block', 'random-controller',
        'shift-register', 'status-monitor', 'stepper',
        'switch-on-and-off-delay', '2-position-controller',
        '3-position-controller', 'mail-generator', 'moving-average',
        'pulse-meter', 'ramp-controller', 'scaler', 'up-down-counter',
        'wind-gauge', 'call-generator', 'energy-flow-monitor',
        'maintenance-counter', 'door-window-monitor', 'text-generator',
        'value-validator',
    ],
    3: [  # Automation - 19 types
        'automatic-rule', 'scene', 'fixed-value-meter',
        'sequential-controller', 'differential-threshold-switch',
        'switch', 'threshold-switch', 'selection-switch-onoff',
        'stairwell-light-switch', 'long-click', 'multifunction-switch',
        'radio-buttons-16x', 'radio-buttons', 'selection-switch-plus',
        'sequencer', 'multiclick', 'push-switch',
        'selection-switch-plus-minus', 'timerscheduler',
    ],
}
# Tiers 4-12 are the remaining types
_assigned = set()
for slugs in TIER_SLUGS.values():
    _assigned.update(slugs)


def gen_uuid():
    """Generate a Loxone-style UUID."""
    u = uuid.uuid4().hex
    return f'{u[:8]}-{u[8:12]}-{u[12:16]}-ffff000000000000'


def load_slug_to_type():
    with open(SCHEMA_DIR / 'slug-to-type.json') as f:
        return json.load(f)


def cmd_generate(args):
    """Add validation blocks to config XML."""
    slug_to_type = load_slug_to_type()

    # Load existing config
    tree = ET.parse(CONFIG_PATH)
    root = tree.getroot()

    # Get existing types
    existing_types = set()
    for ctrl in root.iter('C'):
        existing_types.add(ctrl.get('Type'))

    # Get slugs for requested tier(s)
    if args.tier == 0:
        # All tiers
        slugs = list(slug_to_type.keys())
    elif args.tier in TIER_SLUGS:
        slugs = TIER_SLUGS[args.tier]
    else:
        print(f"Unknown tier {args.tier}. Valid: 0 (all), 1, 2, 3")
        return

    # Filter to schemas that exist
    with open(SCHEMA_DIR / 'index.json') as f:
        index = json.load(f)

    added = 0
    skipped = 0
    for slug in sorted(slugs):
        if slug not in slug_to_type:
            print(f"  ⚠ {slug}: no xml_type mapping")
            continue

        xml_type = slug_to_type[slug]

        if xml_type in existing_types:
            print(f"  ⏭ {slug} ({xml_type}): already in config")
            skipped += 1
            continue

        # Create minimal element
        elem = ET.SubElement(root, 'C')
        elem.set('Type', xml_type)
        elem.set('V', '175')
        elem.set('U', gen_uuid())
        elem.set('Title', f'_validate_{slug}')

        print(f"  ✚ {slug} → {xml_type}")
        added += 1
        existing_types.add(xml_type)

    if added > 0:
        # Backup original
        import shutil
        shutil.copy2(CONFIG_PATH, BACKUP_PATH)
        print(f"\n  Backup: {BACKUP_PATH}")

        # Save modified config
        tree.write(str(CONFIG_PATH), encoding='unicode', xml_declaration=True)
        print(f"  Saved: {CONFIG_PATH}")

    print(f"\nAdded {added}, skipped {skipped} (already exist)")
    print(f"\nNext steps:")
    print(f"  1. Open {CONFIG_PATH.name} in Loxone Config")
    print(f"  2. Loxone Config will generate <Co> connectors for new blocks")
    print(f"  3. Save the config in Loxone Config")
    print(f"  4. Run: python3 scripts/validate-schemas.py extract")


def cmd_extract(args):
    """Extract connector data from config XML after Loxone Config processing."""
    slug_to_type = load_slug_to_type()
    type_to_slug = {v: k for k, v in slug_to_type.items()}

    tree = ET.parse(CONFIG_PATH)
    root = tree.getroot()

    results = {}
    for ctrl in root.iter('C'):
        xml_type = ctrl.get('Type')
        title = ctrl.get('Title', '')

        # Only process our validation blocks or any typed blocks
        slug = type_to_slug.get(xml_type)
        if not slug:
            continue

        connectors = []
        for co in ctrl.findall('Co'):
            k = co.get('K')
            has_input = bool(co.findall('In'))
            default = co.get('Def')
            connectors.append({
                'key': k,
                'has_input_wiring': has_input,
                'default': default,
            })

        if xml_type not in results or title.startswith('_validate_'):
            results[xml_type] = {
                'slug': slug,
                'title': title,
                'connectors': connectors,
                'connector_count': len(connectors),
            }

    # Save extracted data
    out_path = SCHEMA_DIR / 'extracted-connectors.json'
    with open(out_path, 'w') as f:
        json.dump(results, f, indent=2, sort_keys=True)

    print(f"Extracted connectors for {len(results)} types → {out_path}")
    for xml_type, data in sorted(results.items()):
        print(f"  {data['slug']:40} ({xml_type}): {data['connector_count']} connectors")


def cmd_crosswalk(args):
    """Build crosswalk entries from extracted connector data."""
    # Load extracted data
    extracted_path = SCHEMA_DIR / 'extracted-connectors.json'
    if not extracted_path.exists():
        print("No extracted data. Run 'extract' first.")
        return

    with open(extracted_path) as f:
        extracted = json.load(f)

    # Load existing crosswalk
    crosswalk_path = SCHEMA_DIR / 'crosswalk.json'
    with open(crosswalk_path) as f:
        crosswalk = json.load(f)

    # Load schemas for comparison
    slug_to_type = load_slug_to_type()

    added = 0
    for xml_type, data in sorted(extracted.items()):
        slug = data['slug']

        # Skip already validated types
        if xml_type in crosswalk['types']:
            continue

        # Load KB schema
        schema_path = SCHEMA_DIR / f'{slug}.json'
        if not schema_path.exists():
            continue

        with open(schema_path) as f:
            schema = json.load(f)

        # Build connector mapping
        # For now, just record the XML keys — manual review needed for abbreviation mapping
        connectors = {}
        for co in data['connectors']:
            connectors[co['key']] = {
                'ux': co['key'],  # placeholder — needs manual mapping
                'dir': 'unknown',
                'desc': '',
                'has_default': co['default'] is not None,
            }

        crosswalk['types'][xml_type] = {
            'kb_slug': slug,
            'ux_validated': False,
            'auto_extracted': True,
            'connectors': connectors,
        }
        added += 1

    with open(crosswalk_path, 'w') as f:
        json.dump(crosswalk, f, indent=2, ensure_ascii=False)

    print(f"Added {added} types to crosswalk.json")
    print(f"Total types in crosswalk: {len(crosswalk['types'])}")


def main():
    parser = argparse.ArgumentParser(description='Validate schemas against config XML')
    sub = parser.add_subparsers(dest='cmd')

    gen = sub.add_parser('generate', help='Add validation blocks to config')
    gen.add_argument('--tier', type=int, default=1, help='Tier number (1-3, or 0 for all)')

    sub.add_parser('extract', help='Extract connector data from config')
    sub.add_parser('crosswalk', help='Build crosswalk from extracted data')

    args = parser.parse_args()
    if args.cmd == 'generate':
        cmd_generate(args)
    elif args.cmd == 'extract':
        cmd_extract(args)
    elif args.cmd == 'crosswalk':
        cmd_crosswalk(args)
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
