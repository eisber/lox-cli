# Audio Player Fixed Group

Source: https://www.loxone.com/enen/kb/audio-player-fixed-group/

---

An Audio Player Group can be used to combine multiple Audio Player blocks. This allows for a uniform music experience in open spaces.

All Audio Player blocks that are in the same group will always play the same source. However, they remain independent in terms of volume, so that a suitable volume can be selected for different areas of a building.

For each desired group, create an Audio Player Group and assign the desired Audio Players in the block settings.

In the following example, a group was created with the Audio Player blocks in the living room and kitchen, since there is no physical separation between the two areas:

![11.1 audiogroup fixed](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 audiogroup fixed.png)

> **ℹ️ Note:** In contrast to the fixed player group in Loxone Config, dynamic groups of individual players can also be created, edited and removed by users of the Loxone App or Webinterface.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| V+ | Volume+ | Increase volume by value set in parameter (Vsts). Double-click selects next favorite. | - | 0/1 |
| V- | Volume- | Decrease volume by value set in parameter (Vsts). Double-click turns off the player. | - | 0/1 |
| V | Set volume | If the assigned Audio Players are set to different volumes, only the loudest players adopt the new value when the volume is changed; all other players follow accordingly but retain the volume ratio. | % | 0...100 |
| Play | Play | - | 0/1 |
| Pause | Pause | - | 0/1 |
| P | Presence | Starts playback when 1. | - | 0/1 |
| Prev | Previous track | - | 0/1 |
| Next | Next track | - | 0/1 |
| Fav | Set favorite | Selects a favorite based on its assigned ID number. If the selected ID does not exist, the first favorite is selected. | - | ∞ |
| Alarm | Alarm | Plays alarm sound at volume set in parameter (Va). | - | 0/1 |
| FireAlarm | Fire alarm | Plays fire alarm sound at volume set in parameter (Va). | - | 0/1 |
| Bell | Bell | Plays doorbell sound at volume set in parameter (Vbell). | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | - | 0/1 |
| DisPc | Disable periphery control | Disables inputs V+, V-, V, Play, Pause, Prev, Next, Fav, Bell, Buzzer, T5, TTS, Cs, BTp when On. (e.g child lock, cleaning) Control via user interface is still possible. | - | 0/1 |
| DisP | Disable presence | As long as this input is active, any change in value at input (P) is ignored. | - | 0/1 |
| T5 | T5 control | Button 2 : Volume up; double-click starts playback or choses next favorite Button 5 : Volume down; double-click pauses playback Button 3: Double-click activates (2C); triple-click activates (3C); (Roff) = 0: pauses playback | - | ∞ |
| TTS | Text to speech | Converts a Text to speech and plays it at the volume set in parameter (Vtts). | - | - |
| LineIn | Set Line In | Selects the Line In of an Audioserver as source based on its Line In ID. | - | 0...∞ |
| Cs | Custom sound | Plays custom sound [filename] at volume [vol]. E.g. soundcheck.mp3:80 [filename]:[vol] Custom sounds have to be stored on the Audioserver’s SD card in the folder Event_Sounds! Only mp3 files are supported. | - | - |

---

## Outputs

| Abbreviation | Summary | Description |
| --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Tdc | Time double-click | s | 0...10 | 0.35 |
| Roff | Ignore room off command | Ignore Room Off / House Off via T5 (button 3) | - | 0/1 | 0 |
| BTp | Bluetooth pairing | As long as this input is active, bluetooth pairing is available. During this time all bluetooth-enabled Loxone devices can be paired with e.g. a smartphone or other bluetooth-enabled devices. Input is only visible when services and bluetooth are handled by player group This parameter is only visible in certain configurations. | - | 0/1 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Selection | All selected function blocks can be controller together. | - |
| Enable AirPlay | Enable AirPlay for all players of this group | - |
| Enable Spotify Connect | Enable Spotify Connect for all players of this group | - |
| Handle services by group | If services are handled by player group, the configured players are not visible in Airplay and Spotifiy connect. | - |
| Room Favorite Priority | If checked, the first room favorite will always be used when the player is activated. | - |

---