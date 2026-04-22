# Touch Pure Tree Gen. 1

Source: https://www.loxone.com/enen/kb/touch-pure-tree/

---

![Loxone Touch Pure Tree Diagram](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/Touch_Pure_Tree_pdf__1_page_.png)

*dimensions in mm

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Please note temperature and humidity is sensed through the Touch Pure Tree’s closed housing which can mean a delay in sensing time. For the rapid detection of changes in the ambient humidity (eg. For ventilation control in the bathroom), we recommend a sensor with an open housing design as the Loxone Temperature & Humidity Sensor Air.

## MOUNTING SUGGESTIONS

The Loxone Touch, Touch Pure and Motion sensor Tree are very slimline, as such we recommend using/specifying European style circular back-boxes for the best fitment and finish. An example of this can be seen below, [click on the image to purchase them.](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

[
![Orange-circular-back-box](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/03/Orange-circular-back-box.jpg)
](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

## SETUP

To commission the Touch Pure tree connect the 24V power (orange/Orange-white terminals) and the Tree terminals (Green/Green-white terminals). The device will flash orange when connected.

![Loxone Touch Pure Tree Back](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Product_Touch_Pure_Tree_Back.png)

[For instructions on how to work with Tree devices in Loxone Config please see here.](https://www.loxone.com/enen/kb/tree-cabling-setup/)

## TOUCH POINTS

The touch points may be used in Loxone Config as digital inputs. Every touch point is a digital input.

For Press and Hold, you will need to set the ‘Receive Timeout’ properties of the Digital Input to 0.

![Touch Pure Tree Touch Points](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Product_Touch_Pure_Tree_Touch_Points.png)

## COMBINED KEY T5

The T5 output combines all 5 of the digital outputs of the Touch Pure tree. This is designed to speed up the programming of the Loxone standard.

The following functions are supported here:
- Automatic blinds: inputs Cu (touch point 1) and Cd (touch point 4)
- Light Control: Scene + (touch point 3)
- Music Server Zone: inputs V + (touch point 2) and V- (touch point 5)
- Central block control (double/triple click of centre key)

Connect the T5 output to the T5 input of the desired blocks.

In order to program and user-defined functions, ensure check boxes for inputs are activate in properties.

## Downloads:

Datasheets of the [Touch Tree](https://www.loxone.com/enen/datasheets/#100221,%20100222)

Datasheets of the [Touch Pure Tree](https://www.loxone.com/enen/datasheets/#100396,%20100398)

Datasheets of the [Touch Pure Tree Gen. 1](https://www.loxone.com/enen/datasheets/#100219,%20100220)