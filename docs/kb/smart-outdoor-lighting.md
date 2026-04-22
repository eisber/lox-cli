# Smart outdoor lighting

Source: https://www.loxone.com/enen/kb/smart-outdoor-lighting/

---

## Brief: I want to have smart outdoor lighting.

Illuminating the garden and house at night with smart outdoor lighting is not only about security, but outdoor lighting can also be used to set atmospheric lighting accents. For safety reasons, paths and stairs without a clear light-dark contrast should be illuminated evenly. Even with just a few steps up to a door, for example, lighting proves functional. And overall, a well-lit garden can go a long way in deterring burglars.

When it comes to complementing the architecture, there are various ways smart outdoor lighting can make the world of difference. For example, eaves and balconies are perfectly placed for surface-mounted lights to illuminate features. Terraces can be framed with uplighting. And lights with earth spikes create “wow” effects in planters and landscaped areas.

LEDs are ideal for outdoor lighting due to their durability and low energy consumption. In addition, with the right LEDs, coloured light moods can also be set. You should ensure that any lighting or light housing used outdoors has the necessary IP rating depending on where they are being installed.

When combining functional and atmospheric light, a mixture of time-dependent and presence-dependent lighting is ideal for smart lighting in the garden.

While a basic lighting mood provides the right ambience in the garden depending, this can be coupled with brighter lighting turning on, or the dimming increased on existing lights, as someone approaches the house. The lighting, therefore, changes from decorative to functional. Then, if movement is not detected for a period thereafter, the dimmed lighting returns to a dimmed mood.

## Solution: How to set up smart outdoor lighting for the garden.

Smart outdoor lighting in the garden does not have to be active all night. For example, it is likely enough if it provides the right ambience in the garden between dusk and 2:00 a.m. with 20% brightness. This also has positive effects on energy efficiency.

The “Ambient light” mood in the garden is activated using the time parameter of ‘Pulse for Dusk’ and bringing on all lighting to 20% brightness. A pulse at 2am can then take care of turning this ambient lighting off.

If someone approaches the house while the “Ambient light” mood is active, a motion sensor is then responsible for detecting movement. This, in turn, ensures the Lighting Controller mixes in the scene “Movement after dusk” (and a correlating [Operating Mode](https://www.loxone.com/enen/kb/operating-modes/)) to bring on brighter lighting for security – if, of course, artificial lighting is needed at that time. In our use case, when this mode is activated, two lights above the front door are dimmed up to 100%, while the remaining lights in the garden remain at 20% brightness.

If the light scene “Movement after dusk” deactivates after a period of no motion, it simply turns off but the light scene “Ambient light” remains active and all connected lighting stays at or returns to 20% brightness.

This is made possible by the option to mix lighting moods within the [Lighting Controller](https://www.loxone.com/enen/kb/lighting-controller/) Function Block. This works on the basis of determining if the mood to be mixed in has any light sources at a brightness higher than what is currently on, and then brings these on. Any lighting that is not included in the mood that is being mixed in will simply stay in lit as they are.

In our example for smart lighting in the garden, we’ve included:

Ambient light: 10 lights at 20%

Movement after dusk: 8 lights off, 2 lights on 100%

As soon as the “Movement after Dusk” operating mode is deactivated (based on no motion for the set period of time), the lighting returns to the previous Operating Mode that was on.

Hardware:
- [Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)
- [RGBW 24V Dimmer](https://shop.loxone.com/enuk/rgbw-24v-dimmer.html)

### Configuration:

[
![Nr. 56 Smart Outdoor Lighting](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Nr.-56-Smart-Outdoor-Lighting.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Nr.-56-Smart-Outdoor-Lighting.jpg)

### Download the sample file:

### Smart Outdoor Lighting

			[Config 14.2.6.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-56-Smart-Outdoor-lighting.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-56-Smart-Outdoor-lighting.loxone)

## Why you and your customer should consider smart lighting in the garden.

Well-configured smart outdoor lighting not only helps your customers to find their way around the garden better at night, it can also discourage potential burglars.

Outdoor lighting often used by landscape designers, with smart outdoor lighting particularly versatile in allowing for different moods to be set based on how the garden is being used – for a quiet night on the terrace or entertaining guests at a BBQ.

With the addition of time control – these varying lighting moods are only activated when it makes sense to do so. This ensures lighting is not left on during the day or when natural lighting is sufficient, which would waste energy.

Overall, having smart lighting in the garden with different lighting moods can offer practical task lighting, focus lighting for landscaping and architectural features, and a creative source of lighting for entertaining.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)