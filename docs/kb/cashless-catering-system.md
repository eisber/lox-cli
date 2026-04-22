# Cashless Catering System for canteens and restaurants

Source: https://www.loxone.com/enen/kb/cashless-catering-system/

---

## Brief: I want a cashless catering system that can record the billing of food and drink.

The offer of a restaurant in the workplace where staff can conveniently get lunch, coffee or snacks is always a big plus for any employee. However, remembering to bring cash or pay with a card each day can be a bit of a burden. This is why many workplace canteens have gone cashless – cashless systems are also extremely well suited to school canteens as parents have more control over what their children are spending. However, with a cashless catering system, accurate data recording must be guaranteed in order to enable billing at the end of the month. Therefore the consumption of each product must be recorded. This process should be easy and fast because you want to avoid additional workload for the restaurant staff.

In this Use Case, we will show you how to record purchase data for each individual employee and how to use this data for billing.

## Solution: Using NFC technology for a cashless catering system in canteens and restaurants.

The foundation of a cashless catering system is an alternative way of payment – for this, we will use an [NFC Key Fob](https://shop.loxone.com/enuk/nfc-key-fob-set.html). Every employee is issued with their own NFC Key Fob, which they simply place against an [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html) in the restaurant. This is all it takes for the purchase of lunch, coffee, etc. to be recorded. To be able to distinguish what the employee has had, a number is assigned to each product. The employee presses this number on the NFC Code Touch followed by the tick button and presents their NFC Key Fob to the device. The three LED lights on the NFC Code Touch light up green as confirmation that the transaction went through. If payment is not successful, the LED lights will turn red. At the end of the month, this data recording is used and can be deducted directly from each staff members salary.

This data recording is configured through Loxone Config:

When presence is detected in the restaurant by a [Presence Sensor](https://shop.loxone.com/enuk/catalogsearch/result/?q=Presence+Sensor), the background lighting of the NFC Code Touch is activated. The Function Block “Authorisation NFC Code Touch” allows you to define which number on the device represents for which product. **For example, 1 for a cappuccino. In addition, the user groups that have the authorisation to so-called ‘purchase’ items with their NFC Key Fob are stored here.**

For automated data recording, the “Authorisation NFC Code Touch” and “Status” Function Blocks are connected together. The output Ula provides the information about the User ID, Nlo the selected product and Tla the time stamp. The “Status” block combines all this information and writes it into the log of the SD card on the Miniserver. In addition, this information is transferred to an HTTP-server via a virtual output – this HTTP-server is where the billing is done.

If a new employee starts, an additional Key Fob with authorisation can be easily paired via the Loxone App. If an employee leaves the company, the Key Fob can also be deleted via the app.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)
- [NFC Key Fob](https://shop.loxone.com/enuk/nfc-key-fob-set.html)
- [Nano 2 Relay Tree](https://shop.loxone.com/enuk/nano-2-relay-tree.html) or [Nano IO Air](https://shop.loxone.com/enuk/nano-io-air.html) (+ [Air Base Extension](https://shop.loxone.com/enuk/air-base-extension.html))
- [Presence Sensor](https://shop.loxone.com/enuk/catalogsearch/result/?q=Presence+Sensor) (optional)

### Configuration:

[
![Use Case 37 Billingsystem with NFC Code Touch](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-37-Billingsystem-with-NFC-Code-Touch.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-37-Billingsystem-with-NFC-Code-Touch.png)

### Download the sample file:

### Billingsystem with NFC Code Touch

			[Config 14.7.3.6](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-37-Billingsystem-with-NFC-Code-Touch.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/04/Use-Case-37-Billingsystem-with-NFC-Code-Touch.loxone)

## The advantages of a cashless catering system for staff in an office canteen or restaurant.

A cashless catering system simplifies the consumption of lunch, coffee, etc. in every company and organization. Restaurant staff do not need to worry about change. Companies also save on staff who would otherwise have to sit at the cash register. A Loxone cashless catering system enables simple data recording that is easy to handle and expand and provides a good overview. The recorded data can be used for accounting purposes.

The employer knows which employee has consumed what and at what time. If the employee or employer is unclear, this data can be viewed at any time.

The personal NFC Key Fob does not only have to be used for cashless payments. It can also be used to gain access to the building or to start or stop time recording. The employer equips each employee with only 1 Key Fob, which can be used for various functions.

Adding or removing NFC Key Fobs can easily be done with the Loxone App. This can be done directly by the employer. Should an employee lose their Key Fob, it can be deleted and replaced. This is not only much easier from an administrative point of view than with other systems but also far cheaper.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)