# Music Alarm Clock

Source: https://www.loxone.com/enen/kb/music-alarm-clock/

---

## Brief: I want to be able to wake up to a music alarm clock.

The standard method of waking up is an audible alarm sound. It does the job however, very few people would argue that being ripped out of your sleep like that is pleasurable. A Loxone Smart Home offers people a range of ways in which they can be more gentle woken from their slumber. The blinds can slowly open to gradually let light in to ease you into the day – which is a great way to wake up. However, something people commonly want is a music alarm clock – to start the day with their favourite tunes gradually fading in.

This can easily be done with Loxone – either alongside other waking up methods such as the blinds opening or in isolation. In this Use Case, we’ll show you how.

## Solution: Using Loxone to create a music alarm clock.

In order to create a music alarm clock, we’ll use the [Loxone Music Server](https://shop.loxone.com/enuk/loxone-music-server.html) – which enables multiroom audio. The Music Server can be controlled through the “Music Server Zone” Function Block – however, it can also be controlled using Zone Commands. These Zone Commands allow a certain playlist to be set for a specific area in the house, the bedroom for example.

After the “Alarm Clock” Function Block has triggered the alarm, a pre-set playlist can begin to play. When the alarm is active, the switch-on volume is lowered to a value 5 through the Analog Selector Switch. To ensure that the music gradually fades in, the volume will be increased every 30 seconds by using the “Pulse Generator” Block.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
[Music Server](https://shop.loxone.com/enuk/loxone-music-server.html)
[12 Channel Amplifier](https://shop.loxone.com/enuk/loxone-12-channel-amp.html) (optional)
[Speaker](https://shop.loxone.com/enuk/speaker.html) (optional)
- [Speaker Built-in box](https://shop.loxone.com/enuk/speaker-back-box.html)
- [Speaker Back Box for Suspended Ceilings](https://shop.loxone.com/enuk/speaker-back-box-mdf.html)
- [Wall speaker](http://shop.loxone.com/enuk/wall-speaker.html) (optional)

### Configuration:

[
![Music Alarm Clock - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite_2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite_2.png)

[
![Music Alarm Clock - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite_1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite_1.png)

[
![Music Alarm Clock - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite.png)

### Download the sample file:

### Alarm Clock Favourite

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-99-Alarm-Clock-Favourite.loxone)

## Why you and your customer should consider implementing a music alarm clock?

Let’s face it, being woken up isn’t one of the nicest things in the world. However, it can be far more pleasurable if it’s done in a gentle and considered way – as it is with a music alarm clock. No more being ripped out of your sleep by a dreaded alarm tone.

To make it an, almost, enjoyable experience – you can combine a music alarm clock with a light-based alarm clock (either natural or artificial) for the perfect start to the day.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)