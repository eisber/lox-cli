# Requirements — Color / presence-automation gaps (2nd real-world wave)

Status: **handoff / backlog**. Captured by an agent while building a **Teams-status
nightlight** automation on a real Loxone Miniserver (Gen-2, fw 12.x) with lox-cli
**v0.14.2**. Goal: drive a Touch Nightlight Air **RGBW** light to presence colors
(green/yellow/orange/red) Mon–Fri 17:00–23:00, else normal Lichtsteuerung.

The earlier `REQUIREMENTS-sim-config-gaps.md` (sim parse, time/astro/Air blocks,
device-bound AlarmClock) is largely **resolved** — this file is the gaps the *color +
selector + live-verify* work hit next. Same priority scheme: **P0** blocks the
edit→sim→push loop · **P1** forces raw-XML / raw-jdev workarounds · **P2** nice-to-have.

All evidence is from a live config (`config download --extract`) + live `jdev` probes on
the production Miniserver this session. UUIDs are from that export.

---

## P0-1 · `sim` models `AnalogMultiplexer2` with an **unverified, likely-wrong** selector

`lox-sim/src/blocks/math.rs` (`AnalogMultiplexer2`):

```rust
// WARNING: Assumed behavior — Loxone internal implementation unknown.
let selector = inputs.last().copied().unwrap_or(0.0);
vec![if is_high(selector) { i2 } else { i1 }]   // 0 -> Input1, 1 -> Input2
```

The sibling `AnalogMultiplexer` (4-way) is **1-based** and the unit tests assert
`Select=0 -> output 0` (off), `1 -> Input1`, `2 -> Input2`. On the **real Miniserver**,
an `AnalogMultiplexer2` fed `Select = override_active (0/1)` behaved 1-based too:
`Select=0` drove the actor to **0 / off**, not "pass Input1". So the sim's 0→Input1 / 1→Input2
mapping is the opposite of observed hardware. This is a **silent sim-pass / live-fail**:
our 10/10-green matrix asserted the normal-mode cases against `0` (which coincidentally
equals an off LightController output), masking that real `Select=0` means *off*, and that
selecting the color leg needs `Select=2`, not `1`.

**Requirement**
- Verify `AnalogMultiplexer2` against a real Miniserver and fix the model. Strong
  hypothesis: it is 1-based identical to `AnalogMultiplexer` (`0=off, 1=Input1, 2=Input2`),
  i.e. a 2-input case of the same block, **not** a digital `0/1` toggle.
- Until verified, `sim check` should **warn** when an `AnalogMultiplexer2` selector is
  driven by a digital `0/1` signal (the common foot-gun), suggesting `Select = sel ? 2 : 1`.
- Add a regression fixture: selector sweep `0,1,2` → expected `0, I1, I2`.

---

## P0-2 · No way to set an **arbitrary color**; hsv() to a LightController2 is silently ignored

Driving color turned out to have **no offline-verifiable, CLI-supported path** on RGBW:

- `LightController2` over jdev accepts `on`/`off`/`hsv(h,s,v)` (all **Code 200**), but the
  color is **mood-locked**: `on` activates the configured warm-white mood and `hsv(...)` is
  **ignored** (light stayed warm-white/amber, never green). Setting an arbitrary color
  requires **predefined moods**, which lox-cli cannot author.
