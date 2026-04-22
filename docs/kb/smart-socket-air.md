# Smart Socket Air

Source: https://www.loxone.com/enen/kb/smart-socket-air/

---

The Smart Socket Air is a switchable socket with measurement of temperature, power and energy, based on Loxone Air technology.

[**Datasheet Smart Socket Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_SmartSocketAir_100115.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Status LED](#led_states)
- [Country variants / Connector types](#plug_types)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

In delivery state, pairing mode will be active after plugging the device into a socket. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds immediately after plugging the device in. The pairing button is located on the front of the device:

![100115 learnbutton](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100115 learnbutton.png)

---

## Status LED

A detailed listing of the different LED states can be found [here](https://www.loxone.com/enen/kb/led-states/).

The LED can also be used in programming applications. For this you will need to find a suitable actuator in the periphery tree. Via this analogue output, the LED can be controlled as follows:

---

## Country variants / Connector types

### Typ F – Item 100115

Germany, Austria, Afghanistan, Algeria, Andorra, Bosnia-Herzegovina, Bulgaria, Croatia, Estonia, Finland, Greece, Hungary, Indonesia, Iceland, Italy/San Marino/Vatican City State, Korea, Latvia, Lithuania, Luxembourg, Macedonia, Montenegro, Moldova, Netherlands, Norway, Portugal, Romania, Russia, Sweden, Serbia, Slovenia, Spain, Syria, Turkey, Ukraine

### Typ J – Item 100119

Switzerland, Liechtenstein, Maldives, Madagascar, Rwanda, Ethiopia, El Salvator, Jordan

### Typ E – Item 100120

France, Belgium, Monaco, Czech Republic, Slovakia, Tunisia, many former French colonies

### Typ G – Item 100121

Great Britain, Bahrain, Bangladesh, Belize, Bhutan, Botswana, Brunei, Cambodia, Channel Islands, Cyprus, Dominica, El Salvator, Falkland Islands, Gambia, Guatemala, Guyana, Hong Kong, Ireland, Isle of Man, Kenya, Kuwait, Lebanon, Macau, Malawi, Malaysia, Maldicia, Malta, Mauritius, Myanmar, Nigeria, Oman, Seychelles, Tanzania, Singapore

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Temperature | Provides the measured value of the temperature sensor: Every 5 min. | ° | -40...125 |
| Power | Provides the value of the current power: Above 5% and min. 2W change from the last value, at most every 5 sec. Otherwise every 5 min and when the load is switched off. | kW | 0...4 |
| Energy | Provides the consumed energy since the last value: Every 5 min and when the load is switched off. | kWh | 0...∞ |
| Input 1 | Digital input to use the button (pairing button) | - | 0/1 |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Relay | Output to switch the connected load | - | 0/1 |
| Status LED | Analogue output for controlling the status LED. 0 = Off, 1 = Green, 2 = Orange, 3 = Red | - | 0...3 |
| API Connector | Text | - |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Smart Socket Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Application | Specifies the intended use of the actuators. Universal: Outputs can be used freely. Load: Switch any load. Auto-Configuration will create a Switch. Lighting: Control lighting. Will be connected to Lighting Controller by Auto-Configuration. | - |

---

## Safety Instructions

> **ℹ️ Note:** Warning! Before using the product, please follow the safety instructions below. These serve to prevent hazards and personal injury and property damage.
- If the case or the power contacts are damaged, do not operate the Smart Socket Air.
- The Smart Socket Air may only be operated in closed, dry rooms.
- To prevent the risk of fire or electric shock, protect the Smart Socket Air from moisture, rain, and heat.
- The maximum load capacity of the Smart Socket is 13A. When connecting devices, make sure that this value is not exceeded.
- Devices without a cable connection that perform mechanical movements or vibrations must not be connected directly to the Smart Socket Air.
- Cleaning and maintenance work may only be carried out with the mains plug disconnected.
- The Smart Socket Air must not be opened.
- Keep the Smart Socket Air away from children.
- This device may be used by children 8 years old and over, and by persons with reduced physical, sensory, or mental capabilities, or lack of experience and knowledge, if they have been supervised or instructed in the safe use of the device and understand the hazards arising therefrom. Children are not allowed to play with the device. Cleaning and maintenance must not be carried out by children.
- Failure to follow the safety instructions may result in damage to the Smart Socket Air and personal injury.
- Use the Smart Socket Air only as directed.
- The Smart Socket Air must not be inserted one behind the other.
- The Smart Socket Air needs to be positioned near the device and easily accessible.

---

## Documents

[**Datasheet Smart Socket Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_SmartSocketAir_100115.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---