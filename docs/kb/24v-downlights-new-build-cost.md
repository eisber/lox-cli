# 24V Downlights – New Build

Source: https://www.loxone.com/enen/kb/24v-downlights-new-build-cost/

---

## OVERVIEW

Utilising 24V for spotlighting in a new build is the best method over 230V as per conventional. LEDs by their nature don’t work at 230V so there are measures put in place to convert it however this introduces a number of complications and performance concerns. By dropping down to 24VDC however we avoid any of issues of 230V. One of the main reasons for using 24V is cost, dimming mains voltage LEDs requires much more expensive hardware in comparison to that needed for 24VDC. This difference can be up to 1/3 of the cost.

In addition to this, quality of the dimming and the huge reduction in inrush current make 24V lighting our choice when it comes to not just spotlights but also when looking at LED strip lighting.

On this page you’ll find information on what products & configuration are required for LED Spot lights in a new build property.

## REQUIRED PRODUCTS*
- [24V Warm White Spots ](http://shop.loxone.com/enuk/led-spot-ww.html?___SID=U)(in ceiling or surface mounted) or [24V RGBW Spots ](http://shop.loxone.com/enuk/led-spot-rgbw.html)(in ceiling or surface mounted)
- [Tree Extension](http://shop.loxone.com/enuk/tree-extension.html)
- [RGBW 24V Dimmer Tree](http://shop.loxone.com/enuk/led-spot-tree-rgbw.html)
- [24VDC Power Supply](http://shop.loxone.com/enuk/accessories.html?c=psu-network#content)
- [Touch Tree](http://shop.loxone.com/enuk/touch-tree.html) or [Touch Pure Tree](http://shop.loxone.com/enuk/touch-pure-tree.html)
- [Motion Sensor Tree](http://shop.loxone.com/enuk/motion-sensor-tree.html)
- [CAT7 Cable](http://shop.loxone.com/enuk/cat-7-cable.html)

*In addition to a Miniserver

## WIRING

Wiring of the 24V Spotlights is very simple. The RGBW Dimmer Tree would be located in the control panel alongside the other Loxone hardware. From here an appropriately rated power cable for the load and distance can be run. Depending on the type of spot (RGBW or Warm White) you will need either a 5 or a 2 core cable. In the case of the warm white where the connections are 24V and GND this is the two core and with the RGBW Spotlight this is a 5-core for the 4 RGBW and common 24V.

### RGBW

![Wiring RGBW Spot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_RGBW_Spot.png)

### WARM WHITE

![Wiring Loxone Warm White Spot Downlight](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_WW_Spot.png)

### OTHER TREE DEVICES

Wiring for the Motion Sensor Tree and Touch/Touch Pure Tree couldn’t be simpler. These two devices also require a Tree connection and a 24VDC power supply that is provided on a CAT7 cable. Again the topology of Tree allows you to run almost any network except a complete loop and save up to 80% on cabling over a basic STAR wiring layout, see below image for an example.

![IG_tree-wiring-overview](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IG_tree-wiring-overview.png)

The RGBW Dimmer Tree, Motion Sensor Tree and Touch/Touch Pure Tree all simply need a Tree connection and a power supply as per the below diagram.

![Loxone Tree Wiring](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_Tree_Wiring.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us](https://www.loxone.com/enen/about-us/contact/) for more technical data if required.

##

## DESIGN CONSIDERATIONS

The Loxone Touch, Touch Pure and Motion sensor Tree are very slimline, as such we recommend using/specifying European style circular back-boxes for the best fitment and finish. An example of this can be seen below, [click on the image to purchase them.](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

[
![Orange-circular-back-box](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/03/Orange-circular-back-box.jpg)
](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

## CONFIGURE TREE DEVICE

[
![IC_loxone-tree@2x-1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IC_loxone-tree@2x-1.png)
](https://www.loxone.com/enen/kb/tree-cabling-setup/)For information on how to connect your Tree Device with Loxone config then [click here!](https://www.loxone.com/enen/kb/tree-cabling-setup/)

## CONFIGURE LIGHTING CONTROLLER

The next stage in the process is to configure the Lighting Controller in Loxone config, click on the image below to see how:

[**Click Here**](https://www.loxone.com/enen/kb/lighting-solutions-config/)

[
![FS_notebook-on-table-loxone-config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/FS_notebook-on-table-loxone-config.jpg)
](https://www.loxone.com/enen/kb/lighting-solutions-config/)