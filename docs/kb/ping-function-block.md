# Ping

Source: https://www.loxone.com/enen/kb/ping-function-block/

---

Ping

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| Online | Output is active as long as the target device is ping-able | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Pi | Ping interval | Duration between successful pings | s | 0...∞ | 10 |
| Td | Timeout duration | Timeout duration between unsuccessful pings | s | 0...∞ | 30 |
| N | Number of unsuccessful pings | Number of unsuccessful pings before the output is switched | - | 0...∞ | 5 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Ping address | Address to check whether there is an internet connection. E.g. 8.8.8.8 | - | - | - |
| Delay after start | Delay of the first ping in seconds following the restart of the Miniserver. | s | 0...3600 | - |

---