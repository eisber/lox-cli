"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const {
  ChallengeError,
  acquireLock,
  assertArchivePage,
  assertLivePage,
  discoverAttachments,
  discoverPageUrls,
  importDiscoveries,
  listNextJobs,
  openLedger,
  pageNumberFromUrl,
  recordArchiveSuccess,
  releaseLock,
  storeObject,
  threadFromUrl,
  writeSummary,
} = require("../../scripts/loxforum-crawl.js");

test("canonicalizes topic, pagination, and post URLs to one thread", () => {
  const expected = {
    threadId: 300117,
    canonicalUrl:
      "https://www.loxforum.com/forum/faqs-tutorials-howto-s/300117-tutorial",
  };
  assert.deepEqual(
    threadFromUrl(
      "https://www.loxforum.com/forum/faqs-tutorials-howto-s/300117-tutorial?page=1#post300155",
    ),
    expected,
  );
  assert.deepEqual(
    threadFromUrl(
      "https://www.loxforum.com/forum/faqs-tutorials-howto-s/300117-tutorial/page16",
    ),
    expected,
  );
  assert.equal(
    threadFromUrl("https://www.loxforum.com/member/300117-tutorial"),
    null,
  );
});

test("extracts root and paginated page numbers", () => {
  assert.equal(
    pageNumberFromUrl("https://www.loxforum.com/forum/german/42-example"),
    1,
  );
  assert.equal(
    pageNumberFromUrl("https://www.loxforum.com/forum/german/42-example/page16"),
    16,
  );
});

