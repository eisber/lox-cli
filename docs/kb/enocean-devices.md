# EnOcean devices

Source: https://www.loxone.com/enen/kb/enocean-devices/

---

## CONTENTS

[Adding new sensors](#1)

[Adding new actuators](#2)

[Creating New EnOcean devices](#3)

[Compatible Devices](#4)

## ADDING NEW SENSORS

#### ENABLE MONITOR

Select the Miniserver tab and check the ‘EnOcean Monitor’ box:

![en_kb_config_enocean_start_monitor](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Start_Monitor.png)

#### CONFIGURE THE SENSOR

The EnOcean monitor has two modes, ‘Monitor’ and ‘Learn’. In the monitor, click ‘Monitor’, this will display each individual signal:

![EnOcean Monitor Screen](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Monitor.png)

Then click on ‘Learn’ in the monitor and you will see the signals from the same sensor grouped together with a count next to them. Highlight the sensor, enter a ‘Name’ and select a ‘Type’, then click ‘Create sensor’:

![EnOcean Learn Screen](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Learn.png)

The sensor can now be found in Periphery tree:

![EnOcean Periphery Tree](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Periphery.png)

#### CONFIGURING A WINDOW CONTACT

Identify the sensor by magnetically initiating a change in its status.

Highlight the sensor as before, enter a ‘Name’ and its ‘Type’, then click ‘Create sensor’.

![EnOcean Learn Contact](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Learn_Contact.png)

#### CONFIGURING A MOTION DETECTOR

Identify the motion detector by repeatedly pressing its configure button.

Highlight the detector, enter a ‘Name’ and its ‘Type’, then click ‘Create sensor’.

![EnOcean Motion Sensor](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Learn_Motion.png)

## ADDING NEW ACTUATORS

#### CONFIGURE THE ACTUATOR

Add an actuator from Periphery tab:

![EnOcean Adding an Actuator](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Learn_Actuator.png)

#### PAIRING WITH THE ACTUATOR

Highlight the actuator you have just added and click on ‘Learn Device’ in the context tab:

![Pairning an Enocean Actuator](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Parining_Actuator.png)

1. Push the program button on the relay (LED flashes red).

2. Click ‘OK’ in the pop-up dialogue in the Loxone Configuration Software (LED on actuator will go red).

3. Confirm the pairing by pushing the program button on the relay again (LED goes off).

4. Save your configuration in Miniserver again and you are ready to go.

##

## CREATING CUSTOM ENOCEAN DEVICES

With custom EnOcean device can EnOcean devices include the not as Predefined sensors or actuators in the config are available.

The following telegram types can be integrated:
- RPS (05)
- 1BS (06)
- 4BS (07)
- RPS (F6)
- 1BS (D5)
- 4BS (A5)

The telegram type can be found in thedatasheett of their EnOcean device.

#### PASTE IN LOXONE CONFIG

Custom sensors and actuators EnOcean device can include both sensors and actuators.

When you create the custom sensors proceed as described under “Adding new device” before.

You can define custom actuators according to the manual ” learn wireless actuators “.

As a type, select each user-defined unit.

The device is now created, you can add sensors and actuators.

![Creating A Custom EnOcean Device Within Loxone Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Custom_Device.png)

In the Create sensors/actuators, the upper and lower bits of the value range can now, in the settings can be defined. For digital sensors/actuators, a values bit is defined. Which value range when will read their EnOcean device you can read the data sheet of the device . For sensors, you can use the EnOcean monitor to find out what data packets are sent from the device.

![Creating A Custom EnOcean Device Within Loxone Config Properties](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EnOcean_Custom_Device_Properties.png)

A button for example, sends the press ‘0x70’ and ‘0x00’ on release:

From this we can now identify which bits here responsive when switching

0x70 is binary = 0111 0000

0x00 is binary = 0000 0000

In this case, we can use either the 4.5 or 6 bits for the digital actuator. (least significant bit = 0 Index)

For analog sensors, a range of values must be defined. This must be in the datasheet of the sensor.

## COMPATIBLE ENOCEAN DEVICES

### COMPATIBLE ENOCEAN SENSORS

The EnOcean extension supports EnOcean wireless sensors which are based on EnOcean technology.

**PIR**

0 – No one detected

1 – Presence detected

**WINDOW HANDLE**

0 – Closed

1 – Open

2 – Slanted

**HUMIDITY SENSOR**

0 to 100%

**REED SWITCH / CONTACT SWITCH**

0 – Open

1 – Closed

**LIGHT, TEMPERATURE PRESENCE SENSOR**

Type 1 – 0 to 510 lux, 0 to 51°C

Type 2 – 0 to1020 lux, 0 to 51°C

Type 3 – 0 to 1530 lux, -30 to 50°C

**LIGHT SENSOR**

Type 1 – 300 to 60000 lux

Type 2 – 0 to 1024 lux

**ROOM CONTROLLERS**

The EEP number can be obtained from the manufacturer

**EEP 07-10-01/A5-10-01**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

Ventilation: 0 to 255

Presence Switch

**EEP 07-10-02/A5-10-02**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

Ventilation: 0 to 255

Day / Night Switch

**EEP 07-10-03/A5-10-03**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

**EEP 07-10-04/A5-10-04**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

Ventilation: 0 to 255

**EEP 07-10-05/A5-10-05**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

Presence Switch

**EEP 07-10-06/A5-10-06**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

Day / Night Switch

**EEP 07-10-07/A5-10-07**

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 255

**EEP 07-10-08/A5-10-08**

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 255

Presence Switch

**EEP 07-10-09/A5-10-09**

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 255

Day / Night Switch

**EEP 07-10-0A/A5-10-0A**

Temperature Sensor: 0 to 40°C

Controller: 0 to 255

Input Contact

**EEP 07-10-0B/A5-10-0B**

Temperature Sensor: 0 to 40°C

Input Contact

**EEP 07-10-0C/A5-10-0C**

Temperature Sensor: 0 to 40°C

Presence Switch

**EEP 07-10-0D/A5-10-0D**

Temperature Sensor: 0 to 40°C

Day / Night Switch

**EEP 07-10-10/A5-10-10**

Temperature Sensor: 0 to 40°C

Humidity Sensor: 0 to 100%

Controller: 0 to 255

Presence Switch

**EEP 07-10-11/A5-10-11**

Temperature Sensor: 0 to 40°C

Humidity Sensor: 0 to 100%

Controller: 0 to 255

Day / Night Switch

**EEP 07-10-12/A5-10-12**

Temperature Sensor: 0 to 40°C

Humidity Sensor: 0 to 100%

Controller: 0 to 255

**EEP 07-10-13/A5-10-13**

Temperature Sensor: 0 to 40°C

Humidity Sensor: 0 to 100%

Presence Switch

**EEP 07-10-14/A5-10-14**

Temperature Sensor: 0 to 40°C

Humidity Sensor: 0 to 100%

Day / Night Switch

**EEP 07-10-15/A5-10-15**

Temperature Sensor: -10 to 41.2°C

Controller: 0 to 63

**EEP 07-10-16/A5-10-16**

Temperature Sensor: -10 to 41.2°C

Controller: 0 to 63

Presence Switch

**EEP 07-10-17/A5-10-17**

Temperature Sensor: -10 to 41.2°C

Presence Switch

**EEP 07-10-18/A5-10-18**

Light Sensor: 0 to 1000 lux

Temperature Controller: 0 to 40°C

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**EEP 07-10-19/A5-10-19**

Humidity Sensor: 0 to 100%

Temperature Controller: 0 to 40°C

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**EEP 07-10-1A/A5-10-1A**

Supply Voltage: 0 to 5V

Temperature Controller: 0 to 40°C

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**EEP 07-10-1B/A5-10-1B**

Supply Voltage: 0 to 5V

Light Sensor: 0 to 1000 lux

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**EEP 07-10-1C/A5-10-1C**

Light Sensor: 0 to 1000 lux

Light controller: 0 to 1000 lux

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**EEP 07-10-1D/A5-10-1D**

Humidity Sensor: 0 to 100%

Humidity Controller: 0 to 100%

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**EEP 07-10-1E/A5-10-1E**

Supply Voltage: 0 to 5V

Light Sensor: 0 to 1000 lux

Temperature Sensor: 0 to 40°C

Ventilation: 0 to 7

Presence Switch

**SWITCH 1-WAY**

0 – Switch Off

1 – Switch On

**2-WAY SWITCH (WITH 1 UNDIVIDED PANEL)**

0 – Switch Off

1 – Switch On

**SWITCH 2-WAY**

0 – Switch Off

1 – Switch On

**SWITCH 4-WAY**

0 – Switch Off

1 – Switch On

**PUSH-BUTTON 2-WAY**

0 – Button not pressed

1 – Button pressed

**PUSH-BUTTON 4-WAY (WITH ONE-PIECE COVER)**

0 – Button not pressed

1 – Button pressed

**PUSH-BUTTON 4-WAY**

0 – Button not pressed

1 – Button pressed

**PUSH-BUTTON 8-WAY**

0 – Button not pressed

1 – Button pressed

**TEMPERATURE AND HUMIDITY SENSOR**

0 to 40°C, 0 to 100%

**TEMPERATURE SENSOR**

Type 1 -40 to 0°C

Type 2 -30 to 10°C

Type 3 -20 to 20°C

Type 4 -10 to 30°C

Type 5 0 to 40°C

Type 6 10 to 50°C

Type 7 20 to 60°C

Type 8 30 to 70°C

Type 9 40 to 80°C

Type 10 50 to 90°C

Type 11 60 to 100°C

Type 12 -60 to 20°C

Type 13 -50 to 30°C

Type 14 -40 to 40°C

Type 15 -30 to 50°C

Type 16 -20 to 60°C

Type 17 -10 to 70°C

Type 18 0 to 80°C

Type 19 10 to 90°C

Type 20 20 to 100°C

Type 21 30 to 110°C

Type 22 40 to 120°C

Type 23 50 to 130°C

### COMPATIBLE ENOCEAN ACTUATORS

The EnOcean extension supports EnOcean wireless actuators which are based on EnOcean technology.

**4BS EEP A5-38-08**

Dimmer

0 to 100%

**RPS EEP F6-02-01**

Blind actuator

**RPS EEP F6-03-01**

Universal actuator

**4BS EEP A5-20-01**

Valve actuator

**RPS EEP F6-02-01 (DIGITAL OUT)**

Relays

0 – Relay open

1 – Relay closed