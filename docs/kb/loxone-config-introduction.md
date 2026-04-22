# Loxone Config Introduction

Source: https://www.loxone.com/enen/kb/loxone-config-introduction/

---

## USING LOXONE CONFIG

Our software is designed to map control tasks for the Miniserver in the form of PLC logic. The configuration software can do the following essential functions:
- Create programs for the Loxone Miniserver
- Store programs in a file (ending .LoxPLAN)
- Simulate programs offline
- Create a user interface which is generated automatically
- Set up logging, statistics and notification features
- Test the program in online mode with the Miniserver in a real time view
- Options to print out hard copies

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)

The size of a Loxone Config file must not exceed 5 MB.

#### SYSTEM REQUIREMENTS

The configuration software is designed for use with the operating systems Windows XP, Windows Vista, Windows 7 and 8. Program settings like window positions and the like can be saved when you exit the application in the registry of your computer. The software can also be run on a Mac by having multiple operating systems using programs such as Parallels or Bootcamp.

#### DOWNLOAD

The latest version of the configuration software can always be found on our [download page](https://www.loxone.com/enen/support/downloads/).

## OBJECT HELP

Our configuration software provides you with a detailed help manual which you can access this by clicking the ‘i’ in the top right-hand corner of any function block. The help file describes all of the functions, inputs, outputs and parameters for each function block.

The Object Help window can be moved around the screen or pinned to the side as desired.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
You may also find our [example configurations](https://www.loxone.com/enen/kb/videos-sample-files/) useful.

![Loxone Config Information Button](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Information_Popout.png)

In Windows Explorer folder options select “Hidden files, folders and drives show”. This will make the folder ProgramData visible so you can save templates to Loxone Config so they show in the pre-defined devices menu. Select C:/ProgramData/Loxone/Loxone Config current version/ENG. The templates for RS485, RS232 and Modbus are saved in the Comm folder and virtual outputs are saved in the folder VirtualOut. Remote control templates are saved in the RC folder.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)

You should backup this folder ENG as installing new versions of Loxone Config in the future will overwrite this folder.