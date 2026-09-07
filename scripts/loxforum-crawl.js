#!/usr/bin/env node
"use strict";

const crypto = require("node:crypto");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { DatabaseSync } = require("node:sqlite");

const {
  evaluate,
  findForumPage,
  randomDelay,
  sleep,
  writeJsonAtomic,
} = require("./scrape-loxforum.js");

const FORUM_ORIGIN = "https://www.loxforum.com";
const DEFAULT_SUMMARY_PATH = path.join(
  __dirname,
  "..",
  "docs",
  "loxforum",
  "inventory-summary.json",
);
const DEFAULT_HIGH_WATER_PATH = path.join(
  __dirname,
  "..",
  "docs",
  "loxforum",
  "high-water.json",
);
const DEFAULT_CDP_URL = "http://127.0.0.1:9222";
const LIVE_MIN_DELAY_MS = 45_000;
const LIVE_MAX_DELAY_MS = 90_000;
const ARCHIVE_MIN_DELAY_MS = 1_000;
const ARCHIVE_MAX_DELAY_MS = 3_000;
const LIVE_LIMIT = 25;
const ARCHIVE_LIMIT = 100;
const LEASE_MS = 30 * 60 * 1_000;

class ChallengeError extends Error {}

function defaultDataDir() {
  if (process.env.LOXFORUM_DATA_DIR) return process.env.LOXFORUM_DATA_DIR;
  if (process.env.LOCALAPPDATA) {
    return path.join(process.env.LOCALAPPDATA, "loxforum-scraper");
  }
  return path.join(
    process.env.XDG_DATA_HOME || path.join(os.homedir(), ".local", "share"),
    "loxforum-scraper",
  );
}

function parseArgs(argv) {
  const command = argv[0];
  if (!["inventory", "archive-pilot", "pilot", "status", "unblock"].includes(command)) {
    throw new Error(
      "Usage: node scripts/loxforum-crawl.js <inventory|archive-pilot|pilot|status|unblock> [options]",
    );
  }

  const options = {
    command,
    dataDir: defaultDataDir(),
    summaryPath: DEFAULT_SUMMARY_PATH,
    highWaterPath: DEFAULT_HIGH_WATER_PATH,
    cdpUrl: DEFAULT_CDP_URL,
    limit: command === "pilot" ? LIVE_LIMIT : ARCHIVE_LIMIT,
    minDelayMs:
      command === "pilot" ? LIVE_MIN_DELAY_MS : ARCHIVE_MIN_DELAY_MS,
    maxDelayMs:
      command === "pilot" ? LIVE_MAX_DELAY_MS : ARCHIVE_MAX_DELAY_MS,
  };
  const values = {
    "--data-dir": "dataDir",
    "--summary": "summaryPath",
    "--high-water": "highWaterPath",
    "--cdp-url": "cdpUrl",
    "--limit": "limit",
    "--min-delay-ms": "minDelayMs",
    "--max-delay-ms": "maxDelayMs",
  };

  for (let i = 1; i < argv.length; i += 1) {
    const key = values[argv[i]];
    if (!key || i + 1 >= argv.length) {
      throw new Error(`Unknown or incomplete argument: ${argv[i]}`);
    }
    options[key] = ["limit", "minDelayMs", "maxDelayMs"].includes(key)
      ? Number(argv[++i])
      : argv[++i];
  }

  if (
    !Number.isInteger(options.limit) ||
    options.limit < 1 ||
    !Number.isInteger(options.minDelayMs) ||
    !Number.isInteger(options.maxDelayMs) ||
    options.minDelayMs < 0 ||
    options.maxDelayMs < options.minDelayMs
  ) {
    throw new Error(
      "Limit and delays must be integers with limit >= 1 and 0 <= min <= max",
    );
  }
  if (command === "pilot" && options.limit > LIVE_LIMIT) {
    throw new Error(`Live pilot limit cannot exceed ${LIVE_LIMIT} requests`);
  }
  return options;
}

