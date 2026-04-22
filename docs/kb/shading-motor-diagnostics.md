# Shading Diagnostics

Source: https://www.loxone.com/enen/kb/shading-motor-diagnostics/

---

The following message can be received from a GEIGER Shading Motor ([GJ56 Air](https://www.loxone.com/enen/kb/blind-motor-gj56-air/), [SOLIDline Air](https://www.loxone.com/enen/kb/tubular-motor-solidline-air/)) via [System Status](https://www.loxone.com/enen/kb/systemstatus/) or via the diagnostics ports on the device:
- [Obstacle Deteced](#obstacle-detected)
- [Motor Blocked](#motor-blocked)

## Obstacle Detected

This message appears when an obstacle is detected by the Shading Motor. The corresponding direction in which the obstacle was detected has now been locked

To release this lock, you need to operate the motor in the opposite direction. To do this, click on ‘Acknowledge’ on the corresponding message and then carry out the original operation. When operating via a button, the message is automatically acknowledged during the next movement towards the original blockage.

If the lower rail/rod of the blind is frozen or stuck to the ground, this is detected as an obstacle during lifting and the blind will return to the lower position. In this case, the movement direction is not blocked as movement in the opposite direction is not possible.

## Motor Blocked

As soon as the motor receives a command to move up or down, the system checks whether the motor shaft is rotating. If this is not the case, this message will be displayed in System Status.

In order to be able to execute a new command to the motor with the User Interface, the message must first be acknowledged in the corresponding message by clicking Acknowledge. If the motor is operated via a push button, the message is automatically acknowledged during the next successful run.

These are the following reasons for a blocked motor:
- [Mechanical blocking of Motor Shaft or Blind](#mechanically-blocked)
- [Motor Overheating](#motor-overheat)

### Mechanical blocking of Motor Shaft or Blind

Check the motor for a mechanical blockage and remove it. After that a motion should be possible again.

### Motor Overheating

If many trips are made in quick succession, the engine may overheat. In this case, the motor protects itself by not allowing any further movements. This condition continues until the motor has been sufficiently cooled.