# Inrush Current

Source: https://www.loxone.com/enen/kb/inrush-current/

---

## CONTENTS
- [What is Inrush Current?](#what)
- [When does it happen?](#when)
- [How does it effect Loxone?](#howe)
- [How to prevent issues?](#howp)
- [More information](#read more)
- [Contact](#Contact us)

## WHAT IS INRUSH CURRENT?

Inrush current, sometimes also referred to as input surge current, start-up current or switch-on surge, is the maximum current drawn for the instant when an electrical device is switched on. Devices such as transformers may draw several times their normal working current when first powered up. This will last for only a short period of time but it is nevertheless an important effect that needs to be considered.

![cached](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/01/cached.jpeg)

**source: www.eldoled.com*

Inrush current must be accounted for when selecting circuit protection equipment and also any other component in the system that will then be exposed to this.

## WHEN DOES IT HAPPEN?

In short, any time you turn on any electric device there will be an element of inrush current, some devices have very large inrush which can be many times the nominal current, some so little that it does not even need to be considered.

As an example, when turning on a load such as a 230V LED light there will be an element of inrush current. This is because with any 230V LED light there will be a small transformer to step the voltage down and into DC in order to actually run the LED and the starting up of this capacitive load is what creates the inrush current.

## HOW DOES IT EFFECT LOXONE?

#### LIGHTING

The most common situations where inrush current will effect a Loxone system is when controlling 230V LED lighting. This is due to as stated above this requires a transformer in the circuit (normally in the back of the lamps). The effect of this is that each time the circuit of 230V LEDs are turned on the transformers (drivers) are turned on and this is what generates inrush current. It is possible, even common, to have lights where the inrush current is more than 10 times the nominal current.

As a result of this, the inrush then needs to be considered as this current will then be flowing through the relay or dimmer controlling the system.

#### OTHER LOADS

The inrush current of a particular device is hard to predict and this information will need to come directly from the manufacturer of the unit being installed. Devices such as motors, transformers, and even resistive loads such as heaters can all produce some inrush current.

Obviously, such high potential loads must be considered if they are being controlled by any electrical equipment and Loxone systems are no different. To avoid damaging relays and dimmer channels it is essential that the specified limits in the Technical Documentation are adhered to.

#### EXAMPLE

If a circuit of 230V LED spots are to be controlled by a Nano Dimmer Air, this has a maximum load power output of 200W. If 10x 11W (0.048A) LED Spots are specified the nominal load will be 110W, within the maximum recommended nominal loading of the dimmer.

However, if these spotlights exhibit an inrush current of 0.5A per spot (a factor of 10 inrush) there could be an instantaneous load of 5.5A on the Nano Dimmer Air, that is 1265W for an instant on startup. This over time will damage the dimmer (or relay if not specified to account for this).

As you can see it is essential to consider inrush current if wishing to avoid damage to Loxone hardware. This applies to any load being switched or dimmed which includes but is not limited to motors, heating mats, 230V LED lighting.

## HOW TO PREVENT ISSUES

#### USE 24V DC LIGHTING

When it comes to LED lighting the best thing that can be done is to move to using [24V DC lighting](https://shop.loxone.com/enuk/accessories.html?c=lighting#content). As all the control of these lights is done “upstream” of the power supply (transformer) it is not being switched on and off and therefore is not putting any inrush current through the Loxone products. This means that on a 24V 200W dimmer, you can run up to 200W of LED spots without having to worry about inrush.

#### CORRECTLY SPECIFY RELAYS & DIMMERS

With circuits that cannot be converted to 24V DC, such as motors or chandelier lighting then an alternative is required. In the first instance gathering information on what the inrush current on the device will be and ensuring that the controllers are not being overloaded.

For example on a switched lighting circuit that is going to give an inrush of over 5A the relays on the Miniserver or Extension are rated at only 5A, however, those on the Relay Extension are rated at 16A so this may be the better choice.

In some cases, only an individual circuit is going to be subject to a high inrush current, we also have a [Coupling Relay (or contactor)](https://shop.loxone.com/enuk/coupling-relay.html?___SID=U) that is specified for inrush current up to 25A.

## READ MORE

There are lots of great resources on inrush current on the internet as it is such a common issue to be designed around. Here are some of the best:

[Wikipedia – Inrush Current](https://en.wikipedia.org/wiki/Inrush_current)

[muRata – What is inrush current?](https://www.murata.com/en-eu/products/emiconfun/emc/2012/10/29/en-20121029-p1)

[AMETHERM](https://www.ametherm.com/inrush-current/)

## CONTACT US

For further information on how to avoid issues with inrush current in your Loxone installation please talk with our technical support team by submitting a ticket to **[[email protected]](/cdn-cgi/l/email-protection)** or calling us on **01183 130140**.