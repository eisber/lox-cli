# Commissioning Extensions

Source: https://www.loxone.com/enen/kb/configuring-extensions/

---

#### EXTENSIONS ARE CONNECTED TO THE MINISERVER WITH LOXONE LINK

To add one or more extensions to the program, first ensure that the Extensions are correctly wired to the Miniserver, [click here](https://www.loxone.com/enen/kb/wiring-basics/) to find out how. Then conduct a search in Loxone Config by pressing the button “Extension Search”, which will list all of the connected extensions. You can highlight a single extension, give it a name and then click Create device which will add it to your configuration. Alternatively, you can click on ‘Create all Devices’ which will add all the connected extensions in to the program with the correct serial numbers.

![Extension Search](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Extension_Search.png)

#### IDENTIFY DEVICE

If you have installed several extensions of the same type and cannot identify which one is which (for example 2 DMX extensions) then you can use Loxone Config to identify a device. Simply click on the device in the periphery tree and the extension’s left LED will flash between red and green.

#### EXTENSIONS ARE NOT CONNECTED TO THE MINISERVER

If there is no physical connection to the Miniserver then it cannot be found in the search. However, you can add extensions manually to allow you to create a configuration for a system prior to installation. In the configuration software, connect to your Miniserver and ensure your config file matches that on the Miniserver.

Select the ‘Add Extension’ drop-down menu on the Miniserver tab and select the required extension, then fill in the serial number in the properties.

![Add Extension Manually](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Add_Extension_Manually.png)

#### EXTENSION REPLACEMENT

If an extension has been replaced you need to add the serial number in the properties. Click on the extension in the periphery tree. Enter the serial number of the extension in the properties section.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
**Serial number**

The serial number is on the back of your extension. You can find the number just below the barcode (see picture).

![Extension Barcode](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_Extension_Barcode.jpg)

An Extension can also be replaced in the Extension Search.

Install the new Extension and run the Extension Search. Highlight the Extension (not used) and choose the device you would like to replace in the drop-down menu.

Finally press “OK” and save your program into the Miniserver. The Extension has now been replaced.

![Replace Extension](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Replace_Extension.png)