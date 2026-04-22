# Emergency Light Testing

Source: https://www.loxone.com/enen/kb/emergency-light-testing/

---

## Brief: I want to automate emergency light testing in our building.

Emergency lighting systems are used in many commercial premises, offices, public buildings and industrial plants. They must ensure that in the event of a power failure the minimum necessary lighting is provided immediately and automatically. This applies, for example, to escape route signs, fire safety equipment, escape routes themselves and much more.

As an emergency can be a matter of life and death, it is imperative that emergency lighting systems are kept in an effective and reliable state. The most important requirements for emergency lighting are therefore regulated by law.

The most important requirements with regard to emergency light testing, maintenance and the necessary documentation are defined in the BS EN 50172 / BS 5266-8.

The standard outlines the need for monthly testing in specific conditions. That the owner or holder must designate a person responsible for testing and maintaining the emergency lighting and must keep a record of the performance.

External service providers are often called upon for this purpose. It is not uncommon, however, for a separate employee to be entrusted with this task. However, this can result in the maintenance of emergency lighting being missed due to other work obligations or absences, such as illness and holidays.

## Solution: Using Loxone for emergency light testing.

Our solution allows you to implement automatic emergency light testing. With Loxone, the necessary tests can be planned and carried out completely automatically. The results are also conveniently recorded in a log file.

A suitable [coupling relay](https://shop.loxone.com/enuk/coupling-relay.html) connected to the Miniserver initially ensures the permanent power supply of the emergency lighting as well as the room lighting during normal operation. This also ensures that the batteries installed in the emergency lighting are fully charged.

To start the test, a relay on the [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html) switches the coupling relay to interrupt the power supply to the room and emergency lighting. The emergency lighting now works in battery mode.

A [Loxone Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html) simultaneously measures the prevailing lux levels and records them in a log file. If at any time the lux values fall below a defined value (depending on the lux values at the beginning of the test), the test was not successful. In this case, a notification is automatically sent to the responsible person.

For the test, a device with an integrated switch-off delay is required. In our example, we work with the “staircase light switch” device. This allows the start or duration of the test to be recorded in a statistic and sent by e-mail.

Since the emergency lighting tests are carried out and recorded automatically, the presence of the person responsible for maintenance is not absolutely necessary. The tests can thus be carried out outside normal working hours to prevent any disturbance to employees and visitors.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)
- [Coupling Relay](https://shop.loxone.com/enuk/coupling-relay.html)

### Configuration:

[
![Use Case 1 Emergency Lighting Test](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-1-Emergency-Lighting-Test.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/05/Use-Case-1-Emergency-Lighting-Test.jpg)

### Download the sample file:

### Emergency Lighting Test

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/09/Use-Case-1-Emergency-Lighting-Test.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/09/Use-Case-1-Emergency-Lighting-Test.loxone)

### Related Video Tutorial

> **ℹ️ Note:** Config Challenge: Automated Emergency Light Testing The facilities manager of a large office building needs a way of automatically testing the emergency lights in the building View Video >>

## Why you and your customer should consider automating emergency light testing?

Emergency lighting serves our safety and provides important points of reference in critical situations such as fires.

As people’s lives are often at stake in such situations, it is particularly important that the emergency lighting wors perfectly. Even the failure of just one safety light or escape sign could pose a potential risk – and in the event of damage, the operator bears the burden of proof.

The implementation of the automatic emergency light testing ensures that the intervals required by law for the maintenance of the lighting – or its testing – are adhered to.

At the same time, the required obligation to keep a log is also fulfilled, as the results and parameters (time, duration, etc.) of the test are documented in a log file.

In the case of abnormalities, the responsible person is informed about any problems. This allows appropriate measures to be taken promptly and potential liability risks to be minimized.

In order not to impair employees and visitors, the test is carried out fully automatically outside of opening hours.

The reliable, automated maintenance of emergency lighting makes it unnecessary to outsource the test to third parties.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)