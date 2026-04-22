# Locker Management System with Electric Locks

Source: https://www.loxone.com/enen/kb/locker-management-system-with-electric-locks/

---

## Brief: I need a solution to manage lockers with electric locks.

Electronic lockers are often expensive and complicated to manage. The solutions offered are mostly isolated solutions. This means that a locker can only be opened with its respective key – making any loss or change in authorisation costly.

It would be great for any company or club to have a system that takes care of access control (via doors, side gates, etc.) but can also provide an intelligent locker management system. When new a new employee or member joins, it should be as easy as possible to assign them with a locker.

In this Use Case, we show you how you can turn all these requirements into reality with the NFC Code Touch.

## Solution: Using Loxone as a locker management system with electric locks.

We show you how to implement an intelligent locker management system using Near Field Communication (NFC) technology and the Loxone Miniserver. For this, we’ll use an [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html), which is available in Tree (wired) or Air (wireless) versions.

To use the NFC technology, the NFC Code Touch must be supplied with a 24V power supply. If the NFC is battery powered, only code entry is possible.

[NFC Key Fobs](https://shop.loxone.com/enuk/nfc-key-fob-set.html) or NFC Smart Tags act as keys. The Key Fob is a practical fob which can easily be attached to a set of keys and the Smart Tags can be stuck to another item such as a wallet or smartphone – so you always have the key with you.

By pressing the Key Fob or Smart Tag against the NFC Code Touch, the respective locker is opened. An electronic lock is required for each locker which also means one digital output is required per locker – the [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html) would be perfect in this scenario.

In Loxone Config, a user is created for each person and an NFC Key Fob or a Smart Tag is assigned. Additionally, each user is provided with a user ID. This ID is an output on the “Authentication NFC Code Touch” Function Block and transferred to the “Pulse At” Function Block. This now activates the corresponding digital output.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)
- [NFC Key Fob set](https://shop.loxone.com/enuk/nfc-key-fob-set.html)
- [Air Base Extension](https://shop.loxone.com/enuk/air-base-extension.html)
- [Tree Extension](https://shop.loxone.com/enuk/tree-extension.html)
- [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html)

### Configuration:

[
![Use Case 39 Locker management with NFC Code Touch](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-39-Locker-management-with-NFC-Code-Touch.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-39-Locker-management-with-NFC-Code-Touch.png)

### Download the sample file:

### Locker Management with NFC Code Touch

			[Config 14.7.3.6](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-39-Locker-management-with-NFC-Code-Touch.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-39-Locker-management-with-NFC-Code-Touch.loxone)

## Why you and your customer should consider Loxone as a locker management system?

A locker management system with authorization via NFC tags is a simple and expandable solution. Creating, deleting or changing users can be done directly in the Loxone App. Your customer can also pair and assign new NFC tags directly in the Loxone App. There is no need to change the configuration. In addition, the Key Fob or the Smart Tag can also be used for access to the building or other areas.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)