# Central

Source: https://www.loxone.com/enen/kb/central/

---

This block allows you to centralise global functions in one place, such as an all lights off function.

#### CENTRAL CONTROL

This video shows you how to get central functions to work, such as all lights off or controlling all the blinds in your house. You can use memory flags and the central block for this.

## INTRODUCTION

The central block allows you to reduce the number of memory flags used in your program by simplifying global functions between blocks, chiefly the lighting controller and automatic blinds blocks.

In the central block you can see all the lighting controllers and automatic blinds in the program and choose whether to use them as inputs or control them from the central block.

## BASIC SETUP

It is very simple to start using the Central block to implement global functions. Simply connect the virtual inputs that you would have connected to memory flags to all the different inputs to the Central block.

![Replacement Basic setup 300x160](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/Replacement-Basic-setup.png)

Then double click on the block and choose which blocks you would like to control through the Central block by ticking Output. If you want to use a block as an input to the Central block, for example triple clicking on a Lighting controller turns all the lights off, tick Input.

![Example Screenshot Of The Inputs and Outputs Of Central Function Block In Loxone Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Central_Block_Inputs_Outputs.png)

For example the virtual input All blinds up now triggers the central block which will then tell all the Automatic blinds to go to up position (output Cu on the Central block). Any Automatic blind blocks that are not ticked in the Central block will not be affected by the All blinds up virtual input. This is useful because if you wanted to turn all the lights off but perhaps wanted to leave a night light lamp on in a nursery.

Once you have ticked a function block in the Central block that function block will have a small “plus” mark and when hovering over the mark a small tool tip appears telling you that it’s linked to a Central block.

![Being controlled by the central block 1 300x139](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/Being-controlled-by-the-central-block-1.png)

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Inputs that come from the central block are not prevented from working by the Dis input on the block. For example, if the Dis input is active on the Lighting controller block, an all lights off command sent from the Central block would still work.

## SUPPORTED FUNCTION BLOCKS

The following function blocks are controllable with the Central block:
- Automatic blinds
- Blinds
- Central
- Dimmer
- EIB Dimmer
- EIB Push-button
- Intelligent room controller (shading output Qs)
- Lighting controller
- Multifunction switch
- Push-switch
- Push-button
- Push-button (double) +/-
- Push-button (double) On/Off
- Radio buttons (8x)
- Radio buttons (16x)
- RGB lighting controller
- Stairwell light switch

### INPUTS

| Name | Function | Explanation | Value range | Unit |
| --- | --- | --- | --- | --- |
| Loff | Turns lights off | Single pulse sets all lights to OFF | 0/1 | – |
| Lon | Turns lights on | Single pulse sets all lights to ON | 0/1 | – |
| La | Turns optical alarm on/off | When ON light alarm is active | 0/1 | – |
| Ja | Sets blinds to state they would be in when the alarm is active. | When ON blinds are moved to alarm position | 0/1 | – |
| Cu | Moves blinds up | Sing pulse opens blinds (on selected controllers). Doesn’t fully open. | 0/1 | – |
| Cd | Moves blinds down | Single pulse closes blinds (on selected controllers). Doesn’t fully close. | 0/1 | – |
| S | Closes blinds and opens slats. | Single pulse opens the slats and lifts the blinds. Another pulse whilst this action is underway will cancel this. | 0/1 | – |
| AS | Activates auto shading during shading periods | Gives pulse to activate autoshading. | 0/1 | – |
| AD | Disables autopilot shading | Whilst this is on, autopilot shading cannot be on. | 0/1 | – |
| AR | Enables autopilot shading | Pulse to reactivate autopilot shading. | 0/1 | – |
| Sp | Toggles Storm protection | When on, enables/disables storm protection mode. | 0/1 | – |
| St | Stops blinds | Stops movement of blinds where they are. | 0/1 | – |
| T5 | Allows the use of a Touch air, remote air, touch tree | Allows Touch Air, Touch Tree and remote airs to interface with the block. | 0/1 | – |
| Als | Reserved for future use. | N/A | N/A | – |
| lc | Reserved for future use. | N/A | N/A | – |
| Dis | Disables all inputs | When on disables all inputs for the block. | 0/1 | – |

### OUTPUTS

| Name | Function | Explanation | Values | Unit |
| --- | --- | --- | --- | --- |
| QLoff | Triggers when all lights are OFF | Outputs a pulse when all the lights are off. | 0/1 | – |
| QLon | Triggers when all lights are ON | Outputs a pulse when all the lights are on. | 0/1 | – |
| QLa | When visual alarm is triggered gives a pulse | Outputs a pulse when the visual (lights) alarm is triggered on lighting controllers. | 0/1 | – |
| QJa | Outputs when the binds are in alarm mode. | Outputs a pulse when the blinds are in their alarm configuration. | 0/1 | – |
| QCu | Outputs when the blinds are up | Outputs a pulse when the blinds are completely up. | 0/1 |  |
| QCd | Outputs when the blinds are down. | Outputs a signal when the blinds are completely down. | 0/1 | – |
| Qs | Outputs when autopilot shading is active | Outputs a pulse when shading is active. Drives blinds to complete DOWN state then back to put the slats horizontal. | 0/1 | – |
| QAS | Outputs a signal to turn on auto pilot shading during the shading period | Outputs a pulse to activate autopilot shading when in a shading period. | 0/1 | – |
| QAD | When on, autopilot shading cannot be on. | When this output is on, autopilot shading is off. | 0/1 | – |
| QAR | outputs when autopilot shading is turned back on. | Outputs a pulse when autopilot shading is re-enabled. | 0/1 | – |
| QSp | Outputs when storm protection mode is on. | Outputs a signal when safety shutdown is active (storm protection) | 0/1 |  |
| QSt | Outputs when the blinds stop moving. | Outputs a pulse when the blinds stop moving. | 0/1 | – |
| AQT5 | Combined button output | Output for the Touch tree, Touch Air, Air Remote. | 0/1 | – |
| AQs | Reserved for more features. | N/A | 0/1 | – |
| Qc | Reserved for more features. | N/A | 0/1 | – |
| QDis | On when the input Dis is on. | When the input DIS is on, this emits a signal. | 0/1 | – |