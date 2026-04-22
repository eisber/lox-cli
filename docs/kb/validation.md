# Validation

Source: https://www.loxone.com/enen/kb/validation/

---

#### Monitor signals within a specific value range with the validation for analogue inputs.

## INTRODUCTION

It often makes sense to limit the allowed range of an analogue input signal either to detect incorrect values such as floor temperature being too high or to detect a faulty sensor. Validation is especially important with temperature sensors.

To use validation you simply need to check the tick box labelled ‘Use validation’ in the properties of the input.

![Screenshot Example Validation Function Block Properties Loxone Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Validation_Properties.png)

## FURTHER INFORMATION

The allowed value range is defined by the two fields in the input properties, minimum value and maximum value. There is also a field for setting what the default value should be if this range is exceeded.

If the value of the signal is between the minimum and maximum values then the actual value of the input is output at AQ on the input. However if the input value is outside of the range then the default value is set at the AQ output and the Q output will be on.

![Screenshot Example Validation Input Loxone Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Validation_Input.png)

You can see in the diagram below how this will effect what value is output on AQ of the input.

![Diagram Showing The Effects Of Validation On AQ Output Loxone Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Diagram_Validation.png)