test("imports archive discoveries idempotently and prioritizes RSS", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-ledger-"));
  const db = openLedger(directory);
  try {
    const base = {
      threadId: 42,
      canonicalUrl: "https://www.loxforum.com/forum/german/42-example",
      captureUrl: "https://www.loxforum.com/forum/german/42-example",
      capturedAt: "2026-09-01T00:00:00.000Z",
      digest: "digest",
    };
    importDiscoveries(db, [
      { ...base, sourceType: "wayback", priority: 20 },
      {
        ...base,
        canonicalUrl: "https://www.loxforum.com/forum/german/42-renamed",
        captureUrl: "https://www.loxforum.com/forum/german/42-renamed",
        sourceType: "wayback",
        digest: "newer-digest",
        priority: 20,
      },
      { ...base, sourceType: "rss", title: "Example", priority: 100 },
      { ...base, sourceType: "rss", title: "Example", priority: 100 },
    ]);

    assert.equal(db.prepare("SELECT COUNT(*) AS count FROM threads").get().count, 1);
    assert.equal(db.prepare("SELECT COUNT(*) AS count FROM sources").get().count, 3);
    assert.equal(db.prepare("SELECT COUNT(*) AS count FROM jobs").get().count, 1);
    assert.deepEqual(
      { ...db.prepare("SELECT priority, state FROM jobs").get() },
      { priority: 100, state: "queued" },
    );
  } finally {
    db.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("discovers only same-thread pages and records attachment metadata", () => {
  const pageUrl = "https://www.loxforum.com/forum/german/42-example";
  const html = `
    <a href="/forum/german/42-example/page2">Next</a>
    <a href="/forum/german/43-other/page2">Other</a>
    <a href="filedata/fetch?id=7&amp;filename=sample.Loxone">Config</a>
  `;

  assert.deepEqual(discoverPageUrls(html, pageUrl, 42), [{
    url: "https://www.loxforum.com/forum/german/42-example/page2",
    pageNumber: 2,
  }]);
  assert.deepEqual(discoverAttachments(html, pageUrl), [{
    url: "https://www.loxforum.com/filedata/fetch?id=7&filename=sample.Loxone",
    filename: "sample.Loxone",
  }]);
});

test("prevents concurrent live capture processes", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-lock-"));
  const db = openLedger(directory);
  try {
    acquireLock(db, "live-capture", "first");
    assert.throws(
      () => acquireLock(db, "live-capture", "second"),
      /already running/,
    );
    releaseLock(db, "live-capture", "first");
    acquireLock(db, "live-capture", "second");
  } finally {
    db.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("lists blocked jobs before queued jobs", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-next-"));
  const db = openLedger(directory);
  try {
    importDiscoveries(db, [
      {
        threadId: 42,
        canonicalUrl: "https://www.loxforum.com/forum/german/42-example",
        sourceType: "rss",
        captureUrl: "https://www.loxforum.com/forum/german/42-example",
        capturedAt: "2026-09-01T00:00:00.000Z",
        digest: "",
        priority: 100,
      },
      {
        threadId: 43,
        canonicalUrl: "https://www.loxforum.com/forum/german/43-example",
        sourceType: "wayback",
        captureUrl: "https://www.loxforum.com/forum/german/43-example",
        capturedAt: "2026-09-01T00:00:00.000Z",
        digest: "",
        priority: 20,
      },
      {
        threadId: 44,
        canonicalUrl: "https://www.loxforum.com/forum/german/44-example",
        sourceType: "wayback",
        captureUrl: "https://www.loxforum.com/forum/german/44-example",
        capturedAt: "2026-09-01T00:00:00.000Z",
        digest: "",
        priority: 20,
      },
    ]);
    db.prepare("UPDATE jobs SET state = 'blocked' WHERE thread_id = 43").run();
    db.prepare("UPDATE jobs SET state = 'retry_wait' WHERE thread_id = 44").run();
    assert.deepEqual(
      listNextJobs(db, 2).map((job) => job.thread_id),
      [43, 42],
    );
  } finally {
    db.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("archive processing inventories attachments without downloading them", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-archive-"));
  const db = openLedger(directory);
  try {
    importDiscoveries(db, [{
      threadId: 42,
      canonicalUrl: "https://www.loxforum.com/forum/german/42-example",
      sourceType: "wayback",
      captureUrl: "https://www.loxforum.com/forum/german/42-example",
      capturedAt: "2026-09-01T00:00:00.000Z",
      digest: "digest",
      priority: 20,
    }]);
    const source = db.prepare("SELECT * FROM sources").get();
    recordArchiveSuccess(db, directory, source, {
      status: 200,
      contentType: "text/html",
      text: '<html><a href="/attachment/7/sample.Loxone">Config</a></html>',
    });
    assert.equal(
      db.prepare("SELECT COUNT(*) AS count FROM attachments").get().count,
      1,
    );
    assert.equal(
      db.prepare("SELECT archive_state FROM sources").get().archive_state,
      "completed",
    );
  } finally {
    db.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("stores content by hash outside the repository", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-objects-"));
  try {
    const first = storeObject(directory, "<html>example</html>", "text/html");
    const second = storeObject(directory, "<html>example</html>", "text/html");
    assert.deepEqual(first, second);
    assert.equal(
      fs.readFileSync(path.join(directory, first.relativePath), "utf8"),
      "<html>example</html>",
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("treats challenge content as a hard stop", () => {
  assert.throws(
    () =>
      assertLivePage({
        status: 200,
        contentType: "text/html",
        text: `${"<html>".padEnd(2_100, " ")}h-captcha`,
      }),
    ChallengeError,
  );
});

test("rejects archived challenge captures", () => {
  assert.throws(
    () =>
      assertArchivePage({
        status: 200,
        contentType: "text/html",
        text: "<html><title>One moment, please...</title></html>",
      }),
    ChallengeError,
  );
});

test("writes aggregate summaries without raw content", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "loxforum-summary-"));
  const db = openLedger(directory);
  const summaryPath = path.join(directory, "summary.json");
  try {
    importDiscoveries(db, [{
      threadId: 42,
      canonicalUrl: "https://www.loxforum.com/forum/german/42-example",
      sourceType: "rss",
      captureUrl: "https://www.loxforum.com/forum/german/42-example",
      capturedAt: "2026-09-01T00:00:00.000Z",
      digest: "",
      priority: 100,
    }]);
    const summary = writeSummary(db, summaryPath);
    assert.equal(summary.totals.threads, 1);
    assert.equal(summary.totals.live_jobs, 1);
    assert.equal(JSON.parse(fs.readFileSync(summaryPath, "utf8")).sources[0].source_type, "rss");
  } finally {
    db.close();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
