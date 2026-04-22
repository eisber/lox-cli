#!/usr/bin/env python3
"""Scrape all Loxone use cases from the official website.

Uses Playwright to load https://www.loxone.com/dede/use-cases/ and intercepts
the JSON data that the React app fetches. Saves structured results to
tests/eval/use-cases.json.
"""

import json
import re
import sys
from html import unescape
from pathlib import Path
from playwright.sync_api import sync_playwright

SITE_URL = "https://www.loxone.com/dede/use-cases/"
UPLOADS_BASE = "https://www.loxone.com/dede/wp-content/uploads/sites/2"
PIMCORE_BASE = "https://pimcore.loxone.com"
OUTPUT_PATH = Path(__file__).resolve().parent.parent / "tests" / "eval" / "use-cases.json"


def strip_html(html: str | None) -> str:
    """Remove HTML tags and decode entities, returning plain text."""
    if not html:
        return ""
    text = re.sub(r"<[^>]+>", "", html)
    text = unescape(text)
    return text.strip()


def extract_category(node: dict) -> str:
    """Derive category from the fullpath field (e.g. '/06 Use Cases/Climate/...')."""
    fp = node.get("fullpath") or ""
    parts = fp.split("/")
    if len(parts) > 2:
        return parts[2].lower()
    return "other"


def extract_secondary_categories(node: dict) -> list[str]:
    cats = node.get("secondaryUseCaseCategory") or []
    return [c.lower() for c in cats]


def build_config_file_url(fullpath: str) -> str:
    """Build download URL from pimcore fullpath."""
    return f"{PIMCORE_BASE}{fullpath}"


def transform_use_case(node: dict) -> dict:
    """Transform a raw JSON node into the output schema."""
    config_files = []
    for cf in node.get("configFiles") or []:
        elem = cf.get("element") or {}
        if elem.get("filename"):
            config_files.append({
                "name": elem["filename"],
                "url": build_config_file_url(elem.get("fullpath", "")),
            })

    function_blocks = []
    for fb in node.get("functionBlocksList") or []:
        block = fb.get("functionBlocks") or {}
        if block.get("name"):
            entry = {"name": block["name"]}
            if block.get("url"):
                url = block["url"]
                if not url.startswith("http"):
                    url = f"https://{url}"
                entry["url"] = url
            function_blocks.append(entry)

    required_products = []
    for rp in node.get("requiredProductsList") or []:
        prod = rp.get("requiredProducts") or {}
        if prod.get("name"):
            required_products.append({
                "name": prod["name"],
                "sku": prod.get("sku", ""),
            })

    return {
        "id": node.get("id"),
        "name": node.get("useCaseName") or "",
        "name_en": node.get("enUseCaseName") or "",
        "category": extract_category(node),
        "secondary_categories": extract_secondary_categories(node),
        "description": strip_html(node.get("shortDescription")),
        "implementation": strip_html(node.get("implementationDescription")),
        "implementation_html": node.get("implementationDescription") or "",
        "function_blocks": function_blocks,
        "auto_config": node.get("autoConfig") is not None,
        "config_version": node.get("configVersion") or "",
        "config_files": config_files,
        "required_products": required_products,
        "url_slug": node.get("url") or "",
        "remarks": strip_html(node.get("remarks")),
    }


def main():
    print(f"Scraping use cases from {SITE_URL} ...")

    captured_data = []

    def on_response(response):
        if "useCases.json" in response.url:
            try:
                captured_data.append(response.json())
                print(f"  ✓ Intercepted JSON from {response.url}")
            except Exception as e:
                print(f"  ✗ Failed to parse JSON from {response.url}: {e}")

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(
            user_agent=(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
                "(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
            ),
            locale="de-DE",
        )
        page = context.new_page()
        page.on("response", on_response)

        page.goto(SITE_URL, wait_until="networkidle", timeout=30000)
        page.wait_for_timeout(5000)
        browser.close()

    if not captured_data:
        print("ERROR: No use case JSON data intercepted.", file=sys.stderr)
        sys.exit(1)

    raw_items = captured_data[0]
    print(f"  Raw items from JSON: {len(raw_items)}")

    use_cases = []
    for item in raw_items:
        node = item.get("node", item)
        use_cases.append(transform_use_case(node))

    # Sort by category then name for stable output
    use_cases.sort(key=lambda uc: (uc["category"], uc["name_en"] or uc["name"]))

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_PATH, "w", encoding="utf-8") as f:
        json.dump(use_cases, f, indent=2, ensure_ascii=False)

    print(f"\n  Saved {len(use_cases)} use cases to {OUTPUT_PATH}")

    # --- Summary ---
    categories = {}
    with_config = 0
    with_blocks = 0
    with_autoconfig = 0
    with_products = 0
    for uc in use_cases:
        cat = uc["category"]
        categories[cat] = categories.get(cat, 0) + 1
        if uc["config_files"]:
            with_config += 1
        if uc["function_blocks"]:
            with_blocks += 1
        if uc["auto_config"]:
            with_autoconfig += 1
        if uc["required_products"]:
            with_products += 1

    print(f"\n{'='*60}")
    print(f"  Total use cases:       {len(use_cases)}")
    print(f"  With config files:     {with_config}")
    print(f"  With function blocks:  {with_blocks}")
    print(f"  With autoConfig:       {with_autoconfig}")
    print(f"  With required products:{with_products}")
    print(f"\n  Categories:")
    for cat in sorted(categories):
        print(f"    {cat:20s} {categories[cat]:3d}")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
