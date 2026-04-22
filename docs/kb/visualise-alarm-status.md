# Visualise an Alarm Status

Source: https://www.loxone.com/enen/kb/visualise-alarm-status/

---

## Brief: I want to be able to see the status of my burglar alarm using at a glance.

Most alarm systems visualise their current status via display, voice output or LED display. In a Loxone project, it looks quite similar.

With the Loxone App, you can easily see, for example, whether the alarm is currently activated or deactivated. And also the text-to-speech function can be used to verbalise the alarm status.

In a building equipped with Loxone, however, there are many more possibilities for alarm status visualisation: by flashing lights in the entrance area, push notification on a smartphone and much more.

In this Use Case, you will learn how the LEDs on the NFC Code Touch can be used to visualise alarm status.

## Solution: Using Loxone to visualise the alarm status on an NFC Code Touch.

The alarm in our example can have several states:
- Armed with occupants present (alarm does not use Motion Sensors)
- Armed with no occupants (alarm does use Motion Sensors)
- Armed with switch-on delay after a period of absence
- Disarmed
- Activated alarm

These states are to be displayed via the LEDs on the [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html).

To be able to visualise the status, we first need to know it. The analogue outputs (AQ and AQr) of the “Burglar Alarm” Function Block are used for this purpose. They define the current status of the alarm.

This information is used to control the status LEDs on the NFC Code Touch via its inputs (Ir, Ig, Ib).

Thus the following colour codes were defined:
- Armed with occupants present: **3 seconds yellow**
- Armed with no occupants: **3 seconds red**
- Armed with switch-on delay: **Blue flashing until the countdown is complete**
- Armed with switch-on delay and open window detected: **Flashing yellow**
- Disarmed: **3 seconds green**
- Activated alarm: **red**

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)

### Configuration:

[
![Use Case 85 Alarm Status via NFC Code Touch](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-85-Alarm-Status-via-NFC-Code-Touch.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-85-Alarm-Status-via-NFC-Code-Touch.png)

### Download the sample file:

### Alarm Status via NFC Code Touch

			[Config 14.7.3.6](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/03/Use-Case-85-Alarm-Status-via-NFC-Code-Touch-1.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/03/Use-Case-85-Alarm-Status-via-NFC-Code-Touch-1.loxone)

## Why you and your customer should consider visualising the alarm status on an NFC Code Touch?

Did I activate the alarm system? This question is probably on the minds of many people who have an alarm system fairly often.

In a building equipped with Loxone, a quick look on the app is enough to clarify the question. But sometimes this is not simply possible or practical.

The alarm status visualisation directly when leaving the house can help to prevent the question from arising in the first place.

Without additional hardware and with just a few clicks in the Config you can make sure that everything is in order.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)

n