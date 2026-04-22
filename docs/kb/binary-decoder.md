# Binary Decoder

Source: https://www.loxone.com/enen/kb/binary-decoder/

---

Binary Decoder to encode an analog value up to 32 Bits.

The lowest value Bit is at output Bit 0 (1=Bit 0, 2=Bit 1, 4=Bit 2, 8=Bit 3..., i.e.130=Bit 1+Bit 7).

The input value is a decimal unsigned value.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V | Value | Analogue input of the binary decoder | 0...4294967295 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Bit 0-31 | Bit 0-31 | 0/1 |

---