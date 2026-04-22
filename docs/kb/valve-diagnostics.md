# Valve Diagnostics Gen 1

Source: https://www.loxone.com/enen/kb/valve-diagnostics/

---

The following messages can be received from a Loxone Actuator [Air](https://www.loxone.com/enen/kb/valve-actuator-air/)/[Tree](https://www.loxone.com/enen/kb/valve-actuator-tree/) via [System Status](https://www.loxone.com/enen/kb/systemstatus/) or via the diagnostics inputs of the device:
- [No Valve Detected](#no-valve)
- [Valve Stuck](#valve-stuck)
- [Mechanical Defect](#valve-hardware)

## No Valve Detected

This message appears if the Valve Actuator Air does not detect a valve during recalibration. The recalibration is performed automatically each time the actuator is restarted (e.g. After power is disconnected).

Make sure that the actuator is correctly mounted on a valve and that the valve pin can be moved. Then perform a recalibration by clicking on Recalibrate Device.

## Valve Stuck

This message appears when the Valve Actuator Air can no longer move the valve pin during normal operation.

Make sure that the valve pin is movable again. For example, the valve pin may be jammed due to dirt or debris. Then carry out a recalibration by clicking on Recalibrate Device.

## Mechanical Defect

This message appears if the actuator does not detect a mechanical stop during recalibration. The recalibration is performed automatically after each restart of the actuator (e.g. After power is disconnected).

A mechanical stop is normally detected when the actuator plunger presses against the against the valve pin. If the actuator is not mounted on a valve, the mechanical stop is detected when the actuator plunger presses against the actuator housing.

If no mechanical stop is detected, it can be assumed that the actuator is defective. In this case, contact your [Loxone Partner](https://www.loxone.com/enen/shop/find-a-partner/).