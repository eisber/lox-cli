# NFC Code Touch Air

Source: https://www.loxone.com/enen/kb/nfc-code-touch-air-2/

---

The Loxone NFC Code Touch Air is used for access control and features an NFC reader and illuminated keypad on a glass surface.

![nfc front variants rendering](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/nfc_front_variants_rendering.png)

Access control solution via NFC tags, available as key fobs, stickers or cards. Alternatively, a numeric code can be entered on the keypad.

Combining an NFC tag and a numeric code enables two-factor authentication. The keypad also features a freely programmable doorbell button and a tick button to confirm entries.

The device can be operated either with an external power supply or with two AAA batteries.
In battery operation, the permanent keypad illumination and the controllable status LEDs are not available. These are only activated for a short time when the device is operated.

[**Datasheet NFC Code Touch Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NFCCodeTouchAir_100482,100483.pdf)

You can find the previous NFC Code Touch Air model [here](https://www.loxone.com/help/nfc-code-touch-air).

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Configuration](#baseconf)
- [Operation](#operation)
- [LED states](#led_codes)
- [Battery Replacement](#battery_change)
- [Change Sensitivity](#sensitivity)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Fasten the included plastic mounting frame to a flat surface or to a 68mm installation box.

A junction box is highly recommended when using an external power supply, as it makes it much easier to tuck away connecting wires or a spare loop behind the device.
Optionally, a small power supply unit can be installed in the box.

The NFC Code Touch can also be mounted using the single or double aluminium frame, available separately. We recommend the NFC Code Touch Tree in combination with the Intercom.

![NfcAir2 back mounting wiring](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/NfcAir2_back_mounting-wiring.png)

Connect the power supply (orange/white plug-in terminal).
For battery operation, remove the orange/white plug-in terminal and insert two AAA batteries.

A self-adhesive cover is included in the delivery to protect against water. It must be attached above the terminals once connected.
Two markings at the top edge of the housing help with positioning:

![nfc backflapair](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/nfc_backflapair.png)

The NFC Code Touch is first inserted into the top of the frame and then snapped into place at the bottom. To release the device, pull firmly on the bottom to disengage the latch.

A suction cup can be used to detach the device from the aluminium frame.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the right status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for five minutes, pairing mode will be activated for a limited time.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Configuration

The device is programmed and set up using the [Authentication NFC Code Touch](https://www.loxone.com/help/AuthenticationNFC) function block in Loxone Config.
Drag the NFC Code Touch from the periphery tree to the programming page to create the function block.

Only then can the device evaluate access codes and NFC tags and display them correctly via the LED status.

---

## Operation

In order to gain access to a building, a paired NFC tag is held up to the NFC reader on the device. The tag is read and authorization is verified.
When battery powered, the device must be activated by pressing the checkmark or another key before using an NFC tag.

In order to authenticate using an access code, enter the numeric code on the keypad, followed by the 'tick'.

If the authorisation was successful, access is granted and the status LEDs light up green. As an example, the door can then be unlocked. If access is denied, the LEDs light up red.

Additional functions can be selected by entering a prefix.
To do so, first enter the prefix number set in the configuration and then hold the NFC tag up to the reader.
When using a code, first enter the prefix, then confirm with the 'tick', followed by the access code.
After successful authorisation, the function assigned to the prefix is activated.

If an unauthorized NFC tag or numeric code is attempted several times in succession, the device is locked for 2 minutes for further operation, including all keys. This protection prevents brute force attacks.
An authorized NFC tag can unlock the device before that time.

Configuration also allows you to limit access via NFC tag or code to specific times, or set them up for one-time use.

More information can be found in the documentation of the [Authentication NFC Code Touch](https://www.loxone.com/help/AuthenticationNFC) function block.

Water on the surface of the NFC Code Touch can interfere with its operation. After the surface dries, the NFC Code Touch automatically re-calibrates and operates as usual.

---

## LED states

A row of four LEDs at the top of the device actives during operation, and also indicates whether access was granted or denied.
The following table shows the possible LED states:

| LED display | Meaning |
| --- | --- |
| 4x white, counting down | Indicates the timeout after a digit was pressed. The next digit must be entered within this time. After the time has elapsed, the entered code is checked for authorisation. |
| 4x green, fading out | Access granted or confirmation for bell button. In NFC learning mode: Tag was successfully detected |
| 4x red, fading out | Access denied, tag or code invalid, or not authorised. |
| 4x red, flashing fast, counts down slowly | Device is locked after repeated use of an unauthorized NFC tag or code, or by activation of the input (Off) of the function block. If an unauthorized NFC tag or incorrect code has been used 15 times in a row, the device is locked for 2 minutes. By using a valid tag, this lock can be ended early. Manual lock: Device is locked as long as the input (Off) of the block is active. |
| 4x cyan, running light from left to right | NFC learning mode is active. |
| 4x yellow, flashing | Error while reading NFC tag in learning mode. Try again, hold tag steady! The NFC monitor provides information on the cause of the error. |
| Individual | The 4 status LEDs can be controlled together via configuration and display the primary colours. They are only displayed if no other status is currently active. |
| Only right LED active | Indicates the LED status of this Air device. |

---

## Battery Replacement

To replace the batteries, remove the NFC Code Touch Air from the mounting frame. Remove both AAA batteries from the compartment on the back and insert new ones, observing correct polarity.
The device then restarts and the right status LED flashes green three times, indicating that the connection to the Miniserver has been established.
In case the LED does not blink at all or is permanently (faintly) red, the batteries are empty.

---

## Change Sensitivity

In some settings, it may be necessary to change the sensitivity of the keypad using a [device command](https://www.loxone.com/help/device-command/) or [webservice](https://www.loxone.com/help/webservice-command/) command.
If battery powered, first select keep device awake in device status, and press any button.

Each key can be addressed via a TH number:

THProx=Proximity (determines the general sensitivity for the first keypress only)
TH0=Key 1
TH1=Key 4
TH2=Key 7
TH3=Key 2
TH4=Key 5
TH5=Key 8
TH6=Key 3
TH7=Key 6
TH8=Key 9
TH9=Bell Button
TH10=Key 0
TH11=Checkmark

The default value for parameters TH0 to TH11 on the NFC Code Touch is 20.
**Lowering the value** increases the touch sensitivity. For example, halving the default value results in approximately double the sensitivity.
**Raising the value** decreases the touch sensitivity. For example, doubling the default value results in approximately half the sensitivity.
A realistic and recommended range for these values is 15 to 30, depending on the desired touch response and the installation environment.

Each button has a certain value assigned to it. First this value has to be queried. To send a Device Command to the NFC, right-click on it and select **Device Command**. If the device is working on batteries, make sure the device is awake.

![Device Command](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Device Command.png)

Example for button 3:
To retrieve the current value of this button, send the following command: **get/TH6**. The response might be 25, for example.

![get TH6](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/get_TH6.png)

![result 25](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/result_25.png)

You can now reduce the sensitivity by assigning a higher value. For example, you can send the following command: **store/TH6/26**.

![store to 26](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/store_to_26.png)

![store var to 26](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/store_var_to_26.png)

To increase the sensitivity, select a lower number.
Numbers below 10 do not need a leading zero.

Finally, the change must be accepted by restarting the device with a **Reboot** command. You will see a timeout in the monitor, confirming the reboot.

![NFC Reboot](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/NFC_Reboot.png)

![NFC timeout](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/NFC_timeout.png)

Adjust the sensitivity in small increments, as in the example. Check if the button reaction improved. If the change is not sufficient, the command can be repeated with a higher/lower number.

If adjusting the sensitivity commands does not resolve the issue, send the following command: **RevertTouchSettings** followed by the **Reboot** command.

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status NFC Code Touch Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery Low | This input activates when the battery level is