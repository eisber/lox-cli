#!/usr/bin/env python3
"""Replay a pinned public fixture in the official disconnected Windows simulator."""

import argparse
import copy
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path, PureWindowsPath
import re
import shutil
import socket
import struct
import subprocess
import sys
import uuid

from loxone_ux_transport import MCPClient, UxError, parse_json
from official_sim_runner import fixture_basename, native_channels, sampling_plan


APPLICATION_SHA = "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3"
STATE_BOXES = {"pin": (256, 38, 281, 55), "wired_reference": (299, 38, 326, 55)}
MINIMUM_OCR_CONFIDENCE = 90


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read_json(path):
    return parse_json(Path(path).read_text(encoding="utf-8-sig"))


def write_json(path, value):
    with Path(path).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=True, separators=(",", ":"), allow_nan=False)
        stream.write("\n")


def load_ocr_snapshot(path):
    """Execute the same immutable helper bytes whose source is retained."""
    expected = digest(path)
    spec = importlib.util.spec_from_file_location("native_ocr_snapshot_" + uuid.uuid4().hex, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    if digest(path) != expected:
        raise ValueError("Retained OCR helper changed while loading; recognition was refused.")
    return module


def artifact(path, root, kind=None):
    result = {"path": str(Path(path).resolve().relative_to(root)), "sha256": digest(path)}
    if kind:
        result["kind"] = kind
    return result


def validate_case(case, fixture):
    if type(case.get("schema_version")) is not int or case["schema_version"] != 1 or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,100}", case.get("id", "")):
        raise ValueError("Unsupported case identity.")
    if case.get("application", {}).get("sha256") != APPLICATION_SHA:
        raise ValueError("Recover native bindings before changing the application pin.")
    if digest(fixture) != case.get("fixture_sha256"):
        raise ValueError("Fixture bytes do not match the case definition.")
    phases = sampling_plan(case)
    channels = native_channels(case)
    if "simulator_start_time" in case and (
        not isinstance(case["simulator_start_time"], str)
        or not re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", case["simulator_start_time"])
    ):
        raise ValueError("Simulator-local start time must be exact HH:MM:SS; the OS clock is never changed.")
    if case.get("simulator_time_edit", "value-pattern") not in {"value-pattern", "verified-sections"}:
        raise ValueError("Unsupported simulator-local time edit method.")
    expected = case.get("expected_outputs")
    purpose = case.get("purpose", "assertion")
    if purpose not in {"assertion", "exploratory"} or not isinstance(expected, dict):
        raise ValueError("Declare an assertion or an explicitly exploratory native case.")
    if purpose == "exploratory":
        if expected:
            raise ValueError("Exploratory cases retain actual observations without expected output assertions.")
    elif set(expected) != {phase["name"] for phase in phases} or any(
        not isinstance(outputs, dict) or set(outputs) != set(channels)
        or any(type(value) is not int or value not in (0, 1) for value in outputs.values())
        for outputs in expected.values()
    ):
        raise ValueError("Every asserted phase must separately declare every native channel's digital expectation.")
    return case


def judge(case, remote):
    if remote.get("error"):
        return "blocked", remote["error"].get("message", "Native replay failed.")
    if remote.get("native_running_verified") is not True:
        return "blocked", "Native simulator execution/progress was not positively verified."
    if remote.get("cleanup", {}).get("status") != "closed":
        return "blocked", "Owned native application cleanup is unverified."
    by_phase = {}
    for observation in remote.get("observations", []):
        phase = observation.get("phase")
        if not isinstance(phase, str) or phase in by_phase:
            return "blocked", "Ambiguous sample phases."
        by_phase[phase] = observation
    plan = sampling_plan(case)
    channels = native_channels(case)
    for window in plan:
        phase = window["name"]
        actual = by_phase.get(phase, {})
        source = actual.get("source", {})
        if (not actual.get("outputs") or source.get("kind") not in {"uia", "native-readable", "ocr"}
                or not source.get("artifacts")):
            return "blocked", "Native output readback is unavailable or ambiguous; no default state was inferred."
        elapsed = actual.get("elapsed_ms")
        if (type(elapsed) not in (int, float) or not math.isfinite(elapsed)
                or elapsed < window["at_seconds"] * 1000
                or ("latest_seconds" in window and elapsed > window["latest_seconds"] * 1000)):
            return "blocked", "The asserted native phase did not reach its declared calculation interval."
        interval = actual.get("capture_interval_ms")
        if "sampling_phases" in case and interval is None:
            return "blocked", "Named native phases require the actual bounded capture interval."
        if interval is not None and (
            not isinstance(interval, list) or len(interval) != 2
            or any(type(value) not in (int, float) or not math.isfinite(value) for value in interval)
            or not window["at_seconds"] * 1000 <= interval[0] <= elapsed <= interval[1]
            or ("latest_seconds" in window and interval[1] > window["latest_seconds"] * 1000)
        ):
            return "blocked", "Native capture straddles or leaves the declared phase window."
        if "simulator_time_window" in window:
            clock = actual.get("simulator_time")
            if (not isinstance(clock, str)
                    or not re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", clock)
                    or not window["simulator_time_window"][0] <= clock <= window["simulator_time_window"][1]):
                return "blocked", "Actual native simulator clock is missing or outside the declared phase window."
            after = actual.get("simulator_time_after_capture")
            if after is not None and (
                not isinstance(after, str) or not re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", after)
                or not clock <= after <= window["simulator_time_window"][1]
            ):
                return "blocked", "Native capture straddles the declared simulator-clock window."
        if any(type(actual["outputs"].get(key)) is not int or actual["outputs"][key] not in (0, 1) for key in channels):
            return "blocked", "Native output readback is not an unambiguous digital value."
        expected = case["expected_outputs"].get(phase, {})
        if any(actual["outputs"][key] != value for key, value in expected.items()):
            return "fail", "Actual native outputs differ from the separately declared expectation."
    if case.get("purpose") == "exploratory":
        return "blocked", "Exploratory native observations retained; no reviewed behavioral assertion was declared."
    return "pass", None


