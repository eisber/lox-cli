# Miniserver Not Booting – Checklist

Source: https://www.loxone.com/enen/kb/miniserver-not-starting-checklist/

---

The start-up process of the Miniserver (for example after a power cut or software update) can take up to 1 minute, with Gateway-Client systems this can be even longer. When the operation is completed, the left status LED flashes green approximately once a second. If this is not the case, please proceed as follows:

## Miniserver LED: Status LEDs do not flash

If the two status LEDs of the Miniserver do not flash, this means that the Miniserver does not have an active power supply. Please check whether the green terminal for the power supply to the Miniserver’s is correctly plugged in, [wired correctly](https://www.loxone.com/enen/kb/wiring-basics/) and whether the power supply is at 24V.

## Left LED Flashing Red/Orange – Right LED Flashing Green

This behaviour indicates that the Miniserver does not recognise the SD card. Please check whether the SD card is inserted correctly into the slot on the lower left hand side of the Miniserver as depicted below.

![EN KB Product Miniserver SD Card](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Product_Miniserver_SD_Card.png)

If the SD Card is properly inserted then it suggests that the Miniserver is unable to read the operating system from the SD Card. In order for the SD card to be recognised, loaded from and used by the Miniserver, this must first be formatted with Loxone Config. Instructions for formatting the SD card in the Loxone Config are available [here](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/) .

## LED Flashing Is In A Loop

If the Miniserver can not read the data from the SD card correctly, it tries again to start and read the SD Card and hence restarts. If this process is repeated continuously, the miniserver is in a so-called reboot loop. This is reflected by the repeating of the LED flashing pattern.

A format of the SD card used as a short term solution here, but we recommend the card be completely exchanged. The reason for this is normally just some data being written to the SD Card incorrectly (normally due to a power cut or similar) and in some very rare circumstances this can damage poorer quality SD Cards. We only reccomend to use Loxone SD cards, which are designed specifically for use in the Miniserver and will have a longer service life.

					**Use only Loxone SD cards for the Miniserver. ([These can be found in our webshop](https://shop.loxone.com/enuk/sd-card.html?___SID=U)).**

**All SD cards have their own CPU which manages the flash memory. For optimum performance, Loxone OS accesses many low-level functions of the SD card, unlike, for example,  a digital camera. SD cards undergo several weeks of quality testing at Loxone before being released for use in Miniservers and being sold on the webshop. Please only use SD cards approved by Loxone. Using other SD cards is not supported and will affect the stability of the Miniserver**
If, in spite of these measures, the Miniserver does not continue to start, please contact[ Loxone Support.](https://www.loxone.com/enen/support/)