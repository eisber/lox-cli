# Caller

Source: https://www.loxone.com/enen/kb/caller-service/

---

Using the Caller Service the Miniserver places a call on ON/OFF/analog value change and plays the predefined text as a voice message.

For the services (Weather, Caller) the Miniserver must be [registered](https://portal.loxone.com/products) first.
After the services have been purchased in the [shop](https://shop.loxone.com/), they can be activated in the Loxone Partner Portal. The license number can be found on the invoice.
A valid internet connection is required for these services.

Each purchased Caller Service is assigned its own unique telephone number.

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)
- [Add user-defined Caller](#addCaller)
- [Assign logger, mailer, caller, tracker in the properties window](#AssignMessProp)
- [Limitations](#limit)

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Phone Number | Phone number to call Numbers only inc. country code, no spaces or special characters (eg 00441183130140) | - |
| Language | Language of entered text | - |
| Allow Response | When this box is checked, the dial key input corresponding to the number pressed by the callers phone is activated. For this, the Miniserver must be accessible externally. The dial key inputs can only be dialled for 2 minutes from the start of the call! | - |
| Monitor Service | If checked, you will be notified via System Status or Cloud Mailer if this service subscription has expired or is about to expire. | - |

---

## Programming example

In the Config, there are already placeholders for task forces and emergency contacts under "Messages":

![caller placeholder](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/caller_placeholder.png)

Each caller has 10 dial key inputs with which the callee can answer the call:

![caller DialKeyInputs](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/caller_DialKeyInputs.png)

Drag the caller to the programming page and link it to the desired function block. If the caller is selected, a text, name and phone number can be entered in the properties window.
With "Allow response" it is possible to respond to the call for 2 minutes from the beginning of the call by pressing a key. To use the feedback function, the Miniserver must be [accessible externally](https://www.loxone.com/help/remote-connect).

![caller properties](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/caller_properties.png)

If the alarm is triggered in our example, emergency contacts 1 and 2 are called. The callees can acknowledge the alarm by pressing key 1.

![caller example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/caller_example.png)

---

## Add user-defined Caller

To add a Caller, click on "Messages" in the periphery tree and select "Caller" in the menu bar:

![caller new](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/caller_new.png)

Now, the created caller can be defined and linked to the desired function block.

---

## Assign logger, mailer, caller, tracker in the properties window

Alternatively, in the properties window of various blocks, logger, mailer, caller and tracker can be assigned.
For this, the respective message must only be created and the recipient defined.
The text or value defined in the block is output.

![message aal linked](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/message_aal_linked.png)

---

## Limitations

The Caller Service is limited to 10 calls to the same number per minute.
Calls exceeding this limit are blocked.

Calls can be placed again once the number of calls to the same number in the last minute drops below 10.
Each call counts towards this limit, even if it was blocked.

The maximum duration of the call is approximately 40 seconds.