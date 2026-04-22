# Pairing NFC Tags to the NFC code touch

Source: https://www.loxone.com/enen/kb/pairing-nfc-tags-to-the-nfc-code-touch/

---

# Pairing NFC tags/stickers with the NFC code touch

In order to use the [NFC tags](https://shop.loxone.com/enuk/nfc-key-fob-set.html) or [NFC stickers](https://shop.loxone.com/enuk/encrypted-nfc-smart-tags.html), you must first pair them with the NFC code touch. There are some options to consider when pairing the NFC tags/stickers based on what you want to use them for.

Firstly, you need to ensure the NFC code touch has been paired into the system and is working. Instructions on how to do this can be found [here](https://www.loxone.com/enen/kb/nfc-code-touch/).

Once the device is paired to the system, you can begin pairing NFC devices to the NFC code touch. To do this, you need to first enable NFC learn mode by pressing the “Learn NFC tag” button in the top ribbon.

![Screenshot 1 800x776](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/08/Screenshot_1.png)

Once that button has been pressed, it will open the NFC monitor at the bottom of the screen and the NFC code touch will start flashing blue. If you now present the NFC device to the NFC code touch’s NFC reader and it will appear in the monitor below highlight yellow until selected.

![Screenshot 2 800x682](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/08/Screenshot_2.png)

Once you have found your tag in the bottom learn monitor, you need to provide it with a description and then hit on Add by clicking the drop down arrow and selecting an option from there. How you add it here determines a few behavioural features of the NFC device and how it can be used within Loxone Config.

![Screenshot 2 800x682](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/08/Screenshot_2.png)

![Screenshot 3 800x231](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/08/Screenshot_3.png)

Under the add drop down, you can see the 3 options.

### Assign to User

Assigning a tag to a user associates that tag with the user. In order to restrict access, you need to use the “User groups” settings within the NFC code touch block itself. Details on the “User groups” setting can be found on our page about the [NFC authorisation block](https://www.loxone.com/enen/kb/authentication-nfc-code-touch/). One key element here is that if this user is part of a user group that has access to multiple NFC code touches, they will be able to access all NFC code touches that has their group enabled. Perfect for cleaners. You would use this option if you have 1 or more NFC code touches and each person that will use it will have their own tag. If you need to restrict tags based on time of day, this option suits that too (I.E cleaners).

### User for NFC code touch

In the above image, Use for Front Door is the option we are referring to here. This will link the NFC tag/sticker to this specific NFC code touch. This prevents the tag from being usable on more than 1 NFC code touch. This option would be used if you wanted a traditional key system with multiple NFC code touches.

### Use as a digital input

This option creates the NFC tag separate from the NFC code touch. This turns the tag into a digital input that will trigger when the tag is detected. This is perfect for things like locker systems – where you give everyone their own tag and want it to only unlock something specific, I.E some lockers.