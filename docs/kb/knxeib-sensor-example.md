# KNX/EIB Sensor Example

Source: https://www.loxone.com/enen/kb/knxeib-sensor-example/

---

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Based on cost considerations and depending on your installation plans, we generally recommend using conventional push buttons as digital inputs on the Miniserver and Extension or utilising the Loxone Tree or Loxone Air technologies. Under certain conditions, combining EIB sensors with Loxone actuators makes sense. For example the customer desires a specific EIB push button. Below, we focus on the use of EIB sensors.
Please note the following points:
- Since Loxone responds to each rising edge, it is essential that the EIB push button is configured correctly in ETS
- To function properly, an EIB push-button must be programmed to to send a ‘1’ (ON) when pushed, and a ‘0’ (OFF) upon release

#### PUSH BUTTON EXAMPLE

The following screenshots show a Jung-4X push-button in ETS and then in Loxone Config.

![ETS Example for Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Computer_ETS_Config.png)

If the push button is configured as described, you can use the Loxone Miniserver for anything.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If your EIB push button does not support this, use the ‘EIB Extended Sensor’.

![EIB Sensor Properties KNX](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EIB_Sensor_Properties.jpg)

In your configuration, use the function blocks found in the configuration software as you would normally. For the function blocks, it is irrelevant if the input is connected to an EIB sensor or a conventional switch (digital input).

Configuration example: Blinds

![Loxone and EIB Sensors for Blinds](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EIB_Blinds.png)

Configuration example: Lighting

![Loxone and EIB Sensors for Lighting](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_EIB_Lighting.png)