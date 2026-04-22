# Retractive Switch

Source: https://www.loxone.com/enen/kb/retractive-switch/

---

## APPLICATION

The Retractive Switch function block can be used to generate pulses via the visualization (user interface) and via logic.

## BASIC CONFIGURATION

![Retactive](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/08/Retactive.png)

A pulse at the output (Q) can be generated via the button in the visualization or via a preceding logic. The duration of the pulse is determined by parameter T, pressing the button in the user interface or having another trigger input (Tr) will reset the timer.

When a reset signal (R input) is used, the output is switched off immediately. A continuous signal on R has priority over Tr.

If a signal input is in the input Dis, the other inputs Tr and R are ignored. However, the button in the visualization.

## INPUTS

| Input | Designation | Description |
| --- | --- | --- |
| Tr | Trigger | Button input Sets the output to ON and starts the timer |
| R | Reset | Resets the output (Q). |
| Dis | Disable | Locks all inputs. The module can still be operated via the visualization. |

## OUTPUTS

| Output | Designation | Description |
| --- | --- | --- |
| Q | Digital output | Digital output is activated when the button is pressed and deactivated when the time T has elapsed |

## PARAMETERS

| Parameter | Designation | Description |
| --- | --- | --- |
| T | Time | Duration of the output pulse |