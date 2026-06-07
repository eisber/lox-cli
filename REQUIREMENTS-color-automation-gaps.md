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
