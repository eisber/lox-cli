# Sequencer

Source: https://www.loxone.com/enen/kb/sequencer/

---

Trigger switches the next output on.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Trigger | Switches the next output on. | 0/1 |
| P | Position | Chooses a specific output | 0...8 |
| R | Reset | When 1 (O) = (Dv) | 0/1 |
| DisPc | Disable | Disables inputs (Tr) and (P) when On. (Child lock) | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| O1-8 | Output 1-8 | 0/1 |
| Sel | Selected output | 0...8 |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |
| Max | Maximum number of used outputs | 0...8 | 8 |
| Dv | Default value | Output that should be set when (R) is triggered (0 = all OFF) | 0...8 | 1 |

---