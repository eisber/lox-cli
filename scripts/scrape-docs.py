#!/usr/bin/env python3
"""Scrape Loxone KB articles and convert to markdown.

Fetches all articles from loxone.com/enen/kb/, converts HTML to clean
markdown, and stores in docs/kb/. Builds an index mapping type slugs
to file paths.

Usage:
    python3 scripts/scrape-docs.py                    # English (default)
    python3 scripts/scrape-docs.py --lang dede        # German
    python3 scripts/scrape-docs.py --refresh           # re-fetch all
    python3 scripts/scrape-docs.py --slug switch       # single article
"""

import argparse
import html as htmlmod
import json
import os
import re
import sys
import time
import urllib.request
import urllib.error

LANG_MAP = {
    "enen": "English",
    "dede": "German (DE)",
    "enus": "English (US)",
    "frfr": "French",
    "itit": "Italian",
    "eses": "Spanish",
    "nlnl": "Dutch",
}

BASE_URL = "https://www.loxone.com/{lang}/kb/"
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)


def fetch(url: str, retries: int = 2) -> str:
    """Fetch a URL with retries."""
    for attempt in range(retries + 1):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "lox-doc-scraper/1.0"})
            with urllib.request.urlopen(req, timeout=15) as resp:
                return resp.read().decode("utf-8", errors="replace")
        except (urllib.error.URLError, OSError) as e:
            if attempt == retries:
                raise
            time.sleep(1)
    return ""


def fetch_binary(url: str, retries: int = 2) -> bytes:
    """Fetch binary content (images) with retries."""
    for attempt in range(retries + 1):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "lox-doc-scraper/1.0"})
            with urllib.request.urlopen(req, timeout=15) as resp:
                return resp.read()
        except (urllib.error.URLError, OSError):
            if attempt == retries:
                return b""
            time.sleep(1)
    return b""


def _download_image(url: str, images_dir: str, slug: str) -> str:
    """Download an image and return the relative path."""
    os.makedirs(images_dir, exist_ok=True)
    # Generate a filename from the URL
    fname = url.rsplit("/", 1)[-1]
    # Remove query strings
    fname = fname.split("?")[0]
    # Prefix with slug to avoid collisions
    local_name = f"{slug}_{fname}"
    local_path = os.path.join(images_dir, local_name)

    if not os.path.exists(local_path):
        data = fetch_binary(url)
        if data:
            with open(local_path, "wb") as f:
                f.write(data)

    if os.path.exists(local_path):
        return f"images/{local_name}"
    return ""


def describe_image(image_path: str) -> str:
    """Generate a text description of an image using OCR.
    Returns extracted text or empty string."""
    try:
        import subprocess
        result = subprocess.run(
            ["tesseract", image_path, "-", "--psm", "6"],
            capture_output=True, text=True, timeout=10,
        )
        text = result.stdout.strip()
        # Only return if we got meaningful text (>10 chars, not just noise)
        if len(text) > 10:
            return text
    except Exception:
        pass
    return ""


def discover_articles(lang: str) -> list[str]:
    """Find all KB article URLs from the KB index page."""
    base = BASE_URL.format(lang=lang)
    html = fetch(base)
    pattern = rf'href="(https://www\.loxone\.com/{lang}/kb/[^"]+)"'
    urls = sorted(set(re.findall(pattern, html)))
    # Filter out category pages (they have /kb-cat/ or end with /kb/)
    urls = [u for u in urls if "/kb-cat/" not in u and u.rstrip("/") != base.rstrip("/")]
    return urls


