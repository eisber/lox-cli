# Logger

Source: https://www.loxone.com/enen/kb/logger/

---

The logger writes events to a log file.

Log data is stored on the SD card of the Miniserver by default, alternatively it can be transferred to a syslog or UDP server.

Any number of log files can be created.

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)
- [Assign logger, mailer, caller, tracker in the properties window](#AssignMessProp)

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Address of Logger | Location where logs are sent or saved Example: /log/user.log, /dev/syslog/192.168.1.1, /dev/udp/192.168.1.1/1234 (if left empty = logs data to /log/def.log) If the address end in .log, the log will be saved as a text file. If the address end in .csv, the log will be saved as a CSV file. | - |
| Send Email | Interval for sending the Log via Mailer. The sent mail will contain only log entries from the last interval. The file size limit is 1 MB. | - |

---

## Programming example

First, a new logger is created under Messages:

![logger new](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/logger_new.png)

In the following example we give the logger the name "Access House" and drag the object onto the programming page.

In the properties window, a text can be defined for the two states of digital values or at analog value changes, or values of analog or text outputs can be written to the log file with <v>.

In the logger address, we specify the log file to be written to. In our example, a separate log file with the name "Access" is generated.
If this field is left empty, it will be written to the default log def.log.

In the following example, the text of the output "Ula" is written to the log file.
The data is stored on the SD card and can be retrieved from the Miniserver via the FTP (ftp://User:Password@IPMiniserver/log/) or via the webservice command [IPMiniserver]/dev/fsget/log/LogfileName.log.

The log can also be sent via email at a specified interval. To enable this, simply create a [Mailer](https://www.loxone.com/help/Mailer) and specify the recipient.

![logger example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/logger_example.png)

![logger webservice](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/logger_webservice.png)

---

## Assign logger, mailer, caller, tracker in the properties window

Alternatively, in the properties window of various blocks, logger, mailer, caller and tracker can be assigned.
For this, the respective message must only be created and the recipient defined.
The text or value defined in the block is output.

![message aal linked](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/message_aal_linked.png)