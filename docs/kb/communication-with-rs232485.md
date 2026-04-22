# Communication with RS232/485

Source: https://www.loxone.com/enen/kb/communication-with-rs232485/

---

## CONTENTS
- Settings
- Monitor
- Sensor
- Actuator

## SETTINGS

The following settings for the RS-232 and RS-485 extensions can be modified in the Properties window.

| Baud rate | Data transmission speed in bits per second |
| --- | --- |
| Data bits | 8 |
| Stop bits | 1-2 |
| Parity | None, Even, Odd, Always 0, Always 1 |
| End-of-frame symbol | The use of an end-of-frame symbol is optional. The symbol must be a hexadecimal character (i.e., 0x0A). When the RS-232/RS-485 extension detects the end-of-frame symbol, it recognises that the received frame is complete and forwards the frame to the Miniserver. If no end-of frame symbol is specified, a 32 bit period timeout is used to signal end-of-frame. This means that if nothing is received for a period of 32 bits, then the RS-232/RS-485 extension assumes a complete frame has been received and forwards the frame to the Miniserver. |
| Checksum | The use of check summing is optional. The following checksums can be used: XOR byte, byte-sum, CRC byte, Modbus CRC Checksum Fronius. When transmitting over an RS-232/RS-485 relay, the appropriate checksum is calculated and inserted into the data stream. When receiving with an RS-232/RS-485 sensor, command recognition occurs only when the corresponding checksum has been correctly received. The checksum may not be used for command recognition. |

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
All RS-232 and RS-485 communication will be transported via the RS-485 bus to the Loxone Miniserver. Frequent queries of data (more than several times per second) can lead to an overload of information where command execution could be delayed. Please therefore check with the device information whether data must be queried at a high frequency or whether several queries per second are sensible.

## MONITOR

The RS-232/RS-485 Monitor can be activated by ticking ‘RS-232/485 Monitor’ in the Miniserver tab.

For RS-232/RS-485 extensions each received byte is, by default, displayed on the monitor in ASCII format. By checking ‘Hex’, the hexadecimal value of each received byte will be displayed instead.

## SENSOR

Whether an RS-232 or RS-485 sensor takes a digital or analogue input can be defined in the Properties window for that sensor by ticking ‘Use as digital input’, or leaving un-ticked as appropriate.

In the Properties window under ‘command recognition’, a character string can be entered. The digital input places a pulse at the output if the received data contains the command recognition string. For the analogue input, any value can be taken from the received data.

The string is interpreted as ASCII characters.

A sensor can receive up to 512 characters.

If checksum is used, command recognition works only if the corresponding checksum has been received correctly. Checksum may not be used for command recognition.

#### SPECIAL CHARACTERS FOR COMMAND RECOGNITION WHEN USING DIGITAL AND ANALOGUE INPUT

> **ℹ️ Note:** Hexadecimal i.e., \x09 for 0x09 or \x01\x02\x03\x04 for 0x01020304

#### SPECIAL CHARACTERS FOR COMMAND RECOGNITION WHEN USING A ANALOGUE INPUT

> **ℹ️ Note:** The value is accepted as an ASCII string. Decimal points must be separated by a comma or a dot.

#### EXAMPLES WITH DIGITAL INPUT

| Received data stream | Command recognition | Digital output |
| --- | --- | --- |
| This is a test | This is a test | Pulse |
| This is a test | This is a test | o |
| CMD01 OK\n\r | CMD\d\d OK\n\r | Pulse |

#### EXAMPLES WITH ANALOGUE INPUT

| Received data stream | Command Recognition | Analogue output |
| --- | --- | --- |
| 1254 | \v | 1250 |
| 1.254 | \v | 1.254 |
| 1,254 | \v | 1,254 |
| pm 18.5 20 19.25 | pm \v | 18.5 |
| pm 18.5 20 19.25 | pm \# \v | 20 |
| pm 18.5 20 19.25 | pm \# \# \v | 19.25 |
| CMD01 \ Xa5 | CMD01 \ 1 | 0xA5 |
| CMD02 \x01\x02\x03\x04 | CMD02 \1\2\3\4 | 0x04030201 |
| CMD02 \x01\x02\x03\x04 | CMD02 \4\3\2\1 | 0x01020304 |

## ACTUATOR

Whether an RS-232 or RS-485 relay takes a digital or analogue input can be defined in the Properties window for that relay by ticking ‘Use as digital input’, or leaving un-ticked as appropriate.

In the Properties window under ‘Command when ON’ or ‘Command when OFF’ , a character string can be entered. Upon activation/deactivation the digital output sends the appropriate string to the RS-232 interface. For analogue output, the value at the relay input can be forwarded (for any input variation) and placed at the RS-232 output.

The string is interpreted as ASCII characters.

A sensor can receive up to 256 characters.

If checksum is used, the appropriate checksum is calculated and inserted into the data stream to be sent.

#### SPECIAL CHARACTERS FOR DIGITAL AND ANALOGUE OUTPUT

> **ℹ️ Note:** Hexadecimal as \ x09 for 0x09 or \ x01 \ x02 \ x03 \ x04 for 0x01020304

#### SPECIAL CHARACTERS FOR THE ANALOGUE OUTPUT

> **ℹ️ Note:** The value found at the relay input is sent from the RS232 interface (without a decimal point).

#### EXAMPLES OF DIGITAL OUTPUT

| Input | Command when ON | Command when OFF | Transmitted data stream |
| --- | --- | --- | --- |
| Rising edge | CMD ON\n\r | CMD OFF\n\r | CMD ON\n\r |
| Falling edge | CMD ON\n\r | CMD OFF\n\r | CMD OFF\n\r |

#### EXAMPLES OF ANALOGUE OUTPUT

| Input | Command when ON | Transmitted Data Stream |
| --- | --- | --- |
| 36 | CMD03  | CMD03 36 |
| 36.1 | CMD03  | CMD03 36.1 |
| 36.123 | CMD03  | CMD03 36.123 |
| 59 | Time:  | Time: 0:00:59 |
| 100 | Time:  | Time: 0:01:40 |
| 3600 | Time:  | Time: 1:00:00 |
| 36000 | Time:  | Time: 10:00:00 |
| 86400 | Time:  | Time: 1 day, 00:00:00 |
| 400000 | Time:  | Time: 4 days, 15:06:40 |