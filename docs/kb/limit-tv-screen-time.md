# Limit TV screen time for kids

Source: https://www.loxone.com/enen/kb/limit-tv-screen-time/

---

## Brief: I want to limit TV screen time for my kids to 2 hours.

The control of TV consumption is very helpful to make sure that the children are not sitting in front of the TV all day. So how can you limit TV screen time for kids so that they do not end up watching more than, for example, two hours per day? It is possible to do this. Additionally, with Loxone, a message can be played through the multiroom audio in a specific room to announce that there are only 5 minutes of viewing time remaining before the TV automatically turns off 5 minutes later.

Regardless of whether the time is used up in one go or spread over the day, the total time should not exceed two hours. So there is no more discussions/arguments because the TV goes off after the two hours, without you having to physically to do this at the exact mark of 120 minutes.

Of course, you can also control and monitor whether the TV is currently in use and how long it has already been in use on that day via the Loxone App. Yet another great way to limit TV screen time for kids – if the two hours limit has nearly been reached and there are still plenty more hours in the day, you can encourage them to take up other activities, such as playing outside or reading a book in order to save some time for later.

## Solution: How to restrict the amount of time that the TV can be on for.

With Loxone it is possible to monitor the power consumption of a device, such as a television, and to measure the duration of time it has been on for. In this example, a Smart Socket is used for power measurement. For this to work effectively, the socket must not be accessible for children.

The first step is to check if the power at the Smart Socket exceeds a certain value, here as an example 30W. If the measured value is higher, a pulse is sent to the counter every minute. When the limit has been reached, this counter switches off the relay of the Smart Socket via output Q. Also, 5 minutes before the time expires, a Text-to-Speech message is sent to the children’s room saying “Please switch off the TV, otherwise, it will switch itself off in 5 minutes”, see module Status. You can set the maximum TV time via the virtual input “TV time duration” and view the duration of the TV time via the virtual status “Today’s TV time”.

Hardware:
- [Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Air Base](https://shop.loxone.com/enuk/air-base-extension.html)
- [Smart Socket](https://shop.loxone.com/enuk/smart-socket-air.html)
- [Music Server](https://shop.loxone.com/enuk/loxone-music-server.html) (optional for Text-to-Speech)

### Configuration:

[
![Limit TV screen time for kids - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-43-Smart-TV-Times-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-43-Smart-TV-Times-1.png)

### Download the sample file:

### Smart TV Times

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-43-Smart-TV-Times.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-43-Smart-TV-Times.loxone)

## Why you and your customer should consider a smart setup to limit TV screen time?

Adults understand a film from a rational distance, they know that it is a fictional form of presentation. Children under the age of six cannot distinguish between fiction and reality in films. According to the findings of American researchers, the longer toddlers watch television, the more often they notice restlessness and inattentiveness in the first years of school. There are various studies on this subject however, with Loxone it is possible to limit the time your children can watch television easily. After the defined time, the TV goes off and of course cannot be switched on anymore. So even in your absence, the TV time cannot be exceeded. You can check or control the status of the TV at any time via the Loxone App.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)