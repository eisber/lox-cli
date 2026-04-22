#!/usr/bin/env python3
"""Signal-propagation simulator for Loxone eval configs.

Parses agent-built .Loxone XML configs, builds a wiring graph, implements
simple block logic, and validates simulation specs from eval cases.

Usage:
    python3 run-sim-validation.py [--configs-dir /tmp/eval-llm]
                                  [--cases-dir tests/eval/cases]
                                  [--llm-report tests/eval/reports/llm-report.json]
                                  [--output tests/eval/reports/sim-validation-report.json]
"""

from __future__ import annotations

import argparse
import json
import glob
import os
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict, deque
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Set, Tuple

# ── Constants ─────────────────────────────────────────────────────────

DIGITAL_THRESHOLD = 0.5

INFRA_TYPES = frozenset({
    "Document", "Page", "Place", "Category", "Program",
    "User", "UserCaption", "LoxCaption", "VirtualInCaption",
    "LightscenesC", "LightsceneC", "Lightscene",
    "TreeDevice", "LoxAIRDevice", "NetworkDevice",
    "TreeAsensor", "TreeDsensor", "TreeAactuator", "TreeDactuator",
    "IoData", "Display", "HP", "Const", "Note", "SET",
    "WeatherServer",
})

# ── Data types ────────────────────────────────────────────────────────


@dataclass
class Connector:
    uuid: str
    key: str
    block_uuid: str
    default: Optional[float] = None
    # UUIDs of source connectors wired into this one
    input_sources: List[str] = field(default_factory=list)


@dataclass
class Block:
    uuid: str
    title: str
    btype: str
    connectors: Dict[str, Connector] = field(default_factory=dict)
    # key → Connector (by connector key like "Q", "Input1", etc.)
    conn_by_key: Dict[str, Connector] = field(default_factory=dict)


@dataclass
class SimGraph:
    blocks: Dict[str, Block] = field(default_factory=dict)
    # connector_uuid → Connector
    conn_index: Dict[str, Connector] = field(default_factory=dict)
    # title → list of Block (multiple blocks can share titles)
    title_index: Dict[str, List[Block]] = field(default_factory=dict)


# ── XML Parsing ───────────────────────────────────────────────────────

def parse_config(xml_path: str) -> SimGraph:
    """Parse a .Loxone XML file into a SimGraph."""
    tree = ET.parse(xml_path)
    root = tree.getroot()
    graph = SimGraph()

    def walk(elem: ET.Element):
        etype = elem.get("Type", "")
        euuid = elem.get("U", "")
        title = elem.get("Title", "")

        if etype and etype not in INFRA_TYPES and euuid:
            block = Block(uuid=euuid, title=title, btype=etype)

            for child in elem:
                if child.tag == "Co" or child.get("Type", "") == "Co":
                    cu = child.get("U", "")
                    ck = child.get("K", "")
                    if not cu or not ck:
                        continue

                    defval = None
                    raw_def = child.get("Def")
                    if raw_def is not None:
                        try:
                            defval = float(raw_def)
                        except ValueError:
                            pass

                    sources = []
                    for inp in child:
                        if inp.tag == "In":
                            src = inp.get("Input", "")
                            if src and src not in sources:
                                sources.append(src)

                    conn = Connector(
                        uuid=cu, key=ck, block_uuid=euuid,
                        default=defval, input_sources=sources,
                    )
                    block.connectors[cu] = conn
                    block.conn_by_key[ck] = conn

            graph.blocks[euuid] = block
            graph.conn_index.update(block.connectors)
            graph.title_index.setdefault(title, []).append(block)

        for child in elem:
            walk(child)

    walk(root)
    return graph


# ── Block Logic ───────────────────────────────────────────────────────

def is_high(v: float) -> bool:
    return v >= DIGITAL_THRESHOLD


def bool_signal(b: bool) -> float:
    return 1.0 if b else 0.0


