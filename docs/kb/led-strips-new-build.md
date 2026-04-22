# LED Strips – New Build

Source: https://www.loxone.com/enen/kb/led-strips-new-build/

---

## OVERVIEW

When using LED strips there are two main options that are available, RGBW and Warm White. RGBW gives the opportunity to have full colour spectrum control. LEDs by their nature don’t work at 230V so there measures put in place to convert it. However by dropping down to 24V we avoid any of issues of 230V. One of the main reasons for using 24V is cost, dimming mains voltage LEDs requires much more expensive hardware in comparison to that needed for dimming at 24VDC. This difference can be up to 1/3 of the cost. In addition to this quality of the dimming and the huge reduction in inrush current make 24V lighting our choice when it comes to not just LED Strips but also when looking at LED Spot lighting.

This information is in relation to the Loxone articles [200043](https://shop.loxone.com/enuk/led-strip-warm-white-ip20.html), [200061](https://shop.loxone.com/enuk/led-strip-warm-white-ip65.html), [200092.](https://shop.loxone.com/enuk/led-strip-warm-white-ip68.html)

[View Datasheet >>](https://www.loxone.com/wp-content/uploads/datasheets/Datasheet_LEDStrip_WW_200043,200061,200092.pdf)

## REQUIRED PRODUCTS
- [Warm White LED Strip](http://shop.loxone.com/enuk/led-strip-ww.html?___SID=U) or [RGBW LED Strip](http://shop.loxone.com/enuk/led-strip-rgbw.html?___SID=U)
- This information is in relation to the Loxone articles [200043](https://shop.loxone.com/enuk/led-strip-warm-white-ip20.html), [200061](https://shop.loxone.com/enuk/led-strip-warm-white-ip65.html), [200092.](https://shop.loxone.com/enuk/led-strip-warm-white-ip68.html)
- [View Datasheet >>](https://www.loxone.com/wp-content/uploads/datasheets/Datasheet_LEDStrip_WW_200043,200061,200092.pdf)
- RGBW 24V Dimmer Tree
- [Tree Extension](http://shop.loxone.com/enuk/tree-extension.html)
- [24VDC Power Supply](http://shop.loxone.com/enuk/accessories.html?c=psu-network#content)
- [Touch Tree](http://shop.loxone.com/enuk/touch-tree.html) or [Touch Pure Tree](http://shop.loxone.com/enuk/touch-pure-tree.html)
- [Motion Sensor Tree](http://shop.loxone.com/enuk/motion-sensor-tree.html)
- [CAT7 Cable](http://shop.loxone.com/enuk/cat-7-cable.html)

*In addition to a Miniserver

## WIRING

In a new build property the RGBW 24V Dimmer Tree is located in the main cabinet along with the Miniserver, Tree Extension and PSUs. From that location the appropriate cable is taken from the RGBW 24V Dimmer Tree to the strip, be it a RGBW or a Warm White one. Suggested cable from RGBW 24V Dimmer Tree to LED Strip:
- RGBW Spot – 5-core Flex
- Warm White – Twin and Earth

The core thicknesses of cable that you use will be defined by both building regulations and the load/distance over which the cable will be run, thus these must be calculated for your particular project.

The voltage drop can be calculated using the following formula: **ΔU = I · R = I · ((2 · L · ρ) / A)**

I … Current [A]

L … Cable Length [m]

A … Cable Cross-Section [mm²]

ΔU … Voltage Drop [V]

ρ … Material Specific Resistance [( Ω*m)/mm²] ρ= 0.0172 (for Copper)

For technical documentation on the [RGBW 24V Dimmer Tree click here](http://loxone.com/enen/kb/rgbw-24v-dimmer-tree/).

The RGBW 24V Dimmer Tree can be used in two forms to allow it to control either single colour Warm White LED Strips, these two modes are referred to as “Individual dimming channels” or “RGBW dimming”. The common terminal on the RGBW Dimmer Tree is the +24V meaning that we are dimming on the -ve side. If “Individual dimming channels” is selected as PWM configuration in the properties of the device, the channel assignment is as follows:

Red    -> Dimmer 1

Green -> Dimmer 2

Blue    -> Dimmer 3

White  -> Dimmer 4

For “RGBW dimming” we need to use all 4 channels in order to have full RGBW (Red, Green, Blue, White) control on each circuit. In the diagram below we can see how each RGBW LED Strip is connected with each channel wired in parallel.

![Wiring Loxone RGBW Tree Dimmer](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/EN_KB_Diagram_RGBW_Tree_Wiring.png)

Wiring for the Motion Sensor Tree and Touch/Touch Pure Tree couldn’t be simpler. These two devices require a Tree connection and a 24VDC power supply that is provided on a CAT7 cable. The topology of Tree allows you to run almost any network except a complete loop and save up to 80% on cabling over a basic STAR wiring layout, see below image for an example.

![IG_tree-wiring-overview](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IG_tree-wiring-overview.png)

The RGBW Dimmer Tree, Motion Sensor Tree and Touch/Touch Pure Tree all simply need a Tree connection and a power supply as per the below diagram.

![Loxone Tree Wiring](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_Tree_Wiring.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us ](https://www.loxone.com/enen/about-us/contact/)for more technical data if required.

## DESIGN CONSIDERATIONS

When designing a key thing to consider is the overall loading for the system. This effects how many RGBW Dimmers required, how many and of what type power supply is required and as per above what cable thickness will be needed to cope with the load.

## CONFIGURE TREE DEVICE

![IC_tree@2x-1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IC_tree@2x-1.png)
For information on how to connect your Tree Device with Loxone config then [click here!](https://www.loxone.com/enen/kb/tree-cabling-setup/)

## CONFIGURE LIGHTING CONTROLLER

[
![FS_notebook-on-table-loxone-config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/FS_notebook-on-table-loxone-config.jpg)
](https://www.loxone.com/enen/kb/lighting-solutions-config/)