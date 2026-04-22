# Miniserver Diagnostics

Source: https://www.loxone.com/enen/kb/miniserver-diagnostics/

---

## CONTENTS
- Problems on startup
- Problems with connecting

## PROBLEMS ON STARTUP

Unpack your Miniserver from its box and connect the power supply as well as a network cable from the router to the Miniserver. Only power up the Miniserver once all wiring has been completed. The boot process takes about 7-10 seconds. After this process the status LED on the front of the Miniserver will be flashing green.

If this is not the case refer to the following table for the solution:

| LED status | Reason | Solution |
| --- | --- | --- |
| Flashing yellow | The Miniserver is in pause mode. | 1. Reboot the Miniserver and extensions (by turning the power supply off for about 10 seconds until rebooting). The green connector power plug should NOT be removed if the power supply is on. 2. If the behaviour remains the same then format the SD card to factory defaults according to the instructions. |
| Flashing red / yellow | The Miniserver does not recognise the SD card or cannot access the card. | 1. Check that the SD card is inserted correctly. 2. Reboot the Miniserver and extensions (by turning the power supply off for about 10 seconds until rebooting). The green connector power plug should NOT be removed if the power supply is on. 3. If the behaviour remains the same then format the SD card to factory defaults according to the instructions. |
| Flashing red / red / yellow | There is a faulty PLC program in the Miniserver. | 1. Reboot the Miniserver and extensions (by turning the power supply off for about 10 seconds until rebooting). The green connector power plug should NOT be removed if the power supply is on. 2. If the behaviour remains the same then format the SD card to factory defaults according to the instructions. |

## PROBLEMS WITH CONNECTING

If the Miniserver is flashing green but you cannot connect to it in Loxone Config use the following table to resolve the issue:

> **ℹ️ Note:** Solution should be:

## COMISSIONING CHECKLIST

#### INITIAL START-UP

**1.**

Take your Miniserver from the package and connect the power supply and the network. Activate the power supply only when the wiring work has been completed on the mini server. After activating the power supply starts the Mini Server.

**1.1**

The boot process takes about 7 seconds, after the green status LED Miniserver flashes at regular intervals. Flashing the status LED is green, the following shall mean:

**1.1.1 Yellow flashing:**

the Mini Server is in pause mode. Remedy:

**1.1.1.1**

Mini server reboot (power supply for about 10 seconds to disable), if behavior remains the same:

**1.1.1.2**

SD card with Loxone Config loud instructions format

**1.1.2 Red / Yellow flashing:**

Miniserver no SD card recognized / not access the card. Remedy:

**1.1.2.1**

Check the SD card is inserted correctly

**1.1.2.2**

Mini server reboot (power supply for about 10 seconds to disable), if behaviour remains the same:

**1.1.2.3**

SD card with Loxone Config loud instructions format

**1.1.3 Red / Red / Yellow flashing:**

Faulty PLC program in the mini server. Remedy:

**1.1.3.1**

Mini server reboot (power supply for about 10 seconds to disable), if behaviour remains the same:

**1.1.3.2**

Format SD card with Loxone Config

#### MINI SERVER FLASHING GREEN, BUT CAN NOT BE REACHED OVER THE NETWORK

**1.**

Verify the status LEDs on the network (LAN) port of the Miniserver flashing light.

**1.1**

the Green LED lights permanently and the orange LED flashes (by flashing is network traffic signals) all right, continue to step 2.

**1.2**

No Blinking / Lighting of LEDs: Verify that the network cable is plugged in correctly.

Stay the LEDs still dark, please contact the Support.

**1.3**

Both LEDs are lit continuously, even when no cable is connected: please contact Support

**2.**

The Minierver blinks green and the network interface signaling traffic. Now change the Loxone Config Software and click on the “mini server” on “Search”. The Mini Server should now be listed in the search results slip and you can connect to it.

If there is no Miniserver found:

**2.1**

Recheck the network jacks. Possibly also replace the LAN cable.

**2.2**

Check your firewall, or your anti Four program whether the Loxone Config Software was there classified as “Confidential”.

**2.3**

Network Configuration Verify how you are connected to the Mini Server?

Important: The network must on your Windows PC as a home or workplace network have been classified!

**connected 2.3.1 Mini Server Router**

Open now the command prompt. (Start -> Run -> cmd)

Now enter the command ” arp -a ” a. There are now listed all active network connections. Search for the entry with the MAC address EE E0-00 -… this is your mini server.

Now enter the following command with the just released found IP address (in our example, 192.168.1.77): ping 192.168.1.77

Replace 192.168.1.77 with the IP of your Miniserver!

**2.3.1.1 Ping successfully**

They have four times get a response from that address, the mini server is so accessible.

Switch back to the Loxone Config Software. Under “Mini Server” on “Connect”. You should now enter the newly found-out IP address. User and password default is “admin”. Is not possible, disable Test way firewall and virus scanner. Now, a connection should be possible.

**2.3.1.2 Ping failed**

The answer you get “timeout” or another error.

Now assign the mini server a static IP address.

Instructions can be found here . Then run back to ping through.

**2.3.1 Mini server directly connected to PC**

Have the Mini server connected directly to your PC, you must assign your PC and also the mini server a static IP address.

Instructions can be found here .

**2.4 no connection possible**

Please contact our Support.