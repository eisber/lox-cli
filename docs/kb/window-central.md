# Window Central

Source: https://www.loxone.com/enen/kb/window-central/

---

Central control for multiple windows.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Co | Complete open | Stopping not possible. | - | 0/1 |
| Cc | Complete close | Stopping not possible. | - | 0/1 |
| Tg | Toggle | Toggles between open, stop, close. For single button control. | - | 0/1 |
| Pos | Position of window | Move windows to specified position. | % | 0...100 |
| Po | Partial open with push & hold | - | 0/1 |
| Pc | Partial close with push & hold | - | 0/1 |
| So | Slightly Open | Moves window to slightly open position if current position is different. | - | 0/1 |
| Wp | Weather protection | Windows are closed and locked for further operation when on. Control via user interface is still possible. | - | 0/1 |
| Off | Off | Pulse stops movement. On locks the block. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands | - |
| No | Open windows | Number of open windows | ∞ |
| Nc | Closed windows | Number of closed windows | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Selection | All selected window blocks can be controlled together. | - |

---