def eval_block(block: Block, input_vals: Dict[str, float]) -> Dict[str, float]:
    """Evaluate a block's logic given its input connector values.

    input_vals: connector_key → value for this block's input connectors.
    Returns: connector_key → output value.
    """
    btype = block.btype
    outputs: Dict[str, float] = {}

    # Helper to get an input by key, with fallback to connector default
    def inp(key: str, fallback: float = 0.0) -> float:
        if key in input_vals:
            return input_vals[key]
        conn = block.conn_by_key.get(key)
        if conn and conn.default is not None:
            return conn.default
        return fallback

    def inp_or_param(input_key: str, param_key: str, fallback: float = 0.0) -> float:
        """Get input value, falling back to a parameter connector's default."""
        if input_key in input_vals:
            return input_vals[input_key]
        conn = block.conn_by_key.get(param_key)
        if conn and conn.default is not None:
            return conn.default
        return fallback

    # Gather all numbered inputs (I1..I8, Input1..Input8)
    def gather_inputs(prefixes: List[str] = None, max_n: int = 8) -> List[float]:
        if prefixes is None:
            prefixes = ["I", "Input"]
        vals = []
        for pfx in prefixes:
            for i in range(1, max_n + 1):
                k = f"{pfx}{i}"
                if k in input_vals:
                    vals.append(input_vals[k])
                elif k in block.conn_by_key:
                    c = block.conn_by_key[k]
                    if c.input_sources:
                        vals.append(input_vals.get(k, 0.0))
        return vals

    # ── Comparison blocks ──
    if btype in ("GreaterEqual", "GreaterOrEqual"):
        left = inp("Input1")
        right = inp_or_param("Input2", "Input2")
        outputs["Q"] = bool_signal(left >= right)

    elif btype == "Less":
        left = inp("Input1")
        right = inp_or_param("Input2", "Input2")
        outputs["Q"] = bool_signal(left < right)

    elif btype == "Greater":
        left = inp("Input1")
        right = inp_or_param("Input2", "Input2")
        outputs["Q"] = bool_signal(left > right)

    elif btype in ("LessEqual", "LessOrEqual"):
        left = inp("Input1")
        right = inp_or_param("Input2", "Input2")
        outputs["Q"] = bool_signal(left <= right)

    elif btype in ("AnalogThresholdTrigger", "AnalogComparator"):
        # Input compared against On/Off thresholds
        val = inp("Input")
        on_thresh = inp("On", 0.5)
        off_thresh = inp("Off", 0.0)
        # Simplified: output high if input >= on_threshold
        outputs["Q"] = bool_signal(val >= on_thresh)
        outputs["RisingEdge"] = 0.0
        outputs["FallingEdge"] = 0.0

    # ── Logic blocks ──
    elif btype == "And":
        vals = gather_inputs()
        if not vals:
            val = inp("Input", 0.0)
            outputs["Q"] = bool_signal(is_high(val))
        else:
            outputs["Q"] = bool_signal(all(is_high(v) for v in vals))

    elif btype == "Or":
        vals = gather_inputs()
        if not vals:
            val = inp("Input", 0.0)
            outputs["Q"] = bool_signal(is_high(val))
        else:
            outputs["Q"] = bool_signal(any(is_high(v) for v in vals))

    elif btype == "Not":
        val = inp("Input", inp("I1", 0.0))
        outputs["Q"] = bool_signal(not is_high(val))

    elif btype == "Xor":
        vals = gather_inputs()
        high_count = sum(1 for v in vals if is_high(v))
        outputs["Q"] = bool_signal(high_count % 2 == 1)

    elif btype == "Nand":
        vals = gather_inputs()
        outputs["Q"] = bool_signal(not all(is_high(v) for v in vals)) if vals else 1.0

    elif btype == "Nor":
        vals = gather_inputs()
        outputs["Q"] = bool_signal(not any(is_high(v) for v in vals)) if vals else 1.0

    elif btype in ("Equal", "EqualV"):
        left = inp("Input1")
        right = inp_or_param("Input2", "Input2")
        outputs["Q"] = bool_signal(abs(left - right) < 0.001)

    elif btype == "NotEqual":
        left = inp("Input1")
        right = inp_or_param("Input2", "Input2")
        outputs["Q"] = bool_signal(abs(left - right) >= 0.001)

    # ── Math blocks ──
    elif btype == "Mult":
        a = inp("Input1", 1.0)
        b = inp_or_param("Input2", "Input2", 1.0)
        outputs["AQ"] = a * b

    elif btype == "Add":
        a = inp("Input1")
        b = inp_or_param("Input2", "Input2")
        outputs["AQ"] = a + b

    elif btype == "Add4":
        total = 0.0
        for i in range(1, 5):
            total += inp(f"Input{i}")
        outputs["AQ"] = total

    elif btype == "Sub":
        a = inp("Input1")
        b = inp_or_param("Input2", "Input2")
        outputs["AQ"] = a - b

    elif btype == "Div":
        a = inp("Input1")
        b = inp_or_param("Input2", "Input2", 1.0)
        outputs["AQ"] = a / b if b != 0 else 0.0

    elif btype == "Mod":
        a = inp("Input1")
        b = inp_or_param("Input2", "Input2", 1.0)
        outputs["AQ"] = a % b if b != 0 else 0.0

    elif btype == "Int":
        outputs["AQ"] = float(int(inp("Input1")))

    elif btype in ("Minmax", "AMinmax"):
        a = inp("Input1")
        lo = inp("Min", 0.0)
        hi = inp("Max", 100.0)
        outputs["AQ"] = max(lo, min(hi, a))

    elif btype == "AnalogScaler":
        a = inp("Input")
        min_in = inp("MinIn", 0.0)
        max_in = inp("MaxIn", 1.0)
        min_out = inp("MinOut", 0.0)
        max_out = inp("MaxOut", 100.0)
        if max_in != min_in:
            ratio = (a - min_in) / (max_in - min_in)
            outputs["AQ"] = min_out + ratio * (max_out - min_out)
        else:
            outputs["AQ"] = min_out

    elif btype == "Formula":
        # Can't evaluate arbitrary formulas; pass through input
        outputs["AQ"] = inp("Input1", inp("I1", 0.0))

    elif btype in ("Average", "Avg"):
        vals = gather_inputs()
        outputs["AQ"] = sum(vals) / len(vals) if vals else 0.0

    # ── Timer / Pulse blocks (simplified) ──
    elif btype == "Monoflop":
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype in ("OnPulseDelay", "OnDelay"):
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype == "OffDelay":
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype == "OnOffDelay":
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype == "StairwayLS":
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype == "EdgeDetection":
        # Simplified: pass through
        trigger = inp("Input", 0.0)
        outputs["Q"] = bool_signal(is_high(trigger))
        outputs["UpEdge"] = 0.0
        outputs["DownEdge"] = 0.0

    elif btype == "PulseGen":
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    # ── State blocks ──
    elif btype in ("Memory", "AMemory"):
        s = inp("S", inp("Set", 0.0))
        r = inp("R", inp("Reset", 0.0))
        if is_high(s):
            outputs["Q"] = 1.0
            outputs["AQ"] = s
        elif is_high(r):
            outputs["Q"] = 0.0
            outputs["AQ"] = 0.0
        else:
            outputs["Q"] = 0.0
            outputs["AQ"] = 0.0

    elif btype in ("FlipFlop", "RSFlipFlop"):
        s = inp("S", inp("Set", 0.0))
        r = inp("R", inp("Reset", 0.0))
        outputs["Q"] = bool_signal(is_high(s) and not is_high(r))

    elif btype == "Counter":
        trigger = inp("Input", inp("InputTrigger", 0.0))
        outputs["AQ"] = 1.0 if is_high(trigger) else 0.0
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype in ("SampleHold",):
        outputs["AQ"] = inp("Input", inp("I1", 0.0))

    elif btype in ("State", "StateV"):
        outputs["AQ"] = inp("Input", inp("I1", 0.0))
        outputs["Q"] = bool_signal(is_high(inp("Input", inp("I1", 0.0))))

    elif btype == "PushButton":
        trigger = inp("InputTrigger", inp("Input", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype == "PushButton2":
        trigger = inp("InputTrigger", inp("Input", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))

    # ── I/O blocks ──
    elif btype == "VirtualIn":
        # Pass through: outputs are the "current value" (set externally)
        outputs["Q"] = inp("Q", 0.0)
        outputs["AQ"] = inp("AQ", 0.0)

    elif btype == "SysVar":
        outputs["AQ"] = inp("AQ", 0.0)

    elif btype in ("VirtualOut", "VirtualState"):
        outputs["Q1"] = inp("I1", 0.0)
        outputs["AQ"] = inp("I1", inp("Input", 0.0))
        outputs["Q"] = inp("I1", inp("Input", 0.0))

    elif btype in ("InputRef", "OutputRef"):
        outputs["AQ"] = inp("Input", inp("I1", 0.0))
        outputs["Q"] = inp("Input", inp("I1", 0.0))

    # ── Controller blocks (simplified: pass-through or trigger-based) ──
    elif btype in ("LightController2", "LightController"):
        i1 = inp("I1", 0.0)
        brightness = inp("Brightness", 0.0)
        presence = inp("Presence", 0.0)
        move = inp("Move", 0.0)
        # Simplified: output reflects inputs
        active = is_high(i1) or is_high(presence) or brightness > 0
        outputs["AQ1"] = 1.0 if is_high(i1) else (brightness if brightness > 0 else 0.0)
        outputs["AQ"] = outputs["AQ1"]
        outputs["Q"] = bool_signal(active)
        # Reflect all inputs for wiring checks
        outputs["I1"] = i1
        outputs["Presence"] = presence
        outputs["Brightness"] = brightness
        outputs["Move"] = move

    elif btype in ("JalousieUpDown2", "Jalousiemotor", "AutoJalousie"):
        # Inputs are trigger-based; we just reflect them as-is
        outputs["InputTriggerUp"] = inp("InputTriggerUp", 0.0)
        outputs["InputTriggerDown"] = inp("InputTriggerDown", 0.0)
        outputs["InputPos"] = inp("InputPos", 0.0)
        outputs["InputDisable"] = inp("InputDisable", 0.0)
        outputs["AQ"] = inp("InputPos", 0.0)
        outputs["Q"] = bool_signal(
            is_high(inp("InputTriggerUp", 0.0)) or
            is_high(inp("InputTriggerDown", 0.0))
        )

    elif btype in ("HeatIRoomController2", "IRoomcontrol"):
        temp = inp("Temp", 20.0)
        setpoint = inp("SetPoint", inp("SP", 21.0))
        # Simplified heating model: if temp < setpoint → heating demand
        if temp < setpoint:
            outputs["AQh"] = min(1.0, (setpoint - temp) / 5.0)
        else:
            outputs["AQh"] = 0.0
        if temp > setpoint + 2.0:
            outputs["AQc"] = min(1.0, (temp - setpoint - 2.0) / 5.0)
        else:
            outputs["AQc"] = 0.0
        outputs["Temp"] = temp
        # Reflect all inputs for wiring checks
        for k in block.conn_by_key:
            if k not in outputs:
                outputs[k] = inp(k, 0.0)

    elif btype == "AcControl":
        toggle_val = inp("toggle", inp("I1", 0.0))
        outputs["toggle"] = toggle_val
        outputs["I1"] = toggle_val
        outputs["status"] = inp("inTempCurr", inp("status", 0.0))
        outputs["off"] = inp("off", 0.0)
        for k in block.conn_by_key:
            if k not in outputs:
                outputs[k] = inp(k, 0.0)

    elif btype == "PresenceDetector":
        trigger = inp("InputTrigger", 0.0)
        outputs["OutputPresence"] = bool_signal(is_high(trigger))
        outputs["Q"] = bool_signal(is_high(trigger))

    elif btype in ("Alarm", "AlarmChain", "CentralAlarm"):
        trigger = inp("InputTrigger", inp("I1", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))
        outputs["AQ"] = bool_signal(is_high(trigger))

    elif btype in ("Ventilation", "Ventilation2"):
        trigger = inp("InputTrigger", inp("I1", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))
        outputs["AQ"] = 1.0 if is_high(trigger) else 0.0

    elif btype in ("DayTimer", "Calendar"):
        # Schedule blocks: simplified — output AQ always 1.0 (active)
        outputs["AQ"] = 1.0
        outputs["Qon"] = 1.0
        outputs["Qoff"] = 0.0

    elif btype == "AlarmClock":
        outputs["Q"] = 1.0
        outputs["AQ"] = 1.0

    elif btype in ("Irrigation",):
        trigger = inp("InputTrigger", inp("I1", 0.0))
        outputs["Q"] = bool_signal(is_high(trigger))
        outputs["AQ"] = 1.0 if is_high(trigger) else 0.0

    elif btype in ("Pi", "Pid"):
        # PID: simplified pass-through
        error = inp("Input", inp("I1", 0.0))
        outputs["AQ"] = error

    elif btype in ("TwoPoint", "ThreePoint"):
        val = inp("Input", 0.0)
        outputs["Q"] = bool_signal(is_high(val))
        outputs["AQ"] = val

    elif btype in ("Wallbox", "Power"):
        outputs["AQ"] = inp("Input", inp("I1", 0.0))

    # ── Catch-all: reflect all inputs to outputs ──
    else:
        # PassThrough: map inputs to common output keys
        for k, v in input_vals.items():
            outputs[k] = v
        # Also try to set standard output keys from first non-zero input
        first_val = next((v for v in input_vals.values() if v != 0.0), 0.0)
        outputs.setdefault("Q", first_val)
        outputs.setdefault("AQ", first_val)

    return outputs


# ── Signal Propagation ────────────────────────────────────────────────

def resolve_block_connector(
    graph: SimGraph, spec: str
) -> List[Tuple[Block, str]]:
    """Resolve 'Title.ConnectorKey' to (Block, connector_key) pairs.

    Tries in order: exact title → fuzzy title → block type match.
    """
    parts = spec.rsplit(".", 1)
    if len(parts) != 2:
        return []
    title, conn_key = parts

    matches = []
    # 1. Exact title match
    if title in graph.title_index:
        for b in graph.title_index[title]:
            matches.append((b, conn_key))

    # 2. Fuzzy substring match on title
    if not matches:
        for t, blocks in graph.title_index.items():
            if title in t or t in title:
                for b in blocks:
                    matches.append((b, conn_key))

    # 3. Match by block type (e.g. "PushButton" matches any PushButton block,
    #    "Mult" matches any Mult block, "Or" matches any Or block)
    if not matches:
        for block in graph.blocks.values():
            if block.btype == title:
                matches.append((block, conn_key))

    return matches


def build_dependency_order(graph: SimGraph) -> List[str]:
    """Topological sort of blocks based on wiring. Returns block UUIDs."""
    # Build adjacency: src_block → dst_block
    adj: Dict[str, Set[str]] = defaultdict(set)
    in_degree: Dict[str, int] = defaultdict(int)

    for buuid in graph.blocks:
        in_degree.setdefault(buuid, 0)

    for block in graph.blocks.values():
        for conn in block.connectors.values():
            for src_uuid in conn.input_sources:
                src_conn = graph.conn_index.get(src_uuid)
                if src_conn and src_conn.block_uuid != block.uuid:
                    src_buuid = src_conn.block_uuid
                    if block.uuid not in adj[src_buuid]:
                        adj[src_buuid].add(block.uuid)
                        in_degree[block.uuid] += 1

    # Kahn's algorithm
    queue = deque(
        buuid for buuid, deg in in_degree.items() if deg == 0
    )
    order = []
    while queue:
        cur = queue.popleft()
        order.append(cur)
        for nbr in adj.get(cur, set()):
            in_degree[nbr] -= 1
            if in_degree[nbr] == 0:
                queue.append(nbr)

    # Append any remaining (cycles)
    for buuid in graph.blocks:
        if buuid not in order:
            order.append(buuid)

    return order


def propagate(
    graph: SimGraph,
    injected: Dict[str, Dict[str, float]],
    ticks: int = 10,
    dt: float = 0.1,
) -> Dict[str, Dict[str, float]]:
    """Propagate signals through the graph.

    injected: block_uuid → {connector_key: value} for externally set values.
    Returns: block_uuid → {connector_key: output_value} after convergence.
    """
    order = build_dependency_order(graph)

    # connector_uuid → current signal value
    wire_values: Dict[str, float] = {}

    # Track which connector UUIDs are directly injected (protect from
    # block eval overwriting sensor/source outputs).
    injected_conn_uuids: Set[str] = set()

    # Initialize injected values onto their connector UUIDs
    for buuid, kvs in injected.items():
        block = graph.blocks.get(buuid)
        if not block:
            continue
        for key, val in kvs.items():
            conn = block.conn_by_key.get(key)
            if conn:
                wire_values[conn.uuid] = val
                injected_conn_uuids.add(conn.uuid)

    # Iterate ticks for convergence (at least 1 pass)
    effective_ticks = max(ticks, 1)
    for _tick in range(effective_ticks):
        for buuid in order:
            block = graph.blocks[buuid]

            # Collect input values from wired source connectors.
            # When multiple sources wire into the same connector, use
            # the max value (OR semantics for digital, dominant for analog).
            # Also record received values on input connector UUIDs.
            block_inputs: Dict[str, float] = {}
            for conn in block.connectors.values():
                if conn.input_sources:
                    best_val = None
                    for src_uuid in conn.input_sources:
                        if src_uuid in wire_values:
                            v = wire_values[src_uuid]
                            if best_val is None or abs(v) > abs(best_val):
                                best_val = v
                    if best_val is not None:
                        block_inputs[conn.key] = best_val
                        # Record received value on the input connector
                        if conn.uuid not in injected_conn_uuids:
                            wire_values[conn.uuid] = best_val

            # Override with injected values
            if buuid in injected:
                block_inputs.update(injected[buuid])

            # Evaluate block
            outputs = eval_block(block, block_inputs)

            # Write outputs to wire_values, but don't overwrite
            # directly injected values (sensor/source outputs).
            for key, val in outputs.items():
                conn = block.conn_by_key.get(key)
                if conn and conn.uuid not in injected_conn_uuids:
                    wire_values[conn.uuid] = val

    # Build final output map: block_uuid → {connector_key: value}
    result: Dict[str, Dict[str, float]] = {}
    for buuid, block in graph.blocks.items():
        block_out: Dict[str, float] = {}
        for conn in block.connectors.values():
            if conn.uuid in wire_values:
                block_out[conn.key] = wire_values[conn.uuid]
        if block_out:
            result[buuid] = block_out
    return result


# ── Output Checking ───────────────────────────────────────────────────

def check_comparator(actual: float, comparator: str, expected: float) -> bool:
    if comparator == ">":
        return actual > expected
    elif comparator == "<":
        return actual < expected
    elif comparator == ">=":
        return actual >= expected
    elif comparator == "<=":
        return actual <= expected
    elif comparator == "==":
        return abs(actual - expected) < 0.001
    elif comparator == "!=":
        return abs(actual - expected) >= 0.001
    return False


def check_outputs(
    graph: SimGraph,
    sim_results: Dict[str, Dict[str, float]],
    expected_outputs: Dict[str, Dict[str, float]],
) -> List[Dict[str, Any]]:
    """Check simulation results against expected outputs.

    Uses ANY-match semantics: if multiple blocks share the same title,
    the check passes if ANY matching block satisfies the comparators.
    Returns list of check results with pass/fail for each expected output.
    """
    checks = []
    for spec_key, comparators in expected_outputs.items():
        resolved = resolve_block_connector(graph, spec_key)
        if not resolved:
            checks.append({
                "output": spec_key,
                "status": "not_found",
                "message": f"Block/connector '{spec_key}' not found in config",
            })
            continue

        # Collect candidates: (block, conn_key, actual_value)
        candidates = []
        for block, conn_key in resolved:
            buuid = block.uuid
            block_vals = sim_results.get(buuid, {})

            if conn_key not in block.conn_by_key and conn_key not in block_vals:
                continue

            actual = block_vals.get(conn_key, 0.0)
            candidates.append((block, conn_key, actual))

        if not candidates:
            block_titles = [b.title for b, _ in resolved]
            checks.append({
                "output": spec_key,
                "status": "connector_missing",
                "message": (
                    f"Connector key from '{spec_key}' not found on "
                    f"matching blocks: {block_titles}"
                ),
            })
            continue

        # ANY-match: check if ANY candidate satisfies ALL comparators
        best_result = None
        any_passed = False
        for block, conn_key, actual in candidates:
            all_comp_pass = True
            comp_details = []
            for comp_op, comp_val in comparators.items():
                passed = check_comparator(actual, comp_op, comp_val)
                comp_details.append((comp_op, comp_val, passed))
                if not passed:
                    all_comp_pass = False

            if all_comp_pass:
                any_passed = True
                best_result = (block, conn_key, actual, comp_details)
                break
            elif best_result is None:
                best_result = (block, conn_key, actual, comp_details)

        block, conn_key, actual, comp_details = best_result
        for comp_op, comp_val, passed in comp_details:
            checks.append({
                "output": spec_key,
                "block_title": block.title,
                "block_type": block.btype,
                "connector": conn_key,
                "actual": actual,
                "comparator": comp_op,
                "expected": comp_val,
                "status": "pass" if any_passed else "fail",
                "candidates_checked": len(candidates),
            })

    return checks


# ── Main runner ───────────────────────────────────────────────────────

def load_eval_cases(cases_dir: str) -> Dict[str, Dict]:
    """Load all eval cases with simulation specs, keyed by case_id."""
    cases = {}
    for fpath in sorted(glob.glob(os.path.join(cases_dir, "*.json"))):
        with open(fpath) as f:
            try:
                data = json.load(f)
            except json.JSONDecodeError:
                continue
        for case in data:
            sim = case.get("expected", {}).get("simulation", [])
            if sim:
                cases[case["id"]] = case
    return cases


def load_llm_report(report_path: str) -> Dict[str, Dict]:
    """Load LLM report, keyed by case_id."""
    if not os.path.exists(report_path):
        return {}
    with open(report_path) as f:
        data = json.load(f)
    return {c["case_id"]: c for c in data.get("cases", [])}


def run_simulation(
    case_id: str,
    case: Dict,
    config_path: str,
) -> Dict[str, Any]:
    """Run all simulation scenarios for a case. Returns result dict."""
    try:
        graph = parse_config(config_path)
    except Exception as e:
        return {
            "case_id": case_id,
            "config": os.path.basename(config_path),
            "error": f"XML parse error: {e}",
            "scenarios": [],
            "pass": False,
            "passed_count": 0,
            "total_count": 0,
        }

    sims = case.get("expected", {}).get("simulation", [])
    scenario_results = []
    all_passed = True

    for sim in sims:
        sim_name = sim.get("name", "unnamed")
        inputs = sim.get("inputs", {})
        ticks = sim.get("ticks", 10)
        dt_val = sim.get("dt", 0.1)
        expected_outputs = sim.get("expected_outputs", {})

        # Resolve input specs to block UUIDs
        injected: Dict[str, Dict[str, float]] = {}
        unresolved_inputs = []
        for spec_key, val in inputs.items():
            resolved = resolve_block_connector(graph, spec_key)
            if resolved:
                for block, conn_key in resolved:
                    injected.setdefault(block.uuid, {})[conn_key] = val
            else:
                unresolved_inputs.append(spec_key)

        if unresolved_inputs:
            scenario_results.append({
                "name": sim_name,
                "status": "input_not_found",
                "unresolved_inputs": unresolved_inputs,
                "checks": [],
            })
            all_passed = False
            continue

        # Propagate
        sim_results = propagate(graph, injected, ticks=ticks, dt=dt_val)

        # Check outputs
        checks = check_outputs(graph, sim_results, expected_outputs)

        scenario_pass = all(
            c.get("status") == "pass" for c in checks
        ) and len(checks) > 0

        if not scenario_pass:
            all_passed = False

        scenario_results.append({
            "name": sim_name,
            "status": "pass" if scenario_pass else "fail",
            "checks": checks,
        })

    passed_count = sum(1 for s in scenario_results if s["status"] == "pass")
    return {
        "case_id": case_id,
        "config": os.path.basename(config_path),
        "scenarios": scenario_results,
        "pass": all_passed,
        "passed_count": passed_count,
        "total_count": len(scenario_results),
    }


def main():
    parser = argparse.ArgumentParser(
        description="Simulation validation for Loxone eval configs"
    )
    parser.add_argument(
        "--configs-dir", default="/tmp/eval-llm",
        help="Directory with agent-built .Loxone configs",
    )
    parser.add_argument(
        "--cases-dir", default="tests/eval/cases",
        help="Directory with eval case JSON files",
    )
    parser.add_argument(
        "--llm-report", default="tests/eval/reports/llm-report.json",
        help="Path to LLM eval report",
    )
    parser.add_argument(
        "--output", default="tests/eval/reports/sim-validation-report.json",
        help="Output report path",
    )
    args = parser.parse_args()

    # Load data
    print("Loading eval cases...")
    cases = load_eval_cases(args.cases_dir)
    print(f"  Found {len(cases)} cases with simulation specs")

    print("Loading LLM report...")
    llm_report = load_llm_report(args.llm_report)
    print(f"  Found {len(llm_report)} cases in LLM report")

    # Run simulations
    results = []
    matched = 0
    no_config = []

    for case_id, case in sorted(cases.items()):
        config_path = os.path.join(args.configs_dir, f"{case_id}.Loxone")
        if not os.path.exists(config_path):
            no_config.append(case_id)
            continue
        matched += 1
        result = run_simulation(case_id, case, config_path)

        # Cross-reference with LLM report
        llm_entry = llm_report.get(case_id, {})
        result["structural_pass"] = llm_entry.get("pass")
        result["structural_score"] = llm_entry.get("overall_score")
        result["block_f1"] = llm_entry.get("block_f1")
        result["wiring_accuracy"] = llm_entry.get("wiring_accuracy")
        result["trace_score"] = llm_entry.get("trace_score")
        result["category"] = llm_entry.get("category", case.get("category", ""))
        result["difficulty"] = llm_entry.get("difficulty", case.get("difficulty", ""))

        results.append(result)

    # Analyze
    total_scenarios = sum(r["total_count"] for r in results)
    passed_scenarios = sum(r["passed_count"] for r in results)
    cases_passed = sum(1 for r in results if r["pass"])
    cases_failed = sum(1 for r in results if not r["pass"])

    # Cross-reference analysis
    struct_pass_sim_fail = [
        r for r in results
        if r.get("structural_pass") is True and not r["pass"]
    ]
    struct_fail_sim_pass = [
        r for r in results
        if r.get("structural_pass") is False and r["pass"]
    ]
    both_pass = [
        r for r in results
        if r.get("structural_pass") is True and r["pass"]
    ]
    both_fail = [
        r for r in results
        if r.get("structural_pass") is False and not r["pass"]
    ]

    # Build report
    report = {
        "summary": {
            "total_cases_with_sim_specs": len(cases),
            "cases_with_matching_configs": matched,
            "cases_without_configs": len(no_config),
            "missing_config_ids": no_config,
            "total_scenarios": total_scenarios,
            "passed_scenarios": passed_scenarios,
            "failed_scenarios": total_scenarios - passed_scenarios,
            "scenario_pass_rate": (
                round(passed_scenarios / total_scenarios, 4)
                if total_scenarios > 0 else 0
            ),
            "cases_all_pass": cases_passed,
            "cases_any_fail": cases_failed,
            "case_pass_rate": (
                round(cases_passed / matched, 4)
                if matched > 0 else 0
            ),
        },
        "cross_reference": {
            "structural_pass_sim_fail": {
                "count": len(struct_pass_sim_fail),
                "description": (
                    "LLM judge says PASS but simulation FAILS — "
                    "agent built wrong circuit (false positives)"
                ),
                "cases": [
                    {
                        "case_id": r["case_id"],
                        "structural_score": r.get("structural_score"),
                        "wiring_accuracy": r.get("wiring_accuracy"),
                        "trace_score": r.get("trace_score"),
                        "sim_details": [
                            {
                                "name": s["name"],
                                "status": s["status"],
                                "checks": s.get("checks", []),
                            }
                            for s in r["scenarios"]
                            if s["status"] != "pass"
                        ],
                    }
                    for r in struct_pass_sim_fail
                ],
            },
            "structural_fail_sim_pass": {
                "count": len(struct_fail_sim_pass),
                "description": (
                    "LLM judge says FAIL but simulation PASSES — "
                    "eval too strict, agent's alternative works"
                ),
                "cases": [
                    {
                        "case_id": r["case_id"],
                        "structural_score": r.get("structural_score"),
                        "block_f1": r.get("block_f1"),
                        "wiring_accuracy": r.get("wiring_accuracy"),
                    }
                    for r in struct_fail_sim_pass
                ],
            },
            "both_pass": {
                "count": len(both_pass),
                "description": "Both structural and simulation agree: PASS",
            },
            "both_fail": {
                "count": len(both_fail),
                "description": "Both structural and simulation agree: FAIL",
            },
        },
        "cases": results,
    }

    # Write report
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w") as f:
        json.dump(report, f, indent=2, ensure_ascii=False)

    # Print summary
    print("\n" + "=" * 60)
    print("SIMULATION VALIDATION REPORT")
    print("=" * 60)
    print(
        f"Scenarios: {passed_scenarios}/{total_scenarios} passed "
        f"({report['summary']['scenario_pass_rate']:.1%})"
    )
    print(
        f"Cases:     {cases_passed}/{matched} all-pass "
        f"({report['summary']['case_pass_rate']:.1%})"
    )
    print(f"No config: {len(no_config)} cases skipped")

    print("\n── Cross-Reference with LLM Structural Eval ──")
    print(
        f"  Structural PASS + Sim FAIL (false positives): "
        f"{len(struct_pass_sim_fail)}"
    )
    for r in struct_pass_sim_fail:
        failed = [s for s in r["scenarios"] if s["status"] != "pass"]
        print(
            f"    {r['case_id']}: structural={r.get('structural_score'):.3f}, "
            f"{len(failed)} scenario(s) failed"
        )
        for s in failed:
            detail = ""
            for c in s.get("checks", []):
                if c.get("status") != "pass":
                    detail = (
                        f" ({c.get('output','?')}: "
                        f"actual={c.get('actual','?')} "
                        f"{c.get('comparator','?')} "
                        f"{c.get('expected','?')})"
                    )
                    break
            print(f"      ✗ {s['name']}{detail}")

    print(
        f"\n  Structural FAIL + Sim PASS (eval too strict): "
        f"{len(struct_fail_sim_pass)}"
    )
    for r in struct_fail_sim_pass:
        print(
            f"    {r['case_id']}: structural={r.get('structural_score'):.3f}, "
            f"block_f1={r.get('block_f1'):.3f}, "
            f"wiring={r.get('wiring_accuracy'):.3f}"
        )

    print(
        f"\n  Both PASS: {len(both_pass)}, Both FAIL: {len(both_fail)}"
    )

    print(f"\nReport saved to: {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
