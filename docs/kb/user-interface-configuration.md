# User Interface Configuration

Source: https://www.loxone.com/enen/kb/user-interface-configuration/

---

#### SELECT OBJECT

Start the configuration software and click on any input, output or function block (e.g. blinds, burglar alarm, etc.) to select it and set it up for use in the user interface.

**
![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Block functions**

For the so-called block functions there may be extra properties available for the user interface.

#### ASSIGN NAME AND DESCRIPTION

Each input and output, as well as each object, has a name and description. These can be edited in the properties window.

The name of the object is used by default as the label in the user interface. If you want to use an alternative name for an object in the user interface, then enter it in the description field.

**
![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Tip**:

In the configuration software, it is always the name of an object that is displayed. You could therefore use this field for information such as terminal arrangement, location in the consumer unit, cable ID or KNX address to help keep track of things. The description, on the other hand, is used only in the user interface (and only if set).

#### ASSIGN CATEGORY AND ROOM

In the Properties window, scroll to the section titled ‘User Interface’ and select a category and room from the drop down lists. To show the object in the user interface, you must also tick the box next to ‘Use’.

#### SET PROPERTIES FOR DISPLAY

In the section called ‘Display’ of the properties window, you can adjust any further settings unique to that particular object type. This will affect the way the object is shown in the user interface.

#### ASSIGN FAVOURITES

In the ‘User Interface’ section of the properties window, you can also assign a 0-5 star rating. This rating determines the order in which objects are shown on the home screen of the user interface.

You can apply ratings not just to inputs, outputs and functional blocks, but also to categories and rooms. These can be found in the periphery tree. Click on a category or room and then look at the properties window to set its rating.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
On the home screen of the user interface, the 6 highest rated entries in room and category, as well as the 8 highest rated entries out of the inputs, outputs and functional blocks will be displayed.

#### SET PASSWORDS ON UI

You can also use the user passwords on specific blocks to make the user interface even more secure. For each user, a password can be set. Simply select the required user on the Periphery tree and give it a password in the Properties section under “User interface password”.

Then in a block’s Properties check the tick box next to “User interface password”. This will then enable the block to prompt a password when opened.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
So you can, for example, protect the control of the garage door or electric front door lock by adding password protection to the blocks to stop accidental opening or operation.

#### SAVE TO THE MINISERVER

Finally, following the above settings save the program to the Miniserver. If there are any changes to your the settings you must save the program in the Miniserver again for changes to have an effect on the visualisation.