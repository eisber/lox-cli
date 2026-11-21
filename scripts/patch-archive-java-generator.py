"""Fail-closed, archive-contract-specific correction for Java generator 0.5.2."""

import argparse
import hashlib
import json
from pathlib import Path

SPEC_PATH = (
    Path(__file__).resolve().parent.parent
    / "crates" / "lox-bindings" / "patches"
    / "uniffi-bindgen-java-0.5.2-rustbuffer-free.json"
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def correct(generated_dir, generator_version, receipt_path):
    spec = json.loads(SPEC_PATH.read_text(encoding="utf-8"))
    if generator_version != spec["generator_version"]:
        raise ValueError("Unreviewed Java generator version; regenerate and review the correction.")
    target = generated_dir.joinpath(*spec["target"].split("/"))
    companion = generated_dir.joinpath(*spec["companion"].split("/"))
    original = target.read_bytes()
    if digest(original) != spec["original_sha256"]:
        raise ValueError("Unexpected RustBuffer.java (including already corrected output); refusing edits.")
    companion_original = companion.read_bytes()
    if digest(companion_original) != spec["companion_sha256"]:
        raise ValueError("Unexpected UniffiHelpers.java; re-review status/cleanup interaction.")
    old = spec["old_method"].encode("utf-8")
    new = spec["new_method"].encode("utf-8")
    if original.count(old) != 1:
        raise ValueError("Expected exactly one reviewed RustBuffer.free method.")
    corrected = original.replace(old, new, 1)
    if digest(corrected) != spec["corrected_sha256"]:
        raise ValueError("Corrected source fingerprint mismatch; refusing edits.")
    companion_corrected = companion_original
    for edit in spec["companion_edits"]:
        old_fragment = edit["old"].encode("utf-8")
        if companion_corrected.count(old_fragment) != edit["count"]:
            raise ValueError("Unexpected call-status lifetime source; refusing edits.")
        companion_corrected = companion_corrected.replace(old_fragment, edit["new"].encode("utf-8"))
    if digest(companion_corrected) != spec["companion_corrected_sha256"]:
        raise ValueError("Corrected call-status fingerprint mismatch; refusing edits.")
    target.write_bytes(corrected)
    companion.write_bytes(companion_corrected)
    receipt = {
        "patch_id": spec["patch_id"],
        "generator_version": generator_version,
        "patch_spec_sha256": digest(SPEC_PATH.read_bytes()),
        "target": spec["target"],
        "original_sha256": digest(original),
        "corrected_sha256": digest(corrected),
        "companion": spec["companion"],
        "companion_original_sha256": digest(companion_original),
        "companion_corrected_sha256": digest(companion_corrected),
        "scope": "Preserve RustBuffer descriptor; deterministically close owned wait-lane status; GC-managed status on other caller threads. No ABI/checksum change.",
    }
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--generated-dir", type=Path, required=True)
    parser.add_argument("--generator-version", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    correct(args.generated_dir, args.generator_version, args.receipt)
