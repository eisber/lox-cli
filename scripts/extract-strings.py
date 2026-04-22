#!/usr/bin/env python3
"""Extract localized strings from Loxone Config resource DLLs and Qt .qm files.

Parses LoxoneConfigres_*.dll (PE string resources) and loxoneqtgui_*.qm
(Qt compiled translations) to build per-locale JSON string maps.

Usage:
    python3 scripts/extract-strings.py
    python3 scripts/extract-strings.py --config-dir "C:\\Program Files (x86)\\Loxone\\LoxoneConfig"
    python3 scripts/extract-strings.py --locale DEU
"""

import argparse
import json
import os
import re
import struct
import subprocess
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)

DEFAULT_CONFIG_DIR = "/mnt/c/Program Files (x86)/Loxone/LoxoneConfig"

LOCALES = [
    "BGR", "CAT", "CHS", "CSY", "DEU", "ENG", "ENU", "ESN",
    "FRA", "HUN", "ITA", "NLD", "NOR", "PLK", "PTG", "ROM",
    "RUS", "SKY", "THA", "TRK",
]


def extract_strings_from_dll(dll_path: str) -> list[str]:
    """Extract printable strings from a PE DLL using the `strings` command."""
    try:
        result = subprocess.run(
            ["strings", "-n", "4", dll_path],
            capture_output=True, text=True, timeout=30,
        )
        return [s.strip() for s in result.stdout.splitlines() if s.strip()]
    except Exception as e:
        print(f"  Warning: could not extract from {dll_path}: {e}", file=sys.stderr)
        return []


def classify_strings(raw_strings: list[str]) -> dict:
    """Classify extracted strings into categories."""
    classified = {
        "control_types": [],
        "properties": [],
        "ui_labels": [],
        "error_messages": [],
        "tooltips": [],
        "other": [],
    }

    for s in raw_strings:
        if len(s) < 3 or len(s) > 200:
            continue
        # Skip binary/hex/path noise
        if re.match(r'^[0-9a-fA-F]{8,}$', s) or s.startswith('\\') or s.startswith('/'):
            continue

        lower = s.lower()
        if any(kw in lower for kw in ['error', 'invalid', 'failed', 'cannot', 'unable']):
            classified["error_messages"].append(s)
        elif any(kw in lower for kw in ['tooltip', 'hint', 'description']):
            classified["tooltips"].append(s)
        elif re.match(r'^[A-Z][a-z]+[A-Z]', s):  # CamelCase = likely type/property
            if len(s) < 40:
                classified["control_types"].append(s)
        elif re.match(r'^[A-Z][a-z]', s) and len(s) < 50:
            classified["ui_labels"].append(s)
        else:
            classified["other"].append(s)

    # Deduplicate
    for key in classified:
        classified[key] = sorted(set(classified[key]))

    return classified


def extract_locale(config_dir: str, locale: str) -> dict:
    """Extract strings for a single locale."""
    dll_path = os.path.join(config_dir, f"LoxoneConfigres_{locale}.dll")
    qm_path = os.path.join(config_dir, f"loxoneqtgui_{locale}.qm")

    all_strings = []

    if os.path.exists(dll_path):
        all_strings.extend(extract_strings_from_dll(dll_path))

    if os.path.exists(qm_path):
        # Qt .qm files are binary; extract with strings as well
        all_strings.extend(extract_strings_from_dll(qm_path))

    return classify_strings(all_strings)


def main():
    parser = argparse.ArgumentParser(description="Extract i18n strings from Loxone Config")
    parser.add_argument("--config-dir", default=DEFAULT_CONFIG_DIR,
                        help="Path to Loxone Config installation")
    parser.add_argument("--locale", help="Extract single locale (e.g. DEU)")
    parser.add_argument("--out", default=os.path.join(REPO_DIR, "i18n"),
                        help="Output directory")
    args = parser.parse_args()

    os.makedirs(args.out, exist_ok=True)

    locales = [args.locale] if args.locale else LOCALES

    for locale in locales:
        print(f"Extracting {locale}...")
        data = extract_locale(args.config_dir, locale)

        total = sum(len(v) for v in data.values())
        out_path = os.path.join(args.out, f"{locale.lower()}.json")
        with open(out_path, "w") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        print(f"  ✓ {total} strings → {out_path}")

    print(f"\n✓ Done. {len(locales)} locales extracted to {args.out}/")


if __name__ == "__main__":
    main()
