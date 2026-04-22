#!/usr/bin/env python3
"""Generate 150 Reference-inspired eval cases for the Loxone automation eval harness.

Patterns extracted from Reference.Loxone (1776 items, 51 rooms, 1807 wires):
- 13x PushButton → HeatIRoomController2.Reset (fan-out heating reset)
- 12x OnPulseDelay → EIBJalousie.InputTriggerUp (delayed blind up)
-  8x OnPulseDelay → EIBJalousie.InputTriggerDown (delayed blind down)
-  4x DayTimer → LightController2.Reset (timed light off)
-  7x Monoflop for door unlock pulses (3s)
- 10x And gates for combined conditions
-  3x GreaterEqual for thresholds
-  3x StairwayLS for timed lights
- 36x Memory for state latching
- 17x DayTimer for scheduling
- 20x OnPulseDelay for delayed actions
"""

import json
import random

random.seed(42)

# Fixture rooms and their controls
ROOMS = {
    "Wohnzimmer": {
        "lc2": "Lichtsteuerung",
        "blinds": ["Jalousie 1", "Jalousie 2"],
        "heater": "Raumregler",
        "ac": "Klimaanlage",
    },
    "Schlafzimmer": {
        "lc2": "Lichtsteuerung",
        "blinds": ["Jalousie 1", "Jalousie 2"],
    },
    "Küche": {
        "lc2": "Lichtsteuerung",
        "blinds": ["Jalousie 1", "Jalousie 2"],
    },
    "Bad": {
        "lc2": "Lichtsteuerung",
        "blinds": ["Jalousie 1", "Jalousie 2"],
        "fan": "Lüfter Bad",
    },
    "Flur": {
        "lc2": "Lichtsteuerung",
        "blinds": ["Jalousie 1", "Jalousie 2"],
        "motion": "Bewegungsmelder",
        "door": "Türkontakt Eingang",
    },
    "Garten": {
        "lc2": "Gartenbeleuchtung",
        "irrigation": "Bewässerungsventil",
        "motion": "Bewegungsmelder Garten",
    },
    "Garage": {
        "lc2": "Garagenlicht",
        "gate": "Garagentor",
    },
}

WEATHER = {
    "Außentemperatur": {"connector": "AQ", "unit": "°C"},
    "Sonnenschein": {"connector": "AQ", "unit": ""},
    "Windgeschwindigkeit": {"connector": "AQ", "unit": "km/h"},
    "Regen": {"connector": "AQ", "unit": ""},
    "Luftfeuchtigkeit": {"connector": "AQ", "unit": "%"},
    "Helligkeit": {"connector": "AQ", "unit": "lux"},
}

VIS = {
    "Schalter 1": {"connector": "Q", "analog": False},
    "Schalter 2": {"connector": "Q", "analog": False},
    "Bewässerung Manuell": {"connector": "Q", "analog": False},
    "Feuchtesensor Garten": {"connector": "AQ", "analog": True},
    "CO2 Sensor": {"connector": "AQ", "analog": True},
    "Garagentor Sensor": {"connector": "Q", "analog": False},
}

cases = []
case_id = 0

def next_id(prefix):
    global case_id
    case_id += 1
    return f"k{case_id:03d}-{prefix}"


# ============================================================
# Pattern A: Single Threshold (GreaterEqual/Less → Actuator)
# From Reference: 3x GreaterEqual for temp/wind/irrigation thresholds
# ============================================================

threshold_combos = [
    # (sensor, threshold, unit, action_desc, room, target, connector, comparator)
    ("Außentemperatur", "30", "°C", "close {blind} in {room} when temperature exceeds 30°C", "Wohnzimmer", "Jalousie 1", "InputTriggerDown", "GreaterEqual"),
    ("Außentemperatur", "35", "°C", "open {blind} in {room} when it gets above 35 degrees for ventilation", "Küche", "Jalousie 1", "InputTriggerUp", "GreaterEqual"),
    ("Windgeschwindigkeit", "50", "km/h", "raise {blind} in {room} during strong winds above 50 km/h", "Schlafzimmer", "Jalousie 1", "InputTriggerUp", "GreaterEqual"),
    ("Windgeschwindigkeit", "60", "km/h", "protect {blind} in {room} when wind exceeds 60 km/h", "Küche", "Jalousie 2", "InputTriggerUp", "GreaterEqual"),
    ("Außentemperatur", "-5", "°C", "open {blind} in {room} when temperature drops below -5°C to prevent ice damage", "Wohnzimmer", "Jalousie 2", "InputTriggerUp", "Less"),
    ("Außentemperatur", "2", "°C", "raise {blind} in {room} below 2 degrees for frost protection", "Schlafzimmer", "Jalousie 2", "InputTriggerUp", "Less"),
    ("Helligkeit", "50000", "lux", "close {blind} in {room} when it's very bright outside", "Wohnzimmer", "Jalousie 1", "InputTriggerDown", "GreaterEqual"),
    ("Helligkeit", "30000", "lux", "lower {blind} in {room} when brightness exceeds 30000 lux", "Schlafzimmer", "Jalousie 1", "InputTriggerDown", "GreaterEqual"),
    ("Luftfeuchtigkeit", "80", "%", "activate the bathroom fan when humidity is above 80%", "Bad", None, None, "GreaterEqual"),
    ("Luftfeuchtigkeit", "75", "%", "turn on {fan} when humidity exceeds 75%", "Bad", None, None, "GreaterEqual"),
    ("Außentemperatur", "15", "°C", "close the {blind} in {room} when it's cold outside (below 15°C)", "Küche", "Jalousie 1", "InputTriggerDown", "Less"),
    ("Windgeschwindigkeit", "30", "km/h", "raise all blinds in {room} when wind speed exceeds 30 km/h", "Wohnzimmer", None, None, "GreaterEqual"),
    ("Helligkeit", "100", "lux", "turn on garden lights when it gets dark (brightness below 100 lux)", "Garten", None, None, "Less"),
    ("Außentemperatur", "28", "°C", "turn on the AC in the living room when it's above 28 degrees", "Wohnzimmer", None, None, "GreaterEqual"),
]

for sensor, threshold, unit, utterance_tpl, room, target, connector, comparator in threshold_combos:
    room_data = ROOMS[room]
    blind = target or (room_data.get("blinds", [None])[0])
    utterance = utterance_tpl.format(
        blind=blind or "blinds",
        room=room,
        fan=room_data.get("fan", "fan"),
    )

    case = {
        "id": next_id("threshold"),
        "pattern": ["threshold"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [{"type": comparator}],
            "wiring": [
                {"from_title": sensor, "to_type": comparator},
            ],
            "params": [
                {"block_type": comparator, "param": "Input2" if comparator == "GreaterEqual" else "Input2", "value": threshold}
            ],
        },
    }

    # Add actuator wiring based on target
    if target and connector:
        case["expected"]["wiring"].append(
            {"from_type": comparator, "to_title": target, "to_room": room, "to_connector": connector}
        )
    elif room == "Bad" and "fan" in utterance.lower():
        case["expected"]["wiring"].append(
            {"from_type": comparator, "to_title": "Lüfter Bad"}
        )
    elif room == "Garten" and "light" in utterance.lower():
        case["expected"]["wiring"].append(
            {"from_type": comparator, "to_title": "Gartenbeleuchtung"}
        )
    elif room == "Wohnzimmer" and "ac" in utterance.lower():
        case["expected"]["wiring"].append(
            {"from_type": comparator, "to_title": "Klimaanlage", "to_connector": "toggle"}
        )
    elif "all blinds" in utterance.lower():
        for b in room_data.get("blinds", []):
            case["expected"]["wiring"].append(
                {"from_type": comparator, "to_title": b, "to_room": room, "to_connector": "InputTriggerUp"}
            )
        case["difficulty"] = "medium"
        case["pattern"].append("fan-out")

    cases.append(case)


