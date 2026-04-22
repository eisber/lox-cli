# Guest Mode

Source: https://www.loxone.com/enen/kb/guest-mode/

---

## Brief: I want to set up a guest mode for when we have visitors at our home.

Fewer switches, more clarity – this is the motto under which we designed the [Loxone Switch Standard](https://www.loxone.com/enen/smart-home/standards-and-recommendations/#switchstandard). If the Switch Standard is followed, central functions can be triggered with multiple taps on certain switches.

For example, a triple tap on a switch in the master bedroom puts the entire house into Night Mode: all lights are turned off, music is switched off, the alarm system on the ground floor is activated and much more.

Most of the time this is great, it provides a great deal of convenience. However, in certain situations, it’s not always practical. Should a guest stay the night, a centralised Night Mode might not be the best option – as the guest might want to stay up later than the hosts, reading a book for example. To solve this issue, it would be good if a Guest Mode could be configured which would exclude the guest bedroom from the main Night Mode. In this Use Case, we’ll show you how to achieve this.

## Solution: Using Loxone to configure a guest mode.

An [operating mode](https://www.loxone.com/enen/kb/operating-modes/) called “Guest” is created, which signals the Loxone Miniserver that visitors will stay overnight.

This operating mode adjusts the way that certain whole house function take place:
- The guest rooms are excluded from the main Night Mode.
- A triple tap in the master bedroom has no effect on the guest rooms – lights, music and co. remain active.
- A triple tap in the guest room switches that room into its own Night Mode.

And the operating mode has even more effects:
- To save energy, the temperature in the guest rooms is normally set at “Frost Protection”. If the house is set to the “Guest” operating mode, the guest room and en-suite are heated or cooled to Comfort Temperature automatically.
- The guest rooms now also behave like normal living rooms (e.g. privacy protection through shading).
- Although you want to make your guests as comfortable as possible, this should not be at the cost of your sleep. If the main Night Mode has been activated in the master bedroom, the volume of the Multiroom Audio System for the guest room is limited to 30%.

**Note:** If the house does not have a separate guest toilet, the main bathroom will only be set to Night Mode once both the master bedroom and guest room are in Night Mode.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)

### Configuration:

[
![Guest Mode - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-95-Guest-Mode.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-95-Guest-Mode.png)

### Download the sample file:

### Guest Mode

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-95-Guest-Mode.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-95-Guest-Mode.loxone)

## Why you and your customer should consider setting up a guest mode?

The implementation of a guest operating mode significantly increases everyone’s experience when visitors come to stays.

Guests will not experience any unpleasant surprises, such as their lights switching off mid-book when Night Mode is activated in the master bedroom. They can use the guest room independently of the rest of the house.

The homeowners do not have to give up the conveniences of the usual central functions and do not have to change their behaviour in any way. They also don’t need to “prepare” the guest rooms – thanks to the operating mode taking care of, heating, shading and co.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)