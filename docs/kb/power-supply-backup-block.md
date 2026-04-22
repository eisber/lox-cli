# Power Supply & Backup (Function Block)

Source: https://www.loxone.com/enen/kb/power-supply-backup-block/

---

This function block is used to integrate a **[Loxone Power Supply & Backup](https://www.loxone.com/help/powersupply-backup)**.

First the loads connected to the 24V outputs are labeled in the settings of the block.

Then the block provides the current power of the 24V outputs and provides information about the status of the optional backup mode and the backup battery.

Additionally, the battery capacity and the output voltage can be set.

If a Battery is connected, the remaining supply time in backup mode is calculated in the event of a power failure.

The user Interface of the block also provides information about power and other events such as a power failure or a blown fuse.

## Table of Contents
- [Outputs](#Output)
- [Properties](#Property)

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Pt | Power total | Current total power of all power outputs combined. | kW | ∞ |
| Bm | Backup mode | Active when the supply power fails. | - | 0/1 |
| P1-7 | Power 1-7 | This output is only visible in certain configurations. | kW | ∞ |
| Soc | Battery state of charge | Current charge level of the battery. | % | ∞ |
| Ol | Overload | Active when maximum power capacity of 1kW is exceeded for 5 seconds. Device will shut down automatically if output power isn’t reduced! | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 1...100 | 100 |

---