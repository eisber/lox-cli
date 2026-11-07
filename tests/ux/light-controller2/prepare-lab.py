"""Prepare authored lighting in a pinned public sample envelope; never drives a desktop.

The former tiny synthetic envelope did not save natively. Keep the public
default user/permission and periphery structures intact, including their public
demo password metadata: stripping it can save once but fails native validation
after reopening. This is not a production connection or a native parity claim.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[3]
PIN = "f2244a8abcce44cf51955f2f3a4dd30c2aeb4a7c8b52ad14ee0ebb751e49ef99"
ORIGINAL_FIXTURE_SHA256 = "33dbc7a366e2017ab7f74842b206daf08ecec1ac16dae5188c017857a2b5948f"
PUBLIC_ENVELOPE = ROOT / "tests" / "eval" / "golden" / "configs" / "Hotwater_Heating_Rod.Loxone"
PUBLIC_ENVELOPE_SHA256 = "01bdfd27e0d1cf526e1c5bd49784750df22f639f513ae809e8c5c6511b4bac94"
BLOCK_TITLE = "Lighting Slice Fixture 20261106"
PAGE_TITLE = "Lighting Slice Page"
ROOM_TITLE = "Lighting Slice Room"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def native_id(index):
    return f"1b1e0000-0000-{index:04x}-ffff000000000000"


def seed(envelope=PUBLIC_ENVELOPE):
    data = envelope.read_bytes()
    if hashlib.sha256(data).hexdigest() != PUBLIC_ENVELOPE_SHA256:
        raise ValueError("Only the pinned public Hotwater sample may supply the envelope.")
    root = ET.fromstring(data)
    document = root.find("C")
    document.set("Title", "Disposable Lighting Slice Lab 20261106")
    categories = next(n for n in document if n.get("Type") == "CategoryCaption")
    ET.SubElement(categories, "C", Type="Category", V="175", U=native_id(4),
                  Title="Lighting Lab", WF="16384")
    places = next(n for n in document if n.get("Type") == "PlaceCaption")
    ET.SubElement(places, "C", Type="Place", V="175", U=native_id(6),
                  Title=ROOM_TITLE, WF="16384")
    server = next(n for n in document if n.get("Type") == "LoxLIVE")
    server.set("IntAddr", "")
    server.set("ExtAddr", "")
    for node in list(document):
        if node.get("Type") == "Program":
            document.remove(node)
    program = ET.SubElement(document, "C", Type="Program", V="175", U=native_id(8),
                            Title="Lighting Slice Program", Ref=server.get("U"), WF="16384")
    page = ET.SubElement(program, "C", Type="Page", V="175", U=native_id(9),
                         Title=PAGE_TITLE, WF="16384")
    ET.SubElement(page, "IoData", Pr=native_id(6), Cr=native_id(4))
    identities = {n.get("U") for n in root.iter() if n.get("U")}
    for node in root.iter():
        source = node.get("Input")
        if source and re.fullmatch(r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{16}", source, re.I) and source not in identities:
            del node.attrib["Input"]
    document.set("NumO", str(len(list(root.iter("C")))))
    ET.indent(root, space="\t")
    text = '<?xml version="1.0" encoding="utf-8"?>\n' + ET.tostring(root, encoding="unicode")
    return b"\xef\xbb\xbf" + (text + "\n").replace("\n", "\r\n").encode("utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cli", type=Path,
        default=ROOT / ".reverse-engineering" / "native-simulator" / "workstreams"
        / "lighting-build" / "debug" / "lox.exe",
    )
    parser.add_argument(
        "--output", type=Path,
        default=ROOT / ".reverse-engineering" / "native-simulator" / "workstreams"
        / "lighting-preparation" / "batch-01",
    )
    args = parser.parse_args()
    output = args.output.resolve()
    private_root = ROOT / ".reverse-engineering" / "native-simulator" / "workstreams"
    if not output.is_relative_to(private_root.resolve()):
        raise ValueError("Disposable output must remain in the private workstreams directory.")
    if output.exists():
        raise ValueError("Refusing to overwrite an existing preparation/baseline.")
    if digest(args.cli) != PIN:
        raise ValueError("CLI hash changed; freeze/review a new preparation instead of reusing this pin.")
    original_fixture = Path(__file__).with_name("synthetic.Loxone")
    if digest(original_fixture) != ORIGINAL_FIXTURE_SHA256:
        raise ValueError("The original independent fixture changed; do not relabel its old hash.")
    output.mkdir(parents=True)
    project = output / "Lighting-Slice-Lab-20261106.Loxone"
    project.write_bytes(seed())
    commands = []

    def run(*arguments, writes=True, response="json"):
        command = [str(args.cli.resolve()), "config", *map(str, arguments), "-o", "json"]
        before = digest(project)
        result = subprocess.run(command, capture_output=True, text=True, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(f"Preparation failed: {arguments[0]}: {result.stderr}")
        value = json.loads(result.stdout) if response == "json" else result.stdout
        if isinstance(value, dict) and value.get("ok") is False:
            raise RuntimeError(f"Preparation returned unsuccessful data: {arguments[0]}")
        if not writes and digest(project) != before:
            raise ValueError(f"Read-only preparation step changed the project: {arguments[0]}")
        commands.append({
            "argv": command, "writes_disposable_project": writes,
            "before_sha256": before, "after_sha256": digest(project),
            "response_format": response, "stdout": result.stdout, "stderr": result.stderr,
        })
        return value

    def find(typ, title):
        root = ET.fromstring(project.read_bytes().decode("utf-8-sig"))
        matches = [element for element in root.iter("C")
                   if element.get("Type") == typ and element.get("Title") == title]
        if len(matches) != 1:
            raise ValueError(f"Expected one authored {typ} named {title}")
        return matches[0]

    run("add", project, "--type", "LightController2", "--title", BLOCK_TITLE,
        "--room", ROOM_TITLE, "--category", "Lighting Lab", "--page", PAGE_TITLE)
    light = find("LightController2", BLOCK_TITLE)
    block_uuid = light.attrib["U"]
    # Existing add-mood operates on a real LightscenesC tag, not a generic C Type tag.
    tree = ET.fromstring(project.read_bytes().decode("utf-8-sig"))
    light = next(element for element in tree.iter("C") if element.get("U") == block_uuid)
    ET.SubElement(light, "LightscenesC", Outputs="18", Num="0")
    ET.indent(tree, space="\t")
    project.write_bytes(b"\xef\xbb\xbf" + (
        '<?xml version="1.0" encoding="utf-8"?>\n' + ET.tostring(tree, encoding="unicode") + "\n"
    ).replace("\n", "\r\n").encode("utf-8"))
    scaffolding_sha256 = digest(project)
    for index in [1, 2, 10]:
        run("xml", "set-attr", project, f"uuid:{block_uuid}",
            f"NameAI{index - 1}", f"Lab channel storage {index}", response="text")
        title = f"Lighting Q{index} Observer"
        run("add", project, "--type", "StateV", "--title", title,
            "--room", ROOM_TITLE, "--page", PAGE_TITLE)
        observer = find("StateV", title)
        light = find("LightController2", BLOCK_TITLE)
        source = next(co for co in light.findall("Co") if co.get("K") == f"AQ{index}")
        run("wire-connector", project, f"uuid:{observer.attrib['U']}.I", source.attrib["U"])
    for name, sid, cid, value, q2, q10 in [
        ("Lab On", 777, 7, 73, 79, 83),
        ("Lab Off", 778, 0, 0, 0, 0),
        ("Slice A", 2, 9, 11, 37, 63),
        ("Slice B", 3, 10, 19, 41, 67),
    ]:
        run("add-mood", project, f"uuid:{block_uuid}", "--name", name,
            "--color", value, "--sid", sid, "--cid", cid)
        for index, raw in [(2, q2), (10, q10)]:
            run("set-mood-color", project, f"uuid:{block_uuid}", "--mood", sid,
                "--output-index", index, "--color", raw)
    run("layout", project, response="text")
    tree = ET.fromstring(project.read_bytes().decode("utf-8-sig"))
    tree.find("C").set("NumO", str(len(list(tree.iter("C")))))
    ET.indent(tree, space="\t")
    project.write_bytes(b"\xef\xbb\xbf" + (
        '<?xml version="1.0" encoding="utf-8"?>\n' + ET.tostring(tree, encoding="unicode") + "\n"
    ).replace("\n", "\r\n").encode("utf-8"))
    run("validate", project, writes=False)
    inspection = run("lighting", "inspect", project, f"uuid:{block_uuid}", writes=False)
    if inspection["gui_verified"] or inspection["status"] != "read_only_unverified":
        raise ValueError("Preparation must not promote a native capability.")
    tree = ET.fromstring(project.read_bytes().decode("utf-8-sig"))
    identities = {}
    for element in tree.iter():
        for key in ["U", "UUID"]:
            if key in element.attrib:
                value = element.attrib[key]
                if value in identities:
                    raise ValueError(f"Duplicate authored identity: {value}")
                identities[value] = element.tag
    for element in tree.iter():
        for key in ["Input", "Pr", "Cr", "Ref"]:
            if key in element.attrib and element.attrib[key] not in identities:
                raise ValueError(f"Unresolved authored reference: {key}")
    project_bytes = project.read_bytes()
    if not project_bytes.startswith(b"\xef\xbb\xbf") or b"\n" in project_bytes.replace(b"\r\n", b""):
        raise ValueError("BOM/CRLF preservation failed.")
    if digest(original_fixture) != ORIGINAL_FIXTURE_SHA256:
        raise ValueError("Old fixture was modified.")
    manifest = {
        "schema_version": 1,
        "kind": "lighting-disposable-project-preparation",
        "native_loadability": "pending-fresh-GUI-open-not-verified-offline",
        "native_execution_count": 0,
        "desktop_call_count": 0,
        "cli": {"path": str(args.cli.resolve()), "sha256": PIN},
        "generator": {"path": str(Path(__file__).resolve()), "sha256": digest(__file__)},
        "project": {
            "path": str(project), "sha256": digest(project),
            "scaffolding_sha256": scaffolding_sha256,
            "independently_authored": False, "authored_lighting_program": True,
            "contains_production_data": False,
            "public_envelope": {
                "path": str(PUBLIC_ENVELOPE.relative_to(ROOT)),
                "sha256": PUBLIC_ENVELOPE_SHA256,
                "public_demo_password_metadata_preserved": True,
                "connection_addresses_cleared": True,
                "exact_new_output_native_roundtrip": "pending-not-verified-by-template-control",
            },
            "document_uuid": tree.find("C").get("U"), "page_uuid": native_id(9),
            "block_uuid": block_uuid, "block_title": BLOCK_TITLE,
            "connector_count": len(find("LightController2", BLOCK_TITLE).findall("Co")),
            "mood_count": len(inspection["moods"]),
            "identities_unique": True, "references_resolved": True, "bom": True, "crlf": True,
        },
        "existing_fixture_sha256_unchanged": ORIGINAL_FIXTURE_SHA256,
        "commands": commands,
        "limits": "Names are raw NameAI metadata; no CapAI/role/encoding interpretation, native pass or new edit capability.",
    }
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    (output / "inspection.json").write_text(json.dumps(inspection, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"manifest": str(output / "manifest.json"), "project": manifest["project"]}, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, ET.ParseError) as error:
        print(f"Lighting preparation aborted: {error}", file=sys.stderr)
        raise SystemExit(1) from error
