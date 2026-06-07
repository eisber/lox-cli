# Requirements — Simulator & Config gaps (found in a real-world deployment)

Status: **handoff / backlog**. Captured by an agent while configuring a real Loxone
Miniserver (Gen-2, firmware 12.x) with lox-cli **v0.14.2**. The task — adding a
schedule-driven "dim the Touch Nightlight Air at night" automation to a newly-added
device, mirroring two siblings that already have it — could **not be validated offline**
because of the gaps below. All evidence is from a production config downloaded via
`lox config download --extract` (~379 controls, 7 rooms). Names/serial scrubbed.

Priorities: **P0** blocks the core "edit → simulate → push" loop · **P1** forces
raw-XML workarounds · **P2** nice-to-have.

---

## P0-1 · `sim check`/`sim run` hard-fails on a real downloaded config

`lox sim check <real>.Loxone` aborts with:

```
Error: parse_failed: connector 588 is not an output
```

This is a **fatal parse error**, not a PassThrough fallback — it makes *every* sim
command (`check`, `run`, `step`, `dump`) unusable on this config, so no automation in
this house can be validated offline before push. The whole value prop ("self-test with
the offline SPS simulator before anything touches your live system") is unavailable on a
config that the same CLI just downloaded and can otherwise `describe`/`wires`/`check`.

**Requirement**
- The simulator graph builder must not hard-fail on connectors it doesn't recognize.
  Degrade gracefully (skip/PassThrough the offending wire, emit a warning naming the
  block + connector key + uuid) instead of aborting the whole parse.
- Surface *which* block/connector "588" refers to (uuid + title + key), not an opaque
  internal index.
- Add a regression fixture from a real multi-room Gen-2 export (Air + Tree + weather +
  time blocks) so `sim check` on a realistic config stays green.

---

## P0-2 · No way to inject simulated time-of-day → schedule logic is untestable

The house's "off at night" pattern is built from clock blocks:

```
Minuten seit Mitternacht (Time) ─┬─ GreaterEqual (≥300  = after 05:00) ─┐
                                 └─ Less        (<1320 = before 22:00) ─┴─ And ── Mult(×30) ── …BrightInact
```

All of these are unimplemented in the sim (see P0-3) **and** there is no field in the
`--sim` spec to set the simulated wall-clock / minutes-since-midnight / date. So even if
the time blocks were implemented, a schedule/`DayTimer`/sunrise automation still couldn't
be exercised. `config timer-schedule` can *write* schedules the CLI can't *verify*.

**Requirement**
- Extend the sim spec with a clock, e.g. `"clock": {"time": "23:30", "date": "2026-06-07"}`
  (or `minutes_since_midnight`, plus lat/long for sunrise/sunset), and advance it with
  `dt`/`ticks`.
- Make time/astro blocks read from that simulated clock (ties into P0-3).
- Add a worked schedule example to `skills/loxone-sim/SKILL.md` (e.g. "verify a light is
  0 at 23:00 and 30 at 12:00").

---

## P0-3 · Core block types fall back to PassThrough (silently wrong results)

`lox sim check` on this config logs **80+** `Unknown block type '…' is not implemented;
falling back to PassThrough`. PassThrough silently produces wrong values (e.g. a
`GreaterEqual` threshold becomes a wire), so even a partial sim can't be trusted. The most
impactful missing groups for everyday automations:

| Group | Unimplemented types (observed) | Why it matters |
|-------|-------------------------------|----------------|
| **Time / astro** | `Time`, `Hour`, `Minute`, `Second`, `SecondsBoot`, `Day`, `Day2009`, `DayOfWeek`, `Week`, `Month`, `Year`, `DateTime`, `Sunrise`, `Sunset`, `Morningtwilight`, `Eveningtwilight`, `Daylight`, `Daylight2`, `NightTime`, `SunAltitude`, `SunAzimuth`, `StartPulse`, `Impulse{Second,Minute,Hour,Day,Month,Year,Sunrise,Sunset,Morningtwilight,Eveningtwilight}` | Schedules, night/day gating, sun-based shading — the backbone of most real automations |
| **Weather** | `WeatherData`, `WeatherServer`, `GenTSensor`, `GenTActor` | Wind/frost/rain/sun thresholds (the headline README examples) |
| **Loxone Air devices** | `LoxAIRAactor`, `LoxAIRactor` (and `LoxAIRsensor`/`LoxAIRAsensor`/`LoxAIRDevice` referenced) | Touch Nightlight Air, Air actuators/sensors — the actual endpoints you wire to |
| **Tree devices** | `TreeAactor`, `TreeActor` (and `TreeSensor`/`TreeAsensor`/`TreeDevice`) | RGBW dimmers, presence/motion sensors, touch |
| **Schedule/alarm** | `AlarmClock`, `DayTimer` were not exercised here but are the natural targets of `config add`/`timer-schedule` | Wake-up, nightlight, dim windows |
| **System / IO** | `Actor`, `ApiActor`, `Caller`, `CallerVirtualIn`, `IntercomDevice`, `LanInt`, `Logger`, `Mailer`, `Notification`, `MessageCenter`, `Tracker`, `Plugin`, `AutoPilot`, `Devicemonitor`, `GlobalStates`, `LoxLIVE`, `Mode`, `RemoteControls` | Mixed: some need real behavior, many are fine as PassThrough |
| **Structural / captions** (benign as PassThrough, listed for completeness) | `*Caption` (Category/Place/Constant/Memory/Mode/Calendar/Time/Output/AnalogInput/VirtualIn/VirtualOut/Weather/Task/Logger/LoxDevice/UserGroup/User), `User`, `UserGroup`, `Permission`, `RightGroup`, `CalendarEntry`, `PuDe` | OK to no-op; just shouldn't spam warnings |

**Requirement**
- Implement (in rough priority order): time/astro blocks → weather inputs → Air/Tree
  device actor+sensor endpoints → `AlarmClock`/`DayTimer`.
- Distinguish *"intentionally PassThrough (structural/caption)"* from *"unimplemented,
  results unreliable"* — the latter should be a single clearly-flagged warning (or an
  opt-in `--strict` failure), not 80 lines that bury the real ones.
- `sim check` should print a summary footer: `N blocks, M simulated, K passthrough
  (unreliable), J structural`.

---

## P1-1 · Can't introspect a control's connectors / device binding from the CLI

To wire the new device I needed (a) the input connector key+uuid of a `LoxAIRAactor`
("Smartaktor RGBW"), and (b) which `AlarmClock` is bound to which Touch Nightlight device.
Neither is obtainable from the CLI:

- `lox config get-params <file> "uuid:…(LoxAIRAactor)"` prints only the header
  `Smartaktor RGBW (LoxAIRAactor):` with **no params and no connectors**.
- `lox config wires` only lists connectors that are *already wired*, so the unwired input
  `<Co K="I" U="…-6652-00ff…">` is invisible.
- `config controls`/`describe` never surface the `Dev="…"` attribute that binds an
  `AlarmClock` to its `LoxAIRDevice`. I had to grep the raw XML to discover that the
  sibling AlarmClock has `Dev="<deviceA>"` while the generic "Wecker" AlarmClock has **no
  `Dev=`** (i.e. it is *not* the new device's clock).

This forced manual `Get-Content … | IndexOf('<Co K=…')` raw-XML spelunking for every
target — exactly what the CLI is supposed to abstract.

**Requirement**
- Add `lox config connectors <file> <selector> [-o json]` that lists every connector for a
  control: `key`, `uuid`, `direction (I/O)`, `wired?`, and `source/target` if wired.
- Have `get-params` (or `describe <selector>`) include connectors and the resolved device
  binding (`Dev=` → device uuid+title) for device-backed controls.
- Let `wire-connector` accept an unambiguous `"uuid:<actor>.<key>"` target (titles like
  "Smartaktor RGBW" are duplicated per device, so title-based selection is ambiguous).

---

## P1-2 · No way to create/bind a device-bound `AlarmClock` (Touch Nightlight nightlight)

The clean fix for the newly-added device is to give it a device-bound `AlarmClock`
(`Dev=<device>`) whose `BrightInact` is fed by the existing day-gating signal — exactly
how the two sibling devices are set up. But:

- `config add --type` only supports `light, switch, presence, alarm-clock, memory, timer,
  mqtt-sub, mqtt-pub, calendar, autopilot`. `alarm-clock` (if it maps to `AlarmClock`)
  has no way to set `Dev=` to bind it to a specific `LoxAIRDevice`.
- There's no template/recipe for "Touch Nightlight Air nightlight" (bind AlarmClock to
  device + wire `BrightInact`).

**Requirement**
- Allow `config add --type alarm-clock --device "<LoxAIRDevice>"` to emit the `Dev=`
  binding, plus the standard nightlight connectors.
- Add a `loxone-patterns` recipe: "Touch Nightlight Air — dim/off at night" (the
  `Time → GreaterEqual/Less → And → Mult → AlarmClock.BrightInact` chain), so a newly
  added Air device can be brought in line with existing ones.

---

## P1-3 · RGBW / color value encoding is undocumented and has no helper

The "Smartaktor RGBW" input uses display unit `<v.col>` (a composite Loxone color value).
There's no documentation of the encoding (RGB-percent vs Lumitech/tunable-white vs
temperature) and no CLI helper, so I can't compute "warm amber @ 15%" to feed an actor or
a `set-param` with confidence — and (per P0) can't sim-verify the result either.