def html_to_markdown(html_content: str, download_images: bool = False,
                     images_dir: str = "", slug: str = "") -> str:
    """Convert HTML article content to clean markdown."""
    # Extract the actual content — try multiple patterns
    c = None
    for pattern in [
        r'<div class="pakb-content">\s*<body>(.*?)</body>',
        r'<article class="pakb-single">\s*<div class="pakb-content">(.*?)</div>\s*</article>',
        r'<div class="pakb-content">(.*?)</div>\s*</article>',
        r'<div class="entry-content">(.*?)</div>\s*(?:</article>|<footer)',
    ]:
        m = re.search(pattern, html_content, re.DOTALL)
        if m:
            c = m.group(1)
            break
    if not c:
        return ""

    # Remove search forms, breadcrumbs, and navigation noise
    c = re.sub(r'<div class="pakb-header">.*?</div>', '', c, flags=re.DOTALL)
    c = re.sub(r'<ul class="pakb-breadcrumb">.*?</ul>', '', c, flags=re.DOTALL)
    c = re.sub(r'<form[^>]*>.*?</form>', '', c, flags=re.DOTALL)
    c = re.sub(r'<!--.*?-->', '', c, flags=re.DOTALL)

    # Remove back-to-top links within headings
    c = re.sub(r'<a[^>]*href="#ToC"[^>]*>[^<]*</a>', '', c)

    # Convert images to proper markdown with full URLs
    # Skip tiny icons (width < 50) and UI chrome
    def _img_to_md(match):
        tag = match.group(0)
        alt = re.search(r'alt="([^"]*)"', tag)
        src = re.search(r'src="([^"]*)"', tag)
        width = re.search(r'width="(\d+)"', tag)
        alt_text = alt.group(1) if alt else ""
        src_url = src.group(1) if src else ""
        w = int(width.group(1)) if width else 999

        # Skip tiny icons, banners, chevrons
        if w < 50 or not src_url:
            return ""
        if "icon--close" in src_url or "arrow-down" in src_url:
            return ""

        # Clean alt text
        if not alt_text or alt_text in ("", " "):
            alt_text = src_url.rsplit("/", 1)[-1].rsplit(".", 1)[0].replace("-", " ").replace("_", " ")

        # Pick highest-quality src (remove size suffix from URL)
        src_url = re.sub(r'-\d+x\d+\.', '.', src_url)

        if download_images and images_dir and slug:
            local = _download_image(src_url, images_dir, slug)
            if local:
                src_url = local

        return f"\n![{alt_text}]({src_url})\n"

    c = re.sub(r'<img[^>]*/?\s*>', _img_to_md, c)

    # Convert tables to proper markdown tables
    c = _convert_tables(c)

    # Convert headings (strip IDs)
    c = re.sub(r'<h1[^>]*>(.*?)</h1>', r'\n# \1\n', c, flags=re.DOTALL)
    c = re.sub(r'<h2[^>]*>(.*?)</h2>', r'\n## \1\n', c, flags=re.DOTALL)
    c = re.sub(r'<h3[^>]*>(.*?)</h3>', r'\n### \1\n', c, flags=re.DOTALL)
    c = re.sub(r'<h4[^>]*>(.*?)</h4>', r'\n#### \1\n', c, flags=re.DOTALL)

    # Convert inline formatting
    c = re.sub(r'<strong>(.*?)</strong>', r'**\1**', c, flags=re.DOTALL)
    c = re.sub(r'<b>(.*?)</b>', r'**\1**', c, flags=re.DOTALL)
    c = re.sub(r'<em>(.*?)</em>', r'*\1*', c, flags=re.DOTALL)
    c = re.sub(r'<code>(.*?)</code>', r'`\1`', c, flags=re.DOTALL)

    # Convert links (strip empty ones)
    c = re.sub(r'<a[^>]*href="([^"]+)"[^>]*>(.*?)</a>', r'[\2](\1)', c, flags=re.DOTALL)

    # Convert lists — handle nesting
    # First convert nested <ul> inside <li> to indented sub-items
    c = re.sub(r'<li[^>]*>\s*(.*?)\s*<ul[^>]*>(.*?)</ul>\s*</li>',
               lambda m: '- ' + re.sub(r'<[^>]+>', '', m.group(1)).strip() + '\n' +
               re.sub(r'<li[^>]*>\s*(.*?)\s*</li>',
                      lambda m2: '  - ' + re.sub(r'<[^>]+>', '', m2.group(1)).strip() + '\n',
                      m.group(2), flags=re.DOTALL),
               c, flags=re.DOTALL)
    c = re.sub(r'<li[^>]*>\s*(.*?)\s*</li>', r'- \1\n', c, flags=re.DOTALL)
    c = re.sub(r'<[ou]l[^>]*>', '\n', c)
    c = re.sub(r'</[ou]l>', '\n', c)

    # Horizontal rules
    c = re.sub(r'<hr\s*/?>', '\n---\n', c)

    # Convert paragraphs and breaks
    c = re.sub(r'<p[^>]*>\s*</p>', '', c)  # remove empty paragraphs
    c = re.sub(r'<p[^>]*>(.*?)</p>', r'\1\n\n', c, flags=re.DOTALL)
    c = re.sub(r'<br\s*/?>', '\n', c)

    # Remove remaining HTML tags
    c = re.sub(r'<[^>]+>', '', c)

    # Decode HTML entities
    c = htmlmod.unescape(c)

    # Clean up whitespace
    c = re.sub(r'[ \t]+\n', '\n', c)  # trailing spaces
    c = re.sub(r'\n{3,}', '\n\n', c)  # collapse blank lines
    c = re.sub(r'^\s*-\s*\n\s*$', '', c, flags=re.MULTILINE)  # remove empty list items
    c = re.sub(r'\n- \n', '\n', c)  # remove empty list items
    # Clean up ToC-style lists (indented items with just links)
    c = re.sub(r'\n\s+- ', '\n- ', c)  # normalize list indentation
    # Remove leading whitespace on content lines (from HTML indentation)
    lines = []
    for line in c.split('\n'):
        stripped = line.rstrip()
        # Remove leading whitespace unless it's a list item or table/heading
        if stripped and stripped[0] == ' ':
            content = stripped.lstrip()
            # Only preserve indentation for sub-list items (  - item)
            if content.startswith('- ') and stripped.startswith('  - '):
                lines.append(stripped)
            elif content.startswith(('|', '#', '>', '!')):
                lines.append(stripped)
            else:
                lines.append(content)
        else:
            lines.append(stripped)
    c = '\n'.join(lines)
    c = re.sub(r'\n\n\n+', '\n\n', c)  # re-collapse after cleanup
    c = c.strip()

    return c


