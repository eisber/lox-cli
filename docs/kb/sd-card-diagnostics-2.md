# SD Card diagnostics

Source: https://www.loxone.com/enen/kb/sd-card-diagnostics-2/

---

The SD card is an integral part of the Loxone Miniserver, the heart of every Loxone Smart Home. It is therefore all the more important to react early to problems that affect the SD card. Your Miniserver gives you various pieces of information about the [system status.](https://www.loxone.com/enen/kb/systemstatus/)

### SD Card quality test

Once a week, and after each reboot, the Miniserver independently checks its SD card. In addition to a read and write speed check, it also tests whether there are any errors in writing and reading and how high the SD card’s load is on written statistics. With compatible cards, the health status can be directly readout in the form of already used write cycles. If problems are detected, the following messages are displayed:
- [SD card high write load](#high-write-load)
- [Write cycles almost exhausted](#cycles-nearly-done)
- [Write cycles exhausted](#cycles-done)
- [SD card worn](#worn)
- [SD card defective](#broken)

The quality test of the SD card performed by the Miniserver can also be started manually with the [Webservice](https://www.loxone.com/enen/kb/web-services/) “http://miniserver/dev/sys/sdtest”.

### SD cards memory usage

If your Miniserver’s SD card is full, the system may experience unpredictable behaviour, impairments, and system failures. To avoid this, avoid the following messages if the available space on the SD card is already low:
- [SD card nearly full](#nearly-full)
- [SD card is full](#full)

### Use only SD cards from Loxone

The SD cards provided by Loxone for use in the Miniserver undergo several weeks of quality testing. For maximum performance, your Miniserver accesses many low-level features of the SD card. The interaction with Loxone OS – the operating system of the Miniserver – must, therefore, be optimal. Therefore, only use SD cards approved by Loxone! There is no warranty for the use of other SD cards.

## SD card high write load

You will receive this message if you are either recording statistics for a very large number of objects or if you are using a particularly high polling cycle (such as “Any change”) for some statistics. As a result, the Miniserver has to write to the SD card with a high frequency, which drastically shortens its lifecycle.

### Troubleshooting

In order to solve the problem and to acknowledge the message, the number of written statistics entries must be reduced. The [project validation](https://www.loxone.com/enen/kb/project-validation/) in Loxone Config helps you find out exactly which objects generate statistics, so you can disable specific ones. Alternatively, the statistics for the corresponding objects can also be adjusted in the Loxone Smart Home app via the expert mode  (if this is available to your user).

## Write cycles almost exhausted

The available write cycles of the SD card of your Miniserver is almost exhausted. To prevent unwanted behaviour, it is recommended to swap the SD card early.

The exchange of the SD card must be done by a system administrator or a Loxone partner. It is also recommended to make a [backup of](https://www.loxone.com/enen/kb/backup-sd-card/) the SD card. This backup can then be applied to the new SD card when formatting the card.

## Write cycles exhausted

The available write cycles of the SD card of your Miniserver has been exhausted! Take immediate action and replace the SD card. Otherwise, your system will be impacted.

The exchange of the SD card must be done by a system administrator or a Loxone partner. It is also recommended to make a [backup of](https://www.loxone.com/enen/kb/backup-sd-card/) the SD card. This backup can then be [applied when formatting the SD card](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/).

## SD card worn

Your Miniserver classifies the SD card as worn out if it can no longer be guaranteed to function correctly. Reasons can be:
- Low write speed
- Low reading speed
- Accumulation of read and write errors

It is possible that the message disappears after some time and reappears. This does not mean that the message was a false positive!

### Troubleshooting

The cause is either a very high memory usage or mechanical wear. First, check the memory usage of the SD card and delete old data. If this does not solve the problem, it is recommended to replace the SD card.

#### Clean SD card

To check the memory usage of your SD card, perform the following steps:
- Run the web service ” http://miniserver/dev/sys/sdtest “. This takes a few seconds.
- Look for “usage” in the answer (used space of the SD card in%).
- If more than 60% of the memory space is occupied, delete old data. To manage the data stored on your SD card, click [here](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/).
- Start the SD card quality test again with the web service mentioned in step 1.

If the problem has not been solved after cleaning up the SD card, it is recommended to replace the SD card.

#### Swap SD card

The exchange of the SD card must be done by a system administrator or a Loxone partner. It is also recommended to make a [backup of](https://www.loxone.com/enen/kb/backup-sd-card/) the SD card. This backup can then be [applied when formatting the SD card](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/)

## SD card defective

Take action immediately if your Miniserver classifies its SD card as defective! Otherwise, your system will be impacted and degraded. otherwise it will lead to total failure of the system. Reasons for a classified as defective SD card can be:
- Very low write speed
- Very low reading speed
- Failed SD Cards Quality Test

It is possible that the message disappears after some time and reappears. This does not mean that the message was a false trip! Need for action is still given.

### Troubleshooting

The cause is either a very high memory usage or mechanical wear. First, check the memory usage of the SD card and delete old data. If this does not solve the problem, it is recommended to replace the SD card.

#### Clean SD card

To check the memory usage of your SD card, perform the following steps:
- Run the web service ” http://miniserver/dev/sys/sdtest “. This takes a few seconds.
- Look for “usage” in the answer (used space of the SD card in%).
- If more than 60% of the memory space is occupied, delete old data. To manage the data stored on your SD card, click [here](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/).
- Start the SD card quality test again with the web service mentioned in step 1.

If the problem has not been solved after cleaning up the SD card, it is recommended to replace the SD card.

#### Swap SD card

The exchange of the SD card must be done by a system administrator or a Loxone partner. It is also recommended to make a [backup of](https://www.loxone.com/enen/kb/backup-sd-card/) the SD card. This backup can then be [applied when formatting the SD card](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/)

## SD card almost full

There is less than 20% free space on your SD card. In order to prevent a complete replenishment of the memory and the associated impairments at an early stage, old data should soon be deleted from the card.

To manage the data stored on your SD card, click [here](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/).

## SD card full

There is less than 5% free space on your SD card. In order to avoid a complete replenishment of the memory and the associated impairments, old data must be deleted from the card.

To manage the data stored on your SD card, click [here](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/).