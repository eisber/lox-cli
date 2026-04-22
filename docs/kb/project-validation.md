# Project Validation

Source: https://www.loxone.com/enen/kb/project-validation/

---

The Project Validation tool helps you optimise your configuration for best performance and simplicity.  Using Project Validation and following the recommendations will result in a more stable and user-friendly config.
Project Validation will automatically be run on opening a new config file either by loading from the Miniserver or by opening a project from your computer. Validation can also be run manually by going to the “My Project” tab and clicking on the “Project Validation” button.

![Project Validation](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Project-Validation.png)

To get a full analysis, please make sure you are connected to your Miniserver. This will allow current events on the Miniserver to be checked live (for example current stats being logged or the network stability) which can not be detected by the Config File alone without this connection.

Project Validation will categorise the results and provide 3 different levels of feedback.

– **Information**, this is not an error or a mistake however it is useful information that should be paid attention to. For example, unused rooms or categories will be listed here. This would not cause a problem but could be used to make the config file more concise and a little neater.

– **Warning**, this is a minor issue, it may affect the way in which something appears in the user interface or prevent something from working as expected but will not cause any critical functionality issues.

– **Error, **this is potentially a serious error. This could cause functions to not work. An example of this would be a switch that has been deleted leaving inputs unassigned to anything thus meaning they will not be able to be triggered.

#### EXAMPLE

![Windows 8 1 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Windows_8_1_1.png)

In this example, there are two information items, the number of objects in the configuration and the number of unused categories. This is simply useful information to the user.

There are two main warning items. Missing passwords for the two cleaner users and an unused input reference on the Cinema page. These in their own right will not cause any problems but the cleaners would not be able to login to any user interface and if the input reference was intended to be connected somewhere then this may hamper functionality expected if this was a mistake.

By clicking on “Show” it will automatically direct me to the page and highlight. Thus:

![Windows 8 1 1 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2017/10/Windows_8_1_1-1.png)