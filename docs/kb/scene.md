# Scene

Source: https://www.loxone.com/enen/kb/scene/

---

A scene contains an action list that is executed with the object input or via the user interface.

A scene is very similar to the [automatic rule](https://www.loxone.com/help/autorule), one or more actions can be selected.
However, no conditions can be defined for a scene. The actions in the scene are only executed if the scene is triggered via the (Act) input or in the user interface.

Scenes are displayed in the user interface, and can be created, edited or deleted via the menu.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Application Example](#example)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Act | Activate scene | 0/1 |
| Off | Off | On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description |
| --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Configuration | Display configuration | - |

---

## Application Example

The following example shows a scene that triggers a movie mode in the living room.
For this, the actions Switch on TV, Lower shading, and the Theater lighting mood that already exist in programming are selected:

![scene exampleconfig](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/scene_exampleconfig.png)

Next, close the window, assign a name, room and category to the scene, and save the program to the Miniserver.

In the user interface/App, the scene is then displayed in the assigned room and can be triggered by pressing the button:

![scene exampleapp](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/scene_exampleapp.png)

The scene can also be opened, allowing the actions to be viewed and edited:

![scene editapp](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/scene_editapp.png)

All existing scenes are displayed under Scenes in the user interface/App menu. There you can also create new scenes or edit existing ones.