#!/usr/bin/env python3
"""Apply UX-validated corrections to schemas.

Run after build-schemas.py to apply corrections discovered via
Loxone Config UX validation (Anschluss anzeigen popup comparison).

Uses docs/schemas/crosswalk.json as the source of truth for:
- XML key ↔ UX abbreviation mapping
- Missing connectors (e.g., AlarmClock Ca input)
- Input/output misclassification fixes
- Range expansion (e.g., Lc1-4 → Lc1, Lc2, Lc3, Lc4)

Usage:
    python3 scripts/apply-ux-corrections.py
"""

import json
import os
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)
SCHEMA_DIR = os.path.join(REPO_DIR, 'docs', 'schemas')


def load_crosswalk():
    path = os.path.join(SCHEMA_DIR, 'crosswalk.json')
    with open(path) as f:
        return json.load(f)


def add_xml_keys(schema, connectors):
    """Add xml_key field to schema connectors from crosswalk data."""
    ux_to_xml = {info['ux']: xml_key for xml_key, info in connectors.items()}
    for section in ['inputs', 'outputs', 'parameters']:
        for item in schema.get(section, []):
            abbr = item.get('abbreviation', '')
            if abbr in ux_to_xml:
                item['xml_key'] = ux_to_xml[abbr]


def fix_alarm_clock(schema):
    """Add missing Ca (Confirm alarm) input."""
    has_ca = any(i['abbreviation'] == 'Ca' for i in schema['inputs'])
    if not has_ca:
        schema['inputs'].insert(0, {
            "abbreviation": "Ca",
            "summary": "Confirm alarm",
            "description": "Confirms the alarm. When 'Acknowledgement required' is enabled, the alarm must be confirmed to stop.",
            "unit": "-",
            "value_range": "0/1",
            "xml_key": "Acknowledge"
        })
        schema['input_count'] = len(schema['inputs'])


def fix_light_controller(schema):
    """Fix I/O misclassification and expand ranges."""
    output_abbrs = {'On', 'Alarm', 'Buzzer', 'Br', 'DisPc', 'P', 'Rtd', 'MBr'}
    moved, new_inputs = [], []

    for inp in schema['inputs']:
        abbr = inp['abbreviation']
        if abbr in output_abbrs:
            moved.append(inp)
        elif abbr == 'Lc1-4':
            for i in range(1, 5):
                new_inputs.append({
                    'abbreviation': f'Lc{i}', 'summary': f'Light circuit {i}',
                    'description': inp.get('description', ''),
                    'unit': '-', 'value_range': inp.get('value_range', ''),
                    'xml_key': f'I{i}'
                })
        elif abbr == 'T5/1-8':
            for i in range(1, 9):
                new_inputs.append({
                    'abbreviation': f'T5/{i}', 'summary': f'Trigger Mood selection {i}',
                    'description': inp.get('description', ''),
                    'unit': '-', 'value_range': inp.get('value_range', ''),
                    'xml_key': f'Sel{i}'
                })
        else:
            new_inputs.append(inp)

    if not any(i['abbreviation'] == 'Mood' for i in new_inputs):
        new_inputs.append({
            'abbreviation': 'Mood', 'summary': 'Select mood by ID',
            'description': 'Select a specific mood by its ID number.',
            'unit': '-', 'value_range': '0...∞', 'xml_key': 'Select'
        })

    schema['inputs'] = new_inputs

    new_outputs = []
    for out in schema['outputs']:
        if out['abbreviation'] == 'Lc1-18':
            for i in range(1, 19):
                new_outputs.append({
                    'abbreviation': f'AQ{i}', 'summary': f'Analog output {i}',
                    'description': out.get('description', ''),
                    'unit': '-', 'value_range': out.get('value_range', ''),
                    'xml_key': f'AQ{i}'
                })
        else:
            new_outputs.append(out)
    new_outputs.extend(moved)
    schema['outputs'] = new_outputs

    schema['input_count'] = len(schema['inputs'])
    schema['output_count'] = len(schema['outputs'])


FIXERS = {
    'AlarmClock': fix_alarm_clock,
    'LightController2': fix_light_controller,
}


def main():
    crosswalk = load_crosswalk()
    fixed = 0

    for xml_type, data in crosswalk['types'].items():
        kb_slug = data.get('kb_slug')
        if not kb_slug:
            continue

        schema_file = os.path.join(SCHEMA_DIR, f'{kb_slug}.json')
        if not os.path.exists(schema_file):
            print(f"⚠️  {kb_slug}.json not found, skipping")
            continue

        with open(schema_file) as f:
            schema = json.load(f)

        # Add xml_key mappings
        add_xml_keys(schema, data.get('connectors', {}))

        # Apply type-specific fixes
        fixer = FIXERS.get(xml_type)
        if fixer:
            fixer(schema)

        # Mark UX validated
        if data.get('ux_validated'):
            schema['ux_validated'] = True

        with open(schema_file, 'w') as f:
            json.dump(schema, f, indent=2, ensure_ascii=False)

        status = '✅' if data.get('ux_validated') else '⬜'
        print(f"{status} {kb_slug}.json: {schema.get('input_count',0)}i/{schema.get('output_count',0)}o/{schema.get('parameter_count',0)}p")
        fixed += 1

    print(f"\n✓ Applied corrections to {fixed} schemas")


if __name__ == '__main__':
    main()
