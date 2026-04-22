# Touch Pure Flex Tree

Source: https://www.loxone.com/enen/kb/touch-pure-flex-tree/

---

The Touch Pure Flex is your 100% personalized control device with up to 12 freely definable and positionable touch points.
With the integrated LED matrix display, as well as 3 status LEDs (which you can find on the left), menus, values, texts and states can be displayed.
The integrated lighting makes the Touch Pure Flex ideal for operation in low-light conditions or complete darkness.

![tflx variants rendering front](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_variants_rendering_front.png)

[**Datasheet Touch Pure Flex IP44 Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureFlexIP44Tree_100608,100609.pdf)

[**Datasheet Touch Pure Flex Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureFlexTree_100673,100674.pdf)

[**Datasheet Touch Pure Flex CO2 Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchFlexCO2Tree_100612,100613.pdf)

[**Datasheet Touch Pure Flex Tree Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureFlexTreeGen1_100507,100508.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming Example](#TflxBasicConf)
- [Buttons Not Responsive](#ButtonsUnresponsive)
- [Calibrate CO2 Sensor](#calco2)
- [Inputs, Outputs, Properties](#Sensor)
- [Documents](#Documents)

---

## Mounting

Fasten the included plastic mounting frame to a flat surface or to a installation box. Alternatively, the aluminum frame can be used, available separately.

A junction box is not required, but highly recommended, as it makes it much easier to tuck away connecting wires or a spare loop behind the device.

![tflxtree conndia](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflxtree_conndia.png)

Connect the power supply (orange/white plug-in terminal) and the tree data lines (green/white plug-in terminal).

A self-adhesive cover is included in the delivery to protect against water. It must be attached above the terminals once connected.
Two markings at the top edge of the housing help with positioning:

![tflxtree foil](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflxtree_foil.png)

The Touch Pure Flex is first inserted into the top of the frame and then snapped into place at the bottom. To release the device, pull firmly on the bottom to disengage the latch.

A suction cup can be used to detach the device from the aluminum frame.

---

## Commissioning

Switch on the power supply, then the display shows that the device is ready for pairing, if the wiring is correct (connection to Tree Interface/Miniserver is established).

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Programming Example

Via "View/Configure button assignment" a window opens, here the respective Config ID (from the label of the Touch Pure Flex) is entered.

![tflx edit](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_edit.png)

![tflx configID](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_configID.png)

After that, the Touch Pure Flex configuration is downloaded from Loxone and the button icons are visible in the dialog.
If commands were defined in the Online Configurator, these are now also assigned to the device.
Alternatively, the individual button inputs are available after downloading the configuration.

To change or create new commands, click on the respective button and open the API Command Generator in the corresponding line via the cogwheel:

![tflx settings](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_settings.png)

In the following example, we select a lighting controller and the input for selecting the mood. The "MENU" command then enables a menu selection of the moods on the Touch Pure Flex via the display and the arrow keys.

![tflx APICommandGenerator](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_APICommandGenerator.png)

![tflx settings command](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_settings_command.png)

The Touch Pure Flex can now be dropped on the desired Lighting Controller; the API Connector is created automatically. A separate API Connector is required for each function block:

![tflx API](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_API.png)

In the Simulation/Liveview a representation of the Touch Pure Flex can be opened.
This way the created commands can be tested:

![tflx testing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tflx_testing.png)

Commands for numerous blocks can be created according to the same principle. The following document contains a list and explanation of the possible commands:

[**API Commands**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/API_Commands.pdf)

---

## Buttons Not Responsive

If the buttons on the Touch Pure Flex are unresponsive or not functioning correctly, execute the [device command](https://www.loxone.com/help/device-command/) **TouchForceUpdate**. After sending the command, wait a few seconds and check if the buttons are functioning properly.

![TouchForceUpdate](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TouchForceUpdate.png)

---

## Calibrate CO2 Sensor

The CO2 sensor automatically calibrates itself weekly if it is in continuous operation for this period.

Manual calibration of the CO2 sensor is not necessary, but can be performed for special applications.
To do this, first ensure that the CO2 value at the measuring point is as low and constant as possible.
Now measure the CO2 value at the measuring point with a suitable measuring device.
Immediately afterwards, assign the measured value to the sensor via a [device command](https://www.loxone.com/help/device-command/) or [webservice](https://www.loxone.com/enen/kb/web-services) command, in the following example 450ppm:

miniserver/dev/sys/wsdevice/devicename-or-serialnr/ForceRecalibration/450

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Temperature | Measured value of the temperature sensor. For plausible values even with the display always on, its brightness must not be set higher than 70%. | ° | -40...125 |
| Humidity | Measured value of the humidity sensor. For plausible values even with the display always on, its brightness must not be set higher than 70%. | % | 0...100 |
| Button 1-x | Button can be used as digital input | - | - |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| LED 1 | Output to control the status LED. | - | 0/1 |
| LED 2 | Output to control the status LED. | - | 0/1 |
| LED 3 | Output to control the status LED. | - | 0/1 |
| Display always on | As long as the output is active the display will stay on, even if the device is idle. Can be overwritten by the output Disable display when idle (DisD). | - | 0/1 |
| Disable display when idle | As long as the output is active the display will stay off when the device is idle. Overrides the output Display always on (Don) if used. | - | 0/1 |
| Custom display text | Output can be used to show custom information on the display when the device is idle. Output needs to be checked in the setting “Idle display output” in order to work. Allowed characters: A B C D E F G H I J K L M N O P Q R S T U V W X Y Z 0 1 2 3 4 5 6 7 8 9 ? ! . , - + / * ( ) @  : % = ° ' A maximum of 100 characters can be used. Non-Latin writing systems are transcribed phonetically. Every change sent will reset the scrolling of the set text. | - | - |
| Display & LED Brightness | Defines the brightness of the display and the 3 status LEDs. | % | 0...100 |
| API Connector | Intelligent API based connector. API Commands | - | - |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Touch Pure Flex Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - | - |
| View / Configure button assignment | Click here to view and configure the button assignment. | - | - | - |
| Config ID | Config ID of the Touch Pure Flex | - | - | - |
| Brightness | Specifies the brightness of the display and the 3 status LEDs on the front. (0 = Display OFF) When used with the Touch Pure Flex Controller, the display and backlight brightness are controlled by the function block. | % | 0...100 | 30 |
| Display duration after last input | Display will remain active for this duration after the last input and then switch to idle. | s | 0...900 | 15 |
| Idle display output | Specifies the information that is shown on the display when the device is idle. Temperature and humidity values uncorrected, as measured by Touch Pure Flex. The text output Custom display text (CDT) can be used to display custom information. | - | - | - |
| Audible acknowledgement | Audible acknowledgement on button press | - | - | - |
| Button Behaviour | Specifies the behavior when a button is pressed. Pulse: Sends a pulse on rising edge OnOff : Sends ON on rising edge and OFF on falling edge, used for long click | - | - | - |
| Transmission cycle | Specifies how often analog values from sensors are transmitted [s] 0 = Values are updated at least every 60 min, or if temperature deviates by 0.2 °C, humidity deviates by 2% or CO2 deviates by 50 ppm to the last transmitted value | - | 0...3600 | 900 |

---

## Documents

[**Datasheet Touch Pure Flex IP44 Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureFlexIP44Tree_100608,100609.pdf)

[**Datasheet Touch Pure Flex Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureFlexTree_100673,100674.pdf)

[**Datasheet Touch Pure Flex CO2 Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchFlexCO2Tree_100612,100613.pdf)

[**Datasheet Touch Pure Flex Tree Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureFlexTreeGen1_100507,100508.pdf)

---