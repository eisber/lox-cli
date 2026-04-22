# Status

Source: https://www.loxone.com/enen/kb/status/

---

You can use this function block to show user-defined status texts and symbols in the User Interface
In addition, this can be used for decision logic.
Double-click on the program block to open the editing window.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Possible operating modes](#config)
- [Compare Inputs](#compare)

---

## Inputs

> **ℹ️ Note:** Input 1-8

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Txt | Current status text | The current status text resulting from the conditions specified in the editing window of the block. | - |
| Val | Current status value | The current status value resulting from the conditions specified in the editing window of the block. | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Status | Status-text | - |

---

## Possible operating modes

![State Menu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/State_Menu.png)

== : selected input is the same as the value

    > : selected input is greater than the value

    >= : selected input is greater than or equal to the value

<: selected input is less than the value

<=: selected input is less than or equal to the value

    != : selected input is not the same as the value

*= : selected input contains the value

    !* : selected input does not contain the value

: : selected input starts with the value

    !: : selected input does not start with the value

---

## Compare Inputs

Comparing inputs is possible by writing e.g. <v1>in the value field (similar to the Status-text field):

![status compareInputs1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/status_compareInputs1.png)

Comparing text inputs with numerical operators (<, <=, >, >=) is not supported.