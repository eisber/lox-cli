#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const FORUM_ORIGIN = "https://www.loxforum.com";
const FEED_URL = `${FORUM_ORIGIN}/external?type=rss2`;
const DEFAULT_CDP_URL = "http://127.0.0.1:9222";
const DEFAULT_STATE_PATH = path.join(
  __dirname,
  "..",
  "docs",
  "loxforum",
  "high-water.json",
);
const DEFAULT_MIN_DELAY_MS = 12_000;
const DEFAULT_MAX_DELAY_MS = 20_000;
const MAX_RECENT_ITEMS = 100;

function parseArgs(argv) {
  const options = {
    cdpUrl: DEFAULT_CDP_URL,
    statePath: DEFAULT_STATE_PATH,
    minDelayMs: DEFAULT_MIN_DELAY_MS,
    maxDelayMs: DEFAULT_MAX_DELAY_MS,
    dryRun: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--dry-run") {
      options.dryRun = true;
      continue;
    }

    const values = {
      "--cdp-url": "cdpUrl",
      "--state": "statePath",
      "--min-delay-ms": "minDelayMs",
      "--max-delay-ms": "maxDelayMs",
    };
    const key = values[arg];
    if (!key || i + 1 >= argv.length) {
      throw new Error(`Unknown or incomplete argument: ${arg}`);
    }
    options[key] = key.endsWith("Ms") ? Number(argv[++i]) : argv[++i];
  }

  if (
    !Number.isInteger(options.minDelayMs) ||
    !Number.isInteger(options.maxDelayMs) ||
    options.minDelayMs < 0 ||
    options.maxDelayMs < options.minDelayMs
  ) {
    throw new Error("Delay values must be integers with 0 <= min <= max");
  }

  return options;
}

function decodeXml(value) {
  return value
    .replace(/^<!\[CDATA\[([\s\S]*)\]\]>$/i, "$1")
    .replace(/&#x([0-9a-f]+);/gi, (_, hex) =>
      String.fromCodePoint(Number.parseInt(hex, 16)),
    )
    .replace(/&#(\d+);/g, (_, decimal) =>
      String.fromCodePoint(Number.parseInt(decimal, 10)),
    )
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&apos;/g, "'")
    .replace(/&amp;/g, "&")
    .trim();
}

function readElement(xml, name) {
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = xml.match(
    new RegExp(`<${escapedName}(?:\\s[^>]*)?>([\\s\\S]*?)</${escapedName}>`, "i"),
  );
  return match ? decodeXml(match[1]) : "";
}

