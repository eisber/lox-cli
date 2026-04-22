# Miniserver Shortcut

Source: https://www.loxone.com/enen/kb/miniserver-shortcut/

---

The Miniserver Shortcut function block enables connection to another Miniserver within the user interface.

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Link trusted Miniserver | Link a Trust Member instead of manually entering the address and serial number. | - |
| Linked Trust Miniserver | Select an already joined Trust Member. | - |
| Miniserver serial number | - |
| Miniserver local address | - |
| Miniserver external address | - |

---

## Programming example

If access to logic located on another Miniserver is needed within the user interface, the Miniserver Shortcut function block can be used.
There are two ways to link this block to a Miniserver:

### Via selecting a Trust Miniserver

If the Miniserver has already joined a [Trust](https://www.loxone.com/help/trusts), the option "Link trusted Miniserver" can be selected in the properties window, and the desired Trust Miniserver can be chosen:

![MiniserverShortcut trust](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/MiniserverShortcut_trust.png)

### Via entering the Serial number and address

In the properties window, a Miniserver can be linked via the serial number and the local or external address:

![MiniserverShortcut local](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/MiniserverShortcut_local.png)

In the user interface, you can now connect to the linked Miniserver within the category or room.
The user interface attempts to use existing saved access data, if none are found or they have expired, the user must enter their username and password.

![MiniserverShortcut ui](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/MiniserverShortcut_ui.png)