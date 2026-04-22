# Authentication NFC Code Touch

Source: https://www.loxone.com/enen/kb/authentication-nfc-code-touch/

---

The Authentication NFC Code Touch function block is used to program an NFC Code Touch.

NFC tags and numerical codes can be assigned to users to grant access to a building or for various other applications.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Basic Configuration](#baseconf)
- [Using NFC Tags](#NFCtags)
- [Read/Format NFC Tags](#ReadFormatTags)
- [Using numeric codes](#codes)

---

## Inputs

Please Note: Status light inputs cannot be used with battery powered devices!

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. Authentication no longer possible, device will flash red. The name of the connected sensor is used in the user interface. | 0/1 |
| Lr | Turns status LEDs red | 0/1 |
| Lg | Turns status LEDs green | 0/1 |
| Lb | Turns status LEDs blue | 0/1 |
| Lw | Turns status LEDs white | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O1-6 | Output 1-6 | Activates (Output 1-6) for the duration of (Don) when authentication is successful. | 0/1 |
| Cla | Cause of last authorization | Provides date & time, user and output of last authorization. The text is available until a new message replaces it. Observe privacy laws! | - |
| Ula | User of last authorization | Provides the user's NFC Code Touch ID if available (otherwise username), NFC tag or access code name. The text is available until the next message replaces it. | - |
| Nlo | Name or number of last output | Provides name or number of the last output. The text is available until the next message replaces it. | - |
| Tla | Time and date of last authorization | Provides date and time of last authorization. The text is available until the next message replaces it. | - |
| Ad | Authentication denied | Activates output for the duration of (Don) when authentication is denied. | 0/1 |
| As | Authentication successful | Activates output for the duration of (Don) when authentication is successful. | 0/1 |
| Nco | Number of current output | Number of the current output. -1 = Denied | -1...∞ |
| Bell | Doorbell output | 0/1 |
| Bsel | Doorbell pre-select | A number entered before ringing is displayed here. -1 = Output if no pre-selection was entered. | -1...9999 |
| Unla | Username of last authorization | Returns the username of the last authorized user. Trust users are extended with the host name of the trust member. The text is available until the next message replaces it. | - |
| Uidla | User-Id of last authorization | Returns the User-Id of the last authorized user. Trust users are extended with the host name of the trust member. The text is available until the next message replaces it. | - |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Bbl | Activate bell button light | Activates the doorbell button's green light when 1. Not available for battery powered devices. This parameter is only visible in certain configurations. | - | 0/1 | 0 |
| Bbr | Button brightness | Sets the brightness of the keypad illumination. Not available with battery powered devices. | % | 0...100 | 50 |
| Don | On-duration of outputs (O1-6), (Ad) and (As). | s | 0...∞ | 3 |
| Blan | Activate button light at night | Automatically switches on the button lights at night (Daylight 30min) when 1. Only available with NFC Code Touch Gen.1 devices. This parameter is only visible in certain configurations. | - | 0/1 | 0 |
| Bl | Activate button light | Permanently switches on the button lights when 1. Not available for battery-powered devices. | - | 0/1 | 0 |
| Au | Authentication | Specifies the authentication method used. 0 = Two-Factor Authentication 1 = Code or NFC 2 = NFC 3 = Code 4 = OCPP or NFC Default = 1, or when value outside of range Two-factor authentication requires both an access code and an NFC tag to be presented within 30 seconds, in any order. Tags and codes from different users may be used, provided both are valid. Two-factor authentication is not supported on battery-powered NFC Code Touch Air Gen. 1 devices. | - | 0...4 | 1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Assigned Device | The user must identify themselves on the selected device. The following devices can be selected: NFC Code Touch Air NFC Code Touch Tree NFC Code Touch for Nano | - | - |
| Edit Access Authorisations | Click here to edit access permissions for user-independent NFC-tags/access codes | - | - |
| Allow Additional Preselections | If activated, additional preselections from 7 to 9999 can be used. Activated preselections greater than 6 are output at the output (Nco). With this option selected, a prefix must always be entered before each access code. | - | - |
| NFC Confirms Entry | When an NFC tag is placed on the reader, the pre-selection is automatically confirmed. When two-factor authentication is used, the code can also be confirmed this way. | - | - |
| Number of Entries | Maximum number of saved messages. | 0...100 | - |

---

## Basic Configuration

First, the NFC Code Touch is paired.
The function block is then created automatically when the NFC Code Touch is dragged from the periphery tree to the programming page.

In the following example, output (O1) is used to open the front door. Output (O2) is used to control the garage door. The bell button on the NFC Code Touch activates output (Bell), controlling a door chime:

![AuthNFC blockiosbasic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-blockiosbasic.png)

The outputs names used in the user interface can be set the double-clicking the block or in the settings of the block:

![AuthNFC nameoutputs](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-nameoutputs.png)

---

## Using NFC Tags

### User-assigned NFC tags for access

It makes sense to assign NFC tags to all users that regularly access a building.
To do so, the users must first be created in the [User and Rights Management](https://www.loxone.com/help/usermanagement).

To learn an NFC tag, first click on the function block or the corresponding NFC Code Touch, then click on Learn NFC Tag in the menu bar at the top:

![AuthNFC startlearn](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-startlearn.png)

This will open the Learn NFC Tag window, and a cyan-colored running light on the selected NFC Code Touch lets you know that the device is in NFC learning mode.
Hold the NFC tag directly to the reader, which is marked by the circle symbol at the bottom of the device.

> **ℹ️ Note:** In battery operation, an NFC Code Touch Air must be woken up, before the learning process is started, e.g. by pressing a key.

The status LEDs on the device briefly light up green once the NFC tag is recognised and the tag is displayed in the monitor.
Select the entry in the monitor, name the tag and assign it to a user:

![AuthNFC learnusertag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-learnusertag.png)

The NFC tag can then be handed over to the user.
Repeat this process for the NFC tags of the other users.

> **ℹ️ Note:** Only use encrypted NFC/RFID tags for access control. These tags, e.g. the Loxone Smart Tags and Key Fobs are equipped with a MIFARE® DESFire® chip.

In addition, users need to have the right for the Authentication NFC Code Touch block in order to gain access.

When the learned tag is presented to the reader, output (O1) is activated by default. In order to activate outputs (O2-6), the corresponding preselection number 2-6 must first be entered on the NFC Code Touch. Any output of the function block can be activated using this method.

Access can be restricted by creating access schedules in the [User Management](https://www.loxone.com/help/usermanagement).

The user-assigned tags are valid at all NFC Code Touches in the project.

User-assigned NFC tags can also be learned via User Management in the user interface. This requires the additional right for User Management and for the Authentication NFC Code Touch block.

### NFC tags not assigned to a specific user

If an NFC tag is not to be assigned to a specific user, assign the tag to an NFC Code Touch instead:

![AuthNFC learndevicetag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-learndevicetag.png)

This option can be used to grant temporary or permanent access to visitors.
Once learned, these tags will be listed in the NFC tags tab of the function block.

You can define which output is to be activated by default and set a validity period.
Authorization can also be limited to certain outputs via the tick boxes:

![AuthNFC devicetagslist](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-devicetagslist.png)

In the above example, Tag 1 is authorized to activate any output of the function block, output (O1) being the default when no preselection is made. To activate outputs (O2-6), preselection is required.
Tag 2 activates output O2 by default. When preselecting the number 3, output (O3) is activated. Tag 2 is not authorized to activate the remaining outputs.

This configuration is only suitable to a limited extent for regular access, since user-specific functions such as access schedules are not available.

Also, the tag is only valid on the NFC Code Touch to which it was assigned.

### NFC Tag used as input

NFC tags can also be used directly as a digital input. They are created in the peripheral tree of the NFC Code Touch and can be used for any application:

![AuthNFC learninputtag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-learninputtag.png)

![AuthNFC displayinputtag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-displayinputtag.png)

The Authentication NFC Code Touch block is not required to use these inputs.
This configuration is not recommended for access control.

---

## Read/Format NFC Tags

In order to read or format an NFC Tag, select the NFC Code Touch, click on "Read NFC Tag" or "Format NFC Tag" and follow the instructions:

![Tags ReadFormat](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Tags_ReadFormat.png)

### Reading NFC Tags

When reading NFC Tags, you can view the Tag name, the assigned user, and the NFC ID of the Tag.

Additionally, you can see on which Miniservers the NFC Tag is currently in use.

If there are any errors related to the NFC Tag, they will also be displayed on this screen.

![Read NFCTag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Read_NFCTag.png)

### Formatting NFC Tags

Formatting an NFC Tag removes all data, including encryption keys (App ID and App Key, if the NFC was encrypted) and permissions.
After formatting, the NFC Tag is reset to an empty state. A new NFC ID is generated only after relearning the Tag.

Formatting is recommended when reusing a Tag or when permissions must be fully cleared.
It is not mandatory when learning the Tag on another Miniserver for another user but advised to avoid permission conflicts.

### Information about NFC ID & App ID & App Key

**NFC ID**

The NFC Code Touch generates a unique ID and encrypts it before storing it on the Tag.
In Loxone Config, the NFC ID ends with EC.
The actual authentication ID excludes this EC suffix.

Example:
- NFC ID: 0F B9 B6 D2 06 54 03 1F EC
- Authentication ID: 0F B9 B6 D2 06 54 03 1F

**App ID & App Key**

These are used when encrypting/decrypting an NFC Tag.
- App ID: Name of the file written to the NFC Tag.
- App Key: Key which is used to decrypt the content of the file that was written to the NFC Tag

Manual changes to App ID or App Key will cause authentication to fail.

---

## Using numeric codes

### User-specific numeric access codes

A 2-8 digit numeric access code can be assigned to each user.
To do so, open the Authentication dialog in the properties of a user under Security:

![AuthNFC openuserauth](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-openuserauth.png)

In the New... field under Key Code enter a code for the user and confirm with OK:

![AuthNFC setuserkey](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-setuserkey.png)

The code can then be communicated to the user.
Repeat this process for the codes of the other users.

> **ℹ️ Note:** The same rules that apply to passwords also apply to access codes: as long as possible, as difficult to guess as possible.

In addition, users need to have the right for the Authentication NFC Code Touch block in order to gain access.

When the code is entered on the NFC Code Touch, output (O1) is activated by default. In order to activate outputs (O2-6), the corresponding preselection number 2-6 must first be entered, followed by the checkmark and then the code. Any output of the function block can be activated using this method.

Access can be restricted by creating access schedules in the [User Management](https://www.loxone.com/help/usermanagement).

The user-assigned codes are valid on all NFC Code Touches in the project.

User-assigned codes can also be created via User Management in the user interface.

### Numeric codes not assigned to a specific user

If a code is not to be assigned to a specific user, assign the code to an NFC Code Touch instead.
This option can be used to grant temporary or permanent access to visitors.

These codes can be added in the Access Codes tab of the Authentication NFC Code Touch function block, either in the properties or by double-clicking on the block.
Click on the Add button and enter the code:

![AuthNFC devicecodelist](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-devicecodelist.png)

The default output, authorisation for specific outputs and the validity period can also be set here, as previously described for the NFC tags.

This configuration is only suitable to a limited extent for regular access, since user-specific functions such as access schedules are not available.

Also, the code is only valid on the NFC Code Touch to which it was assigned.

A new code can also be assigned to the NFC Code Touch via the user interface of the function block.

### Numeric code used as input

Numeric codes can also be used directly as a digital input. They are created in the peripheral tree of the NFC Code Touch and can be used for any application:

![AuthNFC setupinputcode](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AuthNFC-setupinputcode.png)

The Authentication NFC Code Touch block is not required to use these inputs.
This configuration is not recommended for access control.