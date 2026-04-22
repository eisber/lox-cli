# Fall Detection for Elderly (AAL)

Source: https://www.loxone.com/enen/kb/fall-detection-for-elderly/

---

## Brief: I want a passive monitoring solution with fall detection for elderly relatives.

Fall detection for elderly friends or family members is one of the functions within Ambient Assisted Living (AAL). It provides a higher level of safety for residents and makes caregivers and family members feel more secure.

With Loxone, a potential fall can be detected by monitoring a pre-defined period of time per room – for example, if 15 minutes after motion was detected in one of the passage rooms (a hallway for example) but no motion has been detected in one of the attaching rooms. In this situation, it can be assumed that the occupant has fallen and, in the worst case, injured themself as it is unlikely that anyone would spend 15 minutes in a hallway.

If you want to implement fall detection for elderly and/or impaired resident, read on to find out how this can be achieved with Loxone.

## Solution: Using Loxone for assisted living to detect a potential fall.

To detect a potential fall, we use a predefined time-span per room type to identify the fall. In our example, we configure fall detection on a staircase, as this is one of the riskiest areas. A staircase, a corridor, and a toilet are rooms where we assume a maximum time span of 15 minutes.

In our example, we assume that the occupant needs to access the bedroom or bathroom via the stairs. Once the [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html) has picked up movement on the staircase. Logically, movement should next be detected in the bedroom or bathroom. However, if this has not happened after 15 minutes, the resident may have fallen in the stairwell and an alarm is triggered. The Miniserver sends a signal and the carer/relevant family member is informed immediately.

More than one person can be notified in an emergency. In addition, the [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html) can be integrated into the system, whereby the caregiver receives a telephone call in addition to the push notifications. Here too, several telephone numbers can be linked.

**Tip:** Besides the fall detection the Loxone system can also help to prevent falls. Automated lighting will come on based on presence meaning that the resident will never need to go over to a switch plus they’ll never be left in the dark.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Inputs for Fall Detection for Elderly - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-73-Fall-Detection-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-73-Fall-Detection-1.png)

[
![Fall Detection for Elderly Smart Alarm - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-73-Fall-Detection-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-73-Fall-Detection-2.png)

[
![Fall Detection for Elderly Alarm Sequence - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-73-Fall-Detection-3.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-73-Fall-Detection-3.png)

### Download the sample file:

### Fall Detection for Assisted Living

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-73-Fall-Detection-for-Assisted-Living.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-73-Fall-Detection-for-Assisted-Living.loxone)

## The advantages of passive monitoring with potential fall detection for elderly relatives?

According to the [NHS](https://www.nhs.uk/conditions/falls/), “falls are a common, but often overlooked, cause of injury. Around 1 in 3 adults over 65 who live at home will have at least one fall a year, and about half of these will have more frequent falls.”

Both objects and the circumstances in an elderly person’s (living) environment can lead to risky situations (e.g. inadequate lighting, loose carpets, slippery floors, missing railings or handles, etc.). These all increase the risk of falls, especially for older people.

The aim of Ambient Assisted Living is to improve the quality of life of elderly people through the use of automation and to enable them to live independently for as long as possible, even if they have physical and/or mental disabilities.

Fall detection for elderly friends or family members is only a part of the possibilities that the Loxone system offers for assisted living. Thanks to the extensive automation, passive control can be implemented, potential risks can be identified, visual and/or acoustic alarms can be provided and lifestyle monitoring can be integrated. In addition, everything happens automatically, so that the residents are relieved of many everyday tasks.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)