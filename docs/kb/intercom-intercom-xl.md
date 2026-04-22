# Intercom Gen. 1

Source: https://www.loxone.com/enen/kb/intercom-intercom-xl/

---

This page contains the documentation for the Intercom Gen. 1 and Intercom XL.
Here you will find instructions on commissioning and additional information.

					Go to the [documentation for the new Intercom](https://www.loxone.com/enen/kb/intercom-2/).
[-> Skip Quick Start Guide](#Inhalt)

[Step 1: Mounting and connection](#Schritt1)

[Step 2: Intercom Search](#Schritt2)

[Step 3: Set up Intercom Video](#Schritt3)

[Step 4: Create module configuration as network device](#Schritt4)

[Step 5: Set up Intercom Audio](#Schritt5)

[Step 6: Add module configuration to existing network device](#Schritt6)

[Step 7: Change login credentials](#Schritt7)

[Optional – Adding the Door Controller](#Schritt8)

					Since October 2020, the Intercom is delivered with a new version of the audio module. The web interface for the configuration of the unit is visually different.
The [SIP settings](#sipnewaudio) from this documentation can be followed in a similar way.

Due to the new internal electronics, the product comes with an RJ45 connector instead of an RJ45 socket:

[
![intercom10.2020back](https://www.loxone.com/dede/wp-content/uploads/sites/2/2020/09/intercom10.2020back.jpg)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2020/09/intercom10.2020back.jpg)

The Intercom XL will not have this change and the product itself is being phased out.

## Step 1: Mounting and connection

It is recommended that you mount the intercom with the top edge being 170cm above the ground (67 inches)
- Connect an RJ45 ethernet cable from to your home network to the “Data/LAN In” port of the supplied PoE Injector (PoE = Power over Ethernet).
Connect an RJ45 ethernet cable from the “Power+Data Out” port of the PoE Injector to the Loxone Intercom.
Provide power to the POE Injector.
- The recommended maximum length for an ethernet connection is ca. 100m. It is recommended to place the PoE adapter in close proximity to the intercom.
- For ethernet connections of over 100 meters, a different connection must be used, e.g. fibre optic.
- A data rate of 2 MBit minimum is required for adequate audio communication.
- A data rate of 10 MBit is required for adequate video connections.

## Step 2: Intercom Search

After the Intercom has been connected to the network and powered on, as described in [Step 1](#Schritt1), it can then be configured and set up in [Loxone Config](https://www.loxone.com/dede/produkte/loxone-config/).
- Open the current document in Loxone Config and [connect](https://www.loxone.com/dede/kb/miniserver-erstkonfiguration/) to the Miniserver.
- Click on “**Monitoring and Access**” in periphery.
- Then select “**Device Search**” from the top ribbon:

[
![start search intercom](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/start-search-intercom.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/start-search-intercom.png)

This will open the Network Device Search:

[
![search intercom network](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/search-intercom-network.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/search-intercom-network.png)
- The search is started automatically, all devices detected by the Miniserver will be displayed. It can take a few minutes until all devices are found.
- The audio module (SIP) and video module (CAM) are listed as two separate entries.
- The modules are configured in two separate steps. (video in [Step 3](#Schritt3) and audio in [Step 5](#Schritt5))

## Step 3: Set up Intercom Video

If the Intercom video module was found as described in [Step 2](#Step2), it can now be configured and set up.

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
- Select the entry for Intercom Video.
- Click the “Configure Device” button.

[
![start config intercom video](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/start-config-intercom-video.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/start-config-intercom-video.png)

					If the Intercom Video Module could not be found, [you can find more information here](#DiagnoseNetzwerk).

					This will open the configuration dialog for the video module.
- Assign a static IP address to the module.
- Check and, if necessary, adjust the network settings.
- Then click on “Next”

![Netzwerk Video](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Netzwerk-Video.png)

					The selected network settings must match the existing network (network mask, default gateway, etc.).
The selected IP address must not be in use yet.

In the following dialog, enter the login details for the video module.
- Enter user name (default user name: “admin”)
- Enter password (default password: “admin”)
- We recommend adjusting the default login credentials after the initial configuration is complete. [Information on how to change the password can be found here.](#VideoPasswort)
- (Optionally, the external port for the video module can already be configured here. More information about setting up an external video connection can be found[here](#VideoExtern))
- Confirm the configuration by clicking the “Finish” button.

![Zugangsdaten Video](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Zugangsdaten-Video.png)

					The video module applies the configuration and restarts.
[Proceed with Step 4.](#Schritt4)

## Step 4: Create module configuration as network device

Once the configuration of the Intercom module is completed, it needs to be created as a new network device.
- Enter the installation location.
- Assign to a room.
- Confirm with “Apply”.

![Gerät erstellen](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Gerät-erstellen.png)

					If the dialog shown does not appear, it is possible that a module already exists as a network device.
[In this case, please follow the procedure described in Step 6.](#Schritt6)

					[Proceed with Step 5.](#Schritt5)

## Step 5: Set up Intercom Audio

If the Intercom audio module was found as described in [ Step 2](#Schritt2), it can now be configured and set up.

					**>If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
- Select the entry for Intercom Audio.
- Click the “Configure Device” button.

[
![start config intercom audio](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/start-config-intercom-audio.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/start-config-intercom-audio.png)

					If the Intercom audio module could not be found, [you can find more information here](#DiagnoseNetzwerk).

					This will open the configuration dialog for the audio module.
- It is recommended to assign a static IP address to the module.
- Check and, if necessary, adjust the network settings.
- Then click on “Next”

![Netzwerk Audio](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Netzwerk-Audio.png)
- The selected network settings must match the existing network (network mask, default gateway, etc.).
- The selected IP address must not be in use yet.

In the following dialog, enter the login details for the audio module.
- Username (internal) is not required.
- Enter password (default password: “admin”)
- We recommend adjusting the default login credentials after the initial configuration is complete. Information on how to change the password can be found [here](#AudioPasswort).
- (Optionally, the credentials for the external audio connection can already be entered here. More information about setting up an external audio connection can be found [here](#AudioExtern) genauere Infos.)
- Confirm the configuration by clicking the “Finish” button.

![Zugangsdaten Audio](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Zugangsdaten-Audio.png)

					The audio module applies the configuration and restarts.
[Continue with step 6.](#Schritt6)

## Step 6: Add module configuration to existing network device

			Once the configuration of an Intercom module is completed, that configuration can also be applied to another network device.
- Select the desired Intercom.
- Confirm with “OK”

![Gerät zusammenfügen](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Gerät-zusammenfügen.png)

					If no network device is available, the below dialog may not appear.

[If this is the case, the further procedure is described in step 4.](#Schritt4)

					The configuration is now automatically applied to the existing Intercom.

![Intercom Config Audio](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Config-Audio.png)

[Continue with step 7.](#Schritt7)

## Step 7: Change login credentials

			For security reasons, change the default login credentials for the video and audio module.
- [Changing the login credentials for Intercom Video.](#VideoPasswort)
- [Changing the password for Intercom Audio.](#AudioPasswort)

					**Even if the Intercom is not accessible remotely, the password should be changed!**
**There is NO security when using default login credentials!**

					The basic configuration of the Intercom is now complete.

The remaining configuration is done automatically by [Auto-Configuration](https://www.loxone.com/dede/kb/auto-konfiguration/).

## Optional – Adding the Door Controller

			If Auto-Configuration is not used, this step can be done manually.
- The Door Controller is automatically added by dragging the Intercom onto the desired programmig page.
- The assigned Intercom is automatically selected.
- Save to the Miniserver.

![Intercom Türsteuerung einfügen 1024x359](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Türsteuerung-einfügen.png)

					[Detailed information on how to configure the call buttons of the Intercom XL can be found here.
](#Ruftasten)

# Content
- [Quick Start Guide](#Quick-Start-Guide)
- [Content](#Inhalt)
- [Intercom Video](#VideoInhalt)
- [Intercom Audio ](#AudioInhalt)
- [Intercom XL](#IntercomXL)
- [Diagnostics](#Diagnose)
- [Technical Data](#Daten)

# Intercom Video

### Inhalt
- [Web Interface](#Video)
- [Network Settings](#VideoNetzwerk)
- [Network Settings via Loxone Config](#VideoNetzwerkConfig)
- [Change password and user](#VideoPasswort)
- [Set up external video](#VideoExtern)
- [Update](#VideoUpdate)
- [Restore Factory Settings](#VideoWerkseinstellungen)

## Web Interface

			The web interface is used for detailed configuration of the Intercom Video Module.
- For this, enter the IP address of the module into your browser.
- Then enter the username and password of the module (default username: admin, default password: admin).
- The start page of the audio module shows the [network settings](#VideoNetzwerk).

![Intercom Video Anmelden 1024x630](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Video-Anmelden.png)

## Network Settings

			To change the network settings, access the web interface of the video module ([detailed information can be found here](#Video)).

The network settings can also be sent via Loxone Config to the module ([detailed information can be found here](#VideoNetzwerkConfig)).

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
- We recommend assigning a static IP address.
- Please note that the network settings must match your home network (subnet mask, default gateway, etc.).
- The IP address must not yet be assigned.
- Click on “Send” to apply the settings.

					The module restarts automatically after saving.

![Intercom Video Startseite](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Video-Startseite.png)

## Network Settings via Loxone Config

			The network settings can be sent to the video module via Loxone Config.
For this the MAC address of the video module is required. It can be found on the back of the Intercom.

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
					If the audio module does not restart, the connection to the device could not be established or the entered MAC is wrong.
- Open Loxone Config
- Select “Network Periphery”.
- Select “Configure Device” from the menu.

![Loxone Config Gerät konfigurieren](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Loxone-Config-Gerät-konfigurieren.png)
- Select Intercom Video
- Enter the MAC address of the video module. You can find it on the back of the Intercom.
- Enter the network settings.
- We recommend assigning a static IP address.
- Please note that the network settings must match your home network (subnet mask, default gateway, etc.).
- The IP address must not yet be assigned.
- Click on “Send” to apply the settings.

					The video module will now restart.

					If the video module does not restart, the connection to the device could not be established or the entered MAC is wrong.

![Loxone Config Netzwerk Video](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Loxone-Config-Netzwerk-Video.png)

## Change Password

			To change the password, access the web interface of the video module ([detailed information can be found here](#Video)).
- Select “Advanced”.
- Open “User Management”.

![Intercom Video Benutzerverwaltung 936x1024](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Video-Benutzerverwaltung.png)

Here you can change the current password or create new users.
- Select “Change” user.
- Tick the “Set new password” checkbox.
- Enter your new password.
- Click “Save” to accept the new password.

					The module restarts automatically after saving.

We recommend creating a new Intercom user for video display in the app. This user should only have “Viewer” rights.

![Intercom Video Passwort ändern 1024x808](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Video-Passwort-ändern.png)

## Set up external video

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
- For external video connection, port forwarding must be set up on the router.
- This is an additional port forwarding similar to the one used for the Miniserver. ([More information can be found here](https://www.loxone.com/dede/kb/online-freigabe/))
- For security reasons, always use a random port greater than 5000. 8090 is used in the example.
- Enter the external port in Loxone Config.
- If the Host for Video Stream External is specified with “clouddns:[Port]”, the keyword “clouddns” represents the current external IP address of the connection.

![Intercom Video Config extern](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Video-Config-extern.png)

## Update

[
![download](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/01/download.png)
Download latest firmware](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/08/CP_CAM_Update_Loxone_V51_RC.zip)
- Extract the zip file after downloading.
- Run the “*_CAM_Update.exe” contained in the folder.
- Select the IP address of the video module. If the IP address is not listed, you can select the blank option in the drop-down menu to manually enter the correct IP address.
- The “Start” button transfers the update to the module.
- The default port 4000 does not need to be changed.
- By default, the update file from the extracted folder is used. If necessary, this can be changed via Select File.

					During the update, the user settings are reset to factory settings. The user data must be reconfigured after the update.
[Find more detailed information about customizing the user data here](#VideoPasswort).

![Intercom_Cam_updater](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/08/Intercom_Cam_updater.png)

## Restore Factory Settings

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**

					To reset the video module to factory settings, establish a direct network connection between your PC and the Intercom (not Wifi or via router).
- Power off Loxone Intercom
- Set both DIP switches on the camera module to OFF (normal state: DIP1 = ON, DIP2 = OFF).

![DIPswitch](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/05/DIPswitch.png)
- Restore the power supply to the intercom.
- The IP address of the video module is now set to 10.10.10.10.
- Now adjust the IPv4 settings of your PC.
- IP Address: 10.10.10.50
- Subnet Mask: 255.0.0.0
- Default Gateway: nicht angegeben

![IP](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/05/IP1.png)
- Connect to the web interface of the video module (Address: 10.10.10.10). [Detailed information can be found here](#Video).
- Now configure new network settings for the video module. [Detailed information can be found here](#VideoNetzwerk).
- Please note that the network settings must match your home network (subnet mask, default gateway, etc.).
- The entered IP address must not yet be assigned.
- Set the DIP switches back to their normal position. (DIP1 = ON, DIP2 = OFF)
- Briefly disconnect the Intercom from the power supply.

					After booting successfully, the Loxone Intercom is reachable at the new IP address.

# Intercom Audio

### Inhalt
- [Web Interface](#Audio)
- [Network Settings](#AudioNetzwerk)
- [Network Settings via Loxone Config](#AudioNetzwerkConfig)
- [Change password](#AudioPasswort)
- [Set up external audio](#AudioExtern)
- [Update](#AudioUpdate)
- [Restore Factory Settings](#AudioWerkseinstellungen)

## Web Interface

			The web interface is used for detailed configuration of the Intercom Video Audio Module.
- For this, enter the IP address of the module into your browser.
- Then enter the username and password of the module (default username: admin, default password: admin).
- The start page of the audio module shows the [network settings](#AudioNetzwerk).

![Intercom Audio Anmelden 1024x705](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Audio-Anmelden.png)

## Network Settings

To change the network settings, access the web interface of the audio module ([detailed information can be found here](#Audio)).

The network settings can also be sent via Loxone Config to the module ([detailed information can be found here](#AudioNetzwerkConfig)).

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
- We recommend assigning a static IP address.
- Please note that the network settings must match your home network (subnet mask, default gateway, etc.).
- The IP address must not yet be assigned.
- Click on “Save” to apply the settings.

The module restarts automatically after saving.

![Startseite Intercom Audio](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Startseite-Intercom-Audio.png)

### Network settings with new audio module from 10/2020:

Connect to the web interface of the audio module via browser (configured IP, or **default** IP **192.168.1.98**, default login: **admin/admin**)

Passwords up to 10 characters in length are supported.

First click on **Main (1)**, then on **Main Settings (2).**

Then configure the **IP address** **(3) **and the **network settings **:

[
![IP Setup 1](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/IP-Setup-1.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/IP-Setup-1.png)
**Finally click on “SAVE” to save the settings!**

## Network Settings via Loxone Config

			The network settings can be sent to the audio module via Loxone Config.
For this the MAC address of the audio module is required. It can be found on the back of the Intercom.

					**If you are unfamiliar with network technology, it may be advisable to consult a network specialist to successfully complete this step and continue with the initial configuration.**
					If the audio module does not restart, the connection to the device could not be established or the entered MAC is wrong.
- Open Loxone Config
- Select “Network Periphery”.
- Select “Configure Device” from the menu.

![Loxone Config Gerät konfigurieren](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Loxone-Config-Gerät-konfigurieren.png)
- Select Intercom Audio
- Enter the MAC address of the audio module. You can find it on the back of the Intercom.
- Enter the network settings.
- We recommend assigning a static IP address.
- Please note that the network settings must match your home network (subnet mask, default gateway, etc.).
- The IP address must not yet be assigned.
- Click on “Send” to apply the settings.

					The audio module will now restart.

					If the audio module does not restart, the connection to the device could not be established or the entered MAC is wrong.

![Loxone Config Netzwerk Audio](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Loxone-Config-Netzwerk-Audio.png)

## Change Password

			To change the password, access the web interface of the audio module ([detailed information can be found here](#Audio)).
- Select “Advanced”.
- Open “VoIP Settings”

![Intercom Einstellungen VoIP](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Einstellungen-VoIP.png)
- Select the “System” tab.
- Enter the new password.
- Click “Save” to accept the new password.

					The module restarts automatically after saving.

![Intercom Audio Passwort ändern](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Audio-Passwort-ändern.png)

## Set up external audio

To set up the external audio connection, an **external SIP provider** like Antisip.com is required.

					We cannot offer any support, warranty or liability for the external services
- A SIP account can be compared to a telephone number, so a separate account is also required for each Intercom.
- An account may not be used more than once!

					Create a new SIP account and make a note of the login details.

					Do not set up **port forwarding** for the SIP connection!
To link the created SIP account with the Intercom, open the web interface of the audio module ([detailed information can be found here](#Audio)).

First, select the “Advanced” button, then open “VoIP Settings”.
- Select the “SIP” tab
- Set “SIP Registration” to “Yes”.
- Now enter the SIP account data as shown in the image. (This example uses the data of an Antisip account).

![Intercom Audio SIP 963x1024](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Audio-SIP.png)

## SIP settings with new audio module from 10/2020:

Connect to the web interface of the audio module via browser (configured IP, or **default** IP **192.168.1.98**, default login: **admin/admin**)

Passwords up to 10 characters in length are supported.

First click on **SIP Configuration** **(1)

**Then enter the **username** registered with the [SIP account](#AudioExtern) in **2** and **4**, and the **password** at **5**.

At **3** enter the SIP server, in the example **sip.antisip.com:**

[
![sip setup 1](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/sip-setup-1.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/sip-setup-1.png)

**Finally click on “SAVE” to save the settings!**

			Finally, the SIP account data must be entered in Loxone Config.
- Select Intercom
- Enter the “Host for audio (external)”.
- Enter the “Audio username (external)”.

					After saving to the Miniserver, the configuration of the external audio connection is complete.

![Intercom Config SIP 1024x381](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-Config-SIP.png)

## Update

			**[
![download](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/01/download.png)
](http://popeye.loxone.com/tl_files/loxone/documentation/DE/zubehoer/intercom_xl/Loxone_Intercom_Sip_Update_20160829.zip)****[Download firmware for Loxone Intercom / Intercom XL](https://www.loxone.com/dede/wp-content/uploads/sites/2/2019/10/20191001_LoxoneSipUpdate.zip)**

To update the audio module, access its web interface ([detailed information can be found here](#Audio)).

					This update is not compatible with the new audio module of the Intercom from 10/2020!
- Download the update file and unzip the compressed folder.
- Select the “Advanced” button.
- Select the “Update” setting.
- Select the downloaded update file on your PC (*.bin).
- The update is started by clicking on the “Start transfer” button.

![intercom update 1024x476 1024x476](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/intercom_update-1024x476.png)

					From Config Version 8.0 and App Version 7.0 on, the password for the audio module is also used for communication between App and Intercom. Older versions have to be updated (Config, App, Intercom)!
The password for the audio module must be entered in Config in the properties of the Intercom.

## Factory Settings

### Intercom Gen. 1 Audio Module before 10/2020

**This Version features a reset switch on the circuit board.**
- Power off Loxone Intercom
- Set the DIP switch on the side of the audio module to ON (normal state = OFF).

![SIPswitch](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/05/SIPswitch.png)
The image shows the normal state (OFF) of the DIP switch. (right position [->]).
- Restore the power supply to the Intercom.
- Once the module restarted, you can return the DIP switch to the normal state (OFF).
- The audio module is now accessible at the IP address 192.168.1.98.
- Now connect to the web interface of the audio module (address: 192.168.1.98) [Detailed information can be found here](#Audio).
- Now configure the new network settings for the audio module. [Detailed information can be found here](#AudioNetzwerk).
- Please note that the network settings must match your home network (subnet mask, default gateway, etc.).
The entered IP address must not yet be assigned.

					After booting successfully, the Loxone Intercom is reachable at the new IP address.

### Intercom Gen. 1 Audio Module after 10/2020

**This Version is reset by pressing the doorbell button as follows:**

While pressing the button, power up the station by connecting to a PoE switch
Hold the button until the station audio starts counting, and release the button on **count 1**
Press and hold the button on **count 3** and release on **count 0**
- If there is no 0 count, the procedure has failed and you have to start again
- Press the **button**, and the station will speak its IP address

# Intercom XL

### Inhalt
- [Configure Call Buttons](#Ruftasten)
- [Learn iButton](#iButton)

## Configure Call Buttons

			Call buttons are only available on the Intercom XL!

To configure the call buttons, open the web interface of the audio module ([Detailed information can be found here](#Ruftasten)).
- Select the “Advanced” button.
- Open the “Directory”.

![Intercom XL Verzeichnis 1013x1024](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-XL-Verzeichnis.png)

			Up to 100 call buttons can be configured in the directory
- Name of the call button (shown as text on the display of the Intercom)

![Intercom XL Ruftasten 1024x660](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-XL-Ruftasten.png)

			For each call button a separate Door Controller is added in Loxone Config ([Detailed information can be found here](#Schritt8)).
- Select desired Door Controller
- Enter the call button ID

					The configuration is completed by saving to the Miniserver.

![Intercom XL Config Ruftasten 1024x459](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-XL-Config-Ruftasten.png)

## Learn iButton

			The iButton reader is only available on the Intercom XL!
iButtons can be learned directly at the Intercom XL.
- Select the Intercom in Loxone Config
- Select the 1-Wire Monitor button

![Intercom XL iButton Monitor](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-XL-iButton-Monitor.png)
- Hold the iButton to the reader of the Intercom XL.
- Select the iButton

![Intercom XL iButton einlernen 1024x142](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/Intercom-XL-iButton-einlernen.png)
- **Create iButton as a device**

Enter a name for the device
- Confirm with “Create device”

					The iButton can now be used as an input in Loxone Config
- **Assign the iButton to a user**
- Select the user
- Confirm with “Assign user”

					The [Access Controller ](https://www.loxone.com/enen/kb/access-controller/)is used for further programming

# Diagnostics

### Inhalt
- [Status messages for failed audio connections](#DiagnoseErrorMeldungen)
[LED States](#DiagnoseLEDs)
[No sound during audio connection](#DiagnoseTon)
[No sound on Android devices](#DiagnoseTonAndroid)
[No image with Internet Explorer](#DiagnoseBildInternetExplorer)
[No image with Google Chrome](#DiagnoseBildGoogleChrome)
[Intercom cannot be found on the network](#DiagnoseNetzwerk)

## Status messages for failed audio connections

| “Busy” | Another user is already connected |
| --- | --- |
| “Please try again later” | The connection is busy, try connecting again at a later time. |
| “The party is (temporarily) unavailable” | Code 480 The system is currently unreachable. Possible causes: |
- SIP account is set up incorrectly. Check the settings in the Intercom and Loxone Config.
- SIP server is not available, check if the system is reachable.
For external connection, check whether the server of the SIP provider can be reached.

“An error was received while establishing the voice connection: <error code>”[List of Sip Status Codes](https://en.wikipedia.org/wiki/List_of_SIP_response_codes)“”The other party (__host__) has rejected the call.” Please check the configuration.

“Host or username incorrect”

or.

“Please check the username and host of your audio connection, ‘<userAndHost>’ is invalid.”

Check the account setting in the Intercom and Loxone Config

## LED States

4 status LEDs are located on the side of the SIP module. The following points explain their meaning.

### Network status

| LED IN | Indicates that a connection to a network is established via X1. |
| --- | --- |
| LED OUT | Indicates that the connection is forwarded via X2. |

Flashing LEDs indicate data traffic

### Operating status

| LED2 | LED1 | Description |
| --- | --- | --- |
| off | off | Idle, connection with SIP server established |
| on | off | Idle, no connection to SIP server established |
| off | on | Establishing a connection to an external party |
| on | on | Loading/saving factory settings |
| on off | off on | Alternating flashing: Device is in save mode |
| off | flashing | Factory settings loaded/saved – Jumper can be removed |
| flashing | off | Error loading or saving factory settings or more than one jumper set |

## No sound during audio connection

Please check if the SIP module is on the latest firmware version – [Click here for update instructions](#AudioUpdate).
- Check the network settings of the SIP module and the Miniserver. A firewall may also be blocking SIP connections – consult a network technician if necessary.
- Check for SIP port overlap with other SIP phones on the same network.
- Check the connection to the SIP service from your network, perhaps the service is not available or the network or the ISP blocks this kind of connection. Consult a network technician if necessary.
- If error messages are returned during an external audio connection, [find detailed information here](#DiagnoseErrorMeldungen).

## No sound on Android devices

					Smartphone manufacturers or cellular carriers can influence the Android operating system and may block SIP support.
Since our app relies on Android’s SIP API, the audio connection will not work in such cases.
Please check with your provider whether these functions can be used.

## No image with Internet Explorer

Starting with Internet Explorer version 6, it is no longer possible to include the user name and password in the URL. However, this is essential for automatic authentication when the camera image is updated by the door station.

This means that not all features of the Door Controller are available with Internet Explorer 6 or higher. We recommend using a different browser, such as Google Chrome or Firefox.

[Click here for the Microsoft article discussing this topic](http://support.microsoft.com/default.aspx?scid=kb;[LN];834489)

## No image with Google Chrome

Google Chrome has changed its authentication method in recent versions, so there may be problems when transmitting images.

Solution:

Add the following command to the target path in the Google Chrome icon properties:

“ –allow-cross-origin-auth-prompt“

To do this, right-click on the Google Chrome icon and open the properties.

![chrome_settings](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/05/chrome_settings.jpg)

Now, in the “Shortcut” tab, add ” -allow-cross-origin-auth-prompt” to the Target.

The target field should now contain: (may vary depending on operating system and path)

„C:\Program Files (x86)\Google\Chrome\Application\chrome.exe“ –allow-cross-origin-auth-prompt

![chrome_add](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/05/chrome_add.jpg)

## Intercom cannot be found on the network

If the intercom cannot be found on the local network, follow the steps below.
- Check the connection of the Intercom to your home network ([Step 1](#Schritt1))
- Check the network settings of the Miniserver and whether they match your existing home network (subnet mask, default gateway, etc.). The IP address must not be assigned twice.
- Send new network settings to the Intercom module via Loxone Config.

Intercom Video: [Detailed information can be found here](#VideoNetzwerkConfig)
- Intercom Audio: [Detailed information can be found here](#AudioNetzwerkConfig)

If the settings cannot be sent to the module via Loxone Config:
- Many routers offer an overview of the currently connected devices in your home network. Can you find the Intercom in this overview?

					Try to reach the module via the IP address provided by the router.
- Intercom Video: [Detailed information can be found here](#VideoNetzwerk)
- Intercom Audio: [Detailed information can be found here](#VideoNetzwerk)
- If no IP can be obtained via DHCP when starting the Intercom, the IP of the audio module is set to 192.168.1.98 and the IP of the video module is set to 192.168.1.99. Check if these addresses can be reached using a browser.

					If they are accessible, check the current network settings of the module via the web interface and adjust them if necessary.
- Intercom Video: [Detailed information can be found here](#VideoNetzwerk)
- Intercom Audio: [Detailed information can be found here](#VideoNetzwerk)

If the default addresses are not reachable:
- Connect the network cable of the Intercom directly to your computer.
- Set your computer to a static IP (e.g. 192.168.1.50, subnet mask 255.255.255.0, default gateway remains blank).
- Now try accessing the two default IP addresses again.

					If they are accessible, check the current network settings of the module via the web interface and adjust them if necessary.
- Intercom Video: [Detailed information can be found here](#VideoNetzwerk)
- Intercom Audio: [Detailed information can be found here](#VideoNetzwerk)
- If both IP addresses are still not available, please reset the modules to factory settings.

[Restore factory settings Video](#VideoWerkseinstellungen)
- [Restore factory settings Audio](#AudioWerkseinstellungen)

# Technical Data

[Dimensions Loxone XL Intercom](https://www.loxone.com/dede/wp-content/uploads/sites/2/2016/08/Montageanleitung-Loxone-XL-Intercom.pdf)

[Dimensions Loxone Intercom](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/02/DOC_measures-intercom.pdf)

The front plate of both the Loxone Intercom and Intercom XL is made of anodized aluminum.

Instruction Manual Loxone Intercom / Intercom XL (German, pdf)

[Download](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/02/DOC_Handbuch-_Intercom_XL.pdf)
Intercom CE Declaration of Conformity (German, pdf)

[Download](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/02/DOC_CE-Konformitaetserklaerung-Intercom.pdf)
Intercom XL CE Declaration of Conformity (German, pdf)

[Download](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/02/DOC_CE-Konformitaetserklaerung-Intercom-XL.pdf)