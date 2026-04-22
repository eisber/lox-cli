# Factory Reset and Formatting the SD Card

Source: https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/

---

Resetting to factory defaults is done by formatting the removable microSD card that contains the Server’s operating system, programming and settings.
It is also possible to restore a previously created [**backup**](https://www.loxone.com/enen/kb/backup-sd-card/) or reset the password.

This procedure will also restore the Audioserver’s ability to be paired with the Miniserver.

#### CONTENTS:

[Reset Miniserver via Loxone Config](#ms_ResetSettingsConfig)

[Formatting the SD card](#format_sd)

[Resetting the password](#password_recovery)

[Manage SD card](#manage_sd)

### Basics:

**Please only use Loxone [SD Cards](https://shop.loxone.com/enuk/sd-karte.html)**

These extensively tested industrial cards guarantee error-free operation and functionality!

**SD card error / slow SD cards

**If the Miniserver [notifiied](https://www.loxone.com/enen/kb/systemstatus/) you of SD card errors, we recommend that you replace the card as soon as possible.

The SD card for the Miniserver or Audioserver is formatted on a PC or notebook using the [Loxone Config](https://www.loxone.com/enen/support/downloads/) software.

Insert the card into the card reader of the computer or use a USB card reader.

### Reset Miniserver via Loxone Config

If you are connected to the Miniserver in Loxone Config, you can reset the SD card to factory settings with a right click on the Miniserver.

In the following window you can select whether the network settings should be kept.

![ms ResetSettings](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/08/ms_ResetSettings.png)

Once the SD card has been reset, you can connect to the Miniserver again.

Attention: Default user and password is “admin”.

### Formatting the SD card

**1.** Switch off the power supply to the Server, remove the micro SD card and insert it into the computer’s reader.

Alternatively, you can use a new SD card, which you can prepare while the Server is still running with the old card.

This way you can minimize downtime when swapping out the card.

![ms sdcard](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/08/ms_sdcard.jpg)

[**SD card Audioserver**](https://www.loxone.com/enen/kb/audioserver/#Commissioning)

[**SD card Intercom**](https://www.loxone.com/enen/kb/intercom/#sdcard)

**2.** Open Loxone Config and select either the Miniserver or Audioserver’s “SD Card” in the Periphery Tree.

Then click on “Format SD Card”:

[
![format sd periphery](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/format-sd-periphery.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/format-sd-periphery.png)

The following window will open:

[
![11.1 sd format dialogue](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/11.1-sd-format-dialogue.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/11.1-sd-format-dialogue.png)

**3.** Select the drive that contains the SD card.

**4.** Use this drop-down menu to filter the listed backups for specific devices.

**5.** This is where you can select the [**backup**](https://www.loxone.com/enen/kb/backup-sd-card/) to be restored to the SD card. If you want to create an SD card with factory settings, do not select a backup here.

**6.** This is where backups that are no longer needed can be deleted from the PC.

**7.** Select the type of device (Miniserver, Miniserver Gen. 1, Audioserver) that the card should be created for.

**8.** After selecting the SD card and a (optional) backup, click on “Format” and confirm with “Yes

**9.** If the SD card already contains a Loxone file system, the following warning will be displayed:

[
![sd format](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/08/sd-format.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/11.1-sd-format-q.png)

Select “No” to create a card with factory settings or to restore a backup.

If you click on “Yes”, only the firmware of the Miniserver on the SD card will be updated.

**10.** Remove the SD card from the computer and insert it into the Server, with the power switched off. Then turn the power supply back on and the Server will boot from the new card.

Formatting with factory settings will also reset the network settings. The Miniserver will either obtain an IP address from the DHCP server or, if not available, the IP address will default to 192.168.1.77.

Then proceed with the [inital setup.](https://www.loxone.com/enen/kb/miniserver-setup/)

### Resetting the password

Loxone Config version 12 and higher allows resetting the password using the SD card of the Miniserver.

Please note that the Miniserver’s firmware on the card must also be on version 12 for this to work.

**1.** Switch off the power supply to the Server, remove the micro SD card and insert it into the computer’s reader.

**2.** Open Loxone Config and select the “SD Card” in the Periphery tree.

Then click on “Reset password”:

[
![resetpassword open](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-open.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-open.png)

					 You can also find this feature on the [start page](https://www.loxone.com/enen/kb/project-management/) or by hitting
**F5** and searching for “**password**“
**3.** A window will open, that will guide you through the reset process step by step:

[
![resetpassword intro](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-intro.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-intro.png)

**4.** Select the drive that contains the SD card.

[
![resetpassword selectsd](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-selectsd.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-selectsd.png)

**5.** Enter a new password for the user admin:

[
![resetpassword newpwd](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-newpwd.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-newpwd.png)

**6.** The new password will be saved to the SD card and the windows can be closed.

[
![resetpassword final](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-final.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/01/resetpassword-final.png)

**7.** Remove the SD card from the computer and place it into the Server.

Then turn the power supply back on and the Server will boot.

The Miniserver will send a message to all users that the password has been reset.

Now the admin user can log on to the Miniserver using the new password.

## Manage SD Card

Via “Manage SD Card” old files like statistics, log files etc. that are no longer needed can be deleted from the SD card.

Generally, this is not necessary. Always create a [**backup**](https://www.loxone.com/enen/kb/backup-sd-card/) before deleting files using this option.

Select the **SD Card** in the Periphery Tree and then click on **Manage SD Card**.

![sd_karte_verwalten_01](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/sd_Karte_verwalten_01.png)

A window opens listing all files that can be removed.

Select the files you want to delete and click on Delete.

![sd_karte_verwalten_02](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/07/sd_Karte_verwalten_02.png)