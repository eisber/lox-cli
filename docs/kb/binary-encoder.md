# Binary Encoder

Source: https://www.loxone.com/enen/kb/binary-encoder/

---

Binary Encoder to encode up to 32 input bits to an analog value

The lowest value Bit is at input Bit 0 (Bit 0=1, Bit 1=2, Bit 2=4, Bit 3=8..., i.e.Bit 1+Bit 7=130).

The output value is a decimal unsigned value.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Bit 0-31 | Bit 0-31 | Individual Bits to be combined. Bit 0: least significant bit (LSB) | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| V | Calculated output value | 0...4294967295 |

---