def validate_ocr_profile(path, root):
    profile = read_json(path)
    if (type(profile.get("schema_version")) is not int or profile["schema_version"] != 1 or profile.get("kind") != "native-tesseract-ocr-profile"
            or profile.get("language") != "deu"):
        raise ValueError("Use the independently verified German Tesseract profile, schema 1.")
    prefixes = ["executable", "model"]
    if "header_model_path" in profile or "header_model_sha256" in profile:
        prefixes.append("header_model")
    for prefix in prefixes:
        file = Path(profile[prefix + "_path"]).resolve()
        file.relative_to(root)
        if digest(file) != profile[prefix + "_sha256"]:
            raise ValueError("OCR executable/model does not match its pin.")
    if Path(profile["model_path"]).name != "deu.traineddata":
        raise ValueError("The fixed OCR recipe requires the pinned deu.traineddata model.")
    # Optional, file-only dependency: never install or download a package here.
    from PIL import Image, ImageOps  # noqa: F401
    return profile


def native_state_boxes(anchor_path, image_size=None, offsets=None):
    record = read_json(anchor_path)
    anchor = record.get("header", {})
    if re.sub(r"\s+", "", anchor.get("text", "")) != "EnergyManager":
        raise ValueError("Native output header is not the pinned public Energy Manager.")
    words = anchor.get("words")
    if not isinstance(words, list) or len(words) != 2 or any(
        not isinstance(word, dict) or not isinstance(word.get("rect"), list) or len(word["rect"]) != 4
        or any(type(value) not in (int, float) or not math.isfinite(value) for value in word["rect"])
        or word["rect"][0] < 0 or word["rect"][1] < 0 or word["rect"][2] <= 0 or word["rect"][3] <= 0
        for word in words
    ):
        raise ValueError("Native header anchor geometry is malformed.")
    left = min(word["rect"][0] for word in words)
    top = min(word["rect"][1] for word in words)
    width = max(word["rect"][0] + word["rect"][2] for word in words) - left
    height = max(word["rect"][1] + word["rect"][3] for word in words) - top
    if (width, height) != (107, 14):
        raise ValueError("Native header geometry changed; recover a new raster binding, do not guess coordinates.")
    boxes = {
        name: (int(left + box[0]), int(top + box[1]), int(left + box[2]), int(top + box[3]))
        for name, box in (offsets if offsets is not None else STATE_BOXES).items()
    }
    if image_size is not None and (
        not 0 <= left < left + width <= image_size[0] or not 0 <= top < top + height <= image_size[1]
        or any(not 0 <= box[0] < box[2] <= image_size[0] or not 0 <= box[1] < box[3] <= image_size[1]
               for box in boxes.values())
    ):
        raise ValueError("Native header or asserted pin/reference does not fully fit the actual crop.")
    return boxes


def channel_offsets(case):
    channels = native_channels(case)
    declared = case.get("readback", {}).get("channel_boxes")
    if declared is None:
        if channels != ["OL1"]:
            raise ValueError("Additional native channels require an actual reviewed raster calibration.")
        return {"OL1": STATE_BOXES}
    if not isinstance(declared, dict) or set(declared) != set(channels):
        raise ValueError("Raster calibration must cover exactly the declared native channels.")
    rectangles = []
    for locations in declared.values():
        if not isinstance(locations, dict) or set(locations) != {"pin", "wired_reference"}:
            raise ValueError("Each native channel needs independently wired pin/reference rectangles.")
        pair = []
        for box in locations.values():
            if (not isinstance(box, list) or len(box) != 4 or any(type(value) is not int for value in box)
                    or not 0 <= box[0] < box[2] <= 1600 or not 0 <= box[1] < box[3] <= 1600):
                raise ValueError("Native raster rectangles must be bounded header-relative physical pixels.")
            if box in rectangles or any(
                box[0] < other[2] and box[2] > other[0] and box[1] < other[3] and box[3] > other[1]
                for other in pair
            ):
                raise ValueError("Native channels cannot reuse a rectangle, and each pin/reference pair must be independent.")
            rectangles.append(box)
            pair.append(box)
    return declared


def validate_readback_calibration(case, root):
    offsets = channel_offsets(case)
    if "channel_boxes" not in case.get("readback", {}):
        return []
    calibration = case["readback"].get("calibration", {})
    if not isinstance(calibration, dict) or not isinstance(calibration.get("review"), str) or not calibration["review"].strip():
        raise ValueError("Explicit raster boxes require a retained actual header/row geometry review.")
    files = []
    for key in ("image", "header_anchor"):
        ref = calibration.get(key, {})
        if not isinstance(ref, dict) or not isinstance(ref.get("path"), str):
            raise ValueError("Native raster calibration needs hash-bound source image and actual header anchor.")
        file = (root / ref["path"]).resolve()
        file.relative_to(root)
        if digest(file) != ref.get("sha256"):
            raise ValueError("Native raster calibration source changed.")
        files.append(file)
    with files[0].open("rb") as stream:
        header = stream.read(24)
    if len(header) != 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
        raise ValueError("Native raster calibration source must be its actual PNG capture.")
    size = struct.unpack(">II", header[16:24])
    for locations in offsets.values():
        native_state_boxes(files[1], size, locations)
    evidence = calibration.get("evidence", [])
    if not isinstance(evidence, list) or len(evidence) > 32:
        raise ValueError("Raster calibration evidence must be a bounded artifact list.")
    for ref in evidence:
        if not isinstance(ref, dict) or not isinstance(ref.get("path"), str):
            raise ValueError("Raster calibration evidence must have hash-bound relative paths.")
        file = (root / ref["path"]).resolve()
        file.relative_to(root)
        if digest(file) != ref.get("sha256"):
            raise ValueError("Retained raster calibration evidence changed.")
        files.append(file)
    return files