- The RGBW endpoint `LoxAIRAactor` is **color-typed**: a jdev read returns the literal
  template `<v.col>` (not a number), `hsv(...)` to it → **Code 500**, and a **numeric
  composite written directly is overwritten every tick** by the owning LightController
  (the actor's `OutputRef.AI` is fed by `LightController.AQ1`). So you cannot empirically
  find "green" by poking the actor while the controller owns it.
- Only viable arbitrary-color path found: **detach** the actor by re-pointing its
  `OutputRef.AI` away from `LightController.AQ1` to a value source (mux / VI), then feed a
  composite. That re-point currently needs raw-XML edits (see P1-2).

**Requirement**
- Support **reading, adding and selecting `LightController2` moods** (the mood UUIDs live in
  `uuidSeqencing="…"`). Expose e.g. `lox config moods <file> <selector>` and
  `config add-mood --color hsv(120,100,30) --title "Teams green"`, plus a jdev-side
  `select mood N`. This is the *native* way to get a guaranteed-correct color.
- Document in `loxone-config/SKILL.md`: **hsv()/temp() sent to a LightController2 does not
  set an arbitrary color** — it only re-colors a mood that allows it; otherwise predefine
  moods or detach the actor.
- Carry the **RGBW composite encoding** doc + `lox color encode/decode` helper over from the
  prior file's P1-3 (still open) — but note it is only usable on a **detached** actor.

---

## P1-1 · `config add` has no `virtual-input` type

The automation needs Virtual Inputs as HTTP-push targets (the Teams flags, plus an analog
"Teams Color" value). `config add --type` only supports
`light, switch, presence, alarm-clock, memory, timer, mqtt-sub, mqtt-pub, calendar,
autopilot`. So **every VI had to be hand-inserted as raw XML** (digital `Analog="false"`
and analog `Analog="true"` `VirtualIn` blocks, with hand-picked `IName="VInn"`, UUIDs, and
`IoData Pr/Cr` room/category refs copied from a sibling).

**Requirement**
- `lox config add --type virtual-input [--analog] [--min N --max N] [--unit "<v>"]
  --title "…" [--room …]` that emits a valid `VirtualIn` (jdev-pushable via
  `/jdev/sps/io/<uuid>/<value>`), auto-assigns the next free `IName`, and wires
  `IoData Pr/Cr`. Analog VIs must **not** default to a clamping `MaxVal` (the stock
  `Eingang VIn` has `MaxVal="100"`, which silently clamps a color composite).

---

## P1-2 · Can't add or re-point an `OutputRef` from the CLI (actor "detach")

Re-pointing a `LoxAIRAactor`'s `OutputRef.AI` (to splice an override / detach the actor from
its LightController) is the core move for color control — and there is no CLI support, so it
was done with raw-XML string replacement on the `<Co K="AI">…<In Input="…"/>` node.
`wire-connector` could not target it reliably.

**Requirement**
- Let `wire-connector` target `OutputRef.AI` (e.g. `"uuid:<outputref>.AI"` or
  `"<actor-title>.RGBW.AI"`) and **replace** its source.
- A higher-level `config splice-actor <file> <actor> --source <uuid>` (re-point `OutputRef.AI`
  to a new source, optionally via a generated mux with the old source as the fallback input)
  would make the override pattern a one-liner.

---

## P1-3 · `wire-connector` mishandles `uuid:` sources and **appends** instead of replacing

Two concrete foot-guns observed:
1. Passing a **`uuid:`-prefixed source** stored the literal string `Input="uuid:…"`
   (a broken/dangling ref) and added a spurious `FLG="2"`. Sources must be passed **bare**
   (no `uuid:`), or the prefix must be stripped.
2. Wiring a connector that **already has a source appended a second `<In>`** rather than
   replacing it — an RS-flip-flop ended up with two sources on both `InputS` and `InputR`.
   Had to fix by hand-editing XML.

