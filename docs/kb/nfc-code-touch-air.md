# NFC Code Touch Air Gen. 1

Source: https://www.loxone.com/enen/kb/nfc-code-touch-air/

---

The NFC Code Touch provides access control via encrypted or unencrypted NFC tags or a key code entered via the touch keypad.

Supports permanent, one-time or temporary codes. With multi-purpose / doorbell button

Backlight and NFC functionality are not available when battery powered. Also, the keypad must be activated before entering a code by touching the checkmark.

[**Datasheet NFC Code Touch Air Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NFCCodeTouchAirGen1_100299,100305.pdf)

You can find the newer NFC Code Touch Air model [here](https://www.loxone.com/help/nfc-codetouch-air).

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Blinking codes](#led_codes)
- [Variants](#variations)
- [Battery Replacement](#battery_change)
- [Change Sensitivity](#sensitivity)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The NFC Code Touch is mounted directly to the wall or a suitable back box using the included mounting frame.

The mounting frame must lie flat on the wall in order to ensure secure installation. The NFC Code Touch is placed on the top of the frame and snaps into place at the bottom. To remove the device, firmly pull on the lower edge until it releases.

Water on the surface of the NFC Code Touch can interfere with its operation. After the surface dries, the NFC Code Touch automatically re-calibrates and operates as usual.

![100299 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100299 install.png)

---

## Commissioning

Connect power supply or insert battery. Note that, for energy saving reasons, NFC authentication does not work when battery powered.

In delivery state, pairing mode is active. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply. The pairing button is located on the back of the device.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Blinking codes

A detailed listing of the different LED states can be found at [here](https://www.loxone.com/enen/kb/led-states/).

| Blinking code | Meaning |
| --- | --- |
| WHITE (LEDs indicate countdown) | Starts when a digit is pressed and displays a timeout indicating how long the digit is valid when entering the code |
| GREEN fading out | Indicates "Access granted" after correct code was entered & confirms universal button press (bell symbol) |
| RED fading out | Indicates "Access denied" when incorrect code was entered |
| RED rapid flashing | Device is locked after repeated use of an unauthorized NFC tag or code, or by activation of the input (Off) of the function block. If an unauthorized NFC tag or incorrect code has been used 15 times in a row, the device is locked for 2 minutes. By using a valid tag, this lock can be ended early. Manual lock: Device is locked as long as the input (Off) of the block is active. |
| CYAN fading in/out | Indicates the learning mode for NFC tags |
| Individual | User can control all 4 LEDs (simultaneously) in individual colours via Loxone Config. |

---

## Variants

| Description | NFC Code Touch Air battery powered | NFC Code Touch Air 24V DC |
| --- | --- | --- |
| LED backlight | No | Yes |
| Manually configurable Status LEDs | No | Yes |
| NFC | No | Yes |
| Activation required | Yes | No |

---

## Battery Replacement

To replace the battery, remove the NFC Code Touch Air from the wall. The CR2450 lithium battery is located in the back. Remove the battery and insert a new one. The device will restart and the status LED will flash green 3 times. In case the LED does not blink at all or is permanently (faintly) red, the batteries are empty.

> **ℹ️ Note:** Please only use the lithium battery provided by Loxone, as these will ensure an exact fit!

---

## Change Sensitivity

In some settings, it may be necessary to change the sensitivity of the keypad using a [device command](https://www.loxone.com/help/device-command/) or [webservice](https://www.loxone.com/help/webservice-command/) command.
If battery powered, first select keep device awake in device status, and press any button.

(If battery powered, wake up device first and press any button)
<ip-miniserver>/dev/sys/wsdevice/<name-code-touch>/store/TouchTh/0x0B
<ip-miniserver>/dev/sys/wsdevice/<name-code-touch>/Reboot

If the sensitivity is to be reset to factory default, use **0x0A** instead of 0x0B in the above command.

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status NFC Code Touch Air Gen. 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery Low | This input activates when the battery level is