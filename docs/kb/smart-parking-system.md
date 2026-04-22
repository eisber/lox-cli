# Smart Parking System

Source: https://www.loxone.com/enen/kb/smart-parking-system/

---

## Brief: I want to have a smart parking system that monitors occupancy and controls access.

In the car parking sector, it is beneficial if customers can easily be made aware of parking spaces that are available. In this use case, we’ll show you how to set up a smart parking system which automatically indicated free parking spaces with green LEDs and occupied ones with red LEDs. We will also show you how to automate the opening and closing of the car park doors based on the available parking spaces. As well as how to regulate the air quality in garages for the health of the users.

**In this Use Case, we will assume a garage with 8 parking spaces with two entrance and two exit gates.**

## Solution: Using Loxone to configure a smart parking system.

Each of the eight parking spaces has LED lighting, which indicates (using either red or green) whether the parking space is still free or already occupied. If parking spaces are still available, this is also indicated at the entrance gates by green LEDs. If all parking spaces are occupied, the access gates close and this is also indicated by the red LEDs at the access gates.

As the garage is only designed for eight vehicles, the air quality can quickly reach a critical level. For this reason, the ventilation system is controlled according to the air quality. If a critical value is exceeded, the doors are closed and a push message is sent.

The garage control system ensures that the exit gates are opened automatically as soon as a vehicle approaches the sensor so that leaving the car park is seamless. All of this functionality results in a truly smart parking system.

To detect how many parking spaces are occupied, a seven-segment display visually indicates the number of occupied parking spaces.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- 2 x [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html)
- 0-10V Air Quality Sensor
- 10 x Digital Parking Sensors
- Seven Segment Display

### Configuration:

[
![Use Case 24 Parking Garage](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-24-Parking-Garage.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-24-Parking-Garage.png)

### Download the sample file:

### Parking Garage

			[Config 14.7.3.6](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-24-Parking-Garage.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-24-Parking-Garage.loxone)

## The advantages of a smart parking system automated with Loxone.

With very little hardware you can implement a smart parking system for all of your customers. In addition, your customers receive comprehensive visualization of the smart parking system.

Thanks to the Loxone Miniserver, your customers will always know the status of their car park. They can also instantly be informed about any issues.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)