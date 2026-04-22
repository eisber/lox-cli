# Mailer

Source: https://www.loxone.com/enen/kb/mailer-service/

---

With a mailer, the Miniserver can send emails for almost any purpose.

The Cloud Mailer is an email service from Loxone, which is available for free after [registering](https://portal.loxone.com/products) the Miniserver.

The Mailer requires your own email account, the Miniserver does not need to be registered for this.

Up to 200 emails can be sent per day per Miniserver.

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)
- [Assign logger, mailer, caller, tracker in the properties window](#AssignMessProp)

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Use for System Status Messages | If checked, you will be informed about important messages and errors regarding your Loxone System. | - |
| Monitor Service | If checked, you will be notified via System Status if an email could not be sent. | - |
| SMTP Server | Address of SMTP Server (i.e.: mail.server.com or 192.168.1.1:587) | - |
| SMTP User | User on the SMTP server | - |
| SMTP password | Password for specified user on the SMTP server | - |
| Sender address | Mail sender address (eg: [email protected]) | - |
| Recipient Address | Mail recipient address (eg. [email protected]) To enter multiple recipient use a semi-colon (;) as separator. | - |
| Subject | If this field is empty, the message content will be used as the subject of the email. When statistics are sent via Mailer, the name of the statistics CSV file will be used as the subject of the email. | - |

---

## Programming example

First, a new Mailer is created under Messages:

![CloudMailer Mailer](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/CloudMailer_Mailer.png)

In the following example we name the Loxone Cloud Mailer "Energy Consumption This Month" and drag the object to the programming page.

In the Cloud Mailer settings we define the text or analog value that will be sent.
For the recipient address, we specify who should receive the email. Multiple recipients can simply be entered separated by semicolons (;).
The selected recipients will receive an email whenever the value at the output "Reading this month" changes.

![CloudMailer example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/CloudMailer_example.png)

![CloudMailer email](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/CloudMailer_email.png)

If you use the Mailer for this example, the data of the email account are also entered.

![Mailer world4you](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Mailer_world4you.png)

The current Miniserver supports all encrypted SMTP servers starting from TLS v1.2.

The Miniserer Gen. 1 basically supports encrypted SMTP servers from SSL 3.0 - TLS v1.2.
However, it supports only some of the so-called ciphers.
Therefore, the Cloud Mailer should be used if encrypted email sending does not work.

In case of low security requirements, it is also possible to switch to a server without encryption.

---

## Assign logger, mailer, caller, tracker in the properties window

Alternatively, in the properties window of various blocks, logger, mailer, caller and tracker can be assigned.
For this, the respective message must only be created and the recipient defined.
The text or value defined in the block is output.

![message aal linked](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/message_aal_linked.png)