def native_header_proposals(ocr):
    proposals = []
    for line in ocr.get("lines", []):
        words = line.get("words", [])
        if len(words) != 2 or words[0].get("text") != "Energy":
            continue
        if any(not isinstance(word.get("rect"), list) or len(word["rect"]) != 4
               or any(type(value) not in (int, float) or not math.isfinite(value) for value in word["rect"])
               for word in words):
            continue
        left = min(word["rect"][0] for word in words)
        top = min(word["rect"][1] for word in words)
        right = max(word["rect"][0] + word["rect"][2] for word in words)
        bottom = max(word["rect"][1] + word["rect"][3] for word in words)
        if (right - left, bottom - top) == (107, 14):
            proposals.append({"raw_windows_ocr": copy.deepcopy(line), "box": [int(left), int(top), int(right), int(bottom)]})
    return proposals


def header_anchor_from_recognition(recognized):
    words = recognized.get("words", [])
    source = recognized.get("source", {})
    box = source.get("box", [])
    image_size = source.get("image_size", [])
    if (recognized.get("text") != "Energy Manager" or len(words) != 2
            or [word.get("text") for word in words] != ["Energy", "Manager"]
            or len(box) != 4 or any(type(value) is not int for value in box)
            or len(image_size) != 2
            or not 0 <= box[0] < box[2] <= image_size[0] or not 0 <= box[1] < box[3] <= image_size[1]):
        raise ValueError("Independent native header recognition is malformed.")
    mapped = []
    for word in words:
        value = word.get("box", [])
        confidence = word.get("confidence")
        if (len(value) != 4 or any(type(item) is not int for item in value)
                or value[2] <= 0 or value[3] <= 0
                or type(confidence) not in (int, float) or not math.isfinite(confidence) or not 90 <= confidence <= 100):
            raise ValueError("Independent header word geometry/confidence is invalid.")
        x, y, width, height = value
        left = math.floor(box[0] + (x - 24) / 4)
        top = math.floor(box[1] + (y - 24) / 4)
        right = math.ceil(box[0] + (x + width - 24) / 4)
        bottom = math.ceil(box[1] + (y + height - 24) / 4)
        if not box[0] <= left < right <= box[2] or not box[1] <= top < bottom <= box[3]:
            raise ValueError("Recognized header word leaves its actual source crop.")
        mapped.append({"text": word["text"], "rect": [left, top, right - left, bottom - top], "confidence": confidence})
    return {"text": recognized["text"], "words": mapped}


def validate_canvas_capture(capture, image_path=None):
    if not isinstance(capture, dict) or capture.get("scope") != "owned-native-programming-canvas":
        raise ValueError("Native canvas capture did not retain its scope.")
    rectangle = capture.get("rectangle")
    canvas = capture.get("owned_canvas", {})
    inset = capture.get("inset_pixels")
    root = capture.get("root_handle")
    if (not isinstance(rectangle, list) or len(rectangle) != 4
            or any(type(value) not in (int, float) or not math.isfinite(value) for value in rectangle)
            or rectangle[2] <= 0 or rectangle[3] <= 0
            or type(inset) is not int or inset <= 0
            or type(root) is not int or root <= 0 or root != canvas.get("root_handle")
            or type(canvas.get("native_handle")) is not int or canvas["native_handle"] <= 0
            or capture.get("origin_screen") != rectangle[:2]):
        raise ValueError("Native canvas origin/dimensions/root proof are invalid.")
    left, top, width, height = rectangle
    right, bottom = left + width, top + height
    for label, bounds, margin in (
        ("canvas", canvas.get("bounds"), inset),
        ("main window", canvas.get("main_bounds"), inset),
        ("virtual screen", capture.get("virtual_screen_bounds"), 0),
    ):
        if (not isinstance(bounds, list) or len(bounds) != 4
                or any(type(value) not in (int, float) or not math.isfinite(value) for value in bounds)
                or not bounds[0] + margin <= left < right <= bounds[2] - margin
                or not bounds[1] + margin <= top < bottom <= bounds[3] - margin):
            raise ValueError(f"Native canvas capture leaves its verified {label} bounds.")
    if image_path is not None:
        with Path(image_path).open("rb") as stream:
            header = stream.read(24)
        if (len(header) != 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR"
                or struct.unpack(">II", header[16:24]) != (width, height)):
            raise ValueError("Native canvas image dimensions do not match the retained capture rectangle.")


