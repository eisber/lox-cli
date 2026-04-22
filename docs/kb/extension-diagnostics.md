# Extension Diagnostics

Source: https://www.loxone.com/enen/kb/extension-diagnostics/

---

## WIRING TROUBLESHOOTING

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Whenever work is done on the cabling of the Miniserver of any extensions please ensure that power to the installation is removed. Damage to components can occur if this is not observed.

**NEVER WORK ON A LIVE PANEL**

Before rebooting the extensions you should double check that all the connectors are plugged back in correctly and only then should you power up.

The blue Loxone Link connectors are especially sensitive to being plugged and unplugged if there is power to the devices. When working with the equipment please do not work on it powered up!

## ONLINE TROUBLESHOOTING

If you have an extension with a status LED flashing orange and that won’t show up in the search in Loxone Config or is showing as offline look at the following table for tips on how to solve the problem:

| Reason | Solution |
| --- | --- |
| The Miniserver does not see the extension connected because it has not been added to the program. | Connect to your Miniserver and load the file from the Miniserver. Look in the periphery tree for the extension and check that the serial number is correct. If there is still a problem go to step 2. |
| Adding the extension to the program did not solve the problem or is not showing up in the search so cannot add. | 1. Reboot the Miniserver and extensions (by turning the power supply off for about 10 seconds until rebooting). The green connector power plug should NOT be removed if the power supply is on. 2. Switch off the power supply and check that the Loxone Link blue connector is plugged in correctly. Check that the last extension in the Link is terminated with the 120 Ohm resistor. Check the quality of the wiring and the connections, if there is a loose connection or break in the wire of the Loxone Link this would cause this problem. If multiple power supplies are present, make sure that the grounds of all power supplies are connected. If this is not done damage can be done to the Loxone Link due to potential differences. |
| If the wiring has been checked and the reboot didn’t work check the resistance of the Link connectors with no power to the device. | Take a multimeter and measure the resistance between the two terminals of the blue connector. You can see expected resistances below. If the measured value is a lot different from these and you have checked the wiring and resistor please contact our support. |
| If no errors were found in the resistance between the terminals of the blue connector continue to measure resistance. | 1. Take a multimeter and measure the resistance between each of the two terminals and ground. Depending on the total number of extensions the resistance will vary, see the table below on expected resistances. 2. If this minimum resistance is far from being met then connect the multimeter to ground and each terminal of each extension in turn. If the resistance greatly increases at a certain point then you will have identified the extension causing the problem. Once you have done this please contact our support. |

#### EXPECTED RESISTANCES

|  | Miniserver only | Extension only | Complete installation |
| --- | --- | --- | --- |
| Link to Link | 120 – 140 Ohm | > 20 kOhm | 60 – 65 Ohm |
| Link to GND | > 20 kOhm | > 20 kOhm | > 2 kOhm |

## EXTENSIONS ARE NOT FOUND OR SOMETIMES GO OFFLINE?

**1. Extension flashes orange**

Check that the extension is connected to the correct serial number included in the program and stored in the mini server. Connect to your Miniserver and load out the PLC Config program. Now mark the peripheral tree the respective extension. In the properties window, the serial number is now visible and can be adjusted if necessary. The serial number of the extension can be found on the reverse side of the product.

**. 2**

Turn off the whole system for about 10 seconds by turning off the circuit breaker. The green terminal of Loxone devices (= power supply) must NOT be withdrawn if the power supply is active.

**. 3**

The Extension continues to flash Orange, switch off the power supply.

Now check whether the blue terminal is inserted correctly, or the wires have been clamped. Possibly a wire is broken, or the isolation was clamped.

If multiple power supplies are present, make sure that GND of all power supplies are connected. If this connection is not given, it can also lead to damage to the Loxone Link interface by potential differences between the power supplies.

**. 4**

Now take a resistance measurement device (e.g multimeter) to measure the resistance between the two poles of the Loxone links. (Between the two screws on the Link Terminal)

The resistance should be around 60 ohms.

Please refer to the above table for expected resistances across Miniserver links and GND.

Any divergences between their measured value, check if the Loxone link on the last extension with a 120 ohm resistor has been terminated, and the continuity of the line.

**. 5**

If no errors have been found, then measure the resistance of the individual Loxone Link pole to GND. Depending on the number of connected extension of the resistance value should be at least 1 to 50 ohms.

**. 6**

If this minimum resistance is met, connect the ohmmeter between GND and Loxone Link terminal. Tighten the Loxone link terminals (blue terminal) all extension sequentially from and keep you always display the ohmmeter in mind. Take one extension after another from Loxone link until the resistance increases by leaps and bounds. In this case, then there may be a defect of the respective extension that was just disconnected.

Please contact the support!

**. 7**

Now close again all extension to the Loxone link. After verifying that all connectors are inserted correctly, you can activate the power supply again.

If the problem still persists, please contact the support!