"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const {
  assertFeedResponse,
  parseRss,
  updateState,
  writeJsonAtomic,
} = require("../../scripts/scrape-loxforum.js");

const FEED = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <item>
      <title><![CDATA[Heating &amp; cooling example]]></title>
      <link>https://www.loxforum.com/forum/german/software/120-topic?p=491125#post491125</link>
      <guid isPermaLink="true">https://www.loxforum.com/forum/german/software/120-topic?p=491125</guid>
      <pubDate>Sun, 06 Sep 2026 10:15:00 GMT</pubDate>
    </item>
    <item>
      <title>Older example</title>
      <link>https://www.loxforum.com/forum/german/software/119-topic#post491100</link>
      <guid>post-491100</guid>
      <pubDate>Sat, 05 Sep 2026 08:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>`;

test("parses stable post and thread identifiers from RSS", () => {
  const items = parseRss(FEED);

  assert.equal(items.length, 2);
  assert.deepEqual(items[0], {
    post_id: 491125,
    thread_id: 120,
    title: "Heating & cooling example",
    url: "https://www.loxforum.com/forum/german/software/120-topic?p=491125#post491125",
    guid: "https://www.loxforum.com/forum/german/software/120-topic?p=491125",
    published_at: "2026-09-06T10:15:00.000Z",
  });
});

test("advances the high-water mark monotonically and deduplicates recent items", () => {
  const items = parseRss(FEED);
  const previous = {
    highest_thread_id: 119,
    latest_published_at: "2026-09-06T09:00:00.000Z",
    recent_items: [items[1]],
  };

  const next = updateState(previous, items, "2026-09-07T19:00:00.000Z");

  assert.equal(next.highest_thread_id, 120);
  assert.equal(next.latest_published_at, "2026-09-06T10:15:00.000Z");
  assert.deepEqual(next.last_run, { items_seen: 2, new_threads: 1 });
  assert.equal(next.recent_items.length, 2);
  assert.equal(next.recent_items[0].thread_id, 120);
});

test("rejects challenge HTML before state can advance", () => {
  assert.throws(
    () =>
      assertFeedResponse({
        status: 200,
        contentType: "text/html",
        text: "<title>Security check required</title><p>captcha</p>",
      }),
    /challenged/,
  );
});

test("writes valid JSON through a temporary file", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-state-"));
  const statePath = path.join(directory, "high-water.json");

  try {
    writeJsonAtomic(statePath, { highest_thread_id: 42 });
    assert.deepEqual(JSON.parse(fs.readFileSync(statePath, "utf8")), {
      highest_thread_id: 42,
    });
    assert.deepEqual(fs.readdirSync(directory), ["high-water.json"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
