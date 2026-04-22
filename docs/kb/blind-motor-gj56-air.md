# Blind Motor GJ56 Air

Source: https://www.loxone.com/enen/kb/blind-motor-gj56-air/

---

Instructions for commissioning the Venetian blind actuator GJ56 Air

## Technical Information

|  | 100251 & 100253 | 100252 & 100254 |
| --- | --- | --- |
| Opening | 100251: Top 100253: Bottom | 100252: Top 100254: Bottom |
| Voltage | 230V〜/50Hz | 230V〜/50Hz |
| Power Supply | 0.4A | 0.85A |
| Cos Phi (cos ?) | >0.95 | >0.95 |
| Inrush Current (Factor) | x 1.2 | x 1.2 |
| Power | 93W | 190W |
| Torque | 6Nm | 2 x 10Nm |
| Number of Revolutions | 26 1/min | 26 1/min |
| Protection Class | IP 54 | IP 54 |
| Total Length (m.pl) | 319.5 mm | 356.7 mm |
| Diameter | 55 mm | 55 mm |
| Weight | ca. 1.50 kg | ca. 2.20 kg |
| Operating Mode | S2 4 min. | S2 4 min. |
| Operating Temperature & Humidity | Operation: T = -10 ° C … on request / H = max. 90% Storage: T = -15 ° C … + 70 ° C / H = dry, non-condensing |

##

## Assembly

Here you will find useful documents for dimensioning and assembling the Venetian blind actuator.

![Loxone Download button](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Icon_Download_Button.png)
[Dimensioning diagram of the drive](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/01/GJ56_Auswahldiagramm-6-10-20Nm.pdf?x48792)

![Loxone Download button](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Icon_Download_Button.png)
[Installation and start-up instructions](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/GEIGER_E_BAL_GJ56-AIR-F03_100W2500-001_EN.pdf)

The required motor torque can be determined as follows:

![EN KB Diagram Torque Motor 768x477 300x186](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/EN_KB_Diagram_Torque_Motor-768x477.jpg)

## COMISSIONING

#### Initial commissioning

After the power supply is switched on, the device will automatically be in learn mode for 30 minutes.

#### Re-learn in procedure

Similar to initial learn in, after the power supply is switched on, the device will automatically go into learn mode for after 2 minutes of not connecting to an Airbase. The learn in mode will then be active for 10 minutes.

[For full instructions on learning in Loxone Air devices click here.](https://www.loxone.com/enen/kb/setting-up-air-devices/)

Once the device has been learnt in, the device will be listed under the I/O tree.

To use the blind drive, simply drag it onto a program page.

![EN KB Config Automatic Blinds Integrated 1024x160](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/EN_KB_Config_Automatic_Blinds_Integrated.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)

Also the Geiger transmitter LC Air (wall and hand transmitter) can communicate directly with the Miniserver GO or the Air Base Extension. For this, however, the device must be placed in the Loxone mode and then into the learning mode. To do this, remove the battery and set the switch on the board inside the device to “LX”. As soon as the battery is inserted again, the learning mode starts automatically.
The further teach-in functions as in the case of the [Remote Air](https://www.loxone.com/enen/kb/remote-air/).

## Adjust the end positions

After the motor has been connected and learned into the Airbase, the end positions must now be defined.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)

Depending on the installation of the blind drive, the direction of rotation may be different. For this reason, the blinds should be in a central position when teaching the end positions so that the blind can travel in both directions without encountering an obstacle.

To learn in the end positions you will need to activate the ‘Service mode’ in the app (App version 7.4).

![gj56 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/gj56-1.png)

![gj56 2](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/gj56-2.png)

Once the service mode has been started, you can use the buttons in the app to change the position of your shading.

Once the end position has been reached, save it using “End position”.

![gj56 3](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/gj56-3.png)

First the lower end position must be adjusted, then the upper end position. Use the service mode of the Loxone Smarthome App.

Each end position can be learned independently from the other with or without a mechanical override switch.

The blind drive confirms the learning of the lower end position with “clack-clack”, the upper end position and thus the end of the end position storage is confirmed by “clack-clack-clack-clack”. Between the learning of the lower end position and the learning of the upper end position one has 2 minutes time, otherwise the previously set values (or the factory state) are restored – in this case, the blind drive signals this through 6 “clack” noises.

Once the end positions have been learned, the service mode in the app can be closed.

If you want to re-learn the end positions, you can restart the learning process by switching the motor off and after 30 minutes the service mode can be activated again.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
**

No end positions are set in the delivery state!**

## Programming in Loxone Config

The programming and actuation of the Venetian Blind Actuator GJ 56 Air is carried out with the Function Block “Automatic Venetian Blind Integrated”.

Just pull across the GJ 56 Air from the periphery tree onto the program page. The motor is now linked directly to the block and you can control the drive without additional outputs.

[You can find more information about the automatic shutter programmer here.](https://www.loxone.com/enen/kb/automatic-blinds/)

The following status outputs of the GJ 56 Air Venetian blind actuator are shown at the “Automatic blind integrated” block.

| Shutter position AQp | Analog output | Target position of the shading area: 0.000 (quite open) to 1.000 (at the bottom) |
| --- | --- | --- |
| Slat position AQl | Analog output | Slat position of the shading area: 0.000 (horizontal) to 1.000 (vertical) |
| Motor movement Qm | Analog output | On = motor in motion |
| Motor blocked Qb | Analog output | On = Motor blocked |
| Obstacle detected Qo | Analog output | On = obstacle detected |
| Automatic status Qa | Analog output | Indicates the automatic status of the shading |
| Safety shutdown Status Qs | Analog output | Shows whether the safety shutdown is currently active |
| Lock status Ql | Analog output | Output is active as long as St is On |

## Slat adjustment travel correction

Loxone practice tip

#### Preparation

![EN KB Product Slat Adjustment 1024x631](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/EN_KB_Product_Slat_Adjustment.jpg)

Before you start the correction, attach a piece of wire to one of the slats.

#### STEP 1:

![EN KB Product Slat Adjustment 2 1024x631](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/EN_KB_Product_Slat_Adjustment_2.jpg)

Tilt the slats by approx. 45 ° (guide value). Align the previously attached wire horizontally. If necessary, check with a spirit level whether the wire is also aligned horizontally. Save the slat position.

#### STEP 2:

![EN KB Product Slat Adjustment 3 1024x631](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/EN_KB_Product_Slat_Adjustment_3.jpg)

In step 2, place the slats in the exact same position as in step 1. The previously attached wire should be brought back into the horizontal position. If the slats are too much inclined, you can start the process again, otherwise save the slat position.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
[>> General information about Loxone Air technology can be found here <<](https://www.loxone.com/enen/products/air/)

Original installation and operating instructions together with EC declaration of conformity for the Venetian blind actuator GJ56 Air (pdf)

[Download](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/01/GEIGER_E_BAL_GJ56-AIR-F03_100W2500-001_DE-1.pdf)