def decode_native_image(image_path, anchor_path, destination, profile, root, case=None, ocr_helper=None):
    if ocr_helper is None:
        import official_sim_ocr as ocr_helper
    with Path(image_path).open("rb") as stream:
        header = stream.read(24)
    if len(header) != 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
        raise ValueError("Native output image is not the original PNG capture.")
    size = struct.unpack(">II", header[16:24])
    boxes = {
        channel: native_state_boxes(anchor_path, size, locations)
        for channel, locations in channel_offsets(case or {}).items()
    }
    destination.mkdir(parents=True, exist_ok=False)
    provenance = {
        "kind": "official-native-output-ocr",
        "recipe": profile.get("literal_recipe", "green-wire-minimum-4x-bicubic-pad24-deu-psm7-v1"),
        "image": artifact(image_path, root), "header_anchor": artifact(anchor_path, root),
        "engine": {"kind": "tesseract", "executable_sha256": profile["executable_sha256"],
                   "model_sha256": profile["model_sha256"], "language": "deu",
                   "psm": 7, "default_dawgs": True, "minimum_confidence": MINIMUM_OCR_CONFIDENCE,
                   "tool_version": profile.get("tool_version")},
        "readings": [], "outputs": {},
    }
    for channel, locations in boxes.items():
        reads = []
        for name, box in locations.items():
            actual = ocr_helper.recognize_literal(image_path, box, profile, destination / channel / name)
            reads.append(actual)
            provenance["readings"].append({
                "channel": channel, "location": name, "box": box, "text": actual["text"], "value": actual["value"],
                "confidence": actual["confidence"], "record": artifact(actual["record_path"], root),
            })
        agrees = (
            reads[0]["text"] == reads[1]["text"] and reads[0]["value"] == reads[1]["value"]
            and type(reads[0]["value"]) is int and reads[0]["value"] in (0, 1)
            and all(type(read["confidence"]) in (int, float) and math.isfinite(read["confidence"])
                    and MINIMUM_OCR_CONFIDENCE <= read["confidence"] <= 100 for read in reads)
        )
        if agrees:
            provenance["outputs"][channel] = reads[0]["value"]
    write_json(destination / "readback.json", provenance)
    return provenance


def validate_post_attach_clock_pair(samples, readings, attach_end_epoch):
    if len(samples) != 2 or len(readings) != 2:
        raise ValueError("Two actual post-attach clock samples are required.")
    timestamps = [sample.get("clock_capture_unix_ms") for sample in samples]
    if (type(attach_end_epoch) not in (int, float) or not math.isfinite(attach_end_epoch)
            or any(type(value) not in (int, float) or not math.isfinite(value) for value in timestamps)
            or timestamps[0] <= attach_end_epoch * 1000 or timestamps[1] - timestamps[0] < 3000):
        raise ValueError("Actual clock captures must follow attachment and be at least three seconds apart.")
    for sample, reading in zip(samples, readings):
        if (sample.get("observer_mode") != "native-win32" or sample.get("native_identity_geometry_verified") is not True
                or sample.get("readiness_mode") != "direct-post-attach-clock-ocr"
                or not re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", reading.get("text", ""))
                or type(reading.get("confidence")) not in (int, float) or not 90 <= reading["confidence"] <= 100
                or type(reading.get("seconds_since_midnight")) is not int):
            raise ValueError("Post-attach clock evidence is not an exact high-confidence native observation.")
    delta = (readings[1]["seconds_since_midnight"] - readings[0]["seconds_since_midnight"]) % 86400
    if delta == 0:
        raise ValueError("Post-attach native clock did not advance; pre-attach proof is insufficient.")
    return {"capture_delta_ms": timestamps[1] - timestamps[0], "native_clock_delta_seconds": delta}


def prove_native_clock_progress(original, native_report, local, bound, profile, analysis, root, ocr_helper=None):
    if ocr_helper is None:
        import official_sim_ocr as ocr_helper
    if (original["execution"].get("observer_mode") != "native-win32"
            or original["execution"].get("instrumentation_start") != "post-start"
            or native_report.get("pre_attach_running_verified") is not True
            or native_report.get("post_attach_clock_proof_pending") is not True):
        raise ValueError("Native-only observation lacks its verified pre-attach running state.")
    attachment = native_report.get("attachment") or {}
    if attachment.get("status") != "client_log_opened":
        raise ValueError("Native attachment was not initialized.")
    samples = []
    readings = []
    for phase in ("post_attach_clock_1", "post_attach_clock_2"):
        matches = [sample for sample in native_report.get("observations", []) if sample.get("phase") == phase]
        if len(matches) != 1:
            raise ValueError("Post-attach clock sample is missing or ambiguous.")
        sample = matches[0]
        image = (local / "remote" / sample["clock_image"]).resolve()
        if str(image) not in bound:
            raise ValueError("Actual post-attach clock PNG is not bound by the native receipt.")
        with image.open("rb") as stream:
            header = stream.read(24)
        if len(header) != 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
            raise ValueError("Actual native clock capture is not PNG.")
        width, height = struct.unpack(">II", header[16:24])
        reading = ocr_helper.recognize_clock(image, (0, 0, width, height), profile, analysis / "clock-proof" / phase)
        samples.append(sample)
        readings.append(reading)
    progress = validate_post_attach_clock_pair(samples, readings, attachment.get("attach_end_epoch"))
    proof = {
        "kind": "post-attach-native-clock-proof", "pre_attach_running_verified": True,
        "attachment_end_epoch": attachment["attach_end_epoch"], **progress,
        "readings": [
            {"phase": sample["phase"], "capture_unix_ms": sample["clock_capture_unix_ms"],
             "text": reading["text"], "seconds_since_midnight": reading["seconds_since_midnight"],
             "confidence": reading["confidence"], "record": artifact(reading["record_path"], root)}
            for sample, reading in zip(samples, readings)
        ],
        "scope": "Actual post-attach clock OCR, not pre-attach state or expected time; startup/SimulationStart excluded.",
        "readiness_mode": "direct-post-attach-clock-ocr-no-WM_NULL-proxy",
    }
    write_json(analysis / "clock-proof" / "progress.json", proof)
    return proof