# ============================================================
# Pattern B: Combined Conditions (Sensor + Sensor → And → Actuator)
# From Reference: 10x And gates, most with 2 InputRef sources
# ============================================================

and_combos = [
    # (sensor1, sensor2, s2_threshold, utterance, room, target, connector, difficulty)
    ("Sonnenschein", "Außentemperatur", "25", "Close {blind} in {room} when it's sunny AND above 25°C", "Wohnzimmer", "Jalousie 1", "InputTriggerDown", "medium"),
    ("Sonnenschein", "Außentemperatur", "22", "Lower {blind} in {room} when sunny and warm (above 22°C)", "Schlafzimmer", "Jalousie 1", "InputTriggerDown", "medium"),
    ("Sonnenschein", "Außentemperatur", "20", "Protect {room} furniture: close {blind} when sunny and above 20 degrees", "Küche", "Jalousie 1", "InputTriggerDown", "medium"),
    ("Sonnenschein", "Außentemperatur", "18", "Lower blinds in {room} when sunny and temperature exceeds 18°C", "Wohnzimmer", "Jalousie 2", "InputTriggerDown", "medium"),
    ("Sonnenschein", "Helligkeit", "40000", "Close {blind} in {room} when it's sunny and very bright", "Schlafzimmer", "Jalousie 2", "InputTriggerDown", "medium"),
    ("Regen", "Windgeschwindigkeit", "40", "Raise {blind} in {room} during rain with wind above 40 km/h", "Wohnzimmer", "Jalousie 1", "InputTriggerUp", "medium"),
    ("Regen", "Windgeschwindigkeit", "35", "Protect {blind} in {room}: raise during rainy and windy conditions (>35 km/h)", "Küche", "Jalousie 2", "InputTriggerUp", "medium"),
    ("Sonnenschein", "Außentemperatur", "30", "Turn on {room} AC when it's sunny and above 30 degrees", "Wohnzimmer", "Klimaanlage", "toggle", "medium"),
]

for sensor1, sensor2, threshold, utterance_tpl, room, target, connector, difficulty in and_combos:
    room_data = ROOMS[room]
    blind = room_data.get("blinds", ["blinds"])[0]
    utterance = utterance_tpl.format(blind=target if target != "Klimaanlage" else "AC", room=room)

    case = {
        "id": next_id("combined"),
        "pattern": ["combined-condition", "threshold"],
        "utterance": utterance,
        "difficulty": difficulty,
        "expected": {
            "new_blocks": [
                {"type": "GreaterEqual"},
                {"type": "And"},
            ],
            "wiring": [
                {"from_title": sensor1, "to_type": "And"},
                {"from_title": sensor2, "to_type": "GreaterEqual"},
                {"from_type": "GreaterEqual", "to_type": "And"},
                {"from_type": "And", "to_title": target, "to_room": room, "to_connector": connector},
            ],
            "params": [
                {"block_type": "GreaterEqual", "param": "Input2", "value": threshold},
            ],
        },
    }
    cases.append(case)


# ============================================================
# Pattern C: Negation (Sensor + Not + And → Actuator)
# From Reference: Observed in irrigation (water only when NOT raining)
# ============================================================

not_combos = [
    ("Sonnenschein", "Regen", "Close {blind} in {room} when sunny but NOT when it's raining", "Wohnzimmer", "Jalousie 1", "InputTriggerDown"),
    ("Sonnenschein", "Regen", "Lower {blind} in {room} when there's sunshine, unless it rains", "Küche", "Jalousie 1", "InputTriggerDown"),
    ("Sonnenschein", "Regen", "Shade {room}: close {blind} when sunny and dry (not raining)", "Schlafzimmer", "Jalousie 2", "InputTriggerDown"),
    ("Schalter 1", "Regen", "Water the garden when switch is on, but not when it's raining", "Garten", "Bewässerungsventil", "I1"),
    ("Bewässerung Manuell", "Regen", "Start irrigation when manual switch is triggered, skip if raining", "Garten", "Bewässerungsventil", "I1"),
]

for sensor, negated, utterance_tpl, room, target, connector in not_combos:
    room_data = ROOMS[room]
    blind = room_data.get("blinds", ["blinds"])[0] if "blind" in utterance_tpl else target
    utterance = utterance_tpl.format(blind=blind, room=room)

    case = {
        "id": next_id("negation"),
        "pattern": ["negation", "combined-condition"],
        "utterance": utterance,
        "difficulty": "hard",
        "expected": {
            "new_blocks": [
                {"type": "Not"},
                {"type": "And"},
            ],
            "wiring": [
                {"from_title": sensor, "to_type": "And"},
                {"from_title": negated, "to_type": "Not"},
                {"from_type": "Not", "to_type": "And"},
                {"from_type": "And", "to_title": target, "to_room": room, "to_connector": connector},
            ],
            "params": [],
        },
    }
    cases.append(case)


# ============================================================
# Pattern D: Delayed Action (OnPulseDelay)
# From Reference: 20x OnPulseDelay, mostly for blind timing
# ============================================================

delay_combos = [
    ("Lichtsteuerung", "Bad", "Lüfter Bad", "Bad", 600, "Run bathroom fan for 10 minutes after light turns off"),
    ("Lichtsteuerung", "Bad", "Lüfter Bad", "Bad", 900, "Keep bathroom ventilation running 15 minutes after light off"),
    ("Lichtsteuerung", "Flur", "Lichtsteuerung", "Flur", 180, "Keep hallway light on for 3 minutes after motion stops"),
    ("Garagentor Sensor", "Garage", "Garagenlicht", "Garage", 300, "Turn on garage light for 5 minutes when gate opens"),
    ("Bewegungsmelder Garten", "Garten", "Gartenbeleuchtung", "Garten", 120, "Keep garden light on for 2 minutes after motion detected"),
    ("Türkontakt Eingang", "Flur", "Lichtsteuerung", "Flur", 60, "Turn on hallway light for 1 minute when front door opens"),
    ("Bewegungsmelder", "Flur", "Lichtsteuerung", "Flur", 240, "Leave hallway light on for 4 minutes after last motion"),
    ("Schalter 1", "Wohnzimmer", "Klimaanlage", "Wohnzimmer", 1800, "Run AC for 30 minutes when button pressed"),
]

for source, source_room, target, target_room, delay_s, utterance in delay_combos:
    case = {
        "id": next_id("delay"),
        "pattern": ["delayed-action"],
        "utterance": utterance,
        "difficulty": "medium",
        "expected": {
            "new_blocks": [{"type": "OnPulseDelay"}],
            "wiring": [
                {"from_title": source, "to_type": "OnPulseDelay"},
                {"from_type": "OnPulseDelay", "to_title": target, "to_room": target_room},
            ],
            "params": [
                {"block_type": "OnPulseDelay", "param": "T", "value": str(delay_s)},
            ],
        },
    }
    cases.append(case)


# ============================================================
# Pattern E: Timed Light (StairwayLS)
# From Reference: 3x StairwayLS for timed lighting
# ============================================================

stairway_combos = [
    ("Bewegungsmelder", "Flur", "Lichtsteuerung", "Flur", 120, "Hallway light on for 2 minutes when motion detected"),
    ("Bewegungsmelder Garten", "Garten", "Gartenbeleuchtung", "Garten", 300, "Garden light on for 5 minutes on motion"),
    ("Garagentor Sensor", "Garage", "Garagenlicht", "Garage", 180, "Garage light stays on 3 minutes after door opens"),
    ("Türkontakt Eingang", "Flur", "Lichtsteuerung", "Flur", 90, "Front door opens: hallway light for 90 seconds"),
    ("Schalter 2", "Wohnzimmer", "Lichtsteuerung", "Wohnzimmer", 600, "Press switch, living room light stays on for 10 minutes"),
]

