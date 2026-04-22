#!/usr/bin/env python3
"""Fetch Loxone product datasheets (PDFs) and extract content via docint.

Downloads PDF datasheets referenced in KB articles, extracts text and
tables using Azure Document Intelligence (docint CLI), and stores
the extracted content as markdown alongside the PDFs.

Usage:
    python3 scripts/fetch-datasheets.py                  # extract all
    python3 scripts/fetch-datasheets.py --limit 5        # first 5 only
    python3 scripts/fetch-datasheets.py --download-only  # skip extraction

Configuration:
    Requires docint CLI configured at ~/.config/docint/config.json
    with Azure Document Intelligence endpoint and API key.
    See: .openclaw/workspace/skills/docint/SKILL.md

    Alternatively, set environment variables or use .env file:
      DOCINT_ENDPOINT=https://your-instance.cognitiveservices.azure.com
      DOCINT_API_KEY=your-api-key
"""

import argparse
import json
import os
import re
import subprocess
import sys
import time
import urllib.request
import urllib.error

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)


def load_env(env_file: str = ".env"):
    """Load environment variables from .env file if it exists."""
    path = os.path.join(REPO_DIR, env_file)
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, val = line.split("=", 1)
                    os.environ.setdefault(key.strip(), val.strip())


def collect_pdf_urls(kb_dir: str) -> list[dict]:
    """Collect PDF URLs from all KB markdown files."""
    pdfs = []
    seen_urls = set()

    for md_file in sorted(os.listdir(kb_dir)):
        if not md_file.endswith(".md"):
            continue
        slug = md_file[:-3]
        filepath = os.path.join(kb_dir, md_file)
        with open(filepath) as f:
            content = f.read()

        # Find all PDF URLs
        for url in re.findall(r'https?://[^\s\)\"]+\.pdf', content):
            if url in seen_urls:
                continue
            seen_urls.add(url)

            # Classify
            is_datasheet = "datasheet" in url.lower() or "Datasheet" in url
            is_loxone = "loxone.com" in url or "updatefiles.loxone.com" in url

            if not is_loxone:
                continue  # skip third-party PDFs

            # Extract a name from the URL
            fname = url.rsplit("/", 1)[-1]
            name = fname.replace(".pdf", "").replace("_", " ").replace("-", " ")

            pdfs.append({
                "url": url,
                "filename": fname,
                "name": name,
                "is_datasheet": is_datasheet,
                "source_article": slug,
            })

    return pdfs


def download_pdf(url: str, out_path: str) -> bool:
    """Download a PDF file."""
    if os.path.exists(out_path):
        return True
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "lox-doc-scraper/1.0"})
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = resp.read()
        with open(out_path, "wb") as f:
            f.write(data)
        return True
    except Exception as e:
        print(f"  ✗ Download failed: {e}", file=sys.stderr)
        return False


def extract_with_docint(pdf_path: str, output_path: str) -> bool:
    """Extract content from PDF using docint CLI."""
    if os.path.exists(output_path):
        return True
    try:
        result = subprocess.run(
            ["docint", "layout", "--output", "markdown", pdf_path],
            capture_output=True, text=True, timeout=60,
        )
        if result.returncode == 0 and result.stdout.strip():
            with open(output_path, "w") as f:
                f.write(result.stdout)
            return True
        else:
            print(f"  ✗ docint failed: {result.stderr[:100]}", file=sys.stderr)
            return False
    except subprocess.TimeoutExpired:
        print(f"  ✗ docint timeout", file=sys.stderr)
        return False
    except FileNotFoundError:
        print("  ✗ docint CLI not found. Install it or set DOCINT_ENDPOINT.", file=sys.stderr)
        return False


def main():
    parser = argparse.ArgumentParser(description="Fetch and extract Loxone datasheets")
    parser.add_argument("--limit", type=int, help="Max PDFs to process")
    parser.add_argument("--download-only", action="store_true", help="Skip docint extraction")
    parser.add_argument("--datasheets-only", action="store_true", help="Only process datasheets")
    parser.add_argument("--out", default=os.path.join(REPO_DIR, "docs", "datasheets"),
                        help="Output directory")
    parser.add_argument("--kb-dir", default=os.path.join(REPO_DIR, "docs", "kb"),
                        help="KB markdown directory")
    args = parser.parse_args()

    load_env()

    os.makedirs(args.out, exist_ok=True)
    pdfs_dir = os.path.join(args.out, "pdfs")
    extracted_dir = os.path.join(args.out, "extracted")
    os.makedirs(pdfs_dir, exist_ok=True)
    os.makedirs(extracted_dir, exist_ok=True)

    # Collect URLs
    print("Collecting PDF URLs from KB articles...")
    pdfs = collect_pdf_urls(args.kb_dir)
    if args.datasheets_only:
        pdfs = [p for p in pdfs if p["is_datasheet"]]
    print(f"Found {len(pdfs)} Loxone PDFs ({sum(1 for p in pdfs if p['is_datasheet'])} datasheets)")

    if args.limit:
        pdfs = pdfs[:args.limit]

    # Process each
    downloaded = 0
    extracted = 0
    index = {}

    for i, pdf in enumerate(pdfs):
        fname = pdf["filename"]
        pdf_path = os.path.join(pdfs_dir, fname)
        md_path = os.path.join(extracted_dir, fname.replace(".pdf", ".md"))

        # Download
        if download_pdf(pdf["url"], pdf_path):
            downloaded += 1
            pdf["local_path"] = f"datasheets/pdfs/{fname}"
        else:
            continue

        # Extract
        if not args.download_only:
            if extract_with_docint(pdf_path, md_path):
                extracted += 1
                pdf["extracted_path"] = f"datasheets/extracted/{fname.replace('.pdf', '.md')}"
                print(f"  [{i+1}/{len(pdfs)}] ✓ {pdf['name']}")
            else:
                print(f"  [{i+1}/{len(pdfs)}] ⚠ {pdf['name']} (download OK, extraction failed)")
            # Be polite to Azure
            time.sleep(1)
        else:
            print(f"  [{i+1}/{len(pdfs)}] ↓ {pdf['name']}")

        index[fname] = pdf

    # Write index
    index_path = os.path.join(args.out, "index.json")
    with open(index_path, "w") as f:
        json.dump(index, f, indent=2, sort_keys=True)

    print(f"\n✓ {downloaded} downloaded, {extracted} extracted → {args.out}/")
    print(f"  Index: {index_path}")


if __name__ == "__main__":
    main()
