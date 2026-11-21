"""Bounded file-only hybrid selection checks, never desktop authority or input."""

from datetime import datetime, timezone
import hashlib
import io
import math
import re
from types import SimpleNamespace


METHOD = "hybrid-image-inspector-v1"
STRATEGY = "exact-stable-body-v1"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def image_decoder():
    try:
        from PIL import Image
    except ImportError as error:
        raise ValueError("Optional Pillow dependency is required before hybrid launch.") from error
    return Image


def image(raw):
    Image = image_decoder()
    require(len(raw) <= 32 * 1024 * 1024, "Oversized owned image.")
    with Image.open(io.BytesIO(raw)) as source:
        require(source.format == "PNG" and 0 < source.width * source.height <= 16_000_000,
                "A bounded original PNG is required.")
        source.verify()
    with Image.open(io.BytesIO(raw)) as source:
        require(source.mode in {"RGB", "RGBA"}, "Only RGB or opaque RGBA PNG is admitted.")
        if source.mode == "RGBA":
            require(source.getextrema()[3] == (255, 255), "Transparent capture is unqualified.")
        return source.convert("RGB")


def rectangle(value):
    require(isinstance(value, list) and len(value) == 4
            and all(type(v) in (int, float) and math.isfinite(v) for v in value)
            and value[2] > 0 and value[3] > 0, "Invalid measured rectangle.")
    return value


def scoped(node, pattern):
    if not isinstance(node, dict) or not isinstance(pattern, dict):
        return False
    selector, ancestors = pattern.get("selector", {}), pattern.get("ancestors", [])
    actual = node.get("ancestors", [])
    return (bool(selector) and bool(ancestors) and len(actual) >= len(ancestors)
            and all(node.get("selector", {}).get(key) == value for key, value in selector.items())
            and all(all(actual[index].get(key) == value for key, value in item.items())
                    for index, item in enumerate(ancestors)))


