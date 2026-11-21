#!/usr/bin/env python3
"""Archive public loxforum pages with local Scrapling; export posts for curation.

Example (Windows):
  python scripts\\scrape-forum.py --scrapling-path E:\\work\\Scrapling crawl
  python scripts\\scrape-forum.py export --output .lox\\forum\\posts.jsonl

The default archive is .lox/forum/archive.sqlite3 (git-ignored). Crawl requests
are sequential, obey robots.txt, and stop on HTTP errors. Re-run to resume;
use crawl --refresh to revisit saved pages for new posts and edits.
"""

import argparse
from contextlib import closing
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import sqlite3
import sys
import time
from urllib.parse import parse_qsl, urlencode, urljoin, urlsplit, urlunsplit
from urllib.robotparser import RobotFileParser


USER_AGENT = "lox-forum-archive/1.0"
HOSTS = {"loxforum.com", "www.loxforum.com"}
DEFAULT_BASE = "https://www.loxforum.com/"


def canonical_url(href, base=DEFAULT_BASE):
    """Keep public forum navigation, dropping tracking and post fragments."""
    parts = urlsplit(urljoin(base, href))
    if parts.scheme != "https" or parts.netloc.lower() not in HOSTS:
        return None
    path = parts.path or "/"
    if path != "/" and path != "/forum" and not path.startswith("/forum/"):
        return None
    if re.search(r"/(?:search|member|members|login|register|private-message)(?:/|$)", path):
        return None
    query = parse_qsl(parts.query, keep_blank_values=True)
    if any(key.lower() not in {"page", "pagenumber", "p", "perpage"} for key, _ in query):
        return None
    if any(not value.isdigit() or int(value) < 1 for _, value in query):
        return None
    host = urlsplit(base).netloc.lower()
    return urlunsplit(("https", host, path, urlencode(sorted(query)), ""))


