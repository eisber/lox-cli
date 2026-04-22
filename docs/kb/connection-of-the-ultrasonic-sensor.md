# Connection of the ultrasonic sensor

Source: https://www.loxone.com/enen/kb/connection-of-the-ultrasonic-sensor/

---

The ultrasonic sensor is supplied with 24V DC via the brown connection wire. Blue is connected to ground. The 0-10V analogue value is output on the white wire.
[
![ultrasonicwiring](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/01/ultrasonicwiring.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/01/ultrasonicwiring.png)

#### Setting the limits
- Place the object at Position 1 (lower limit).
Press T1 until both LEDs flash **simultaneously** (approximately 3 seconds).
- Both LEDs will flash alternately.
- Place the object at Position 2 (upper limit).
Press T1 for about 1 second.
The sensor returns to **Normal Operating Mode**.

#### Setting Rising/Falling Output Characteristic Curve:

Press T1 until both LEDs flash **alternately** (approximately 13 seconds).
- LED D2 flashes.
- LED D1 indicates the output characteristic:

**On**: Rising output characteristic.
- **Off**: Falling output characteristic.
- To change the output characteristic, press T1 for about 1 second.
**Wait for 10 seconds.**
The sensor returns to **Normal Operating Mode.**

#### Activating/Deactivating Teach-In:

**Turn OFF the power supply.**
While pressing T1, turn ON the power supply.
Keep T1 pressed until both LEDs flash **simultaneously** (approximately 3 seconds).
- LED D2 flashes.
- LED D1 indicates the Teach-In status:

**On**: Teach-In activated.
- **Off**: Teach-In deactivated.
- To activate or deactivate Teach-In, press T1 for about 1 second.
- **Wait for 10 seconds.**
- The sensor returns to **Normal Operating Mode.**

---

#### Resetting to Factory Settings:
- **Turn OFF the power supply.**
- While pressing T1, turn ON the power supply.
- Keep T1 pressed until both LEDs flash **alternately** (approximately 13 seconds).
- The sensor resets and returns to **Normal Operating Mode.**

More information can be found in the [datasheet](https://www.loxone.com/wp-content/uploads/datasheets/200054-Ultraschallsensor.pdf).