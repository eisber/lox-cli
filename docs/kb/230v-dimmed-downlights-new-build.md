# 230V Dimmed Lighting – New Build

Source: https://www.loxone.com/enen/kb/230v-dimmed-downlights-new-build/

---

## OVERVIEW

When looking to utilise a current light fittings in a project often these are 230V fittings. If we are looking to dim these lights then we will need to look at hardware specific to the task. If not being lead into 230V dimming by a product (light fitting) limitation we would suggest looking into 24V dimming as this is a much more cost effective option better performance, [more here](https://www.loxone.com/enen/kb/24v-spotlights-new-build/).

When doing 230V dimming we use a technology called phase dimming and the product required to do this is the Dimmer Extension.

## REQUIRED PRODUCTS*
- Dimmer Extension
- [Touch Tree](http://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=touch+tree)
- [Motion Sensor Tree](http://shop.loxone.com/enuk/motion-sensor-tree.html)
- [Tree Extension](http://shop.loxone.com/enuk/tree-extension.html)
- [CAT7 Cable](http://shop.loxone.com/enuk/cat-7-cable.html)

*in addition to a Miniserver

## WIRING

The Dimmer Extension along with other Extensions and the Miniserver would be mounted centrally in the panel with the distribution board. Then the cable we suggest using is a Twin & Earth out on a radial to each circuit/group of lights you wish to control together. i.e. If there are two circuits of 6 lights in the kitchen and each group of 6 is being controlled separately each group would require it’s own T&E.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Please note that each dimmer channel (as per below diagram) needs the needs to have the neutral connected from the light circuit as well as a feed in. As such is is not possible to run multiple dimmer channels with a common neutral.

For technical documentation on the Dimmer Extension including loadings please go to the [**documentation**](https://www.loxone.com/enen/kb/dimmer-setup/) page for the dimmer.

![Example On How To Wire Lights To Extension Dimmer Loxone Miniserver](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Wiring_Lights_Dimmer.png)

Wiring for the Motion Sensor Tree and Touch/Touch Pure Tree couldn’t be simpler. These two devices require a Tree connection and a 24VDC power supply that is provided on a CAT7 cable. The topology of Tree allows you to run almost any network except a complete loop and save up to 80% on cabling over a basic STAR wiring layout, see below image for an example.

![IG_tree-wiring-overview](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IG_tree-wiring-overview.png)

The RGBW Dimmer Tree, Motion Sensor Tree and Touch/Touch Pure Tree all simply need a Tree connection and a power supply as per the below diagram.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us](https://www.loxone.com/enen/about-us/contact/) for more technical data if required.

## DESIGN CONSIDERATIONS

When specifying LEDs for 230V dimming we suggest that you back-rate the maximum load by 50%, this is to account for heating and potential inrush current. As such a 400W channel should not have more than 200W of LEDs being controlled by it. However as inrush varies hugely between products it is a good idea to check with the manufacturer what the potential peak load could be with their product.

As the circuitry inside a mains 230V dimmable LED will effect the dimming performance and this varies hugely between manufacturer and even within batches of the same manufacturer. We suggest you test your proposed fittings with the Dimmer Extension before purchasing them for the project.

The Loxone Touch, Touch Pure and Motion sensor Tree are very slimline, as such we recommend using/specifying European style circular back-boxes for the best fitment and finish. An example of this can be seen below, [click on the image to purchase them.](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

[
![Orange-circular-back-box](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/03/Orange-circular-back-box.jpg)
](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

## CONFIGURE EXTENSION

[
![Extension Search](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Extension_Search.png)
](https://www.loxone.com/enen/kb/configuring-extensions/)

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