# Analogue Multi Click

Source: https://www.loxone.com/enen/kb/analogue-multi-click/

---

## INFORMATION

This block is similar to the Multiclick block except it does not have the pulsed outputs Q1 to Q4. The Tr input will detect whether there has ben a single, double, triple or quadruple click with a switch. Depending on the multiclick a different value will be output at AQ.

Parameter M allows you to adjust the time between a double click, so the slower you want your double click to be the larger M needs to be. The parameters V1 to V4 allow you to set the value that will be output with the different clicks.

A pulse on input R will set AQ to 0.

## OVERVIEW

#### INPUTS

> **ℹ️ Note:** Trigger

#### PARAMETERS

> **ℹ️ Note:** Maximum time span between two pulses (s)

#### OUTPUTS

> **ℹ️ Note:** Analogue output depending on the click.