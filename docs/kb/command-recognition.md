# Command Recognition

Source: https://www.loxone.com/enen/kb/command-recognition/

---

This block can be used to extract values from any text. The search pattern is defined in the properties.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Programming example](#CommRecExample)

---

## Inputs

| Abbreviation | Summary | Description |
| --- | --- | --- |
| T | Text Input | Command text. |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Lv | Last extracted value | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Command recognition | Characters used to extract a value: \v = value, \1 = Byte interpreted as 1st byte of the output value (\2, \3, ...), \h = value interpreted as hexadecimal number Characters to navigate through the text: \. = any character, \w = any word, \# = any number, \d = digits 0-9, \m = character A-Z/a-z/0-9, \a = character A-Z/a-z, \s12 = skip 12 characters, \iText\i = jump to 'Text' Special characters: \x = Hexadecimal number (e.g. 0x09), \\ = Slash, \t = Tab (0x09), \b = Space (0x02) or Tab (0x09), \r = Return (0x0d), \n = Newline (0x0a) | - |
| Signed values | If checked, the values \1, \2, \3, etc. are used with algebraic sign in command recognition (Signed Integer). | - |

---

## Programming example

Command recognition can be used to navigate in a text or source text and to retrieve values.

Characters used to extract a value:

**\v** = numeric value

**\1** = value of byte interpreted as 1st byte of the output (**\2**, **\3**, ...)
- If the received data is interpreted as hexadecimal (e.g., "\x0A"), the decimal equivalent (10 in this example) is obtained. This method also works for multiple hexadecimal bytes (e.g., "\x0A\x0B"). When combined, these bytes form the 32-bit integer 0x0A0B, and **\2\1** returns 2571 in decimal. The same value, when extracted with **\1**, results in 10 in decimal.
- If the received data is interpreted as text, each byte represents an ASCII character. For instance, "Loxone" corresponds to the Line Feed character with an ASCII code of 76.
- The data can be used as a signed integer by considering the correct byte order and applying sign extension if necessary.

**\h** = value interpreted as hexadecimal number
- If the received data is a hex string represented as text (e.g., "0A"), it can be converted to its decimal equivalent (10 in this example).
- This method also works for multiple hex data (e.g., "0A0B"). When converted from hex to decimal, "0A0B" corresponds to 2571.

Characters to navigate through the text:

**\.** = any character

**\w** = any word

**\#** = any number

**\d** = digits 0-9

**\m** = character A-Z/a-z/0-9

**\a** = character A-Z/a-z

**\s12** = skip 12 characters
- For example, **\s12** skips 12 characters when the received data is a text string.
- If the received data is a hex string represented with escape sequences (e.g., \xFE\xFF\x00\x12\xA0\xB0\xC1\xD2\xE3\xF4\x11\x16\x17), then **\s12** will skip 12 hex bytes. In this case, it will jump behind \x16, effectively skipping a total of 48 text characters (since each hex byte is represented by four text characters).

**\iText\i** = jump behind "Text"

Special characters:

**\x** = Hexadecimal number (e.g. 0x09)

**\\** = Slash

**\t** = Tab (0x09)

**\b** = Space (0x02) or Tab (0x09)

**⧵r** = Return (0x0d)

**⧵n** = Newline (0x0a)

In the following example, we want to extract the current price of natural gas:

![commrec priceoverview](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/commrec_priceoverview.png)

Since the text "Natural Gas" is constant, we can jump to this position in the text. To extract the current price, we have to skip the time indication. There are several options to do so:

**Variant 1 via \d\a**
With \d single digits and with \a single characters can be skipped:

![commrec var1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/commrec_var1.png)

**Variant 2 via \#\w**
With \# a number and with \w any word can be skipped:

![commrec var2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/commrec_var2.png)

**Variant 3 via \s8**
Since in this example the time will always have the same number of digits and characters, they can be skipped with \s8:

![commrec var3](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/commrec_var3.png)