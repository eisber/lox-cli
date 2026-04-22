#!/usr/bin/env python3
"""Download Loxone use-case config files (.Loxone).

The use-cases.json references pimcore.loxone.com URLs which have no public
DNS. The actual CDN host is pim.loxone.com with path prefix /var/assets/.
We rewrite URLs accordingly and download via simple HTTP requests.

Output: /tmp/use-cases/<filename>.Loxone
"""

import json
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

import requests

USE_CASES_JSON = Path(__file__).resolve().parent.parent / "tests" / "eval" / "use-cases.json"
OUT_DIR = Path("/tmp/use-cases")


def load_use_cases() -> list[dict]:
    with open(USE_CASES_JSON) as f:
        return json.load(f)


def rewrite_url(url: str) -> str:
    """Rewrite pimcore.loxone.com URL → pim.loxone.com/var/assets/..."""
    parsed = urlparse(url)
    if parsed.hostname == "pimcore.loxone.com":
        return url.replace("https://pimcore.loxone.com/", "https://pim.loxone.com/var/assets/")
    return url


def main():
    use_cases = load_use_cases()
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    # Collect all (url, filename) tuples
    downloads: list[tuple[str, str]] = []
    for uc in use_cases:
        for cf in uc.get("config_files", []):
            url = cf["url"]
            fname = cf.get("name") or unquote(urlparse(url).path.split("/")[-1])
            downloads.append((url, fname))

    print(f"Found {len(downloads)} config files to download\n")

    session = requests.Session()
    succeeded = 0
    failed = 0
    skipped = 0

    for i, (orig_url, fname) in enumerate(downloads, 1):
        dest = OUT_DIR / fname
        if dest.exists() and dest.stat().st_size > 100:
            print(f"[{i}/{len(downloads)}] SKIP (exists): {fname}")
            skipped += 1
            continue

        url = rewrite_url(orig_url)
        print(f"[{i}/{len(downloads)}] {fname}")

        try:
            resp = session.get(url, timeout=30)
            if resp.ok and len(resp.content) > 100:
                dest.write_bytes(resp.content)
                size_kb = len(resp.content) / 1024
                print(f"    ✓ {size_kb:.0f} KB")
                succeeded += 1
            else:
                print(f"    ✗ HTTP {resp.status_code}, {len(resp.content)} bytes")
                failed += 1
        except Exception as e:
            print(f"    ✗ {e}")
            failed += 1

    print(f"\n{'='*50}")
    print(f"Results: {succeeded} downloaded, {skipped} skipped (existed), {failed} failed")
    total = len(list(OUT_DIR.glob("*.[Ll]oxone")))
    print(f"Total files in {OUT_DIR}: {total}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
