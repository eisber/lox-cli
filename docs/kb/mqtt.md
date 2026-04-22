# Mqtt

Source: https://www.loxone.com/enen/kb/mqtt/

---

The MQTT plugin enables communication with an MQTT broker (server) within a network. The Miniserver always acts as a "client" and supports receiving ("subscribe") and sending ("publish") data to/from the broker.

MQTT (Message Queuing Telemetry Transport) is a simple and lightweight protocol that can be used for communication between devices. It is widely used in the "Internet of Things (IoT)" domain, e.g., for collecting data from sensors or controlling smart devices.

An external broker is therefore required, into which the Miniserver can be integrated. The Miniserver can manage up to 16 subscriptions as well as 16 publish inputs and outputs.

> **ℹ️ Note:** Please note that the Miniserver Gen. 1 is not supported.

## Table of Contents
- [Creating and Configuring an Object](#createobject)
- [Sending/Receiving Data](#sendreceive)
- [Inputs, Outputs, Properties](#Sensor)

---

## Creating and Configuring an Object

The MQTT plugin can be integrated via the network peripherals:

![mqtt addNetDev](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mqtt_addNetDev.png)

A plugin instance can only connect to one broker instance at a time and therefore has the following settings:

![mqtt settings](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mqtt_settings.png)

---

## Sending/Receiving Data

To receive data from the broker (Subscription) or send data to the broker (Publish), IO objects must be created:

![mqtt PublishSubscription](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mqtt_PublishSubscription.png)

Sending/receiving always operates on so-called topics, which represent a path/address to data identity. Subscriptions can listen to multiple topics simultaneously ([Wildcards](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901241))), while Publish does not support wildcards.

**Example of a Topic:**
If data needs to be sent to a temperature sensor in the living room, the corresponding topic could look like this:
haus/wohnzimmer/temperatur

![mqtt topic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mqtt_topic.png)

### Subscription:

Subscribe (Receiving):
Receives value changes on the specified topic from the broker.
Another device or application can subscribe to this topic to receive the transmitted messages. For example, a thermostat could subscribe to the topic home/livingroom/temperature to obtain the temperature data.

If JSON data or similar is received, it is recommended to use the "Command Recognition Block" to extract the analog value from the data.

> **ℹ️ Note:** Value changes of a subscription are evaluated at most every 2 seconds. If the last value change was received less than 2 seconds ago, additional changes will be cached. After the 2-second interval, the most recently received change will be processed. This measure is designed to relieve the Miniserver, especially when multiple changes per second are sent to a topic.

**Wildcards in Subscriptions:**
To avoid having to subscribe to each topic individually, MQTT offers so-called wildcards, which allow multiple topics to be subscribed to simultaneously:

**Single-Level Wildcard (+):** Replaces exactly one level in the topic path.

Example:
home/+/temperature

This subscription receives messages from all rooms in the house that send temperature data, such as:
home/livingroom/temperature
home/kitchen/temperature

**Multi-Level Wildcard (#):** Covers all following levels in the topic path and must appear at the end.

Example:
home/#

This subscription receives all messages starting with home/, regardless of the number of subsequent levels, such as:
home/livingroom/temperature
home/kitchen/humidity
home/garage/car/battery

> **ℹ️ Note:** Wildcards can only be used when subscribing (Subscribe), not when publishing (Publish). This means that a device can only send messages to a specific topic, not to multiple topics simultaneously.

### Publish:

**Publish (Sending):**
A device (client) sends a message to a specific topic. In the example above, the temperature sensor would send the current temperature to home/livingroom/temperature.

---

## Sensors

| Summary | Unit |
| --- | --- |
| Subscription | Text |

---

## Actuators

| Summary | Unit |
| --- | --- |
| Publish | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Onlinestatus | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Broker address | Network address of the MQTT broker | - | - |
| Broker port | Network port of the MQTT broker (Default: 1883) | 0...65535 | - |
| Protocol version | The MQTT protocol version supported by the MQTT broker (default: MQTTv5) | - | - |
| Client ID | The ID of this MQTT client. The ID must be unique when connecting to the MQTT Broker! When using the MQTTv3 protocol the ID is truncated to max. 23 characters. | - | - |
| Username | Username used for authentication on the MQTT broker. Leave empty to perform anonymous login (if allowed by the MQTT broker). | - | - |
| Password | Password used for authentication on the MQTT broker. Leave empty to perform anonymous login (if allowed by the MQTT broker). | - | - |
| Use SSL/TLS | Use SSL/TLS connection to the MQTT broker. | - | - |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - | - |

---