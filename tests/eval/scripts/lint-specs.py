#!/usr/bin/env python3
"""Lint eval case simulation specs against fixture.Loxone.

Validates that simulation specs reference blocks that actually exist in the
fixture, checks edge-sensitivity rules, DayTimer time requirements, and
LightController2 scene-latching risks.

Exit codes: 0 = clean, 1 = errors found, 2 = setup failure.
"""

import json
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

EVAL_DIR = Path(__file__).resolve().parent.parent
FIXTURE_PATH = EVAL_DIR / "fixture.Loxone"
CASES_DIR = EVAL_DIR / "cases"

EDGE_SENSITIVE_TYPES = frozenset({
    "FlipFlop", "RSFlipFlop", "SRFlipFlop", "State",
    "EdgeDetection", "MultiClick", "MultiFuncSW", "LongClick",
})

SCHEDULE_KEYWORDS = frozenset({
    "schedule", "timer", "night", "daytime", "morning", "evening",
})

# Structural container types — not functional blocks
SKIP_TYPES = frozenset({
    "Document", "Place", "Category", "Page", "Program",
    "VirtualInCaption", "WeatherServer", "LightscenesC",
    "LightsceneC", "TreeDevice", "LoxAIRDevice",
})


# ── Fixture parsing ────────────────────────────────────────────────


def parse_fixture(path: Path) -> dict[str, list[dict]]:
    """Parse fixture XML and build block index.

    Returns {title: [{"type": str, "room": str|None, "connectors": set}]}
    """
    tree = ET.parse(path)
    root = tree.getroot()

    # Map Place UUIDs → room names
    room_map: dict[str, str] = {}
    for el in root.iter("C"):
        if el.get("Type") == "Place":
            room_map[el.get("U", "")] = el.get("Title", "")

    blocks: dict[str, list[dict]] = {}

    def walk(el, page_room=None):
        etype = el.get("Type", "")
        title = el.get("Title", "")

        cur_page = page_room
        if etype == "Page":
            cur_page = title

        if etype and title and etype not in SKIP_TYPES:
            room = None
            io = el.find("IoData")
            if io is not None and io.get("Pr"):
                room = room_map.get(io.get("Pr"))
            if room is None:
                room = cur_page

            connectors = {co.get("K") for co in el.findall("Co") if co.get("K")}
            blocks.setdefault(title, []).append({
                "type": etype,
                "room": room,
                "connectors": connectors,
            })

        for child in el:
            if child.tag == "C":
                walk(child, cur_page)

    for child in root:
        if child.tag == "C":
            walk(child)

    return blocks


# ── Signal-key parsing ─────────────────────────────────────────────


_RE_ROOM = re.compile(r"^(.+?)\s*\[([^\]]+)\](?:\.(\w+))?$")
_RE_CONN = re.compile(r"^(.+)\.([A-Za-z]\w*)$")


def parse_signal_key(key: str) -> tuple[str, str | None, str | None]:
    """Parse signal key → (title, room, connector).

    Accepted formats:
      "Title [Room].Conn"  → (Title, Room, Conn)
      "Title.Conn"         → (Title, None, Conn)
      "Title [Room]"       → (Title, Room, None)
      "Title"              → (Title, None, None)
    """
    m = _RE_ROOM.match(key)
    if m:
        return m.group(1).strip(), m.group(2).strip(), m.group(3)
    m = _RE_CONN.match(key)
    if m:
        return m.group(1).strip(), None, m.group(2)
    return key.strip(), None, None


# ── Helpers ────────────────────────────────────────────────────────


def block_exists(title: str, room: str | None, blocks: dict) -> bool:
    entries = blocks.get(title, [])
    if not entries:
        return False
    if room is None:
        return True
    return any(e["room"] == room for e in entries)


def block_type_from_fixture(title: str, room: str | None, blocks: dict) -> str | None:
    entries = blocks.get(title, [])
    if not entries:
        return None
    if room:
        for e in entries:
            if e["room"] == room:
                return e["type"]
    return entries[0]["type"]


def resolve_type(
    title: str,
    room: str | None,
    fixture_blocks: dict,
    nb_titles: dict[str, str],
    nb_types: set[str],
) -> str | None:
    """Resolve a block's type from fixture first, then new_blocks."""
    t = block_type_from_fixture(title, room, fixture_blocks)
    if t:
        return t
    t = nb_titles.get(title)
    if t:
        return t
    # Block title may equal the type name (e.g. "MultiClick")
    if title in nb_types:
        return title
    return None


def sim_has_steps(sim: dict) -> bool:
    return isinstance(sim.get("steps"), list)


def collect_keys(sim: dict) -> tuple[list[str], list[str]]:
    """Return (input_keys, output_keys) from a sim spec."""
    inp: list[str] = []
    out: list[str] = []
    if sim_has_steps(sim):
        for step in sim["steps"]:
            inp.extend(step.get("inputs", {}).keys())
            out.extend(step.get("expected_outputs", {}).keys())
    else:
        inp.extend(sim.get("inputs", {}).keys())
        out.extend(sim.get("expected_outputs", {}).keys())
    return inp, out