for source, source_room, target, target_room, timer_s, utterance in stairway_combos:
    case = {
        "id": next_id("stairway"),
        "pattern": ["timed-switch"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [{"type": "StairwayLS"}],
            "wiring": [
                {"from_title": source, "to_type": "StairwayLS"},
                {"from_type": "StairwayLS", "to_title": target, "to_room": target_room},
            ],
            "params": [
                {"block_type": "StairwayLS", "param": "T", "value": str(timer_s)},
            ],
        },
    }
    cases.append(case)


# ============================================================
# Pattern F: Direct Wiring (Sensor → Actuator, no logic)
# From Reference: 207x sensor → actuator connections
# ============================================================

direct_combos = [
    ("Bewegungsmelder", "Flur", "OutputPresence", "Lichtsteuerung", "Flur", "Presence", "Connect motion detector to hallway light"),
    ("Bewegungsmelder Garten", "Garten", "OutputPresence", "Gartenbeleuchtung", "Garten", "Presence", "Link garden motion sensor to garden lights"),
    ("Außentemperatur", None, "AQ", "Raumregler", "Wohnzimmer", "Temp", "Feed outside temperature to living room thermostat"),
    ("Sonnenschein", None, "AQ", "Jalousie 1", "Wohnzimmer", "InputTriggerDown", "Lower living room blind when sunshine detected"),
    ("Regen", None, "AQ", "Jalousie 1", "Schlafzimmer", "InputTriggerDown", "Close bedroom blind when it rains"),
    ("Regen", None, "AQ", "Jalousie 2", "Schlafzimmer", "InputTriggerDown", "Close second bedroom blind on rain"),
    ("Regen", None, "AQ", "Jalousie 1", "Küche", "InputTriggerDown", "Shut kitchen blind when rain starts"),
    ("Schalter 1", None, "Q", "Jalousie 1", "Wohnzimmer", "InputDisable", "Use switch 1 to disable automatic blind control in living room"),
    ("Schalter 2", None, "Q", "Jalousie 2", "Wohnzimmer", "InputDisable", "Use switch 2 to override living room blind 2"),
    ("Schalter 1", None, "Q", "Lichtsteuerung", "Wohnzimmer", "I1", "Toggle living room light with switch 1"),
    ("Schalter 2", None, "Q", "Lichtsteuerung", "Schlafzimmer", "I1", "Toggle bedroom light with switch 2"),
    ("Garagentor Sensor", None, "Q", "Garagenlicht", "Garage", "I1", "Turn on garage light when door opens"),
    ("Türkontakt Eingang", None, "Q", "Lichtsteuerung", "Flur", "I1", "Turn on hallway light when front door opens"),
    ("CO2 Sensor", None, "AQ", "Klimaanlage", "Wohnzimmer", "inTempCurr", "Feed CO2 level to AC system"),
]

for source, source_room, src_conn, target, target_room, dst_conn, utterance in direct_combos:
    case = {
        "id": next_id("direct"),
        "pattern": ["direct-wiring"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [],
            "wiring": [
                {"from_title": source, "from_connector": src_conn, "to_title": target, "to_room": target_room, "to_connector": dst_conn},
            ],
            "params": [],
        },
    }
    cases.append(case)


# ============================================================
# Pattern G: Fan-Out (One signal → Multiple targets)
# From Reference: PushButton "Heizung" → 13x HeatIRoomController2.Reset
# ============================================================

fanout_combos = [
    # Wind protection: close ALL blinds in multiple rooms
    {
        "utterance": "When wind exceeds 45 km/h, raise ALL blinds in living room and bedroom for protection",
        "sensor": "Windgeschwindigkeit",
        "threshold": "45",
        "targets": [
            ("Jalousie 1", "Wohnzimmer", "InputTriggerUp"),
            ("Jalousie 2", "Wohnzimmer", "InputTriggerUp"),
            ("Jalousie 1", "Schlafzimmer", "InputTriggerUp"),
            ("Jalousie 2", "Schlafzimmer", "InputTriggerUp"),
        ],
    },
    {
        "utterance": "Rain protection: close all blinds across the entire house",
        "sensor": "Regen",
        "threshold": None,
        "targets": [
            ("Jalousie 1", "Wohnzimmer", "InputTriggerDown"),
            ("Jalousie 2", "Wohnzimmer", "InputTriggerDown"),
            ("Jalousie 1", "Schlafzimmer", "InputTriggerDown"),
            ("Jalousie 2", "Schlafzimmer", "InputTriggerDown"),
            ("Jalousie 1", "Küche", "InputTriggerDown"),
            ("Jalousie 2", "Küche", "InputTriggerDown"),
        ],
    },
    {
        "utterance": "When it gets dark (brightness < 50 lux), close blinds in bedroom and kitchen",
        "sensor": "Helligkeit",
        "threshold": "50",
        "comparator": "Less",
        "targets": [
            ("Jalousie 1", "Schlafzimmer", "InputTriggerDown"),
            ("Jalousie 2", "Schlafzimmer", "InputTriggerDown"),
            ("Jalousie 1", "Küche", "InputTriggerDown"),
            ("Jalousie 2", "Küche", "InputTriggerDown"),
        ],
    },
    {
        "utterance": "Disable automatic blind control in all rooms with one switch",
        "sensor": "Schalter 1",
        "threshold": None,
        "targets": [
            ("Jalousie 1", "Wohnzimmer", "InputDisable"),
            ("Jalousie 2", "Wohnzimmer", "InputDisable"),
            ("Jalousie 1", "Schlafzimmer", "InputDisable"),
            ("Jalousie 2", "Schlafzimmer", "InputDisable"),
        ],
    },
]

for combo in fanout_combos:
    comparator = combo.get("comparator", "GreaterEqual")
    has_threshold = combo["threshold"] is not None

    wiring = []
    blocks = []
    params = []

    if has_threshold:
        blocks.append({"type": comparator})
        wiring.append({"from_title": combo["sensor"], "to_type": comparator})
        params.append({"block_type": comparator, "param": "Input2", "value": combo["threshold"]})
        for target, room, conn in combo["targets"]:
            wiring.append({"from_type": comparator, "to_title": target, "to_room": room, "to_connector": conn})
    else:
        for target, room, conn in combo["targets"]:
            wiring.append({"from_title": combo["sensor"], "to_title": target, "to_room": room, "to_connector": conn})

    case = {
        "id": next_id("fanout"),
        "pattern": ["fan-out"] + (["threshold"] if has_threshold else []),
        "utterance": combo["utterance"],
        "difficulty": "hard" if len(combo["targets"]) > 4 else "medium",
        "expected": {
            "new_blocks": blocks,
            "wiring": wiring,
            "params": params,
        },
    }
    cases.append(case)


# ============================================================
# Pattern H: Time Window + Action
# From Reference: DayTimers for scheduled actions
# ============================================================

time_combos = [
    ("22:00", "06:00", 1320, 360, "Dim bedroom light to 20% between 10pm and 6am", "Schlafzimmer", "Brightness", 20),
    ("23:00", "05:00", 1380, 300, "Set hallway light to 15% between 11pm and 5am", "Flur", "Brightness", 15),
    ("21:00", "07:00", 1260, 420, "Dim living room light to 40% in the evening (9pm to 7am)", "Wohnzimmer", "Brightness", 40),
    ("00:00", "06:00", 0, 360, "Set bathroom light to 10% between midnight and 6am", "Bad", "Brightness", 10),
    ("20:00", "08:00", 1200, 480, "Night mode: dim kitchen light to 25% from 8pm to 8am", "Küche", "Brightness", 25),
    ("22:00", "05:00", 1320, 300, "Close all blinds in bedroom between 10pm and 5am", "Schlafzimmer", None, None),
    ("23:00", "06:00", 1380, 360, "Raise blinds in living room between 11pm and 6am", "Wohnzimmer", None, None),
]

