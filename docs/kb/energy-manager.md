# Energy Manager Gen. 1

Source: https://www.loxone.com/enen/kb/energy-manager/

---

With this function block, surplus energy, which is otherwise fed into the grid, can be used optimally. Up to 12 loads can be controlled.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Functionality](#functionality)
- [Activation time](#activationtime)
- [Calculation cycle](#calcCycle)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| P | Current power | kW | ∞ |
| Ps | Power energy storage | Must be connected if output (Re) is used. | kW | ∞ |
| Prio | Priority selection | Starts the selected load immediately. | 0/1 | 0...12 |
| L1-12 | Start Load 1-12 | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| L1-12 | Load 1-12 | - | 0/1 |
| Re | Residual energy | Can be used, for example, to charge a battery or a heating element (energy storage). If this output is used, the input (Ps) must be connected. | kW | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| O | Offset available energy | Only so many consumers are activated that the current power (P) does not exceed this value. 0 = Outputs are activated only with excess energy. | kW | ∞ | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Properties | Edit energy manager | - |

---

## Functionality

Priorities are assigned to the connected loads (1: high, 12: low). If the input (P) is indicating available surplus energy, the energy manager will switch as many devices on as possible, starting with the highest priority.

The loads are active for at least the minimum switch-on duration and inactive after switching off for at least the minimum switch-off duration. If a daily minimum switch-on duration is specified, the corresponding devices are still activated for this duration before sunset or the user-defined time (even without a current surplus).

If input (P) is no longer indicating any surplus energy, then devices are switched off again. A device that has already reached it's minimum run-time may also be switched off to give other devices the ability to achieve their minimum run-time requirements.

The inputs (L1-12) and (Prio) can be used to override this behavior and enable the manual control of devices. When these inputs are triggered by a short pulse (< 1s), the input remains active until the minimum run-time per day is reached. If this time is already reached, the output is activated at least for the minimum run-time. With a continuous signal (> 1s), the output is deactivated immediately after the falling edge (L) or a change in value (P), the minimum run-time is ignored.

---

## Activation time

If an activation period is configured for a load, the corresponding (Start Load) input must be activated before the load can be switched on.

The load is switched on for the activation period in order to be able to carry out necessary preparatory work (e.g. selecting a washer cycle). If there is no energy surplus at the end of the activation period, the load is deactivated again.

From this point on, the load is active and now operates as if no activation time was configured. It can be controlled both automatically based on priority and by manually (input L or Prio).

The status of the load is reset to inactive at the end of the day. Manual resetting is possible via the (Off) input.

---

## Calculation cycle

To avoid constant switching of the outputs and to allow time for input (P) to react, the value at (P) is only evaluated at a maximum rate of once per minute.