# Operating Modes

Source: https://www.loxone.com/enen/kb/operating-modes/

---

With Operating Modes, functions can be started depending on the date or on specific occasions. This allows, for example, an adjustment of the heating times in winter or during absence.

Auto configuration uses the predefined operating modes and links them accordingly.

## Table of Contents
- [Programming example](#baseconf)
- [Create Operating Mode](#createMode)

---

## Programming example

There are already predefined operating modes in the periphery tree:

![opm predef](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/opm_predef.png)

### Activate operating modes via output

To use an operating mode as a digital output, select the mode in the periphery tree, click on "Add Output Reference" in the menu bar and connect it to the desired logic:

![opm output](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/opm_output.png)

In this example we activate the operating mode "All Out" via a switch:

![opm outputExample](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/opm_outputExample.png)

### Activate operating modes via operating time

We assign the operating mode "Vacation" to the created operating time "Winter Vacation". On the set date, the linked operating mode then becomes active:

![opt editEntry](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/opt_editEntry.png)

For country-specific holidays, the operating modes are already stored.

### Using operating modes in programming

If an operating mode is used as a digital input, the mode can be dragged from the periphery tree to the programming page and connected.
This allows an operating mode to trigger certain actions when it is active.

![opm inputExample](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/opm_inputExample.png)

---

## Create Operating Mode

With "Operating Mode" a user defined Operation Mode can be created:

![opm newMode](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/opm_newMode.png)

This operating mode can then be activated again via a logic or operating time or control actions via digital input.