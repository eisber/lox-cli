# BMS Data and Statistics Logging

Source: https://www.loxone.com/enen/kb/bms-data-and-statistics-logging/

---

## Brief: I need a solution for BMS data and statistics logging.

The recording of data is an important tool for companies to effectively manage a building. Data, such as room temperature, can be used to detect changes in the building or to ensure that energy is used efficiently. If there are unexpected outliers in the values, this indicates a problem that might otherwise have gone unnoticed.

There’s a wide range of reasons why a company might need to record data. In this Use Case, we would like to show you how easy data recording with Loxone is and how you can pass on the data collected, to servers for processing for example. Giving you intelligent BMS data and statistics logging.

## Solution: Using Loxone to log BMS data and statistics

With Loxone there are numerous ways to record data and display statistics, and even to send them via e-mail for example.

The data collected can be stored on the SD card of the Miniserver or on external servers.

The statistics, e.g. on room temperature, can be displayed in a graph in the Loxone App using the [Intelligent Room Controller](https://www.loxone.com/enen/kb/irc-v2/) Function Block. Allowing the heating and cooling curves of rooms to be easily displayed.

With the [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html), any access to the building can be recorded – which, from a security perspective, is important for companies. If there’s a fire, for example, it is possible to check who’s currently in the building. To achieve this, logging can be activated in the [Authentication NFC Code Touch](https://www.loxone.com/enen/kb/authentication-nfc-code-touch/) Function Block. To record data to an external log, you can enter the address for this in the [Logger](https://www.loxone.com/enen/kb/logger/) properties from the periphery tree in Loxone Config. If the address is left empty, the standard log on the Miniserver will be used (def.log)

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)

### Configuration:

[
![Use Case 8 Stats and Data Analysis](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-8-Stats-and-Data-Analysis.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-8-Stats-and-Data-Analysis.png)

### Download the sample file:

### Statistics and Data Analysis

			[Config 14.7.3.6](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-8-Stats-and-Data-Analysis.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-8-Stats-and-Data-Analysis.loxone)

## Why you and your customer should consider Loxone for BMS data and statistics logging?

Data is important to run a business successfully and also to manage a building efficiently. With Loxone your customers will be able to record, visualize and view a wide range of data at any time.

Data can also be helpful for maintenance. Downtime can be reduced and operations can be continued without restrictions.

Recorded access control is a massive help when it comes to security. In the event of a fire, for example, your customer can immediately check who is in the building.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)