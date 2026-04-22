# Wallbox Error-Codes

Source: https://www.loxone.com/enen/kb/wallbox-error-codes/

---

**Error Code Transmission for Improved Diagnostics**

To enhance error diagnostics, the Wallbox will now transmit error codes to the Miniserver. These codes will be displayed in the system status message and linked to the corresponding entries in the online documentation.

**Error Codes Overview:**

| Code | Title | Description |
| --- | --- | --- |
| 101 | Fault Current | A fault current was detected. Please check the vehicle and cabling. |
| 102 | RDC Init Timeout | The internal RDC initialization timed out. Please reboot the device. |
| 103 | RDC Test Timeout | The internal RDC did not respond during the self-test. Reboot the device. |
| 201 | Unexpected State | The EV entered an unexpected state. Unplug the vehicle and reconnect. |
| 202 | RDC Not Ready | The internal RDC is not ready. |
| 203 | Digital Communication | Unsupported communication standard detected. |
| 204 | Ventilation Required | The vehicle requires ventilation to resume the charging session. |
| 300 | Voltage Dip | A voltage dip was detected. Please check the Wallbox’s power supply. |