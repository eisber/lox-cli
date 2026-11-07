#!/usr/bin/env python3
"""Generate factual built-in lookup metadata from a supplied IconLibrary.zip.

No artwork, localized tags, absolute input paths or executable identity is emitted.
This is exact loaded-catalog membership, not a Qt SVG rendering oracle.
"""

import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET
import zipfile


NATIVE_ID = re.compile(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{16}\Z")


def inventory(archive):
    original = Path(archive).read_bytes()
    with zipfile.ZipFile(io.BytesIO(original)) as source:
        names = source.namelist()
        if len(set(names)) != len(names):
            raise ValueError("Duplicate archive entry names")
        catalogs = sorted(n for n in names if re.fullmatch(r"IconLibrary(?:_[A-Z]{3})?\.xml", n))
        if "IconLibrary.xml" not in catalogs:
            raise ValueError("Missing canonical fallback IconLibrary.xml")
        version = source.read("version").decode("ascii").strip()
        if not re.fullmatch(r"[0-9]+", version):
            raise ValueError("Catalog version must be an independent decimal archive version")
        identities = None
        loaded = []
        for catalog in catalogs:
            root = ET.fromstring(source.read(catalog))
            if root.tag != "Icons":
                raise ValueError(f"{catalog}: unsupported root")
            rows, seen_ids, seen_names = [], set(), set()
            for icon in root:
                if icon.tag != "Icon" or len(icon):
                    raise ValueError(f"{catalog}: unsupported catalog child")
                reference = icon.get("uuid", "")
                if not NATIVE_ID.fullmatch(reference) or int(reference.replace("-", ""), 16) == 0:
                    raise ValueError(f"{catalog}: malformed or zero icon identity")
                reference = reference.lower()
                if reference in seen_ids:
                    raise ValueError(f"{catalog}: duplicate icon identity")
                seen_ids.add(reference)
                filename = icon.get("Id", "")
                filled = icon.get("filled", "false")
                if filled not in ("true", "false"):
                    raise ValueError(f"{catalog}: unsupported filled value")
                if filename:
                    if (not filename.endswith(".svg") or "/" in filename or "\\" in filename
                            or any(ord(char) < 32 for char in filename)):
                        raise ValueError(f"{catalog}: invalid canonical filename")
                    if filename in seen_names:
                        raise ValueError(f"{catalog}: duplicate canonical filename")
                    seen_names.add(filename)
                rows.append((reference, filename, icon.get("Idx"), filled))
                if catalog == "IconLibrary.xml" and filename and filled == "true":
                    member = "IconsFilled/" + filename
                    if member not in names or not source.read(member):
                        raise ValueError(f"{catalog}: missing or empty loaded asset {member}")
                    loaded.append({"native_id": reference, "filename": filename})
            identity = sorted(rows)
            if identities is not None and identity != identities:
                raise ValueError(f"{catalog}: localized catalog identity differs from fallback")
            identities = identity
        if not loaded:
            raise ValueError("No loaded filled icons")
        return {
            "schema_version": 1,
            "catalog": {
                "version": version,
                "archive_sha256": hashlib.sha256(original).hexdigest(),
            },
            "icons": sorted(loaded, key=lambda row: row["native_id"]),
        }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = inventory(args.archive)
        if args.archive.resolve() == args.output.resolve():
            raise ValueError("Output must not overwrite the source archive")
        args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
    except (OSError, ValueError, KeyError, zipfile.BadZipFile, ET.ParseError) as error:
        parser.exit(2, f"Icon inventory failed: {error}\n")
    print(json.dumps({"catalog": result["catalog"], "loaded_icons": len(result["icons"])}))


if __name__ == "__main__":
    main()
