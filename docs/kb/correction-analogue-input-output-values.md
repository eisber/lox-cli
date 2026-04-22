# Correction of Analogue Input & Output Values

Source: https://www.loxone.com/enen/kb/correction-analogue-input-output-values/

---

#### Information on how to correct measured values

## INTRODUCTION TO CORRECTION

All analogue inputs and outputs in the Loxone Config have got a correction section in their properties. This makes it possible to scale and also correct analogue inputs and outputs. For example, a temperature sensor connected to an analogue input.

The sensor used in this example has a measuring range of -30°C to 70°C. The analogue input however provides a signal from 0-10V. You can scale this in the software so that the 0-10V signal is displaying the temperature range, not the voltage. This is done in the properties of the analogue input, so click on the input and look down to the Correction section.

![EN KB Config Correction 2 1024x586](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Correction_2.png)

For this sensor you would put the following values in the correction as displayed in the screenshot.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | – 30 | 10 | 70 |

The signal is now adjusted in this linear graph:

![Correction Graph](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_Correction-1.png)

## SENSOR CORRECTION VALUES

[Brightness](#BRIGHTNESS SENSOR)

[Indoor Temperature](#INDOOR TEMPERATURE SENSOR)

[Indoor Temperature & Humidity](#INDOOR TEMPERATURE & HUMIDITY SENSOR)

[Indoor Temperature, Humidity & CO2](# INDOOR TEMPERATURE, HUMIDITY & CO2 SENSOR)

[Outdoor Temperature](#OUTDOOR TEMPERATURE SENSOR)

[Outdoor Temperature & Humidity](#OUTDOOR TEMPERATURE & HUMIDITY SENSOR)

[Pressure](#PRESSURE SENSORS)

[Sauna Temperature & Humidity](#SAUNA TEMPERATURE SENSOR)

[Ultrasonic](#ULTRASONIC SENSOR)

[Enocean](#Enocean)

#### BRIGHTNESS SENSOR

Product code: 200032

Brightness is adjustable using DIP switches from 0 to 100,000 lux. The sensor is supplied with the default measuring range set from 0 to 100 lux.

Inside the device, the DIP switches 8, 9 & 10 define the brightness range. The correction values would be:

| Switch 8 | Switch 9 | Switch 10 | Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 | 0 | 0 | 0 | 10 | 100 |
| 0 | 0 | 1 | 0 | 0 | 10 | 500 |
| 0 | 1 | 0 | 0 | 0 | 10 | 1000 |
| 0 | 1 | 1 | 0 | 0 | 10 | 5000 |
| 1 | 0 | 0 | 0 | 0 | 10 | 20,000 |
| 1 | 0 | 1 | 0 | 0 | 10 | 60,000 |
| 1 | 1 | 0 | 0 | 0 | 10 | 100,000 |

For more information & DIP switch settings, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200032-brightness-sensor.pdf).

#### INDOOR TEMPERATURE SENSOR

Product code: 200006

Temperature range is adjustable using DIP switches from -50°C to 200°C. The sensor is supplied with the default measuring range set from -30°C to 70°C.

Inside the device, the DIP switches 1, 2, 3 & 4 define the temperature range. The correction values would be:

| Switch 1 | Switch 2 | Switch 3 | Switch 4 | Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 | 0 | 0 | 0 | – 50 | 10 | 50 |
| 1 | 0 | 0 | 0 | 0 | – 50 | 10 | 150 |
| 0 | 1 | 0 | 0 | 0 | – 30 | 10 | 70 |
| 1 | 1 | 0 | 0 | 0 | – 20 | 10 | 50 |
| 0 | 0 | 1 | 0 | 0 | – 20 | 10 | 80 |
| 1 | 0 | 1 | 0 | 0 | – 20 | 10 | 120 |
| 0 | 1 | 1 | 0 | 0 | 0 | 10 | 50 |
| 1 | 1 | 1 | 0 | 0 | 0 | 10 | 70 |
| 0 | 0 | 0 | 1 | 0 | 0 | 10 | 70 |
| 1 | 0 | 0 | 1 | 0 | 0 | 10 | 100 |
| 0 | 1 | 0 | 1 | 0 | 0 | 10 | 150 |
| 1 | 1 | 0 | 1 | 0 | 0 | 10 | 200 |

For more information & DIP switch settings, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200006-indoor-temperature-sensor.pdf).

#### INDOOR TEMPERATURE & HUMIDITY SENSOR

Product code: 2000010

The temperature and humidity range on this device is fixed. The correction values for this device are:

Temperature range from -30°C to 70°C.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | – 30 | 10 | 70 |

Humidity is from 0 to 100% relative humidity.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 100 |

For more information, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200006-indoor-temperature-sensor.pdf).

####

INDOOR TEMPERATURE, HUMIDITY & CO2 SENSOR

Product code: 2000017

The temperature, humidity and CO2 range on this device are fixed. The correction values for this device are:

Temperature range from 0°C to 50°C.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 50 |

Humidity is from 0 to 100% relative humidity.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 100 |

CO2 measuring range is from 0 to 2000 ppm.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 2000 |

For more information, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200017-co2-temperature-sensor.pdf).

#### OUTDOOR TEMPERATURE SENSOR

Product code: 200003

Temperature range is adjustable using DIP switches from -50°C to 200°C. The sensor is supplied with the default measuring range set from -30°C to 70°C.

Inside the device, the DIP switches 1, 2, 3 & 4 define the temperature range. The correction values would be:

| Switch 1 | Switch 2 | Switch 3 | Switch 4 | Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 | 0 | 0 | 0 | – 50 | 10 | 50 |
| 1 | 0 | 0 | 0 | 0 | – 50 | 10 | 150 |
| 0 | 1 | 0 | 0 | 0 | – 30 | 10 | 70 |
| 1 | 1 | 0 | 0 | 0 | – 20 | 10 | 50 |
| 0 | 0 | 1 | 0 | 0 | – 20 | 10 | 80 |
| 1 | 0 | 1 | 0 | 0 | – 20 | 10 | 120 |
| 0 | 1 | 1 | 0 | 0 | 0 | 10 | 50 |
| 1 | 1 | 1 | 0 | 0 | 0 | 10 | 70 |
| 0 | 0 | 0 | 1 | 0 | 0 | 10 | 70 |
| 1 | 0 | 0 | 1 | 0 | 0 | 10 | 100 |
| 0 | 1 | 0 | 1 | 0 | 0 | 10 | 150 |
| 1 | 1 | 0 | 1 | 0 | 0 | 10 | 200 |

For more information & DIP switch settings, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200003-outside-temperature-sensor.pdf).

#### OUTDOOR TEMPERATURE & HUMIDITY SENSOR

Product code: 200031

The temperature and humidity range on this device is fixed. The correction values for this device are:

Temperature range from -30°C to 70°C.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | – 30 | 10 | 70 |

Humidity is from 0 to 100% relative humidity.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 100 |

For more information, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200031-outside-temperature-humidity-sensor.pdf).

#### PRESSURE SENSORS

0-0.3bar Product code: 2000154

0-6bar Product code: 200203

The pressure sensing range on these devices are fixed. The correction values for this device are:

Pressure range from 0bar to 0.3bar.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 0.3 |

Pressure range from 0bar to 6bar.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 6 |

For more information on the Pressure Sensor 0-0.3bar, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200154-PressureSensor.pdf).

For more information on the Pressure Sensor 0-6bar, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200203-Drucksensor.pdf).

#### SAUNA TEMPERATURE SENSOR

Product code: 200136

The temperature and humidity range on this device is fixed. The correction values for this device are:

Temperature range from -30°C to 120°C.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | – 30 | 10 | 120 |

Humidity is from 0 to 100% relative humidity.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 0 | 10 | 100 |

For more information, see the data sheet of the device: [**Download**](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/07/EN_Datasheet_200136_Sauna_TemperaturFeuchtefuehler.pdf).

#### ULTRASONIC SENSOR

Product code: 200054

Measuring range from 350mm to 6000mm.

Humidity is from 0 to 100% relative humidity.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | 350 | 10 | 6000 |

#### USING ENOCEAN SENSORS WITH DIFFERENT TEMPERATURE RANGES

There are countless EnOcean sensors available and whilst there is some standardisation within the EnOcean protocol for temperature ranges, there are many sensors that do not conform to this. Even though the configuration software may not list the range of a particular EnOcean sensor it is still possible to use this sensor, by scaling the values that are received to reflect the correct range.

For instance, if a sensor with a range from -20°C to 55°C is used there is no suitable EnOcean protocol. If one therefore selects the protocol of a sensor that usually measures 0°C to 40°C, then a reading of 0°C actually reflects a real temperature of -20°C and a reading of 40°C a real temperature of 55°C.

To adjust the scaling both the upper and the lower values need to be scaled. Please always use input value 1 and display value 1 for the lower adjustment and input value 2 and display value 2 for the upper adjustment. Hence for the above example one would enter:

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | – 20 | 40 | 55 |

##

## DEVIATION ADJUSTMENTS

Quite often when measuring analogue signals, especially temperature there can be some deviation from the true temperature.

For example if a temperature sensor was behind a switch plate on a wall and near a door, this temperature sensor could be measuring say 21°C. However using another hand held thermostat the room temperature in the middle of the room is actually 23°C.

The calculation for this is relatively straight forward. The steps are:

1. Work out the total range of the sensor. In this example we are using an indoor temperature sensor with a range from -30°C to 70°C. Total range = 100.

2. Divide the analogue range by the total range: 10/100 = 0.1

3. Therefore 0.1V = 1°C

4. Calculate the voltage at zero: -30 to 0 is a range of 30. Multiply 30 by 0.1 = 3V at zero degrees.

5. Finally multiply the temperature sensor’s reading of 21°C by 0.1 and add 3 for the voltage = 5.1V.

Values:

Temperature measured by the sensor: 21°C

Measuring voltage: 5.1V

Actual room temperature: 23°C

The correction values for this sensor would be:

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | – 30 | 5.1 | 23 |

The temperature has now been shifted upwards by 2°C upwards so the temperature will be displaying accurately now.

#### CORRECTION WITH 1-WIRE, AIR OR ENOCEAN SENSORS

If the temperature measurement is incorrect and needs to be corrected this can be done by adjusting the upper parameters.

**Temperature measured by the sensor:** 21°C

**Actual temperature in the room:** 22.5°C

The correction values would be: Now we just edit the correction settings (input value 2 and display value 2) in the Properties of the temperature input.

| Input Value 1 | Display Value 1 | Input Value 2 | Display Value 2 |
| --- | --- | --- | --- |
| 0 | -0 | 21 | 22.6 |

The temperature has just been shifted upwards by 1.5°C upwards so the temperature is displayed accurately.