def exact_matches(source, template, box):
    """Return at most two exact RGB candidates; never rank fuzzy candidates."""
    x, y, width, height = rectangle(box)
    require(all(type(v) is int for v in box) and x >= 0 and y >= 0
            and x + width <= source.width and y + height <= source.height
            and width * height <= 4_000_000, "Canvas pixel scope is unqualified.")
    tw, th = template.size
    require(2 <= tw <= width and 2 <= th <= height and tw * th <= 100_000,
            "Stable body template is unqualified.")
    rows = [source.crop((x, row, x + width, row + 1)).tobytes()
            for row in range(y, y + height)]
    needle = [template.crop((0, row, tw, row + 1)).tobytes() for row in range(th)]
    candidates = []
    for row in range(height - th + 1):
        offset = 0
        while True:
            found = rows[row].find(needle[0], offset)
            if found < 0:
                break
            offset = found + 1
            if found % 3 or found // 3 + tw > width:
                continue
            if all(rows[row + index][found:found + tw * 3] == needle[index]
                   for index in range(1, th)):
                candidates.append([x + found // 3, y + row, tw, th])
                if len(candidates) > 1:
                    return candidates
    return candidates


def click_point(match, specification, transform):
    x, y, width, height = match
    box, offset = specification.get("safe_click_box"), specification.get("safe_click_offset")
    require(isinstance(box, list) and len(box) == 4 and all(type(v) is int for v in box)
            and 0 <= box[0] < box[2] <= width and 0 <= box[1] < box[3] <= height
            and isinstance(offset, list) and len(offset) == 2
            and all(type(v) is int for v in offset)
            and box[0] <= offset[0] < box[2] and box[1] <= offset[1] < box[3],
            "Reviewed template-relative safe click box/offset is missing or invalid.")
    sx, sy, tx, ty = transform
    point = [round(tx + (x + offset[0]) * sx), round(ty + (y + offset[1]) * sy)]
    require(tx + (x + box[0]) * sx <= point[0] < tx + (x + box[2]) * sx
            and ty + (y + box[1]) * sy <= point[1] < ty + (y + box[3]) * sy,
            "Transformed click is outside its reviewed safe body region.")
    return point


def read_bound(executor, at=None):
    descriptors = executor.handoff.get("execution_binding")
    require(isinstance(descriptors, dict)
            and set(descriptors) == {"effect_authority", "runtime_binding", "selector_evidence"},
            "Immutable reviewed execution binding is missing.")
    require(hashlib.sha256(executor.canonical_json(descriptors)).hexdigest()
            == executor.binding_anchor, "Mutable binding cannot replace the journal anchor.")
    values = {}
    for role, descriptor in descriptors.items():
        path = executor.queue_tool().checked_artifact(executor.root, descriptor)
        raw = path.read_bytes()
        require(hashlib.sha256(raw).hexdigest() == descriptor["sha256"],
                "Immutable execution binding changed during read.")
        values[role] = executor.structured_json(raw)
    grant, runtime, evidence = [values[key] for key in (
        "effect_authority", "runtime_binding", "selector_evidence")]
    for key in ("run_id", "worker", "physical_identity_sha256", "application"):
        require(grant.get(key) == executor.context[key]
                and runtime.get(key) == executor.context[key], "Execution binding scope changed.")
    require(grant.get("definition_sha256") == executor.case["definition_sha256"]
            and grant.get("resources") == executor.context["resources"]
            and grant.get("scope") == "native-property-execution",
            "Reviewed effect authority scope is missing.")
    expiry = datetime.fromisoformat(grant["valid_until"])
    require(expiry.tzinfo is not None and expiry > (at or datetime.now(timezone.utc)),
            "Reviewed effect authority is stale.")
    require(runtime.get("method") == METHOD and evidence.get("method") == METHOD
            and evidence.get("strategy") == STRATEGY
            and evidence.get("mutable_title_excluded") is True
            and runtime.get("effect_authority") == descriptors["effect_authority"]
            and runtime.get("selector_evidence") == descriptors["selector_evidence"]
            and runtime.get("authority_source_sha256") == executor.handoff["authority_source_sha256"]
            and runtime.get("document_path") == executor.handoff["document_path"]
            and runtime.get("executable_path") == executor.handoff["executable_path"],
            "Runtime/selector evidence does not bind the sealed handoff.")
    source_pin = evidence.get("capture_source_sha256")
    require(isinstance(source_pin, str) and len(source_pin) == 64
            and all(char in "0123456789abcdef" for char in source_pin),
            "Reviewed capture source RAW pin is missing.")
    methods = evidence.get("capture_methods")
    require(isinstance(methods, list) and 1 <= len(methods) <= 4
            and all(isinstance(item, str) and item for item in methods)
            and len(set(methods)) == len(methods), "Reviewed native capture methods are missing.")
    require(evidence.get("native_result_contract", {}).get("version")
            == "printwindow-native-result-v1", "Reviewed raw native-result contract is missing.")
    return runtime, evidence


def preflight(executor):
    image_decoder()
    runtime, evidence = read_bound(executor)
    verifier = getattr(executor.authority, "preflight", None)
    require(callable(verifier), "Reviewed real effect-authority verifier is missing.")
    descriptor = verifier(dict(executor.context), executor.handoff["execution_binding"])
    raw = executor.queue_tool().checked_artifact(executor.root, descriptor).read_bytes()
    receipt = executor.structured_json(raw)
    require(all(receipt.get(key) == value for key, value in executor.context.items())
            and receipt.get("execution_binding") == executor.handoff["execution_binding"]
            and receipt.get("outcome") == "available", "Real effect-authority preflight refused.")
    timestamp = datetime.fromisoformat(receipt["source_utc"])
    require(timestamp.tzinfo is not None, "Effect preflight has no timezone.")
    age = (datetime.now(timezone.utc) - timestamp).total_seconds()
    require(0 <= age <= 5, "Effect preflight is stale.")
    executor.artifact("hybrid-preflight.json", raw, "effect-authority-receipt")
    template_provenance(executor, evidence)
    return runtime, evidence


def template_provenance(executor, evidence):
    body, label = evidence["body_template"], evidence["baseline_label"]
    require(body["source_image"] == label["source_image"],
            "Body and baseline label must come from the same reviewed frame.")
    canvas = evidence["baseline_canvas_crop"]
    require(len(canvas) == 4 and all(type(v) is int for v in canvas),
            "Reviewed baseline canvas association is missing.")
    for item in (body, label):
        box = item["crop"]
        require(canvas[0] <= box[0] < box[2] <= canvas[2]
                and canvas[1] <= box[1] < box[3] <= canvas[3],
                "Body/baseline label crop lies outside the reviewed canvas.")
    a, b = body["crop"], label["crop"]
    require(a[2] <= b[0] or b[2] <= a[0] or a[3] <= b[1] or b[3] <= a[1],
            "Stable body overlaps the mutable baseline Title label.")
    # These are reviewed source crops, not learned/rebound templates.
    for role in ("body_template", "baseline_label", "inspector_label_template"):
        item = evidence[role]
        original = image(executor.bound_bytes(item["source_image"]))
        template = image(executor.bound_bytes(item["template"]))
        if role == "body_template":
            click_point([0, 0, template.width, template.height], item, (1, 1, 0, 0))
        if role == "body_template":
            require(0 <= canvas[0] < canvas[2] <= original.width
                    and 0 <= canvas[1] < canvas[3] <= original.height,
                    "Reviewed baseline canvas is outside the original frame.")
        box = item["crop"]
        require(len(box) == 4 and all(type(v) is int for v in box)
                and 0 <= box[0] < box[2] <= original.width
                and 0 <= box[1] < box[3] <= original.height
                and original.crop(tuple(box)).size == template.size
                and original.crop(tuple(box)).tobytes() == template.tobytes(),
                "Reviewed original/crop/template provenance mismatch.")


def mapped_frame(metadata, identity, display_hash, size):
    mapping = metadata["mapping"]
    require(metadata.get("identity") == identity and mapping.get("identity") == identity
            and mapping.get("display_inventory_sha256") == display_hash,
            "Image root/display calibration changed.")
    transform = mapping.get("pixel_to_physical")
    require(isinstance(transform, list) and len(transform) == 6
            and all(type(v) in (int, float) and math.isfinite(v) for v in transform)
            and transform[1] == transform[3] == 0 and transform[0] == transform[4]
            and 0 < transform[0] <= 8, "Missing measured axis-aligned image calibration.")
    sx, _, tx, _, sy, ty = transform
    proof = mapping.get("calibration", {})
    require(proof.get("image_pixels") == [[0, 0], list(size)]
            and proof.get("physical_points") == [[tx, ty],
                                                [tx + size[0] * sx, ty + size[1] * sy]]
            and proof.get("method") == "measured-owned-image-v1",
            "Image endpoint calibration is missing or contradictory.")
    return sx, sy, tx, ty


def owned_image(executor, metadata, identity, evidence):
    require(metadata.get("capture_source_sha256") == evidence["capture_source_sha256"]
            and metadata.get("native_method") in evidence["capture_methods"]
            and metadata.get("native_success") is True,
            "Reviewed owned-window capture method/success is unproved.")
    ack = executor.structured_json(executor.bound_bytes(metadata["capture_ack"]))
    value = ack.get("structuredContent", {}).get("result")
    match = re.fullmatch(r"Response: (.*)\r?\nStatus Code:\s*(-?\d+)\s*",
                         value or "", re.DOTALL)
    require(ack.get("isError") is False and ack.get("completion") not in {"UNKNOWN", "pending"}
            and match is not None and int(match[2]) == 0,
            "Owned capture actual SDK status is not zero.")
    native = executor.structured_json(match[1])
    require(isinstance(native, dict) and bool(native), "Owned capture returned no native metadata.")
    raw = executor.bound_bytes(metadata["image"])
    source = image(raw)
    transform = mapped_frame(metadata, identity, executor.display_hash, source.size)
    sx, sy, tx, ty = transform
    require(metadata.get("window_bounds") == [tx, ty, source.width * sx, source.height * sy],
            "Owned image does not cover its measured window bounds.")
    contract = evidence["native_result_contract"]
    require(contract.get("version") == "printwindow-native-result-v1"
            and contract.get("native_method") == "PrintWindow2",
            "Reviewed native-result contract is missing or unsupported.")
    def field(path):
        require(isinstance(path, list) and 1 <= len(path) <= 12
                and all(type(key) in (str, int) for key in path), "Invalid reviewed native field path.")
        value = native
        for key in path:
            value = value[key]
        return value
    fields = contract["capture"]
    require(all(path[0] == "capture" for path in fields.values()),
            "Reviewed capture fields must refer to the raw capture object.")
    require(field(fields["sha256"]) == hashlib.sha256(raw).hexdigest()
            and field(fields["returned"]) is True
            and field(fields["method"]) == "PrintWindow"
            and type(field(fields["flags"])) is int and field(fields["flags"]) == 2
            and type(field(fields["width"])) is int and field(fields["width"]) == source.width
            and type(field(fields["height"])) is int and field(fields["height"]) == source.height
            and field(fields["pixel_to_physical"]) == metadata["mapping"]["pixel_to_physical"]
            and metadata["native_method"] == contract["native_method"],
            "Raw native capture hash/method/success/geometry contradicts normalized metadata.")
    for phase in ("before", "after"):
        paths = contract[phase]
        require(all(path[0] == phase for path in paths.values()),
                "Reviewed native before/after fields cannot alias the other phase.")
        start = field(paths["start"])
        if contract.get("start_encoding") == "kernel-utc-100ns":
            stamp = re.fullmatch(r"(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)\.(\d{7})(?:Z|\+00:00)", start)
            require(stamp is not None, "Native creation timestamp lacks exact 100ns UTC precision.")
            seconds = datetime.fromisoformat(stamp[1]).replace(tzinfo=timezone.utc)
            epoch = datetime(1601, 1, 1, tzinfo=timezone.utc)
            delta = seconds - epoch
            ticks = (delta.days * 86400 + delta.seconds) * 10_000_000 + int(stamp[2])
            start = {"kind": "kernel-100ns", "value": str(ticks)}
        else:
            require(contract.get("start_encoding") == "identity-object",
                    "Unsupported reviewed native start identity encoding.")
        require(type(field(paths["pid"])) is int and field(paths["pid"]) == identity["pid"]
                and type(field(paths["root"])) is int and field(paths["root"]) == identity["root"]
                and start == identity["start_identity"]
                and field(paths["session"]) == identity["session"]
                and field(paths["executable"]) == identity["path"]
                and field(paths["exe_sha256"]) == identity["application"]["sha256"]
                and field(paths["project"]) == executor.handoff["document_path"]
                and field(paths["calibration_ok"]) is True
                and field(paths["physical_rect"]) == [tx, ty, tx + source.width * sx,
                                                      ty + source.height * sy],
                "Raw native before/after process/root/start/project/calibration contradicts metadata.")
    return source, raw, transform


def capture(executor, facts, identity, *, bound_evidence=None):
    evidence = bound_evidence if bound_evidence is not None else read_bound(executor)[1]
    metadata = facts["owned_capture"]
    require(metadata.get("method") == "reviewed-owned-window-png-v1",
            "An explicit owned-window read-only capture is required.")
    _, raw, _ = owned_image(executor, metadata, identity, evidence)
    return raw, metadata


def select(executor, facts, identity, *, bound_evidence=None):
    evidence = bound_evidence if bound_evidence is not None else read_bound(executor)[1]
    metadata = facts["hybrid"]
    require(metadata.get("method") == METHOD
            and metadata.get("expected_title") == executor.expected_title
            and metadata.get("baseline_sha256") == executor.case["fixture_sha256"]
            and metadata.get("capture_source_sha256") == evidence["capture_source_sha256"],
            "Image phase/baseline/source binding changed.")
    canvas = metadata["canvas"]
    require(scoped(canvas, evidence["canvas"]) and canvas.get("visible") is True
            and bool(canvas.get("runtime_id"))
            and metadata.get("hit_canvas_runtime_id") == canvas["runtime_id"],
            "Native canvas scope is not proved.")
    source, raw, transform = owned_image(executor, metadata, identity, evidence)
    template_raw = executor.bound_bytes(evidence["body_template"]["template"])
    template = image(template_raw)
    sx, sy, tx, ty = transform
    left, top, width, height = rectangle(canvas["physical_bounds"])
    pixel_box = [(left - tx) / sx, (top - ty) / sy, width / sx, height / sy]
    require(all(v == int(v) for v in pixel_box), "Canvas/image edge calibration is nonintegral.")
    matches = exact_matches(source, template, [int(v) for v in pixel_box])
    require(len(matches) == 1, "Rendered body candidate is absent or ambiguous.")
    x, y, tw, th = matches[0]
    point = click_point(matches[0], evidence["body_template"], transform)
    require(left <= point[0] < left + width and top <= point[1] < top + height
            and tx + x * sx <= point[0] < tx + (x + tw) * sx
            and ty + y * sy <= point[1] < ty + (y + th) * sy
            and facts.get("hit") == {"physical_point": point, "identity": identity},
            "Physical hit is not inside the measured body and canvas.")
    crop = io.BytesIO()
    source.crop((x, y, x + tw, y + th)).save(crop, format="PNG")
    prefix = "hybrid-" + str(len(executor.steps))
    executor.artifact(prefix + "-original.png", raw, "selection-image")
    executor.artifact(prefix + "-capture-ack.json", executor.bound_bytes(metadata["capture_ack"]),
                      "owned-capture-ack")
    executor.artifact(prefix + "-template.png", template_raw, "selection-template")
    executor.artifact(prefix + "-crop.png", crop.getvalue(), "selection-derived-crop")
    executor.artifact(prefix + "-match.json", executor.canonical_json({
        "method": METHOD, "strategy": STRATEGY, "expected_title": executor.expected_title,
        "identity": identity, "image": metadata["image"], "mapping": metadata["mapping"],
        "canvas": canvas, "candidate_pixels": matches[0], "physical_point": point,
        "block_uia_identity_claim": False,
    }), "hybrid-selection")
    return point


def inspector(executor, facts, identity, *, focused=False, bound_evidence=None, retain=True):
    evidence = bound_evidence if bound_evidence is not None else read_bound(executor)[1]
    observed = facts["inspector"]
    patterns = evidence["inspector"]
    headers = [node for node in observed["selected_headers"]
               if scoped(node, patterns["selected_header"])]
    require(observed.get("identity") == identity
            and observed.get("expected_title") == executor.expected_title
            and len(headers) == 1
            and patterns.get("xml_type") == executor.case["property"]["type"],
            "Selected native inspector type/header changed.")
    editors = [node for node in observed["editors"] if scoped(node, patterns["editor"])]
    require(1 <= len(editors) <= 128, "Native inspector editor census is missing or oversized.")
    row_index = patterns.get("row_ancestor_index")
    require(type(row_index) is int and 0 <= row_index < len(patterns["editor"]["ancestors"]),
            "The reviewed native row ancestor is missing.")
    source, source_raw, transform = owned_image(
        executor, {**observed, "image": observed["label_source_image"]}, identity, evidence)
    expected_label = image(executor.bound_bytes(evidence["inspector_label_template"]["template"]))
    sx, sy, tx, ty = transform
    qualified = []
    for candidate in editors:
        candidate_row = candidate["ancestors"][row_index]
        left, top, width, height = rectangle(candidate_row["physical_bounds"])
        box = [(left - tx) / sx, (top - ty) / sy, width / sx, height / sy]
        require(all(v == int(v) for v in box), "Inspector row/image calibration is nonintegral.")
        matches = exact_matches(source, expected_label, [int(v) for v in box])
        require(len(matches) <= 1, "Rendered inspector row label is ambiguous.")
        if matches:
            qualified.append((candidate, matches[0]))
    require(len(qualified) == 1, "Native editor ancestor/rendered-label scope is ambiguous.")
    editor, matched_label = qualified[0]
    row = editor["ancestors"][row_index]
    require(editor.get("visible") is True and editor.get("enabled") is True
            and editor.get("read_only") is False
            and editor.get("method") == "read-only-ValuePattern"
            and editor.get("value") == executor.expected_title
            and (not focused or editor.get("has_keyboard_focus") is True),
            "Native inspector writable value/focus does not match the declared phase.")
    require(observed.get("label_row_runtime_id") == row.get("runtime_id")
            and isinstance(patterns.get("label_semantic"), str) and bool(patterns["label_semantic"])
            and isinstance(patterns.get("native_locale"), str) and bool(patterns["native_locale"])
            and patterns.get("xml_attribute") == executor.case["property"]["attribute"],
            "Reviewed rendered label is not bound to the current editor row.")
    label = image(executor.bound_bytes(observed["label_image"]))
    crop = observed["label_crop"]
    x, y, width, height = matched_label
    require(crop == [x, y, x + width, y + height],
            "Current label crop differs from the independently matched native row.")
    require(len(crop) == 4 and all(type(v) is int for v in crop)
            and 0 <= crop[0] < crop[2] <= source.width
            and 0 <= crop[1] < crop[3] <= source.height
            and source.crop(tuple(crop)).size == label.size
            and source.crop(tuple(crop)).tobytes() == label.tobytes(),
            "Current label crop is not from its retained owned image.")
    left, top, width, height = rectangle(row["physical_bounds"])
    require(left <= tx + crop[0] * sx < tx + crop[2] * sx <= left + width
            and top <= ty + crop[1] * sy < ty + crop[3] * sy <= top + height,
            "Rendered label is not inside the current native inspector row.")
    require(label.size == expected_label.size and label.tobytes() == expected_label.tobytes(),
            "Current native inspector label image does not match reviewed evidence.")
    executor.correlate_xml(executor.saved_xml if executor.fresh else executor.before_xml,
                           executor.case["property"]["after"] if executor.fresh
                           else executor.case["property"]["before"])
    if retain:
        prefix = "inspector-" + str(len(executor.steps))
        executor.artifact(prefix + "-original.png", source_raw, "inspector-image")
        executor.artifact(prefix + "-capture-ack.json",
                          executor.bound_bytes(observed["capture_ack"]), "owned-capture-ack")
        executor.artifact(prefix + "-label.png", executor.bound_bytes(observed["label_image"]),
                          "inspector-label-crop")
        executor.artifact(prefix + "-binding.json", executor.canonical_json(observed),
                          "inspector-proof")
    return editor


def validate_capture(case, result, root, events, facts, handoff_event, queue,
                     before, saved, by_kind, xml_reader):
    """Recompute hybrid proofs from retained bytes; never run the live authority."""
    binding = result.get("execution_binding")
    require(queue.definition_hash(binding) == handoff_event.get("execution_binding_sha256"),
            "Collected effect/runtime/selector binding is not the sealed handoff.")
    context = {key: result[key] for key in (
        "run_id", "worker", "physical_identity_sha256", "definition_sha256",
        "application", "resources", "input_deadline_utc")}
    adapter = SimpleNamespace(
        root=root, case=case, context=context, before_xml=before, saved_xml=saved, fresh=None,
        handoff={key: result[key] for key in (
            "execution_binding", "authority_source_sha256", "executable_path", "document_path")},
        queue_tool=lambda: queue, structured_json=queue.read_json,
        canonical_json=queue.canonical, steps=[], expected_title=case["property"]["before"],
        binding_anchor=handoff_event["execution_binding_sha256"],
    )
    from loxone_ux_transport import parse_json
    adapter.structured_json = parse_json
    def bound_bytes(descriptor):
        raw = queue.checked_artifact(root, descriptor).read_bytes()
        require(hashlib.sha256(raw).hexdigest() == descriptor["sha256"],
                "Collected hybrid artifact changed during read.")
        return raw
    adapter.bound_bytes = bound_bytes
    def correlate(raw, title):
        document, _ = xml_reader(raw)
        matches = [node for node in document.iter() if node.get("Title") == title]
        require(len(matches) == 1 and matches[0].get("Type") == case["property"]["type"]
                and matches[0].get("U") == case["property"]["uuid"],
                "Collected global Title/type/UUID correlation failed.")
    adapter.correlate_xml = correlate
    correlate(before, case["property"]["before"])
    correlate(saved, case["property"]["after"])
    # Audit at the retained run time, not against today's grant expiry.
    original_now = datetime.fromisoformat(result["source_utc"])
    _, evidence = read_bound(adapter, at=original_now)
    template_provenance(adapter, evidence)
    preflights = by_kind.get("effect-authority-receipt", [])
    require(len(preflights) == 1, "Real preflight receipt is missing.")
    preflight_receipt = parse_json(preflights[0].read_bytes())
    require(preflight_receipt.get("execution_binding") == binding
            and preflight_receipt.get("outcome") == "available"
            and all(preflight_receipt.get(key) == value for key, value in context.items()),
            "Collected preflight scope differs from the immutable binding.")
    recorded_matches = [parse_json(path.read_bytes()) for path in by_kind.get("hybrid-selection", [])]
    checkpoints = {event["facts"]["sha256"]: event["seq"] for event in events
                   if event.get("run_id") == result["run_id"] and event["type"] == "checkpoint"}
    receipt_hashes = {queue.definition_hash(parse_json(path.read_bytes())):
                      hashlib.sha256(path.read_bytes()).hexdigest()
                      for path in by_kind["authority-receipt"]}
    type_intents = [event for event in events if event.get("run_id") == result["run_id"]
                    and event["type"] == "step-intent" and event["intent"]["tool"] == "Type"]
    require(len(type_intents) == 1, "Exactly one declared Type is required.")
    targets, types = 0, 0
    for receipt in facts:
        intent = receipt.get("input")
        if not intent:
            continue
        native = receipt["facts"]
        expected_arguments = dict(intent["arguments"])
        receipt_sha = receipt_hashes[queue.definition_hash(receipt)]
        checkpoint = checkpoints.get(receipt_sha)
        following = next((event for event in events
                          if checkpoint is not None and event["seq"] > checkpoint
                          and event.get("run_id") == result["run_id"]
                          and event["type"] == "step-intent"), None)
        require(following is not None and following.get("effect") is True
                and following["intent"]["tool"] == intent["tool"],
                "Hybrid input is not the next actual dispatch after its checkpoint.")
        observed_at = datetime.fromisoformat(receipt["source_utc"])
        dispatched_at = datetime.fromisoformat(following["time"])
        require(observed_at.tzinfo is not None and 0 <=
                (dispatched_at - observed_at).total_seconds() <= 5,
                "Hybrid input used stale native observation.")
        identity = native["identity"]
        adapter.fresh = result["fresh_identity"] if identity == result["fresh_identity"] else None
        require(identity in (result["old_identity"], result["fresh_identity"]),
                "Hybrid input identity is not this run's old/fresh process.")
        expected = case["property"]["after"] if adapter.fresh or intent["tool"] == "Shortcut" else case["property"]["before"]
        require(receipt.get("expected_title") == expected
                and receipt.get("selection_method") == METHOD,
                "Old/fresh input phase was rebound or failed to propagate AFTER.")
        adapter.expected_title = expected
        adapter.display_hash = receipt["display_inventory_sha256"]
        require(native.get("target_visible") is True
                and native.get("foreground_visible") is True
                and receipt.get("control_state") in {"ready", "ai"},
                "Hybrid input visibility/control state is unproved.")
        if intent["selector"] == "target":
            foreground = native.get("foreground", {})
            require(foreground == identity or (
                type(foreground.get("pid")) is int and foreground["pid"] > 0
                and foreground["pid"] != identity["pid"]
                and type(foreground.get("root")) is int and foreground["root"] > 0
                and foreground.get("session") == identity["session"]),
                "Hybrid pointer foreground relationship is ambiguous.")
            outputs = []
            adapter.artifact = lambda name, raw, kind: outputs.append((kind, raw))
            # read_bound uses actual time during execution; this audit uses the
            # already checked historical binding and does not renew authority.
            point = select(adapter, native, identity, bound_evidence=evidence)
            expected_arguments["loc"] = point
            metadata = next(parse_json(raw) for kind, raw in outputs if kind == "hybrid-selection")
            require(metadata in recorded_matches, "Recomputed exact image match was not retained.")
            targets += 1
        else:
            require(native.get("foreground") == identity,
                    "Collected keyboard/property input foreground changed.")
            editor = inspector(adapter, native, identity, focused=intent["tool"] == "Type",
                               bound_evidence=evidence, retain=False)
            if intent["selector"] in {"property", "value"}:
                left, top, width, height = rectangle(editor["physical_bounds"])
                point = [round(left + width / 2), round(top + height / 2)]
                require(native.get("hit") == {"physical_point": point, "identity": identity},
                        "Inspector actual physical hit differs from its scoped native field.")
                expected_arguments["loc"] = point
            if intent["tool"] == "Type":
                require(checkpoints.get(receipt_sha, type_intents[0]["seq"]) < type_intents[0]["seq"]
                        and intent["arguments"] == {
                            "text": case["property"]["after"], "clear": True, "press_enter": True},
                        "Inspector proof is not before the exact declared Type.")
                types += 1
            if intent["tool"] in {"Type", "Shortcut"}:
                focus = native.get("input_focus", {})
                require(focus.get("identity") == identity and type(focus.get("hwnd")) is int
                        and focus["hwnd"] > 0 and focus.get("thread_active_root") == identity["root"]
                        and focus.get("visible") is True and focus.get("enabled") is True,
                        "Collected true keyboard focus is missing.")
        require(following["intent"].get("arguments") == expected_arguments,
                "Actual dispatched input arguments differ from recomputed hybrid proof.")
    require(targets == 2 and types == 1, "Hybrid old/fresh selection or pre-Type proof is missing.")
    captures = [item for item in facts if item.get("stage") == "capture"]
    require(len(captures) == 4 and len(by_kind.get("owned-capture-proof", [])) == 4,
            "Owned lifecycle captures are missing.")
    retained_proofs = [parse_json(path.read_bytes()) for path in by_kind["owned-capture-proof"]]
    screenshots = {hashlib.sha256(path.read_bytes()).hexdigest() for path in by_kind["screenshot"]}
    for receipt in captures:
        identity = receipt["facts"]["identity"]
        require(identity in (result["old_identity"], result["fresh_identity"]),
                "Owned capture identity is not this run's old/fresh root.")
        adapter.display_hash = receipt["display_inventory_sha256"]
        raw, proof = capture(adapter, receipt["facts"], identity, bound_evidence=evidence)
        require(proof in retained_proofs and hashlib.sha256(raw).hexdigest() in screenshots,
                "Owned capture original/metadata was not retained.")
