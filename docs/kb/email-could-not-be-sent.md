# Email could not be sent

Source: https://www.loxone.com/enen/kb/email-could-not-be-sent/

---

If the miniserver cannot send an email immediately, it will try to send the message again at different intervals. If the email could not be sent even after the fourth attempt, you will be notified via a [system status message](https://www.loxone.com/enen/kb/systemstatus/). In addition to the description of the reason for the error, this system message also contains the last email that could not be sent

### Possible Solutions
- To send an email, the miniserver needs an active internet connection. So first check the network connection and network configuration of the miniserver. You can find more information [here](https://www.loxone.com/enen/kb/no-internet/).
If you use the [Loxone Cloud Mailer](https://www.loxone.com/enen/kb/mailer-service/), the Miniserver must be registered. This registration is free of charge and can be done [here](https://portal.loxone.com/products) (a user account on the Loxone homepage is required for this).
If a custom SMTP server is used, there are the following requirements:
- IP or host name and port of the mail server must be entered correctly.
- The login data to the SMTP server must be correct.
- The connection to the SMPT server must use the EHLO command and the server must understand ESMTP.
- If the server indicates that TLS is possible, the e-mail is transmitted in encrypted form.