function openLedger(dataDir) {
  fs.mkdirSync(dataDir, { recursive: true });
  const db = new DatabaseSync(path.join(dataDir, "ledger.sqlite"));
  db.exec(`
    PRAGMA foreign_keys = ON;
    PRAGMA journal_mode = WAL;

    CREATE TABLE IF NOT EXISTS threads (
      thread_id INTEGER PRIMARY KEY,
      canonical_url TEXT NOT NULL UNIQUE,
      title TEXT,
      first_seen_at TEXT NOT NULL,
      last_seen_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS sources (
      id INTEGER PRIMARY KEY,
      thread_id INTEGER NOT NULL REFERENCES threads(thread_id),
      source_type TEXT NOT NULL,
      capture_url TEXT NOT NULL,
      captured_at TEXT NOT NULL DEFAULT '',
      digest TEXT NOT NULL DEFAULT '',
      archive_state TEXT NOT NULL DEFAULT 'queued',
      attempts INTEGER NOT NULL DEFAULT 0,
      available_at TEXT,
      lease_until TEXT,
      content_sha256 TEXT,
      last_error TEXT,
      UNIQUE(source_type, capture_url, captured_at, digest)
    );

    CREATE TABLE IF NOT EXISTS jobs (
      id INTEGER PRIMARY KEY,
      thread_id INTEGER NOT NULL REFERENCES threads(thread_id),
      page_number INTEGER NOT NULL,
      url TEXT NOT NULL UNIQUE,
      priority INTEGER NOT NULL DEFAULT 0,
      state TEXT NOT NULL DEFAULT 'queued',
      attempts INTEGER NOT NULL DEFAULT 0,
      available_at TEXT,
      lease_until TEXT,
      content_sha256 TEXT,
      last_http_status INTEGER,
      last_error TEXT,
      updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS objects (
      sha256 TEXT PRIMARY KEY,
      relative_path TEXT NOT NULL,
      byte_length INTEGER NOT NULL,
      content_type TEXT NOT NULL,
      created_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS attachments (
      url TEXT PRIMARY KEY,
      thread_id INTEGER NOT NULL REFERENCES threads(thread_id),
      page_url TEXT NOT NULL,
      filename TEXT,
      state TEXT NOT NULL DEFAULT 'discovered',
      discovered_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS crawl_locks (
      name TEXT PRIMARY KEY,
      owner TEXT NOT NULL,
      lease_until TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS jobs_ready
      ON jobs(state, available_at, priority DESC, thread_id DESC);
    CREATE INDEX IF NOT EXISTS sources_archive_ready
      ON sources(source_type, archive_state, captured_at DESC);
  `);
  migrateLedger(db);
  reclassifyStoredChallenges(db, dataDir);
  return db;
}

function ensureColumn(db, table, column, definition) {
  const columns = db.prepare(`PRAGMA table_info(${table})`).all();
  if (!columns.some((candidate) => candidate.name === column)) {
    db.exec(`ALTER TABLE ${table} ADD COLUMN ${column} ${definition}`);
  }
}

function migrateLedger(db) {
  ensureColumn(db, "sources", "available_at", "TEXT");
  ensureColumn(db, "sources", "lease_until", "TEXT");
  begin(db);
  try {
    db.exec(`
      DELETE FROM jobs
      WHERE id NOT IN (
        SELECT selected.id
        FROM jobs AS selected
        WHERE selected.id = (
          SELECT candidate.id
          FROM jobs AS candidate
          WHERE candidate.thread_id = selected.thread_id
            AND candidate.page_number = selected.page_number
          ORDER BY candidate.priority DESC, candidate.id DESC
          LIMIT 1
        )
      );
      CREATE UNIQUE INDEX IF NOT EXISTS jobs_thread_page
        ON jobs(thread_id, page_number);
    `);
    commit(db);
  } catch (error) {
    rollback(db);
    throw error;
  }
}

function looksLikeChallenge(text) {
  const prefix = text.slice(0, 20_000).toLowerCase();
  return (
    /captcha|security check required|botguard|h-captcha/.test(prefix) ||
    /<title[^>]*>\s*(?:one|just) moment,?\s*please/i.test(prefix) ||
    /cf-chl-|challenge-platform/.test(prefix)
  );
}

