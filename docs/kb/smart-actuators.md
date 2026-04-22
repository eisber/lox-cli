# Smart Actuators

Source: https://www.loxone.com/enen/kb/smart-actuators/

---

“Smart Actuators” have been introduced to Loxone config as of version 9.0. Smart Actuators are a new type of output for all lighting products. For each lighting device, this can be set in the properties panel under “Actuator Type”. All Loxone Lighting products (Air and Tree) will automatically be set to Smart Actuators.
The Smart Actuators allow for perfectly smooth fading times when combined with Loxone Air and Tree lighting products. This allows a step-free slow fade either in brightness of colour over long periods up to an hour. They also combine for RGBW the RGB and Warm White elements into one simple and easy to use output with a much friendlier user interface.

Smart Actuators are also supported on DMX and Mains dimming (via Dimmer extension) but please note that due to some of the limitations of 230V dimming and DMX protocol the step-free fading performance will not be as good as it would be with Tree or Air lighting products.

## IMPORTANT INFORMATION

Smart Actuators are only compatible with the Lighting Controller function block, NOT the legacy Lighting Controller V1 block. The Lighting Controller block automatically detects whether the Smart Actuator is for RGBW or for simply White.

## ADVANTAGES
- Easier configuration
- Different fading times between changing moods, changing dim value and direct changes through the app
- Colour sequencing
- Morning alarm clock lighting can be slowly faded in over extended periods of time

## ACTUATOR TYPES

In the properties panel of supported devices, you will see “Actuator type” here the will a range of options that will include some of the following options.
- Smart – Creates a Smart Actuator depending on RGBW or WW operation.
- Smart Individual Channels – Creates 4 individual channels that are each used as Smart Actuators (only on RGBW Dimmer Tree/Air)
- Standard – Creates either standard RGB and W channels or a single standard WW channel
- Standard Individual Channels – Creates 4 standard individual channels (only on RGBW Dimmer Tree/Air)

![Windows 8 1 1 2](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Windows_8_1_1-2.png)

## USE

Smart Actuators can ONLY be connected to the Lighting Controller NOT the legacy Lighting Controller V1 block.

![Windows 8 1 1 3](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Windows_8_1_1-3.png)

In the configuration of the Lighting Controller, the Smart Actuator type is selected under the Light Circuits tab.

![Windows 8 1 1 4](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Windows_8_1_1-4.png)

#### For information on how to use the smart actuators with the new lighting controller, please see the page about our [Lighting Controller](https://www.loxone.com/enen/kb/lighting-controller/).