for time_from, time_to, minutes_from, minutes_to, utterance, room, target_conn, dim_value in time_combos:
    blocks = [
        {"type": "GreaterEqual"},
        {"type": "Less"},
    ]
    params = [
        {"block_type": "GreaterEqual", "param": "Input2", "value": str(minutes_from)},
        {"block_type": "Less", "param": "Input2", "value": str(minutes_to)},
    ]

    if target_conn == "Brightness" and dim_value is not None:
        # Time window → Mult → LC2.Brightness
        blocks.append({"type": "Or"})
        blocks.append({"type": "Mult"})
        params.append({"block_type": "Mult", "param": "Input2", "value": str(dim_value)})
        wiring = [
            {"from_type": "GreaterEqual", "to_type": "Or"},
            {"from_type": "Less", "to_type": "Or"},
            {"from_type": "Or", "to_type": "Mult"},
            {"from_type": "Mult", "to_title": "Lichtsteuerung", "to_room": room, "to_connector": "Brightness"},
        ]
        difficulty = "hard"
    else:
        # Time window → Or → Jalousie
        blocks.append({"type": "Or"})
        target_blind = ROOMS[room].get("blinds", [None])[0]
        conn = "InputTriggerDown" if "close" in utterance.lower() else "InputTriggerUp"
        wiring = [
            {"from_type": "GreaterEqual", "to_type": "Or"},
            {"from_type": "Less", "to_type": "Or"},
            {"from_type": "Or", "to_title": target_blind, "to_room": room, "to_connector": conn},
        ]
        difficulty = "medium"

    case = {
        "id": next_id("timewindow"),
        "pattern": ["time-window"] + (["dimming"] if dim_value else []),
        "utterance": utterance,
        "difficulty": difficulty,
        "expected": {
            "new_blocks": blocks,
            "wiring": wiring,
            "params": params,
        },
    }
    cases.append(case)


# ============================================================
# Pattern I: Combined + Negation + Fan-Out (Expert)
# From Reference: complex irrigation / pool control chains
# ============================================================

