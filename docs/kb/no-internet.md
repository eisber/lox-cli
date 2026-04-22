# No Internet Connection

Source: https://www.loxone.com/enen/kb/no-internet/

---

Some of the functionality of your Loxone Smart Home requires an active Internet Connection. If this connection is not available, the [Weather Service](https://www.loxone.com/enen/kb/weather-service/), for example, cannot retrieve current data, [Push Notifications](https://www.loxone.com/enen/kb/push-notifications/) cannot be sent, nor can [E-Mails](https://www.loxone.com/enen/kb/mailer-service/), and calls using the [Caller Service](https://www.loxone.com/enen/kb/caller-service/) cannot be made. Your Miniserver automatically checks at regular intervals whether you are connected to the Internet. In case of no available internet connection you will be informed about this via [System Status](https://www.loxone.com/enen/kb/systemstatus/).
The following section provides information about possible causes and [Troubleshooting](#troubleshooting). To learn how to set your system as offline and how to disable automatic internet connection checking, click [here](#offline-mode).

## Possible Causes and Troubleshooting

A distinction is made between the following cases:
- [No Internet connection across the entire system](#miniserver)
- [No Internet connection on a single client](#client)

### No Internet Connection across the Entire System

If you Miniserver tells you that there is no active internet connection, check the follow points:
- Test if your local network has an active internet connection. For example, visit a website of your choosing on your smartphone or desktop computer. If this does not load, the problem is with the Internet connection on your local network.
- Check both the network configuration of your local network and that of the affected Miniserver. Information on the network configuration or initial configuration of a Miniserver can be found [here](https://www.loxone.com/enen/kb/miniserver-setup/).

### No Internet Connection on a Single Client

If one of the client Miniservers in a [Client-Gateway System](https://www.loxone.com/enen/kb/miniserver-clientgateway-concentrator/) (with concentrator) informs you that it does not have an Internet Connection, but the gateway Miniserver does, check the network configuration of both your local network and that of the affected Miniserver. Information on the network configuration or the initial configuration of a Miniserver can be found [here](https://www.loxone.com/enen/kb/miniserver-setup/).

## Enable Offline Operation

If you want to use your system without an Internet Connection, you can deactivate the Automatic Internet Connection check in the follow ways:
- Execute the corresponding action “Offline-Betrieb aktivieren” for the respective [System Status](https://www.loxone.com/enen/kb/systemstatus/) message.
- In Loxone Config with the help of the setting “Offline-Betrieb”, which can be found in the settings of the System Variable “Internet Connection.”.