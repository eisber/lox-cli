# Firewall Settings for Miniserver Search

Source: https://www.loxone.com/enen/kb/firewall-miniserver-search/

---

If the firewall blocks Loxone Config, it can lead to issues with the Miniserver search.
In order for Loxone Config to find Miniservers on the network, the Windows firewall must allow incoming UDP packets on port 7071. During installation, Loxone Config automatically creates the corresponding firewall rule. If this rule is deleted or creation fails, the Miniserver search may be affected.

To resolve the issue, Loxone Config can be reinstalled, or the rule can be manually added to the firewall.

## Add a Rule to the Firewall

Open “Windows Defender Firewall” through Windows Search and click on “Advanced settings”:

![Firewall AdvancedSettings](https://www.loxone.com/dede/wp-content/uploads/sites/2/2025/03/Firewall_AdvancedSettings.png)

In the following window, select “Inbound Rules” and right-click to choose “New Rule…”:

![Firewall InboundRules](https://www.loxone.com/dede/wp-content/uploads/sites/2/2025/03/Firewall_InboundRules.png)

The wizard will then open; select the following:

![firewall wizard](https://www.loxone.com/dede/wp-content/uploads/sites/2/2025/03/firewall_wizard.gif)
- Rule Type: Port
- Protocol and Ports: UDP and Specific local ports: 7071
- Action: Allow the connection
- Profile: Ccheck Domain, Private, and Public
- Name: Set a name (e.g., Loxone Config Miniserver Search)