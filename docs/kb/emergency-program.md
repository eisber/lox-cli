# Emergency Program

Source: https://www.loxone.com/enen/kb/emergency-program/

---

In the event of an emergency program, the Loxone Miniserver can remain operable even in case of failure.

## BASIC FEATURES

With the Emergency document you can control the basic functions of the Smart Homes in case of failure or file corruption.

If within 10 minutes at least 4 Miniserver crashes (or tries to read the main Config file and fails) the emergency document is found loaded.

This document will remove all blocks from the programming and actuators are used are directly to the virtual inputs/push buttons.

The outputs of the Miniserver can then be operated via the web interface or application.

## FUNCTION AND RECOVERY

In the UI Virtual inputs with the names of the actuators are available for all actuators used.

![File 000 1](/enen/wp-content/uploads/sites/3/2017/04/File_000-1.png)

The Miniserver can therefore continue to operate. You can switch the lighting or use the blinds for example.

To regain the full configuration of the Miniserver connect your Miniserver to the network. The original program is stored on the Miniserver (if you don’t have a saved offline backup of the system already).

To download the config file, connect to the Windows Explorer (ftp: // user: password @ ipMiniserver: FTPPort) or an FTP program (eg Filezilla) with the Miniserver.

In Program folder the files are stored with the latest timestamp.

Warning : Before you save the program back into the Miniserver, make sure that the cause for the crashes has been eliminated. If you are unsure at any point, please contact our [support](https://www.loxone.com/enen/about-us/contact/).