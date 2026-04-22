# Damper Tree

Source: https://www.loxone.com/enen/kb/damper-tree/

---

# **Damper Tree**

##  TECHNICAL DATA
- Power supply: 15 … 30 VDC
- Duct diameter: 6”, 7”, 8”, 10” available, other sizes upon request
- Operating temperature: 0…55°C / 32…131°F
- Humidity: 80% RH
- Maximum power consumption: 0.2W, max. 2.3W (when motor is active)
- IP rating: IP20
- Wire cross-section: 0.25 … 0.8mm² / AWG18 … AWG23
- Wire stripping length: 5mm / 13/64”
- No maintenance required

## INSTALLATION

The installation of this device must be carried out by a qualified technician. When installed, the device must follow building regulations for electrical and fire safety. If the device is not installed according to manufacturer’s guidelines, the warranty of the device may be affected. Install the damper between the ventilation ducts and fix them with a foil tape that is certified to in that kind of installation.

![Damper Tree Drawing](https://www.loxone.com/enus/wp-content/uploads/sites/13/2018/01/Damper-Tree-Drawing.png)

## SETUP

Connect the power supply and Tree communication to the Damper Tree. Once connected, power up the device and configure it using Loxone Config.

[Click here to learn to how to pair Tree devices.](https://www.loxone.com/enen/kb/tree-cabling-setup/)

The following outputs for available for troubleshooting:
- Online Status

##

## CONFIGURATION

The Damper can be connected directly to the HC output of the Intelligent Room Controller function block.

![loxone damper tree irc config 800x156](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/02/loxone-damper-tree-irc-config.png)

## LED STATES

| RED steady blinking (ca. 0.5s on; 0.5s off) | No Tree communication possible, please check the wiring. |
| --- | --- |
| ORANGE steady blinking (ca. 0.5s on; 0.5s off) | Device is connected properly, but not paired with the Miniserver. |
| GREEN 3x short flashes | Device is paired, communication OK. |
| RED flashing (ca. 1Hz) | Online, Damper >75% open |
| ORANGE flashing (ca. 1Hz) | Online, Damper between 25% – 75% |
| GREEN flashing (ca. 1Hz) | Online, Damper < 25% open |