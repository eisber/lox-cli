# 230V Switched Lighting – Retrofit

Source: https://www.loxone.com/enen/kb/230v-switched-lighting-retrofit/

---

## OVERVIEW

A switching light circuit is one that has only two states, on or off. For switching light circuits we utilise relays. Our retrofit wirelessly controlled relay device is called the Nano IO Air.

## REQUIRED PRODUCTS*
- Nano IO Air
- Touch Air
- Motion Sensor Air

*In addition to a Miniserver Go

## WIRING

The Nano IO Air is designed to fit in-line with the current light circuit to enable control of the circuit. The Nano IO Air can be mounted either in the ceiling void or in the switch back box.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Please note that if mounting the Nano IO Air in a back box then it needs to be at least 35mm deep and ideally 45mm in order to fit.

The diagram below shows how the Nano Dimmer is wired inline for the light circuit. On the diagram you can see the Nano IO Air has 2 relay outputs. These can be separately controlled meaning that the Nano IO can be used to control 2 completely separate light circuits.

![Wiring Nano IO Air Relays](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_Wiring_Nano_IO_Air.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
To power the Nano IO Air the Neutral is also required. As such in UK homes the best place to locate the Nano IO Air is in the ceiling void as we rarely have a neutral available at the light switch back box to facilitate this.

If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us](https://www.loxone.com/enen/about-us/contact/) for more technical data if required.

## DESIGN CONSIDERATIONS

When specifying LEDs for 230V we suggest that you back-rate the maximum load by 50%, this is to account for heating and potential inrush current. As such a 5A channel should not have more than 2.5A of LEDs being controlled by it. However as inrush varies hugely between products it is a good idea to check with the manufacturer what the potential peak load could be with their product.

## CONFIGURE AIR DEVICE

[Click here to see documentation on how to learn a Loxone Air Device in to Loxone config.](https://www.loxone.com/enen/kb/setting-up-air-devices/)

## CONFIGURE LIGHTING CONTROLLER

The next stage in the process is to configure the Lighting Controller in Loxone config, click on the image below to see how:

[**Click Here**](https://www.loxone.com/enen/kb/lighting-solutions-config/)

[
![FS_notebook-on-table-loxone-config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/FS_notebook-on-table-loxone-config.jpg)
](https://www.loxone.com/enen/kb/lighting-solutions-config/)