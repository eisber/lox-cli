# Smart Air Conditioner Control

Source: https://www.loxone.com/enen/kb/smart-air-conditioner-control/

---

## Brief: I want to implement smart air conditioner control.

If you’ve worked in an office you’ll know there’s normally constant debate as to whether the office is too hot or too cold.

Let’s paint the picture, it’s a hot day outside so the air conditioning has been turned on in the office. It’s a great temperature for some people however, for others it’s way too cold. To avoid the conflict, they might simply open a window near them to let some of the outside heat in.

Problem solved for them. However, now we have an air conditioning system which is, quite literally, throwing both energy and money out of the window. To avoid this complete waste, it would be sensible if there was smart air conditioner control – in this Use Case we’ll show you how to set that up using Loxone.

## Solution: Using Loxone to commission smart air conditioner control.

In order to create smart air conditioner control, the system needs to know if any windows or doors are open. To achieve this you can simply use a [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html). In Loxone Config, this is then connected to the air conditioning system via the ‘lw’ input on the “[Room Ventilation Controller](https://www.loxone.com/enen/kb/ventilation/)” Function Block. If a window is opened, the air conditioning will be deactivated.

We have used a “Switch-On Delay” block, to ensure that the air con is only turned off when a window has actually been left open rather than temporarily opened. Only when the window has been open for longer than 30 seconds will the air conditioning be switched off.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html)

### Configuration:

[
![Smart Air Conditioner Control - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-71-Turn-Off-Office-Aircon-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-71-Turn-Off-Office-Aircon-2.png)

### Download the sample file:

## Why you and your customer should consider implementing smart air conditioner control?

The thermoregulatory centre of the human brain is located in the hypothalamus – deep inside the brain. There where unconscious body functions are controlled and regulation mechanisms take place.

When it is cold, the hypothalamus and thyroid gland are stimulated to increase metabolic rate and produce more heat. However, this process does not happen in the same way for all people. Weight, sex, age and stress levels are just some of the factors that influence this very complicated biological process.

The result, however, is anything but complicated; windows are opened. In combination with an active air conditioning system, this leads to unnecessary costs. This Use Case can help to minimize these costs.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)