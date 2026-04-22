# Indoor climate in a Real Smart Home

Source: https://www.loxone.com/enen/kb/overview-indoor-climate/

---

A perfect indoor climate in a Real Smart Home means enjoying the ideal room temperatures at any time of the day or night with the best indoor air quality at all times. When the seasons change, this is a particular challenge. Especially during the transition period, the Smart Home is often required to cool certain rooms and heat others.

In order for you to enjoy a perfect indoor climate, we have developed function blocks in Loxone Config that make the configuration of the heating, cooling and ventilation systems simple and straightforward.

## Interaction of the individual function blocks

Depending on the degree of possible integration with the HVAC system, the configuration is extended by adding various function blocks or additional logics.

### Level 1

Level 1 is to be selected when there is no way to influence the heating system. Use the Intelligent Room Controller + Valve Actuator Tree or Air. We have integrated the necessary sensors in many Loxone products for the precise determination of room temperature and humidity. It is recommended to use one of the following products:
- [Touch Tree/Air](https://shop.loxone.com/enen/loxone-touch.html)
- [Touch Pure Tree/Air](https://shop.loxone.com/enen/loxone-touch-pure.html)
- [Room Comfort Sensor](https://shop.loxone.com/enen/room-comfort-sensor.html)

![climate level1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/09/climate_level1.png)

![IC Temp rot](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/08/IC_Temp-rot.png)
**

Heating:

**

In the first level of integration of a heating system, only the flow rate is regulated by the actuators. There is no influence on the heating system. If the outside temperature is higher than the inside temperature, the ventilation system can support the heating to reach the desired temperature.

![IC Temp blau](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/08/IC_Temp-blau.png)

**Cooling:**

With a controllable ventilation system the ventilation can support the cooling of the room, taking into account the outside temperature.

Especially for ventilation control we recommend the use of our Room Comfort Sensor. In addition to temperature and humidity the CO2 value in the room can be measured.

### Level 2

In this level, the heating and/or cooling requirements of the entire building are determined with the aid of the Climate Controller. This function block determines whether priority should be given to heating or cooling. Even if you do not have an active cooling system, the Climate Controller decides whether the heating source is switched on or off.

![climate level2](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/09/climate_level2.png)

### Level 3

In order to increase the energy saving potential to a maximum, it is wise to determine the heating system’s flow temperature according to demand. This can be done using the “Intelligent temperature control” block. The module determines the flow and buffer target temperature based on the room sizes and current heating demand.

![climate level3](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/09/climate_level3.png)

### Level 4

The highest level of expansion in terms of a perfect room climate is to provide the calculated flow temperature via a heating mixer. The circulation pump can be controlled via the “Mixing Valve Controller” to control the exact flow temperature required.

![climate level4 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/09/climate_level4-1.png)

![Pro-Tipp](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/02/IC_Profi-Tipp-Security-DE.png)

**It is important that the necessary [integrations](https://www.loxone.com/enus/products/technology/) are coordinated with the responsible HVAC installer and technician. In this way you achieve the best possible comfort and increase the energy saving potential.**