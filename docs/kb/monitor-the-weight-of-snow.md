# Monitor the weight of snow

Source: https://www.loxone.com/enen/kb/monitor-the-weight-of-snow/

---

## Brief: I need a solution to monitor the weight of snow on the roof.

Heavy snowfalls can lead to enormous snow loads, especially on flat roofs, and thus to a severe danger of collapse. Since it is usually very difficult to assess how high the actual load is, the wrong measures are often taken – or in extreme cases none at all. This costs money and in the worst case even human lives. By placing sensors on the roof you can effectively monitor the weight of snow. Then, if the weight exceeds a safe value, an alarm is immediately triggered – so the snow can be removed from the room as quickly as possible to prevent a collapse.

## Solution: Using Loxone as an intelligent way to monitor the weight of snow.

To monitor the weight of snow on the roof with the Miniserver, a 0-10V sensor is needed. The larger the area of the sensor, the more accurate the measurement. Once the sensor has been linked to Loxone Config, a minimum and maximum value must be set. In our example, the maximum value is 100 kg per square meter. If the snow load is too high, the Caller Service will inform the responsible parties so that they can remove the snow from the roof. When the maximum value is exceeded, the following message will be sent: “Snow load warning! The snow load on the roof is over 100 kg per square meter! Remove the snow on the roof to prevent damage to the building!”

**Info:** For the implementation of snow load monitoring with Loxone, we require a 0-10V sensor. If you have an interface with 4-20mA, an analogue converter is required.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- 0-10V Sensor

### Configuration:

[
![monitor the weight of snow - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-48-Snow-Load-Monitoring.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-48-Snow-Load-Monitoring.jpg)

### Download the sample file:

### Snow Load Monitoring

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-48-Snow-Load-Monitoring.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-48-Snow-Load-Monitoring.loxone)

## Why you and your customer should consider Loxone to monitor the weight of snow on a roof?

With intelligent snow load monitoring, you can monitor the weight of snow and be sent an early warning about excessive loads on the roof. This allows you to remove the snow from the roof and prevent massive damage or even collapse.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)