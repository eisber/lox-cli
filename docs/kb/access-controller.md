# Access Controller

Source: https://www.loxone.com/enen/kb/access-controller/

---

In the properties of the access control block the 1-wire Extension that has got the iButton reader connected must be selected. Select which user groups have got access permissions and also set the times for each group. (Periphery tree > user groups)
(Sel) input to identify the iButton reader if multiple are connected to the same 1-Wire Extension. Following a pulse on (Sel) the permissions for this block are used to verify the identity of the iButton for the duration of (Dsel)

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Sel | Select access controller | Pulse to select access controller (when using multiple access controllers) | 0/1 |
| Eid | External Authentication ID | This External Authentication ID can be provided e.g. by a fingerprint through a virtual input. This ID must be present in the user authentication. | - |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| P | Permission given | Activates output for the duration set in parameter (Pd) when permission is given. | 0/1 |
| Txt | Providing the last authorisation details The text is available as long as the output (P) is on. | - |
| Pd | Permission denied | Activates output for the duration set in parameter (Pd) when permission is denied. | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Dsel | Duration Access Controller selected | If (Dsel) = 0 the authentication check is carried out immediately, (Sel) is not used. | ∞ | 0 |
| Pd | Pulse duration | Pulse duration of outputs (P), (Pd). | ∞ | 3 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Assigned Device | The following devices are supported: Loxone Intercom 1-wire extension Devices with 1-wire interface | - |

---