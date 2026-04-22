www.loxone.com
LOXONE The Miniserver based Smart Home.
24V PWM DIMMER
Part No .: 200037
+ 24V
+
- 24V DMX +
24V PWM LED DIMMER
Red Green
DMX -
Blue
Ts/Ana1
Status LED
Tz/Ana2
Program
DESCRIPTION
'All in one' LED controller for LED strips and similiar that have constant voltage control.
INSTALLATION INSTRUCTIONS
. Unit is mounted using two screws, using double-sided tape or cable ties.
· For the DMX wiring, a STOP (shielded twisted pair) should be used (eg CAT5). The cable needs to be terminated with a 120 ohm resistor.
· Under analogue operation it is important to ensure a stable control signal.
· When wiring to other devices, a total current of 24A per terminal must not be exceeded ..
· Sufficient heat dissipation of the device needs to be ensured. The amibient temperature must not exceed 40°C.
. If more than one power supply is used to supply the LEDs it is important to ensure balanced potentials between the power supplies.
SPECIFICATIONS
Rated voltage
24VDC
Permissable input voltage
12-24VDC
Output current
6.3A / 3x 2.1A
Power consumption
155 W
Consumption with 24VDC
150W / 50W per channel
Consumption with 12VDC
75W / 25W per channel
Control options
Analog (0-10V signal), push buttons, DMX
Output signal
CV - PWM, DMX-512A
Max. secondary cable length
15m
Other features
Inbuilt short circuit protection
Cable diameter
Max. 2.5mm2
Dimensions
Approx. 185×45×20mm
Housing
Plastic
Weight
100 g
Operating temperature range
5-40℃
Enclosure rating
IP20
Max. voltage drop
1V
Version
V_20_34
We reserve the right to make changes to these sepcifications :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected:
www.loxone.com
LOXONE The Miniserver based Smart Home.
24V PWM DIMMER
Part No .: 200037
OPERATING MODES
Mode 1 - Analog Dimming:
The brightness of all three channels is controlled by a single 1-10V control signal applied to terminal ANA1. Any voltage below 1V turns the device off.
For example:
Single colour LED application (3 channels are bridged)
Mode 2 - Analog Sequencer:
The overall brightness is controlled using a 1-10V signal applied to terminal ANA1, whilst the colour is picked using a 1-10V signal applied to terminal ANA2.
Colour Gradients 1 - 10 V:
Red
1 V
2.125 V
Yellow
2.125 V
3.25 V
Green
3.25 V
4.375 V
Purple
4.375 V
5.5 V
Blue
5.5 V
6.625 V
Magenta
6.625 V
7.75 V
Red
7.75 V
8.875 V
White
8.875 V
10 V
Mode 3 - Pushbutton - Single Channel Dimming:
The device can be operated as a single or two button dimmer.
Single button operation:
· The two pushbutton inputs are shorted out for a single button operation.
· A short press toggles the output between off and the previous PWM level.
. A long press increases or decreases the brightness. The direction of dimming is reversed with every long press of the press button.
Two button operation:
. A short press of the TS-button turns the PWM output off.
. A short press of the TZ-button turns the PWM output to the previous brightness setting.
. A long press of the TS-button reduces the brightness to the PWM output.
. A long press of the TZ-button increases the brightness of the PWM output.
Mode 4 - Pushbutton Sequencer:
In mode 4 the dimmer provdies 8 speed settings and 8 sequences.
. A short press of the TS-button stops the currently running sequence.
. A short press of the TZ-button starts the sequence.
. A short press of both TZ & TS together toggles the device on and off.
. A long press of the TS-button switches to the next sequence. The change is signalled by flashing the blue LEDS.
. A long press of the TZ-button switches to the next speed setting. The change is signalled by flashing the red LEDs.
Mode 5 - DMX Dimmer (single colour):
In this mode the unit uses only one DMX address. All three output channels are controlled through this one address and will have the same PWM value (brightness). The output channels are bridgeable.
Mode 6 - DMX (RGB):
In this module the unit uses 3 consecutive DMX addresses in the order R,G,B. When programming the address of the dimmer only the address of the red channel is used.
www.loxone.com
LOXONE The Miniserver based Smart Home.
24V PWM DIMMER
Part No .: 200037
Learning DMX address:
· Switch PWM Dimmer to DMX modes (5 or 6).
. Start the Loxone Config software: Click 'Learn Device', confirm the popup message with 'OK'.
. Press and hold the 'Program' button on the PWM dimmer button for at least 5 seconds to confirm. The LED will flash quickly to acknowledge the learning of the DMX address.
In operation modes 1,3 and 5 the outputs may be operated in parallel.
IMPORTANT NOTICE: Modes 1 & 2:
The voltage range of 0.5V to 1V for the input ANA1 is not defined. If the analogue voltage is in the range, then you may experience a slight flickering of the output.
Mode Selection
1. Connect the unit to the 12-24V power supply. Press and hold the 'Program' button until the status LED flashes rapidly (NOTE: When in DMX mode this may take about 30 seconds).
2. Once the 'Program' button is released the status LED blinks once, then twice and so forth up to six times, to indicate the 6 different operating modes.
3. When the Status LED flashes to indicate the desired mode, press and hold the 'Program' button again until the status LED flash quickly again.
4. Once released the unit will confirm the newly selected mode by flashing the status LED once, twice or up to six times in correspondance to the selection. This step completes the mode selection.
Display Current Mode
To display the currently selected mode, briefly press the 'Program' button. The Status LED will flash once, twice or up to six times in correspondance to the current operating mode.
Increase Output Power
If the output power rating of a single dimmer unit is not sufficient for desired application, then a second device can be synchronised. In this case the 'master device' will transmit the brightness values of 3 output channels on the DMX-512A interface. The 'slave device' is then connected to this interface and must be set to use mode 6 and DMX channels 1, 2 and 3. The 'slave device' does not even have to be the same type, but any DMX-512A compatible device can be used.
*In modes 1-4 multiple devices can be combined. All connected devices will have to be set to mode 6 as described.
Status LED
· On solid Unit is operational.
· Flashes slowly (only applicable to DMX modes) Unit is in programming mode for DMX address ..
· Flashes rapidly Unit is in mode selection mode.

