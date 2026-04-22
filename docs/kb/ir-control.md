# IR Controller

Source: https://www.loxone.com/enen/kb/ir-control/

---

The IR Controller is used for complex IR codes, e.g. buttons that send a different IR signal every time the button on the remote control is pressed.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Structure of an IR command](#IRstructure)
- [Creating an IR command](#IRcreate)
- [Programming example](#baseconf)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| I1-6 | Input 1-6 | ∞ |

---

## Outputs

> **ℹ️ Note:** Text

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| IR Command | In conjunction with the value received on the inputs this string is used to create the message that is sent via the IR actuator. Refer to help for further information. | - |

---

## Structure of an IR command

In an IR command the different command blocks are separated by a space, therefore no space may be used within the command blocks.
The command sequence may have a maximum length of 256 bits.
The start bit and end bit are omitted for the IR command.

### Value block

With this block the command sequence can be influenced by the inputs. The keywords "V1-V6" are used to represent the inputs "I1-I6". Bit sequences for different input values can be defined for an input in a value block.
For example, a value block starts with the keyword "V1" and is separated from the value definitions by a colon ":". This is followed by a list of value definitions separated by a semicolon ";". A value definition consists of the input value and the corresponding bit sequence it represents, separated by a colon.
Decimal numbers are specified with a separating point (e.g. 16.5).
Example:
Value block: V1:0:0000;1:0001;2:0010;3:0011;4:0100; => If value 3 is present at I1, bit sequence 0011 is used for this value block.

### Constant block

This block can be used to define a constant bit sequence in the command sequence. A constant block begins with the keyword "C" and is separated from the corresponding bit sequence by a colon ":".
Example:
Constant block: C:0101101000001 => The static bit sequence 0101101000001 is used in the command sequence.

### Checksum block

This block can be used to insert an 8-bit checksum in the command sequence. A checksum block starts with the keyword "S" and is separated from the definitions by a colon ":". This is followed by the checksum method to be used, either of two common checksum methods for IR commands (S:1 or S:2) can be selected. The checksum is calculated from the entire command sequence.

### Inverter block

This block can be used to copy and invert bit strings in the command sequence. An inverter block starts with the keyword "I" and is separated from the definitions by a colon ":". This is followed by the relative position of the bits to be copied and separated from the number of bits to be inverted by a semicolon ";".
Example:
Inverter block: I:8;4 => The Inverter will count back 8 positions and use 4 bits in inverted form for the command sequence.
IR Command: C:0101101000001 I:8;4 => The part that is copied corresponds to "0100" here, this is inverted ("1011") and added to the command sequence. In this example, the command sequence results in "01011010000011011".

---

## Creating an IR command

To create an IR command for a remote control it is necessary to know its command structure. You can obtain the structure either directly from the manufacturer of the device or by analyzing the transmitted commands.
In most cases, the IR control itself is required for air conditioners. These remote controls often have a display that shows the current state of the air conditioner. On these devices, with each change, the entire status is sent to the air conditioner via an IR command.
In order to be able to understand the structure of this command, it is recommended to record several commands of the remote control and note which status was active for each command. Based on this information, it is possible to determine the meaning of each bit sequence in the IR command sequence. The transmitted command sequence can be seen in Loxone Config. Once the exact structure of the sequence is known, it can be converted into an IR command. It is recommended to determine which functions of the device are actually needed to control it. Any special commands can be ignored in most cases.

![InfraredData](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/InfraredData.png)

---

## Programming example

### Hitachi air conditioner

The command sequence for the Hitachi air conditioner is entered in the properties window of the IR controller:

![IR hitachi](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/IR_hitachi.png)

![IR hitachi example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/IR_hitachi_example.png)

### Mitsubishi air conditioner

The command sequence for the Mitsubishi air conditioner is entered in the properties window of the IR controller:

![IR mitsubishi](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/IR_mitsubishi.png)