# Virtual HTTP Input

Source: https://www.loxone.com/enen/kb/virtual-http-input/

---

With the Virtual HTTP Input, values can be read from a web page. This facilitates the data retrieval from devices with a web interface.

From the [Loxone Library](https://library.loxone.com/) suitable [templates](https://www.loxone.com/help/templates) for the integration of devices can be imported.

Only the Basic Authentication Scheme is supported.

> **ℹ️ Note:** Automated extraction of data from websites (Web scraping) and any subsequent use of the extracted data should be conducted in accordance with the website's terms of service, local regulations and applicable standards. Loxone does not assume liability for any consequences arising from non-compliance.

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)
- [Command Recognition](#CommRecExample)

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| URL | URL for HTTP(S) request e.g.: http://192.168.1.7:80/request.php https://192.168.1.7:443/request.php | - | - | - |
| Polling cycle | Polling cycle in seconds (Minimum 10s) | s | 10...604800 | - |
| Timeout | Time that the Miniserver waits for a response from the device after a read or write command before the operation is considered to have failed. Increase for slow devices or poor connections. | ms | 10...8000 | - |
| Number of permitted timeouts | Number of failed queries before a system message is displayed. 0 deactivates monitoring. | - | 0...100 | - |

---

## Programming example

First, a "Virtual HTTP Input" is created under Virtual Inputs:

![httpInput Add](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/httpInput_Add.png)

The web address, a suitable polling cycle, timeout and the number of permitted timeouts are defined in the properties window.
In this example, the value of the "Temperature" input is retrieved from the Miniserver using a web service command.
If user authentication is required, the user name and password can be entered as follows: http://User:Password@IP-address

![httpInput properties](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/httpInput_properties.png)

### Virtual HTTP Input Command

To extract values from this page, a "Virtual HTTP Input Command" is required.
Any number of commands can be created to retrieve values.

![httpInput AddCommand](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/httpInput_AddCommand.png)

The search pattern is defined in the settings. Therefore, an editing window can be opened, here the source code of the website is displayed to make it easier to navigate to the desired value.

![httpInput CommRec](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/httpInput_CommRec.png)

This input can be moved to the programming page via drag & drop.

---

## Command Recognition

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