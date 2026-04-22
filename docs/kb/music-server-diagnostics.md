# Music Server Diagnostics

Source: https://www.loxone.com/enen/kb/music-server-diagnostics/

---

If your Loxone Music Server is experiencing problems, you will be notified via various messages in the system status. You can find more details about the respective events on the admin page of the Music Server (type the IP address of the Music Server into a web browser). Below you will find information on the causes of the individual messages as well as troubleshooting tips:
- [DNS Error](#nodns)
- [Streaming service account data incorrect](#servicelogin)
- [Poor Internet connection](#slowinternet)
- [Licence errors](#license)
- [No Internet connection](#nointernet)

## DNS Error

Check the network settings on the Music Server Admin page (“http://musicserver.ip.adresse”) and enter another DNS server if necessary. Avoid public DNS servers such as 8.8.8.8 (Google DNS service) and use the IP of your router the DNS server of your Internet provider instead. These can be found by searching online for “ISP DNS addresses” and replace ISP with the service provider (I.E BT).

## Streaming service account data is incorrect

If your Music Server is reporting an error with streaming services, please have a look below at the relevant one.
- [Google Music](#googlemusic)
- [Spotify](#spotify)

### Google Music

When setting up a Google Music account for the first time, you must use your main account (the account under which the Google Music subscription was purchased). You’ll also need to turn off two-factor authentication for your Google account when you set it up, you can reactivate this after successfully logging in to the Music Server. For information on enabling and disabling two-factor authentication on your Google account, click [here](https://support.google.com/accounts/topic/2954345?hl=en&ref_topic=7667090).

### Spotify

You must ensure you are using a Spotify Premium account.

When adding the Spotify account to the Music Server, you will be redirected to a Spotify confirmation page. It is crucial that you allow the Music Server access to your Spotify account. Furthermore, please ensure that you do not close this page or the Loxone app whilst doing this. If you are carrying this out through a web browser and not the Loxone app, a pop-up blocker could block this confirmation page from appearing. Whilst adding Spotify accounts, please ensure that you do not have any popup blockers/filters enabled temporarily. Please also double-check that the confirmation page is referencing the account you have just tried to add and not a different one.

A maximum of 6 Spotify accounts can be added to the Music Server at once. The accounts are available for all users to use, you have to manually add them to zones you want them to be usable in. Spotify only allows for 1 stream per account so if you try to use an account in parallel (I.E play 2 different things in 2 different zones) you will be notified of this.

## Poor Internet connection

This message can appear due to a sub-optimal internet connection. Packet loss on the network or high ping times to the DNS are some examples of what could cause this message.

Check your internet connection with a speedtest and the status of your router. Compare the upload and download speeds with the information listed in the Music Server webUI (http://musicServerIPAddress).

If the information is the same, try to ping the DNS address used by the music server to determine a connection to that DNS address. You can also try to ping your router whilst plugged into the network where the Music Server is to test for packet loss.

If the information is the same between the speed test and what the Music Server is, you need to contact your Internet Service Provider (ISP). If it is only the Music Server being affected, you need to check the network cabling and ensure that the cable between the Music server & router is fault-free. It is recommended to have the Music Server plugged directly into the router and not go through any switches.

## Licence errors

If you receive a Licence error message, please contact Loxone support with the following details.
- Music Server zone count
- Date of purchase
- Music Server Serial Number
- Music Server MAC address

Note: Both of the last items can be found on the underside of the Music Server.

## No Internet connection

This message appears when the music server cannot connect to the Internet at all.

Firstly, go into the router and check to see if the Music Server is reporting back/is showing as online and not as offline. If the Music Server has a fixed IP addressed, check to ensure that nothing else is also on that address and could be causing an IP conflict.

If only the Music Server is affected, please make sure that all network cabling between the Music Server & Router is error-free. On the web UI of the Music Server, please also ensure that the DNS address and default gateway are set correctly. You may also need to use a different DNS address if the one you currently are using doesn’t work.

If required – restart the router and then once that is booted back up, restart the music server.