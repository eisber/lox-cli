# KNX/EIB Data Types

Source: https://www.loxone.com/enen/kb/knxeib-data-types/

---

## KNX/EIB Data Types

| KNX/EIB Function | Length | EIS | DPT | Sensor | extended Sensor | Actuator | extended Actuator |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Switch | 1 Bit | EIS 1 | DPT 1 | ✓ | Off: x/x/x:0 On: x/x/x:1 | ✓ | Off: x/x/x:0 On: x/x/x:1 |
| Dimming (Position, Control, Value) | 1 Bit, 4 Bit, 8 Bit | EIS 2 | DPT 3 |  |  | ✓ |  |
| Time | 3 Byte | EIS 3 | DPT 10 | ✓ |  | ✓ | x/x/x:t |
| Date | 3 Byte | EIS 4 | DPT 11 | ✓ |  | ✓ | x/x/x:t |
| Floating point | 2 Byte | EIS 5 | DPT 9 | ✓ |  | ✓ |  |
| Relative value | 1 Byte | EIS 6 | DPT 5, DPT 6 | ✓ | Min: x/x/x:0% On: x/x/x:255% | ✓ | Min: x/x/x:0% On: x/x/x:255% |
| Blinds / Roller shutter | 1 Bit | EIS 7 | DPT 1 |  |  | ✓ |  |
| Priority | 1 Bit | EIS 8 | DPT 2 |  |  |  |  |
| IEEE Floating point | 4 Byte | EIS 9 | DPT 14 | ✓ |  | ✓ |  |
| 16-bit counter values | 2 Byte | EIS 10 | DPT 7, DPT 8 | ✓ | Min: x/x/x:0# Max: x/x/x:65535# | ✓ | Min: x/x/x:0# Max: x/x/x:65535# |
| 32-bit counter values | 4 Byte | EIS 11 | DPT 12, DPT 13 | ✓ |  | ✓ |  |
| Access control | 1 Byte | EIS 12 | DPT 15 |  | Min: x/x/x:0% Max: x/x/x:255% |  | Min: x/x/x:0% Max: x/x/x:255% |
| ASCII character | 1 Byte | EIS 13 | DPT 4 |  | Min: x/x/x:0% Max: x/x/x:255% |  | Min: x/x/x:0% Max: x/x/x:255% |
| 8-bit counter values | 1 Byte | EIS 14 | DPT 5, DPT 6 | ✓ | Min: x/x/x:0! Max: x/x/x:255! | ✓ | Min: x/x/x:0! Max: x/x/x:255! |
| Text | 14 Byte | EIS 15 | DPT 16 | ✓ |  | ✓ |  |