# Motion sensor water valve for pet drinking water

Source: https://www.loxone.com/enen/kb/motion-sensor-water-valve/

---

## Brief: Configure a motion sensor water valve to release water for pets.

One of the problems with leaving your dog home alone for long periods of time is ensuring they always have access to fresh water – especially in the summer months. So we need to configure a motion sensor water valve that will release a suitable amount of water with various considerations to prevent overfilling.

When the dog approaches its drinking bowl, using a motion sensor water valve, fresh water is automatically refilled. So the dog has always fresh water available and you don’t have to worry about your loved ones anymore.

So, if you are looking for a solution regarding a smart water supply for your pet, then read on to see how we would achieve this using Loxone Hardware and Loxone Config software.

## Solution: How to supply fresh drinking water with a motion sensor water valve.

With Loxone it is possible to set up a motion sensor water valve based on a movement for a defined time.

In this example, a Motion Sensor is used, which generates an impulse with movement. The customer can set the time for the refill himself via the Up/Down button in the app. If constant movement is detected because the dog is permanently in front of the drinking bowl, water is only refilled for the set time by the customer. This is to prevent water from running all the time if the dog is constantly there.

If no movement is detected for more than 5 minutes, the refill can run again if there is motion detected. For this purpose, a tracker was created to visualise the watering.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)

### Configuration:

[
![Motion sensor water valve - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-40-Smart-Water-Supply.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-40-Smart-Water-Supply.png)

### Download the sample file:

### Smart Water Supply

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-40-Smart-Water-Supply.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-40-Smart-Water-Supply.loxone)

## Why you and your customer should consider a motion sensor water valve setup for pet drinking water?

If a dog does not have enough fresh water available, this has negative health effects therefore they must have a continuous supply of fresh water. In the summer month, dogs need an even greater amount of water per day. After drinking, the body temperature drops which has a positive effect on the dog’s circulation. That is why it is so important that your dog is always supplied with fresh water, especially in summer. The smart automatic water supply ensures that your dog is constantly supplied with fresh water as soon as he approaches the bowl. Furthermore, it is also guaranteed that if the dog stays in front of the drinking bowl, it will not be constantly refilled.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)