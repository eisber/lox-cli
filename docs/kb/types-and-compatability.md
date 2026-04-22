# Types and Compatibility

Source: https://www.loxone.com/enen/kb/types-and-compatability/

---

## SUPPORTED BROWSERS
- Google Chrome (from Version 8)
- Apple Safari (from Version 5)
- Mozilla Firefox (from Version 4) *
- Apple Safari on iPod/iPhone and iPad (from iOS 4.2)

** Firefox Version 4 has their websocket disabled by default. Please see below.*

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
The Android mobile browser currently does not support WebSockets. Support will come probably in a future version of the Android operating system.

## ACTIVATION OF WEBSOCKETS IN MOZILLA FIREFOX 4 AND OPERA 11

In Firefox 4 and Opera 11, you must activate the WebSockets before being able to use the WebSocket client:
- Open the browser and type ‘about:config’ in the address bar
- Confirm that you want to continue when a warning notice appears (Firefox only)
- Type ‘WebSocket’ in the filter
- For Firefox: Set the value of ‘network.websocket.override-security-block’ to true
- For Opera: Tick ‘Enable Web Sockets’ and click ‘Save’.

## TROUBLESHOOTING CONNECTION PROBLEMS

#### CLEAR THE CACHE

If your browser appears to hang or freeze when loading the web interface then it is often worth clearing the cache of your browser to clear any internal processes to improve the performance.

The keyboard shortcut for Google’s Chrome browser is “CTRL” + “F5” to clear the cache.

For Safari this is “CMD” + “ALT” + “E”.

With Internet Explorer click the menu “View” and then go to “Internet Options”. In the section “Temporary Internet Files” you can clear the cache by clicking on the button “Delete Files”.

You can find a tutorial online for clearing the cache in Mozilla Firefox.

#### SET PORT TO 80

In the Miniserver’s admin interface (http://<MiniserverIP>/admin) set the HTTP port to 80 and try connecting again.

If you want to use a different port than the port 80, you will need to adjust the port forwarding settings on your router.

#### ADD EXCEPTION ON ANTIVIRUS AND FIREWALL SOFTWARE

Add the address of the Miniserver as an exception in the network scanner on your firewall and antivirus software.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
In the current version of the Avast!® free virus scanner, adding an exception rule has no effect. A connection is possible only with complete deactivation.In this case, we recommend using a different virus scanner.