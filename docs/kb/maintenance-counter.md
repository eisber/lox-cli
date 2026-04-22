# Runtime Counter

Source: https://www.loxone.com/enen/kb/maintenance-counter/

---

The Runtime Counter enables the implementation of a total operating time measurement and maintenance intervals. Examples are maintenance intervals of filters, runtime detection of motors etc...
As long as Input (En) is On, the time is measured. The total operating time is displayed at the output (To). The interval of the statistics is fixed to 30 minutes.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| En | Enable | Counters running while on. | 0/1 |
| Rmc | Reset maintenance counter | Resets maintenance counter to parameter Mi. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Me | Maintenance interval exceeded | 0/1 |
| To | Total operating time | ∞ |
| Lst | Last start time | ∞ |
| Rtm | Remaining time maintenance | Time remaining until maintenance interval is reached. | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Tu | Time unit | 0=seconds 1=minutes 2=hours 3=days Applies only to the analog outputs, not to the user interface! | 0...3 | 0 |
| Mi | Maintenance interval | Block: Value specified in seconds Property window: 1d12:00:00.000 (Days, Hours, Minutes, Seconds, Ms) 0 = Output (Me) is not used. | ∞ | 0 |

---