function reclassifyStoredChallenges(db, dataDir) {
  const captures = db.prepare(`
    SELECT DISTINCT o.sha256, o.relative_path
    FROM objects AS o
    JOIN sources AS s ON s.content_sha256 = o.sha256
    WHERE s.archive_state = 'completed'
  `).all();
  const invalidHashes = [];
  for (const capture of captures) {
    const fullPath = path.resolve(dataDir, capture.relative_path);
    if (!fullPath.startsWith(`${path.resolve(dataDir)}${path.sep}`)) continue;
    if (!fs.existsSync(fullPath)) continue;
    if (looksLikeChallenge(fs.readFileSync(fullPath, "utf8"))) {
      invalidHashes.push(capture.sha256);
    }
  }
  if (!invalidHashes.length) return;

  begin(db);
  try {
    const mark = db.prepare(`
      UPDATE sources
      SET archive_state = 'challenge_capture', content_sha256 = NULL,
          last_error = 'Archived response is a security challenge capture'
      WHERE content_sha256 = ?
    `);
    for (const hash of invalidHashes) mark.run(hash);
    commit(db);
  } catch (error) {
    rollback(db);
    throw error;
  }

  const orphaned = db.prepare(`
    SELECT o.sha256, o.relative_path
    FROM objects AS o
    WHERE NOT EXISTS (
      SELECT 1 FROM sources AS s WHERE s.content_sha256 = o.sha256
    )
    AND NOT EXISTS (
      SELECT 1 FROM jobs AS j WHERE j.content_sha256 = o.sha256
    )
  `).all();
  const removeObject = db.prepare("DELETE FROM objects WHERE sha256 = ?");
  for (const object of orphaned) {
    const fullPath = path.resolve(dataDir, object.relative_path);
    if (fullPath.startsWith(`${path.resolve(dataDir)}${path.sep}`)) {
      fs.rmSync(fullPath, { force: true });
    }
    removeObject.run(object.sha256);
  }
}

function isoFromCdxTimestamp(timestamp) {
  if (!/^\d{14}$/.test(timestamp || "")) return "";
  return new Date(
    `${timestamp.slice(0, 4)}-${timestamp.slice(4, 6)}-${timestamp.slice(6, 8)}` +
      `T${timestamp.slice(8, 10)}:${timestamp.slice(10, 12)}:${timestamp.slice(12, 14)}Z`,
  ).toISOString();
}

function threadFromUrl(input) {
  try {
    const url = new URL(input);
    let pathname = url.pathname.replace(/\/page\d+\/?$/i, "");
    pathname = pathname.replace(/\/$/, "");
    const match = pathname.match(/\/(\d+)-([^/]+)$/);
    if (!match) return null;
    const threadId = Number.parseInt(match[1], 10);
    if (!Number.isSafeInteger(threadId)) return null;
    return {
      threadId,
      canonicalUrl: `${FORUM_ORIGIN}${pathname}`,
    };
  } catch {
    return null;
  }
}

function begin(db) {
  db.exec("BEGIN IMMEDIATE");
}

function commit(db) {
  db.exec("COMMIT");
}

function rollback(db) {
  try {
    db.exec("ROLLBACK");
  } catch {
    // The transaction may already have ended.
  }
}

function upsertDiscovery(db, discovery) {
  const now = new Date().toISOString();
  db.prepare(`
    INSERT INTO threads(thread_id, canonical_url, title, first_seen_at, last_seen_at)
    VALUES (?, ?, ?, ?, ?)
    ON CONFLICT(thread_id) DO UPDATE SET
      canonical_url = excluded.canonical_url,
      title = COALESCE(excluded.title, threads.title),
      last_seen_at = excluded.last_seen_at
  `).run(
    discovery.threadId,
    discovery.canonicalUrl,
    discovery.title || null,
    now,
    now,
  );
  db.prepare(`
    INSERT OR IGNORE INTO sources(
      thread_id, source_type, capture_url, captured_at, digest
    ) VALUES (?, ?, ?, ?, ?)
  `).run(
    discovery.threadId,
    discovery.sourceType,
    discovery.captureUrl,
    discovery.capturedAt || "",
    discovery.digest || "",
  );
  db.prepare(`
    INSERT INTO jobs(thread_id, page_number, url, priority, updated_at)
    VALUES (?, 1, ?, ?, ?)
    ON CONFLICT(thread_id, page_number) DO UPDATE SET
      url = CASE
        WHEN excluded.priority >= jobs.priority THEN excluded.url
        ELSE jobs.url
      END,
      priority = MAX(jobs.priority, excluded.priority)
  `).run(
    discovery.threadId,
    discovery.canonicalUrl,
    discovery.priority || 0,
    now,
  );
}

async function fetchText(url, attempts = 3) {
  let lastError;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const response = await fetch(url, {
        headers: { "User-Agent": "lox-cli-research/1.0" },
        signal: AbortSignal.timeout(120_000),
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status} from ${new URL(url).hostname}`);
      }
      return await response.text();
    } catch (error) {
      lastError = error;
      if (attempt < attempts) await sleep(attempt * 2_000);
    }
  }
  throw lastError;
}

async function loadWaybackInventory() {
  const params = new URLSearchParams({
    url: "loxforum.com/forum/*",
    output: "json",
    fl: "timestamp,original,statuscode,digest,mimetype",
    filter: "statuscode:200",
    collapse: "urlkey",
    limit: "100000",
  });
  const rows = JSON.parse(
    await fetchText(`https://web.archive.org/cdx/search/cdx?${params}`),
  );
  const [header, ...records] = rows;
  const indexes = Object.fromEntries(header.map((name, index) => [name, index]));
  return records.flatMap((record) => {
    const original = record[indexes.original];
    const thread = threadFromUrl(original);
    if (!thread) return [];
    const timestamp = record[indexes.timestamp];
    return [{
      ...thread,
      sourceType: "wayback",
      captureUrl: original,
      capturedAt: isoFromCdxTimestamp(timestamp),
      digest: record[indexes.digest] || "",
      archiveTimestamp: timestamp,
      priority: 20,
    }];
  });
}

