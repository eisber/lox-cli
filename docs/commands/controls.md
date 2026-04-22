---
title: Controls
layout: default
parent: Command Reference
nav_order: 1
---

# Control Commands
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## On / Off

Turn controls on or off by name, UUID, or alias.

```bash
lox on "Licht Wohnzimmer"
lox off "Licht Wohnzimmer"
```

Apply to all controls in a room:

```bash
lox on --all-in-room "Wohnzimmer"
lox off --all-in-room "Schlafzimmer"
```

| Flag | Description |
|:-----|:------------|
| `-r`, `--room` | Disambiguate by room name |
| `--all-in-room` | Apply to all controls in the specified room |

---

## Lights

### Moods

```bash
lox light mood "Licht Wohnzimmer" plus      # next mood
lox light mood "Licht Wohnzimmer" minus     # previous mood
lox light mood "Licht Wohnzimmer" off       # all off (mood 778)
lox light mood "Licht Wohnzimmer" 704       # set by mood ID
lox light moods "Licht Wohnzimmer"          # list available moods (via WebSocket)
lox light moods "Licht" -o json             # moods as JSON with control metadata
```

### Dimmer

```bash
lox light dim "Stehlampe" 75               # set dimmer 0-100%
```

### Color

```bash
lox light color "LED Strip" "#FF0000"              # hex RGB
lox light color "LED Strip" "hsv(120,100,100)"     # HSV
```

---

## Blinds

```bash
lox blind "Beschattung Sud" up
lox blind "Beschattung Sud" down
lox blind "Beschattung Sud" stop
lox blind "Beschattung Sud" pos 50          # position 0-100%
lox blind "Beschattung Sud" shade           # automatic shading
lox blind "Beschattung Sud" full-up
lox blind "Beschattung Sud" full-down
```

| Action | Description |
|:-------|:------------|
| `up` | Move blind up |
| `down` | Move blind down |
| `stop` | Stop movement |
| `pos <0-100>` | Set exact position (0=open, 100=closed) |
| `shade` | Automatic shading position |
| `full-up` | Move fully up |
| `full-down` | Move fully down |

---

## Gates

```bash
lox gate "Garagentor" open
lox gate "Garagentor" close
lox gate "Garagentor" stop
```

---

## Thermostats

```bash
lox thermostat "Heizung" temp 22.5              # set comfort temperature
lox thermostat "Heizung" mode auto               # auto | manual | comfort | eco
lox thermostat "Heizung" override 24 120         # override 24C for 120 minutes
lox thermostat "Heizung"                         # show current state
```

---

## Alarm

```bash
lox alarm "Alarmanlage" arm
lox alarm "Alarmanlage" arm --no-motion    # arm without motion detection
lox alarm "Alarmanlage" arm-home
lox alarm "Alarmanlage" disarm
lox alarm "Alarmanlage" quit               # acknowledge / silence
```

| Flag | Description |
|:-----|:------------|
| `--no-motion` | Arm without motion detection |
| `--code` | Provide access code |
| `-r`, `--room` | Disambiguate by room |

---

## Door locks

```bash
lox door "Haustur" lock
lox door "Haustur" unlock
lox door "Haustur" open        # open (e.g. electric strike)
```

---

## Intercom

```bash
lox intercom "Turklingel" answer
lox intercom "Turklingel" hangup
lox intercom "Turklingel" open
```

---

## EV Charger

```bash
lox charger "Wallbox" start
lox charger "Wallbox" stop
lox charger "Wallbox" pause
lox charger "Wallbox" start --limit 16     # set current limit
```

---

## Analog / Virtual inputs

```bash
lox input set "Sollwert Heizung" 21.5
lox input set "Sollwert Heizung" 21.5 -r "Wohnzimmer"
lox input pulse "Taster"
```

---

## Music server

```bash
lox music play                    # play zone 1
lox music pause 2                 # pause zone 2
lox music stop
lox music volume 50               # volume 0-100
lox music next
lox music prev
lox music mute
lox music unmute
```

All music commands accept an optional zone number (default: 1).

---

## Lock / Unlock controls

Administrative lock to prevent changes:

```bash
lox lock "Heizung" --reason "Wartung"
lox unlock "Heizung"
```

---

## Raw commands

Send any command directly to a control by UUID:

```bash
lox send <uuid> <command>
lox send <uuid> <command> --secured <hash>
```

---

## Scenes

Run multi-step automation scenes:

```bash
lox run abend                    # run a scene
lox run abend --dry-run          # preview without executing
```

See the [Scenes guide](/lox-cli/guides/scenes) for defining scenes.
