# Sequence Controller

Source: https://www.loxone.com/enen/kb/sequential-controller/

---

The Sequence Controller function block allows programming function sequences in text form.
Various processes that require sequential control can be automated this way.

The function block can simplify these processes, which often require several conventional function blocks with complex programming.

The individual steps are programmed line by line, a group of lines that performs a specific function is referred to as a sequence.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Application Example](#baseconf)
- [Functional Description](#function)
- [Operators, Inputs/Outputs (IOs) and Variables](#operators)
- [Commands](#commands)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| S1-8 | Activate sequence 1-8 | 0/1 |
| AI1-8 | Inputs 1-8 | ∞ |
| S | Select sequence | 0...8 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| AQ1-8 | Outputs 1-8 | ∞ |
| S | Current active sequence | ∞ |
| L | Current program line | ∞ |
| TQ | Text output | Output can be used by sequences. | - |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Configuration | Program code editor | - | - |
| Interval[ms] | Interval from line to line in milliseconds. Influences the processing speed of sequences. Low values increase the processing speed, but can also increase the CPU load. | 20...1000 | 500 |

---

## Application Example

The following example shows a simple application for domestic water heating, in which two hot water tanks are heated consecutively by a central heater.

The temperatures of the heater, the two hot water tanks and the temperature setpoint for the hot water are provided at the function block's inputs.
Sequence 1 is started via the (S1) input, in this example by a pulse at a specific time.
The heater, pumps and valves are controlled via the outputs:

![SeqCtrl Ex1 IOs](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SeqCtrl-Ex1-IOs.png)

Double-click on the function block to open the programming window.
The library on the left side of the window lists the available inputs, outputs, commands and variables.
Double-clicking on one of these objects will insert it into the editor window on the right where it can be further customized:

![SeqCtrl Ex1 sequence](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SeqCtrl-Ex1-sequence.png)

Each line must begin with a command, as can be seen in the example above.
The green comments are not required for the sequence to function, but allow to document the individual steps.

Note: This example is intended to provide an overview of the function block and its functionality. It can be applied to a specific application only to a limited extent.

The functionality of a sequence can be tested in Simulation / Liveview. The current active line is marked yellow in the programming window. For Liveview, the whole document must be identical with the Miniserver.

---

## Functional Description

Sequences are executed line by line at an adjustable interval. Specific sections can be skipped using conditions or delayed by time commands.
It is possible to create multiple sequences, each sequence can be started individually or switched to using a command.

The function block also includes a syntax check, which highlights syntax errors while editing a sequence. Invalid or empty lines are skipped when the sequence is running.

A programming line always starts with the desired function, followed by the required parameters and operators, each separated by a space.

The inputs and outputs specified in the library of the block can be set or queried.
If an input is connected to logic, the last set value is valid. Changes to the input overwrite the value set by a sequence.

The commands supported by the [Formula](https://www.loxone.com/help/Formula) function block can be used to set and compare outputs(e.g. SET AQ1 = AI2 * 3).

---

## Operators, Inputs/Outputs (IOs) and Variables

Available operators are =, >=, >, <=, <, !=

Available inputs: (AI1 - 8). The input names can be customized in the dialog.

Available outputs: (AQ1 - 8). The output names can be customized in the dialog.

Variables: value1 - value5. The variable names can be customized in the dialog.
Variables are used to process values that are not available at the IOs. They can be set like IOs, used in calculations or as a memory.

---

## Commands

**Sleep**
Wait for the specified time
Syntax: sleep [value] [s: seconds, m: minutes]
Examples:
sleep 300 s (waits for 300 seconds, then proceeds with next line)
sleep 10 m (waits for 10 minutes, then proceeds with next line)

**Waitcondition**
Wait for condition to be met
Syntax: waitcondition [IO] [operator] [value]
Examples:
waitcondition AI1 > AQ1 (waits until AI1 is greater than AQ1, then proceeds with next line)
waitcondition AI1 + 3 > value1 (waits until AI1 + 3 is greater than the variable value1, then proceeds with next line)

**Set**
Set an input, output or variable to a value
Syntax: set [IO] = [value]
Examples:
set AQ1 = AI1 - AI2 (subtracts AI2 from AI1 and outputs the result on AQ1)
set value1 = AI2 + AI3 (adds AI2 and AI3 and sets the variable value1 to the result)

**Setpulse**
Pulses an input or output
Syntax: setpulse [IO]
Examples:
setpulse AQ1 (provides a short pulse to AQ1)
setpulse AI2 = 4 (provides a short pulse with value 4 on AI2)

**Startsequence**
Starts a specific sequence
Syntax: startsequence [number] or startsequence [name of sequence]
Examples:
startsequence 2 (starts the second sequence)
startsequence Alarm (starts the sequence named “Alarm”)

**Return**
If the current sequence was started by another sequence with [startsequence], this command returns to the calling sequence and proceeds to its next line
Syntax: return

**Goto**
Go to specific line
Syntax: goto [value]
Examples:
goto 1 (starts the sequence from the beginning)
goto 23 (jumps to line 23 and continues from there)

**If**
"If" checks whether a condition is true first. Only if the condition is true, the subsequent commands up to "endif" are executed.
Syntax: if [left term] [operator] [right term] ... endif
Example:
if AQ3 >= 4
set AQ1 = AQ3
endif

### Comments and Texts

**Comment**
Add a comment to the current line or disable the rest of the line
Syntax: //
Examples:
waitcondition AI2 > AI4 // wait until AI2 is greater than AI4 (comment is used to describe the line)
// waitcondition AI2 > AI4 (comment slashes are used at the beginning to disable the whole line)

**Text output TQ**
Characters in quotation marks are output as text, otherwise they are handled like analog values
Syntax: set TQ = "..."
Examples:
[set TQ = AQ2] will output "27.5"
[set TQ = "Value of AQ2 is" AQ2] will output "Value of AQ2 is 27.5"