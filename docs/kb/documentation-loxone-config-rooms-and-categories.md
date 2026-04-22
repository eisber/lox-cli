# Rooms and Categories

Source: https://www.loxone.com/enen/kb/documentation-loxone-config-rooms-and-categories/

---

Within Loxone Config there is the ability to set each room to a room type and also assign categories to different items (such as some relay outputs as shading for use with blinds). The Room, room type and category all have an effect on what Auto Config does with the item, and also how it appears in the app. Some more details on these different behaviours can be found below.

#### ROOM TYPES

Once you have set up all of your rooms, you will then be required to assign them a Room type. As of Config v.9, there are 4 different Room Types, each will change how certain blocks and how auto config behaves. These 4 are:

Bedroom

Entertaining

Thoroughfare

Central

#### CHANGING ROOM TYPE

You can set the room type when you first create it at the initial startup, however, you can change it later on (say in the case of a conversion/renovation). In order to change the room type, please find the Rooms section in the periphery and then press the edit rooms button.

![roomsEditRooms 219x300](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/roomsEditRooms.jpg)

Once you have clicked on here, you will be greeted with a window that allows you to edit the properties of the different rooms, such as the icon they use in the app and whether or not they are used as a favourite.

![roomsEditProperty 300x246](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/roomsEditProperty.jpg)

For more detail about each of these tabs, please see the below part about editing rooms and categories

## EDITING ROOMS AND CATEGORIES

Once you have that window open, you will be able to change different parts of your current rooms and categories, along with also adding new ones. Primarily this affects how they appear in the user interface.

#### EDITING ICONS

On the right-hand side of the tabs that appear at the top of this window, there are a collection of different icon tabs. These allow you to edit the pre-made icons, by double clicking on one of them, or even make your own icons by hitting the green plus at the bottom. You can also import images to use, but they must be a file that is in  .PNG format.

![roomsIcons 300x241](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/roomsIcons.jpg)

The editor looks as follows (also note the import button for importing your own images):

![roomsIconsEdit 300x249](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/roomsIconsEdit.jpg)

#### EDITING CATEGORIES

In the categories tab you have the ability to add categories and edit pre-existing one. Here you can: name/rename a category, assign them to a category group, give each category a star-based rating, select them as favourites, change which icon that specific category uses in the app and change the backdrop colour. All of these directly change how they appear in the app

![roomsCategory 300x248](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/roomsCategory.jpg)

#### EDITING ROOMS

The final tab to cover is the Rooms tab. Similar to the categories tab, you can edit a few properties that affect how the rooms appear in the app and also how some block behave.
![roomsRoomsRooms 300x248](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/roomsRoomsRooms.jpg)

Rating, Favourite and icon are the same as they are in categories, with the new part being Room Size. This directly affects the Intelligent Temperature controller as it will provide a volume for that block’s calculations. It directly affects output AQt on the [Intelligent Temperature Controller](https://www.loxone.com/enen/kb/intelligent-temperature-controller/) (heating demand calculations).