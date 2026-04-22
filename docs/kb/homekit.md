# HomeKit

Source: https://www.loxone.com/enen/kb/homekit/

---

With Apple HomeKit many functions of a Loxone installation can be controlled directly from compatible iOS devices.

This also enables voice control with Apple’s Siri.

![workswith homekit icon259](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/11/workswith-homekit-icon259.png)

Current Miniserver / Go required, not supported by Gen 1. variants!

#### CONTENT

[Setup](#setup)

[Operation](#operation)

[Supported Functions](#functions)

## Setup

The Miniserver requires at least version 12.2.12.1 and an internet connection for the initial setup.

In the project properties of Loxone Config [auto update](https://www.loxone.com/enen/kb/installing-updates/#autoupdate) must be activated.

The Loxone App must be updated to the latest version.

First, connect to the Miniserver locally on an iOS device in the [Loxone App](https://www.loxone.com/enen/products/apps/) with a [user with full access](https://www.loxone.com/enen/kb/user-and-rights-management/), and open the settings menu:

![LoxoneApp settings](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/LoxoneApp_settings.png)

Select “Apple HomeKit” and follow the setup wizard in the app.

					 In case of difficulties, restart the Miniserver and the Loxone App, and try again in a few minutes.

![AHK connect](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_connect.png)

![AHK myHome](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_myHome.png)

A setup code is generated, which is needed in the next step.

![AHK code share](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_code_share.png)

The button “Share HomeKit setup code” creates a PDF file with a QR code.

![AHK setup qr](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_setup_qr.png)

Scan the QR code or enter the setup code manually.

					 While only one Apple account can be connected to the Miniserver, it is possible to share it with additional Apple users [by inviting them](https://support.apple.com/HT208709).

![AHK Code MoreOptions](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_Code_MoreOptions.png)

In the next step, you can specify a user whose functions HomeKit will have access to.

![AHK access](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_access.png)

All supported objects in the programming can be added to Apple Home.

![AHK functions](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_functions.png)

Afterwards the added functions are ready for use.

![AHK successfulConfiguration](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_successfulConfiguration.png)

In the Loxone App settings the added functions can be edited.

					 Up to 149 objects per HomeKit connection are supported.

![LoxoneApp HK](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/LoxoneApp_HK.png)

After a successful configuration, an Internet connection is no longer mandatory on the Miniserver.

## Operation

The previously selected functions can be operated in the Apple Home app.

![AHK home](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/03/AHK_home.png)

### Example with Siri Speech Recognition:

With “Hey Siri, turn on the light relaxing in the living room” the lighting mood relaxing is activated. Siri gives the feedback “OK, the relaxing is on.”

And with “Hey Siri, turn off the light relaxing in the living room” the mood is switched off again.

### Control Functions Remotely

Apple Home functions can be controlled remotely with additional[ Apple devices and iCloud](https://support.apple.com/guide/iphone/control-your-home-remotely-iph1d10f7f2b).

## Supported Functions

The following functions are supported by HomeKit:

[Lighting Controller](https://www.loxone.com/enen/kb/lighting-controller/)

[Scene](https://www.loxone.com/enen/kb/scene/)

[Automatic Shading](https://www.loxone.com/enen/kb/automatic-blinds/), [Automatic Shading Integrated](https://www.loxone.com/enen/kb/automatic-blinds-integrated/) and [Roof Window Shading](https://www.loxone.com/enen/kb/skylight-blinds/)

[Room Ventilation Controller](https://www.loxone.com/enen/kb/ventilation/)

[Intelligent Room Controller](https://www.loxone.com/enen/kb/intelligent-room-controller/)

[Audio Player](https://www.loxone.com/enen/kb/audio-player/) (On/Off)

[Switch](https://www.loxone.com/enen/kb/switch/)

[Stairwell Light –](https://www.loxone.com/enen/kb/stairwell-light-switch/) and [Comfort Switch](https://www.loxone.com/enen/kb/multifunction-switch/)

[Virtual Input](https://www.loxone.com/enen/kb/virtual-inputs-outputs/) as a Switch

					Blocks with a visualization password are not supported!
[↑ back to the top](#top)