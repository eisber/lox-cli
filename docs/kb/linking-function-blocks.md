# Linking Function Blocks

Source: https://www.loxone.com/enen/kb/linking-function-blocks/

---

The **Link function blocks** feature allows you to display additional objects such as values or other blocks within the graphical interface of a function block.

Even objects that are not otherwise used in the user interface can be selected, giving the interface a clean and organized look.

#### CONTENT:

[Linking blocks](#linkblocks)

[Appearance](#linkedblocksvisu)

## Linking blocks

In the following example, we link the kitchen’s Intelligent Room Controller with additional objects.

First we select the block, then we click on **Select** linked function blocks in the properties:

[
![linkedblocks open selection](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-open-selection.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-open-selection.png)

No blocks are linked yet, so we click on the **Add** button:

[
![linkedblocks add blocks](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-add-blocks.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-add-blocks.png)

This opens a window in which all existing objects are listed.

The list can be filtered using the search field. Also, each column can be sorted by name.

In our example, we search for the room, and only the objects assigned to that room will be displayed.

We then select the humidity, CO2 value and the Room Ventilation Controller of the kitchen, and click on **Apply**:

[
![linkedblocks select objects](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-select-objects.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-select-objects.png)

Now the selected objects are linked and if desired their order can be changed using the arrow buttons:

[
![linkedblocks objects linked](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-objects-linked.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-objects-linked.png)

Finally, we again click on **Apply**, and save the program to the Miniserver.

## Appearance

In the Loxone App or the web interface, we now open the Intelligent Room Controller.

The objects that were linked to the Intelligent Room Controller are now displayed below the functions of the block:

[
![linkedblocks linked visu](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-linked-visu.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/linkedblocks-linked-visu.png)

As a result, all the values and controls relevant to the room climate are combined in the Intelligent Room Controller block.

Other blocks can directly be opened without having to add them as an additional object to the interface.

This improves and simplifies the user experience by reducing the number of objects displayed per room.