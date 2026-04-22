# Lighting Groups

Source: https://www.loxone.com/enen/kb/lighting-groups/

---

With Lighting Groups, several lights can be combined into a group and controlled together. The programming becomes clearer and fewer commands have to be sent for control.

Only Loxone Tree lights with Smart Actuator are supported. RGBW dimmers, Air lights and standard actuators are not supported.

The auto configuration automatically inserts corresponding lighting groups and assigns them to the appropriate devices.

## Table of Contents
- [Programming example](#baseconf)

---

## Programming example

Select "Lighting Groups" in the periphery tree and add a lighting group via the menu bar. It can be edited in the properties.

![lightgroup create](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/lightgroup_create.png)

![lightgroup edit](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/lightgroup_edit.png)

The lighting group can be assigned in the properties of the respective devices:

![lightgroup dev](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/lightgroup_dev.png)

The assigned devices are listed under the lighting group in the periphery tree.
The lighting group can be dragged onto the programming page and connected to the respective function block.
All lights in this group are switched together.

![lightgroup example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/lightgroup_example.png)