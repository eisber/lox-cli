# Users & Groups

Source: https://www.loxone.com/enen/kb/users-groups/

---

## USER MANAGEMENT

#### HOW TO ON USERS AND GROUPS

This video will talk you through setting up users and user groups for different levels of access with the Loxone App.

## USERS & USER GROUPS

With users and user groups it is very easy to control access rights to the user interface. This has many practical uses including multiple office access control, giving only adults in a property access to the heating settings, or perhaps controlling hotel room lighting.

#### INSERT A USER AND A USER GROUP

To create a new user, click on the User branch in the Periphery tree then you will see the button to add in a new user in the blue bar at the top of the configuration screen. Alternatively, use the F4 shortcut, and type ‘user’.

You can change the User name in the properties of each user.

![EN KB Config Users](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Users.png)

To create a new user group, click on the User groups branch in the Periphery tree then you will see the button to add in a new user group in the blue bar at the top of the configuration screen. Alternatively, use the F4 shortcut, and type ‘user group’.

You can change the User group name in the properties of each group.

![EN KB Config User Groups 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Groups-1.png)

#### SETTING A PASSWORD

In the ‘Properties’ window for the user, you can set a password

![EN KB Config User Passwords 1024x577](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Passwords.png)

#### ADDING A USER TO A GROUP

A user can be added to a user group simply by clicking and dragging the user into the correct user group.

![EN KB Config User Group Children 1024x576](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Group_Children.png)
.

#### USER GROUP RIGHTS

Each user group can now be assigned certain access rights, the users who belong to the group will then be afforded access rights. The level of access is only required in one group the user is a member of to allows them this.

> **ℹ️ Note:** Allows users to access the Miniserver back-end over FTP.

NOTE: Previously Expert Mode is as of V9.0 only available to users in the “Administrators” group. “Interface Settings” allows an interface level option to make changes but function block parameters are not available here.

#### ADJUST ACCESS TIMES

If you have set up an access control system with iButtons, you can adjust the times at which the user group should have access. You can find more information on the [access controller ](https://www.loxone.com/enen/kb/access-controller/)in our documentation.

![EN KB Config User Groups Times 1 1024x657](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Groups_Times-1.png)

If you have created a user group for your children, don’t forget to add yourself, otherwise only your children will have access to the object.

In the user group ‘All’, every user is automatically added. For every object, you can set a group which has sole access to that object.

![EN KB Config User Groups Function Blocks](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Groups_Function_Blocks.png)

#### ASSIGN A USER GROUP TO A PAGE

To help speed up configuration, you can also assign a user group to a whole page. This means any objects inserted onto that page will automatically be assigned to that user group.

![EN KB Config User Groups Page 300x150](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Groups_Page.png)

#### EXAMPLE PERIPHERY TREE OF USERS AND USER GROUPS

In the example below we can see a Loxone system with family of 4 an administrator and cleaner, all with relevant User groups.

![EN KB Config User Groups Example 231x300](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_User_Groups_Example.png)