def open_archive(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(path)
    db.row_factory = sqlite3.Row
    db.executescript("""
        CREATE TABLE IF NOT EXISTS queue (
            url TEXT PRIMARY KEY,
            state TEXT NOT NULL DEFAULT 'pending'
        );
        CREATE TABLE IF NOT EXISTS pages (
            url TEXT PRIMARY KEY,
            fetched_at TEXT NOT NULL,
            html BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS posts (
            post_id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            topic_url TEXT NOT NULL,
            topic_title TEXT NOT NULL,
            author TEXT NOT NULL,
            published_at TEXT NOT NULL,
            text TEXT NOT NULL,
            html TEXT NOT NULL,
            fetched_at TEXT NOT NULL
        );
    """)
    return db


def first_text(element, selectors):
    for selector in selectors:
        nodes = element.css(selector)
        if nodes:
            return nodes[0].get_all_text(separator=" ", strip=True)
    return ""


def extract_posts(page, url, fetched_at):
    title = first_text(page, ["h1", "title"])
    posts = []
    seen = set()
    for node in page.css('[id^="post"], [id^="node"], [data-node-id]'):
        # vBulletin 5 uses both post123 and post_123 / data-node-id.
        identity = node.attrib.get("data-node-id", "")
        match = re.fullmatch(r"(?:post|node)[_-]?(\d+)", node.attrib.get("id", ""))
        if not identity.isdigit() and match:
            identity = match.group(1)
        if not identity.isdigit() or identity in seen:
            continue
        bodies = []
        for selector in [".js-post__content-text", ".postcontent", ".b-post__content"]:
            bodies = node.css(selector)
            if bodies:
                break
        if not bodies:
            continue
        body = bodies[0]
        text = body.get_all_text(separator="\n", strip=True)
        if not text:
            continue
        seen.add(identity)
        timestamp = node.css("time")
        published = timestamp[0].attrib.get("datetime", "") if timestamp else ""
        if not published:
            dates = node.css("[data-timestamp]")
            published = dates[0].attrib.get("data-timestamp", "") if dates else ""
        posts.append({
            "post_id": identity,
            "url": f"{url}#post{identity}",
            "topic_url": re.sub(r"/page\d+/?$", "", urlsplit(url)._replace(query="").geturl()),
            "topic_title": title,
            "author": first_text(node, [".b-userinfo__username", ".username", ".author"]),
            "published_at": published,
            "text": text,
            "html": body.html_content,
            "fetched_at": fetched_at,
        })
    return posts


def ingest(db, page, url):
    fetched_at = datetime.now(timezone.utc).isoformat()
    # Keep the original response even if changed markup prevents extraction.
    with db:
        db.execute(
            "INSERT OR REPLACE INTO pages VALUES (?, ?, ?)",
            (url, fetched_at, bytes(page.body)),
        )
    posts = extract_posts(page, url, fetched_at)
    expected_posts = page.css(".js-post__content-text, .b-post__content, .postcontent, .b-post")
    topic_path = re.search(r"/\d+-[^/]+(?:/page\d+)?/?$", urlsplit(url).path)
    if (expected_posts or topic_path) and not posts:
        raise RuntimeError(f"Post markup not recognized at {url}; raw HTML saved, page still pending")
    links = set()
    for anchor in page.css("a[href], link[rel='next'][href]"):
        link = canonical_url(anchor.attrib["href"], url)
        if link:
            links.add(link)
    if not links and not posts:
        raise RuntimeError(f"No forum navigation or posts recognized at {url}; raw HTML saved")
    with db:
        db.executemany("INSERT OR IGNORE INTO queue(url) VALUES (?)", ((u,) for u in links))
        db.executemany(
            """INSERT INTO posts VALUES (
                :post_id, :url, :topic_url, :topic_title, :author,
                :published_at, :text, :html, :fetched_at
            ) ON CONFLICT(post_id) DO UPDATE SET
                url=excluded.url, topic_url=excluded.topic_url,
                topic_title=excluded.topic_title, author=excluded.author,
                published_at=excluded.published_at, text=excluded.text,
                html=excluded.html, fetched_at=excluded.fetched_at""",
            posts,
        )
        db.execute("UPDATE queue SET state='done' WHERE url=?", (url,))
    return len(posts)


class ForumFetcher:
    def __init__(self, fetcher, delay):
        self.fetcher = fetcher
        self.delay = delay
        self.last_request = None

    def get(self, url):
        if self.last_request is not None:
            time.sleep(max(0, self.delay - (time.monotonic() - self.last_request)))
        self.last_request = time.monotonic()
        response = self.fetcher.get(
            url,
            headers={"User-Agent": USER_AGENT},
            impersonate=None,
            stealthy_headers=False,
            follow_redirects=False,
            retries=0,
            timeout=45,
        )
        return response


def robots_policy(fetcher, base):
    url = urljoin(base, "robots.txt")
    response = fetcher.get(url)
    policy = RobotFileParser(url)
    if response.status == 404:
        policy.parse([])
    elif response.status == 200:
        content = response.body.decode("utf-8-sig")
        if "<html" in content.lower() or "<!doctype" in content.lower():
            raise RuntimeError(f"{url} returned HTML instead of crawl rules; crawl not started")
        policy.parse(content.splitlines())
    else:
        raise RuntimeError(
            f"{url} returned HTTP {response.status}; crawl not started. "
            "Ask the forum operator for crawler access or an authorized export."
        )
    fetcher.delay = max(fetcher.delay, policy.crawl_delay(USER_AGENT) or 0)
    rate = policy.request_rate(USER_AGENT)
    if rate:
        fetcher.delay = max(fetcher.delay, rate.seconds / rate.requests)
    return policy


def crawl(db, fetcher, base, max_pages=None, refresh=False):
    policy = robots_policy(fetcher, base)
    with db:
        if refresh:
            db.execute("UPDATE queue SET state='pending'")
        db.execute("INSERT OR IGNORE INTO queue(url) VALUES (?)", (base,))
    visited = 0
    while max_pages is None or visited < max_pages:
        row = db.execute(
            "SELECT url FROM queue WHERE state='pending' ORDER BY rowid LIMIT 1"
        ).fetchone()
        if row is None:
            break
        url = row["url"]
        if not policy.can_fetch(USER_AGENT, url):
            with db:
                db.execute("UPDATE queue SET state='robots-denied' WHERE url=?", (url,))
            continue
        response = fetcher.get(url)
        if response.status != 200:
            raise RuntimeError(f"{url} returned HTTP {response.status}; stopped, resume state retained")
        count = ingest(db, response, url)
        visited += 1
        print(f"{url}: {count} posts", file=sys.stderr, flush=True)
    return visited


def export_posts(db, output):
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_name(output.name + ".tmp")
    count = 0
    try:
        with temporary.open("w", encoding="utf-8") as stream:
            for row in db.execute("SELECT * FROM posts ORDER BY CAST(post_id AS INTEGER)"):
                item = dict(row)
                item["source"] = "loxforum.com"
                item["content_sha256"] = hashlib.sha256(item["html"].encode("utf-8")).hexdigest()
                stream.write(json.dumps(item, ensure_ascii=False) + "\n")
                count += 1
        temporary.replace(output)
    finally:
        temporary.unlink(missing_ok=True)
    return count


def summary(db):
    return {
        "pages": db.execute("SELECT COUNT(*) FROM pages").fetchone()[0],
        "posts": db.execute("SELECT COUNT(*) FROM posts").fetchone()[0],
        "queue": {
            row["state"]: row["n"]
            for row in db.execute("SELECT state, COUNT(*) AS n FROM queue GROUP BY state")
        },
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--scrapling-path", type=Path, help="Local Scrapling source checkout")
    parser.add_argument("--archive", type=Path, default=Path(".lox") / "forum" / "archive.sqlite3")
    commands = parser.add_subparsers(dest="command", required=True)
    crawl_parser = commands.add_parser("crawl", help="Crawl reachable public forum pages; resume by default")
    crawl_parser.add_argument("--base-url", default=DEFAULT_BASE, choices=[DEFAULT_BASE, "https://loxforum.com/"])
    crawl_parser.add_argument("--delay", type=float, default=3, help="Minimum seconds between requests (default: 3)")
    crawl_parser.add_argument("--max-pages", type=int, help="Limit fetched pages for a bounded pilot")
    crawl_parser.add_argument("--refresh", action="store_true", help="Revisit previously completed and denied URLs")
    export_parser = commands.add_parser("export", help="Export deduplicated posts to JSONL for curation")
    export_parser.add_argument("--output", type=Path, required=True)
    commands.add_parser("status", help="Show archive and queue counts as JSON")
    args = parser.parse_args(argv)
    if args.command == "crawl" and (
        not 0 < args.delay < float("inf") or args.max_pages is not None and args.max_pages < 1
    ):
        parser.error("--delay must be positive and finite; --max-pages must be at least 1")
    if args.scrapling_path:
        if not (args.scrapling_path / "scrapling" / "__init__.py").is_file():
            parser.error("--scrapling-path must point to a Scrapling source checkout")
        sys.path.insert(0, str(args.scrapling_path.resolve()))
    if args.command != "crawl" and not args.archive.is_file():
        parser.error(f"Archive does not exist: {args.archive}")
    try:
        with closing(open_archive(args.archive)) as db:
            if args.command == "crawl":
                from scrapling.fetchers import Fetcher

                crawl(db, ForumFetcher(Fetcher, args.delay), args.base_url, args.max_pages, args.refresh)
            elif args.command == "export":
                print(json.dumps({"exported": export_posts(db, args.output), "output": str(args.output)}))
            print(json.dumps(summary(db)))
    except (ImportError, OSError, RuntimeError, sqlite3.Error) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