**Requirement**
- Strip/accept `uuid:` on the **source** arg (it's already accepted on the target).
- Replace an existing single source by default (or require `--add` to append / `--force` to
  overwrite), and **validate** that the resulting `Input=` is a real output connector.

---

## P1-4 · `sim` doesn't model `LightController2.OutputReset` → "double-tap disable" untestable

The natural manual-disable gesture is the Touch **double-tap** = LightController2
"leave room" → its `OutputReset` output pulse, used to reset an arm-latch. `LightController2`
in the sim emits only `[AQ1, AQ2, Scene, PresenceActive]` (`blocks/controllers.rs`), so
`OutputReset` can't be exercised offline and the disarm path had to be **deferred** to a live
gesture test.

**Requirement**
- Add `OutputReset` (and document its pulse semantics) to the `LightController2` sim model,
  and to `--sim` inputs so a "double-tap" can be injected.

---

## P1-5 · No live read/write helper → every verification is hand-rolled `jdev` curl

lox-cli is config-as-code only, so confirming the live result (VI echoes, actor state, MS
clock, driving inputs for a state-walk) required raw
`Invoke-WebRequest https://<ms>/jdev/sps/io/<uuid>[/<value>]` with basic-auth +
`-SkipCertificateCheck` for **every** check. Also: a color/composite actor read returns the
template `<v.col>`, not a value — there's no way to read back what color is actually showing.

**Requirement**
- A thin, read-only-by-default `lox live get <uuid>` / `lox live set <uuid> <value>` wrapping
  `jdev/sps/io` (reuse the configured host/creds, honor the self-signed cert), plus
  `lox live time`. Gate writes behind `--write`/confirmation.
- Document that color-typed states return `<v.col>` over HTTP polling and are **not**
  readable as a number (need the websocket/visu channel) — so verification of *perceived*
  color stays a human-in-the-loop step.

---

## P2 · Smaller items

- **`config check` false-positive errors**: a clean real export reports **12 "errors"** of
  the form `UUID '00000000-0000-0002-1' missing serial suffix '234d69b98eb1'` for operating
  modes (`Feiertag/Urlaub/Freier Tag`) and `205d5330-…-f` registered devices. These are
  legitimate short-form system UUIDs in every Gen-2 export, but they set a non-zero exit and
  bury real errors. Treat known system/mode/registered-device short UUIDs as **info**, not
  errors.
- **`config push --file <f>` is rejected** — `FILE` is positional (`config push <f> --force`),
  yet the staged-push hint prints `Apply with: lox config push --file … --reboot --force`,
  which then errors `unexpected argument '--file'`. Fix the hint (or accept `--file`).
- **Two-step push is non-obvious**: `config push <f> --force` *uploads* `sps_new.zip` but does
  **not** apply; you must re-run with `--reboot --force` to trigger the fast `/wsx` reload.
  The first run should say so unambiguously (it half-does).

---

### Repro (sanitized)

```bash
lox config download --extract
# color: no CLI path — had to hand-insert VirtualIn XML + raw-edit OutputRef.AI, then:
curl -sk https://<ms>/jdev/sps/io/<lightcontroller>/hsv(120,100,30)   # Code 200 but ignored
curl -sk https://<ms>/jdev/sps/io/<rgbw-actor>                        # -> "<v.col>" (not a number)
curl -sk https://<ms>/jdev/sps/io/<rgbw-actor>/hsv(120,100,30)        # Code 500
lox config add --type virtual-input …            # ERROR: no such type
lox config push <f> --force                       # uploads sps_new.zip, does NOT apply
lox config push --file <f> --reboot --force       # ERROR: unexpected argument '--file'
lox config push <f> --reboot --force              # OK: fast /wsx reload (~4s)
```

---

## Wave 3 (2026-06-11) — follow-ups from a live Teams-nightlight brightness change + weather VI fix

Context: changed Markus's 4 Teams moods from 5%→10% brightness, and fixed a weather VI that read 0. Both required **raw-XML edits** despite the relevant verbs now existing.

### W3-1 · `config moods` is read-only — no way to EDIT a mood's stored color/brightness (extends P0-2)
`config moods <file> <selector>` now **lists** moods (good), and `splice-actor` + the fixed `lox color` base-1000 encoder landed — but there is still **no command to change an existing mood's color/brightness**. To re-brightness the 4 Teams moods 5%→10% I had to hand-edit the `Q1` attribute of each `<LightsceneC>` on the `LightController2`.

**Crucial implementation fact (please bake into any `set-mood-color`/`add-mood`):** a mood's per-output value is **NOT** the same scale as the actor `<v.col>` composite that `color_cmd.rs` documents (`R+G*1000+B*1000000`, channels **0..255**). The `LightsceneC` `Q1` packing is **percent channels (0..100)** plus a mood-type prefix:

```
Q1 = 0x60000000 + (R% + G%*1000 + B%*1000000)     # each channel 0..100
  green @ 5%  = 0x60000000 + 5000    = 1610617736   (hsv(120,100,5))
  green @ 10% = 0x60000000 + 10000   = 1610622736   (hsv(120,100,10))
  orange@~5%  = 0x60000000 + 1002005 = 1611614741   (R5 G2 B1)
```
Scaling all channels x2 cleanly doubles V while preserving hue/sat. Verified live via the WS state read (`activeMoods.color` -> `hsv(120,100,10)` after the edit).

**Requirement**
- `lox config set-mood-color <file> <controller-selector> --mood <SID|name> --color hsv(...)|rgb(...)` that rewrites the mood's `Q1..Qn` using the **percent + `0x60000000`** packing.
- `lox color` should optionally emit the **mood** form (percent + prefix), not only the 0..255 actor composite — they are different and easy to confuse.

**✅ Resolved (2026-06-11).** Both landed:
- `lox config set-mood-color <file> <selector> --mood <SID|Name> --color hsv(...)|rgb(...) [--output-index N]` rewrites the matched `<LightsceneC>`'s `Q{N}` (default Q1) using the percent + `0x60000000` packing. Verified: TeamsGruen 5%→10% rewrote `Q1="1610617736"`→`"1610622736"`.
- `lox color encode --mood` emits the mood form (`mood_value`, percent channels), and `lox color decode <mood_value>` recognises the `0x60000000` prefix. Matches the documented examples exactly (green@10% = 1610622736, orange = R5/G2/B1). The note clarifies mood-form ≠ actor `<v.col>` composite.
- `config moods` already shows the live-select hint (`lox live set <ctrl> changeTo/<SID> --write`).
- `config add-mood` (create a brand-new mood) is now **✅ Resolved (2026-06-12)** — see W3-5 below.

### W3-5 · `config add-mood` — author a brand-new LightController2 mood

**✅ Resolved (2026-06-12).** `lox config add-mood <file> <controller-selector> --name <name> --color hsv(...)|rgb(...) [--sid N] [--cid N]` appends a fresh `<LightsceneC>` to the controller's `<LightscenesC>` container with:
- a generated Loxone-format UUID (correct Miniserver serial suffix),
- the next free **custom** scene id (`SID`, base 2 — reserved 1/776/777/778 skipped) and color id (`CID`, base 9),
- `Outputs` mirrored from the container, `Q1` = packed mood value, `Q2..Qn` = 0,
- the container's `Num` count incremented.

Validated against the real config (`sps_0267_20260611184427.Loxone`, Markus controller): adding `TeamsPurple` after `TeamsYellow` (SID 5 / CID 12) correctly produced SID 6 / CID 13, matching Loxone's own numbering. `--sid`/`--cid` override the defaults; duplicate name or SID is rejected.

### W3-2 · Can't `set-param` an existing `VirtualIn`'s analog range (MaxVal/MinVal) — clamps silently (extends P1-1)
Real second instance of the P1-1 clamping hazard: the weather pressure VI (`pressure_msl`) had the stock `MaxVal="1000"`, but real sea-level pressure is ~1024 hPa -> **out of range -> the VI read `0`**. Fix required a raw-XML edit of `MaxVal="1000"->"1100"` because **`config set-param` cannot target `MaxVal`/`MinVal`/`MinChange` on a `VirtualIn`** (grep of `src/commands/*.rs` shows no `MaxVal`/`MinVal` handling at all).

**Requirement**
- Let `config set-param <file> <vi-selector> MaxVal 1100` (and `MinVal`, `MinChange`, `MinTime`) work on `VirtualIn`/analog inputs.
- `config check` lint: warn on an analog `VirtualIn` whose `MaxVal` is implausibly low for its purpose (esp. a pressure VI with `MaxVal <= 1013` — guaranteed to read 0 at normal sea-level pressure). Silent, hard-to-spot failure.

**✅ Resolved (2026-06-11).** Both landed:
- `ConfigEditor::set_param` now falls back to setting an element **attribute** when the param isn't a `<Co>` connector and is one of `MaxVal`/`MinVal`/`MinChange`/`MinTime`/`Step`. So `config set-param <vi> MaxVal 1100` works; unknown params still error.
- `config check` warns when an analog `VirtualIn` whose title/`Unit` looks like pressure (`druck`/`press`/`hpa`/`mbar`) has `MaxVal <= 1013`.

### W3-4 · (new, 2026-06-11) `config check` warns when a config exceeds the Program-block limit
Adding a sample "hello world" Program (Code) block surfaced the Miniserver limit: only the **first 8 Program blocks** (`Code1/Code4/Code8/Code16`) are executed; the rest are silently ignored (Loxone Config warns about this on save).

**✅ Resolved (2026-06-11).** `config check` now counts `Code1/4/8/16` blocks and warns when there are more than 8, naming the blocks that will be ignored.

### W3-3 · (positive) push hint is now correct
The staged-push hint now prints the **positional** apply form (`lox config push <file> --reboot --force`), not the broken `--file` form from P2.
