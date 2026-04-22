# Power Supplies

Source: https://www.loxone.com/enen/kb/power-supplies/

---

Loxone provides various power supplies that all have inputs of 230v and output 24v at varying amperage. These are the following power supplies that we can provide;
9.6w~0.4A

31.2w~1.3A

60w~2.5A*

100.8w~4.2A

150w~6.25A**

240w~10A

**This Power supply is recommended for LED strips. Can support up to 7 meters of LED lights.

****This is a standalone power supply, ideal for retrofitting. *

Shop links to the power supplies:

[0.4A](https://shop.loxone.com/enuk/psu-24v-04a.html)                [2.5A](https://shop.loxone.com/enuk/led-psu-24v-25a.html)                 [6.25A](https://shop.loxone.com/enuk/power-adapter-24v-605.html)

[1.3A](https://shop.loxone.com/enuk/psu-24v-13a.html)                [4.2A](https://shop.loxone.com/enuk/psu-24v-42a.html)                 [10.0A](https://shop.loxone.com/enuk/psu-24v-10a.html)

### RECOMMENDATION

When assembling the panel we recommend keeping the panel organized into layers with the terminal blocks at the top, main modules in the middle and the power supplies at the bottom. If setup properly this allows room for growth and also groups each component into function blocks. Furthermore, by keeping the power supplies nearby each other, it makes it far easier to setup common grounds between the power supplies which should be done to prevent floating voltage. As always ensure that you have slightly more wattage than what you need to compensate for any fluctuations that might occur. Using the dedicated [Project Planning](https://www.loxone.com/enen/kb/project-planning/) feature in Loxone Config will help you determine the power supplies you will require. An example of a properly setup panel can be here or on the [wiring basics page](https://www.loxone.com/enen/kb/wiring-basics/) (note setup designs may vary depending on free space within the panel).

![EN KB Wiring Cabinet Layout 1080x874](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Wiring_Cabinet_Layout.png)

It is also important to split up the modules on the 2nd row into different groups and power each group individually. By segregating the modules and therefore, splitting up the power supply, you can ensure that if one power supply fails and it’s not connected to the Miniserver, you avoid total failure. Our setup recommendation is to group the Miniserver and Extensions together and dimmer modules together.  A final note is that only the TDK DRF240-24-1 power supply unit is suitable for parallel operation. More information about this topic can be found [here](https://www.loxone.com/enen/kb/connecting-power-supplies-parallel/).

Illustration for the cabling required to set up a Miniserver and Extension with multiple power supplies (for the GND we recommend a cross-section of 2.5mm2).

![Loxone Miniserver Being Connected To Multiple 24V Power Supply](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Wiring_Miniserver_Multiple_PSU.png)

#### Further Links

Cabinets, designed specifically for Loxone can be found [here](http://www.futureautomation.co.uk/Enclosure/Loxone-Lighting-Enclosure).

Loxone Tree Cable can be found [here](https://shop.loxone.com/enuk/cat-7-cable.html).