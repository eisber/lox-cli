# Fish Tank Automation

Source: https://www.loxone.com/enen/kb/aquarium-fish-tank-automation/

---

## Brief: I want my fish tank to have automation and smart monitoring.

In a fish tank, the fish require a specific water temperature and a certain amount of water. The water filter must also be checked regularly. If this is not the case, the water becomes too polluted and can have negative consequences for the fish. Feeding the fish is also one of the daily tasks. The maintenance of an aquarium/fish tank must be carried out at regular intervals to provide a healthy living environment for the fish – this work takes a lot of time. With one tank it may be okay, however with several the maintenance becomes a time-consuming process. If you or your customer are not at home, a neighbour, friend or family will need to take over in your absence.

In this Use Case, we show you how you can implement fish tank automation to take care of fish and save you time.

## Solution: How to use Loxone for aquarium or fish tank automation.

With Loxone, many steps in the maintenance can be taken care of with fish tank automation.
- The water temperature is monitored and automatically adjusted.
- The water quantity is automatically monitored. If the water quantity falls below a certain level, the water pump is brought on automatically and water is refilled.
- The water filter is monitored and automatically activated.
- The lighting in the tank is switched on and off at sunrise and sunset.
- The fish are fed once a day.

All these tasks are automated, allowing better control and helping to save time. All this can be done with Loxone.

**Temperature measurement

**With fish tank automation, the temperature is constantly monitored. For this purpose, a [1-Wire Temperature Probe](https://shop.loxone.com/enuk/1-wire-temperature-probe.html) is used in conjunction with the [1-Wire Extension](https://shop.loxone.com/enuk/1-wire-extension.html). The temperature regulation is controlled by an on/off Function Block. If the water temperature falls below the predefined range, the heating element is automatically activated.

**Water level

**The [Ultrasonic Sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html) is used to measure the amount of water in the tank. If the sensor detects that the water level is too low, the ultrasonic sensor passes this information to the Miniserver. The Miniserver switches on the water pump and the tank is filled with additional water.

**Filter**

To ensure clean water, it is recommended to use a filter. This is where the [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) comes in. The filter is switched on and off using Smart Socket Air. Should the filter fail, this is detected and an alarm can be triggered immediately.

**Lighting**

With the [Lighting Controller](https://www.loxone.com/enen/kb/lighting-controller-v2/) Function Block in [Loxone Config](https://www.loxone.com/enen/products/loxone-config/), the lighting in the tank is controlled. The pulses of sunrise and sunset automatically adjust the lighting mood.

**Feeding**

The fish can be fed either manually or by means of an automatic feeder that can be controlled. For example, an impulse can be given to the feeder every day at 8 am.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [1-Wire temperature probe](https://shop.loxone.com/enuk/1-wire-temperature-probe.html)
- [1-Wire Extension](https://shop.loxone.com/enuk/1-wire-extension.html)
- [Ultrasonic Sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html)
- [RGBW LED Strip](https://shop.loxone.com/enuk/led-strip-rgbw.html)
- [RGBW 24V Compact Dimmer](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer.html)
- Water Pump
- A filter (which would be controlled with a [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html))
- Electronic feeder

### Configuration:

[
![Fish Tank Automation - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-20-Aquarium-Automation.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-20-Aquarium-Automation.png)

### Download the sample file:

### Aquarium Automation

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-20-Aquarium-Automation.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-20-Aquarium-Automation.loxone)

## Considering the advantages of aquarium or fish tank automation.

The care of pets takes a lot of time – this is especially true with fish. Fish are very sensitive regarding water temperature, light, water quantity or the quality of the water. If there is an imbalance here, this can lead to the death of the fish.

By implementing fish tank automation, your customer can save a lot of time and also monitor certain areas at all times. Especially when it comes to commercial fish farming, there is a cost factor that can be saved here.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)