# Blind control for privacy

Source: https://www.loxone.com/enen/kb/automatic-blind-control-for-privacy/

---

## Brief: I want my blinds to automatically close in the evening to offer privacy.

Means of shading, such as curtains or blinds, not only protects the fabric of a building from the sun’s rays and thus from overheating, but also offers privacy. In addition to the visual protection, there is a second effect, namely thermal insulation during the cold months. Because the additional layer in front of the window panes also has an insulating effect. While during the day the [automatic shading](https://www.loxone.com/enen/kb/automatic-blinds/) is activated automatically depending on the position of the sun, closing the blinds for privacy in the evening is often a manual task. So how can we realise automatic blind control for privacy?

In addition, if we are setting up automated blind control for privacy in the evening, then we need to deactivate this feature in the mornings, but at the appropriate time. This is all possible with Loxone, so let’s take a look.

## Solution: How to commission automatic blind control for privacy.

It is worth pointing out first that this solution requires a separation of living and bedrooms so that your customers aren’t woken up by deactivating the privacy function in the morning.

With the function block “[Shading Overview](https://www.loxone.com/enen/kb/shading-overview/)” you can create groups of specific doors and windows based on your customer’s requirements. It is recommended to differentiate between living rooms and bedrooms. Several groups can also be formed at the customer’s request. Different groups are necessary so that the privacy function is not activated or deactivated at the same time throughout.

You can define these groups if you double-click to open the dialogue box. Now you can link the Shading Overview with the respective rooms.

Example with all bedrooms combined:

If the shading on all necessary windows and doors is combined via the Function Block, you can use then the function “Times” in the peripheral tree and select the option ‘Pule for Sunset’. These time functions are calculated on the basis of the geo-coordinates of the property and require neither an active Internet connection nor a weather service subscription. Simply connect the “Pulse for Sunset” to the Cd input of the Shading Overview Function Block. You can do the same for the Shading Overview Function Block responsible for the recreational rooms by using the “Pulse for Sunrise” function to raise the shading.

					It is not recommended to use the function “Pulse at sunrise” to raise the shade in bedrooms. Because, especially in the summer months, it can be very early in the morning. This might not be favourable if your customer is not an early riser.

Hardware:
- [Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Shading Actuator Air (optional)](https://shop.loxone.com/enuk/shading-actuator-air.html)
- [Nano 2 Relay Tree (optional)](https://shop.loxone.com/enuk/nano-2-relay-tree.html)
- [Relay Extension (optional)](https://shop.loxone.com/enuk/relay-extension.html)
- [Nano IO Air (optional)](https://shop.loxone.com/enuk/nano-io-air.html)

### Configuration:

[
![Automatic Blind Control - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-61-Automatic-Blinds-Close-Recreational.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-61-Automatic-Blinds-Close-Recreational.png)

[
![Automatic Blind Control - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-61-Automatic-Blinds-Close-Bedrooms.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-61-Automatic-Blinds-Close-Bedrooms.png)

### Sample file:

### Automatic Blinds Close

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-61-Automatic-Blinds-Close.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-61-Automatic-Blinds-Close.loxone)

## Why you and your customer should consider automatic blind control for privacy.

The automatic closing or opening of the blinds offers your customers privacy once the sun goes down. Thanks to the geo-coordinates functions, you can automate blinds depending on location and season. The possibility of blind automation on windows and doors in groups increases convenience. The functions such as “Pulse for Sunset” are exactly calculated based on the geo-coordinates – so this feature is exceptionally precise.

Now, you can increase the comfort features in your customer’s home by considering automatic blind control to let some of the morning sun in as a natural way to wake up and heat a room.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)