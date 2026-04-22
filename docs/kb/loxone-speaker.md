# Loxone Speaker

Source: https://www.loxone.com/enen/kb/loxone-speaker/

---

This page is no longer updated. The information on this page was created for the Loxone Music Server and Loxone Speakers. Both products are no longer available and have been replaced by the [Audioserver](https://www.loxone.com/enen/kb/audioserver/) and the Install Speaker 7 Passive.

#### Information about the Loxone Speakers and Loxone Music Server.

## Content:
- [Technical specification](#tek)
[Wiring the Loxone Speaker](#Wyre)
[Basic diagnosis](#diag)

## Technical Specification

|  | Loxone Speaker | Loxone Wall Speaker |
| --- | --- | --- |
| Dimensions | Diameter: 224mm | 440x180x80mm |
| Installation diameter | 204mm | N/A |
| Ceiling installation depth | up to 40mm | N/A |
| Power usage | 60 Watts | 40 Watts |
| Maximum volume | 86dB | 86dB |
| Frequency range | 60-20000 Hz | 20-20000 Hz |
| Mono/Stereo switch | Yes | Yes |
| Peak power handling for Woofer (60w) | Yes | No |
| Speaker backbox volume | 10L | N/A |
| Depth | 90mm |  |
| Protection class | IP54 | IP20 |
| Dimensions of round cover | 232x3mm | N/A |
| Mass | N/A | 3.4Kg |
| Impedence | 8 Ohms | 8 Ohms |

![Icon To Represent Downloading Files For Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Download.png)
[Datasheet for the Loxone Speaker](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Loxone_Speaker_Datasheet.pdf)
[Datasheet for the Loxone Wall Speaker](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/10/200153-Loxone-Wall-Speaker.pdf?x16511)

##

##

## Wiring the Loxone Speaker

![Wiring Diagram For Loxone Speakers](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/10/EN_KB_Diagram_Loxone_Speaker.png)
- Cable cross-section between a speaker and the music server: up to 20m should be 1.5 mm², from 20 to 50m should be 2.5mm². If the cable length is more than 50m, the cross-section rule must be adapted accordingly. You can use any standard speaker wire.
- Speaker installation boxes for cast concrete ceilings should have a volume of 7-10l as well as suitable options for mounting the speaker.

For more assistance on cabling or wiring the Music server please follow [here](https://www.loxone.com/enen/kb/audio-cabling/ ‎).

## Diagnosis

If you are encountering hum from one or more speakers, then please follow the basic troubleshooting guide below

#### Ensure the following in advance
- The gain level directly on the channel of the amplifier should be set as low as possible. If this is turned up too high, this can cause a buzzing sound. The controller for this can be on the back of the amplifier at the top of the respective channel in the form of a small potentiometer.
- If you have installed the amplifier and the music server in a rack, make sure that the housings of the devices are not grounded. It is best to place the music server on the amplifier so that the case does not touch.
- The Music server and amplifier are on the same circuit.

#### Hum is audible in all zones
- Is there a Sat or DVBT cable near the amp / music server? Keep them away as far as possible, as otherwise disturbing influences can occur.
- Is the noise also to be heard if you connect headphones directly to the output of the Music Server? If yes, please contact support.

#### Hum is only audible in one or more zones
- Is the humming also present if, for example, a cell phone (ungrounded device) on the Line In of the amplifier?

If so, please check the cabling of the speakers. A 230V cable in the immediate vicinity of an inferior speaker cable can also induce interference voltages.

Once you have checked all the tips that we have described in your system, there is still the possibility to place a mass separation filter between amplifier and music server, which can filter out the interference voltages.