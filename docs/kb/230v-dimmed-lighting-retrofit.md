# 230V Dimmed Lighting – Retrofit

Source: https://www.loxone.com/enen/kb/230v-dimmed-lighting-retrofit/

---

## OVERVIEW

When looking to utilise a current light fittings in a project often these are 230V fittings. If we are looking to dim these lights then we will need to look at hardware specific to the task. If not being lead into 230V dimming by a product (light fitting) limitation we would suggest looking into 24V dimming as this is a much more cost effective option better performance, [more here](https://www.loxone.com/enen/kb/24v-spotlights-retrofit/).

When doing 230V dimming we use a technology called phase dimming and the product required to do this is the Nano Dimmer Air.

## REQUIRED PRODUCTS*
- Nano Dimmer Air
- Touch Air
- Motion Sensor Air

*In addition to a Miniserver Go

## WIRING

The Nano Dimmer Air is designed to fit in-line with the current light circuit to enable dimming control of the circuit. The Nano Dimmer Air can be mounted either in the ceiling void or in the switch back box.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Please note that if mounting the Nano Dimmer air in a back box then it needs to be at least 35mm deep and ideally 45mm in order to fit.

The diagram below shows how the Nano Dimmer is wired inline for the light circuit. Although the Nano Dimmer Air has two connection points for light circuits it is only a single channel dimmer therefore they can only be controlled together.]

![Wiring for the Loxone Nano Dimmer Air](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Product_Nano_Dimmer_Air_Wiring.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
As with all Loxone 230V phase dimming the Neutral is also required both “in” and “out”. As such in UK homes the best place to locate the Nano Dimmer Air is in the ceiling void as we rarely have 4 cores available at the light switch back box to facilitate this.

If you are unsure at any point when wiring up any of our devices, please contact a suitably qualified electrician. Alternatively you can [contact us](https://www.loxone.com/enen/about-us/contact/) for more technical data if required.

## DESIGN CONSIDERATIONS

When specifying LEDs for 230V dimming we suggest that you back-rate the maximum load by 50%, this is to account for heating and potential inrush current. As such a 400W channel should not have more than 200W of LEDs being controlled by it. However as inrush varies hugely between products it is a good idea to check with the manufacturer what the potential peak load could be with their product.

As the circuitry inside a mains 230V dimmable LED will effect the dimming performance and this varies hugely between manufacturer and even within batches of the same manufacturer. We suggest you test your proposed fittings with the Dimmer Extension before purchasing them for the project.

## CONFIGURE AIR DEVICE

[Click here to see documentation on how to learn a Loxone Air Device in to Loxone config.](https://www.loxone.com/enen/kb/setting-up-air-devices/)

## CONFIGURE LIGHTING CONTROLLER

The next stage in the process is to configure the Lighting Controller in Loxone config, click on the image below to see how:

[**Click Here**](https://www.loxone.com/enen/kb/lighting-solutions-config/)

[
![FS_notebook-on-table-loxone-config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/FS_notebook-on-table-loxone-config.jpg)
](https://www.loxone.com/enen/kb/lighting-solutions-config/)