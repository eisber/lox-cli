#!/usr/bin/env python3
"""Signal-flow tracer for the Loxone eval harness.

Uses BFS over the XML wiring graph to verify that expected signal paths
exist in a generated config.  Can be imported as a module or run standalone:

    python3 sim_check.py <config.Loxone> '<wiring-json>'

Returns a trace_score in [0, 1]: reachable / total expected wiring.
"""

from __future__ import annotations

import json
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict, deque
from typing import Dict, List, Optional, Set, Tuple

# ── Infrastructure types excluded from control graph ──────────────────

INFRA_TYPES = frozenset({
    "Document", "Page", "Place", "Category", "Program",
    "User", "UserCaption", "LoxCaption", "VirtualInCaption",
    "LightscenesC", "LightsceneC", "Lightscene",
    "TreeDevice", "LoxAIRDevice", "NetworkDevice",
    "TreeAsensor", "TreeDsensor", "TreeAactuator", "TreeDactuator",
    "Co", "IoData", "Display", "HP", "Const", "Note", "SET",
})

TYPE_EQUIVALENTS = {
    "Memory": {"Memory", "AMemory"},
    "AMemory": {"Memory", "AMemory"},
    "GreaterEqual": {"GreaterEqual", "AnalogThresholdTrigger"},
    "AnalogThresholdTrigger": {"GreaterEqual", "AnalogThresholdTrigger"},
}


# ── Graph building ────────────────────────────────────────────────────

class BlockNode:
    """A control block in the wiring graph."""
    __slots__ = ("uuid", "title", "btype", "room_uuid",
                 "connectors")

    def __init__(self, uuid: str, title: str, btype: str,
                 room_uuid: str = ""):
        self.uuid = uuid
        self.title = title
        self.btype = btype
        self.room_uuid = room_uuid
        # connector_uuid → connector_key
        self.connectors: Dict[str, str] = {}


def build_block_graph(root: ET.Element):
    """Parse XML into blocks and a block-level directed adjacency list.

    Returns
    -------
    blocks : dict[uuid, BlockNode]
    adjacency : dict[src_block_uuid, set[dst_block_uuid]]
        An edge (A, B) means some output of A is wired to some input of B.
    conn_to_block : dict[connector_uuid, block_uuid]
    """
    blocks: Dict[str, BlockNode] = {}
    conn_to_block: Dict[str, str] = {}  # connector_uuid → block_uuid

    # Pass 1: collect blocks and their connectors
    def walk_blocks(elem: ET.Element, page_title: str = ""):
        etype = elem.get("Type", "")
        euuid = elem.get("U", "")
        title = elem.get("Title", "")

        if etype == "Page":
            page_title = title

        room_uuid = ""
        for child in elem:
            if child.tag == "IoData" or child.get("Type") == "IoData":
                room_uuid = child.get("Pr", "")

        if etype and etype not in INFRA_TYPES and euuid:
            node = BlockNode(euuid, title, etype, room_uuid)
            blocks[euuid] = node

            # Collect connectors
            for child in elem:
                if child.tag == "Co" or child.get("Type") == "Co":
                    cu = child.get("U", "")
                    ck = child.get("K", "")
                    if cu:
                        node.connectors[cu] = ck
                        conn_to_block[cu] = euuid

        for child in elem:
            walk_blocks(child, page_title)

    walk_blocks(root)

    # Pass 2: collect wiring edges (connector-level)
    # Two wiring mechanisms:
    #   a) <In Input="src_conn_uuid"> on a connector element
    #   b) UUID-sharing: same connector UUID owned by two different blocks
    adjacency: Dict[str, Set[str]] = defaultdict(set)

    def walk_wiring(elem: ET.Element):
        for child in elem:
            if child.tag == "Co" or child.get("Type") == "Co":
                dst_conn_uuid = child.get("U", "")
                dst_block = conn_to_block.get(dst_conn_uuid, "")

                for inp in child:
                    if inp.tag == "In":
                        src_conn_uuid = inp.get("Input", "")
                        if src_conn_uuid and dst_conn_uuid:
                            src_block = conn_to_block.get(src_conn_uuid, "")
                            if src_block and dst_block and src_block != dst_block:
                                adjacency[src_block].add(dst_block)
            walk_wiring(child)

    walk_wiring(root)

    # UUID-sharing wiring
    uuid_blocks: Dict[str, List[str]] = defaultdict(list)
    for cu, buuid in conn_to_block.items():
        uuid_blocks[cu].append(buuid)
    for cu, buuids in uuid_blocks.items():
        unique = list(set(buuids))
        if len(unique) >= 2:
            for a in unique:
                for b in unique:
                    if a != b:
                        adjacency[a].add(b)

    return blocks, adjacency, conn_to_block


# ── BFS tracer ────────────────────────────────────────────────────────

