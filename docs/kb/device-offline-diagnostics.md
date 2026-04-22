# Device Offline Diagnostics

Source: https://www.loxone.com/enen/kb/device-offline-diagnostics/

---

If a device cannot be reached for a certain time, the device is reported as offline. The time taken for a device to report offline depends on the device type. Below you will find information on possible causes and troubleshooting.
An overview of the online status and batter level of Loxone devices can be found in Device Status in Loxone Config.

In the following [section](#troubleshooting), you will find information about possible causes and troubleshooting. To learn how to disable online status monitoring for a particular device, see [here](#deactivate).

## Possible Causes and Troubleshooting

### General
- Check if the device is powered. Most devices have a status indicator (An LED, a display, or similar). More detailed information can be found in the leaflet that comes with the device.
- If this is a battery-powered device, check that the batteries are sufficiently charged.
- Battery-powered devices only work when they are ‘awake’. To ‘wake up’ the device, press a button, trigger a sensor, or similar. Check if the device is online again after waking it up.
- If an extension is displayed as offline, the connected devices are also displayed as offline. In this case, focus on the issue with the extension.
- Further details about the device can be found in the leaflet that comes with the device.

If none of the above points has led to a solution to the problem, you will find more detailed information here:
- [Air Device](#air)
- [Tree Device](#tree)
- [Extension](#extension)
- [Network Device](#networkdevice)
- [Miniserver](#Miniserver)

### Air Device
- First, check the list of [general](#general) points.
- For further information on Loxone Air Technology, please refer to [here](https://www.loxone.com/enen/kb/loxone-air/).
- A detailed checklist for an Air Device displaying as offline can be found [here](https://www.loxone.com/enen/kb/air-troubleshooting/).

### Tree Device
- First, check the list of [general](#general) points.
- Check the Tree wiring. If multiple devices are offline, it often possible to see where the error is. Track the cabling to the affected devices and check for any issues along the line. If only one Tree device is offline, check the wiring on that device.
- Loxone Config has a [Tree Diagnostics](https://www.loxone.com/enen/kb/tree-cabling-setup/#dxdiag) tool. Run this tool to get more detailed information about the status of the connected Tree devices.
- For more detailed information on Tree Cabling, see [here](https://www.loxone.com/enen/kb/tree-cabling-setup/).

### Extension
- First, check the list of [general](#general) points.
- Check the link wiring. If multiple extensions are offline, it is often possible to see where the error is. Track the cabling to the affected device and check of any issues along the line. If only one Extension is offline, check the wiring on that device.
- Loxone Config has a [Link Diagnostics](https://www.loxone.com/enen/kb/link-diagnostics/) tool. Run this tool to get more detailed information about the status of the connected Extensions.
- The link bus may only be installed in a strict bus, with the Miniserver as the first device. A terminating resistor (supplied with the Miniserver) must be installed at the end of the line. Further information can be found [here](https://www.loxone.com/enen/kb/extension-diagnostics/).
- If the above points did not solve the problem, you will find [here](https://www.loxone.com/enen/kb/extension-diagnostics/) a more detailed checklist in case an extension displays as offline.

### Network Device
- First check the list of [general](#general) points.
- Check if the device is switched on.
- Many network devices have status LEDs on the network interface. If these are off, there is no cable connection to the destination (e.g. Router, Switch, etc.)
- If your Router/Switch has a maintenance page, open it, and if a list of active network devices is available, check the IP address of your device can be seen by the Router.
- Execute a ping command. If there is a response to this command, a network device exists with the IP address entered.
- Open the maintenance page of the network device (If available) and check its status.
- If the device has been assigned a manual IP address, it should be check whether it is unique on the network, and no other device has been assigned the same address.
- If the above points did not lead to a solution, you will find further information in the device’s user manual.
- For newer devices (e.g., Audio Server, Intercom), data exchange with the Miniserver is handled using a certificate stored on the devices.
- To establish a successful connection, the device times must be synchronized as closely as possible. If the times do not match, pairing or learning-in errors may occur.
- If an error message appears containing the text “certificate error” or similar, first check the date and time on the devices and adjust the settings (e.g., NTP server) if necessary.
- If the issue persists, the devices can be reset to factory settings.

### Miniserver
- First check the list of [general](#general) points.
- Check the instructions for [Network Devices](#networkdevice).
- If a client Miniserver in a client gateway system is offline, the cause may be an incorrect program in the respective client miniserver. Check this by connecting to the relevant Miniserver using Loxone Config. More detailed information about the correct configuration of a client gateway system can be found [here](https://www.loxone.com/enen/kb/miniserver-clientgateway-concentrator/).

## Disable Monitoring

If you no longer want a particular device to be reported as offline, you deactivate its online status monitoring. This can be useful if, for example your Fronius inverter is always switched off overnight. Online status monitoring can be deactivated in the following ways:
- Execute the corresponding action “Stop Monitoring this Device” on the respective [System Status](https://www.loxone.com/enen/kb/systemstatus/) message.
- In Loxone Config using the “Monitor Online Status” option, which can be found in the settings of the respective device.

For client Miniservers in a client gateway system, the monitoring cannot be deactivated.