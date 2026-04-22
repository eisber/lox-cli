# LED States

Source: https://www.loxone.com/enen/kb/led-states/

---

## LED STATES

The Miniserver, extensions and most devices all have a set of LEDs that can be used to quickly diagnose the state of the device. By using the tables below you will be able to compare the LED states on a device with the matching set and then diagnose what those lights represent.

Below you can find several lists of possible LED states and their associated status.
- [Miniserver](#Miniserver)
- [Extensions](#Extensions)
- [Dimmer Extensions](#Dimmer)
- [Tree Actuators](#tree)
- [Air Devices](#Air)
- [Tree Devices](#Tree)
- [Miniserver LAN Port](#LAN)

**It is important to note that every device has an identifying method. On every device that has an LED, this includes it rapidly flashing between red and green. On some other devices, it also has another method (like the touch switch clicking) in which it identifies itself.**

*Solid colours are just on. Not flashing – constantly on.

## LED STATES FOR THE MINISERVER

| Left LED | Right LED | Meaning |
| --- | --- | --- |
|  |  | Everything OK, device is online. |
|  |  | One or more System Status messages are active. |
|  |  | Device was selected in Loxone Config and is identifying. |
|  |  | Update is in progress. |

**Boot Phase:**

| Left LED | Right LED | Meaning |
| --- | --- | --- |
|  |  | Miniserver is booting. |
|  |  | Miniserver is loading the bootloader image from the SD card. |
|  |  | Miniserver has successfully loaded the image and will unpack it as the next step. |
|  |  | Miniserver has successfully unpacked the image. |
|  |  | Operating system is started. |
|  |  | Miniserver is loading the program file. |
|  |  | SD card cannot be read. Check SD card. |
|  |  | No compatible operating system on the SD card. |

## LED STATES FOR EXTENSIONS

Loxone extensions share similar LED states as the Miniserver but with different meanings with the exception of the Dimmer Extension. By matching your Extension’s current LED status to the table below, you will be able to figure out what your extension is trying to tell you.

| Left LED | Right LED | Meaning | Solution (if required) |
| --- | --- | --- | --- |
|  |  | Extension is operating fine | N/A |
|  |  | Can communicate to a Miniserver just not leared in yet. | Learn the device in via Loxone Config. |
|  |  | Extension cannot communicate to a Miniserver. | Extension cannot communicate to a Miniserver. Check Loxone Link connection. |
|  |  | Device was selected in Loxone Config and is identifying. |  |
|  |  | Extension is updating | Please wait for the Extension to finish updating. |
|  |  | Miniserver includes a programming error | Please address the Miniserver using the table above. |

## LED STATES FOR THE DIMMER EXTENSION

On top of the above set of LED states, the Dimmer Extension has an extra LED state:

| Status of Left LED | Status of Right LED | Meaning |
| --- | --- | --- |
| Flashing Green | Red (1sec) | Mains voltage was switched on. |

## TREE VALVE ACTUATORS

> **ℹ️ Note:** No communication to Miniserver possible. Please check the wiring.

## LED STATES FOR AIR DEVICES

| LED State | Meaning | Solution |
| --- | --- | --- |
| Flashing Red/Green/Orange | Device is in pairing mode, ready for pairing. | Pair device |
| Flashing Red/Green rapidly | Device was selected in Loxone Config and is identifying. |  |
| Flashes Green or Flashes Green 3 times then off | Device has just been paired or has restarted and is now online. |  |
| Flashing Orange | Device is paired but cannot connect to the Air Base (offline). | Information about error analysis can be found here |
| Red for 10 seconds, then off | Device is low on battery | Replace the battery of the device. For more detailed information on battery replacement, refer to the documentation page of the Air device. |

					For many battery-operated Air devices with serial number 504F94FFFE**B**/**C**….., no LED status is displayed after removing and reinserting the battery. (Exception: Pairing Mode)
For such devices, after removing the battery, either press a button or wait for a minute to display the status after reinserting the battery.

## LED STATES FOR TREE DEVICES

| LED State | Meaning | Solution |
| --- | --- | --- |
| Flashing Red/Green rapid | Device was selected in Loxone Config and is identifying. |  |
| Flashes Green or Off | Device has just been paired or has restarted and is now online. |  |
| Flashing Orange | Tree wiring is fine. Device has not been paired yet. | Pair device |
| Flashing Red | Device cannot connect to the Miniserver via the Tree interface. | Check the wiring and the LED Status of the Tree Extension. Information about error analyisis can be found here |
|  |  |  |

## LAN PORT LED STATES

The Miniserver LAN port has the standard two LED indicator lights, here are the various states and the information they represent.

#### Orange: Activity LED

       – Off: No Link (cable may not be connected – check both ends or it may be severed)
       – On Link (But no activity)
       – Blinking Data Activity (the faster it is blinking the more activity)

#### Green: Speed LED

       – Off: 10 Mbps (or it is not connected -> Activity LED is also Off)
       – On: 100 Mbps

**Please note that the Miniserver Go’s LAN LEDs have been disabled and thus will not blink/turn on.**