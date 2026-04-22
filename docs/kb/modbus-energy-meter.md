# Modbus Energy Meter

Source: https://www.loxone.com/enen/kb/modbus-energy-meter/

---

#### How to get started with your Modbus Electricity Meter

## TECHNICAL DATA

#### SINGLE PHASE
- Single-phase energy meter, 230VAC 50Hz
- Direct measurement up to 32A
- Display of active power, voltage and current
- Modbus RTU interface to query the data
- Reactive power and cosφ available through interface
- 7-digits display
- Accuracy class B according to EN50470-3
- Accuracy class 1 according to IEC62053-21
- Bidirectional
- Easy to connect to Modbus Extension using existing template
- Ambient temperature: -20 … +55°C

#### 3 PHASE
- 3-phase energy meter, 3 x 230/400VAC 50Hz
- Direct measurement up to 65A
- Display of active power, voltage and current for every phase
- Display of active power for all phases
- Modbus RTU interface to query the data
- Reactive power for every and/or all phases available
- 7-digits display
- Accuracy class B according to EN50470-3
- Accuracy class 1 according to IEC62053-21
- Bidirectional
- Easy to connect to Modbus Extension using existing template
- Ambient temperatur: -20 … +55°C

## WIRING

#### SINGLE PHASE

![Single Phase Wiring Modbus ](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Mod_Bus_Single_Phase_Wiring.jpg)

#### 3 PHASE

![Three Phase Wiring Modbus ](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Mod_Bus_Three_Phase_Wiring.jpg)

## DIMENSIONS

#### SINGLE PHASE

![Single Phase Dimensions Modbus ](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Mod_Bus_Single_Phase_Dimensions.png)

#### 3 PHASE

![Three Phase Dimensions Modbus ](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Mod_Bus_Three_Phase_Dimensions.png)

## BASIC SETUP

In Loxone Config, there is a template for the Loxone Modbus Electricity Meter.

To add the template highlight your Modbus Extension in the Periphery window, click on the drop-down menu “Sensors and Actuators” and under “Predefinded devices” you will find the Loxone Modbus Electricity meter.

![modbus image 1 678x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/modbus-image-1.png)

The device is now listed in the Periphery tree with all the predefined sensors. If needed, you can find some more information in the datasheet.

If you use more than one Loxone Modbus Electricity meter, the Modbus address has to be adjusted. The default address is 1.

![Screenshot 6 800x625](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Screenshot_6.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If you want to create any other sensors (see datasheet), choose the following settings in the Properties of the sensor:

**IO address:** depends on data point, see datasheet

**Command:** 3 – Read holding register(4x)

**Data type:** 16-bit signed integer

#### POTENTIAL INSTALLATION

![Modbus Example Instaliation Setup Diagram](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Modbus_Energy_Meter_Inverter_Distribution.png)

## FIND OUT MORE

[Contact us](https://www.loxone.com/enen/about-us/contact/)

## DOWNLOADS

[Datasheet Loxone Modbus Electricity meter single phase](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/08/Datasheet_ModbusEnergyMeter_1phase_200156.pdf)

[Datasheet Loxone Modbus Electricity meter 3-phase](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/08/Datasheet_ModbusEnergyMeter_3phase_200157.pdf)

EC declaration of conformity

[Download](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Loxone-Modbus-Conform.pdf)