async function loadCommonCrawlInventory() {
  const collections = JSON.parse(
    await fetchText("https://index.commoncrawl.org/collinfo.json"),
  );
  if (!collections.length) throw new Error("Common Crawl returned no collections");
  const collection = collections[0];
  const params = new URLSearchParams({
    url: "loxforum.com/forum/*",
    output: "json",
    filter: "status:200",
    collapse: "urlkey",
  });
  const text = await fetchText(`${collection["cdx-api"]}?${params}`);
  return text
    .split(/\r?\n/)
    .filter(Boolean)
    .flatMap((line) => {
      const record = JSON.parse(line);
      const thread = threadFromUrl(record.url);
      if (!thread) return [];
      return [{
        ...thread,
        sourceType: `common-crawl:${collection.id}`,
        captureUrl: record.url,
        capturedAt: isoFromCdxTimestamp(record.timestamp),
        digest: record.digest || "",
        priority: 10,
      }];
    });
}

function loadRssInventory(highWaterPath) {
  if (!fs.existsSync(highWaterPath)) return [];
  const state = JSON.parse(fs.readFileSync(highWaterPath, "utf8"));
  return (state.recent_items || []).flatMap((item) => {
    const thread = threadFromUrl(item.url);
    if (!thread) return [];
    return [{
      ...thread,
      title: item.title,
      sourceType: "rss",
      captureUrl: item.url,
      capturedAt: item.published_at || "",
      digest: item.guid || "",
      priority: 100,
    }];
  });
}

function importDiscoveries(db, discoveries) {
  begin(db);
  try {
    for (const discovery of discoveries) upsertDiscovery(db, discovery);
    commit(db);
  } catch (error) {
    rollback(db);
    throw error;
  }
}

function writeSummary(db, summaryPath) {
  const bySource = db.prepare(`
    SELECT source_type, COUNT(*) AS captures, COUNT(DISTINCT thread_id) AS threads
    FROM sources GROUP BY source_type ORDER BY source_type
  `).all();
  const byState = db.prepare(`
    SELECT state, COUNT(*) AS count FROM jobs GROUP BY state ORDER BY state
  `).all();
  const archiveStates = db.prepare(`
    SELECT archive_state AS state, COUNT(*) AS count
    FROM sources
    WHERE source_type = 'wayback'
    GROUP BY archive_state
    ORDER BY archive_state
  `).all();
  const totals = db.prepare(`
    SELECT
      (SELECT COUNT(*) FROM threads) AS threads,
      (SELECT COUNT(*) FROM sources) AS source_records,
      (SELECT COUNT(*) FROM jobs) AS live_jobs,
      (SELECT COUNT(*) FROM objects) AS stored_objects,
      (SELECT COALESCE(SUM(byte_length), 0) FROM objects) AS stored_bytes,
      (SELECT COUNT(*) FROM attachments) AS discovered_attachments
  `).get();
  const summary = {
    schema_version: 1,
    generated_at: new Date().toISOString(),
    totals,
    sources: bySource,
    live_jobs: byState,
    wayback_jobs: archiveStates,
  };
  writeJsonAtomic(summaryPath, summary);
  return summary;
}

function printSummary(summary) {
  console.log(JSON.stringify(summary, null, 2));
}

function recoverExpiredLeases(db) {
  db.prepare(`
    UPDATE jobs
    SET state = 'queued', lease_until = NULL,
        last_error = 'Recovered expired lease', updated_at = ?
    WHERE state = 'in_progress' AND lease_until < ?
  `).run(new Date().toISOString(), new Date().toISOString());
}

