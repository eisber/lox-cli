# Setting up DMX Devices

Source: https://www.loxone.com/enen/kb/setting-up-dmx-devices/

---

## OVERVIEW

Once you have setup your DMX Extension, ensured that it is learned in and powered up, you can begin learning in or adding your DMX devices to Loxone Config. You can manually add DMX actuators, based off of their channels/channel use or simply learn in a Loxone RGBW 24v Dimmer DMX via the DMX device search.

## LEARNING IN DMX DEVICES VIA SEARCH

If you are using a Loxone RGBW 24v DMX driver (highly recommended as they have been developed side by side), then you can use the DMX Device search feature in Loxone Config to find any devices that are on the DMX BUS.

To do so, please select your DMX extension in the periphery tree and then hit the DMX device search button at the top.

![Screenshot 102717 092333 AM 251x300](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Screenshot_102717_092333_AM.jpg)

Once you have found your DMX device in the search, simply name it and create the device and it will appear in your periphery tree for use within Loxone Config.

## ADDING 3rd PARTY DMX DEVICES

If you are not using a Loxone RGBW 24v DMX driver, but a different, 3rd party driver that isn’t an RGBW driver, you can add devices manually and also specify their channels.

![dmx 2 219x300](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/dmx-2.png)

Here you can see the numerous actuator types that are on offer. You would pick which one you require depending on:
- Your type of driver – an RGBW one would warrant the use of an RGB and W actuator if you want the coloured lighting
- Your Driver’s channels – what if I have a driver with just 4 channels? Then you can use the DMX 4 actuators.
- How you want to control the channels – Say if you have an RGBW driver, but only controlling white lights off of it, that’s effectively 4 channels, so you would use a DMX 4 actuator instead.

When adding them manually, it is key that you take into consideration – the available channels and what is being controlled off of them as mismatching driver and actuator type would result in odd behaviour.

For reference, the Loxone RGBW 24v DMX driver is referred to as an RGB and W actuator

####

## DMX CHANNEL ADJUSTMENT

Adding DMX actuators will automatically assign channels, so for example if you add two DMX RGBW actuators the first one will be channel 1 and the second will be channel 4. This is because the DMX extension has 128 lines of communication, each line will control 1 channel on a driver. an RGBW driver has 4 – R, G, B and W, so, therefore, it needs to reserve 4 channels for this 1 driver.

To reassign the DMX channel go to the appropriate (DMX actuator / DMX RGB actuator) ‘Properties’ window, and enter your selection. Then save the program in the Miniserver.

![Loxone DMX Properties](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_DMX_Actuator_Properties.png)

## ADDING OTHER DMX DEVICES

For details on how to insert the Loxone RGBW 24V DIMMER DMX, please refer to [RGBW 24v Dimmer DMX](https://shop.loxone.com/enuk/rgbw-24v-dimmer-dmx.html).

If you are not using a Loxone DMX device, then please refer to [PWM Dimmer](https://www.loxone.com/enen/kb/pwm-dimmer/). Please note that the PWM Dimmer is discontinued and it is recommended to use the RGBW 24v DMX dimmers instead.