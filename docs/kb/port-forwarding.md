# Port Forwarding

Source: https://www.loxone.com/enen/kb/port-forwarding/

---

## TECHNICAL SUPPORT

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
Please note that as Port Forwarding or Port Redirection is a feature of the router on your network the Loxone support team will be unable to assist you in setting this up. Please ensure you contact your router manufacturer or Internet Service Provider (ISP) for support in this area.

## INTRODUCTION

Port forwarding, sometimes also known as port redirection, is essential for being able to connect to the Miniserver with the app or config when not on the site of your project. This is referred to as Remote Access and is defined as when you are not connected directly to the same network as the Miniserver. For other info on remote access please see [this page](https://www.loxone.com/enen/kb/remote-access/).

## PORT FORWARDING RULE

In order for the Miniserver to be accessible remotely we need to open up a port in the firewall of the router to allow controls traffic through to the Miniserver. There are three key aspects to port forwarding, internal port, external port and the device that the forwarding is associated with. As such there are typically two steps to setting up a port forward, and the first is setting up a port forwarding “rule”.

This could be for example, the external port 7878 is forwarded to the internal port 80 for both TCP and UDP communications.

This simply means that when a device connects to the router from the internet if it requests to use port 7878 then TCP and UDP protocol communications are forwarded (or redirected) through to port 80. Hence the name port forwarding or port redirection.

For a guide on exactly how to do this for your router there is an excellent website called[ PortForward.com ](https://portforward.com/)that has guides for pretty much all common routers with screen shots, they have a long list of routers and their guides [here](https://portforward.com/router.htm).

## ASSIGNING RULE TO DEVICE

Now that a rule has been created, it should now be assigned to a device. This is like instructing the router than when the router passes the communications through on a port to which device is this actually being forwarded on to. This can either be done by MAC Address of the device or by the IP address of the device if it is static. For Loxone of course the device in question is the Miniserver.

Once both the port forwarding rule is created and it is assigned to the device, you can check it’s functionality with [this](http://www.yougetsignal.com/tools/open-ports/) website using your external IP address (which it detects for you) and the External Port number you have forwarded.

## RECCOMENDATION

At Loxone we suggest leaving the Miniserver on it’s default internal port of 80, this reduces complexity and the number of steps required in the setup. The external port can be randomly selected but we suggest something above 7000 as this reduces the risk of a conflict and give some additional security by obscurity.

![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
We strongly advise against using Port 80 externally as there are many malicious internet programs that simply spam this port and this can cause communication errors with the Miniserver.

## CONCLUSION

This is all the steps to port forwarding for a Miniserver. Once this is done the port forwarding is complete and please refer back to the [Remote Access](https://www.loxone.com/enen/kb/remote-access/) documentation to complete the process.