function acquireLock(db, name, owner) {
  begin(db);
  try {
    const now = new Date().toISOString();
    const existing = db.prepare(
      "SELECT owner, lease_until FROM crawl_locks WHERE name = ?",
    ).get(name);
    if (existing && existing.lease_until >= now) {
      throw new Error(
        `${name} is already running under another process until ${existing.lease_until}`,
      );
    }
    db.prepare(`
      INSERT INTO crawl_locks(name, owner, lease_until)
      VALUES (?, ?, ?)
      ON CONFLICT(name) DO UPDATE SET
        owner = excluded.owner,
        lease_until = excluded.lease_until
    `).run(name, owner, new Date(Date.now() + LEASE_MS).toISOString());
    commit(db);
  } catch (error) {
    rollback(db);
    throw error;
  }
}

function renewLock(db, name, owner) {
  const result = db.prepare(`
    UPDATE crawl_locks SET lease_until = ?
    WHERE name = ? AND owner = ?
  `).run(new Date(Date.now() + LEASE_MS).toISOString(), name, owner);
  if (result.changes !== 1) {
    throw new Error(`Lost the ${name} process lock`);
  }
}

function releaseLock(db, name, owner) {
  db.prepare("DELETE FROM crawl_locks WHERE name = ? AND owner = ?").run(
    name,
    owner,
  );
}

function leaseLiveJob(db) {
  recoverExpiredLeases(db);
  begin(db);
  try {
    const now = new Date().toISOString();
    const job = db.prepare(`
      SELECT * FROM jobs
      WHERE state = 'queued'
         OR (state = 'retry_wait' AND available_at <= ?)
      ORDER BY priority DESC, thread_id DESC, page_number ASC
      LIMIT 1
    `).get(now);
    if (!job) {
      commit(db);
      return null;
    }
    db.prepare(`
      UPDATE jobs
      SET state = 'in_progress', lease_until = ?, updated_at = ?
      WHERE id = ?
    `).run(new Date(Date.now() + LEASE_MS).toISOString(), now, job.id);
    commit(db);
    return { ...job };
  } catch (error) {
    rollback(db);
    throw error;
  }
}

function markLiveAttempt(db, job) {
  const now = new Date().toISOString();
  db.prepare(`
    UPDATE jobs SET attempts = attempts + 1, updated_at = ? WHERE id = ?
  `).run(now, job.id);
  return { ...job, attempts: job.attempts + 1 };
}

function storeObject(dataDir, content, contentType) {
  const bytes = Buffer.from(content, "utf8");
  const sha256 = crypto.createHash("sha256").update(bytes).digest("hex");
  const relativePath = path.join("objects", sha256.slice(0, 2), `${sha256}.html`);
  const fullPath = path.join(dataDir, relativePath);
  fs.mkdirSync(path.dirname(fullPath), { recursive: true });
  if (!fs.existsSync(fullPath)) {
    const temporaryPath = `${fullPath}.${process.pid}.tmp`;
    fs.writeFileSync(temporaryPath, bytes, { flag: "wx" });
    fs.renameSync(temporaryPath, fullPath);
  }
  return { sha256, relativePath, byteLength: bytes.length, contentType };
}

function discoverPageUrls(html, pageUrl, threadId) {
  const pages = new Map();
  for (const match of html.matchAll(/href=["']([^"']+)["']/gi)) {
    const href = match[1].replace(/&amp;/g, "&");
    let url;
    try {
      url = new URL(href, pageUrl);
    } catch {
      continue;
    }
    if (url.origin !== FORUM_ORIGIN) continue;
    const thread = threadFromUrl(url.href);
    if (!thread || thread.threadId !== threadId) continue;
    const pageMatch = url.pathname.match(/\/page(\d+)\/?$/i);
    const pageNumber = pageMatch ? Number.parseInt(pageMatch[1], 10) : 1;
    const canonicalPage =
      pageNumber === 1 ? thread.canonicalUrl : `${thread.canonicalUrl}/page${pageNumber}`;
    pages.set(canonicalPage, pageNumber);
  }
  return [...pages].map(([url, pageNumber]) => ({ url, pageNumber }));
}

function discoverAttachments(html, pageUrl) {
  const attachments = new Map();
  for (const match of html.matchAll(/href=["']([^"']+)["']/gi)) {
    const href = match[1].replace(/&amp;/g, "&");
    try {
      const normalizedHref = /^(?:attachment|filedata\/fetch)(?:\/|\?|$)/i.test(href)
        ? `/${href}`
        : href;
      const url = new URL(normalizedHref, pageUrl);
      if (url.origin !== FORUM_ORIGIN) continue;
      if (!/\/(?:attachment|filedata\/fetch)(?:\/|$)/i.test(url.pathname)) {
        continue;
      }
      const filename =
        url.searchParams.get("filename") ||
        decodeURIComponent(url.pathname.split("/").pop() || "");
      attachments.set(url.href, filename);
    } catch {
      // Ignore malformed links from otherwise valid HTML.
    }
  }
  return [...attachments].map(([url, filename]) => ({ url, filename }));
}

