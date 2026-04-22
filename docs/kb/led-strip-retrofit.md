# LED Strip Retrofit

Source: https://www.loxone.com/enen/kb/led-strip-retrofit/

---

## OVERVIEW

LED Strip tape is becoming more and more popular in new build homes but also in existing homes where renovations or retrofit systems are being installed. Due to the almost infinite colours and mood settings the Loxone LED strip tape can bring an all new atmosphere to a. With the low voltage lighting, LED tape has a very long lifetime which means they are used a lot for everyday lighting now.

Some examples would be low level stair lighting, plinth lighting, TV backing, under bed/mirrors & so much more.

On this page you’ll find information on what products & configuration are required for LED Spot lights in a new build property.

## REQUIRED PRODUCTS*
- [24V Power Supply](http://shop.loxone.com/enuk/accessories.html?c=psu-network#content)
- [RGBW Dimmer Air](http://shop.loxone.com/enuk/rgbw-24v-dimmer-air.html?___SID=U)
- [LED Tape](http://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=led)
- [LED Strip Accessories*](http://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=led+accessories)
- [Airbase Extension*](http://shop.loxone.com/enuk/air-base-extension.html)

*In addition to the Miniserver you may need an Airbase Extension if you are not using a Miniserver Go.

You may also wish to consider the LED strip accessories depending on the instal requirement.

## Wiring

In an existing property the RGBW 24V Dimmer Air is normally located near where the LED strip will be run. The RGBW Dimmer Air will be getting power from a local PSU as well. From that location the appropriate cable is taken from the RGBW 24V Dimmer Air to the LED strip, be it multiple colour strip or same colours.

Suggested cable from RGBW 24V Dimmer Tree to LED Strip:
- RGBW Spot – 5-core Flex
- Warm White – Twin and Earth

The core thicknesses of cable that you use will be defined by both building regulations and the load/distance over which the cable will be run, thus these must be calculated for your particular project.

For technical documentation on the [RGBW 24V Dimmer Tree click here. ](https://www.loxone.com/enen/kb/rgbw-24v-dimmer-air/)

The RGBW 24V Dimmer Air can be used in two forms to allow it to control either single a colour of LED Strip or multiple colours (using each channel for normally red, green, blue, white), these two modes are referred to as “Individual dimming channels” or “RGBW dimming”.

In the diagram below we can see an example of the “RGBW dimming”.

![RGBW Dimmer Air Wiring](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_RGBW_Dimmer_Air.png)

*Please note that the lamp icons represent light outputs and in this case are representing LED strip tape.

If you are wanting to use single channels and use the device as “Individual dimming channels” function, you would need to common the + 24V to each + 24V cable on each individual strip and then the remaining colour cable will have their own separate channel.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us](https://www.loxone.com/enen/about-us/contact/) for more technical data if required.

## CONFIGURE AIR DEVICE

[Click here to see documentation on how to learn a Loxone Air Device in to Loxone config](https://www.loxone.com/enen/kb/setting-up-air-devices/).

## CONFIGURE LIGHTING CONTROLLER

[
![FS_notebook-on-table-loxone-config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/FS_notebook-on-table-loxone-config.jpg)
](https://www.loxone.com/enen/kb/lighting-solutions-config/)