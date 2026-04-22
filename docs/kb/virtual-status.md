# Virtual Status

Source: https://www.loxone.com/enen/kb/virtual-status/

---

The Virtual Status is used to display values in the user interface.

## Table of Contents
- [Inputs](#Input)
- [Properties](#Property)
- [Programming example](#baseconf)

---

## Inputs

> **ℹ️ Note:** Input

---

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Use as digital input | If checked, the analog input is used as a digital input. | - |

---

## Programming example

In this example, a moving average is first formed from the temperature value and then displayed in the user interface via the Virtual Status.

![vstate config](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/vstate_config.png)

![vstate app](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/vstate_app.png)

Activating the statistics in the Virtual Status has the advantage that the data remains saved if a sensor or the logic before the block is changed.