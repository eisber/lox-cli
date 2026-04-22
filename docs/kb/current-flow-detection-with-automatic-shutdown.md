# Current Flow Detection with Automatic Shutdown

Source: https://www.loxone.com/enen/kb/current-flow-detection-with-automatic-shutdown/

---

## Brief: Is there a way to automate current flow detection and shut down the power if necessary?

Many devices can be switched on and off automatically – depending on predefined criteria. However, what happens if one of these devices happens to break down? The owner of the home or building presumes that everything is running safely thanks to automation, however, a part of their system is not working and they are none the wiser. To prevent damage, it is therefore important to monitor whether a device is actually on using current flow detection.

A good example of where current flow detection would be useful is a rainwater tank (cistern) which has a pump installed. When the tank reaches a certain water level the pump is automatically switched on to allow the water to be used for watering the garden. However, should the pump stop working no water will be available for the sprinkling system. Without current flow detection in place, the sprinklers would turn on anyway and if there’s no water available this could cause them to break.

Another example is a ventilation system. Depending on predefined criteria such as temperature or humidity, the ventilation is automatically activated. Over time, this can lead to deposits building up in the ventilation outlet. This leads to higher resistance and can cause a problem for the ventilation system.

To avoid these scenarios, it is important to monitor such devices using current flow detection. If a failure occurs, the owner or the person in charge should be informed as soon as possible. All this can be achieved with Loxone and we’ll show you how in this Use Case.

## Solution: Using Loxone for monitoring and detecting current flow.

In this Use Case, we’ll work with a central ventilation system in a building. Two fans are integrated into the Loxone system through a [Nano 2 Relay Tree](https://shop.loxone.com/enuk/nano-2-relay-tree.html). The Nano 2 Relay Tree is a flush-mounted relay that includes current detection – meaning it can detect the electrical current flow of the connected fans. If a blockage occurs in the fan, the resistance in the ventilation system increases. The ventilation system must, therefore, use more energy to continue to do its job. This increased power requirement is detected by the Nano 2 Relay Tree. If the current requirement is higher than 4 amperes, the ventilation is automatically switched off. The building owner or another relevant person will be informed via a push notification: “Shutdown of the central ventilation system due to excessive power consumption! Please check the ventilation system!”

In Loxone Config, the inputs of the Nano 2 Relay Tree provide the information on the current flow to the “Switch” Function Block. This block switches the fans on or off based on the information from the Nano 2 Relay Tree.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Nano 2 Relay Tree](https://shop.loxone.com/enuk/nano-2-relay-tree.html)

### Configuration:

[
![Current Flow Detection with Automatic Shutdown - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-63-Current-Flow-Detection-Central-Ventilation-Shut-down.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-63-Current-Flow-Detection-Central-Ventilation-Shut-down.jpg)

### Download the sample file:

### Current Flow Detection

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-63-Current-Flow-Detection.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-63-Current-Flow-Detection.loxone)

## What are the advantages of using Loxone for current flow detection?

Automated processes take over many tasks and help to save time – assuming that devices run reliably. Therefore it’s very important to monitor devices using current flow detection to detect if a device is not working. If a problem with a device is detected too late, this can result in costs and damage. Early detection of issues will help your customer to save time and money.

Current flow detection can be implemented in a private home as well as in commercial properties. It is often difficult to keep an eye on all devices, especially in larger buildings – monitoring them helps the company to save resources.

The Nano 2 Relay Tree can be used for many applications such as automatic blind control. If you use the Nano 2 Relay Tree for controlling blinds, the end position of the blinds will be automatically detected. For this is a power consumption of >50W is needed.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)