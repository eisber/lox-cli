# Flood Alarm

Source: https://www.loxone.com/enen/kb/flood-alarm/

---

## Brief: I need an intelligent flood alarm.

Every year, we hear reports of floods across the globe. For people living in coastal areas or near rivers, being constantly aware of the flood level is just a part of everyday life. If the worst happens and the area begins to flood, every second counts in attempting to protect possessions from water damage.

Many local authorities and organisations provide information on the current water levels and the likelihood of an imminent flood. However, this information usually has to be manually looked up. This is, obviously, less than ideal as it’s not always possible to constantly check these organisations’ websites. If the water levels change drastically at night, for example, people would typically have no way of knowing in good time.

This is why it would be helpful to have an intelligent flood alarm which automatically notifies you in the event of the water levels rising – meaning that not a second is wasted in trying to mitigate the damage.

## Solution: Using Loxone to create a flood alarm.

With the help of the Loxone Miniserver values from websites can be read. Since authorities and organisations provide very reliable information about the current situation, we take advantage of this and record the current water levels with the help of a virtual HTTP input.

**Virtual HTTP Input**

In the properties of the virtual HTTP input, the address of the web page from where the flood level will be read from is entered. In our example, we’ll be monitoring the current water levels of three rivers in Upper Austria – for this, the values are continually read from the website of the State Office for the Environment. For example, we are taking the water level for the Inn from this URL: [https://www.hnd.bayern.de/pegel/inn/passau-18009000](https://www.hnd.bayern.de/pegel/inn/passau-18009000)

**Virtual HTTP Command**

To be able to read out the water level, you have to insert a virtual HTTP command. The correct command recognition is crucial here. Before you can properly configure command recognition, you must know how the web page is structured.

![Water Level for Flood Alarm](https://www.loxone.com/dede/wp-content/uploads/sites/2/2020/04/IG_Landesamt-Bayern.png)

To do this, look at the source code of the page and find the value to be read.

Letzter Messwert vom <b>22.04.20 09:15</b> Uhr: <b>447</b> cm</span>
This tells us that the last time that the value was read was at 09:15 on 22 April, at which point the water level was at 447cm.

The text “Uhr: ” is our indication, which is placed immediately before the value we want to read. It’s important that this text “Uhr:” appears only once in the source code.

To read out the value now, enter the following command identification:

**Uhr: \s3\v**

The command recognition is structured in such a way that the first step is to jump to the prominent text passage. (Uhr:)

With \s3 three characters are skipped and \v reads the actual value. Now the current water level is displayed. It is recommended to tick the “Statistics” checkbox to display the values in the Loxone App.

**Automatic alarm in case of a critical flood level**

The “Status” Function Block is used to trigger an automatic alarm. Here you can set various conditions. For example, if the water level is above X cm. The block outputs a status value and a status text – these are then connected to the “Fire and Water Alarm” block. If the value defined in the Status block is exceeded, the alarm will be triggered. The lighting will begin to flash, the music server will play an audible alarm and a call will be made using the Loxone Caller Service.

If the website no longer provides data, this can be detected via the error output of the virtual HTTP input. If this is the case, a call is also made via [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html).

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Flood Alarm - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-107-flood-alarm_1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-107-flood-alarm_1.png)

[
![Flood Alarm - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-107-flood-alarm.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-107-flood-alarm.png)

### Download the sample file:

### Flood Warning

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-107-Flood-Warning.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-107-Flood-Warning.loxone)

## Why you and your customer should consider setting up a flood alarm?

By following the steps above you can implement an intelligent flood alarm. This will enable automatic monitoring of a website to inform you or your customers about any impending flood. This kind of flood alarm means that no time is wasted from the alert being put out and you or your customers being able to react accordingly.

The Miniserver will make sure that the warning is not missed by alerting you in multiple ways from the lights flashing to a phone call through the Loxone Caller Service.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)