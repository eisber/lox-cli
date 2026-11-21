import importlib.util
import json
from contextlib import closing
from pathlib import Path
import tempfile
import unittest

from scrapling import Selector


SPEC = importlib.util.spec_from_file_location(
    "scrape_forum", Path(__file__).with_name("scrape-forum.py")
)
FORUM = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FORUM)

BASE = "https://www.loxforum.com/"
TOPIC = BASE + "forum/german/software/123-topic"
HTML = b"""
<h1>Timer troubleshooting</h1>
<article id="post_123" data-node-id="123">
  <a class="b-userinfo__username">Example author</a>
  <time datetime="2026-01-01T10:00:00Z"></time>
  <div class="js-post__content-text">Set <b>TH</b> to 20.<pre>x &lt; 2</pre></div>
</article>
<article id="post124"><div class="postcontent">Second post.</div></article>
<a href="/forum/german/software/123-topic/page2#post125">Next</a>
<a href="/member/42">Profile</a>
<a href="/forum/german/software/123-topic?do=delete">Action</a>
"""


class Page:
    def __init__(self, body, status=200):
        self.selector = Selector(body)
        self.body = body
        self.status = status

    def css(self, selector):
        return self.selector.css(selector)


class Fetcher:
    def __init__(self, responses):
        self.responses = responses
        self.urls = []
        self.delay = 3

    def get(self, url):
        self.urls.append(url)
        return self.responses[url]


class ForumTests(unittest.TestCase):
    def test_navigation_excludes_actions_external_links_and_profiles(self):
        self.assertEqual(
            FORUM.canonical_url("/forum/german/topic/page2#post2", TOPIC),
            BASE + "forum/german/topic/page2",
        )
        self.assertEqual(FORUM.canonical_url("?page=2", TOPIC), TOPIC + "?page=2")
        for href in ["https://example.com/forum/x", "//www.loxforum.com.evil/forum/x",
                     "/member/42", "?do=delete", "?page=invalid", "http://loxforum.com/forum/x"]:
            self.assertIsNone(FORUM.canonical_url(href, TOPIC), href)

    def test_ingest_deduplicates_edits_and_exports_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with closing(FORUM.open_archive(root / "archive.sqlite3")) as db:
                self.assertEqual(FORUM.ingest(db, Page(HTML), TOPIC), 2)
                self.assertEqual(
                    FORUM.ingest(db, Page(HTML.replace(b"to 20", b"to 25")), TOPIC), 2
                )
                output = root / "posts.jsonl"
                self.assertEqual(FORUM.export_posts(db, output), 2)
                posts = [json.loads(line) for line in output.read_text(encoding="utf-8").splitlines()]
                self.assertEqual(posts[0]["post_id"], "123")
                self.assertIn("25", posts[0]["text"])
                self.assertIn("<b>TH</b>", posts[0]["html"])
                self.assertEqual(posts[0]["url"], TOPIC + "#post123")
                self.assertEqual(posts[0]["published_at"], "2026-01-01T10:00:00Z")
                self.assertEqual(posts[0]["author"], "Example author")
                self.assertEqual(posts[0]["source"], "loxforum.com")
                queued = [r[0] for r in db.execute("SELECT url FROM queue")]
                self.assertEqual(queued, [TOPIC + "/page2"])
                self.assertIn(b"to 25", db.execute("SELECT html FROM pages").fetchone()[0])

    def test_robots_failure_never_fetches_forum_or_seeds_queue(self):
        fetcher = Fetcher({BASE + "robots.txt": Page(b"Forbidden", 403)})
        with tempfile.TemporaryDirectory() as temporary:
            with closing(FORUM.open_archive(Path(temporary) / "archive.sqlite3")) as db:
                with self.assertRaisesRegex(RuntimeError, "HTTP 403"):
                    FORUM.crawl(db, fetcher, BASE)
                self.assertEqual(fetcher.urls, [BASE + "robots.txt"])
                self.assertEqual(FORUM.summary(db)["queue"], {})

    def test_resume_pagination_and_robots_denials(self):
        next_page = TOPIC + "/page2"
        fetcher = Fetcher({
            BASE + "robots.txt": Page(
                b"User-agent: *\nDisallow: /forum/private\nCrawl-delay: 9\n"
            ),
            BASE: Page(f'<a href="{TOPIC}">Topic</a><a href="/forum/private">Private</a>'.encode()),
            TOPIC: Page(HTML),
            next_page: Page(
                b'<h1>Page 2</h1><article id="post125">'
                b'<div class="postcontent">Third post.</div></article>'
            ),
        })
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "archive.sqlite3"
            with closing(FORUM.open_archive(path)) as db:
                self.assertEqual(FORUM.crawl(db, fetcher, BASE, max_pages=1), 1)
                self.assertEqual(fetcher.delay, 9)
            with closing(FORUM.open_archive(path)) as db:
                self.assertEqual(FORUM.crawl(db, fetcher, BASE), 2)
                self.assertEqual(FORUM.summary(db)["queue"], {"done": 3, "robots-denied": 1})
                self.assertEqual(
                    db.execute("SELECT topic_url FROM posts WHERE post_id='125'").fetchone()[0],
                    TOPIC,
                )
            self.assertEqual(fetcher.urls.count(BASE), 1)
            self.assertNotIn(BASE + "forum/private", fetcher.urls)

    def test_http_failure_preserves_pending_url(self):
        fetcher = Fetcher({
            BASE + "robots.txt": Page(b"", 404),
            BASE: Page(b"Rate limited", 429),
        })
        with tempfile.TemporaryDirectory() as temporary:
            with closing(FORUM.open_archive(Path(temporary) / "archive.sqlite3")) as db:
                with self.assertRaisesRegex(RuntimeError, "HTTP 429"):
                    FORUM.crawl(db, fetcher, BASE)
                self.assertEqual(FORUM.summary(db)["queue"], {"pending": 1})

    def test_unrecognized_post_markup_saves_html_without_completing_page(self):
        body = b'<div id="unexpected"><div class="postcontent">Text</div></div>'
        with tempfile.TemporaryDirectory() as temporary:
            with closing(FORUM.open_archive(Path(temporary) / "archive.sqlite3")) as db:
                with db:
                    db.execute("INSERT INTO queue(url) VALUES (?)", (TOPIC,))
                with self.assertRaisesRegex(RuntimeError, "markup not recognized"):
                    FORUM.ingest(db, Page(body), TOPIC)
                self.assertEqual(FORUM.summary(db)["queue"], {"pending": 1})
                self.assertEqual(db.execute("SELECT html FROM pages").fetchone()[0], body)

    def test_empty_success_page_is_not_treated_as_a_completed_crawl(self):
        with tempfile.TemporaryDirectory() as temporary:
            with closing(FORUM.open_archive(Path(temporary) / "archive.sqlite3")) as db:
                with self.assertRaisesRegex(RuntimeError, "No forum navigation or posts"):
                    FORUM.ingest(db, Page(b"<h1>Access denied</h1>"), BASE)
                with self.assertRaisesRegex(RuntimeError, "Post markup not recognized"):
                    FORUM.ingest(db, Page(b"<h1>Unknown post markup</h1>"), TOPIC)


if __name__ == "__main__":
    unittest.main()
