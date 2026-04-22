# Function Block Basics

Source: https://www.loxone.com/enen/kb/function-block-basics/

---

In order for the user interface to be generated, a configuration must first be created with correct settings & properties in the configuration software.

## SELECT OBJECT

Start the configuration software and click on any input, output or function block (e.g. Automatic Blinds, Burglar Alarm, etc.) to select it and set it up for use in the user interface.

![EN KB Config Configuration Select](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_Select.png)

## ASSIGN NAME AND DESCRIPTION

Each input and output, as well as each object, has a name and description. These can be edited in the properties window.

The name of the object is used by default as the label in the user interface. If you want to use an alternative name for an object in the user interface, then enter it in the description field.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
**Tip: **In the configuration software it is always the name of an object that is displayed. You could therefore use this field for information such as terminal arrangement, location in the consumer unit, cable ID or KNX address to help keep track of things. The description on the other hand is used only in the user interface (and only if set).

**For security reasons, not all characters in names and names used in the APP are allowed. Such as “/”**

![EN KB Config Configuration Name 1024x599](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_Name.png)

## ASSIGN CATEGORY AND ROOM

In the Properties window, scroll to the section titled ‘User Interface’ and select a category and room from the drop down lists. To show the object in the user interface, you must also tick the box next to ‘Use’.

![EN KB Config Configuration Category Room 1024x599](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_Category_Room.png)

## SET PROPERTIES FOR DISPLAY

In the section called ‘Display’ of the properties window, you can adjust any further settings unique to that particular object type. This will affect the way the object is shown in the user interface.

![EN KB Config Configuration Display Input Type 1 1024x494](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_Display_Input_Type-1.png)

## ASSIGN FAVOURITES

In the ‘User Interface’ section of the properties window, you can also assign a 0-10 star rating. This rating determines the order in which objects are shown on the home screen of the user interface.

Furthermore, you can also set the checkbox “Show favourites”. The selected object will be displayed among the favourites.

You can apply ratings not just to inputs, outputs and functional blocks, but also to categories and rooms. These can be found in the periphery tree. Click on a category or room and then look at the properties window to set its rating.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
On the home screen of the user interface, the 6 highest rated entries in room and category, as well as the 8 highest rated entries out of the inputs, outputs and functional blocks will be displayed.

![EN KB Config Configuration Rating 1024x599](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_Rating.png)

## SET PASSWORDS ON UI

You can also use the user passwords on specific blocks to make the user interface even more secure. For each user a password can be set. Simply select the required user on the Periphery tree and give it a password in the Properties section under “User interface password”.

![EN KB Config Configuration User Interface Password 1024x402](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_User_Interface_Password.png)

Then in a block’s Properties check the tickbox next to “User interface password”. This will then enable the block to prompt a password when opened.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
So you can, for example protect the control of the garage door or electric front door lock by adding password protection to the blocks to stop accidental opening or operation.

![EN KB Config Configuration UI Password 1024x599](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Config_Configuration_UI_Password.png)

## SAVE TO THE MINISERVER

Finally, following the above settings save the program to the Miniserver. If there are any changes to your the settings you must save the program in the Miniserver again for changes to have an effect on the visualisation.

## FUNCTION BLOCKS ON THE UI

Many of the function blocks in our software have user interface elements. This means you can control the function block from the user interface, so either from the web interface, iPhone or iPod apps, iPad app or Android app.

New function blocks are regularly being added with each software update – head on over to our Function Block section for a detailed description on most blocks, this list is always growing!