function assertLivePage(response) {
  const challenged =
    response.status === 403 ||
    response.status === 429 ||
    looksLikeChallenge(response.text);
  if (challenged) {
    throw new ChallengeError(
      `Live crawl blocked or challenged (HTTP ${response.status}); manual verification is required`,
    );
  }
  if (response.status >= 500) {
    const error = new Error(`Live forum returned HTTP ${response.status}`);
    error.retryable = true;
    throw error;
  }
  if (
    response.status !== 200 ||
    !/^text\/html\b/i.test(response.contentType) ||
    response.text.length < 2_000
  ) {
    throw new Error(
      `Unexpected live response (HTTP ${response.status}, ${response.contentType || "no content type"}, ${response.text.length} bytes)`,
    );
  }
}

async function fetchLivePage(tab, url) {
  const expression = `(
    async () => {
      const response = await fetch(${JSON.stringify(url)}, {
        credentials: "include",
        headers: {"Accept": "text/html,application/xhtml+xml"}
      });
      return JSON.stringify({
        status: response.status,
        contentType: response.headers.get("content-type") || "",
        finalUrl: response.url,
        text: await response.text()
      });
    }
  )()`;
  const value = await evaluate(tab, expression);
  if (typeof value !== "string") throw new Error("Browser returned no page response");
  return JSON.parse(value);
}

function recordLiveSuccess(db, dataDir, job, response) {
  const object = storeObject(dataDir, response.text, response.contentType);
  const pages = discoverPageUrls(response.text, response.finalUrl || job.url, job.thread_id);
  const attachments = discoverAttachments(response.text, response.finalUrl || job.url);
  const now = new Date().toISOString();

  begin(db);
  try {
    db.prepare(`
      INSERT OR IGNORE INTO objects(
        sha256, relative_path, byte_length, content_type, created_at
      ) VALUES (?, ?, ?, ?, ?)
    `).run(
      object.sha256,
      object.relativePath,
      object.byteLength,
      object.contentType,
      now,
    );
    db.prepare(`
      UPDATE jobs
      SET state = 'completed', lease_until = NULL, available_at = NULL,
          content_sha256 = ?, last_http_status = ?, last_error = NULL, updated_at = ?
      WHERE id = ?
    `).run(object.sha256, response.status, now, job.id);
    for (const page of pages) {
      db.prepare(`
        INSERT OR IGNORE INTO jobs(
          thread_id, page_number, url, priority, updated_at
        ) VALUES (?, ?, ?, ?, ?)
      `).run(job.thread_id, page.pageNumber, page.url, job.priority, now);
    }
    for (const attachment of attachments) {
      db.prepare(`
        INSERT OR IGNORE INTO attachments(
          url, thread_id, page_url, filename, discovered_at
        ) VALUES (?, ?, ?, ?, ?)
      `).run(
        attachment.url,
        job.thread_id,
        response.finalUrl || job.url,
        attachment.filename,
        now,
      );
    }
    commit(db);
  } catch (error) {
    rollback(db);
    db.prepare(`
      UPDATE jobs
      SET state = 'quarantined', lease_until = NULL, content_sha256 = ?,
          last_error = ?, updated_at = ?
      WHERE id = ?
    `).run(object.sha256, error.message, now, job.id);
    throw error;
  }
}

function recordLiveFailure(db, job, error) {
  const now = new Date().toISOString();
  if (error instanceof ChallengeError) {
    db.prepare(`
      UPDATE jobs
      SET state = 'blocked', lease_until = NULL, last_error = ?, updated_at = ?
      WHERE id = ?
    `).run(error.message, now, job.id);
    return;
  }
  if (error.retryable && job.attempts < 3) {
    const delayMinutes = job.attempts === 1 ? 15 : 60;
    db.prepare(`
      UPDATE jobs
      SET state = 'retry_wait', lease_until = NULL, available_at = ?,
          last_error = ?, updated_at = ?
      WHERE id = ?
    `).run(
      new Date(Date.now() + delayMinutes * 60_000).toISOString(),
      error.message,
      now,
      job.id,
    );
    return;
  }
  db.prepare(`
    UPDATE jobs
    SET state = 'failed', lease_until = NULL, last_error = ?, updated_at = ?
    WHERE id = ?
  `).run(error.message, now, job.id);
}