def _convert_tables(html: str) -> str:
    """Convert HTML tables to markdown tables."""
    def _table_to_md(match):
        table_html = match.group(0)
        rows = re.findall(r'<tr[^>]*>(.*?)</tr>', table_html, re.DOTALL)
        if not rows:
            return ''

        md_rows = []
        is_info_box = False

        for i, row in enumerate(rows):
            cells = re.findall(r'<t[hd][^>]*>(.*?)</t[hd]>', row, re.DOTALL)
            # Clean cell content: strip tags but preserve list items
            clean_cells = []
            for cell in cells:
                # Convert <li> to inline markers before stripping
                cell = re.sub(r'<li[^>]*>\s*', '• ', cell, flags=re.DOTALL)
                cell = re.sub(r'</li>', '', cell)
                cell = re.sub(r'<br\s*/?>', ' ', cell)
                cell = re.sub(r'<[^>]+>', '', cell)
                # Also strip markdown image syntax (from earlier image conversion)
                cell = re.sub(r'!\[[^\]]*\]\([^)]*\)', '', cell)
                cell = htmlmod.unescape(cell).strip().replace('\n', ' ')
                # Collapse multiple spaces
                cell = re.sub(r'\s+', ' ', cell)
                clean_cells.append(cell)

            if not clean_cells:
                continue

            # Detect info-box pattern: 2 cells where first contains an icon image
            if len(clean_cells) == 2 and (
                len(clean_cells[0]) < 5 or
                'info' in clean_cells[0].lower() or
                'exclamation' in clean_cells[0].lower() or
                clean_cells[0].startswith('!') or
                clean_cells[0] == ''
            ):
                is_info_box = True

            if is_info_box and len(clean_cells) == 2:
                # Render as blockquote instead of table
                return '\n> **ℹ️ Note:** ' + clean_cells[1] + '\n'

            md_rows.append('| ' + ' | '.join(clean_cells) + ' |')
            if i == 0:
                md_rows.append('| ' + ' | '.join(['---'] * len(clean_cells)) + ' |')

        return '\n' + '\n'.join(md_rows) + '\n'

    return re.sub(r'<table[^>]*>.*?</table>', _table_to_md, html, flags=re.DOTALL)


