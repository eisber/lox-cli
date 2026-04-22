# Connecting Power Supplies in Parallel

Source: https://www.loxone.com/enen/kb/connecting-power-supplies-parallel/

---

For parallel operation of power supplies, a few technical details must be observed.

					**Only the TDK DRF240-24-1 power supply unit is suitable for parallel operation.** As of April 2020, we no longer sell this item on our webshop. This [Instruction Manual](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/03/DRF240241-Installation-Manual.pdf) for the DRF-240 power supply range contains important information regarding operation. All other power supplies on the Loxone webshop are not suitable for parallel operation.

## WIRING

Should a power supply be switched off or in the worst case fail, it is necessary to install a diode which protects the secondary power supply from damage caused by an opposite current flow. A [Schottky diode](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/03/40A-Schottky.pdf?x48792) can be used for this application . Please do not forget the cooling of the diodes in the form of a suitable heat sink.

![verkabelung mit diode 768x414](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/verkabelung-mit-diode.png)
- Set the output voltage of the power supply to be slightly higher than 24V in order to account for any voltage drop caused by the diode.
- Make sure that this is the same for all power supplies in the setup.
- Disconnect the connection between pin 1 & 2 from the connector CN201 (black wire).

![cn201 stecker 300x153](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/04/cn201-stecker.png)

4.Use the same cross-section and same cable length for optimum load distribution on the output side.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)

A maximum of 5 DRF-240 power supplies can be connected. Only 80% of each power supply can be used.