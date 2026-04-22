# Tracker

Source: https://www.loxone.com/enen/kb/tracker/

---

The tracker is used to display events in the user interface.

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)
- [Assign logger, mailer, caller, tracker in the properties window](#AssignMessProp)

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 2...100 | - |
| Remanence | Keep the entries across the restart | - | - |

---

## Programming example

First, a new tracker is created under Messages:

![tracker new](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tracker_new.png)

In the following example we give the tracker the name "Access House" and drag the object onto the programming page.

In the properties window, a text can be defined for the two states of digital values or at analog value changes, or values of analog or text outputs can be written to the tracker with <v>.
The maximum number of entries in the tracker can also be set.

If the house is now entered in our example, this is displayed in the user interface with time stamp, tracker name and the ID or name of the respective person, which is output from the text output of the authentication block.

![tracker example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tracker_example.png)

![tracker visu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tracker_visu.png)

---

## Assign logger, mailer, caller, tracker in the properties window

Alternatively, in the properties window of various blocks, logger, mailer, caller and tracker can be assigned.
For this, the respective message must only be created and the recipient defined.
The text or value defined in the block is output.

![message aal linked](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/message_aal_linked.png)