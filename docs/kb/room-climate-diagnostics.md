# Room Climate Diagnostics

Source: https://www.loxone.com/enen/kb/room-climate-diagnostics/

---

The following message can be received from the [Intelligent Room Controller V2](https://www.loxone.com/enen/kb/irc-v2/) via [System Status](https://www.loxone.com/enen/kb/systemstatus/) or output at the module’s error output:
- [Heat Protection Temperature Exceeded](#frost-heat)
[Frost Protection Temperature Undershot](#frost-heat)
[Large Difference between Actual and Target Temperatures](#big-difference)

## Frost and Overheat Protection

Frost and Overheat Protection temperatures server to protect the building. If the overheat protection temperature has been exceeded or the frost protection temperature has been fallen below, immediate measures should be taken to correct the temperature again.
The temperature limits for Frost and Overheat protection should be selected so that a warning is only given if damage is possible. Such damages are, for example, the freezing of pipes in cold conditions or the swelling of wooden floors in excessively hot conditions. The corresponding frost and overheat temperatures can be set in Loxone Config in the parameters of the [Intelligent Room Controller (V2)](https://www.loxone.com/enen/kb/irc-v2/) or can be set in the Loxone Smart Home App in [Expert Settings](https://www.loxone.com/enen/kb/expert-mode/).
Possible causes can be found [here](#causes).

## Large Difference between Actual and Target Temperatures

The note ‘Large Difference between Actual (room) Temperature and Target Temperature’ is output if the Room Temperature is at least 1.5ºC below the Target Temperature during Heating Operation or at least 1.5ºC above the Target Temperature during Cooling Operation.
The messages serves as information as an indicator that there may be something wrong with the Heating or Cooling system.
Possible causes can be found [here](#causes).

## Possible Causes

In the following you will find further information on possible causes in the event of a message from an [Intelligent Room Controller (V2)](https://www.loxone.com/enen/kb/irc-v2/).
- [Temperature is only reached in certain rooms](#only-some-rooms)
- [Temperature is not consistent across the whole system](#whole-system)
- [Function blocks with influence on the Heating or Cooling system.](#other-controls)

### Temperature is only reached in certain rooms

In the event that your Heating or Cooling system is running but the temperatures in individual rooms are not reached check the following points:
- Are the displayed room temperatures realistic?

For Example, temperature sensors in the outer walls can indicate too low values during the winter months.
- Is the target temperature set correctly? This must be lower than the lower than the flow temperature.
Do the actuators function and are they accessible?
- If a message is displayed in [System Status](https://www.loxone.com/enen/kb/systemstatus/) by one of the Actuators, you will find more detailed information [here](https://www.loxone.com/enen/kb/valve-diagnostics/). In the case that the device is inaccessible, check the points on [this page](https://www.loxone.com/enen/kb/device-offline-diagnostics/).

### Temperature is not correct across whole system

Depending on which parts of your Heating or Cooling system you have access to, certain messages can be checked or corrected.
- Is the Heating or Cooling system running or does it report an error?
Is the flow temperature okay?
Are the heating pumps active?

### Function blocks that influence the Heating or Cooling system

The following components can (In addition to Intelligent Room Controllers) influence the behaviour of your Heating or Cooling system:
- [Climate Controller](#climate-controller)
- [Intelligent Temperature Controller/Heat Mixer](#irc-v2)

#### Climate Controller
- Is the correct mode active?

Automatic: Heating and Cooling requirements are compared, the greatest requirement determines the mode.
- Heating Only or Cooling Only: Rooms with the opposite requirement to the one set are not served.
- Standby: The function block stays in standby mode as long as the set minimum opening of the valves is not maintained. This mode cannot be set manually.
- Is the Heating or Cooling requirement (Output H or C on the device) correctly transmitted to the Energy Source?
You will find more detailed information about the function block [here](https://www.loxone.com/enen/kb/climate-controller/).

#### Intelligent Temperature Controller/Heating Mixer
- Is the output for the pump or mixer active?

If the input is used for the current buffer temperature, the output is only active when this is reached.
- Has the target flow temperature of the Temperature Controller at the Heating Mixer been reached?
- Is the mixer open?
- Does the mixer module receive the correct Target Temperature?
- Here you can find more detailed information on [Intelligent Temperature Controllers](https://www.loxone.com/enen/kb/intelligent-temperature-controller/) and [Heating Mixers](https://www.loxone.com/enen/kb/mixing-valve-controller/).