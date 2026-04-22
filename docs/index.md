---
title: Home
layout: default
nav_order: 1
---

<div class="hero">
  <div class="hero-badges">
    <span class="hero-badge"><span class="hero-badge-icon">&#9889;</span> Single binary</span>
    <span class="hero-badge"><span class="hero-badge-icon">&#128274;</span> No cloud</span>
    <span class="hero-badge"><span class="hero-badge-icon">&#129302;</span> AI-ready</span>
    <span class="hero-badge"><span class="hero-badge-icon">&#128230;</span> ~4MB</span>
  </div>

  <h1 class="hero-title">lox</h1>
  <p class="hero-tagline">
    Fast, scriptable CLI for Loxone Miniserver.<br>
    Control lights, blinds, and more from your terminal.
  </p>

  <div class="install-block">
    <div class="install-row">
      <span class="install-label">macOS / Linux</span>
      <code>brew install eisber/lox-cli/lox</code>
    </div>
    <div class="install-row">
      <span class="install-label">Windows</span>
      <code>irm https://raw.githubusercontent.com/eisber/lox-cli/main/install.ps1 | iex</code>
    </div>
  </div>

  <div class="hero-buttons">
    <a href="/lox-cli/getting-started" class="btn-hero btn-hero-primary">Get Started</a>
    <a href="/lox-cli/commands/" class="btn-hero btn-hero-secondary">Command Reference</a>
    <a href="https://github.com/eisber/lox-cli" class="btn-hero btn-hero-secondary">GitHub</a>
  </div>
</div>

<div class="features">
  <div class="feature-card">
    <span class="feature-icon">&#128161;</span>
    <div class="feature-title">Script your home</div>
    <div class="feature-desc">Bash, Python, cron — use any tool. Chain commands with pipes and exit codes like any Unix CLI.</div>
  </div>
  <div class="feature-card">
    <span class="feature-icon">&#129302;</span>
    <div class="feature-title">AI agent ready</div>
    <div class="feature-desc">JSON output, schema discovery, fuzzy matching, dry-run mode, and structured errors designed for LLM tool use.</div>
  </div>
  <div class="feature-card">
    <span class="feature-icon">&#128225;</span>
    <div class="feature-title">Real-time streaming</div>
    <div class="feature-desc">WebSocket state streaming with NDJSON output. Filter by room, type, or specific controls.</div>
  </div>
  <div class="feature-card">
    <span class="feature-icon">&#128202;</span>
    <div class="feature-title">OpenTelemetry</div>
    <div class="feature-desc">Push metrics, logs, and traces to Dynatrace, Datadog, Grafana Cloud, or any OTLP backend.</div>
  </div>
  <div class="feature-card">
    <span class="feature-icon">&#128196;</span>
    <div class="feature-title">Config versioning</div>
    <div class="feature-desc">GitOps for your Miniserver. Track config changes with semantic diffs and git history.</div>
  </div>
  <div class="feature-card">
    <span class="feature-icon">&#128279;</span>
    <div class="feature-title">Multi-context</div>
    <div class="feature-desc">Manage multiple Miniservers with named contexts. Switch with <code>lox ctx use</code> or override per-command with <code>--ctx</code>.</div>
  </div>
  <div class="feature-card">
    <span class="feature-icon">&#9889;</span>
    <div class="feature-title">Fast</div>
    <div class="feature-desc">~80ms warm, ~1.2s cold. Structure cache with 24h TTL. Static Rust binary with zero runtime dependencies.</div>
  </div>
</div>

---

<div class="section-label">Quick start</div>

## See it in action

<div class="terminal">
  <div class="terminal-header">
    <span class="terminal-dot terminal-dot-red"></span>
    <span class="terminal-dot terminal-dot-yellow"></span>
    <span class="terminal-dot terminal-dot-green"></span>
    <span class="terminal-title">Terminal</span>
  </div>
  <div class="terminal-body">
<pre><code><span class="terminal-comment"># Discover your controls</span>
<span class="terminal-prompt">$</span> lox ls --type LightControllerV2 -o json | jq '.[].name'

<span class="terminal-comment"># Control devices</span>
<span class="terminal-prompt">$</span> lox on "Licht Wohnzimmer"
<span class="terminal-prompt">$</span> lox blind "Beschattung Sud" pos 50
<span class="terminal-prompt">$</span> lox thermostat "Heizung" temp 22.5

<span class="terminal-comment"># Conditional automation</span>
<span class="terminal-prompt">$</span> lox if "Temperatur" gt 28 && lox blind "Beschattung Sud" pos 70

<span class="terminal-comment"># Real-time monitoring</span>
<span class="terminal-prompt">$</span> lox stream --room "Kitchen" -o json

<span class="terminal-comment"># GitOps config backup</span>
<span class="terminal-prompt">$</span> lox config pull</code></pre>
  </div>
</div>

---

<div class="section-label">Devices</div>

## Supported control types

| Type | Commands |
|:-----|:---------|
| `LightControllerV2` | `on`, `off`, `mood plus/minus/off/<id>` |
| `Jalousie` / `CentralJalousie` | `up`, `down`, `stop`, `pos <0-100>`, `shade`, `full-up`, `full-down` |
| `Switch` | `on`, `off`, `pulse` |
| `Dimmer` | `dimmer <name> <0-100>` |
| `Gate` / `CentralGate` | `gate <name> open/close/stop` |
| `ColorPickerV2` | `color <name> #RRGGBB` or `hsv(h,s,v)` |
| `IRoomControllerV2` | `thermostat <name> --temp/--mode/--override` |
| `Alarm` | `alarm <name> arm/disarm/quit` |
| `InfoOnlyAnalog` / `Meter` | `get` (read-only) |
| Any | `send <uuid> <raw-command>`, `lock`, `unlock` |

---

<div class="section-label">Performance</div>

## Benchmarks

Structure cache at `~/.lox/cache/structure.json` (24h TTL):

| Operation | Cold | Warm |
|:----------|:-----|:-----|
| `lox on "Licht"` | ~1.2s | **~80ms** |
| `lox ls` | ~1.2s | **~80ms** |
| `lox ls --values` | ~1.2s + N reqs | slower |
| `lox status` | ~120ms | **~120ms** |
