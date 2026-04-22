# Central Security Alarm

Source: https://www.loxone.com/enen/kb/central-security-alarm/

---

## Brief: I want a central security alarm.

It’s quite common for business owners and landlords to have large commercial premises that they sublet to other companies. Offices or complexes (such as shopping centres) usually have communal areas like foyers, staircases, etc.

When this is the case, it can be difficult to set a burglar alarm for the whole building. The last person to leave can’t easily arm the alarm as they’d need to check all off the other offices, shops, etc. to ensure that they were actually the last person. For this reason, it would be far more beneficial if there was an automated central security alarm, which detected when the last person leaves and automatically sets the alarm. Similarly, the central security alarm should be automatically deactivated as soon as the first person enters the building the next morning.

## Solution: Using Loxone to set up a central security alarm.

In our example, three separated offices are all located within one building. Loxone is already being used in each office for lighting, heating and security – with each office having its own alarm system. A triple tap on a [Touch Switch](https://shop.loxone.com/enuk/loxone-touch.html) (located next to the office entrance) activates their individual alarm, switches off the lighting and lowers the heating. When this is done, it lets the central security alarm know that no one is in that office anymore. Once all three office alarms have been set the [“Status”](https://www.loxone.com/enen/kb/status/) Function Block knows that the building should now be empty and it passes this information on to the [“Burglar Alarm”](https://www.loxone.com/enen/kb/burglar-alarm/) Function Block – which will then activate the central security alarm.

The following morning when the first person arrives back at the office, the alarm will be automatically deactivated. This is done using an [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html) which is placed next to the main entrance to the office. When the person presents their [NFC Key Fob](https://shop.loxone.com/enuk/nfc-key-fob-set.html) the central security alarm will be deactivated. Additionally, as the NFC Key Fob also stores information on what individual office that person is from, their separate office alarm will also be deactivated.

**Highlight: **The owner/ landlord of the building’s fob will act as somewhat of a master key. If they present their NFC Key Fob, the central security alarm and all three separate office alarms will all be deactivated. Allowing them to move around the whole complex without triggering an alarm.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)
- [NFC Key Fob](https://shop.loxone.com/enuk/nfc-key-fob-set.html)
- [Touch Switch ](https://shop.loxone.com/enuk/loxone-touch.html)

### Configuration:

[
![Central Security Alarm - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-93-Communal-Alarm-01.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-93-Communal-Alarm-01.png)

[
![Central Security Alarm - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-93-Communal-Alarm-02.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-93-Communal-Alarm-02.png)

[
![Central Security Alarm - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-93-Communal-Alarm-03.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-93-Communal-Alarm-03.png)

### Download the sample file:

### Communal Alarm

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-93-Config-Communal-Alarm.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-93-Config-Communal-Alarm.loxone)

## Why you and your customer should consider implementing a central security alarm?

By implementing a central security alarm, you make everyone’s lives much easier. Employees no longer have to worry about whether they are the last person to leave the building and landlords no longer have the stress of worrying that no one has set the alarm.

Since a user ID is stored within each NFC Key Fob, the alarm system in the communal areas and in the respective office is automatically deactivated when an employee enters the building.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)