**Requirement**
- Document the Loxone composite color encodings (RGB, Lumitech, temperature+brightness) in
  `loxone-config/SKILL.md`.
- Add a helper, e.g. `lox color encode --rgb 100,40,0 --brightness 15` /
  `--kelvin 2700 --brightness 15`, that prints the integer value to use in
  `set-param`/`wire`, with the inverse `lox color decode <value>`.

---

## P2 · Smaller items

- **`config check` vs `sim check` confusion**: `config check` passed structurally while
  `sim check` couldn't even parse the same file — worth aligning messaging so users know
  structural validation ≠ behavioral validation.
- **Warning volume**: 80+ identical-shaped warnings on a normal config train users to
  ignore them; collapse duplicates and rank by impact.
- **`describe` ordering**: device sub-controls (sensors/actors of one physical device) are
  interleaved; grouping by parent `LoxAIRDevice`/`TreeDevice` would make "which actor
  belongs to which device" obvious without reading `Dev=` from XML.

---

### Repro (sanitized)

```bash
lox config download --extract                 # real Gen-2 export, ~379 controls, 7 rooms
lox config describe   <file>.Loxone           # OK
lox config wires      <file>.Loxone -o json   # OK (only shows *wired* connectors)
lox config get-params <file>.Loxone "uuid:<LoxAIRAactor>"   # header only — no params/connectors
lox sim check         <file>.Loxone           # 80+ PassThrough warnings, then:
                                              #   Error: parse_failed: connector 588 is not an output
```
