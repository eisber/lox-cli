# Thermal Solar Controller

Source: https://www.loxone.com/enen/kb/solar-pump-controller/

---

This function block is used to control a solar thermal system. The solar pump is activated based on collector and storage tank temperatures. Up to 5 hot water or buffer storage tanks can be integrated, one output per storage tank is available to control the storage tank valve. By means of efficient storage tank selection, the block ensures that a maximum of thermal power is stored. Depending on the system design, additional programming may be required.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Basic Programming](#Basic)
- [Configuration](#Config)
- [Regulation](#Control)
- [Storage selection](#TankSelect)
- [Manual mode](#Manual)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| ϑc | Current collector temperature | ° | ∞ |
| ϑs1-5 | Current storage temperature 1-5 | ° | ∞ |
| Sel | Select storage | Defines which buffer storage should be heated when in manual mode (M). 0 = Solar pump stays Off. | - | 0...5 |
| M | Manual mode | Activates manual mode and heats up buffer storage specified at input (Sel). Manual mode is disabled when the collector is overheating. | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Sp | Solar pump | Digital output for controlling the solar pump. | 0/1 |
| Spa | Solar pump analog (0-10V) | Analog output (0-10V) for controlling the solar pump. | 0...10 |
| S1-5 | Storage 1-5 state | Outputs are active when the respective buffer is allowed to heat up. | 0/1 |
| S | Current storage | >0 = Current storage that is being heated up. 0 = Solar pump is Off. -1 = Maximum temperature of all storages is reached. | -1...5 |
| Minϑs | Min. storage temperature exceeded | Active when all buffer storages have exceeded their target temperature. | 0/1 |
| Maxϑs | Max. storage temperature exceeded | Active when all buffer storages have reached their maximum temperature. Pump is switched off. | 0/1 |
| Co | Collector overheating | Active when the collector is overheating. | 0/1 |
| Hs | Heating surplus | Active before the last buffer storage reaches it's maximum temperature. Time of activation depends on parameter (ϑHs). | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Pon | Pump switch-on threshold | This value determines how much higher the collector temperature must be compared to the storage temperature in order to start the solar pump. | ° | ∞ | 8 |
| Poff | Pump switch-off threshold | This value determines how much higher the collector temperature must be compared to the storage tank temperature before the pump is switched off. | ° | ∞ | 4 |
| Maxϑc | Max. temperature collector | The pump is disabled when the collector temperature exceeds this value. | ° | ∞ | 120 |
| Minϑs1-5 | Min. temperature storage | When a buffer storages reaches set temperature the next storage is heated up is loaded until all storages have reached at least their minimum temperature. | ° | ∞ | 60 |
| Maxϑs1-5 | Max. temperature storage | When all buffer storages have reached their maximum temperature, the pump is switched off. | ° | ∞ | 70 |
| ϑHs | Temperature heating surplus | This value determines how many degrees the last memory must be away from its maximum temperature before output (Hs) is activated. | ° | ∞ | 5 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Configuration | Configure the numbers and names for all thermal stores. | - | - |
| Number of Thermal Stores | Number of buffers that can be independently heated up from the solar panels. (Maximum: 5 buffers) | 1...5 | 1 |

---

## Basic Programming

The module is used to control a solar thermal system. There are outputs for controlling a solar pump (digital or analog) to heat up up to 5 buffer storages.

![SolarGrundprogrammierung](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SolarGrundprogrammierung.PNG)

---

## Configuration

By double-clicking the block, the configuration dialog opens where the numbers and names of the puffer storages can be assgined.

![SolarDlg](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SolarDlg.PNG)

---

## Regulation

The module has two outputs for controlling a solar pump. One analog output (Spa) and one digital output (Sp). If the solar pump is controlled via a pulse width modulation, use a the [Pulse Width Modulation](https://www.loxone.com/enen/kb/pulse-width-modulation/) on the analog output (Spa).

---

## Storage selection

Tanks are prioritized according to their number. If the collector temperature exceeds the tank temperature, the tank will be heated until it reaches the minimum required temperature.

When all buffer storages have reached their minimum temperature, the function block starts heating up the storages to their maximum. When all storages have reached the maximum temperature, the pump is switched off.

In order to prevent unnecessary switching of the valves, a short locking of one minute is installed when switching between valves.

---

## Manual mode

The function block can also be operated manually. To do this, a buffer storage must be selected at input (Sel). Then the manual mode can be started with input (M). If input (Sel) = 0, the solar stays off and no storage is heated. Manual mode is disabled when the collector is overheating.