def admit_receipt(source_path, root, profile_path):
    source_path = source_path.resolve()
    source_path.relative_to(root)
    original = read_json(source_path)
    if (original.get("schema_version") != 1 or original.get("kind") != "official-native-replay"
            or original.get("application", {}).get("sha256") != APPLICATION_SHA):
        raise ValueError("OCR admission requires a pinned official native replay receipt.")
    run_id = original.get("execution", {}).get("run_id") or original.get("run_id")
    if (run_id != source_path.parent.name
            or PureWindowsPath(original.get("remote", {}).get("run_dir", "")).name != run_id):
        raise ValueError("Native execution identity is missing or does not match the original run.")
    bound = {}
    for entry in original.get("artifacts", []):
        file = (root / entry["path"]).resolve()
        file.relative_to(root)
        if digest(file) != entry["sha256"]:
            raise ValueError("An original replay artifact changed; OCR admission was refused.")
        bound[str(file)] = entry["sha256"]
    definition = original["case"]["definition"]
    case_path = (root / definition["path"]).resolve()
    if (case_path.parent != source_path.parent or case_path.name != "case.definition.json"
            or bound.get(str(case_path)) != definition["sha256"]
            or digest(case_path) != definition["sha256"] or definition["sha256"] != original["case"]["sha256"]):
        raise ValueError("Original case-definition binding is invalid.")
    case = read_json(case_path)
    if (case["id"] != original["case"]["id"] or case["fixture_sha256"] != original["case"]["fixture_sha256"]
            or case["application"] != original["application"]):
        raise ValueError("Original application/case/fixture identities conflict; relabeling was refused.")
    if not any(value == original["case"]["fixture_sha256"] for value in bound.values()):
        raise ValueError("Original replay does not bind its actual fixture bytes.")
    for calibration in validate_readback_calibration(case, root):
        if str(calibration) not in bound:
            raise ValueError("Native raster calibration is not bound by the original receipt.")
    profile = validate_ocr_profile(profile_path, root)
    local = source_path.parent
    name = "ocr-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-") + uuid.uuid4().hex
    analysis = local / name
    analysis.mkdir(exist_ok=False)
    shutil.copyfile(profile_path, analysis / "profile.json")
    shutil.copyfile(Path(__file__), analysis / "decoder-source.py")
    shutil.copyfile(Path(__file__).with_name("official_sim_ocr.py"), analysis / "ocr-helper-source.py")
    ocr_helper = load_ocr_snapshot(analysis / "ocr-helper-source.py")
    receipt = copy.deepcopy(original)
    observations = []
    errors = []
    header_model_used = False
    native_report_path = local / "remote" / "remote-report.json"
    native_report = read_json(native_report_path) if str(native_report_path.resolve()) in bound else {}
    native_clock_proof = None
    if original["execution"].get("observer_mode") == "native-win32":
        try:
            header_model_used = True
            native_clock_proof = prove_native_clock_progress(original, native_report, local, bound, profile, analysis, root, ocr_helper)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            errors.append(str(error))
    phase_names = {phase["name"] for phase in sampling_plan(case)}
    actual_phase_names = [sample.get("phase") for sample in original.get("native_progress", []) if sample.get("phase") in phase_names]
    if len(actual_phase_names) != len(set(actual_phase_names)):
        raise ValueError("Original native capture phases are ambiguous.")
    for sample in original.get("native_progress", []):
        if sample["phase"] not in phase_names:
            continue
        image = local / "remote" / (sample["phase"] + ".png")
        anchor = local / "remote" / (sample["phase"] + ".output-anchor.json")
        windows_ocr = local / "remote" / (sample["phase"] + ".ocr.json")
        if str(windows_ocr.resolve()) not in bound and original["execution"].get("observer_mode") == "native-win32":
            windows_ocr = local / "remote" / "progress_running.ocr.json"
        if str(image.resolve()) not in bound:
            errors.append("Actual phase image was not bound by the native receipt.")
            continue
        phase_directory = analysis / sample["phase"]
        extra_sources = []
        try:
            if original["execution"].get("capture_scope") == "canvas":
                validate_canvas_capture(sample.get("capture"), image)
            if str(anchor.resolve()) not in bound:
                if str(windows_ocr.resolve()) not in bound:
                    raise ValueError("Native header proposal source is not bound by the original receipt.")
                proposals = native_header_proposals(read_json(windows_ocr))
                if len(proposals) != 1:
                    raise ValueError("Native header proposal geometry is missing or ambiguous; no caption was corrected.")
                proposal = proposals[0]
                if not profile.get("header_model_path") or not profile.get("header_model_sha256"):
                    raise ValueError("A pinned English header model is required for independent caption confirmation.")
                header_model_used = True
                recognized = ocr_helper.recognize_header(image, proposal["box"], profile, phase_directory / "header")
                anchor = phase_directory / "header-anchor.json"
                write_json(anchor, {
                    "header": header_anchor_from_recognition(recognized),
                    "text_source": artifact(recognized["record_path"], root),
                    "raw_windows_ocr_proposal": proposal,
                    "raw_windows_ocr_source": artifact(windows_ocr, root),
                    "geometry": "Actual English TSV word boxes mapped from 4x/border24 pixels into immutable source image.",
                })
                extra_sources.append(windows_ocr)
            decoded = decode_native_image(image, anchor, phase_directory / "outputs", profile, root, case, ocr_helper)
            clock_reading = None
            if original["execution"].get("observer_mode") == "native-win32" and "sampling_phases" in case:
                clock_image = (local / "remote" / sample.get("clock_image", "")).resolve()
                if str(clock_image) not in bound:
                    raise ValueError("Named native phases require their actual hash-bound clock PNG.")
                with clock_image.open("rb") as stream:
                    clock_header = stream.read(24)
                if len(clock_header) != 24 or clock_header[:8] != b"\x89PNG\r\n\x1a\n":
                    raise ValueError("Native phase clock capture is not PNG.")
                width, height = struct.unpack(">II", clock_header[16:24])
                clock_reading = ocr_helper.recognize_clock(clock_image, (0, 0, width, height), profile, phase_directory / "clock")
                extra_sources.append(clock_image)
                header_model_used = True
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            errors.append(str(error))
            continue
        if set(decoded["outputs"]) != set(native_channels(case)):
            errors.append("Exact native pin/reference words did not agree at confidence >=90; no fuzzy correction or default OFF.")
        observation = {key: copy.deepcopy(sample[key]) for key in ("phase", "elapsed_ms", "simulator_time")}
        if "capture_interval_ms" in sample:
            observation["capture_interval_ms"] = copy.deepcopy(sample["capture_interval_ms"])
        if "simulator_time_after_capture" in sample:
            observation["simulator_time_after_capture"] = sample["simulator_time_after_capture"]
        if clock_reading is not None:
            observation["simulator_time"] = clock_reading["text"]
            observation["simulator_time_source"] = artifact(clock_reading["record_path"], root)
        observation["outputs"] = decoded["outputs"]
        observation["source"] = {
            "kind": "ocr", "artifacts": [
                str(image.relative_to(root)), str(anchor.relative_to(root)),
                *[str(file.relative_to(root)) for file in extra_sources],
                *[str(file.relative_to(root)) for file in phase_directory.rglob("*") if file.is_file()],
            ],
        }
        observation["source"]["artifacts"] = list(dict.fromkeys(observation["source"]["artifacts"]))
        observations.append(observation)
    record = {
        "native_running_verified": (native_clock_proof is not None
                                    if original["execution"].get("observer_mode") == "native-win32"
                                    else original["execution"]["native_running_verified"]),
        "error": None, "cleanup": original["execution"].get("cleanup", {}),
        "observations": observations,
    }
    if str(native_report_path.resolve()) in bound:
        record["error"] = native_report.get("error")
    elif original["execution"]["native_running_verified"] is not True:
        record["error"] = {"message": original["execution"].get("reason") or "Native execution was not verified."}
    if original["execution"].get("observer_mode") == "native-win32" and native_clock_proof is None and not record["error"]:
        record["error"] = {"message": "Actual post-attach native clock proof is unavailable. " + (errors[0] if errors else "")}
    if original["execution"]["mode"] == "drcov":
        logs = []
        for path in original.get("coverage_logs", []):
            file = (root / path).resolve()
            file.relative_to(root)
            if str(file) not in bound:
                raise ValueError("Instrumentation log is not bound by the original native artifact manifest.")
            logs.append(file)
        if not any(file.stat().st_size > 0 for file in logs):
            flush_error = "No nonempty actual native drcov log was flushed; instrumentation remains blocked."
            source_error = record.get("error")
            record["error"] = {
                "message": ((source_error.get("message", "") + " ") if source_error else "") + flush_error,
                "native_error": source_error,
            }
    status, reason = judge(case, record)
    receipt["observations"] = observations
    receipt["execution"].update(status=status, reason=reason)
    receipt["execution"]["native_running_verified"] = record["native_running_verified"]
    receipt["execution"]["run_id"] = run_id
    receipt["ocr_admission"] = {
        "source_receipt": artifact(source_path, root), "analysis_errors": errors,
        "scope": "File-only postprocessing of immutable actual native captures; native execution was not repeated or fabricated.",
    }
    if native_clock_proof is not None:
        receipt["native_running_proof"] = native_clock_proof
    additional = [source_path, *[file for file in analysis.rglob("*") if file.is_file()],
                  Path(profile["executable_path"]), Path(profile["model_path"])]
    if header_model_used:
        additional.append(Path(profile["header_model_path"]))
    seen = {entry["path"] for entry in receipt["artifacts"]}
    for file in additional:
        entry = artifact(file, root)
        if entry["path"] not in seen:
            receipt["artifacts"].append(entry)
            seen.add(entry["path"])
    destination = local / ("receipt-" + name + ".json")
    write_json(destination, receipt)
    return {"status": status, "reason": reason, "receipt": str(destination.relative_to(root)),
            "source_receipt": str(source_path.relative_to(root))}, 0 if status == "pass" else 2


