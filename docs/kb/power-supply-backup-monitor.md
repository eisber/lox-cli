# Using the Power Supply & Backup to monitor usage and as a UPS

Source: https://www.loxone.com/enen/kb/power-supply-backup-monitor/

---

## Brief: Monitor and maintain the power supply for multiple devices efficiently, particularly during power outages, without needing individual monitors for each component.

In a traditional installation, all devices run on 230V mains power. Although 230V can operate all electric appliances, the actual power usage of devices is much lower than that. In a Loxone installation, peripheral devices run on 24V instead, which is supplied by centrally-installed power supplies. This is both safer and more efficient. It’s also more energy conscious because the integrated devices are operated by a lower voltage, saving your customers money and energy.

Sourcing power from a lower voltage source with Loxone is a great start to improve energy efficiency and lower bills for your clients, but it shares some of the obstacles of a traditional installation. Like in a conventional system, monitoring the flow of electricity is essential, and in the past every component of the Loxone system required individual power supplies and energy meters, taking up even more space in the cabinet.

The Loxone Power Supply & Backup is a compact solution that manages all the above: it offers 7 outputs providing 24VDC for up to 10A each, in-built fuses and integrated energy monitors for each output, and a 36VDC battery to provide a backup for your installation in case of a power failure.

In the screenshot below, we can see an example of the monitoring function of the 7 outputs. Being aware of the power usage on a more granular level better empowers consumers to make informed decisions about their own energy use, and adjusting their behaviours if necessary.

[
![PH EN UseCase PowerSupplyBackup 555x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-UseCase-PowerSupplyBackup.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-UseCase-PowerSupplyBackup.png)

## Solution: The Power Supply & Backup has 7 in-built channels with integrated monitoring and a backup battery to protect against power cuts.

The Power Supply & Backup includes 7 built-in channels with integrated monitoring, visualised in the Loxone App via the Energy Flow Monitor. This enables your clients to understand the power consumption of their connected smart sensors, switches, lights and cabinet components anytime from anywhere. If a fuse blows, this will be visible both in the app as well as on the Power Supply & Backup unit itself. If batteries have been connected (optional) and Backup Mode is activated, a notification will be sent to the end-user.

The 7 24V outputs provide a total of up to 40A – enough power to reliably supply all components in the cabinet and the peripheral devices in your Loxone installation with the required voltage.

The backup function is equally important. A power cut will have a significant effect on the operations of your clients’ home and business, but not many people prepare for an emergency. As electrics have become an essential part of our daily lives that we have come to rely on – WiFi, lighting, audio, access, etc – this can cause severe issues in daily operations and compromise your customers’ comfort. The Power Supply & Backup can ensure that your 24V Loxone installation will maintain functionality during an outage. For example, with a 20Ah battery, this means approximately 8 hours. To achieve this, you can connect 3x 12V batteries in series that will give you a total output of 36V at the rated current to ensure continuous and seamless operation during power outages by taking over from mains power. This can include lighting, audio, peripherals and sensors. The Power Supply & Backup ensures these elements are fully functional even if mains power is unavailable.

What’s more, the intelligent automation built into every Loxone system can prolong the life of this battery supply. Loxone’s intelligent automation extends the emergency battery life by reducing power usage in certain Operating Modes, like dimming lights via the Lighting Controller Function Block. For example, if Backup Mode is activated, the maximum brightness of all the lights in the building can be lowered to 20%. This will still provide visibility but will not use as much power as if left on full brightness.

In case of a blown fuse, the Power Supply & Backup’s design prevents damage to other components, and spare fuses are included.

With the 10A outputs, the fuses, the energy monitors and the backup battery fitted into one device, this single unit efficiently manages power supply monitors 7 groups of peripheral devices, offering a space-saving and energy-efficient solution compared to traditional systems with multiple extensions and meters.

### Hardware:
- Miniserver (can be [Miniserver](https://shop.loxone.com/enuk/miniserver.html), [Miniserver Compact](https://shop.loxone.com/enuk/miniserver-compact.html) or [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html))
- [Power Supply & Backup](https://shop.loxone.com/enuk/power-supply-backup.html)
- Any 3x 12V lead-acid battery (specified accordingly based on the size of the system)

### Configuration:

[
![PH EN UseCase PowerSupplyBackup Config 800x494](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-UseCase-PowerSupplyBackup-Config.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-UseCase-PowerSupplyBackup-Config.jpg)

### Download the sample Config file:

### Using-the-Power-Supply-Backup-to-monitor-usage-and-as-a-UPS

			[14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Using-the-Power-Supply-Backup-to-monitor-usage-and-as-a-UPS.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Using-the-Power-Supply-Backup-to-monitor-usage-and-as-a-UPS.loxone)

## Video:

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.