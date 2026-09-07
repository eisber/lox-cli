# Loxforum research intake

This directory tracks public loxforum.com metadata used to identify material
that may improve lox-cli. It does not mirror forum posts or store login
credentials, browser cookies, or CAPTCHA data.

`high-water.json` is updated manually from the forum's authenticated RSS feed.
The feed discovers newly created threads; it does not expose IDs for subsequent
replies. The collector makes one feed request after a randomized 12-20 second
delay. It does not open topics or download attachments, and it refuses to
update state if the response looks like a security challenge.

## Refresh

1. Start Chrome with a dedicated profile outside the repository:

   ```powershell
   & "$env:ProgramFiles\Google\Chrome\Application\chrome.exe" `
     --user-data-dir="$env:LOCALAPPDATA\loxforum-scraper" `
     --remote-debugging-port=9222 `
     https://www.loxforum.com/forum
   ```

2. Complete any security check and sign in in the browser.
3. Run the collector:

   ```powershell
   node scripts/scrape-loxforum.js
   ```

Use `--dry-run` to inspect the item counts without changing the high-water
file. The browser profile must remain outside the repository.

Forum material should only be incorporated after review. Prefer attributed
summaries and independently authored tests or examples over copying post text.

## Historical inventory and crawl ledger

The larger crawl uses a local SQLite ledger and content-addressed object store.
Both default to `%LOCALAPPDATA%\loxforum-scraper` on Windows and remain outside
Git. Only aggregate progress in `inventory-summary.json` is committed.

Build the inventory without requesting pages from loxforum.com:

```powershell
node scripts/loxforum-crawl.js inventory
node scripts/loxforum-crawl.js archive-pilot --limit 100
```

The inventory combines RSS metadata with Internet Archive and Common Crawl
indexes. `archive-pilot` retrieves snapshots from the Internet Archive, not
from loxforum.com.

After opening the authenticated browser described above, a live pilot can
process at most 25 pages:

```powershell
node scripts/loxforum-crawl.js pilot --limit 25
```

Live requests are serialized with randomized 45-90 second delays. HTTP 403,
HTTP 429, CAPTCHA, BotGuard, or hCaptcha content blocks the current job and
stops the run immediately. After manually restoring access, explicitly resume:

```powershell
node scripts/loxforum-crawl.js unblock
node scripts/loxforum-crawl.js pilot --limit 25
```

Expired leases are recovered automatically after 30 minutes. Server failures
receive no immediate retry: they become eligible after 15 minutes, then 60
minutes, and fail permanently after the third attempt. Attachment URLs are
inventoried but attachment bodies are not downloaded.
