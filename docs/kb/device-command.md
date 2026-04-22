# Device Command

Source: https://www.loxone.com/enen/kb/device-command/

---

For Air and Tree devices, a device command can be used to retrieve information, adjust settings and trigger actions.

The corresponding information is displayed in the monitor of the relevant device.

#### Table of contents

[call Command](#devcom_call)

[Examples](#devcom_examples)

## call Command

With a right click on the desired device, device command can be selected and a command can be sent.

With “call” all supported commands of the selected device are listed in the respective monitor.

![devcom callCom](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/12/devcom_callCom.png)

					Only apply commands whose purpose and effects are fully known or if you are prompted to do so by Loxone, for example for diagnostics. Many commands are only intended for experienced developers. Some commands may have a negative effect on the product life or lead to a defect of the device and the expiration of the warranty.

## Examples

**TechReport**: Retrieves the TechReport of the device.

![devcom TechReport](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/12/devcom_TechReport.png)

**Reboot**: Reboots the device. During the reboot, “timeout” is displayed in the monitor.

![devcom Reboot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/12/devcom_Reboot.png)

**ForceUpdate**: Forces a device update.

![devcom ForceUpdate](https://www.loxone.com/enen/wp-content/uploads/sites/3/2022/12/devcom_ForceUpdate.png)