class Transfer:
    def __init__(self, profile):
        if set(profile) != {"host", "port", "user", "identity_file", "known_hosts_file", "desktop_host"}:
            raise ValueError("SSH transfer profile contains unsupported fields.")
        if (not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.-]*", profile["host"])
                or not re.fullmatch(r"[A-Za-z0-9_.-]+", profile["user"])
                or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", profile["desktop_host"])
                or type(profile["port"]) is not int or not 1 <= profile["port"] <= 65535):
            raise ValueError("Invalid SSH transfer destination.")
        self.destination = profile["user"] + "@" + profile["host"]
        self.args = [
            "scp", "-q", "-P", str(profile["port"]), "-i",
            str(Path(os.path.expandvars(profile["identity_file"])).expanduser()),
            "-o", "BatchMode=yes", "-o", "IdentitiesOnly=yes",
            "-o", "StrictHostKeyChecking=yes", "-o", "ConnectTimeout=8",
            "-o", "UserKnownHostsFile=" + str(Path(os.path.expandvars(profile["known_hosts_file"])).expanduser()),
        ]

    def copy(self, source, destination, *, recursive=False):
        result = subprocess.run(
            self.args + (["-r"] if recursive else []) + [str(source), str(destination)],
            capture_output=True, timeout=180, check=False,
        )
        if result.returncode:
            raise RuntimeError("Authenticated SSH artifact transfer failed; inspect the owned run before retry.")

    def push(self, local, remote):
        self.copy(local, self.destination + ":" + str(remote))

    def fetch(self, remote, local):
        self.copy(self.destination + ":" + str(remote), local, recursive=True)


def ps_path(path):
    return "'" + str(path).replace("'", "''") + "'"