function extractPostId(...values) {
  for (const value of values) {
    const match = value.match(/(?:#post|[?&](?:p|postid)=)(\d+)/i);
    if (match) {
      return Number.parseInt(match[1], 10);
    }
  }
  return null;
}

function extractThreadId(url) {
  try {
    const match = new URL(url).pathname.match(/\/(\d+)(?:-|\/|$)/);
    return match ? Number.parseInt(match[1], 10) : null;
  } catch {
    return null;
  }
}

function parseRss(xml) {
  const items = [];
  for (const match of xml.matchAll(/<item(?:\s[^>]*)?>([\s\S]*?)<\/item>/gi)) {
    const itemXml = match[1];
    const url = readElement(itemXml, "link");
    const guid = readElement(itemXml, "guid");
    const published = readElement(itemXml, "pubDate");
    const publishedDate = new Date(published);

    items.push({
      post_id: extractPostId(url, guid),
      thread_id: extractThreadId(url),
      title: readElement(itemXml, "title").replace(/<[^>]*>/g, "").trim(),
      url,
      guid,
      published_at: Number.isNaN(publishedDate.valueOf())
        ? null
        : publishedDate.toISOString(),
    });
  }
  return items;
}

function assertFeedResponse(response) {
  const contentType = response.contentType || "";
  const prefix = response.text.slice(0, 1_000).toLowerCase();
  if (
    response.status !== 200 ||
    !/^(?:application|text)\/(?:rss\+xml|xml)\b/i.test(contentType) ||
    !/<rss(?:\s|>)/i.test(response.text) ||
    /captcha|security check required|botguard/.test(prefix)
  ) {
    throw new Error(
      `Forum feed unavailable or challenged (HTTP ${response.status}, ${contentType || "no content type"}); high-water mark was not changed`,
    );
  }
}

function loadState(statePath) {
  if (!fs.existsSync(statePath)) {
    return {
      schema_version: 1,
      source_feed: FEED_URL,
      updated_at: null,
      highest_thread_id: 0,
      latest_published_at: null,
      last_run: null,
      recent_items: [],
    };
  }
  return JSON.parse(fs.readFileSync(statePath, "utf8"));
}

function laterTimestamp(first, second) {
  if (!first) return second;
  if (!second) return first;
  return new Date(first) >= new Date(second) ? first : second;
}

function updateState(previous, items, updatedAt) {
  const threadIds = items
    .map((item) => item.thread_id)
    .filter((threadId) => Number.isInteger(threadId));
  const highestFeedThreadId = threadIds.length ? Math.max(...threadIds) : 0;
  const previousThreadId = previous.highest_thread_id || 0;
  const newThreads = items.filter(
    (item) => item.thread_id !== null && item.thread_id > previousThreadId,
  );

  const merged = new Map();
  for (const item of [...(previous.recent_items || []), ...items]) {
    const key = item.guid || item.url;
    if (key) merged.set(key, item);
  }
  const recentItems = [...merged.values()]
    .sort((a, b) => (b.thread_id || 0) - (a.thread_id || 0))
    .slice(0, MAX_RECENT_ITEMS);

  const latestFeedTimestamp = items.reduce(
    (latest, item) => laterTimestamp(latest, item.published_at),
    null,
  );

  return {
    schema_version: 1,
    source_feed: FEED_URL,
    updated_at: updatedAt,
    highest_thread_id: Math.max(previousThreadId, highestFeedThreadId),
    latest_published_at: laterTimestamp(
      previous.latest_published_at,
      latestFeedTimestamp,
    ),
    last_run: {
      items_seen: items.length,
      new_threads: newThreads.length,
    },
    recent_items: recentItems,
  };
}

function writeJsonAtomic(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporaryPath = `${filePath}.${process.pid}.tmp`;
  try {
    fs.writeFileSync(temporaryPath, `${JSON.stringify(value, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
    });
    fs.renameSync(temporaryPath, filePath);
  } finally {
    if (fs.existsSync(temporaryPath)) fs.unlinkSync(temporaryPath);
  }
}

function randomDelay(minimum, maximum) {
  return minimum + Math.floor(Math.random() * (maximum - minimum + 1));
}

function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function findForumPage(cdpUrl) {
  const response = await fetch(`${cdpUrl.replace(/\/$/, "")}/json/list`);
  if (!response.ok) {
    throw new Error(`Cannot inspect Chrome DevTools tabs: HTTP ${response.status}`);
  }
  const tabs = await response.json();
  const tab = tabs.find(
    (candidate) =>
      candidate.type === "page" &&
      candidate.url.startsWith(`${FORUM_ORIGIN}/`) &&
      candidate.webSocketDebuggerUrl,
  );
  if (!tab) {
    throw new Error(
      "No loxforum.com tab found. Open the forum in the authenticated Chrome profile first.",
    );
  }
  return tab;
}

function evaluate(tab, expression) {
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(tab.webSocketDebuggerUrl);
    const requestId = 1;

    socket.addEventListener("open", () => {
      socket.send(
        JSON.stringify({
          id: requestId,
          method: "Runtime.evaluate",
          params: {
            expression,
            awaitPromise: true,
            returnByValue: true,
          },
        }),
      );
    });
    socket.addEventListener("error", () => {
      reject(new Error("Chrome DevTools WebSocket connection failed"));
    });
    socket.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (message.id !== requestId) return;
      socket.close();
      if (message.error || message.result?.exceptionDetails) {
        reject(
          new Error(
            message.error?.message ||
              message.result.exceptionDetails.text ||
              "Browser evaluation failed",
          ),
        );
        return;
      }
      resolve(message.result?.result?.value);
    });
  });
}

async function fetchFeed(tab) {
  const expression = `(
    async () => {
      const response = await fetch(${JSON.stringify(FEED_URL)}, {
        credentials: "include",
        headers: {
          "Accept": "application/rss+xml, application/xml;q=0.9, text/xml;q=0.8"
        }
      });
      return JSON.stringify({
        status: response.status,
        contentType: response.headers.get("content-type") || "",
        text: await response.text()
      });
    }
  )()`;
  const result = await evaluate(tab, expression);
  if (typeof result !== "string") {
    throw new Error("Browser returned no feed response");
  }
  return JSON.parse(result);
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const tab = await findForumPage(options.cdpUrl);
  const delay = randomDelay(options.minDelayMs, options.maxDelayMs);
  console.log(`Authenticated forum tab found; waiting ${delay} ms before one RSS request...`);
  await sleep(delay);

  const response = await fetchFeed(tab);
  assertFeedResponse(response);
  const items = parseRss(response.text);
  if (!items.length) {
    throw new Error("RSS feed contained no items; high-water mark was not changed");
  }

  const previous = loadState(options.statePath);
  const next = updateState(previous, items, new Date().toISOString());
  if (!options.dryRun) {
    writeJsonAtomic(options.statePath, next);
  }

  console.log(
    `${options.dryRun ? "Would record" : "Recorded"} ${items.length} feed items, ` +
      `${next.last_run.new_threads} new; highest thread ID ${next.highest_thread_id}.`,
  );
}

if (require.main === module) {
  main().catch((error) => {
    console.error(`Error: ${error.message}`);
    process.exitCode = 1;
  });
}

module.exports = {
  assertFeedResponse,
  decodeXml,
  extractPostId,
  extractThreadId,
  parseArgs,
  parseRss,
  updateState,
  writeJsonAtomic,
};
