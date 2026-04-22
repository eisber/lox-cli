# Notification when washing machine is done

Source: https://www.loxone.com/enen/kb/notification-when-washing-machine-is-done/

---

## Brief: Can I get a reminder when the washing machine cycle has finished?

Maybe you have an old washing machine that doesn’t ring when it’s finished. Maybe your washing machine is in the garage or in the pantry at the other end of your house and you can’t hear it when it finishes. This can lead to forgetting to hang the laundry up which, if left long enough, forces you to wash your clothes all over again. Something as simple as getting a notification when the washing machine is done a can make this chore a little easier. Loxone offers a solution that ensures that the end of the washing machine cycle will not be missed. It allows the ability to start your wash at a certain time. If you are at work, for example, it should start at 15:00 so it is finished at 18:00 when you return.

## Solution: Configuring a notification for when the washing machine is done

A Loxone Smart Socket Air allows you to monitor the actual power consumption of the device connected to it. It is, therefore, necessary to connect the washing machine to a Smart Socket which will always be on. When the machine starts its washing cycle, the smart socket will detect the current power of the machine. Once the washing machine has finished its cycle, the power will drop almost to 0 and at this time the Loxone system will send a notification when washing machine is done to either a phone and/or tablet to alert you. Be careful to put a long enough delay before sending a notification, because washing machines often pause during cycles which drops the power to almost 0. To start it at a certain time, most machines have a power loss protection. This means it will continue where it was when it has power again. So you switch off the smart socket when you load and start it, then you turn it on again by a schedule or via the app.

### Hardware:
- [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)

### Configuration:

[
![Notification when washing machine is done - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Full-Config.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Full-Config.png)

### Download the sample file:

### Smart Washing Machine

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/09/Use-Case-19-Notification-Washing-Machine.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/09/Use-Case-19-Notification-Washing-Machine.loxone)

## Why you and your customer should consider having a notification if the washing machine is ready.

Whether you are at home or outside you will never forget your clean laundry at the bottom of your washing machine. Loxone will send you a notification when the washing machine is done. This is to prevent you from having to put the washing on another cycle if it has been sitting for too long – wasting time and money. Overall, getting a Notification when the washing machine is done might seem like a minor feature, yet it makes doing chores more convenient and more energy-efficient.
					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)