def _match_block(blocks: Dict[str, BlockNode], *,
                 title: str = "", btype: str = "",
                 room_uuid: str = "",
                 rooms: Optional[Dict[str, str]] = None,
                 room_name: str = "") -> List[str]:
    """Return UUIDs of blocks matching the selector."""
    matches = []
    for uuid, node in blocks.items():
        if btype:
            if not _type_matches(btype, node.btype):
                continue
        if title:
            base = title.split("[")[0].strip()
            if base not in node.title:
                continue
        if room_name and rooms:
            block_room = rooms.get(node.room_uuid, "")
            if block_room != room_name:
                continue
        if room_uuid and node.room_uuid != room_uuid:
            continue
        matches.append(uuid)
    return matches


def _type_matches(expected: str, actual: str) -> bool:
    if expected == actual:
        return True
    return actual in TYPE_EQUIVALENTS.get(expected, set())


def bfs_reachable(adjacency: Dict[str, Set[str]],
                  sources: List[str]) -> Set[str]:
    """Return all block UUIDs reachable from any source via BFS."""
    visited: Set[str] = set()
    queue = deque(sources)
    for s in sources:
        visited.add(s)
    while queue:
        cur = queue.popleft()
        for nbr in adjacency.get(cur, set()):
            if nbr not in visited:
                visited.add(nbr)
                queue.append(nbr)
    # Exclude starting nodes from "reachable" set
    return visited - set(sources)


def trace_signal(blocks: Dict[str, BlockNode],
                 adjacency: Dict[str, Set[str]],
                 src_selector: dict,
                 dst_selector: dict,
                 rooms: Optional[Dict[str, str]] = None) -> bool:
    """Check if ANY block matching src_selector can reach ANY block
    matching dst_selector through the wiring graph."""
    src_uuids = _match_block(
        blocks,
        title=src_selector.get("title", ""),
        btype=src_selector.get("type", ""),
        room_name=src_selector.get("room", ""),
        rooms=rooms,
    )
    if not src_uuids:
        return False

    dst_uuids = _match_block(
        blocks,
        title=dst_selector.get("title", ""),
        btype=dst_selector.get("type", ""),
        room_name=dst_selector.get("room", ""),
        rooms=rooms,
    )
    if not dst_uuids:
        return False

    reachable = bfs_reachable(adjacency, src_uuids)
    # Also include direct match (src IS dst for self-loops, though unusual)
    return bool(set(dst_uuids) & (reachable | set(src_uuids)
                                  if src_uuids == dst_uuids else reachable))


# ── Public API ────────────────────────────────────────────────────────

def collect_rooms(root: ET.Element) -> Dict[str, str]:
    """uuid → room name"""
    rooms: Dict[str, str] = {}
    for elem in root.iter("C"):
        if elem.get("Type") == "Place":
            rooms[elem.get("U", "")] = elem.get("Title", "")
    return rooms


def check_signal_flow(config_path: str,
                      expected_wiring: List[dict]) -> Tuple[float, List[dict]]:
    """Compute trace_score for a config against expected wiring.

    Returns (score, details) where score ∈ [0, 1] and details is a list
    of per-wire results.
    """
    tree = ET.parse(config_path)
    root = tree.getroot()

    rooms = collect_rooms(root)
    blocks, adjacency, _ = build_block_graph(root)

    results = []
    for wire in expected_wiring:
        src_title = wire.get("from_title", "")
        src_type = wire.get("from_type", "")
        src_room = wire.get("from_room", "")
        dst_title = wire.get("to_title", "")
        dst_type = wire.get("to_type", "")
        dst_room = wire.get("to_room", "")

        src_sel = {}
        if src_title:
            src_sel["title"] = src_title
        if src_type:
            src_sel["type"] = src_type
        if src_room:
            src_sel["room"] = src_room

        dst_sel = {}
        if dst_title:
            dst_sel["title"] = dst_title
        if dst_type:
            dst_sel["type"] = dst_type
        if dst_room:
            dst_sel["room"] = dst_room

        if not src_sel or not dst_sel:
            # Can't trace without both endpoints
            continue

        found = trace_signal(blocks, adjacency, src_sel, dst_sel, rooms)

        label = " → ".join(filter(None, [
            src_title or src_type, dst_title or dst_type]))
        results.append({"src": src_title or src_type,
                        "dst": dst_title or dst_type,
                        "label": label,
                        "found": found})

    if not results:
        return 1.0, []
    score = sum(1 for r in results if r["found"]) / len(results)
    return score, results


# ── CLI entry point ───────────────────────────────────────────────────

def main():
    if len(sys.argv) < 3:
        print("Usage: sim_check.py <config.Loxone> '<wiring-json>'",
              file=sys.stderr)
        sys.exit(2)

    config_path = sys.argv[1]
    wiring = json.loads(sys.argv[2])

    score, details = check_signal_flow(config_path, wiring)
    out = {"trace_score": round(score, 3), "details": details}
    print(json.dumps(out, indent=2, ensure_ascii=False))
    sys.exit(0 if score >= 0.8 else 1)


if __name__ == "__main__":
    main()
