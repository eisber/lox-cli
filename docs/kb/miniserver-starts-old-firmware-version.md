# Miniserver Starts with Old Firmware Version

Source: https://www.loxone.com/enen/kb/miniserver-starts-old-firmware-version/

---

If your Miniserver sends you an error message by means of our mailer service, stating that your Miniserver has been started with an older firmware version, you will find further information here.
This behaviour can be caused by a possible data error. This can cause the Miniserver to try to start independently with an older firmware version as a safety procedure.

**
![IC attention exclamation mark green](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/11/IC_attention-exclamation-mark-green.png)

Please note:**

For compatibility reasons, an older Miniserver document may have been loaded as well!

It is possible that the Miniserver skips the current Miniserver firmware version by a read error on the SD card when starting, then loads an older version.

This is a security feature that the Miniserver can also start in this manner due to a faulty image on the SD. You can then make another update to restore functionality.

## POSSIBLE FIXES:

– Restart the Miniserver. This gives the Miniserver the chance to load the image again

– Perform a new update. The Loxone Config image is rewritten.

– Format a new Loxone SD card

– Check the date and time of the Miniserver