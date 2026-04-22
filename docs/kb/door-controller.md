# Door Controller

Source: https://www.loxone.com/enen/kb/door-controller/

---

Door controller with door bell and camera
The door control uses two separate methods for audio and video streaming, so it is possible to use a video only, audio only, or a combined audio / video stream for your intercom / camera.
First, create an intercom to be selected in the properties of the door controller as the linked intercom
To do this, select the item ''Network devices'' in the periphery and then insert an intercom using the ''Add network device'' button
If you are using a Loxone Intercom, you can also add it via ''Network Device Search''

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Image when ringing](#ring)
- [Supported IP cameras](#IPCameras)
- [Loxone Intercom](#Intercom)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Bell | Activate bell | Activates output (Bell) and assigned audio players. If a Loxone Intercom is used, it is not necessary to connect this input. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Bell | Bell | Output for controlling a doorbell. | 0/1 |
| O1 | Custom output 1 | Output is named in the settings and triggered from the user interface. | 0/1 |
| O2 | Custom output 2 | Output is named in the settings and triggered from the user interface. | 0/1 |
| O3 | Custom output 3 | Output is named in the settings and triggered from the user interface. | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| maxB | Maximum bell duration | Output (Bell) remain active for set duration. Answering or rejecting the call deactivates the output (Bell) immediately. If set to 0, the bell will remain active until its answered in the visualization. | s | 0...∞ | 60 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Assigned Intercom | This is where you can assign the door controller to an Intercom. (Note: The Intercom is listed as an object in the periphery tree) | - |
| O1: User Interface Action 1 | Name for function 1 e.g. door release | - |
| O2: User Interface Action 2 | Name for function 2 e.g. outside light | - |
| O3: User Interface Action 3 | Name for function 3 e.g. Inside Light | - |
| Configuration | Configuration of the Inputs and Outputs Used. | - |
| Show video stream when doorbell rings | Shows the video stream in the user interface instead of a snapshot, when the doorbell rings. Disable this option if the Intercom can’t manage video streaming to multiple user interfaces at once. | - |

---

## Image when ringing

If the bell rings and no one answers, an image is automatically saved. Via the user interface, you can view the last 10 images. If the memory of your Miniserver is already quite full, the Miniserver might automatically cancel the download of the image. There is no limit to the maximum size of an image, this depends on the free memory of the Miniserver.
The "Digest Authentication" is not supported.

---

## Supported IP cameras

To integrate IP cameras, the requirements listed in the following document must be met:

[**Camera Requirements for User Defined Intercoms**
](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Requirements_UserDefinedIntercoms.pdf)

[Loxone Intercom Gen. 1](https://www.loxone.com/enen/kb/intercom-intercom-xl/)

**Mobotix T24/T25**
http://ip:port/cgi-bin/faststream.jpg?stream=html&fps=16

---

## Loxone Intercom

Learn more about the [ Loxone Intercom ](https://www.loxone.com/enen/kb/intercom-intercom-xl/)