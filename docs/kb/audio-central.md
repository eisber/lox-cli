# Audio Central

Source: https://www.loxone.com/enen/kb/audio-central/

---

This central block can be used to control multiple audio block together.
The functions that can be used depend on the linked blocks such as Audio Player or Music Server Zone, and are set via their parameters.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Basic Programming](#basic)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| TgZ | Toggle Zone | Toggles zone on and off | - | 0/1 |
| Zon | Zone On | Digital Input - Single pulse turns zone on | - | 0/1 |
| Zoff | Zone Off | Digital Input - Single pulse turns zone off When playing a playlist, pressing Play will restart playback at the beginning of the title. | - | 0/1 |
| V+ | Volume+ | Increase volume by value set in parameter (Vsts). Double-click selects next favorite. | - | 0/1 |
| V- | Volume- | Decrease volume by value set in parameter (Vsts). Double-click turns off the player. | - | 0/1 |
| V | Set volume | If the assigned Audio Players are set to different volumes, only the loudest players adopt the new value when the volume is changed; all other players follow accordingly but retain the volume ratio. | % | 0...100 |
| S+ | Casatunes Music Server: Next Source | Trigger next source | - | 0/1 |
| Loxone Music Server: Next zone favorite | Trigger next zone favourite. If a user-defined selection was made last, the first zone favourite is played. | - | 0/1 |
| Audioserver: Next zone favorite | Trigger next zone favourite. If a user-defined selection was made last, the first zone favourite is played. | - | 0/1 |
| Play | Play | - | 0/1 |
| AIs | Casatunes Music Server: Source | Analogue Input Source This input is only visible in certain configurations. | - | ∞ |
| Loxone Music Server: Zone favorite | Analog input zone favourite This input is only visible in certain configurations. | - | 0...8 |
| Pause | Pause | - | 0/1 |
| Stop | Stop playback | Trigger stop playback When playing a playlist, pressing Play will restart playback at the beginning of the title. | - | 0/1 |
| Shuffle | Shuffle | Trigger shuffle This input is only visible in certain configurations. | - | 0/1 |
| Repeat | Repeat | Analogue Input - Value to indicate repeat state 0 = Off, 1 = Repeat Continuously, 2 = Repeat Once This input is only visible in certain configurations. | - | 0...2 |
| Sleep | Sleep timer input | Zone is muted and switched off after duration specified in Ts The sleep timer is reset when switching off the zone (Off, Pause, Stop, R). This input is only visible in certain configurations. | - | 0/1 |
| TTS | Text to Speech input | Text to Speech input Maximum length 400 characters | - | - |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Next | Next track | - | 0/1 |
| Prev | Previous track | - | 0/1 |
| T5 | T5 control | Button 2 : Volume up; double-click starts playback or choses next favorite Button 5 : Volume down; double-click pauses playback Button 3: Double-click activates (2C); triple-click activates (3C); (Roff) = 0: pauses playback | - | ∞ |
| DisPc | Disable periphery control | Disables inputs V+, V-, V, Play, Pause, Prev, Next, Fav, Bell, Buzzer, T5, TTS, Cs, BTp when On. (e.g child lock, cleaning) Control via user interface is still possible. | - | 0/1 |
| DisP | Disable presence | As long as this input is active, any change in value at input (P) is ignored. | - | 0/1 |
| Alarm | Alarm | Plays alarm sound at volume set in parameter (Va). | - | 0/1 |
| FireAlarm | Fire alarm | Plays fire alarm sound at volume set in parameter (Va). | - | 0/1 |
| Bell | Bell | Plays doorbell sound at volume set in parameter (Vbell). | - | 0/1 |
| Buzzer | Buzzer | Starts alarm clock action as specified in the properties. If the action "Alarm Clock Chime" is selected, the alarm clock sound is played at the volume set in parameter (Vbuzzer). | - | 0/1 |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | - | 0/1 |
| Cs | Custom sound | Plays custom sound [filename] at volume [vol]. E.g. soundcheck.mp3:80 [filename]:[vol] Custom sounds have to be stored on the Audioserver’s SD card in the folder Event_Sounds! Only mp3 files are supported. | - | - |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands | - |
| Na | Active Audio Players | Number of active Audio Players | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Tdc | Time double-click | s | 0...10 | 0.35 |
| Roff | Ignore room off command | Ignore Room Off / House Off via T5 (button 3) | - | 0/1 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Selection | All selected Music Server Zones and Audio Players can be controlled together. | - |

---

## Basic Programming

Double-click on the block to open the following window, where the desired audio blocks can be selected:

![audiocentral assignblocks](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/audiocentral-assignblocks.png)

Central commands are not blocked by an active (DisPc) input at the respective function block. If a function block is used in the central block, this is indicated by the central symbol on the respective block.
The functions that can be used on the central block depend on the linked blocks and are set via their parameters. If a function block does not support a function, it cannot be controlled.