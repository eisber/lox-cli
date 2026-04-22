---
title: OpenTelemetry
layout: default
parent: Guides
nav_order: 4
---

# OpenTelemetry Export
{: .no_toc }

Push metrics, logs, and traces to any OTLP-compatible backend.
{: .fs-5 .fw-300 }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Overview

`lox otel` exports your smart home data via the OpenTelemetry Protocol (OTLP) to backends like Dynatrace, Datadog, Grafana Cloud, Prometheus, or any OTLP-compatible collector.

### What gets exported

**Metrics**: Control state gauges, system diagnostics, SD card health, network counters (CAN/LAN), weather data.

**Logs**: State change events, text-state messages, Miniserver system log entries.

**Traces**: Synthetic automation traces — correlates autopilot rule fires with temporally-close state changes.

### Metric reference

| Metric | Unit | Description |
|:-------|:-----|:------------|
| `loxone.system.heap` | kBy | Heap memory usage |
| `loxone.system.cpu` | % | CPU load |
| `loxone.system.tasks` | {tasks} | Task count |
| `loxone.system.interrupts` | {interrupts} | Hardware interrupt count |
| `loxone.system.comm_interrupts` | {interrupts} | Communication interrupt count |
| `loxone.system.context_switches` | — | Context switch count (cumulative) |
| `loxone.system.context_switches_idle` | — | Context switches idle (cumulative) |
| `loxone.sd.read_speed` | kBy/s | SD card read speed |
| `loxone.sd.write_speed` | kBy/s | SD card write speed |
| `loxone.sd.errors` | {errors} | SD card error count |
| `loxone.sd.usage` | % | SD card storage usage |
| `loxone.can.packets_sent` | — | CAN bus packets sent |
| `loxone.can.packets_received` | — | CAN bus packets received |
| `loxone.can.receive_errors` | — | CAN bus receive errors |
| `loxone.can.frame_errors` | — | CAN bus frame errors |
| `loxone.can.overruns` | — | CAN bus buffer overruns |
| `loxone.can.parity_errors` | — | CAN bus parity errors |
| `loxone.lan.tx_packets` | — | LAN TX packets |
| `loxone.lan.tx_errors` | — | LAN TX errors |
| `loxone.lan.tx_collisions` | — | LAN TX collisions |
| `loxone.lan.tx_underruns` | — | LAN TX underruns |
| `loxone.lan.rx_packets` | — | LAN RX packets |
| `loxone.lan.rx_overflow` | — | LAN RX overflow |
| `loxone.lan.eof_errors` | — | LAN EOF errors |
| `loxone.lan.exhaustion` | — | LAN buffer exhaustion events |
| `loxone.lan.no_buffers` | — | LAN no-buffer events |
| `loxone.control.value` | — | Control state value (per control, with room/type/category attributes) |
| `loxone.weather.temperature` | Cel | Outdoor temperature |
| `loxone.weather.perceived_temperature` | Cel | Perceived temperature |
| `loxone.weather.dew_point` | Cel | Dew point |
| `loxone.weather.humidity` | % | Relative humidity |
| `loxone.weather.wind_speed` | m/s | Wind speed |
| `loxone.weather.wind_direction` | deg | Wind direction |
| `loxone.weather.pressure` | hPa | Barometric pressure |
| `loxone.weather.precipitation` | mm | Precipitation |
| `loxone.weather.solar_radiation` | W/m2 | Solar radiation |

CAN bus and LAN counters are cumulative; use `--delta` to report only increments per interval.

## Continuous export

Run as a daemon that pushes data at regular intervals:

```bash
# Push metrics + logs + traces every 30 seconds
lox otel serve --endpoint http://localhost:4318 --interval 30s

# With auth header (Dynatrace, Datadog, etc.)
lox otel serve --endpoint https://otlp.example.com:4318 \
  --header "Authorization=Bearer xxx" --interval 1m

# Filter by room or control type
lox otel serve --endpoint ... --room "Kitchen" --type LightControllerV2

# Metrics only (disable logs and traces)
lox otel serve --endpoint ... --no-logs --no-traces

# Metrics + logs only
lox otel serve --endpoint ... --no-traces
```

| Flag | Description |
|:-----|:------------|
| `--endpoint` | OTLP endpoint URL |
| `-i`, `--interval` | Push interval (e.g., `30s`, `1m`, `5m`) |
| `-t`, `--type` | Filter by control type |
| `-r`, `--room` | Filter by room |
| `--header` | HTTP header for auth (`Key=Value`) |
| `--delta` | Use delta temporality for counters |
| `--no-logs` | Disable log export |
| `--no-traces` | Disable trace export |

## One-shot push

For cron jobs or periodic collection:

```bash
# Push once and exit
lox otel push --endpoint http://localhost:4318

# Metrics only
lox otel push --endpoint http://localhost:4318 --no-logs
```

## Cron example

```bash
# Push metrics every 5 minutes
*/5 * * * * /usr/local/bin/lox otel push --endpoint http://localhost:4318
```
