# SD Card Diagnostics

Source: https://www.loxone.com/enen/kb/sd-card-diagnostics/

---

The SD Card is an important part of the Loxone Miniserver, the heart of every Loxone Smart Home. This makes it all the more important to react early to any problems related to the SD Card. Your Miniserver will notify you of any issues through [System Status](https://www.loxone.com/enen/kb/systemstatus/).

### SD Card Quality Test

Once a week, and after each restart, the Miniserver automatically checks its SD Card. In addition to checking the write and read speed, it also tests whether any errors occur during writing and reading as well as reading the load on the SD Card (E.g. Statistics & Logging). If problems are detected, one of the following messages is displayed:
- [SD Card Write Load High](#writeload)
- [SD Card Damaged](#worn)
- [SD Card Defective](#broken)

The SD Card quality test carried out by the Miniserver can be carried out manually with the [Web Service](https://www.loxone.com/enen/kb/web-services/) “http://miniserver/dev/sys/sdtest”.

### Only use Loxone SD Cards

The SD Cards released by Loxone for use in the Miniserver undergo a quality test that lasts several weeks. For maximum performance, your Miniserver access many low-level function of the SD card. The interaction between Loxone OS – the operating system of the Miniserver – and the SD Card must therefore be optimal. Hence you should only use SD cards released by Loxone! There is no warranty for the use of other SD Cards.

## SD Card High Write Load

You will receive this message if you either record statistics for a large number of objects or use a particularly high polling cycle (e.g ‘Every Change’) for some statistics. This means that the Miniserver has to frequently write to the SD Card, beyond its designed specification, which rapidly shortens its lifespan.

### Troubleshooting

To fix the problem and acknowledge the message, the number of statistics being written must be reduced. The [Project Validation](https://www.loxone.com/enen/kb/project-validation/) feature in Loxone Config helps you to find out exactly which objects are generating statistics, and how many statistics entries are being written per day. This allows you to selectively deactivate or reduce the quantity of written statistics.

Information for recording statistics via Loxone Config can be found [here](https://www.loxone.com/enen/kb/statistics). Alternatively, the statistics for the relevant objects can also be customized in the Loxone Smart Home App via the [Expert Mode](https://www.loxone.com/enen/kb/expert-mode/) (if your user account has access).

## SD Card Damaged

Your Miniserver has classified the SD Card as worn out and as such, error-free functionality can no longer be guaranteed. Reasons for this could be:
- Low Writing Speed
- Low Reading Speed
- An Accumulation of Read and Write Errors

### Troubleshooting

The cause of this could be either very high storage usage or physical wear. First check the storage usage of the SD Card and delete old data. If this does not solve the problem, it is recommended that the SD Card is replaced.

#### SD Card Cleanup

To check the storage usage of the SD Card, follow these steps:
- Run the [Web Service](https://www.loxone.com/enen/kb/web-services/) “http://miniserver/dev/sys/sdtest”. This takes a few seconds.
- Search the output string for “Usage” (Occupied storage space in %).
- If more than 60% of the disk space is occupied, delete old data. For information on managing the data stored on your SD Card, see [here.](https://www.loxone.com/enen/kb/statistics)
- Start the SD Card quality test again with the Web Service mentioned in Step 1.

If the problem is not fixed after cleaning the SD Card, it is recommended that the SD Card is replaced.

#### Replace SD Card

To replacement of the SD Card must be performed by a System Administrator or by a [Loxone Partner](https://www.loxone.com/enen/shop/find-a-partner/). It is also recommended to create a [backup](https://www.loxone.com/enen/kb/backup-sd-card/) of the SD Card. This backup can then be used to [format](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/) the new SD Card.

## SD Card Defective

Act immediately if your Miniserver classifies its SD Card as defective! Otherwise, the result will be disruptions to everyday tasks and system disruptions. If not, the system will fail completely. Reasons for an SD Card being classified as defective can be:
- Very Low Write Speed
- Very Low Read Speed
- Failed SD Card Quality Test

### Troubleshooting

The cause of this could be either very high storage usage or physical wear. First check the storage usage of the SD Card and delete old data. If this does not solve the problem, it is recommended that the SD Card is replaced.

#### SD Card Cleanup

To check the storage usage of the SD Card, follow these steps:
- Run the [Web Service](https://www.loxone.com/enen/kb/web-services/) “http://miniserver/dev/sys/sdtest”. This takes a few seconds.
- Search the output string for “Usage” (Occupied storage space in %).
- If more than 60% of the disk space is occupied, delete old data. For information on managing the data stored on your SD Card, see [here.](https://www.loxone.com/enen/kb/statistics)
- Start the SD Card quality test again with the Web Service mentioned in Step 1.

If the problem is not fixed after cleaning the SD Card, it is recommended that the SD Card is replaced.

#### Replace SD Card

To replacement of the SD Card must be performed by a System Administrator or by a [Loxone Partner](https://www.loxone.com/enen/shop/find-a-partner/). It is also recommended to create a [backup](https://www.loxone.com/enen/kb/backup-sd-card/) of the SD Card. This backup can then be used to [format](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/) the new SD Card.