def extract_title(html_content: str) -> str:
    """Extract page title from the article heading or <title> tag."""
    # Try the h1 inside the article first
    m = re.search(r'<h1[^>]*class="entry-title[^"]*"[^>]*>(.*?)</h1>', html_content, re.DOTALL)
    if m:
        return htmlmod.unescape(re.sub(r'<[^>]+>', '', m.group(1)).strip())
    # Try the breadcrumb active item
    m = re.search(r'<li class="active">(.*?)</li>', html_content)
    if m:
        return htmlmod.unescape(m.group(1).strip())
    # Fallback to <title>
    m = re.search(r'<title>(.*?)</title>', html_content)
    if m:
        title = htmlmod.unescape(m.group(1))
        title = re.sub(r'\s*[-|–].*$', '', title)
        return title.strip()
    return ""


def scrape_article(url: str, download_images: bool = False,
                   images_dir: str = "") -> dict:
    """Scrape a single KB article."""
    try:
        html = fetch(url)
    except Exception as e:
        return {"url": url, "error": str(e)}

    title = extract_title(html)
    slug = url.rstrip("/").rsplit("/", 1)[-1]
    markdown = html_to_markdown(html, download_images=download_images,
                                images_dir=images_dir, slug=slug)

    if not markdown:
        return {"url": url, "title": title, "error": "no content extracted"}

    return {
        "url": url,
        "slug": slug,
        "title": title,
        "markdown": markdown,
    }


def main():
    parser = argparse.ArgumentParser(description="Scrape Loxone KB docs to markdown")
    parser.add_argument("--lang", default="enen", help="Language code (enen, dede, ...)")
    parser.add_argument("--refresh", action="store_true", help="Re-fetch all articles")
    parser.add_argument("--slug", help="Scrape a single article by slug")
    parser.add_argument("--out", default=os.path.join(REPO_DIR, "docs", "kb"),
                        help="Output directory")
    parser.add_argument("--limit", type=int, help="Max articles to scrape")
    parser.add_argument("--download-images", action="store_true",
                        help="Download images locally to docs/kb/images/")
    args = parser.parse_args()

    os.makedirs(args.out, exist_ok=True)
    images_dir = os.path.join(args.out, "images") if args.download_images else ""

    if args.slug:
        url = BASE_URL.format(lang=args.lang) + args.slug + "/"
        article = scrape_article(url, download_images=args.download_images,
                                 images_dir=images_dir)
        if "error" in article:
            print(f"Error: {article['error']}", file=sys.stderr)
            sys.exit(1)
        out_path = os.path.join(args.out, f"{article['slug']}.md")
        with open(out_path, "w") as f:
            f.write(f"# {article['title']}\n\n")
            f.write(f"Source: {article['url']}\n\n---\n\n")
            f.write(article["markdown"])
        print(f"✓ {article['title']} → {out_path}")
        return

    # Discover all articles
    print(f"Discovering articles from {BASE_URL.format(lang=args.lang)}...")
    urls = discover_articles(args.lang)
    print(f"Found {len(urls)} articles")

    if args.limit:
        urls = urls[:args.limit]

    # Scrape each
    index = {}
    errors = 0
    for i, url in enumerate(urls):
        slug = url.rstrip("/").rsplit("/", 1)[-1]
        out_path = os.path.join(args.out, f"{slug}.md")

        if not args.refresh and os.path.exists(out_path):
            # Load existing
            index[slug] = {"title": slug, "path": f"docs/kb/{slug}.md", "url": url}
            continue

        article = scrape_article(url, download_images=args.download_images,
                                 images_dir=images_dir)
        if "error" in article:
            print(f"  ✗ {slug}: {article['error']}", file=sys.stderr)
            errors += 1
            continue

        with open(out_path, "w") as f:
            f.write(f"# {article['title']}\n\n")
            f.write(f"Source: {article['url']}\n\n---\n\n")
            f.write(article["markdown"])

        index[slug] = {
            "title": article["title"],
            "path": f"docs/kb/{slug}.md",
            "url": url,
        }
        print(f"  [{i+1}/{len(urls)}] {article['title']}")

        # Be polite
        time.sleep(0.3)

    # Write index
    index_path = os.path.join(args.out, "index.json")
    with open(index_path, "w") as f:
        json.dump(index, f, indent=2, sort_keys=True)
    print(f"\n✓ {len(index)} articles indexed → {index_path}")
    if errors:
        print(f"⚠ {errors} errors")


if __name__ == "__main__":
    main()
