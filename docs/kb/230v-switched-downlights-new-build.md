# 230V Switched Lighting – New Build

Source: https://www.loxone.com/enen/kb/230v-switched-downlights-new-build/

---

## OVERVIEW

A switching light circuit is one that has only two states, on or off. For switching light circuits we utilise relays. The relays we have on extensions in Loxone are volt free contact closures. Meaning we will switch out whatever is fed in to the relay.  There are a number of extensions in Loxone that have relays in different quantities 9between 8 and 16), these are the Miniserver, Extension, Relay Extension and Multi-Extension Air.

## REQUIRED PRODUCTS*
- Relay Extension, Miniserver or Extension
- [Touch Tree](http://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=touch+tree)
- [Motion Sensor Tree](http://shop.loxone.com/enuk/motion-sensor-tree.html)
- [Tree Extension](http://shop.loxone.com/enuk/tree-extension.html)
- [CAT7 Cable](http://shop.loxone.com/enuk/cat-7-cable.html)

*In addition to a Miniserver

## WIRING

The Extension along with other Extensions and the Miniserver would be mounted centrally in the panel with the distribution board. Then the cable we suggest using is a Twin & Earth out on a radial to each circuit/group of lights you wish to control together. i.e. If there are two circuits of 6 lights in the kitchen and each group of 6 is being controlled separately each group would require it’s own T&E.

For technical documentation on the Extensions including loadings please go to the [**documentation**](https://www.loxone.com/enen/kb-cat/extensions/) pages.

![Example On How To Wire Switches And Actuators To Loxone Miniserver](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Wiring_Switches_Actuators.png)

Wiring for the Motion Sensor Tree and Touch/Touch Pure Tree couldn’t be simpler. These two devices require a Tree connection and a 24VDC power supply that is provided on a CAT7 cable. The topology of Tree allows you to run almost any network except a complete loop and save up to 80% on cabling over a basic STAR wiring layout, see below image for an example.

![IG_tree-wiring-overview](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IG_tree-wiring-overview.png)

The RGBW Dimmer Tree, Motion Sensor Tree and Touch/Touch Pure Tree all simply need a Tree connection and a power supply as per the below diagram.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us](https://www.loxone.com/enen/about-us/contact/) for more technical data if required.

## DESIGN CONSIDERATIONS

When specifying LEDs for 230V we suggest that you back-rate the maximum load by 50%, this is to account for heating and potential inrush current. As such a 5A channel should not have more than 2.5A of LEDs being controlled by it. However as inrush varies hugely between products it is a good idea to check with the manufacturer what the potential peak load could be with their product.

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