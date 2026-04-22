# Automatic Rule

Source: https://www.loxone.com/enen/kb/automatic-rule/

---

The Automatic Rule block offers a way to create programming that is not only defined in Loxone Config, but can also be edited by users in the app.

This is useful when programming automation that may or may not meet the user's preferences and may need to be adjusted at a later time.

Objects or events are selected as conditions, logically linked to each other, and later used to trigger an action.

The objects used in the automatic rule have to exist already.
These could be manually added objects like function blocks in various rooms, but also basic objects that exist in every program like the time.

Every rule is listed in the Automatic Designer menu in the Loxone App and can be activated, deactivated or edited.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Basics](#basics)
- [Application Example](#example)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle | Enable / disable automatic rule | 0/1 |
| Off | Disable automatic rule | 0/1 |
| On | Enable automatic rule | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| E | Enabled | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Configuration | Display configuration | - |
| Notification when executing | Show notification in the App when action list is executed. | - |

---

## Basics

To create a rule, first add the Automatic Rule block in Loxone Config, either by selecting it from the menu bar or by pressing **F5** and entering it in the search field:

![autorule insertblock](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule insertblock.png)

A double click on the block opens the window where a rule can be created:

![autorule add condition](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule add condition.png)

By **adding condition** you can specify events that will trigger the automatic rule.
Events like times, functions, weather data or operating modes can be used.
Functions include objects like blocks, inputs or outputs, and are navigated by rooms and categories.
One or more conditions can be selected and linked using logic **and** or **or** operators.

**Actions** define which actions will be executed by the automatic rule when the specified conditions are met.
Operating modes, functions, messages or program flows are available.
One or more actions can be selected.

**History** lists an automatic rule whenever it was executed.

For users to be able to edit an automatic rule in the Loxone App, they must be granted the necessary right in [User Management](https://www.loxone.com/enen/kb/user-and-rights-management):

![autodesign rights](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autodesign rights.png)

---

## Application Example

In the following example, an automatic rule is created that fully opens the east facing kitchen blinds at sunrise, but only if the outdoor temperature is low enough.

We add the block and assign a name for the automatic rule:

![autorule block name](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule block name.png)

After double-clicking on the block we select **Add condition**, then under Times - Time of day we select the Pulse at sunrise and add it to the conditions with the plus button:

![autorule add sunrise](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule add sunrise.png)

![autorule sunrise added](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule sunrise added.png)

Then we click on **AND IF**, select Weather and then Temperature. Here we specify at which temperature the condition will apply:

![autorule add temp](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule add temp.png)

Next, we also add this condition with the plus sign. Now the two objects sunrise AND the set temperature are linked as a condition:

![autorule conditions linked](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule conditions linked.png)

Then we switch to **Add action**, and navigate to Functions - Rooms - Kitchen and select the Automatic Shading East block:

![autorule choose shading](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule choose shading.png)

There we select Open shading and add this to the actions with the plus button:

![autorule add blinds open](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule add blinds open.png)

![autorule shades added](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autorule shades added.png)

Now that we have created the automatic rule, we can close the window and save the program to the Miniserver.

The created rule can now be viewed and edited in the Automatic Designer in the Loxone App:

![autodesign rule edit](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autodesign rule edit.png)