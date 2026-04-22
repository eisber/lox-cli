# Block Presets

Source: https://www.loxone.com/enen/kb/block-presets/

---

A block preset allows you to save parameters and settings of a block to a template.
For this, a block is first configured with its parameters and settings, and then a block preset is generated.

Then the preset can be used for other blocks of the same type, and the blocks will adopt the values specified in the preset.

#### CONTENT:

[Creating a preset](#create)

[Assigning a preset](#assign)

[Managing presets](#manage)

[Resetting a function block](#reset)

## Creating a preset

In the following example, we create a preset from a Lighting Controller.

First, we set all the applicableparameters. Also, block settings, such as light circuits, lighting scenes and automatic functions, can be defined.

Next, we click on **Create new preset** in the block’s properties.

[
![blockpresets create preset](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-create-preset.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-create-preset.png)

Then we assign a name and whether we want to include the parameters, the settings or both in the preset:

[
![blockpresets create preset 2](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-create-preset-2.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-create-preset-2.png)

The **Create** button will create the preset.

## Assigning a preset

Now we can add more Lighting Controllers and apply the newly created preset including it’s parameters and settings to them.

We select a block and chose the previously created preset from the drop-down list in the block preset properties:

[
![blockpresets choose preset](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-choose-preset.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-choose-preset.png)

The block then adopts the values from the preset.

Using the same method, we can assign the preset to other blocks.

## Managing presets

You can also **Manage:** the presets from the block’s presets.

[
![blockpresets open manage](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-open-manage.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-open-manage.png)

In the following window, presets can be renamed and deleted, or the assigned blocks can be reset to the default preset:

[
![blockpresets manage presets](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-manage-presets.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-manage-presets.png)

**Assign blocks** allows you to assign presets to multiple function blocks with one click:

[
![blockpresets select multiple blocks](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-select-multiple-blocks.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-select-multiple-blocks.png)

If you change settings of a block that has a preset assigned to it, you can apply those changes to the assigned preset.

To do this, click on **Update preset** in the block properties:

[
![blockpresets update preset](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-update-preset.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-update-preset.png)

A click on **Restore preset** will reset the block to its default values as specified by the assigned preset.

## Resetting a function block

Supported blocks have a **RtD input** that can reset the block to its preset in case the any of the settings have been changed by the user:

[
![blockpresets reset to preset](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-reset-to-preset.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/02/blockpresets-reset-to-preset.png)

Use case hotel room: Guests have adjusted the light control or temperature control to their liking. The block can be reset to its default values as per the preset via the RtD input before the next guest checks in.

**The following blocks support a reset via the RtD input:**

Lighting Controller

Intelligent Room Controller

Audio Player

Room Ventilation Controller

Alarm Clock

Schedule