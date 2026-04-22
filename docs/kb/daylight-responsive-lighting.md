# Use Case – Daylight Responsive Lighting

Source: https://www.loxone.com/enen/kb/daylight-responsive-lighting/

---

## Brief: Configure lighting that responds to natural daylight levels.

In most of the European countries, the requirements for suitable workplace lighting are defined by European guidelines. Particular attention is paid to the levels of light. For a VDU workstation, for example, sufficient light is defined as at least 500 lux. This requirement is usually referred to as daylight responsive lighting. It ensures that if natural lighting levels are not sufficient, then artificial light sources are brought on to a suitable level to ensure the lux level is maintained.

To ensure that the required lux level is as consistent as possible, constant lighting control is used in commercial premises and offices in particular. If sufficient daylight enters a room, little or no artificial light is added. If little or no daylight enters a room, artificial light sources provide an adequate lighting level.

Daylight responsive lighting is, therefore, the daylight-dependent control of artificial light sources with the aim of maintaining constant levels of light.

## Solution: Commissioning daylight responsive lighting.

Since December 2018, the Dutch “Goede Doelen Loterijen” (in English: “Charity Lottery”) has its headquarters in a renovated industrial building in the south of Amsterdam. The building is a prize-winning prime example of sustainable building renovation in the Netherlands.

The lighting of the approximately 600 workstations is controlled by a daylight responsive lighting system. This ensures a constant light level of 500 lux at the height of the work surface (approx. 75 cm).

For this purpose, numerous Loxone Motion Sensors first measure the current natural light levels. Controlled by the Loxone Miniserver and depending on the currently prevailing lux values, some 40 DALI Extensions take over the control of the installed lights.

The difference in lighting intensity between the installation height of the Motion Sensors and the working height is adjusted by a specific correction factor.

In order to avoid excessively unsteady lighting conditions (even flickering), the intensity of the light is not adjusted immediately but over several minutes, responding just as natural daylight would. The adjustment is carried out by a gentle dimming up or down the light intensity as pleasant and discreet as possible.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [DALI Extension](https://shop.loxone.com/enuk/dali-extension.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)

### Configuration:

[
![Daylight responsive lighting - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Config-Use-Case-36-Constant-Light-Control.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Config-Use-Case-36-Constant-Light-Control.png)

### Download the sample file:

### Constant Light Control

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/use-case-36-constant-light-control.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/use-case-36-constant-light-control.loxone)

## Why you and your customer should consider daylight responsive lighting?

Light influences many facets of our lives, especially in the workplace therefore, appropriate lighting is essential. It not only influences our actual vision but also our activity levels and productivity. A number of physiological studies have looked at how daylight responsive lighting has an impact on our psyche. Inadequate lighting can lead to symptoms of fatigue, lack of concentration, headaches which are all counterproductive.

Constant lighting control not only creates ideal working conditions – because only the amount of artificial light actually required is added, constant lighting control also helps save energy.

So, maintaining constant light levels in office spaces by commissioning daylight responsive lighting has a lot of advantages for employees and their productivity.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)