def sim_has_time(sim: dict) -> bool:
    if "time" in sim:
        return True
    if sim_has_steps(sim):
        return any("time" in s for s in sim["steps"])
    return False


# ── Per-case linting ───────────────────────────────────────────────


def lint_case(case: dict, fixture_blocks: dict) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []

    expected = case.get("expected") or {}
    sims = expected.get("simulation", [])
    if isinstance(sims, dict):
        sims = [sims]
    if not sims:
        return errors, warnings

    new_blocks = expected.get("new_blocks") or []
    utterance = (case.get("utterance") or "").lower()

    # Index new_blocks by title and collect types
    nb_types: set[str] = set()
    nb_titles: dict[str, str] = {}
    for nb in new_blocks:
        bt = nb.get("type", "")
        nb_types.add(bt)
        for field in ("title", "title_contains"):
            if nb.get(field):
                nb_titles[nb[field]] = bt

    for sim in sims:
        input_keys, output_keys = collect_keys(sim)
        uses_steps = sim_has_steps(sim)
        has_time = sim_has_time(sim)

        # ── Rule 1: expected_outputs must reference fixture blocks ──
        for key in output_keys:
            title, room, _ = parse_signal_key(key)
            if not block_exists(title, room, fixture_blocks):
                errors.append(
                    f"Rule 1: expected_output '{key}' references "
                    f"non-fixture block '{title}'"
                )

        # ── Rule 2: inputs must reference fixture blocks ───────────
        for key in input_keys:
            title, room, _ = parse_signal_key(key)
            if not block_exists(title, room, fixture_blocks):
                errors.append(
                    f"Rule 2: input '{key}' references "
                    f"non-fixture block '{title}'"
                )

        # ── Rule 3: edge-sensitive blocks need steps ───────────────
        if not uses_steps:
            edge_flagged = False

            # Check types of blocks in expected_outputs
            for key in output_keys:
                title, room, _ = parse_signal_key(key)
                rt = resolve_type(title, room, fixture_blocks, nb_titles, nb_types)
                if rt and rt in EDGE_SENSITIVE_TYPES:
                    errors.append(
                        f"Rule 3: expected_output '{key}' is edge-sensitive "
                        f"(type {rt}) but spec has no 'steps'"
                    )
                    edge_flagged = True

            # Fallback: check if new_blocks include edge-sensitive types
            if not edge_flagged:
                for nb in new_blocks:
                    if nb.get("type", "") in EDGE_SENSITIVE_TYPES:
                        sim_name = sim.get("name", "unnamed")
                        errors.append(
                            f"Rule 3: circuit uses edge-sensitive block "
                            f"'{nb['type']}' but sim '{sim_name}' has no 'steps'"
                        )
                        break

        # ── Rule 4: DayTimer circuits need a time field ────────────
        utterance_has_schedule = any(kw in utterance for kw in SCHEDULE_KEYWORDS)
        if utterance_has_schedule and "DayTimer" in nb_types and not has_time:
            errors.append(
                "Rule 4: utterance mentions schedule/time, circuit uses "
                "DayTimer, but no 'time' in spec"
            )

        # ── Rule 5: scene latching warning ─────────────────────────
        for key in output_keys:
            title, room, conn = parse_signal_key(key)
            if conn and conn.startswith("Sel"):
                rt = block_type_from_fixture(title, room, fixture_blocks)
                if rt == "LightController2":
                    warnings.append(
                        f"Rule 5: checking {conn} on LightController2 "
                        f"'{title}' — scene may latch between specs"
                    )

    return errors, warnings


# ── Main ───────────────────────────────────────────────────────────


def main() -> int:
    if not FIXTURE_PATH.exists():
        print(f"ERROR: fixture not found: {FIXTURE_PATH}", file=sys.stderr)
        return 2
    if not CASES_DIR.is_dir():
        print(f"ERROR: cases dir not found: {CASES_DIR}", file=sys.stderr)
        return 2

    fixture_blocks = parse_fixture(FIXTURE_PATH)
    case_files = sorted(CASES_DIR.glob("*.json"))
    if not case_files:
        print("No case files found.", file=sys.stderr)
        return 2

    total_cases = 0
    total_errors = 0
    total_warnings = 0
    passed = 0
    messages: list[str] = []

    for cf in case_files:
        try:
            cases = json.loads(cf.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as exc:
            messages.append(f"\nERROR: {cf.name}\n  Could not load: {exc}")
            total_errors += 1
            continue
        if not isinstance(cases, list):
            continue

        for case in cases:
            total_cases += 1
            cid = case.get("id", "unknown")
            errs, warns = lint_case(case, fixture_blocks)

            for e in errs:
                total_errors += 1
                messages.append(f"\nERROR: {cf.name} / {cid}\n  {e}")
            for w in warns:
                total_warnings += 1
                messages.append(f"\nWARNING: {cf.name} / {cid}\n  {w}")
            if not errs and not warns:
                passed += 1

    print(f"Linting {total_cases} cases across {len(case_files)} files...")
    for msg in messages:
        print(msg)
    print(f"\nOK: {passed} cases pass, {total_errors} errors, {total_warnings} warnings")

    return 1 if total_errors > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
