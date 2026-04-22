# Workshop Dust Collection

Source: https://www.loxone.com/enen/kb/workshop-dust-collection/

---

## Brief: I want to have an automatic solution for workshop dust collection.

In the commercial wood processing industry, effective extraction of wood dust and wood chips is recommended by the trade association – there are good reasons for this.

Firstly, wood dust and wood chips impair the machines used, by damaging bearings and clogging filters, for example.

But even more importantly, the dust poses very serious health risks. The wood dust from hardwoods such as oak and beech is even suspected of causing cancer. In addition, wood dust and wood shavings can also cause asthmatic and allergic reactions.

On a slightly less serious level, the fact that wood dust gets stuck in shoes and clothing and is thus sooner or later carried from the workshop into living spaces is also annoying. Therefore any wood processing workspace could seriously benefit from a workshop dust collection system.

## Solution: How workshop dust collection can be automated with Loxone.

With Loxone, a workshop dust collection system can be easily implemented. First of all, a suction line is led from a central vacuum cleaner to each workstation in a kind of line topology.

![Workshop Dust Collection System Diagram ](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/IG_Absauganlage-Holzstaub-Holzspa%CC%88ne-Beispiel.png)

If – as in our example – several tools are connected to a central vacuum cleaner by a suction pipe, the individual suction ends should each be equipped with an electronic shut-off valve (NC). This prevents loss of suction power through the various openings.

The central vacuum cleaner as well as all relevant electrical tools such as band saws, drill bits, etc. are connected to the power supply via Smart Socket Air.

When the band saw is activated, for example, the [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) registers the increased power consumption, which in turn activates the vacuum cleaner and opens the corresponding shut-off valve.

**Tip:** In our example, all tools are connected to the power supply via Smart Socket Air. Heavy machines, in particular, are often connected to the mains via 3 phases. In such cases, automated extraction systems for wood dust and wood chips can be implemented by using our [Modbus Electricity Meters](https://shop.loxone.com/enuk/catalogsearch/result/?f=modbus+energiez%C3%A4hler&q=modbus) in combination with fixed cabling.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)

### Configuration:

[
![Workshop Dust Collection - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-12-Woodworking-Dust-Collection-EN-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-12-Woodworking-Dust-Collection-EN-2.png)

[
![Workshop Dust Collection - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-12-Woodworking-Dust-Collection-EN-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-12-Woodworking-Dust-Collection-EN-1.png)

### Download the sample file:

### Dust Collection

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-12-Dust-Collection.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-12-Dust-Collection.loxone)

## Why you and your customer should consider an automatic workshop dust collection set up?

A workshop dust collection system can help to prevent damage to health as well as damage to the machines used. This is of course only if the extraction system is used reliably.

With manual systems vacuum cleaners, shut-off valves, etc. have to be activated manually. The results in the system often not being used, especially during shorter sessions, out of convenience.

An automatic workshop dust collection system is the ideal solution here – especially considering how easy it is to implement.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)