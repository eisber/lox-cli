# Push notification could not be sent

Source: https://www.loxone.com/enen/kb/push-notification-could-not-be-sent/

---

If a push message could not be sent, your Miniserver informs you about a [system status message](https://www.loxone.com/enen/kb/systemstatus/). Since the messages can contain important information, you should check the system for possible problems or errors that require your immediate attention (fire/water detectors, alarm system etc.).

### Possible Causes
- No Internet connection
- The Miniserver is not registered
- The maximum number of notifications has already been exceeded for this day (max. 200 messages per day).
- Loxone Cloud services not available
- The Miniserver is currently set as a Client, which prevents Push Notifications from being activated. Push Notifications can only be enabled for a Gateway Miniserver. Please ensure that you are connected to the correct Miniserver and verify that it has not been mistakenly configured as a Client in Loxone Config.

[
![Miniserver Client](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/02/Miniserver_Client.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/02/Miniserver_Client.png)

### Possible solutions
- Make sure that your miniserver is connected to the Internet. For more information, see [here](https://www.loxone.com/enen/kb/no-internet/).
- Check the registration of your miniserver and ensure that it is registered.