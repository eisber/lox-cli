# Formula

Source: https://www.loxone.com/enen/kb/formula/

---

This function block can be used for mathematical calculations. A formula with a maximum of 4 analogue values can be calculated, the result will be provided at the output. An area of application is the calculation of the level of a water tank. The formula is defined in the properties of the function block.

## Table of Contents
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Supported Features](#Functions)

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| R | Result | ∞ |
| E | Error | E.g. an illegal mathematical operation. | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| I1-4 | Value 1-4 | This value can be used in the formula. A constant value can be also be set in properties. | ∞ | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Formula | The 4 inputs are referred to as I1,I2,I3,I4. The following operands are available: +,-,*,/,^ The following functions are available: PI, ABS, SQRT, LN, LOG, EXP, SIN, COS, TAN, ARCSIN, ARCCOS, ARCTAN, SINH, COSH, TANH, RAD, DEG, SIGN, INT, IF, MIN, MAX Trigonometric functions use Radians as the unit of measure, inputs in degrees have to be converted first (e.g SIN(RAD(I1))). Example: (I1+(I2*0,005))/SIN(I3) | - |

---

## Supported Features

+ : Add
- : Subtract

* : Multiply

/ : Divide

^ : Power x^n

PI : PI (~3,14)

ABS : Absolute value ABS(I1) => |I1|

SQRT : Square Root SQRT(I1)

LN : natural logarithm with the base e LN (I1)

LOG : LOG logarithm base 10 LOG(I1)

EXP : Exponential Function EXP(I1) => e^I1

SIN : Sine SIN(I1)

COS : Cosine COS(I1)

TAN : Tangent TAN(I1)

ARCSIN : Arcsine ARCSIN(I1)

ARCCOS : Arc Cosine ARCCOS(I1)

ARCTAN : Arc Tangent ARCTAN(I1)

SINH : Hyperbolic Sine SINH(I1)

COSH : Hyperbolic Cosine COSH(I1)

TANH : Hyperbolic Tangent TANH(I1)

RAD : Angular representation in radians (360°=2PI)

DEG : Angle display in degrees (360°)

SIGN : Signum function (Sign Function) SIGN(I1)

INT : Integer function (removes decimals) INT(I1)

IF : If function, examples:
IF(I1 > 0;1;0)
IF(I1==0;0;1)
Comparison operators: == (equal), != (not equal), > (greater), >= (greater or equal), < (less), <= (less or equal)

MIN : Minimum function. Returns the smallest result of two expressions, examples:
MIN(I1;I2+10)

MAX : Maximum function. Returns the largest result of two expressions, examples:
MAX(I1*I2;100)