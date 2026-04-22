LOXONE
Date:
2025-06-03
Authors:
Loxone
Network-Communication: Required Ports & Domains
Miniserver
· DNS
o UDP Port 53 :unselected:
· mDNS
O UDP 224.0.0.251:5353 :unselected:
· Search/Discovery
o mDNS
o Upnp :unselected:
O UDP Port 7070-7071 :unselected:
· Communication with App & Config
O TCP Port 443 :unselected:
O TCP Port [HTTP Port set in Miniserver settings] :unselected:
· FTP
O TCP Port 21 (Control) :unselected:
o TCP Port 20 (Data) :unselected:
. Cloud - Outgoing connections
o Weather Service :unselected:
TCP/HTTP weather.loxone.com:6066 :selected:
TCP/HTTPS weahter.loxonecloud.com:443 :selected:
o Caller Service :unselected:
TCP caller.loxone.com:80/443 :selected:
o Cloud-Mailer :unselected:
TCP mail.loxonecloud.com:443 :selected:
o Cloud-DNS :unselected:
UDP dns.loxonecloud.com:7700 :selected:
o Push :unselected:
TCP push.loxonecloud.com:443 :selected:
O Remote Connect :unselected:
MQTT Connection :selected:
· TCP *. ccbroker.loxonecloud.com:8443
SSH Tunneling Connection :selected:
· SSH *. loxonecloud.com:22
. Example: eu.ccd2.loxonecloud.com:22
Connections to Miniservers via Remote Connect :selected:
. TCP *. dyndns.loxonecloud.com:[Port]
· Attention: Port is assigned randomly! (20000-65000) :unselected: :selected: :selected: :selected: :selected:
LOXONE
Date:
2025-06-03
Authors:
Loxone
. Example: *. dyndns.loxonecloud.com:30357
Data Center Request
. TCP api.loxonecloud.com:443
. TCP secureapi.loxonecloud.com:443
HTTPS Certificate Request :selected:
. ca.loxonecloud.com:443
O Provisioning & Cloud Service Status :unselected:
TCP api.loxonecloud.com:443 :selected:
TCP secureapi.loxonecloud.com:443 :selected:
· Debug-Monitor
O UDP Port 7777 (Default) :unselected:
o Port can be set in Config :unselected:
· Auto-Update
o TCP update.loxone.com:80/443 :unselected:
o TCP updatefiles.loxone.com:80/443 :unselected:
· Crash-Log Server
O UDP log.loxone.com:7707 :unselected:
· BacNET
O UDP+TCP Port 47808 :unselected:
¢ Client/Gateway
O UDP 7070 - 7077 :unselected:
O Broadcasts must be allowed :unselected:
¢ KNX-IP (only Miniserver Gen.1)
O IGMP Multicasts :unselected:
¢ Blink synchronization of Network Devices
O UDP Broadcast to 255.255.255.255:7079 :unselected:
· Online Check
O The Miniserver regularly checks if he has internet by pinging addresses :unselected:
O ICMP :unselected:
O The addresses are checked sequentially: if the first one succeeds, the Miniserver assumes it has :unselected: internet, and no further addresses are checked. If the first one fails, the second is then checked.
o Addresses: :unselected:
dns.loxonecloud.com :selected:
icann.org :selected:
w3c.org :selected:
Loxone Config
· Auto-Update
o TCP update.loxone.com:80/443 :unselected:
o TCP updatefiles.loxone.com:80/443 :unselected:
· Project Planning
O Price List updates :unselected:
O TCP shop.loxone.com:443 :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE
Date:
2025-06-03
Authors:
Loxone
. Local Connection to Miniserver
O TCP Port 443 :unselected:
O TCP Port [Defined HTTP Port] :unselected:
· Remote Connection to Miniserver
O TCP dns.loxonecloud.com:80/443 :unselected:
o TCP *. dyndns.loxonecloud.com:[Port] :unselected:
Attention: Port is assigned randomly! (20000-65000)
· Geo-Coordinates
o TCP geo.loxone.com:443 :unselected:
· Loxone-Library
o Update of Templates and Template-Index :unselected:
o TCP api.library.loxone.com:443 :unselected:
· Data Center Request
O Determination which loxonecloud.com datacenter the config should use for connecting to the :unselected:
Miniserver
TCP api.loxonecloud.com:443 :selected:
TCP secureapi.loxonecloud.com:443
· Documentation
o TCP loxone.com:80/443 :unselected:
App
· Local Connection to Miniserver
O TCP Port 443 :unselected:
O TCP Port [Defined HTTP Port] :unselected:
. Detecting Remote Connection to Miniserver
o TCP dns.loxonecloud.com:80/443 :unselected:
. Remote connection via Remote Connect
o TCP *. dyndns.loxonecloud.com:[Port] :unselected:
o Attention: Port is assigned randomly! (20000-65000) :unselected:
O Firewalls may not allow domain based rules and the IPs of the servers behind the domain are not fixed per MS, they are assigned to the MS on the connection attempt: :unselected:
195.201.222.243 / 2a01:4f8:1c1c:57c8 :: 1 :selected:
168.119.185.175 / 2a01:4f8:c010:2f7 :: 1 :selected:
please beware: they will change at some point in 2023 to the following :selected:
· 5.75.128.138 / 2a01:4f8:1c1c:cbb4 :: 1
· 88.99.85.148 / 2a01:4f8:c012:494f :: 1
· Remote connection via CloudDNS
O TCP [IP]: [Port] :unselected:
o Attention: Both the IP and the Port are dynamically provided by dns.loxonecloud.com, as :unselected: configured on the Miniserver.
· Miniserver Search
o UDP Port 7070-7071 :unselected:
o Upnp :unselected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE
Date:
2025-06-03
Authors:
Loxone
· Auto-Update
o TCP update.loxone.com:80/443 :unselected:
o TCP updatefiles.loxone.com:80/443 :unselected:
· News/Infos/Device-Documentation
o TCP loxone.com:80/443 :unselected:
· Air/Tree Pairing- and Battery-Device-Information
o TCP extended-app-content.s3.eu-central-1.amazonaws.com:443 :unselected:
· Push Service
o TCP push.loxonecloud.com:443 :unselected:
· SIP (Intercom Audio)
o UDP Ports :unselected:
5060 (default)
Audioserver
· Search/Discovery
o mDNS :unselected:
o Upnp :unselected:
O UDP Ports 7070-7071 :unselected:
· Communication with Miniserver
O TCP Port 7095 :unselected:
o TCP Port 80 :unselected:
O TCP Port 443 :unselected:
· Communication with App
O TCP Port 7091 :unselected:
· Communication AudioStreams Audioserver <- > Audioserver
o UDP Port 7788 :unselected:
O UDP Ports 14000 - 14999 :unselected:
· General Communication Audioserver <- > Audio Extensions
O Boot & NFS TCP + UDP 111, :unselected:
O Boot II TCP + UDP 2049 :unselected:
· Exception for internet radio streams:
o All listening and request ports from / to radio stream stations are assigned dynamically, Radio station provider assigns the http / TCP streaming ports :unselected:
· Apple AirPlay
O TCP Ports 7000-7004 AirPlay streaming data :unselected:
O TCP/UDP Ports 49152-65535 Random high ports for dynamic sessions during streaming. :unselected:
Loxone Intercom
· Search/Discovery
o mDNS :unselected: :selected:
LOXONE
Date:
2025-06-03
Authors:
Loxone
o Upnp :unselected:
O UDP Port 7070-7071 :unselected:
· Communication with Miniserver
O TCP Port 7091 :unselected:
· Communication with App
o TCP Port 7091 :unselected:
o Video/Audio: UDP Ports are assigned dynamically :unselected:
External Video/Audio: UDP stun.loxonecloud.com:3478 :unselected:
The Loxone STUN server is used to reflect the public IP to the Intercom which then can be :selected: used as connection candidate for video/audio connection.
· Auto-Update
o TCP update.loxone.com:80/443 :unselected:
o TCP updatefiles.loxone.com:80/443 :unselected:
Intercom Gen. 1
· Search/Discovery
o UDP Port 5000 :unselected:
O UDP Port 8110 (Video-Module) :unselected:
O UDP Port 8112 (Audio-Module) :unselected:
· Communication with Miniserver
o UDP Port 8110 (Video-Module) :unselected:
O UDP Port 8111 (Video-Module) :unselected:
O UDP Port 8112 (Audio-Module) :unselected:
O UDP Port 8113 (Audio-Module) :unselected:
· Communication with App
o TCP Port 80 :unselected:

---
## Extracted Tables

### Table (2x2)
Date: | 2025-06-03
--- | ---
Authors: | Loxone

### Table (2x2)
Date: | 2025-06-03
--- | ---
Authors: | Loxone

### Table (2x2)
Date: | 2025-06-03
--- | ---
Authors: | Loxone

### Table (2x2)
Date: | 2025-06-03
--- | ---
Authors: | Loxone

### Table (2x2)
Date: | 2025-06-03
--- | ---
Authors: | Loxone