def client_from_profile(profile):
    spec = importlib.util.spec_from_file_location("oracle_ux_profile", Path(__file__).with_name("loxone-ux.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    validated = module.validate_profile(profile)
    token = validated["token_file"].read_text(encoding="utf-8-sig").strip()
    if not re.fullmatch(r"[A-Za-z0-9._~+/\-]+=*", token):
        raise ValueError("Invalid private MCP token file.")
    client = MCPClient(validated["mcp_url"], token)
    client.initialize()
    return client, validated


def execute(args):
    root = args.artifact_root.resolve()
    definition = args.case.resolve()
    fixture = args.fixture.resolve()
    definition.relative_to(root)
    definition_bytes = definition.read_bytes()
    case = validate_case(parse_json(definition_bytes.decode("utf-8-sig")), fixture)
    validate_readback_calibration(case, root)
    if args.ocr_profile:
        validate_ocr_profile(args.ocr_profile, root)
    if args.observer_mode == "native-win32" and (
        args.ocr_profile is None or not read_json(args.ocr_profile).get("header_model_path")
    ):
        raise ValueError("Native-only observation requires the pinned English clock/header OCR profile.")
    transfer_profile = read_json(args.transfer_profile)
    transfer = Transfer(transfer_profile)
    client, profile = client_from_profile(read_json(args.profile))
    name = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-") + uuid.uuid4().hex
    local = root / ".reverse-engineering" / "native-simulator" / "replay" / name
    local.mkdir(parents=True, exist_ok=False)
    remote = PureWindowsPath(args.remote_root) / name
    if not re.fullmatch(r"[A-Za-z]:\\[A-Za-z0-9_.\\ -]+", str(remote)):
        raise ValueError("Remote root must be a plain absolute Windows directory.")
    request = {
        "schema_version": 1, "run_dir": str(remote), "mode": args.mode,
        "run_id": name,
        "fixture_filename": "input-" + name + ".Loxone",
        "application": {**case["application"], "path": args.application},
        "fixture_sha256": case["fixture_sha256"],
        "sampling_phases": sampling_plan(case),
        "timeout_seconds": 300,
        "ux_backend": profile["backend_path"],
        "instrumentation_start": args.instrumentation_start,
        "capture_scope": args.capture_scope,
        "observer_mode": args.observer_mode,
        "recovery_policy": args.recovery_policy,
        "controller_host": os.environ.get("COMPUTERNAME") or socket.gethostname(),
        "expected_desktop_host": transfer_profile["desktop_host"],
    }
    if "wait_seconds" in case:
        request["wait_seconds"] = case["wait_seconds"]
    if "simulator_start_time" in case:
        request["simulator_start_time"] = case["simulator_start_time"]
        request["simulator_time_edit"] = case.get("simulator_time_edit", "value-pattern")
    if args.mode == "drcov":
        if not args.instrumentation_profile:
            raise ValueError("drcov mode requires the independently verified launcher profile.")
        request["instrumentation"] = read_json(args.instrumentation_profile)
        shutil.copyfile(args.instrumentation_profile, local / "instrumentation-profile.json")
    if args.observer_mode == "native-win32":
        if args.exit_dump_profile is None:
            raise ValueError("Native Win32 observation requires the console-verified exit-and-dump profile.")
        request["exit_dump_method"] = read_json(args.exit_dump_profile)
        shutil.copyfile(args.exit_dump_profile, local / "exit-dump-profile.json")
    write_json(local / "request.json", request)
    project = local / fixture_basename(request)
    shutil.copyfile(fixture, project)
    if digest(project) != case["fixture_sha256"]:
        raise ValueError("Fixture changed while preparing the immutable run copy.")
    with (local / "case.definition.json").open("xb") as snapshot:
        snapshot.write(definition_bytes)
    client.powershell(
        "$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';"
        f"if(Test-Path -LiteralPath {ps_path(remote)}){{throw 'Run already exists'}};"
        f"New-Item -ItemType Directory -Path {ps_path(remote)}|Out-Null;"
        "@{created=$true}|ConvertTo-Json -Compress", 30,
    )
    for source in (local / "request.json", project,
                   Path(__file__).with_name("official_sim_runner.py"), Path(__file__).with_suffix(".ps1")):
        transfer.push(source, remote / source.name)
    transport_error = None
    try:
        reply = client.powershell(
            "$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';"
            f". {ps_path(remote / 'official-sim.ps1')};"
            f"Invoke-OfficialReplay -RequestPath {ps_path(remote / 'request.json')}", request["timeout_seconds"] - 30,
        )
        write_json(local / "transport-reply.json", reply)
    except UxError as error:
        transport_error = {"code": error.code, "message": str(error)}
        write_json(local / "transport-failure.json", transport_error)
        try:
            cleanup = client.powershell(
                "$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';"
                f". {ps_path(remote / 'official-sim.ps1')};"
                f"Invoke-OfficialReplayRecovery -RequestPath {ps_path(remote / 'request.json')}|ConvertTo-Json -Depth 10 -Compress", 70,
            )
            write_json(local / "recovery-reply.json", cleanup)
        except UxError as recovery_error:
            write_json(local / "recovery-failure.json", {"code": recovery_error.code, "message": str(recovery_error)})
    finally:
        # A transport failure never implies that input did not execute remotely.
        transfer.fetch(remote, local / "remote")
    result = finalize(local, root, case, request, transport_error)
    if args.ocr_profile:
        return admit_receipt(root / result[0]["receipt"], root, args.ocr_profile)
    return result


def finalize(local, root, case, request, transport_error=None):
    remote_dir = local / "remote"
    report_path = remote_dir / "remote-report.json"
    if report_path.exists():
        report = read_json(report_path)
    else:
        owner = remote_dir / "owned.json"
        recovery = local / "recovery-reply.json"
        report = {
            "native_running_verified": False, "observations": [],
            "error": transport_error or {"message": "Native execution report is missing; startup/progress/output are unverified."},
            "cleanup": read_json(recovery) if recovery.exists() else {"status": "unverified"},
            "target": read_json(owner) if owner.exists() else None,
            "timing": "No native timing or output success was inferred from transport/launcher completion.",
        }
    observations = []
    for sample in report["observations"]:
        observation = dict(sample)
        sources = []
        for path in sample["source"]["artifacts"]:
            source = (remote_dir / path).resolve()
            source.relative_to(remote_dir.resolve())
            if not source.is_file():
                raise ValueError("Native output source artifact was not fetched.")
            sources.append(str(source.relative_to(root)))
        observation["source"] = {
            **sample["source"],
            "artifacts": sources,
        }
        observations.append(observation)
    report["observations"] = observations
    status, reason = judge(case, report)
    artifacts = [artifact(path, root) for path in local.rglob("*") if path.is_file()]
    artifacts.extend(artifact(path, root, "native-raster-calibration") for path in validate_readback_calibration(case, root))
    receipt = {
        "schema_version": 1, "kind": "official-native-replay", "run_id": local.name,
        "artifact_root": str(Path(os.path.relpath(root, local))), "application": case["application"],
        "case": {
            "id": case["id"], "sha256": digest(local / "case.definition.json"),
            "definition": artifact(local / "case.definition.json", root),
            "fixture_sha256": case["fixture_sha256"],
        },
        "execution": {
            "mode": request["mode"], "status": status, "run_id": local.name,
            "instrumentation_start": request.get("instrumentation_start", "launch"),
            "capture_scope": request.get("capture_scope", "application"),
            "observer_mode": request.get("observer_mode", "uia"),
            "recovery_policy": request.get("recovery_policy", "block"),
            "native_running_verified": report["native_running_verified"] is True,
            "reason": reason, "cleanup": report["cleanup"],
        },
        "inputs": case.get("inputs", {}), "settings": case.get("settings", {}),
        "observations": [sample for sample in observations if sample.get("outputs") and sample["phase"] in {
            phase["name"] for phase in sampling_plan(case)
        }],
        "initial_state": next((sample for sample in observations if sample["phase"] == "initial_running"), None),
        "native_progress": observations, "artifacts": artifacts,
        "coverage_logs": [str(path.relative_to(root)) for path in remote_dir.rglob("*.log") if path.parent.name == "coverage"],
        "timing": report["timing"], "remote": {"run_dir": request["run_dir"], "target": report["target"]},
        "attachment": report.get("attachment"),
        "observer": {
            "mode": report.get("observer_mode", "uia"),
            "pre_attach_running_verified": report.get("pre_attach_running_verified", False),
            "post_attach_clock_proof_pending": report.get("post_attach_clock_proof_pending", False),
            "coverage_scope": report.get("coverage_scope"),
        },
    }
    write_json(local / "receipt.json", receipt)
    return {"status": status, "reason": reason, "receipt": str((local / "receipt.json").relative_to(root))}, 0 if status == "pass" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path)
    parser.add_argument("--transfer-profile", type=Path)
    parser.add_argument("--case", type=Path)
    parser.add_argument("--fixture", type=Path)
    parser.add_argument("--ocr-profile", type=Path)
    parser.add_argument("--admit-receipt", type=Path, help="File-only OCR admission; never touches a desktop")
    parser.add_argument("--artifact-root", type=Path, default=Path.cwd())
    parser.add_argument("--remote-root", default=r"C:\ProgramData\LoxOracle\native-replay")
    parser.add_argument("--application", default=r"C:\Program Files (x86)\Loxone\LoxoneConfig\LoxoneConfig.exe")
    parser.add_argument("--mode", choices=("plain", "drcov"), default="plain")
    parser.add_argument("--instrumentation-profile", type=Path)
    parser.add_argument("--instrumentation-start", choices=("launch", "post-init", "post-start"), default="launch")
    parser.add_argument("--capture-scope", choices=("application", "canvas"), default="application")
    parser.add_argument("--observer-mode", choices=("uia", "native-win32"), default="uia")
    parser.add_argument("--exit-dump-profile", type=Path)
    parser.add_argument("--recovery-policy", choices=("block", "owned-skip"), default="block",
                        help="Opt in only to declining the exact owned fixture's reviewed startup recovery question")
    args = parser.parse_args()
    if args.admit_receipt:
        if args.ocr_profile is None:
            parser.error("--admit-receipt requires --ocr-profile")
    elif any(value is None for value in (args.profile, args.transfer_profile, args.case, args.fixture)):
        parser.error("Replay requires --profile, --transfer-profile, --case and --fixture")
    elif args.instrumentation_start in ("post-init", "post-start") and args.mode != "drcov":
        parser.error("Post-init/post-start instrumentation requires --mode drcov")
    elif args.observer_mode == "native-win32" and (
        args.mode != "drcov" or args.instrumentation_start != "post-start" or args.capture_scope != "canvas"
    ):
        parser.error("--observer-mode native-win32 requires post-start drcov and canvas capture")
    elif args.instrumentation_start == "post-start" and args.observer_mode != "native-win32":
        parser.error("Post-start attachment requires the explicit native-win32 observer")
    try:
        output, code = (
            admit_receipt(args.admit_receipt, args.artifact_root.resolve(), args.ocr_profile)
            if args.admit_receipt else execute(args)
        )
    except (OSError, ValueError, RuntimeError, ImportError, subprocess.SubprocessError, UxError) as error:
        output, code = {"status": "blocked", "reason": str(error)}, 2
    print(json.dumps(output, ensure_ascii=True, separators=(",", ":")))
    return code


if __name__ == "__main__":
    sys.exit(main())
