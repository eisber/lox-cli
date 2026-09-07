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