---
## Extracted Tables

### Table (5x5)
:unselected: | + 24V |  | + | :unselected:
--- | --- | --- | --- | ---
:unselected: :unselected: | - 24V DMX + | 24V PWM LED DIMMER | Red Green | :unselected:
:unselected:
:unselected: | DMX - |  | Blue | :unselected:
:unselected: | Ts/Ana1 |  | Status LED | :unselected:
:unselected: | Tz/Ana2 |  | Program | :selected:

### Table (18x2)
Rated voltage | 24VDC
--- | ---
Permissable input voltage | 12-24VDC
Cable diameter | Max. 2.5mm2
Dimensions | Approx. 185×45×20mm
Housing | Plastic
Weight | 100 g
Operating temperature range | 5-40℃
Enclosure rating | IP20
Max. voltage drop | 1V
Version | V_20_34
Output current | 6.3A / 3x 2.1A
Power consumption | 155 W
Consumption with 24VDC | 150W / 50W per channel
Consumption with 12VDC | 75W / 25W per channel
Control options | Analog (0-10V signal), push buttons, DMX
Output signal | CV - PWM, DMX-512A
Max. secondary cable length | 15m
Other features | Inbuilt short circuit protection

### Table (8x3)
Red | 1 V | 2.125 V
--- | --- | ---
Yellow | 2.125 V | 3.25 V
Green | 3.25 V | 4.375 V
Purple | 4.375 V | 5.5 V
Blue | 5.5 V | 6.625 V
Magenta | 6.625 V | 7.75 V
Red | 7.75 V | 8.875 V
White | 8.875 V | 10 V