function leaseWaybackSource(db) {
  const now = new Date().toISOString();
  db.prepare(`
    UPDATE sources
    SET archive_state = 'queued', lease_until = NULL,
        last_error = 'Recovered expired lease'
    WHERE archive_state = 'in_progress' AND lease_until < ?
  `).run(now);
  begin(db);
  try {
    const source = db.prepare(`
      SELECT * FROM sources
      WHERE source_type = 'wayback'
        AND (
          archive_state = 'queued'
          OR (archive_state = 'retry_wait' AND available_at <= ?)
        )
      ORDER BY captured_at DESC, thread_id DESC
      LIMIT 1
    `).get(now);
    if (!source) {
      commit(db);
      return null;
    }
    db.prepare(`
      UPDATE sources
      SET archive_state = 'in_progress', lease_until = ?
      WHERE id = ?
    `).run(new Date(Date.now() + LEASE_MS).toISOString(), source.id);
    commit(db);
    return { ...source };
  } catch (error) {
    rollback(db);
    throw error;
  }
}

function markArchiveAttempt(db, source) {
  db.prepare("UPDATE sources SET attempts = attempts + 1 WHERE id = ?").run(
    source.id,
  );
  return { ...source, attempts: source.attempts + 1 };
}

function compactCdxTimestamp(isoTimestamp) {
  return (isoTimestamp || "").replace(/\D/g, "").slice(0, 14);
}

async function fetchArchivePage(initialUrl) {
  let url = initialUrl;
  for (let redirect = 0; redirect <= 5; redirect += 1) {
    const response = await fetch(url, {
      headers: { "User-Agent": "lox-cli-research/1.0" },
      redirect: "manual",
      signal: AbortSignal.timeout(120_000),
    });
    if (response.status >= 300 && response.status < 400) {
      const location = response.headers.get("location");
      if (!location) throw new Error(`Wayback returned HTTP ${response.status} without a location`);
      const next = new URL(location, url);
      if (next.hostname !== "web.archive.org") {
        throw new Error(`Refusing Wayback redirect to ${next.hostname}`);
      }
      url = next.href;
      continue;
    }
    return {
      status: response.status,
      contentType: response.headers.get("content-type") || "",
      text: await response.text(),
    };
  }
  throw new Error("Wayback exceeded five redirects");
}

function assertArchivePage(response) {
  if (looksLikeChallenge(response.text)) {
    throw new ChallengeError("Archived response is a security challenge capture");
  }
  if (
    response.status !== 200 ||
    !/^text\/html\b/i.test(response.contentType) ||
    response.text.length < 500
  ) {
    const error = new Error(
      `Wayback returned HTTP ${response.status}, ${response.contentType || "no content type"}, ${response.text.length} bytes`,
    );
    error.retryable = response.status >= 500 || response.status === 429;
    throw error;
  }
}

function recordArchiveSuccess(db, dataDir, source, response) {
  const object = storeObject(
    dataDir,
    response.text,
    response.contentType,
  );
  const attachments = discoverAttachments(response.text, source.capture_url);
  const now = new Date().toISOString();
  begin(db);
  try {
    db.prepare(`
      INSERT OR IGNORE INTO objects(
        sha256, relative_path, byte_length, content_type, created_at
      ) VALUES (?, ?, ?, ?, ?)
    `).run(
      object.sha256,
      object.relativePath,
      object.byteLength,
      object.contentType,
      now,
    );
    db.prepare(`
      UPDATE sources
      SET archive_state = 'completed', content_sha256 = ?,
          lease_until = NULL, available_at = NULL, last_error = NULL
      WHERE id = ?
    `).run(object.sha256, source.id);
    for (const attachment of attachments) {
      db.prepare(`
        INSERT OR IGNORE INTO attachments(
          url, thread_id, page_url, filename, discovered_at
        ) VALUES (?, ?, ?, ?, ?)
      `).run(
        attachment.url,
        source.thread_id,
        source.capture_url,
        attachment.filename,
        now,
      );
    }
    commit(db);
  } catch (error) {
    rollback(db);
    throw error;
  }
  return object;
}

