# URL Scheme Smart Home App

Source: https://www.loxone.com/enen/kb/visualisation/

---

## Loxone Smart Home 4.0.2 URL-Start/URL Scheme

You need at least Version 4.0.2 of Loxone Smart Home App.

If you need any UUID’s for navigation info, you can find them in our LoxAPP3.json Structure-File (just enter “<HOST>/data/LoxAPP3.json” in your Browser)

If you have our old App (Loxone Classic) installed, please uninstall it in order to use it without problems – otherwise:

on Android you get asked, which App you want to use

on iOS it’s undefined which one launches

#### App Schema:

loxone:// simply opens the Smart Home App

#### Examples:

loxone://ms?mac=<MAC>

loxone://ms?host=<IP>

loxone://ms?host=<IP>&usr=<USER>&pwd=<PASSWORD>

loxone://ms?host=<URL>&usr=<USER>

loxone://ms?host=<IP>&usr=<USER>&pwd=<PASSWORD>&loc=favorites

loxone://ms?mac=<MAC>&loc=home

loxone://ms?mac=<MAC>&loc=weather

loxone://ms?mac=<MAC>&loc=room%2F<roomUUID>

loxone://ms?mac=<MAC>&loc=category%2F<categoryUUID>

loxone://ms?mac=<MAC>&loc=control%2F<controlUUID>

ms?host=<HOST>

ms?mac=<MAC> (User+PW is not necessary!)

ms?host=<HOST>&usr=<USER>

ms?host=<HOST>&usr=<USER>&pwd=<PASSWORD>

ms?host=<HOST>&usr=<USER>&pwd=<PASSWORD>&loc=<LOCATION>

ms?mac=<MAC>&loc=<LOCATION>

<HOST> can be IP or URL of Miniserver

<MAC> MAC Adress of Miniserver (Miniserver must be stored in the App!)
- Attention: the parameters (URIComponents) (<HOST>, <MAC>, <USER>, <PASSWORD>, <LOCATION>) must be URL-Encoded
- http://en.wikipedia.org/wiki/URL-Encoding
- http://www.w3schools.com/jsref/jsref_encodeURIComponent.asp

#### App-Navigation Informations:

ms?host=<HOST>&loc=<LOCATION>
- keep URL Encoding in mind! eg. “/” → “%2F”

#### <LOCATION> Options:

**navigate to a screen:

**

> **ℹ️ Note:** Home-Screen

navigate to a room:

room%2F<roomUUID>

navigate to a category:

category%2F<categoryUUID>

navigate to a control:

control%2F<controlUUID>