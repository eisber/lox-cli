# Text-to-Speech via Music Server

Source: https://www.loxone.com/enen/kb/text-to-speech-via-music-server/

---

## SETUP GUIDE

			In order to use the Text-to-speech functionality of the Music Server, you will need to set up a small bit of logic beforehand using a specific block. Each Music Server zone can play its own TTS signal and they can also play the same thing synchronously. As of version 8 onwards, memory flags can transport text data as well, so they are perfect for this task.

The first item you will need is a state block. More information on this can be found on its own documentation page [here](https://www.loxone.com/enen/kb/state/).

With the state block, you will need to setup trigger logic and then have a text output for that line. The easiest and one of the most common ways is to simply use a switch/button that triggers input 1 of the state block that then emits a text signal.

![tts 1 300x187](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/02/tts-1.png)

By then using the TQ output (dedicated text output), you can then forward the text signal out to anything that can accept one. The TTS input on a Music Server is the best for this, as you will then get direct feedback for how it behaves.

![tts 2 300x60](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/02/tts-2.png)

You can also use memory flags to transfer this same text signal across multiple pages.

![tts3 300x157](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/02/tts3.png)

The final edit that needs to be made is in the Music Server block itself. There is a parameter called Vt and this is the %age volume that TTS will play at. By setting this to a number you want (0-100), the TTS will play at that volume and so it does not adopt the volume of the zone, it has its own volume.

![tts4 300x104](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/02/tts4.png)

## FINAL CONSIDERATIONS

			Once setup has been completed, there are a few things that you and the end client need to be aware of and a couple of extra tweaks that can be made in order to get the most out of your TTS setup.
- State blocks appear in the app. Be sure to untick “Use” if you do not want this to be seen.
- Please note that the language cannot be changed individually with the state block and the text will play in the language that the Config is in.
- We do not have our own custom TTS service and so we cannot alter the way certain things are said. With TTS, it can be more beneficial to spell words phonetically and other unconventional ways to get the proper pronunciation if you encounter that issue.
- If the Music Server and/or amplifier is sleeping beforehand, you need to give it a few seconds to wake up, much like with when playing music.
- If music is playing, the Music Zone will pause, play the TTS sound and then resume.