async function runArchivePilot(db, options) {
  let completed = 0;
  for (; completed < options.limit; completed += 1) {
    const source = leaseWaybackSource(db);
    if (!source) break;
    const delay = randomDelay(options.minDelayMs, options.maxDelayMs);
    await sleep(delay);
    const attemptedSource = markArchiveAttempt(db, source);
    const timestamp = compactCdxTimestamp(attemptedSource.captured_at);
    const archiveUrl =
      `https://web.archive.org/web/${timestamp}id_/${attemptedSource.capture_url}`;
    try {
      const response = await fetchArchivePage(archiveUrl);
      assertArchivePage(response);
      const object = recordArchiveSuccess(
        db,
        options.dataDir,
        attemptedSource,
        response,
      );
      console.log(
        `[${completed + 1}/${options.limit}] archived thread ${attemptedSource.thread_id} (${object.byteLength} bytes)`,
      );
    } catch (error) {
      let state = "failed";
      let availableAt = null;
      if (error instanceof ChallengeError) {
        state = "challenge_capture";
      } else if (error.retryable && attemptedSource.attempts < 3) {
        state = "retry_wait";
        const delayMinutes = attemptedSource.attempts === 1 ? 15 : 60;
        availableAt = new Date(Date.now() + delayMinutes * 60_000).toISOString();
      }
      db.prepare(`
        UPDATE sources
        SET archive_state = ?, available_at = ?, lease_until = NULL, last_error = ?
        WHERE id = ?
      `).run(state, availableAt, error.message, attemptedSource.id);
      console.error(
        `[${completed + 1}/${options.limit}] thread ${attemptedSource.thread_id}: ${error.message}`,
      );
    }
  }
  return completed;
}

async function runLivePilot(db, options) {
  const lockName = "live-pilot";
  const owner = crypto.randomUUID();
  acquireLock(db, lockName, owner);
  try {
    const tab = await findForumPage(options.cdpUrl);
    let completed = 0;
    for (; completed < options.limit; completed += 1) {
      renewLock(db, lockName, owner);
      const job = leaseLiveJob(db);
      if (!job) break;
      const delay = randomDelay(options.minDelayMs, options.maxDelayMs);
      console.log(
        `[${completed + 1}/${options.limit}] waiting ${delay} ms before thread ${job.thread_id}, page ${job.page_number}`,
      );
      await sleep(delay);
      renewLock(db, lockName, owner);
      const attemptedJob = markLiveAttempt(db, job);
      try {
        const response = await fetchLivePage(tab, attemptedJob.url);
        assertLivePage(response);
        recordLiveSuccess(db, options.dataDir, attemptedJob, response);
        console.log(
          `[${completed + 1}/${options.limit}] stored thread ${attemptedJob.thread_id}, page ${attemptedJob.page_number}`,
        );
      } catch (error) {
        recordLiveFailure(db, attemptedJob, error);
        if (error instanceof ChallengeError) throw error;
        console.error(
          `[${completed + 1}/${options.limit}] thread ${attemptedJob.thread_id}: ${error.message}`,
        );
      }
    }
    return completed;
  } finally {
    releaseLock(db, lockName, owner);
  }
}

async function runInventory(db, options) {
  const [wayback, commonCrawl] = await Promise.all([
    loadWaybackInventory(),
    loadCommonCrawlInventory(),
  ]);
  const rss = loadRssInventory(options.highWaterPath);
  importDiscoveries(db, [...wayback, ...commonCrawl, ...rss]);
  console.log(
    `Imported ${wayback.length} Wayback, ${commonCrawl.length} Common Crawl, and ${rss.length} RSS records.`,
  );
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const db = openLedger(options.dataDir);
  let operationError;
  try {
    try {
      if (options.command === "inventory") {
        await runInventory(db, options);
      } else if (options.command === "archive-pilot") {
        await runArchivePilot(db, options);
      } else if (options.command === "pilot") {
        await runLivePilot(db, options);
      } else if (options.command === "unblock") {
        const result = db.prepare(`
          UPDATE jobs
          SET state = 'queued', last_error = NULL, available_at = NULL,
              lease_until = NULL, updated_at = ?
          WHERE state = 'blocked'
        `).run(new Date().toISOString());
        console.log(`Unblocked ${result.changes} live jobs after manual verification.`);
      }
    } catch (error) {
      operationError = error;
    }
    const summary = writeSummary(db, options.summaryPath);
    printSummary(summary);
  } finally {
    db.close();
  }
  if (operationError) throw operationError;
}

if (require.main === module) {
  main().catch((error) => {
    console.error(`Error: ${error.message}`);
    process.exitCode = 1;
  });
}

module.exports = {
  ChallengeError,
  acquireLock,
  assertArchivePage,
  assertLivePage,
  discoverAttachments,
  discoverPageUrls,
  importDiscoveries,
  isoFromCdxTimestamp,
  openLedger,
  parseArgs,
  recordArchiveSuccess,
  releaseLock,
  reclassifyStoredChallenges,
  storeObject,
  threadFromUrl,
  writeSummary,
};
