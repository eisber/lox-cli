# Audio Player

Source: https://www.loxone.com/enen/kb/audio-player/

---

With the Audio Player function block you can define an Audio Zone. This is usually a room or an area of a building where audio playback is desired. The block is used with the outputs of the [Audioserver](https://www.loxone.com/help/audioserver) or a [Stereo Extension](https://www.loxone.com/help/stereo-extension).

It is possible to create [groups](https://www.loxone.com/help/audiogroupfix) across rooms, providing a uniform music experience in open spaces.

For larger rooms, multiple speaker outputs are assigned to the same Audio Player function block. Each speaker output can be configured to output either the left or right channel signal, or a downmix of both channels.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Programming example](#desc1)
- [Possible uses of the Stereo L and R outputs](#desc2)
- [User Interface and Operation](#visu)
- [Internet Radio](#tunein)
- [Adding a Custom Internet Radio Stream URL to the Loxone App](#custominternetradio)
- [Spotify](#spotify)
- [Soundsuit](#soundsuit)
- [Library](#library)
- [Files on USB storage devices](#usb)
- [Files on the Audioserver/Miniserver Compact](#local)
- [Files on network shares](#shares)
- [Line In](#linein)
- [AirPlay](#AirPlay)
- [Text to Speech (TTS)](#TTSdesc)
- [Play Custom Sound](#CustomSound)
- [Presence Simulation](#PresenceSimulation)
- [History](#history)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| V+ | Volume+ | Increase volume by value set in parameter (Vsts). Double-click selects next favorite. | - | 0/1 |
| V- | Volume- | Decrease volume by value set in parameter (Vsts). Double-click turns off the player. | - | 0/1 |
| V | Set volume | If the player is turned off, playback will start automatically. | % | 0...100 |
| Play | Play | - | 0/1 |
| Pause | Pause | - | 0/1 |
| P | Presence | Starts playback when 1. | - | 0/1 |
| Prev | Previous track | - | 0/1 |
| Next | Next track | - | 0/1 |
| Fav | Set favorite | Selects a favorite based on its assigned ID number. If the selected ID does not exist, the first favorite is selected. | - | ∞ |
| Alarm | Alarm | Plays alarm sound at volume set in parameter (Va). | - | 0/1 |
| FireAlarm | Fire alarm | Plays fire alarm sound at volume set in parameter (Va). | - | 0/1 |
| Bell | Bell | Plays doorbell sound at volume set in parameter (Vbell). | - | 0/1 |
| Buzzer | Buzzer | Starts alarm clock action as specified in the properties. If the action "Alarm Clock Chime" is selected, the alarm clock sound is played at the volume set in parameter (Vbuzzer). | - | 0/1 |
| LineIn | Set Line In | Selects the Line In of an Audioserver as source based on its Line In ID. | - | 0...∞ |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| DisPc | Disable periphery control | Disables inputs V+, V-, V, Play, Pause, Prev, Next, Fav, Bell, Buzzer, T5, TTS, Cs, BTp when On. (e.g child lock, cleaning) Control via user interface is still possible. | - | 0/1 |
| DisP | Disable presence | As long as this input is active, any change in value at input (P) is ignored. | - | 0/1 |
| T5 | T5 control | Button 2 : Volume up; double-click starts playback or choses next favorite Button 5 : Volume down; double-click pauses playback Button 3: Double-click activates (2C); triple-click activates (3C); (Roff) = 0: pauses playback | - | ∞ |
| TTS | Text to speech | Converts a Text to speech and plays it at the volume set in parameter (Vtts). | - | - |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | - | 0/1 |
| Tg | Toggle | Toggles between play and pause | - | 0/1 |
| Cs | Custom sound | Plays custom sound [filename] at volume [vol]. E.g. soundcheck.mp3:80 [filename]:[vol] Custom sounds have to be stored on the Audioserver’s SD card in the folder Event_Sounds! Only mp3 files are supported. | - | - |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Stereo LR | Stereo Left & Right | Outputs the full stereo signal | 0/1 |
| Play | Play status | 1 when player is currently playing | 0/1 |
| Volume | Current volume | ∞ |
| 2C | Pulse on double-click | Pulse on a double- or triple-click or pulse on input (Off). | 0/1 |
| 3C | Pulse on triple-click | Pulse on a triple-click. | 0/1 |
| Stereo L | Stereo Left | Outputs the left channel of the stereo signal. | 0/1 |
| Stereo R | Stereo Right | Outputs the right channel of the stereo signal. | 0/1 |
| V+ | Pulse on Volume+ | Functionality is only enabled when Stereo Outputs with external controlled "Volume-Mode" are connected. | 0/1 |
| V- | Pulse on Volume- | Functionality is only enabled when Stereo Outputs with external controlled "Volume-Mode" are connected. | 0/1 |
| Sub | Subwoofer | Outputs the subwoofer channel of the stereo signal. This output can only be used with Master/Client subwoofers. | 0/1 |
| BTp | Bluetooth Pairing | As long as this output is active, Bluetooth pairing is active. During this time all Bluetooth-capable Loxone devices can be paired with a smartphone or other Bluetooth devices. This output is only visible in certain configurations. | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Von | Power on volume | Volume when playback is started or resumed. -1 = Saves the last volume setting as the startup volume. | % | -1...100 | 10 |
| Vm | Maximum volume | Setting for maximum volume | % | 0...100 | 100 |
| Vsts | Step size volume | Decrease volume by value set in parameter (Vsts). Double-click turns off the player. | % | 1...100 | 1 |
| Va | Minimum volume alarm sounds | If the current playback volume is higher, it is used instead of the minimum volume. | % | 0...100 | 75 |
| Vbell | Minimum volume doorbell | If the current playback volume is higher, it is used instead of the minimum volume. | % | 0...100 | 50 |
| Vbuzzer | Minimum volume alarm clock | If the current playback volume is higher, it is used instead of the minimum volume. | % | 0...100 | 10 |
| Vtts | Minimum volume TTS and announcements | If the current playback volume is higher, it is used instead of the minimum volume. | % | 0...100 | 20 |
| Tdc | Time double-click | s | 0...10 | 0.35 |
| Roff | Ignore room off command | Ignore Room Off / House Off via T5 (button 3) | - | 0/1 | 0 |
| BuzzerFav | Alarm clock favorite | Room favorite ID for alarm clock, if the alarm clock action room favorite is used. | - | ∞ | 1 |
| BTp | Bluetooth pairing | As long as this input is active, bluetooth pairing is available. During this time all bluetooth-enabled Loxone devices can be paired with e.g. a smartphone or other bluetooth-enabled devices. This parameter is only visible in certain configurations. | - | 0/1 | 0 |
| Ft | Volume Fading Time | Volume adjustments made via input V are applied within the configured time when the player is already active. The fading time also applies to the Buzzer input. 0 = No fading; volume changes occur immediately. | s | 15...1800 | 0 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Alarm Clock Action | Action that is carried out when the alarm goes off | - | - |
| Enable AirPlay | Enable AirPlay for this player. Option is not available if music services are handled by Player-Group. | - | - |
| Enable Spotify Connect | Enable Spotify Connect for this player. Option is not available if music services are handled by Player-Group. | - | - |
| Room Favorite Priority | If checked, the first room favorite will always be used when the player is activated. | - | - |
| Activity Log Entries | Number of entries in the activity log. 0: log is disabled The activity log tracks relevant changes since program start. | 0...100 | 20 |
| Player-ID | ID of the player on the Audioserver or audio device | ∞ | - |
| Automatic Highpass-Filter | When a subwoofer is connected to the Audio Player, all other speakers automatically stop reproducing bass frequencies, reducing their load. This behavior can be disabled by unchecking the corresponding checkbox, allowing all connected speakers to play the full frequency range. | - | - |

---

## Programming example

The following image shows a programming example with an Audioserver and two Stereo Extensions:

![11.1 audioplayer example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audioplayer example.png)

The first Audio Player block in the living room uses the 2 stereo speaker outputs of the Audioserver, i.e. 2 pairs of speakers. Since it is the same room or area, only one Audio Player block is used. The (Stereo LR) output provides both channels and the combined actuator (ALR) passes them on to the Audioserver. On the device itself, the channels are output at the corresponding terminals for the left and right speakers.

The second Audio Player block in the bedroom is used with the stereo output of the Stereo Extension 1, i.e. one pair of speakers.

One of the two [separated](https://www.loxone.com/help/audioserver#SeparateSpkrOutputs) channels of Stereo Extension 2 was used on each of the two Audio Player blocks in the kitchen and garden, i.e. only one speaker per room.
The speakers are connected to the (Stereo LR) output of the block, so each speaker receives the full stereo signal, although the stereo effect is lost in this case. (Downmix)

A Loxone Touch of the respective room is connected to each block for operation according to the Loxone Switch Standard. Depending on the customer's preference, a Presence Sensor can be used, as shown in our example in the living room and kitchen.

---

## Possible uses of the Stereo L and R outputs

The (Stereo L) and (Stereo R) outputs of the block provide only the signal of the respective channel.
The following examples show possible applications for this:

**Example 1:** The layout or usage of a room has changed, and the left and right channels must be swapped.
The stereo output can be [separated](https://www.loxone.com/help/audioserver#SeparateSpkrOutputs), the right speaker output is then connected to the (Stereo L) output of the block, and vice versa. In this way, the channels can be swapped without having to change the wiring:

![11.1 audioplayer LR example1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audioplayer LR example1.png)

**Example 2:** An odd number (3,5,7...) of speakers is used in a room.
The additional speaker could be provided with the full stereo signal, but if it is located on the left side of the room, for example, it makes sense to provide it with the left signal only:

![11.1 audioplayer LR example2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audioplayer LR example2.png)

**Example 3:** Only single speaker outputs are free on different Stereo Extensions, but a Stereo Zone is needed.
(Stereo L) is connected to a separated output of one Extension, and (Stereo R) is connected to a separated output of the other Extension, thus merging the outputs:

![11.1 audioplayer LR example3](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audioplayer LR example3.png)

**Example 4:** A large room is equipped with many speakers.
It may be beneficial to install the Stereo Extensions on the side of the room where the speakers are located. Connect all outputs of the Stereo Extensions on the left side to the (Stereo L) output of the Audio Player block. The same is done for the right side:

![11.1 audioplayer LR example4](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audioplayer LR example4.png)

---

## User Interface and Operation

Once the configuration has been saved to the Miniserver and it has been restarted, the Audio Player blocks are available in the corresponding rooms.

Options for music selection are available by clicking on the block in the app or the user interface:

![11.1 audioplayer visu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audioplayer visu.png)

Browse and select Internet radio stations, or add your Spotify account or a network location. Create favourites for each room to quickly access them.

A graphic 10-band equalizer is available in the settings of the user interface.

When using a Loxone Touch, operation is via the two right buttons, according to the [switch standard](https://www.loxone.com/enen/smart-home/switch-standard/).

The same pattern of operation is used both in the App and for the Loxone Touch:

**Button + (up):** Increase volume. Double click: Skip to next favorite. If switched off, pressing + will also switch on.

**Button - (down):** Decrease volume. Double click: Switch off. If switched off, pressing - will also switch on.

The playback starts with the **Startup volume** according to parameter (Von). If you want the Audio Player to save the last selected volume and start with it, set parameter (Von) to -1.

---

## Internet Radio

The Internet radio service TuneIn offers numerous radio stations and podcasts. You can search for stations in the app or user interface and add them to your favourites. To do this, click on the menu icon (3 dots) and select **Save as Favourite**.

![radiosearch](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/radiosearch.png)

---

## Adding a Custom Internet Radio Stream URL to the Loxone App

The Loxone App allows users to add custom audio stream URLs to the Audio Player. This feature is useful for accessing radio stations that are not natively supported, such as BBC. Please note that an active internet connection is required for this functionality.

**Steps to Add a Custom URL Stream:**

**1. Find the Stream URL:**
- Open a web browser and search for the stream URL of the desired radio station.
- For example, BBC Radio stream URLs can be found at: [BBC Radio Streams](https://gist.github.com/bpsib/67089b959e4fa898af69fea59ad74bc3)
- Copy the URL of the stream you want to add. Make sure it’s a valid, directly playable stream URL (we recommend testing it first in a media player like VLC to confirm compatibility).

![BBC RadioStream](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/BBC_RadioStream.png)

**2. Add the Stream to the Loxone App:**
- Open the Loxone App and navigate to the desired Audio Player.
- Go to Internet Radio.
- Tap the + icon in the top right corner.

![Internet Radio](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Internet_Radio.png)

![Add Custom URL](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Add_Custom_URL.png)

**3. Enter Stream Details:**
- In the new window, paste the copied URL.
- Optionally, add a custom name and an image for the stream.
- Click the checkmark icon in the top right corner to confirm.

![Add Radio Station](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Add_Radio_Station.png)

Once these steps are completed, the radio station will be added to the Audio Player and ready for use.

---

## Spotify

The integration with Spotify allows playback of Spotify music or playlists. A **Spotify Premium** account is required.
One Premium account supports a single stream to one or multiple zones at once, using **Fixed** or **Dynamic Groups**.
To stream different content to multiple zones simultaneously, multiple **Premium** accounts are required. These do not need to be part of a Family plan.

![spotifyadd](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/spotifyadd.png)

In the Loxone App click on Spotify, then click on Add Spotify Account. You will be redirected to the Spotify website. Log into your Spotify account and confirm access to your account.

The service can now be used.

**Spotify Connect**
This means that all Audio Players are visible and controllable by the Spotify App.

---

## Soundsuit

[Soundsuit](https://soundsuit.fm/) is a music service for commercial use that can be added in the Loxone app. Click on the gear symbol to the right of the search field to display Soundsuit.
A [Soundsuit account](https://soundsuit.fm/loxone-audio-for-business/) is required.

![audioplayer soundsuit](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/audioplayer_soundsuit.png)

The service can then be used.
Click on the cogwheel at Schedule to open Soundsuit, where music can be set for different target groups at the defined times.

![audioplayer soundsuit schedule](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/audioplayer_soundsuit_schedule.png)

---

## Library

The library is visible when audio files are available on local drives or devices on the network:

![library](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/library.png)

If necessary, display the library using the gear symbol to the right of the search field.

The following sections explain how to add available sources for audio files.

---

## Files on USB storage devices

USB storage devices containing audio files can be connected to the USB port on the Audioserver/Miniserver Compact. Details on supported drives and audio formats are described in the [Audioserver](https://www.loxone.com/help/audioserver#)/[Miniserver Compact](https://www.loxone.com/help/miniserver-compact#) data sheet. Hard drives with a power consumption of more than 500mA must be connected to an external power supply.

As soon as a USB drive is plugged in, the audio files on the drive are listed in the Library in the App or user interface.

Note that local files on USB storage devices are only available on Audio Player blocks that are linked to the Audioserver/Miniserver Compact that the USB device is connected to.
If you want to access your music files on multiple Audioservers/Miniserver Compact, we recommend adding the files to the library via a central network share e.g. a NAS.

**Supported formats:** MP3, AAC, ALAC, FLAC, WMA, WMA lossless, WAV, M4A, OGG

---

## Files on the Audioserver/Miniserver Compact

Files can also be stored on the SD card of the Audioserver or Miniserver Compact. The folders can be accessed via an SMB network share provided by the Audioserver/Miniserver Compact.

In Windows, you can access them by entering the IP address or hostname of the Audioserver/Miniserver Compact in the File Explorer using this format: \\192.168.1.7 or \\hostname

![AS share exp ip](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AS share exp ip.png)

The **Library** folder on the SD card can be used to store audio files that can then be played from the library. Please keep in mind that the SD card is limited in size and speed. We recommended using a USB drive or a network share for larger music collections.

The folder **Event_Sounds** on the SD card contains audio files for events such as doorbell, alarm clock, or alarms. The sounds can be changed by replacing these audio files. The new files must match the original file name and format.

The **usb** folder is visible when a USB drive is connected to the Audioserver/Miniserver Compact. This allows you to add or remove files on the USB disk.

The **Updates** folder is reserved for update files that are manually copied to the Audioserver/Miniserver Compact.

These local folders are intended for easy file management and maintenance, not for use as a network share to be added to the library on other Audioservers/Miniserver Compact.

Note that local files on the SD card are only available on Audio Player blocks that are linked to the Audioserver/Miniserver Compact with that SD card.
If you want to access your music files on multiple Audioservers/Miniserver Compact, we recommend adding the files to the library via a central network share e.g. a NAS.

**Possible Reasons Why the SD Card Is Not Accessible in Windows Explorer**

1. SMB1 Protocol Not Supported

The outdated SMB1 protocol is no longer supported by default in Windows. To enable access, you may need to modify two registry entries in Windows 10.

Open **Regedit** on the Windows computer.

Navigate to the following location:

Computer\HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\LanmanWorkstation\Parameters

It may be necessary to change two registry entries in Windows 10 to the following values:

![regsmb](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/regsmb.png)

Also ensure that **RequireSecuritySignature** is visible and that it is set as a DWORD with a value of 0.

2. Windows 11 Update (Version 24H2) Login Prompt Issue

After updating to Windows 11 version 24H2, you may encounter a login prompt when attempting to access the SD card. This could be due to the registry entry **AllowInsecureGuestAuth** being deleted.

To resolve this:

Navigate to the same registry path mentioned above.

If the **AllowInsecureGuestAuth** entry is missing, create a new DWORD (32-bit) Value with this name.

Set its value to 1.

---

## Files on network shares

SMB network shares can be added to the library. You only need to add a specific share once, it will then be available across all Audioservers or Audio Players.
To do this, click on Library in the App, then on the menu icon in the top right corner (three dots), then select **Add network folder**.

The following window opens. Enter the information for your network share:

![setup share pre](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/setup share pre.png)

Finally, click Add, after which the files will be imported and are then ready to be played from the library.

The outdated protocol SMB1 is not supported for access.

---

## Line In

The analogue audio input (Line In) of the Audioserver can also be selected as source. Click on the gear symbol to the right of the search field to display the Line In. It can then be used:

![linein](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/linein.png)

---

## AirPlay

As of version 2.5.01.11, the Audioserver supports audio playback via AirPlay 2.
Each output will be available as an AirPlay receiver, and can be selected as a streaming device on Apple devices.

Streaming via AirPlay is independent of the Audio Player function block and takes priority as long as the Apple device is connected. Any playback from another source that is already in progress will be interrupted when AirPlay streaming is started.

![airplayiphone](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/airplayiphone.jpg)

If there is only one Audio Player in a room, the Audio Player is named after the room, otherwise after the respective block.
These zone names are used in AirPlay as well as in Spotify Connect.
Within an installation, the naming of Apple devices and Audio Zones must be unique.

For more information on how to use AirPlay and the system requirements, please refer to the [Apple guide](https://support.apple.com/en-us/HT202809).

### Block operation during AirPlay playback:

The Audio Player block's functionality is limited during AirPlay playback, as AirPlay is controlling the playback during this time. This results in the following special properties:

When AirPlay playback is started on one output, playback on other outputs linked to that output is stopped and the link is temporarily broken, regardless of whether multiple outputs are linked to one Audio Player block, or multiple Audio Player blocks are grouped by the Audio Player Fixed Group block. Outputs that are not linked will continue to run normally.

This is necessary because each individual output can be selected as an AirPlay streaming receiver. Audio groups are then managed by AirPlay, and can be created by manually selecting the desired outputs (speakers) in the app of the respective AirPlay device.

The following functions are also available during AirPlay playback:
Pausing playback
Changing volume
Events such as doorbell, alarm clock, and alarms.

Other functions are not available during AirPlay playback, regardless of whether they are triggered from the App, by a button press, or logic.
Stop AirPlay playback or disconnect on the respective device to restore the previous functions and groups.

---

## Text to Speech (TTS)

The Text-to-Speech function enables texts to be converted into audible speech.
To achieve this, a text is input to the function block (TTS) input, which is then output through the assigned audio outputs.

> **ℹ️ Note:** An Internet connection is required for the TTS function. Texts of up to 300 characters each (including spaces) are supported.

By default, a TTS output is generated in the system language.
To use a different language, specify a prefix at the beginning of the text, separated by the pipe symbol.
Examples:
de|Das ist ein deutscher Text
en|Now the text is English

Supported languages:
de German
en British English
us American English
es Spanish
ca Catalan
cz Czech
sk Slovakian
pl Polish
fr French
nl Dutch
be Flemish
it Italian
cn Chinese
hu Hungarian
ru Russian
bg Bulgarian
fi Finnish
se Swedish
nb Norwegian
pt Portuguese
tr Turkish
ja Japanese

---

## Play Custom Sound

Start by adding the custom sound file you wish to use into the **Event_Sounds** folder located on the Audioserver, Miniserver Compact, or Master Speaker. You can access this folder via: \\{ip-address}\Event_Sounds

Example: soundcheck.mp3

![Soundcheck](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Soundcheck.png)

Next, create the logic in Loxone Config to trigger this custom sound.

For example, you can use a **Status block** or a **Text Generator block** to do so.
Remember to specify the playback volume by adding it at the end of the file path in this format:
[filename]:[volume]

Configure the logic to specify when the custom sound should play. For example, you could link it to the intercom’s bell output to play a specific bell sound in a designated room.

![CustomSoundLogic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/CustomSoundLogic.png)

**Note:** The custom sound file will play completely and cannot be stopped.

**Note:** A 2-second delay is applied to all custom sounds due to the required on-sync time before playback.

---

## Presence Simulation

This function block has a presence simulation.
Activate and define the presence simulation in the properties window:

![PresenceSimulation AudioPlayer](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PresenceSimulation_AudioPlayer.png)

---

## History

In the user interface, the history of the function block can be displayed.
A maximum of 100 entries can be shown.
When you restart or save to the Miniserver, the history is cleared.

![History AudioPlayer](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/History_AudioPlayer.png)