expert_combos = [
    {
        "id_prefix": "expert-shade",
        "utterance": "When sunny and above 23°C, close living room and kitchen blinds, but NOT when it's raining",
        "difficulty": "expert",
        "pattern": ["combined-condition", "negation", "fan-out"],
        "blocks": [
            {"type": "GreaterEqual"},
            {"type": "Not"},
            {"type": "And"},
        ],
        "wiring": [
            {"from_title": "Außentemperatur", "to_type": "GreaterEqual"},
            {"from_title": "Sonnenschein", "to_type": "And"},
            {"from_type": "GreaterEqual", "to_type": "And"},
            {"from_title": "Regen", "to_type": "Not"},
            {"from_type": "Not", "to_type": "And"},
            {"from_type": "And", "to_title": "Jalousie 1", "to_room": "Wohnzimmer", "to_connector": "InputTriggerDown"},
            {"from_type": "And", "to_title": "Jalousie 1", "to_room": "Küche", "to_connector": "InputTriggerDown"},
        ],
        "params": [{"block_type": "GreaterEqual", "param": "Input2", "value": "23"}],
    },
    {
        "id_prefix": "expert-irrigate",
        "utterance": "Water the garden when soil moisture is below 25% and temperature is above 5°C, but not when it's raining",
        "difficulty": "expert",
        "pattern": ["combined-condition", "negation", "threshold"],
        "blocks": [
            {"type": "Less"},
            {"type": "GreaterEqual"},
            {"type": "Not"},
            {"type": "And"},
        ],
        "wiring": [
            {"from_title": "Feuchtesensor Garten", "to_type": "Less"},
            {"from_title": "Außentemperatur", "to_type": "GreaterEqual"},
            {"from_title": "Regen", "to_type": "Not"},
            {"from_type": "Less", "to_type": "And"},
            {"from_type": "GreaterEqual", "to_type": "And"},
            {"from_type": "Not", "to_type": "And"},
            {"from_type": "And", "to_title": "Bewässerungsventil"},
        ],
        "params": [
            {"block_type": "Less", "param": "Input2", "value": "25"},
            {"block_type": "GreaterEqual", "param": "Input2", "value": "5"},
        ],
    },
    {
        "id_prefix": "expert-night-motion",
        "utterance": "At night (10pm-6am) when motion is detected in the hallway, turn on light at 20% brightness",
        "difficulty": "expert",
        "pattern": ["time-window", "dimming", "presence"],
        "blocks": [
            {"type": "GreaterEqual"},
            {"type": "Less"},
            {"type": "Or"},
            {"type": "And"},
            {"type": "Mult"},
        ],
        "wiring": [
            {"from_type": "GreaterEqual", "to_type": "Or"},
            {"from_type": "Less", "to_type": "Or"},
            {"from_type": "Or", "to_type": "And"},
            {"from_title": "Bewegungsmelder", "from_room": "Flur", "to_type": "And"},
            {"from_type": "And", "to_type": "Mult"},
            {"from_type": "Mult", "to_title": "Lichtsteuerung", "to_room": "Flur", "to_connector": "Brightness"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "1320"},
            {"block_type": "Less", "param": "Input2", "value": "360"},
            {"block_type": "Mult", "param": "Input2", "value": "20"},
        ],
    },
    {
        "id_prefix": "expert-storm",
        "utterance": "Storm protection: when wind above 50 km/h OR heavy rain, raise all blinds in the house and turn on all lights",
        "difficulty": "expert",
        "pattern": ["threshold", "fan-out", "combined-condition"],
        "blocks": [
            {"type": "GreaterEqual"},
            {"type": "Or"},
        ],
        "wiring": [
            {"from_title": "Windgeschwindigkeit", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "Or"},
            {"from_title": "Regen", "to_type": "Or"},
            {"from_type": "Or", "to_title": "Jalousie 1", "to_room": "Wohnzimmer", "to_connector": "InputTriggerUp"},
            {"from_type": "Or", "to_title": "Jalousie 2", "to_room": "Wohnzimmer", "to_connector": "InputTriggerUp"},
            {"from_type": "Or", "to_title": "Jalousie 1", "to_room": "Schlafzimmer", "to_connector": "InputTriggerUp"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "50"},
        ],
    },
    {
        "id_prefix": "expert-vent-humid",
        "utterance": "When bathroom humidity exceeds 65%, turn on fan and keep it running for 20 minutes after humidity drops, but only between 7am and 10pm",
        "difficulty": "expert",
        "pattern": ["threshold", "delayed-action", "time-window"],
        "blocks": [
            {"type": "GreaterEqual"},
            {"type": "GreaterEqual"},
            {"type": "Less"},
            {"type": "And"},
            {"type": "OnPulseDelay"},
        ],
        "wiring": [
            {"from_title": "Luftfeuchtigkeit", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "And"},
            {"from_type": "And", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Lüfter Bad"},
        ],
        "params": [
            {"block_type": "OnPulseDelay", "param": "T", "value": "1200"},
        ],
    },
]

for combo in expert_combos:
    case = {
        "id": next_id(combo["id_prefix"]),
        "pattern": combo["pattern"],
        "utterance": combo["utterance"],
        "difficulty": combo["difficulty"],
        "expected": {
            "new_blocks": combo["blocks"],
            "wiring": combo["wiring"],
            "params": combo["params"],
        },
    }
    cases.append(case)


# ============================================================
# Pattern J: Override / Manual Disable
# From Reference: PushButton used extensively for mode control
# ============================================================

override_combos = [
    ("Schalter 1", "Jalousie 1", "Wohnzimmer", "InputDisable", "Add override switch to disable living room blind 1 automation"),
    ("Schalter 1", "Jalousie 2", "Wohnzimmer", "InputDisable", "Allow manual override of living room blind 2"),
    ("Schalter 2", "Jalousie 1", "Schlafzimmer", "InputDisable", "Disable automatic bedroom blind control with switch 2"),
    ("Schalter 1", "Klimaanlage", "Wohnzimmer", "toggle", "Toggle AC on/off with switch 1"),
    ("Schalter 2", "Raumregler", "Wohnzimmer", "Temp", "Connect switch 2 to thermostat"),
    ("Bewässerung Manuell", "Bewässerungsventil", "Garten", "I1", "Manual irrigation trigger with dedicated switch"),
]

for source, target, room, conn, utterance in override_combos:
    case = {
        "id": next_id("override"),
        "pattern": ["override", "direct-wiring"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [],
            "wiring": [
                {"from_title": source, "to_title": target, "to_room": room, "to_connector": conn},
            ],
            "params": [],
        },
    }
    cases.append(case)


# ============================================================
# Pattern K: Dimming / Brightness Control
# From Reference: Mult used for brightness scaling
# ============================================================

dimming_combos = [
    ("Lichtsteuerung", "Wohnzimmer", 50, "Set living room light to 50% brightness"),
    ("Lichtsteuerung", "Schlafzimmer", 30, "Dim bedroom light to 30%"),
    ("Lichtsteuerung", "Küche", 75, "Set kitchen light brightness to 75%"),
    ("Lichtsteuerung", "Bad", 40, "Dim bathroom light to 40%"),
    ("Lichtsteuerung", "Flur", 20, "Set hallway light to 20% brightness"),
    ("Gartenbeleuchtung", "Garten", 60, "Set garden lights to 60%"),
]

for target, room, brightness, utterance in dimming_combos:
    case = {
        "id": next_id("dimming"),
        "pattern": ["dimming"],
        "utterance": utterance,
        "difficulty": "medium",
        "expected": {
            "new_blocks": [{"type": "Mult"}],
            "wiring": [
                {"from_type": "Mult", "to_title": target, "to_room": room, "to_connector": "Brightness"},
            ],
            "params": [
                {"block_type": "Mult", "param": "Input2", "value": str(brightness)},
            ],
        },
    }
    cases.append(case)


# ============================================================
# Pattern L: Multi-step chains (real Reference complexity)
# From Reference: InputRef → And → OnPulseDelay → EIBJalousie
# ============================================================

chain_combos = [
    {
        "id_prefix": "chain-sun-delay-blind",
        "utterance": "When sunny, wait 10 minutes then lower bedroom blinds (avoid brief sunshine)",
        "difficulty": "hard",
        "pattern": ["threshold", "delayed-action"],
        "blocks": [{"type": "OnPulseDelay"}],
        "wiring": [
            {"from_title": "Sonnenschein", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Jalousie 1", "to_room": "Schlafzimmer", "to_connector": "InputTriggerDown"},
        ],
        "params": [{"block_type": "OnPulseDelay", "param": "T", "value": "600"}],
    },
    {
        "id_prefix": "chain-wind-delay-up",
        "utterance": "Wait 5 minutes after wind exceeds 40 km/h, then raise living room blinds",
        "difficulty": "hard",
        "pattern": ["threshold", "delayed-action"],
        "blocks": [{"type": "GreaterEqual"}, {"type": "OnPulseDelay"}],
        "wiring": [
            {"from_title": "Windgeschwindigkeit", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Jalousie 1", "to_room": "Wohnzimmer", "to_connector": "InputTriggerUp"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "40"},
            {"block_type": "OnPulseDelay", "param": "T", "value": "300"},
        ],
    },
    {
        "id_prefix": "chain-rain-delay-close",
        "utterance": "When rain starts, wait 2 minutes then close kitchen blinds (avoid false triggers from sprinklers)",
        "difficulty": "hard",
        "pattern": ["delayed-action"],
        "blocks": [{"type": "OnPulseDelay"}],
        "wiring": [
            {"from_title": "Regen", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Jalousie 1", "to_room": "Küche", "to_connector": "InputTriggerDown"},
        ],
        "params": [{"block_type": "OnPulseDelay", "param": "T", "value": "120"}],
    },
    {
        "id_prefix": "chain-humid-temp-fan",
        "utterance": "Turn on bathroom fan when humidity >70% AND temperature >20°C",
        "difficulty": "hard",
        "pattern": ["combined-condition", "threshold"],
        "blocks": [{"type": "GreaterEqual"}, {"type": "GreaterEqual"}, {"type": "And"}],
        "wiring": [
            {"from_title": "Luftfeuchtigkeit", "to_type": "GreaterEqual"},
            {"from_title": "Außentemperatur", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "And"},
            {"from_type": "And", "to_title": "Lüfter Bad"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "70"},
        ],
    },
]

for combo in chain_combos:
    case = {
        "id": next_id(combo["id_prefix"]),
        "pattern": combo["pattern"],
        "utterance": combo["utterance"],
        "difficulty": combo["difficulty"],
        "expected": {
            "new_blocks": combo["blocks"],
            "wiring": combo["wiring"],
            "params": combo["params"],
        },
    }
    cases.append(case)


# ============================================================
# Pattern M: Additional variety (room × action permutations)
# Fill remaining cases to reach 150 total
# ============================================================

# More threshold variants with different rooms
extra_thresholds = [
    ("Außentemperatur", "32", "Turn on AC when outside temperature reaches 32°C", "Wohnzimmer", "Klimaanlage", "toggle", "GreaterEqual"),
    ("Außentemperatur", "26", "Close kitchen blinds when it gets above 26 degrees", "Küche", "Jalousie 2", "InputTriggerDown", "GreaterEqual"),
    ("Windgeschwindigkeit", "25", "Lower bathroom blind when wind exceeds 25 km/h", "Bad", "Jalousie 1", "InputTriggerDown", "GreaterEqual"),
    ("Luftfeuchtigkeit", "60", "Open bedroom blinds when humidity drops below 60%", "Schlafzimmer", "Jalousie 1", "InputTriggerUp", "Less"),
    ("Helligkeit", "200", "Turn off garden lights when brightness exceeds 200 lux (dawn)", "Garten", "Gartenbeleuchtung", "I1", "GreaterEqual"),
    ("Helligkeit", "80", "Turn on hallway light when it gets dark (below 80 lux)", "Flur", "Lichtsteuerung", "I1", "Less"),
    ("Außentemperatur", "10", "Close garage door when temperature drops below 10°C", "Garage", "Garagentor", "I1", "Less"),
    ("Windgeschwindigkeit", "70", "Emergency: raise all blinds in kitchen above 70 km/h", "Küche", "Jalousie 1", "InputTriggerUp", "GreaterEqual"),
]

for sensor, threshold, utterance, room, target, connector, comparator in extra_thresholds:
    case = {
        "id": next_id("extra-threshold"),
        "pattern": ["threshold"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [{"type": comparator}],
            "wiring": [
                {"from_title": sensor, "to_type": comparator},
                {"from_type": comparator, "to_title": target, "to_room": room, "to_connector": connector},
            ],
            "params": [
                {"block_type": comparator, "param": "Input2", "value": threshold},
            ],
        },
    }
    cases.append(case)


# More combined-condition variants
extra_combined = [
    ("Windgeschwindigkeit", "Regen", "30", "GreaterEqual", "Close all blinds in bad weather (wind >30 AND rain)", "Wohnzimmer", "Jalousie 1", "InputTriggerDown"),
    ("Helligkeit", "Außentemperatur", "20000", "GreaterEqual", "Lower bedroom blind when bright AND warm", "Schlafzimmer", "Jalousie 1", "InputTriggerDown"),
    ("Sonnenschein", "Luftfeuchtigkeit", "40", "Less", "Irrigate garden when sunny and humidity below 40%", "Garten", "Bewässerungsventil", "I1"),
    ("Helligkeit", "Windgeschwindigkeit", "10000", "GreaterEqual", "Close kitchen blind when bright and windy (>10000 lux and wind)", "Küche", "Jalousie 2", "InputTriggerDown"),
]

for sensor1, sensor2, threshold, comparator, utterance, room, target, connector in extra_combined:
    case = {
        "id": next_id("extra-combined"),
        "pattern": ["combined-condition", "threshold"],
        "utterance": utterance,
        "difficulty": "medium",
        "expected": {
            "new_blocks": [{"type": comparator}, {"type": "And"}],
            "wiring": [
                {"from_title": sensor1, "to_type": "And"},
                {"from_title": sensor2, "to_type": comparator},
                {"from_type": comparator, "to_type": "And"},
                {"from_type": "And", "to_title": target, "to_room": room, "to_connector": connector},
            ],
            "params": [
                {"block_type": comparator, "param": "Input2", "value": threshold},
            ],
        },
    }
    cases.append(case)


# More time window variants for remaining rooms
extra_time = [
    (1260, 360, 35, "Evening mode: dim living room to 35% from 9pm to 6am", "Wohnzimmer"),
    (1320, 420, 10, "Night mode: set bathroom light to 10% between 10pm and 7am", "Bad"),
    (1200, 300, 50, "Reduce kitchen light to 50% from 8pm to 5am", "Küche"),
]

for from_min, to_min, brightness, utterance, room in extra_time:
    case = {
        "id": next_id("extra-time"),
        "pattern": ["time-window", "dimming"],
        "utterance": utterance,
        "difficulty": "hard",
        "expected": {
            "new_blocks": [
                {"type": "GreaterEqual"},
                {"type": "Less"},
                {"type": "Or"},
                {"type": "Mult"},
            ],
            "wiring": [
                {"from_type": "GreaterEqual", "to_type": "Or"},
                {"from_type": "Less", "to_type": "Or"},
                {"from_type": "Or", "to_type": "Mult"},
                {"from_type": "Mult", "to_title": "Lichtsteuerung", "to_room": room, "to_connector": "Brightness"},
            ],
            "params": [
                {"block_type": "GreaterEqual", "param": "Input2", "value": str(from_min)},
                {"block_type": "Less", "param": "Input2", "value": str(to_min)},
                {"block_type": "Mult", "param": "Input2", "value": str(brightness)},
            ],
        },
    }
    cases.append(case)


# ============================================================
# Pattern N: Additional scenarios to reach 150 total
# ============================================================

# N1: More negation combos across rooms
more_negation = [
    ("Sonnenschein", "Windgeschwindigkeit", "40", "Lower {blind} in {room} when sunny, unless wind above 40 km/h", "Wohnzimmer", "Jalousie 2", "InputTriggerDown"),
    ("Schalter 1", "Regen", None, "Activate garden lights with switch, but not when raining", "Garten", "Gartenbeleuchtung", "I1"),
    ("Bewegungsmelder", "Regen", None, "Turn on garden light on motion, unless it's raining", "Garten", "Gartenbeleuchtung", "I1"),
    ("Sonnenschein", "Regen", None, "Close bathroom blinds when sunny, not raining", "Bad", "Jalousie 1", "InputTriggerDown"),
    ("Außentemperatur", "Regen", "25", "Turn on AC when above 25°C, but not when raining (windows might be open)", "Wohnzimmer", "Klimaanlage", "toggle"),
]

for sensor, negated_sensor, threshold, utterance_tpl, room, target, connector in more_negation:
    room_data = ROOMS[room]
    blind = target
    utterance = utterance_tpl.format(blind=blind, room=room)

    blocks = [{"type": "Not"}, {"type": "And"}]
    wiring_list = [
        {"from_title": negated_sensor, "to_type": "Not"},
        {"from_type": "Not", "to_type": "And"},
        {"from_type": "And", "to_title": target, "to_room": room, "to_connector": connector},
    ]
    params = []

    if threshold:
        blocks.insert(0, {"type": "GreaterEqual"})
        wiring_list.insert(0, {"from_title": sensor, "to_type": "GreaterEqual"})
        wiring_list.insert(1, {"from_type": "GreaterEqual", "to_type": "And"})
        params.append({"block_type": "GreaterEqual", "param": "Input2", "value": threshold})
    else:
        wiring_list.insert(0, {"from_title": sensor, "to_type": "And"})

    case = {
        "id": next_id("negation-extra"),
        "pattern": ["negation", "combined-condition"],
        "utterance": utterance,
        "difficulty": "hard",
        "expected": {"new_blocks": blocks, "wiring": wiring_list, "params": params},
    }
    cases.append(case)

# N2: More delayed actions
more_delays = [
    ("Sonnenschein", "Garten", "Gartenbeleuchtung", "Garten", 600, "Turn off garden lights 10 minutes after sunrise"),
    ("Bewegungsmelder", "Flur", "Lichtsteuerung", "Flur", 300, "Keep hallway light on 5 minutes after motion ends"),
    ("Schalter 2", "Wohnzimmer", "Lichtsteuerung", "Wohnzimmer", 3600, "Living room light on for 1 hour after switch press"),
    ("Regen", None, "Jalousie 1", "Wohnzimmer", 180, "Wait 3 minutes after rain starts, then close living room blind"),
    ("Regen", None, "Jalousie 1", "Küche", 240, "Close kitchen blind 4 minutes after rain begins"),
    ("Windgeschwindigkeit", None, "Jalousie 1", "Schlafzimmer", 120, "Wait 2 minutes after strong wind, then raise bedroom blind"),
]

for source, source_room, target, target_room, delay_s, utterance in more_delays:
    case = {
        "id": next_id("delay-extra"),
        "pattern": ["delayed-action"],
        "utterance": utterance,
        "difficulty": "medium",
        "expected": {
            "new_blocks": [{"type": "OnPulseDelay"}],
            "wiring": [
                {"from_title": source, "to_type": "OnPulseDelay"},
                {"from_type": "OnPulseDelay", "to_title": target, "to_room": target_room},
            ],
            "params": [{"block_type": "OnPulseDelay", "param": "T", "value": str(delay_s)}],
        },
    }
    cases.append(case)

# N3: More direct wiring
more_direct = [
    ("Außentemperatur", None, "AQ", "Klimaanlage", "Wohnzimmer", "inTempCurr", "Send outside temperature to AC unit"),
    ("Luftfeuchtigkeit", None, "AQ", "Lüfter Bad", "Bad", "I1", "Connect humidity sensor to bathroom fan"),
    ("Helligkeit", None, "AQ", "Gartenbeleuchtung", "Garten", "I1", "Link brightness sensor to garden lights"),
    ("Windgeschwindigkeit", None, "AQ", "Jalousie 2", "Wohnzimmer", "InputPos", "Feed wind speed to blind position"),
    ("Sonnenschein", None, "AQ", "Jalousie 1", "Küche", "InputTriggerDown", "Connect sunshine to kitchen blind"),
    ("Regen", None, "AQ", "Jalousie 2", "Küche", "InputTriggerDown", "Close second kitchen blind on rain"),
    ("Schalter 1", None, "Q", "Lüfter Bad", "Bad", "I1", "Manual fan control with switch 1"),
    ("Schalter 2", None, "Q", "Bewässerungsventil", "Garten", "I1", "Manual irrigation trigger with switch 2"),
    ("Bewegungsmelder", "Flur", "OutputPresence", "Garagenlicht", "Garage", "I1", "Turn on garage light from hallway motion"),
    ("Garagentor Sensor", None, "Q", "Lichtsteuerung", "Flur", "I1", "Alert hallway when garage opens"),
    ("Türkontakt Eingang", None, "Q", "Gartenbeleuchtung", "Garten", "I1", "Turn on garden lights when front door opens"),
    ("CO2 Sensor", None, "AQ", "Lüfter Bad", "Bad", "I1", "Activate fan when CO2 high"),
]

for source, source_room, src_conn, target, target_room, dst_conn, utterance in more_direct:
    case = {
        "id": next_id("direct-extra"),
        "pattern": ["direct-wiring"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [],
            "wiring": [
                {"from_title": source, "from_connector": src_conn, "to_title": target, "to_room": target_room, "to_connector": dst_conn},
            ],
            "params": [],
        },
    }
    cases.append(case)

# N4: More fan-out
more_fanout = [
    {
        "utterance": "When temperature drops below 0°C, raise blinds in ALL rooms for frost protection",
        "sensor": "Außentemperatur",
        "threshold": "0",
        "comparator": "Less",
        "targets": [
            ("Jalousie 1", "Wohnzimmer", "InputTriggerUp"),
            ("Jalousie 2", "Wohnzimmer", "InputTriggerUp"),
            ("Jalousie 1", "Schlafzimmer", "InputTriggerUp"),
            ("Jalousie 2", "Schlafzimmer", "InputTriggerUp"),
            ("Jalousie 1", "Küche", "InputTriggerUp"),
            ("Jalousie 2", "Küche", "InputTriggerUp"),
            ("Jalousie 1", "Bad", "InputTriggerUp"),
            ("Jalousie 2", "Bad", "InputTriggerUp"),
        ],
    },
    {
        "utterance": "Turn on all lights when it gets dark outside (brightness < 30 lux)",
        "sensor": "Helligkeit",
        "threshold": "30",
        "comparator": "Less",
        "targets": [
            ("Lichtsteuerung", "Wohnzimmer", "I1"),
            ("Lichtsteuerung", "Flur", "I1"),
            ("Gartenbeleuchtung", "Garten", "I1"),
            ("Garagenlicht", "Garage", "I1"),
        ],
    },
    {
        "utterance": "Storm alert: when wind above 55 km/h, raise blinds in bedroom, kitchen, and bathroom",
        "sensor": "Windgeschwindigkeit",
        "threshold": "55",
        "comparator": "GreaterEqual",
        "targets": [
            ("Jalousie 1", "Schlafzimmer", "InputTriggerUp"),
            ("Jalousie 2", "Schlafzimmer", "InputTriggerUp"),
            ("Jalousie 1", "Küche", "InputTriggerUp"),
            ("Jalousie 1", "Bad", "InputTriggerUp"),
        ],
    },
]

for combo in more_fanout:
    comparator = combo.get("comparator", "GreaterEqual")
    wiring = [{"from_title": combo["sensor"], "to_type": comparator}]
    for target, room, conn in combo["targets"]:
        wiring.append({"from_type": comparator, "to_title": target, "to_room": room, "to_connector": conn})

    case = {
        "id": next_id("fanout-extra"),
        "pattern": ["fan-out", "threshold"],
        "utterance": combo["utterance"],
        "difficulty": "expert" if len(combo["targets"]) > 6 else "hard",
        "expected": {
            "new_blocks": [{"type": comparator}],
            "wiring": wiring,
            "params": [{"block_type": comparator, "param": "Input2", "value": combo["threshold"]}],
        },
    }
    cases.append(case)

# N5: More expert combos
more_expert = [
    {
        "id_prefix": "expert-morning",
        "utterance": "Morning routine: at 7am, open all bedroom blinds if it's not raining",
        "difficulty": "expert",
        "pattern": ["time-window", "negation", "fan-out"],
        "blocks": [{"type": "GreaterEqual"}, {"type": "Less"}, {"type": "And"}, {"type": "Not"}],
        "wiring": [
            {"from_title": "Regen", "to_type": "Not"},
            {"from_type": "Not", "to_type": "And"},
            {"from_type": "And", "to_title": "Jalousie 1", "to_room": "Schlafzimmer", "to_connector": "InputTriggerUp"},
            {"from_type": "And", "to_title": "Jalousie 2", "to_room": "Schlafzimmer", "to_connector": "InputTriggerUp"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "420"},
            {"block_type": "Less", "param": "Input2", "value": "480"},
        ],
    },
    {
        "id_prefix": "expert-evening",
        "utterance": "Evening mode: at sunset (brightness <100 lux), close all blinds and dim lights to 40%",
        "difficulty": "expert",
        "pattern": ["threshold", "fan-out", "dimming"],
        "blocks": [{"type": "Less"}, {"type": "Mult"}],
        "wiring": [
            {"from_title": "Helligkeit", "to_type": "Less"},
            {"from_type": "Less", "to_title": "Jalousie 1", "to_room": "Wohnzimmer", "to_connector": "InputTriggerDown"},
            {"from_type": "Less", "to_title": "Jalousie 2", "to_room": "Wohnzimmer", "to_connector": "InputTriggerDown"},
            {"from_type": "Mult", "to_title": "Lichtsteuerung", "to_room": "Wohnzimmer", "to_connector": "Brightness"},
        ],
        "params": [
            {"block_type": "Less", "param": "Input2", "value": "100"},
            {"block_type": "Mult", "param": "Input2", "value": "40"},
        ],
    },
    {
        "id_prefix": "expert-garage-secure",
        "utterance": "Security: if garage door is open for more than 10 minutes, flash garage light and close door",
        "difficulty": "expert",
        "pattern": ["delayed-action", "fan-out"],
        "blocks": [{"type": "OnPulseDelay"}],
        "wiring": [
            {"from_title": "Garagentor Sensor", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Garagenlicht", "to_room": "Garage"},
            {"from_type": "OnPulseDelay", "to_title": "Garagentor", "to_room": "Garage"},
        ],
        "params": [{"block_type": "OnPulseDelay", "param": "T", "value": "600"}],
    },
    {
        "id_prefix": "expert-smart-irrigate",
        "utterance": "Smart irrigation: water garden when soil dry (<20%), temp above 10°C, no rain, and only between 6am-8am",
        "difficulty": "expert",
        "pattern": ["threshold", "negation", "time-window", "combined-condition"],
        "blocks": [
            {"type": "Less"},
            {"type": "GreaterEqual"},
            {"type": "GreaterEqual"},
            {"type": "Less"},
            {"type": "Not"},
            {"type": "And"},
        ],
        "wiring": [
            {"from_title": "Feuchtesensor Garten", "to_type": "Less"},
            {"from_title": "Außentemperatur", "to_type": "GreaterEqual"},
            {"from_title": "Regen", "to_type": "Not"},
            {"from_type": "Less", "to_type": "And"},
            {"from_type": "GreaterEqual", "to_type": "And"},
            {"from_type": "Not", "to_type": "And"},
            {"from_type": "And", "to_title": "Bewässerungsventil"},
        ],
        "params": [
            {"block_type": "Less", "param": "Input2", "value": "20"},
            {"block_type": "GreaterEqual", "param": "Input2", "value": "10"},
        ],
    },
]

for combo in more_expert:
    case = {
        "id": next_id(combo["id_prefix"]),
        "pattern": combo["pattern"],
        "utterance": combo["utterance"],
        "difficulty": combo["difficulty"],
        "expected": {
            "new_blocks": combo["blocks"],
            "wiring": combo["wiring"],
            "params": combo["params"],
        },
    }
    cases.append(case)

# N6: More stairway / timed switch variants
more_stairway = [
    ("Schalter 1", "Wohnzimmer", "Lichtsteuerung", "Wohnzimmer", 300, "Press switch, living room light on for 5 minutes"),
    ("Türkontakt Eingang", "Flur", "Gartenbeleuchtung", "Garten", 600, "Open front door, garden lights on for 10 minutes"),
    ("Bewegungsmelder Garten", "Garten", "Garagenlicht", "Garage", 120, "Garden motion activates garage light for 2 minutes"),
]

for source, source_room, target, target_room, timer_s, utterance in more_stairway:
    case = {
        "id": next_id("stairway-extra"),
        "pattern": ["timed-switch"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [{"type": "StairwayLS"}],
            "wiring": [
                {"from_title": source, "to_type": "StairwayLS"},
                {"from_type": "StairwayLS", "to_title": target, "to_room": target_room},
            ],
            "params": [{"block_type": "StairwayLS", "param": "T", "value": str(timer_s)}],
        },
    }
    cases.append(case)

# N7: More override combos
more_override = [
    ("Schalter 2", "Jalousie 1", "Küche", "InputDisable", "Disable automatic kitchen blind with switch 2"),
    ("Schalter 2", "Jalousie 2", "Küche", "InputDisable", "Manual override for kitchen blind 2"),
    ("Schalter 1", "Jalousie 1", "Bad", "InputDisable", "Override bathroom blind automation with switch"),
    ("Schalter 1", "Lüfter Bad", "Bad", "I1", "Manual fan control in bathroom"),
    ("Schalter 2", "Garagentor", "Garage", "I1", "Remote garage door trigger with switch 2"),
]

for source, target, room, conn, utterance in more_override:
    case = {
        "id": next_id("override-extra"),
        "pattern": ["override", "direct-wiring"],
        "utterance": utterance,
        "difficulty": "easy",
        "expected": {
            "new_blocks": [],
            "wiring": [
                {"from_title": source, "to_title": target, "to_room": room, "to_connector": conn},
            ],
            "params": [],
        },
    }
    cases.append(case)

# N8: More dimming combos
more_dimming = [
    ("Garagenlicht", "Garage", 80, "Set garage light to 80% brightness"),
    ("Lichtsteuerung", "Wohnzimmer", 10, "Dim living room to 10% for movie mode"),
    ("Lichtsteuerung", "Schlafzimmer", 5, "Minimum 5% bedroom light for night mode"),
]

for target, room, brightness, utterance in more_dimming:
    case = {
        "id": next_id("dimming-extra"),
        "pattern": ["dimming"],
        "utterance": utterance,
        "difficulty": "medium",
        "expected": {
            "new_blocks": [{"type": "Mult"}],
            "wiring": [
                {"from_type": "Mult", "to_title": target, "to_room": room, "to_connector": "Brightness"},
            ],
            "params": [{"block_type": "Mult", "param": "Input2", "value": str(brightness)}],
        },
    }
    cases.append(case)

# N9: Remaining cases to reach 150 exactly
remaining = [
    {
        "id_prefix": "chain-temp-ac-delay",
        "utterance": "When temperature exceeds 30°C for 5 minutes, turn on the AC",
        "difficulty": "hard",
        "pattern": ["threshold", "delayed-action"],
        "blocks": [{"type": "GreaterEqual"}, {"type": "OnPulseDelay"}],
        "wiring": [
            {"from_title": "Außentemperatur", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Klimaanlage", "to_room": "Wohnzimmer", "to_connector": "toggle"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "30"},
            {"block_type": "OnPulseDelay", "param": "T", "value": "300"},
        ],
    },
    {
        "id_prefix": "chain-co2-delay",
        "utterance": "If CO2 stays above 800 ppm for 3 minutes, activate bathroom fan",
        "difficulty": "hard",
        "pattern": ["threshold", "delayed-action"],
        "blocks": [{"type": "GreaterEqual"}, {"type": "OnPulseDelay"}],
        "wiring": [
            {"from_title": "CO2 Sensor", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "OnPulseDelay"},
            {"from_type": "OnPulseDelay", "to_title": "Lüfter Bad"},
        ],
        "params": [
            {"block_type": "GreaterEqual", "param": "Input2", "value": "800"},
            {"block_type": "OnPulseDelay", "param": "T", "value": "180"},
        ],
    },
    {
        "id_prefix": "combo-sun-wind",
        "utterance": "Lower bedroom blind when it's sunny but wind is below 20 km/h (safe to lower)",
        "difficulty": "hard",
        "pattern": ["combined-condition", "negation"],
        "blocks": [{"type": "GreaterEqual"}, {"type": "Not"}, {"type": "And"}],
        "wiring": [
            {"from_title": "Sonnenschein", "to_type": "And"},
            {"from_title": "Windgeschwindigkeit", "to_type": "GreaterEqual"},
            {"from_type": "GreaterEqual", "to_type": "Not"},
            {"from_type": "Not", "to_type": "And"},
            {"from_type": "And", "to_title": "Jalousie 2", "to_room": "Schlafzimmer", "to_connector": "InputTriggerDown"},
        ],
        "params": [{"block_type": "GreaterEqual", "param": "Input2", "value": "20"}],
    },
    {
        "id_prefix": "direct-humidity-display",
        "utterance": "Show humidity reading on the bathroom light controller",
        "difficulty": "easy",
        "pattern": ["direct-wiring"],
        "blocks": [],
        "wiring": [
            {"from_title": "Luftfeuchtigkeit", "from_connector": "AQ", "to_title": "Lichtsteuerung", "to_room": "Bad", "to_connector": "I1"},
        ],
        "params": [],
    },
    {
        "id_prefix": "stairway-door-garage",
        "utterance": "When front door opens, turn on garage light for 4 minutes",
        "difficulty": "easy",
        "pattern": ["timed-switch"],
        "blocks": [{"type": "StairwayLS"}],
        "wiring": [
            {"from_title": "Türkontakt Eingang", "to_type": "StairwayLS"},
            {"from_type": "StairwayLS", "to_title": "Garagenlicht", "to_room": "Garage"},
        ],
        "params": [{"block_type": "StairwayLS", "param": "T", "value": "240"}],
    },
    {
        "id_prefix": "dimming-hallway-50",
        "utterance": "Set hallway brightness to 50%",
        "difficulty": "medium",
        "pattern": ["dimming"],
        "blocks": [{"type": "Mult"}],
        "wiring": [
            {"from_type": "Mult", "to_title": "Lichtsteuerung", "to_room": "Flur", "to_connector": "Brightness"},
        ],
        "params": [{"block_type": "Mult", "param": "Input2", "value": "50"}],
    },
    {
        "id_prefix": "override-garden-irrig",
        "utterance": "Use switch 1 to manually toggle garden irrigation",
        "difficulty": "easy",
        "pattern": ["override", "direct-wiring"],
        "blocks": [],
        "wiring": [
            {"from_title": "Schalter 1", "to_title": "Bewässerungsventil", "to_room": "Garten", "to_connector": "I1"},
        ],
        "params": [],
    },
    {
        "id_prefix": "threshold-bright-garage",
        "utterance": "Turn on garage light automatically when outside brightness drops below 150 lux",
        "difficulty": "easy",
        "pattern": ["threshold"],
        "blocks": [{"type": "Less"}],
        "wiring": [
            {"from_title": "Helligkeit", "to_type": "Less"},
            {"from_type": "Less", "to_title": "Garagenlicht", "to_room": "Garage", "to_connector": "I1"},
        ],
        "params": [{"block_type": "Less", "param": "Input2", "value": "150"}],
    },
]

for combo in remaining:
    case = {
        "id": next_id(combo["id_prefix"]),
        "pattern": combo["pattern"],
        "utterance": combo["utterance"],
        "difficulty": combo["difficulty"],
        "expected": {
            "new_blocks": combo["blocks"],
            "wiring": combo["wiring"],
            "params": combo["params"],
        },
    }
    cases.append(case)

print(f"Generated {len(cases)} eval cases")

# Difficulty distribution
from collections import Counter
diff_dist = Counter(c["difficulty"] for c in cases)
print(f"Difficulty: {dict(diff_dist)}")

# Pattern distribution
pat_dist = Counter()
for c in cases:
    for p in c["pattern"]:
        pat_dist[p] += 1
print(f"Patterns: {dict(pat_dist)}")

# Output
output = json.dumps(cases, indent=2, ensure_ascii=False)
print(f"Output size: {len(output)} bytes")

# Write to file
with open("tests/eval/reference-cases.json", "w") as f:
    f.write(output)
print("Written to tests/eval/reference-cases.json")
