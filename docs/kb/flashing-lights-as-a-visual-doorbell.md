# Visual doorbell for hearing impaired

Source: https://www.loxone.com/enen/kb/flashing-lights-as-a-visual-doorbell/

---

## Brief: I want to have flashing lights as a visual doorbell.

The traditional audible doorbell is great in many scenarios however, a visual doorbell where the lights flash to let you know that someone is at the door can also be very useful. A visual doorbell is great for anyone who is hard of hearing or has a impairment – it could also be used in conjunction with an audible doorbell to give maximal notification.

Visual doorbells are also great in household with young children. All parents know the feeling: the children have finally fallen asleep at the end of a long and exhausting day. Now there is still some time left to relax and maybe have a glass of wine with your partner. When suddenly the doorbell rings. The children who have just fallen asleep are wide awake again. Therefore it would be extremely helpful to deactivate the audible doorbell at night and replace it with a visual one so that the children can sleep in peace.

## Solution: Using Loxone to intelligently commission a visual doorbell for the hearing impaired.

The bell button, on the [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html) or the [Intercom](https://shop.loxone.com/enuk/intercom.html), triggers a ringing event on the Audioserver Zone. This means that an traditional doorbell sound can be heard in all rooms. This is great to ensure that you can hear the doorbell wherever you are in the home however it could very easily wake up any sleeping children. This can be easily prevented with the help of the [Schedule](https://www.loxone.com/enen/kb/timerscheduler/) Function Block.

A time in which the audible bell signal is allowed to ring is stored in the block. Outside of this time, the traditional bell sound will not be played.

In order to configure a visual doorbell which is done by flashing the lights, the bell button signal is transferred to a [Retractive Switch](https://www.loxone.com/enen/kb/retractive-switch/) Function Block. This component ensures that the output pulse is limited to 2 seconds. This two-second pulse is now connected to input (Alarm) of the [Lighting Controller](https://www.loxone.com/enen/kb/lighting-controller-v2/). Input (Alarm) is an alarm input that makes all the lighting in the room flash. As the pulse has been limited to two seconds, the lighting only flashes briefly and your customers will know that someone is at the door.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Audioserver](https://shop.loxone.com/enen/audioserver.html)
- [Loxone Intercom](https://shop.loxone.com/enuk/intercom.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)

### Configuration:

[
![Use Case 83 Silent Doorbell](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-83-Silent-Doorbell.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-83-Silent-Doorbell.png)

### Download the sample file:

### Silent Doorbell

			[Config 14.7.3.6](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/03/Use-Case-83-Silent-Doorbell.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/03/Use-Case-83-Silent-Doorbell.loxone)

## Why you and your customer should consider a visual alert doorbell?

With Loxone Config you can tailor an automated building exactly to the needs of your customers. This Use Case shows how the living comfort of your customers can be increased with a few simple steps. Whether that’s for a customer who is hard of hearing or customers who simply do not want their children being woken when they’re sleeping.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)

n