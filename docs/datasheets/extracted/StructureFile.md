LOXONE
Create Automation
STRUCTURE FILE
Date 2025.06.03
LOXONE
Miniserver
LOXONE Create Automation
The aim of this document is to give you a fundamental understanding on how the static structure of a Miniservers Configuration is represented. At Loxone we call this representation "Structure-File" and it is available as "LoxAPP3.json".
In order to create a UI for remote controlling a Smart-Home different infos are required. There is the static structure which changes due to modifications in the configuration itself. On the other side there are the states that change over time due to the permanently changing environment (temperature, movement, .. ) or actions taken.
While "Communicating with the Miniserver" did go into detail on how the dynamic states are structured and communicated, this document is going to focus on the fixed structure.
Table of contents
Table of contents
2
General Info
8
lastModified
8
msInfo
8
globalStates
10
rooms
11
cats
12
weatherServer
12
times
13
caller
13
autopilot.
13
mediaServer
13
Loxone Audioserver
14
Miniserver Compact
14
Loxone Music Server
14
messageCenter
14
Controls
15
16.0
Structure File
Page 2 of 174
Create Automation
LOXONE
Mandatory fields
15
Optional fields
15
Locking and Unlocking Controls
16
Which controls can be (un)locked via API?
17
How to know a control is locked and why ?.
17
How to (un)lock a control via API?
17
Statistic
17
Commands
19
BinaryFormat
19
StatisticV2
20
Structure
20
Commands
22
Binary Result
25
Secured Details
26
Commands
26
Control History
26
Commands
26
Data Structure
26
Trigger Types
26
Control Types
27
AalEmergency
27
AalSmartAlarm
28
Alarm
29
AlarmChain
31
AlarmClock
32
Application
36
AudioZone
36
AudioZoneV2
41
16.0
Structure File
Page 3 of 174
LOXONE Create Automation
CarCharger
44
BMW Wallbox specific
46
Central Objects
46
ClimateController.
48
ClimateControllerUS
52
ColorPickerV2
58
ColorPicker
60
Daytimer
61
Intelligent Room Controller Daytimer v2
63
Intelligent Room Controller Daytimer
64
Pool Daytimer
64
Dimmer
64
EnergyManager
65
EnergyManager2
66
EnergyFlowMonitor
69
Fronius
72
Gate
74
Heatmixer
75
Hourcounter
76
InfoOnlyAnalog
77
InfoOnlyDigital
77
InfoOnlyText
78
Intelligent Room Controller v2
79
Intelligent Room Controller
85
Intercom
89
IntercomV2
91
Irrigation
93
Jalousie
95
16.0
Structure File
Page 4 of 174
Create Automation
LOXONE
NFC Code Touch
98
LightController
103
LightControllerV2
104
LightsceneRGB
109
LoadManager
110
MailBox
111
Meter
112
MsShortcut
114
PoolController
115
Pushbutton
122
Radio
123
PresenceDetector
124
PulseAt
125
Remote
125
Sauna
128
Sequential
131
Slider
131
SmokeAlarm
132
SolarPumpController
134
SpotPriceOptimizer
135
StatusMonitor
137
SteakThermo
138
Switch
143
SystemScheme
144
TextState
145
TextInput
145
TimedSwitch
145
Tracker
146
16.0
Structure File
Page 5 of 174
Create Automation
LOXONE
UpDownLeftRight digital
147
UpDownLeftRight analog
148
ValueSelector
148
Ventilation
149
Webpage
155
Window
155
WindowMonitor
156
PowerUnit
158
Wallbox2
159
WallboxManager
162
ACControl
165
Revision History
171
15.3 [ WIP ].
171
15.1
171
15.0
171
14.7
171
14.5
171
14.4
171
14.2
171
14.1
172
14.0
172
13.2
172
13.1
172
13.0
172
12.2
173
12.1
173
12.0
173
16.0
Structure File
Page 6 of 174
LOXONE
Create Automation
16.0
Structure File
Page 7 of 174
LOXONE Create Automation
General Info
A Smart Home is a set of various sensors and actuators that are linked together by our Miniserver. Rooms and Categories are used to group these sensors and actuators (which we'll be calling controls from now on). And besides those controls and the rooms and categories which they belong to, there is some other global and external information, like the weather- server or info about the Miniserver itself. These different types of information lay out the basic structure of this file and in the next chapters, we're going to go into detail on each one of these.
lastModified
Since the Structure-File can grow rather large, it would not be a good idea to download a completely new version each time the UI is built up. So once you download the Structure-File, make sure that you cache it and save the "lastModified" attribute value. Every time you reconnect you can use the command "jdev/sps/LoxAPPversion3" to compare whether or not the Structure you've cached is still up to date.
msInfo
The msInfo area contains static information on the Miniserver and it's configuration. While some of these are pretty self explanatory, the need for others might be unclear at first.
· serialNr
o serial number of this Miniserver :unselected:
· msName
o Name of the Miniserver as specified in the configuration document
· projectName
o Name of the configuration document used for this Miniserver.
. localUrl
o IP & Port that are used to connect to this Miniserver inside its local network. :unselected:
· remoteUrl o Url/IP & Port using which the Miniserver is globally reachable.
· hostname
o Miniserver hostname including domain
o Only when Miniserver is set to dhcp
o Since V13.3
· tempUnit
16.0
Structure File
Page 8 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected:
LOXONE Create Automation
o Gives info what temperature unit is used by this Miniserver. :unselected:
. 0 = ℃ # 1 = ºF
· currency
o a string containing the currency symbol to use (€, $, ... ) :unselected:
· squareMeasureligh
o the unit of area (for rooms) :unselected:
· location
o The address of this Miniserver, where it is located. This info is also used for calculating the sunrise & sunset as well as for the cloud-weather. :unselected:
· latitude, longitude and altitude
o Since 12.16.14, resolved coordinates and altitude based on the location :unselected: provided.
· heatPeriodStart
o DEPRECATED :unselected:
Replaced by the command jdev/sps/calendargetheatperiod :selected:
o Month and day when the heating period starts each year, used by the intelligent room-controller. :unselected:
· heatPeriodEnd
o DEPRECATED :unselected:
Replaced by the command jdev/sps/calendargetheatperiod :selected:
o End of the heating period. :unselected:
· coolPeriodStart
o DEPRECATED :unselected:
Replaced by the command jdev/sps/calendargetcoolperiod o Start of the cooling period. :unselected:
· coolPeriodEnd
o DEPRECATED :unselected:
Replaced by the command jdev/sps/calendargetcoolperiod o End of the cooling period. :unselected:
· catTitle
o Top level name for all the categories, maybe there is a different kind of :unselected: grouping.
· roomTitle
Some configurations handle the location grouping differently, they might not :unselected: want to call it "Room" but "Zone" or alike.
· miniserverType
16.0
Structure File
Page 9 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o 0 - Miniserver (Gen 1)
o 1 - Miniserver Go (Gen 1)
o 2 - Miniserver (Gen 2)
o 3 - Miniserver Go (Gen 2)
o 4 - Miniserver Compact
· sortByRating
o Indicates whether or not the controls are to be sorted based on their rating, which was specified in the config using the stars (0-5).
· currentUser
o name
o uuid - used e.g. to identify the user in user-management
o isAdmin
o changePassword
if the user is allowed to change the password via WebService
o userRights
List of permissions available for this user.
Some of these may already be present in the JWT returned during auth
Others set here can be requested explicitly using getJWT
See "Communicating with the Miniserver" on more details (Permissions & Tokens section)
globalStates
This section lists all states that affect not only a single control but the whole Miniserver. The UUIDs here can be used to lookup the corresponding state values that arrive with all the other controls state updates.
· sunrise
o seconds since midnight, time when the sun will rise at the Miniservers location.
· sunset
o minutes since midnight, time when the sun will go down.
· favColorSequences
o An array of color sequences that have been marked as favorites in any light controller v2.
· favColors
o An array of colors that have been marked as favorites in any light controller v2
· notifications
16.0
Structure File
Page 10 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected:
LOXONE
Create Automation
o Push Notifications are also sent to open WebSocket connections (if the user is allowed & registered for these notifications)
o {
"uid": String,
// unique message id
"ts": Number,
// unix timestamp
"type": 10,
// type, 10 = normal message
"title": String,
// title
"message": String,
// message
"data": {
// additional data
“Lvl": Number,
// level, 1 = Info, 2 = Error, 3 = SystemError
["uuid": String]
// optional uuid (from Control, eg. Alarm)
} }
· miniserverTime
o the current date, time & UTC-offset of the Miniserver. (e.g. "2017-07-03 13:01:36 +02:00:00")
o Updates only when the time is changed (e.g. shift to/from daylight saving time or manual time change).
· liveSearch
o Json with current Information about device learning via app
· modifications
o structural changes made via API (not via Loxone Config) are published as text events using this UUID.
operatingModes
Operating Modes are used by the Miniserver to respond to time-based events, like a weekday, weekend, vacation or alike. Mostly this is used in daytimers and the intelligent roomcontrollers.
rooms
In this section, there's a list of all the rooms that are used to group the controls in the configuration.
· uuid
o Unique identifier for this room on this Miniserver
· name
· image
o Icon for this Room
· defaultRating
16.0
Structure File
Page 11 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected:
LOXONE Create Automation
o Based on this number, the rooms are sorted in the UI (depending on the sortByRating-Attribute in msInfo
cats
Just like the rooms that group controls based on their location, categories group them logically.
· type
o Categories can be given a type, which provide semantic info on what the controls in this category are for.
lights
indoortemperature
· shading
· media
· color
o categories can be given a color on the UI
weatherServer
If a Cloud Weather is configured, this section is added to the Structure-File.
states
There are only two states listed here, actual and forecast. Along with the state-updates of the other controls, the weather-updates are also delivered. See the corresponding section in the "Communicating with the Miniserver" document for details on how to parse the Weather-State- Events.
· actual
o the current weather data for the moment
· forecast
o the weather data for the future (for the next 96 hours)
format
This is a list on what (C-Style) formats to use for the different states.
· relativeHumidity
· temperature
· windSpeed
· precipitation
16.0
Structure File
Page 12 of 174 :unselected: :selected: :unselected: :selected: :unselected:
LOXONE
Create Automation
· barometricPressure
weatherType Texts
Each forecast and the actual weather situation has a type that is visualized differently. This section gives the user friendly texts for each of this weather situations.
weatherFieldTypes
available since Miniserver 8. Returns the possible weather field types. This types are the same as in the Loxone Config (e. g. Temperature)
times
available since Miniserver 8. Returns a list of possible time fields. This types are the same as in the Loxone Config (e. g. Minutes until sunset).
caller
available since Miniserver 8. Returns a list of configured caller service in the Loxone Config.
autopilot
available since Miniserver 8. This section is used for the autopilot generator configuration. An autopilot is a rule which can be created on the app side and is executed on the Miniserver as soon as all events of the rule are matched. The API isn't publicly available.
mediaServer
This section is present as soon as one or more Loxone Music Servers (or Castatunes Servers) are in use on this Miniserver. They are generically referred to as "mediaServer" and each one is identified by a UUID.
· type
o 2 = Loxone Audioserver / Miniserver Compact (see subtype)
o 1 = Loxone Music Server :unselected:
o 0 = Casatunes :unselected:
· subtype (only available for type 2)
o 0 (or missing) = Loxone Audioserver
o 1 = Miniserver Compact
· host
o IP and port used for communicating with the server
· MAC
16.0
Structure File
Page 13 of 174 :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
o Mac address of server
o Available since 11.1
. localIP
o Local resolved IP address
o Only for Audioserver/Miniserver Compact
Loxone Audioserver
Din-Rail mounted Loxone Audioserver with a more powerful API. Clients will connect directly to the Audioserver, just like with the Loxone Music Server. When remote, a proxy on the Loxone Miniserver is used.
Miniserver Compact
The Miniserver Compact contains the audio service of the Loxone Audioserver, while the API remains the same, it is reachable on the same host as the Miniserver. For external connection, there is still a proxy to be used, as remote-connect will only make the HTTPS/WSS port reachable, not the port for the audio-web-socket.
Loxone Music Server
The Loxone Music Server provides a powerful API to clients that are connected directly to it, here are some examples:
· adding/removing and browsing external services like Spotify or Google Music
· browsing the music stored directly on the Music Server
· creating playlists
· browsing web-radio-stations
· modifying per zone favorites
This API is not covered in this documentation and is not publicly available as of now.
messageCenter
available since Miniserver 10.0. This section is used for the Systemstatus, the ultimative trouble guide for your Smart Home.
States
· changed
o Unixtimestamp when the the Systemstate entries have been modified the last time
16.0
Structure File
Page 14 of 174 :unselected: :unselected: :unselected: :unselected: :unselected:
LOXONE Create Automation
Controls
Controls are a term that covers both actuators and sensors, simple in- or outputs and complex block-controls like an intelligent roomcontroller.
Mandatory fields
· name
· type
o this attribute identifies what kind of control this is (Jalousie, Daytimer, ... ) :unselected:
o an empty string as type indicates a control that should not be visualized. :unselected:
· uuidAction
o unique identifier for this control :unselected:
· defaultRating
o just like rooms and categories, controls can also be rated :unselected:
· isSecured :selected:
o whether or not the visualisation password has to be entered for this control :unselected:
Optional fields
The optional fields might differ between the various types of controls as well as between different controls of the same type. More detailed information can be found in the documentation for the different types of controls.
· room
o uuid of the room this control belongs to :unselected:
· cat
o uuid of the category this control belongs to :unselected:
· states
o list of state uuids for the control :unselected:
· securedDetails
o indicates that there is sensitive information available. for details see Secured :unselected: Details
· details
o visualisation details, like the format or jLockable :unselected:
· statistic
o if a control is scheduled for recording its values to a statistic, this attributes :unselected:
contains all info necessary.
· restrictions
16.0
Structure File
Page 15 of 174 :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o available since 11.0 :unselected:
o bitmap providing info on restrictions for this block. :unselected:
0 = referenced only (internal)
. if this bit is set, it means that such a control must only be shown e.g. inside the system-scheme block, but not in a rooms list of controls.
· 1 = read only (internal)
. Not allowed to send any commands only visualizes values
. 2 .. 3 = reserved
4 = referenced only (external)
· just like bit #0, but for external connections.
5 = read only (external)
· same as bit #1
· hasControlNotes
o available since 11.0 :unselected:
o This flag indicates that there are notes/helptexts assigned to this control. :unselected:
o The webservice jdev/sps/io/{controlUUID}/controlnotes returns these notes in :unselected:
plaintext.
o the string has a maximum length of 500 characters :unselected:
· preset
o available since 11.3 :unselected:
o Only added if the control uses a preset
o uuid :unselected:
uuid of preset
to be used in commands like '/dev/sps/resettodefaultall/[preset uuid]'
o name :unselected:
Name of the preset that can be used for display
· links
o available since 11.3 :unselected:
o uuid array of linked controls
o the order of the uuids is the order defined in Loxone config :unselected:
Locking and Unlocking Controls
Available since version 11.3.2.11
16.0
Structure File
Page 16 of 174 :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
All controls can potentially be (un)locked via API calls and provide their current lock state via a status. Controls that are locked can't be controlled via API or logic inputs, just like when the Reset-Input is active.
Which controls can be (un)locked via API?
Whether or not a control supports locking via API calls is defined in a controls details section, via the boolean attribute "jLockable".
How to know a control is locked and why?
Whether or not a control is locked is provided by the "jLocked" status via Status Updates. This is a text-status that, if locked, provides a JSON containing the following attributes:
· locked :selected:
o 0 -> not locked :unselected:
o 1 -> locked by visu :unselected:
o 2 -> locked by logic :unselected:
· reason :selected:
o Text describing the reason of the lock. :unselected:
Please note that if the status is an empty string it has to be assumed that the control is not locked. It may be used so that less data is transmitted.
How to (un)lock a control via API?
· Locking via Json Get request :selected:
o Only available for admins! :unselected:
o /jdev/sps/io/{controlUuid}/lockcontrol/{0,1,true,false}/{uriEncodedReason} :unselected:
Reason is optional :selected: :selected: · Unlocking via Get Request
o Only available for admins! :unselected:
o Can be used alternatively to post request with 'lock' attribute set to false :unselected:
o /jdev/sps/io/{controlUuid}/unlockcontrol :unselected:
Statistic
Each control that has statistics enabled, will provide the following infos in its statistic attribute:
· frequency
16.0
Structure File
Page 17 of 174 :selected:
LOXONE Create Automation
o how often is the statistic written
o interval (8-11)
new as of Version 11.0
Will periodically write down the current value
When a value remains unchanged, no value will be written.
o average (2-6)
deprecated as of Version 12.0
o Possible values
. 0 = none
= 1 = every change
= 2 = average per minute
3 = average per 5 minutes
4 = average per 10 minutes
5 = average per 30 minutes
6 = average per hour
7 = < Reserved>
8 = Interval, 5 Minutes
9 = Interval, 10 Minutes
10 = Interval, 30 Minutes
11 = Interval, 60 Minutes
12 = Interval, 15 Minutes
· outputs
o an array of outputs for whom statistic data is recorded
o each output has the following attributes
" id = Index, what datapoint-row is used for this output (0-6)
name
format
· format specifier for analog values
uuid
visuType
· 0 = line chart
· 1 = digital
· 2 = bar chart
Example
The statistic infos for a line chart showing the hourly current CO2 sensor values in PPM.
16.0
Structure File
Page 18 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
{ "frequency": 10, "outputs": [ { "id": 0, "name": "CO2 Sensor Tree", "format": "%. 0fppm", "uuid": "12917710-0034-43af-ffff11707f41df48", "visuType": 0 }
]
}
Commands
· jdev/sps/getstatsdate :selected:
o date of statistics.json file :unselected:
o Will get set after requesting the file at least once :unselected:
· statistics.json :selected:
o contains info of all stored statistics :unselected:
o Also available via HTTP-Request: {ip}/stats/statistics.json :unselected:
o Request valid via Websocket :unselected:
o Via HTTP: stats/statistics.json :unselected:
· statistics.json/{controlUUID} :selected:
o filtered statistic file :unselected:
o available since 6.1.10.16 :unselected:
o Also available via HTTP-Request: {ip}/stats/statistics.json/{controlUUID} :unselected:
· statisticdata.xml/{controlUUID}/{date} :selected:
o XML file containing the statistics :unselected:
o {date} = "YYYYMM" or "YYYYMMDD" :unselected:
o Also available via HTTP-Request: :unselected: {ip}/stats/statisticdata.xml/{controlUUID}/{date}
· binstatisticdata/{controlUUID}/{date} :selected:
o returns a binary "stream", format below o {date} = "YYYYMM" or "YYYYMMDD" :unselected: :unselected:
BinaryFormat
Via websocket binstatisticdata can be downloaded. Here's how it is structured.
16.0
Structure File
Page 19 of 174
LOXONE Create Automation
· ts: Uint32 (4 Bytes)
o seconds since 1.1.2009 in local Miniserver-time
· values Float64 (8 Bytes)
o amount of values can be found in the LoxAPP3.json for each Control (property "statistic", length of "outputs" array)
o if multiple outputs are available (eg. meter) the order is the same as in outputs array of the control statistic details
StatisticV2
With the introduction of the energy flow monitor, a new way of handling statistics has been introduced. Meters may also support this new statistic handling.
If a control supports the new handling, the "statisticV2" property must be set in the control, containing the structure below.
Structure
· groups
o values with equal frequency are grouped together, e.g. the actual storage power is in one group and the total stored/used energy is in a second group.
o id
the id of a group, used for the getStatistics request
o mode
the recording mode of the statistics
. 0 = none
1 = every change (max 1 per minute)
7 = every change
8 = Current value recorded every 5 minutes
9 = Current value recorded every 10 minutes
10 = Current value recorded every 30 minutes
11 = Current value recorded every 60 minutes
12 = Current value recorded every 15 Minutes
o accumulated
if present & true, this indicates that this is an accumulated meter value, such as energy produced by a solar panel. This means the values are incremented over time and to display a sensible graph, the diff between those values is to be shown.
o dataPoints
16.0
Structure File
Page 20 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
array of dataPoints that are part of this statistic group - see section dataPoints for more information.
the order of the datapoints in this array represents the order they will be returned when requested via getStatistics without filtering by outputs.
dataPoint
· title - user friendly name to show in a graph
· format -> the format to use when showing the value of the datapoint
. output -> the name of the output/state used for recording the values.
Example
The statistics entry of a grid meter keeping track of the consumption/production exported to or consumed.
{ "groups": [ { "id":"1", "mode": 1, "activeSince":19989888, //Unix UTC Timestamp "dataPoints": [ { "title": "Current", "format": "%.3 kW", "output": "actual", } ] }, {
"id":"2", "mode": 2, "accumulated": true "dataPoints": [ { "title": "Grid export", "format": "%.3 kWh", "output": "totalNeg"
16.0
Structure File
Page 21 of 174
LOXONE Create Automation
"title": "Grid Import",
"format": "%.3 kWh",
"output": "total",
] }, {
}
] }
Commands
Unlike with the commands of the previous statistic handling, where the data could only be transferred via a websocket connection, these commands will now work using HTTP-Get requests.
Acquiring info about statistic data
Get info about statistics of a control.
Request: jdev/sps/getStatisticInfo/{controlUuid}
Value field of answer: [{"id":2,"activeSince":1661990400},{"id":1,"activeSince":1661990400}]
· id :selected:
o Statistic group id :unselected:
· activeSince :selected:
o unixUtc-Timestamp, specifies since what date statistics are available :unselected:
Acquiring raw statistic data
· Request :selected:
o dev/sps/getStatistic/{controlUuid}/raw/{fromUnixUtc}/{toUnixUtc}/{dataPointUn it}/{groupId}/{outputName, e.g. actual} :unselected: :selected: returns the statistic data as recorded on the Miniserver during the specified timespan (including from & until).
· {fromUnixUtc}
16.0
Structure File
Page 22 of 174 :selected:
LOXONE Create Automation
. from what point in time onwards the statistic is requested.
· unix timestamp in UTC, not local device or Miniserver time.
· e.g. start of a day
o 2022.09.01 00:00:00 in Austria (GMT+2) is 1661983200
o 2022.09.01 00:00:00 in US (PA) (GMT-4) is 1661961600
= {untilUnixUtc}
· end time of statistic, still included in dataset.
· e.g. end of a day
o 2022.09.01 23:59:59 in Austria (GMT+2) is 1662069599
o 2022.09.01 23:59:59 in US (PA) (GMT-4) is 1662047999
= {dataPointUnit}
· all = all available data points
· hour = a new datapoint for each hour
o e.g. sum of 00:00:00 until 00:59:59
· day = a new datapoint for each day
o e.g. sum of 00:00:00 until 23:59:59
· month = a new datapoint for each month
o e.g sum of 1.2.2022 00:00:00 until 28.2.2022 23:59:59
· year = a new datapoint for each year
= {groupld}
· Which statistic group is requested
= {outputName}
· optional - not required, if not provided, all outputs of the group are returned
. the name of the output for which the statistic is requested for
· Result
o Binary content, see Binary Result
Acquiring preprocessed statistic data
· Request
o dev/sps/getStatistic/{controlUuid}/diff/{fromUnixUtc}/{untilUnixUtc}/{dataPoint Unit}/{groupId}/{outputName, e.g. total}
returns a statistic datapoint for each {dataPointUnit}, where it sums up the difference between the recorded values between {fromUnixUtc} and {untilUtcUnix}
= {fromUnixUtc}
. from what point in time onwards the statistic is requested.
16.0
Structure File
Page 23 of 174 :selected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
. unix timestamp in UTC, not local device or Miniserver time.
· e.g. start of a day
o 2022.09.01 00:00:00 in Austria (GMT+2) is 1661983200
o 2022.09.01 00:00:00 in US (PA) (GMT-4) is 1661961600
= {untilUnixUtc}
· end time of statistic, still included in dataset.
· e.g. end of a day
o 2022.09.01 23:59:59 in Austria (GMT+2) is 1662069599
o 2022.09.01 23:59:59 in US (PA) (GMT-4) is 1662047999
= {dataPointUnit}
· all = all available data points
· hour = a new datapoint for each hour
o e.g. sum of 00:00:00 until 00:59:59
· day = a new datapoint for each day
o e.g. sum of 00:00:00 until 23:59:59
· month = a new datapoint for each month
o e.g sum of 1.2.2022 00:00:00 until 28.2.2022 23:59:59
· year = a new datapoint for each year
= {groupld}
· Which statistic group is requested
= {outputName}
. optional - not required, if not provided, all outputs of the group are returned
. the name of the output for which the statistic is requested for
· Result
o Binary content, see Binary Result
Examples
· request the hourly energy production of a production meter (where >0 means production) from a Miniserver located in Austria on the 1.9.2022
o Request:
getStatistic/diff/1661983200/1662069599/hour/1/total
o Response:
Binary array of entries that contain the following information:
" [unixUtcTimeStamp][hourlyValueDiff]
· 1661983200;0 = 0 kWh von 00:00:00 bis einschließlich 00:59:59
· 1661986800;0 = 0 kWh von 01:00:00 bis einschließlich 01:59:59
16.0
Structure File
Page 24 of 174 :unselected: :unselected: :selected: :unselected: :selected: :unselected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :unselected: :unselected:
LOXONE Create Automation
. ...
· 1662004800;6,5 = 6,5 kWh von 12:00:00 bis einschließlich 12:59:59
. ...
· 1662066000;0 = 0 kWh von 23:00:00 bis einschließlich 23:59:59
· request the energy export power levels of a grid meter (where <0 means export, >0 means import) of a Miniserver located in Austria in August 2022
o Request: :unselected: · getStatistic/raw/1661983200/1662069599/all/1/actual
o Response:
Binary array of entries with the following information
[unixUtcTimeStamp][rawValue]
· 1661983200;3,2 = at 00:00:00 3,2 KW were consumed from the grid
· 1661983262;2,9 = at 00:01:02 2,9 kW were consumed from the grid . ....
· 1662004833 ;- 6,5 = at 12:00:33 6,5 kW were exported to the grid
. ...
· energy stored into a battery daily throughout august 2022 in Austria (GMT+2)
o Request: :unselected:
· getStatistic/diff/1659304800/1661983199/day/1/totalNeg
o Response: :unselected:
Binary array of entries that contain the following information:
[unixUtcTimeStamp][dailyValueDiff]
· 1659304800;6,5 o 6,5k Wh stored on 2022.08.01 from 00:00:00 to 23:59:59
. ....
· 1661896800;12,1 o 12,1 kWh stored, 2022.08.31 from 00:00:00 to 23:59:59
Binary Result
Via websocket binstatisticdata can be downloaded. Here's how it is structured.
· ts Uint32 (4 Bytes)
o unix timestamp in UTC
· values Float64 (8 Bytes)
o amount of values per datapoint depends on the number of outputs requested
o if multiple outputs have been requested, the values are listed in the order of the request.
16.0
Structure File
Page 25 of 174 :unselected: :unselected: :unselected: :selected: :selected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
Secured Details
If the flag "securedDetails" is set, this indicates that a control has sensitive information available, such as credentials for accessing an Intercoms video stream. This sensitive information needs to be requested using a separate, encrypted request.
Commands
· jdev/sps/io/{controlUUID}/securedDetails
o Will return all securedDetails of the control
Control History
Available since version 14.5 .?.?
Some control blocks support tracking a history, providing insight on why the block is currently operating in the way it does. E.g. why did the shading just close?
Blocks supporting the control block history provide a flag "hasHistory: true" within their details property. If this is missing or false, the block does not support it.
Commands
· jdev/sps/io/{controlUUID}/gethistory
o Will return a JSON array containing the history entries.
Data Structure
· Array of JSON objects, each object contains the following properties
o ts - unix-timestamp of the entry
o what - String indicating the action that has been performed, e.g. "Lights off"
o trigger - String indicating the reason as to why it has been performed, e.g. "Input off".
o impacts - array of strings describing what has happened due to the trigger, may be empty or missing if the trigger had no direct impact.
o triggerType - see Trigger Types section for more information on this.
o triggerUuid - optional, depending on the type, a uuid may give away e.g. the user that did perform the action, which can be used to link to the user management do adopt permissions.
Trigger Types
Trigger types provide a non-user friendly string identifying the cause of the action.
16.0
Structure File
Page 26 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected:
LOXONE Create Automation
. user - user interacted with the block using the app.
. control - another control did cause the change of behavior (e.g. via input or central block)
· logic - the control itself
· automaticRule - an automatic designer rule did trigger it
· scene - scene did trigger it.
. centralGw - trigger came from a central block on another MS in a client gw system. trigger may be empty, if the name couldn't be looked up by the MS
· device - a device connected to a block did trigger it.
· generic - may be something identifiable by the triggerUuid (e.g. a memory flag) but may not be visualized
Control Types
In the following sections you will find details on the different types for controls in our LoxAPP3.json. It will not provide a detailed documentation on how and what these controls are being used for. If you lack this info, please see our online documentation.
It will provide info on what commands these controls support and what states they will provide.
AalEmergency
Covered Config Items
· Emergency Alarm
States
· status
o 0: running, normal operation, waiting for emergency button press :unselected:
o 1: alarm triggered :unselected:
o 2: reset input in config asserted, control is shut down :unselected:
o 3: app has temporarily disabled control :unselected:
· disableEndTime
o end time (unix) when control will start to operate again. :unselected:
· resetActive
o text state with the active reset input (if control is in reset) :unselected:
16.0
Structure File
Page 27 of 174 :selected: :selected:
LOXONE Create Automation
Commands
· trigger
o trigger an alarm from the app :unselected:
· quit
o quit an active alarm :unselected:
· disable/<timespan>
o disable the control for the given time in seconds. Set to 0 to start control again :unselected:
if it is disabled.
AalSmartAlarm
Covered Config Items
. AAL Smart Alarm
States
· alarmLevel
o state of alarm :unselected:
0 = no alarm :selected:
1 = immediate alarm :selected:
2 = delayed alarm :selected:
· alarmCause
o A string representing the last cause for an alarm :unselected:
· isLocked
o Reset active, inputs will be ignored and therefore no alarms will be executed :unselected:
· isLeaveActive
o Leave input is set, no alarms will be executed :unselected:
· disableEndTime
o End time for the control to be disabled :unselected:
Commands
· confirm
o Confirm pending alarm :unselected:
· disable/{seconds}
o Disable control for a certain period of time, no alarms will be executed :unselected:
16.0
Structure File
Page 28 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o disable/0 will reenable the Smart Alarm :unselected:
· startDrill
o Execute test alarm :unselected:
Alarm
Covered Config Items
¢ Burglar alarm
Details
· alert
o Not used :unselected:
· presenceConnected
o TRUE if a presence detector is connected to DisMv :unselected:
States
· armed
o If the AlarmControl is armed :unselected:
· nextLevel
o the ID of the next alarm level :unselected:
1 = Silent
2 = Acustic
3 = Optical
4 = Internal
5 = External
6 = Remote
· nextLevelDelay (DEPRECATED as of Version 13.0, use nextLevelAt)
o The delay of the next level in seconds, this can be specified with the parameters :unselected: D1 - D6 in Loxone Config. This increments every second ...
· nextLevelAt
o unix Timestamp when the next alarm level goes off
o initial value = 0 :unselected:
. nextLevelDelayTotal
o The total delay of the next level in seconds, this can be specified with the parameters D1 - D6 in Loxone Config.
16.0
Structure File
Page 29 of 174 :unselected: :unselected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· level
o The ID of the current alarm level :unselected:
= 1 = Silent :selected:
2 = Acustic
3 = Optical
4 = Internal
5 = External
6 = Remote
· armedDelay (DEPRECATED as of Version 13.0, use armedAt)
o The delay of the alarm control being armed :unselected:
· armedAt
o unix timestamp when the alarm is armed :unselected:
o initial value = 0 :unselected:
· armedDelayTotal
o The total delay of the alarm control being armed :unselected:
· sensors (DEPRECATED, handled by subcontrol)
o A string of sensors separated by a pipe ("l") :unselected:
· disabledMove
o If the movement is disabled or not :unselected:
· startTime
o timestamp when alarm started :unselected:
Commands
· on
o Arms the AlarmControl :unselected:
· on/{number}
o number can be 0 or 1 :unselected:
0 = arm without movement :selected:
1 = arm with movement :selected:
o available since Miniserver 7.4.4.14 :unselected:
· delayedon
o Arms the AlarmControl with the given delay (Parameter "Da") :unselected:
· delayedon/{number}
o number can be 0 or 1 :unselected:
0 = delayed arm without movement :selected:
1 = delayed arm with movement :selected:
o available since Miniserver 7.4.4.14 :unselected:
16.0
Structure File
Page 30 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· off
o Disarms the AlarmControl :unselected:
· quit
o Acknowledge the alarm (quit the alarm) :unselected:
· dismv/{number}
o number can be 0 or 1 :unselected:
0 = disable movement
1 = enable movement
AlarmChain
Covered Config Items
· Alarm Sequence
States
· activeAlarmType:
o Bitmap :unselected:
0: Inactive
# 1: Acknowledged
. user has quitted the alarm
2: Alarm
. Equals Input "A" of the AlarmSequence block
4: Urgent Alarm
· Equals Input "AU" of the AlarmSequence block
8: EMS Alarm
. Equals Input "AEs" of the AlarmSequence block
· nextAlarmLevelAt:
o Seconds since 2009 when the next level will be escalated to :unselected:
o null if there is no next level :unselected:
· activeAlarmText:
o Json array with zero to two elements, describing the currently active alarm sequence. Can be an empty string at startup
· nextAlarmText:
o The text describing the next alarm Sequence :unselected:
o empty if there is no next alarm :unselected:
16.0
Structure File
Page 31 of 174 :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· iterationCount: :selected:
o Counts how many iterations have already been made :unselected:
Commands
· quit :selected:
o Acknowledge the alarm (quit the alarm) :unselected:
Subcontrol
· Tracker :selected:
AlarmClock
Operating mode 0, 1 and 2 are prioritized!
Covered Config Items
Alarm clock
Details
· hasNightLight :selected:
o If the control has a Touch Nightlight :unselected:
o Available since Miniserver 9.3 :unselected:
o As of Version 13.0 this flag will always be true, even without a Touch Nightlight :unselected:
Air connected to the block, a default entry exists.
· brightInactiveConnected :selected:
o Is the connector for the nightlight brightness inactive connected :unselected:
o Available since Miniserver 10.0 :unselected:
· brightActiveConnected :selected:
o Is the connector for the nightlight brightness active connected :unselected:
o Available since Miniserver 10.0 :unselected:
· snoozeDurationConnected :selected:
· wakeAlarmSounds :selected:
o Array of all available sounds :unselected:
0 [ :unselected:
■ :selected: {
· "id": 0 :selected:
· "name": "SOUND_NAME" :selected:
■ :selected: }
16.0
Structure File
Page 32 of 174 :selected:
LOXONE Create Automation
■ ...
○ ]
o Available since Miniserver 10.3 :selected: · wakeAlarmSoundConnected
o true if the parameter input S is defined by logic :unselected:
o Available since Miniserver 10.3 :unselected:
· wakeAlarmVolumeConnected
o true if the parameter input Sv is defined by logic :unselected:
o Available since Miniserver 10.3 :unselected:
· wakeAlarmSlopingConnected
o true if the parameter input Sr is defined by logic :unselected:
o Available since Miniserver 10.3 :unselected:
States
· isEnabled
o If the AlarmClock is enabled :unselected:
· isAlarmActive
o If an entry is ringing
· confirmationNeeded
o If the User needs to confirm the entry
· entryList
o Object with all the entries :unselected:
■ :selected: {
· "entryID":
o { :unselected:
" "name": "AlarmClock1",
"isActive": true,
"alarmTime": 29940,
· "modes": [0,1,2,3,5],
· "nightLight: false,
"daily": false
○ :unselected: }
■
}
nightLight and daily are available since Miniserver 9.3
· wakeAlarmSoundSettings
o The settings as a JSON containing the following properties :unselected:
sound
16.0
Structure File
Page 33 of 174 :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
. The ID of the sound found in "wakeAlarmSounds" control details volume
. The numerical volume of the wake alarm sound
isSloping
· If sloping is enabled
o Available since Miniserver 10.3 :unselected:
· currentEntry
o The "entryID" of the current entry :unselected:
o -1 if there is no current entry :unselected:
· nextEntry
o The "entryID" of the next entry :unselected:
o -1 if there is no next entry :unselected:
· nextEntryMode
o Represents operating modes 3 - 9 from our structure file :unselected:
· ringingTime
o countdown in seconds how long the alarmClock will be ringing until it's going :unselected:
to snooze again
· ringDuration
o The duration the AlarmClock is ringing :unselected:
· prepareDuration
o The preparation time in seconds :unselected:
· snoozeTime
o Seconds until snoozing ends :unselected:
· snoozeDuration
o Duration of snoozing :unselected:
. nextEntryTime
o Date of next entry in seconds since 1.1.2009 :unselected:
o available since Miniserver 8.1 :unselected:
· deviceState
o 0 = No Touch Nightlight is connected :unselected:
o 1 = Touch Nightlight is offline :unselected:
o 2 = Touch Nightlight is online :unselected:
o available since Miniserver 9.3 :unselected:
· deviceSettings
o json object that is empty when no nightlight is used :unselected:
o available since Miniserver 10.0 :unselected:
o { :unselected:
o "beepUsed":true, // BOOLEAN if the Buzzer on the nightlight is used :unselected:
16.0
Structure File
Page 34 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o "brightInactive":0, // value - brightness of display when inactive o "brightActive:100 // value - brightness of display when active 0 }
Commands
· snooze
o snoozes the current active entry
· dismiss
o dismisses the current active entry
· entryList/put/{entryID}/{name}/{alarmTime}/{isActive}/{modes|daily}
o Creates an entry or overrides it if the entryID is the same
o entryID :unselected:
The ID of the entry (Existing IDs will be overridden) :selected:
o name :unselected:
The name of the entry :selected:
o alarmTime :unselected:
Alarmtime in seconds since midnight :selected:
o isActive :unselected:
If the entry should be activated per default :selected:
o modes :unselected:
an array of mode IDs (operating modes found in our structure file)
Note: Only for entries without the "nightLight" property
· Example: "1,3,5,6,7" -> "Holidays, Mondays, Wednesday, Thursday, Friday"
o daily :unselected:
Integer value
Note: Only for entries with the "nightLight" property
. 0 = Ringing only once
· 1 = Ringing daily
· entryList/delete/{entryID}
o Deletes an entry with the same entryID
o entryID :unselected:
The ID of the entry
· setPrepDuration/{number}
o Sets the prepare duration :unselected:
number = The Prepare duration in seconds
· setRingDuration/{number}
16.0
Structure File
Page 35 of 174 :unselected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Sets the ringing duration :unselected:
number = The Ringing duration in seconds
· setSnoozeDuration/{number}
o Sets the snoozing duration :unselected:
number = The snoozing duration in seconds
Minimum = 60 :selected:
· setBeepOn/{1 / 0}
o Set if the buzzer on the nightlight should be used :unselected:
· setBrightnessInactive/{0-100}
o Set the display brightness of the nightlight when inactive :unselected:
· setBrightnessActive/{0-100}
o Set the display brightness of the nightlight when active :unselected:
· setWakeAlarmSound/{number}
o Number = Sound ID from details.wakeAlarmSounds :unselected:
· setWakeAlarmVolume/{number}
· setWakeAlarmSlopingOn/{0 or 1}
Application
Covered Config Items
Application
Details
. url: the defined url to the application
· image
· iconColor
o Color of the Icon :unselected:
o Since 13.1? :unselected:
AudioZone
Covered Config Items
16.0
Structure File
Page 36 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
& Music Server Zone
Details
· server
o the UUID of the Loxone Music Server this zone belongs to. See section on :unselected: Loxone Music Server for details.
· playerid
o the ID used to identify this zone within the Loxone Music Server :unselected:
· clientType
o 0 = physically connected :unselected:
o 1 = UPNP :unselected:
· parameterConnections
o bitmask which parameters are controlled via logic :unselected:
■
o the index is in the order of the parameters of the control :unselected:
States
· serverState
o -3 = unknown/invalid zone :unselected:
o -2 = not reachable :unselected:
o -1 = unknown :unselected:
o 0 = offline :unselected:
o 1 = initializing (booting, trying to reach it) :unselected:
o 2 = online :unselected:
· playState
o -1 = unknown :unselected:
o 0 = stopped :unselected:
o 1 = paused (only Casatunes) :unselected:
o 2 = playing :unselected:
· clientState
o only used for UPNP clients! :unselected:
o 0 = offline :unselected:
o 1 = initializing (booting, trying to reach it) :unselected:
o 2 = online :unselected:
· power
o whether or not the client power is active :unselected:
· volume
16.0
Structure File
Page 37 of 174 :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o current volume :unselected:
· maxVolume
o zones can be assigned a maximum volume :unselected:
· volumeStep
o how large a single volume step is (important for button-control) :unselected:
· shuffle
o shuffle/0 = off :unselected:
o shuffle/1 = on :unselected:
· sourceList
o JSON containing all zone-favorites. :unselected:
o e.g .: { "getroomfavs_result": [ {"id": 3, "type":4, "totalitems":6, "start":0, "items": :unselected: [ { "slot": 1, "name": "Led Zeppelin" }, { "slot": 8, "name": "Stüberl" }, { "slot": 7, "name": "Dein Mix der Woche" }, { "slot": 2, "name": "07 Interlude 2.mp3" }, { "slot": 5, "name": "Johnny Cash" }, { "slot": 3, "name": "Große Liste" } ] } ], "command":"audio/cfg/getroomfavs/3/0/10" }
· repeat
o -1 = unknown :unselected:
o 0 = off :unselected:
o 1 = repeat all
o 2 = - not used-
o 3 = repeat current item
· songName
· duration
o how long the whole track is, -1 if not known (stream) :unselected:
· progress
o current position in the track, will be updated every 10 seconds :unselected:
· album
· artist
· station
· genre
· cover
o path to an image representing the current item. :unselected:
· source
o current selected source identifier (integer) :unselected:
o available since Miniserver 7.4.4.14, Music Server 1.1.4.14 :unselected:
· queueIndex
o current song-index in audioqueue :unselected:
16.0
Structure File
Page 38 of 174 :unselected: :unselected: :unselected:
LOXONE Create Automation
o available since Miniserver 10.3.10.5
· enableAirPlay
o Airplay is enabled on the Musicserver
o available since Miniserver 10.3.11.5
· enableSpotifyConnect
o SpotifyConnect is enabled on the Musicserver
o available since Miniserver 10.3.11.5
· alarmVolume
o current volume for Alarm-Events
o available since Miniserver 10.3.11.5
· bellVolume
o current volume for Alarm-Events
o available since Miniserver 10.3.11.5
· buzzerVolume
o current volume for Buzzer-Events
o available since Miniserver 10.3.11.5
· ttsVolume
o current volume for TTS-Events
o available since Miniserver 10.3.11.5
· defaultVolume
o current default-volume
o available since Miniserver 10.3.11.5
· maxVolume
o current max-volume
o available since Miniserver 10.3.11.5
· equalizerSettings
o equalizersettings for this Zone
o comma-separated list of equalizer-values
o available since Miniserver 10.3.11.5 :unselected:
· mastervolume
o mastervolume for grouped zones
o available since Miniserver 10.5.3.24
Commands
· volume/{newVolume}
· volstep/{step}
16.0
Structure File
Page 39 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o increases or decreases the current volume by a step. E.g." volstep/3" or :unselected:
"volstep/-3" o available since Miniserver 8.0 :unselected: :selected: · prev
o previous track :unselected:
· next
o next track :unselected:
· play :selected:
o (urns the client on if needed :unselected:
· pause
· progress/{seconds}
· shuffle :selected:
o toggles the shuffle state on/off :unselected:
· repeat/{repeatState} :selected:
o 0 = off :unselected:
o 1 = repeat list :unselected:
o 3 = repeat track :unselected:
· on
o turns the client on and starts playing right away :unselected:
· off
o turns the client off :unselected:
· svpower/on :selected:
o will wake the Music Server from standby :unselected:
· svpower/off
o will send the Music Server into standby :unselected:
· source/{sourceNumber}
o 1-8, starts playing the corresponding zone-favorite as specified by the app. :unselected:
o as of now, the list of zone-favorites cannot be obtained from the Miniserver via :unselected: this API
· enablespotifyconnect/{1/0} :selected:
o enables spotify Connect on the Musicserver :unselected:
· enableairplay/{1/0} :selected:
o enables Airplay on the Musicserver :unselected:
· alarmvolume/{0-100} :selected:
o set minimum-volume for Alarm-Events :unselected:
· bellvolume/{0-100} :selected:
o set minimum-volume for Bell-Events :unselected:
16.0
Structure File
Page 40 of 174 :selected: :unselected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· buzzervolume/{0-100}
o set minimum-volume for Buzzer-Events :unselected:
· ttsvolume/{0-100}
o set minumum-volume for TTS-Output :unselected:
· defaultvolume/{0-100}
o set default-volume :unselected:
· equalizersettings/{string}
o set equalizer-settings for this zone :unselected:
o equalizersettings/0.0,12.0,7.0,0.0,0.0,8.0,0.0,0.0,0.0,2.0 :unselected:
· mastervolume/{0-100}
o sets mastervolume for grouped zones :unselected:
AudioZoneV2 Covered Config Items
· Music Server Zone Gen. 2
Details
· server
o the UUID of the Loxone Music Server this zone belongs to. See section on :unselected:
Loxone Music Server for details.
· playerid
o the ID used to identify this zone within the Loxone Music Server :unselected:
· clientType
o 0 = physically connected :unselected:
o 1 = UPNP :unselected:
· parameterConnections
o bitmask which parameters are controlled via logic :unselected:
o the index is in the order of the parameters of the control :unselected:
o Parameter-Bits :unselected: :unselected: Bit 1 = Default Volume :unselected: Bit 3 = Alarm Volumes (Alarm + Firealarm) :unselected: Bit 4 = Bell :unselected: Bit 5 = Buzzer :unselected: Bit 6 = TTS :unselected: Bit 10 = Bluetooth
· validOutputs
16.0
Structure File
Page 41 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o FALSE when no Audio output is connected and player is therefore not working :unselected:
· groupObjects
o Array of objects containing uuid of linked music player (only when in a group) :unselected:
· groupName
o Name of group (only when in a group) :unselected:
· volumeModes
o Bitmask of all used Volumemodes :unselected:
0: Audioserver :unselected: :unselected: 1: External Absolute Value
2: External up/down pulses :unselected:
· volup/voldown commands are to be sent instead of absolute values or volume step changes.
o Available since 12.2 :unselected: :unselected:
disableFeature
o Bitmask for disabled features :unselected: :unselected: Bit 0: Spotify :unselected: Bit 1: Airplay :unselected: Bit 2: Bluetooth :unselected: Bit 3: Wlan :unselected: Bit 4: equalizer
· speakerConfig
o Bitmask for information about connected Hardware :unselected: :unselected:
Value 0: no speakers :unselected: Bit 1: Stereo Extension(s) :unselected: Bit 2: Master Speakers :unselected:
Bit 3: Audioserver Channel :unselected: Bit 9: Subwoofer :unselected: Bit 10: Client Speaker :unselected: Bit 11: Any Speaker :unselected: Bit 12: Bluetooth is supported
· playerGroup
o Uuid of player group :unselected:
o Since V15.0 :unselected:
· presence
o Bitmask that tells if presence is allowed on the control :unselected: :unselected: Bit0: Presence is allowed by control-input DisP :unselected: Bit1: Presence is allowed by App-Setting
16.0
Structure File
Page 42 of 174
LOXONE Create Automation :unselected: Bit2: Presence is used on block (Input is connected)
o Presence on the control is allowed when both bits are active :unselected:
o Since V15.3 :unselected:
States
₡ serverState
o -5 = server is rebooting :unselected:
o -2 = not reachable :unselected:
o -1 = unknown :unselected:
o 0 = offline :unselected:
o 1 = initializing (booting, trying to reach it) :unselected:
o 2 = online :unselected:
₡ playState
o -1 = unknown :unselected:
o 0 = stopped :unselected:
o 1 = paused :unselected:
o 2 = playing :unselected:
₡ clientState
o -5 = rebooting :unselected:
o 0 = offline :unselected:
o 1 = initializing (booting, trying to reach it) :unselected:
o 2 = online :unselected:
₡ power
o whether or not the client power is active :unselected:
₡ volumeStep o how large a single volume step is (important for button-control) ₡ :unselected: defaultVolume
₡ volume
₡ alarmVolume
₡ bellVolume
₡ buzzerVolume
₡ ttsVolume
₡ presenceFrom
o Presence simulation start time :unselected:
₡ presenceTo
₡
o Presence simulation end time :unselected: isLocked
16.0
Structure File
Page 43 of 174 :selected:
LOXONE Create Automation
o Status of Reset Input :unselected:
₡ bluetooth
o Info if Bluetooth parameter is on/off :unselected:
o Since V15.0 :unselected:
Commands
₡ volUp ₡ volDown ₡ volume/{value}
₡ È tts/{text}/{volume}
o text = The Text that should be spoken :unselected:
o volume = optional :unselected:
₡ playZoneFav/{id}
₡ prev
o previous track :unselected:
next
o next track :unselected:
& play o (turns the client on if needed :unselected:
¢ Pause
bluetooth/[1/0]
¿ Enable or disable bluetooth
¢ Will return an error if parameter is set by logic
Since V15.0
₡ resetbluetoothpairings
o Resets all bluetooth pairings :unselected:
o Since V15.0 :unselected:
¢ presence/{on,off}
o Enable/disable presence functionality :unselected:
CarCharger
Covered Config Items
₡ Wallbox Gen. 1
16.0
Structure File
Page 44 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
Details
· chargerType
o -1 = no external charger, block only :unselected:
o 0 = KEBA :unselected:
o 1 = BMW :unselected:
· limitAllowed
o whether or not the charging limit input is used. If false, limitMode 2 :unselected: (Automatic) will default to limitMode 0 (maxLImit)
o available since Miniserver 8.0 :unselected:
States
· status (0=Offline, 1=Initializing, 2=Online)
· charging (0,1)
· connected (0,1)
· chargingFinished (0,1)
· power (KW)
· energySession (kWh)
· limitMode (0=Off, 1 = Manual, 2= Automatic)
· currentLimit (KW)
· minLimit (kW)
· maxLimit (kW)
· chargeDuration (Secs)
· showLoadManagement (0,1)
o Standalone, Keba :unselected:
always 1 :selected:
o BMW :unselected:
"Loadmanagement with Miniserver" must be enabled on the BMW :selected: Wallbox
Commands
· charge/on (start charging)
· charge/off (stop/pause charging)
· limitMode/{mode}
o 0 = maxLimit :unselected:
o 1 = manually with App :unselected:
o 2 = Automatic (Input (All) is used) :unselected:
· limit/{limit}
o charging limit between minLimit and maxLimit (KW) :unselected:
o changes only take every 15min affect (BMW only) :unselected:
16.0
Structure File
Page 45 of 174
LOXONE Create Automation
BMW Wallbox specific
Status:
· profiles (String, "l"-separated)
o profiles must be set up on the BMW Wallbox :unselected:
o separate statistics (energy) are supported for each profile :unselected:
· currentProfile (0,1)
Commands:
· profile/{profile}
o switches the profile (0,1) :unselected:
Central Objects
available since Miniserver 9.0
Covered Config Items
₡ CentralAlarm
₡ CentralAudioZone
₡ CentralGate
₡ CentraUJalousie
₡ CentralLightController
₡ CentralWindow
Details
· controls - an array of JSON-Objects
o uuid - uuid of the control :unselected:
o id - ID for selectedControls Command :unselected:
Commands
. Command for selective control of objects
o selectedcontrols/{ids - separated with comma}/{command} :unselected:
o If a selected control requires a visu password, it needs to be provided when :unselected: sending the command to the Central Object.
o If a control isn't accessible to a certain user, it also can't be controlled it via the :unselected:
Central Object.
o for possible values for {command}, please see the concrete types below. :unselected:
· CentralAlarm
o on, off, quit, delayedon :unselected:
16.0
Structure File
Page 46 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· CentralAudio
o play, pause, volup, voldown :unselected:
· CentralGate o open, close, stop :unselected:
· CentraUalousie
o FullUp, FullDown, shade, auto, NoAuto, stop :unselected:
· CentralLightController
o on, reset :unselected:
o setMoods/{uuidLightController1}:{moodld1},{uuidLightController2}:{moodld2}, ... :unselected:
sets the mood of light controller with uuidLightController1 to moodld1, uuidLightController2 to moodId2 and so on
· CentralWindow
o toggle, fullopen, fullclose, moveToPosition, open/[on/off], close/[on/off], :unselected: slightlyOpen, protection, stop
16.0
Structure File
Page 47 of 174 :selected:
LOXONE Create Automation
ClimateController available since Miniserver 10.0
Covered Config Items
ClimateController
Details
· capabilities :selected:
o Defined, what outputs are used :unselected:
o Possible values: :unselected:
0 = None :selected:
1 = Only heating :selected:
= 2 = Only cooling :selected:
3 = Heating and cooling :selected:
· connectedParameters :selected:
o Bitmask which parameters are controlled by logic and may not be adapted by commands :unselected:
o Bit 0: Mode (autoMode/) :unselected:
o Bit 13: Heating Boundary (setHeatingBoundary/) :unselected:
o Bit 14: Cooling Boundary (setCoolingBoundary/) :unselected:
· connectedInputs :selected:
o Bit 0: Outdoor Temperature :unselected:
o Bit 1: Boost :unselected:
o Bit 2: Off :unselected:
o Bit 3: Additional Heating :unselected:
o Bit 4: Fan :unselected:
o Bit 5: Confirm filter change :unselected:
o Bit 6: Excess Cool :unselected:
o Bit 7: Excess Heat :unselected:
o Bit 8: Manual Heating :unselected:
Commands
· resetMaintenance :selected:
o Resets the maintenance counter :unselected: :selected: · setServiceMode/{active}
o Activates or deactivates the service mode :unselected:
o active: :unselected:
Please check "serviceMode" :selected:
· ventilation/{active} :selected:
o Activates the ventilator :unselected:
16.0
Structure File
Page 48 of 174
LOXONE Create Automation
· autoMode/{mode}
o Activated the automatic mode :unselected:
o mode :unselected:
" -1 = Off :selected:
0 = Heating and cooling
· 1 = heating 2 = cooling
· setHeatingBoundary/{temp} :selected:
· setCoolingBoundary/{temp}
States
· controls :selected:
o List of controls (IRCv2) added to the ClimateController :unselected:
o Possible values: :unselected: :selected: uuid = The uuidAction of the represented IRCv2
demand = The demand of the represented IRCv2 :selected:
· - 1 = Cooling :selected:
· 0 = None :selected:
· 1 = Heating
o Example: :unselected:
■ :selected: [
●
{ o uuid: "" o demand: - 1, 0, 1 :unselected: :unselected:
. } :selected:
[ ] :selected:
· currentMode
o The current active mode :unselected:
o Possible values: :unselected:
0 = No requirement :selected:
· 1 = Heating
2 = Cooling
· 3 = Heating boost
4 = Cooling boost
5 = Service mode
6 = External Heater
· autoMode
0 -1 = Off :unselected:
o 0 = Heating and cooling
o 1 = Heating :unselected:
o 2 = Cooling
· currentAutomatic
o The current active automatic mode :unselected:
16.0
Structure File
Page 49 of 174 :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Possible values: :unselected:
0 = Automatic like conditions :selected:
1 = Automatik like average temperature
· temperatureBoundaryInfo
o Information about the temperature boundaries :unselected:
o Possible values :unselected:
· 0 = Not enough data :selected:
· 1 = Ok :selected:
2 = No data at all :selected:
· heatingTempBoundary
o Temperature boundary for heating :unselected:
· coolingTempBoundary
o Temperature boundary for cooling :unselected:
· actualOutdoorTemp
o The outdoor temperature :unselected:
o -1000 = No temperature available :unselected:
· averageOutdoorTemp
o the calcualted average-temperature :unselected:
o -1000 = no 48h average available yet :unselected:
· overwriteReason
o How the control is overwritten :unselected:
o Possible values: :unselected:
. 0 = Automatic :selected:
· 1 = Boost
2 = External Heater
· 3 = Stop
· 4 = Custom Info
· infoText
o The name of the control connected to the currently active overwrite input :unselected:
o Possible values: :unselected:
" ... " = The name of the connected control
· serviceMode
o What service mode setting is currently active :unselected:
o Possible values: :unselected:
. 0 = Off :selected:
= 1 = Standby (Heating & Cooling OFF) :selected:
2 = Heating On
3 = Cooling On
4 = Fan On :selected:
· nextMaintenance
o Unix timestamp when the next maintenance must occur :unselected:
· ventilation
o State of Ventilation-Output :unselected:
16.0
Structure File
Page 50 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· excessEnergy
o Bitmask State of EH / EC-Input: :unselected:
o Bit 0: EH is Active :unselected:
o Bit 1: EC is Active :unselected:
· standbyReason
o Reason why control is not heating or cooling :unselected:
o 0: not in standby :unselected:
o 1: no request from RoomControllers :unselected:
o 2: demand below threshold :unselected:
o 3: mode prevented by outdoor temperature :unselected:
16.0
Structure File
Page 51 of 174
LOXONE Create Automation
ClimateControllerUS available since Miniserver 12.1
Covered Config Items
HVAC Controller
Details
· capabilities :selected:
o Defined, what outputs are used :unselected:
o Bitmask :unselected:
1 Bit = Heating :selected:
2 Bit = Cooling :selected:
3 Bit = Emergency Heat :selected:
4 Bit = Fan (Since 15.1) :selected:
5 Bit = Switch On-Threshold (Since 15.1) :selected:
6 Bit = Servicemode (Since 15.1) :selected:
· type :selected:
o 0 = Furnace (Fossil Fuel) :unselected:
o 1 = Heatpump (Fossil Fuel) :unselected:
o 2 = Heatpump (Electric) :unselected:
· ctype :selected:
o Since 15.1 :unselected:
o Which control type is used for this visualization :unselected:
o Climate Controller, HVAC Controller, ... :unselected:
o Used to provide proper filter-functionality in automatic designer :unselected:
· connectedOutputs :selected:
o Bitmask :unselected:
o 1 Bit = Heat Stage 1 :unselected:
o 2 Bit = Heat Stage 2 :unselected:
o 3 Bit = Compressor :unselected:
o 4 Bit = Cool Stage 2 :unselected:
o 5 Bit = Heat Emergency :unselected:
o 6 Bit = Reversing Valve :unselected:
o 7 Bit = Fan :unselected:
0 8 Bit = Humidifer :unselected:
· connectedInputs :selected:
o Bitmask :unselected:
o 1 Bit = Mode :unselected:
o 2 Bit = Outside temperature :unselected:
o 3 Bit = Boost :unselected:
16.0
Structure File
Page 52 of 174
LOXONE Create Automation
o 4 Bit = Stop
o 5 Bit = Emergency :unselected:
o 6 Bit = Actual Humidity :unselected:
. connectedParameters
o Bitmask :unselected:
o 1 Bit = Minimum off time
o 2 Bit = On Threshold :unselected:
o 3 Bit = Fan overrun time :unselected:
o 4 Bit = Time for second stage :unselected:
o 5 Bit = Delta temp for second stage :unselected:
o 6 Bit = Minimum temperature for cooling :unselected:
o 7 Bit = Maximum temperature for heating
o 8 Bit = Humidity Target :unselected:
Commands
· ventilation/{active}
o Only when not defined via logic (connectedInputs) :unselected:
o 0 / false = automatic ventilation based on demand :unselected:
o 1 / true = activates the ventilator and opens all the vents in the rooms :unselected:
· startVentilationTimer/{secondsSince2009Until}
o Stop with 0 :unselected:
o -1 for always on :unselected:
· setMode/{mode}
o Activated the automatic mode
o Only when not defined via logic (connectedInputs) :unselected:
o mode :unselected:
0 = Off :selected:
# 1 = Heating and cooling :selected:
2 = heating :selected:
3 = cooling :selected:
· startmodetimer/{mode}/{until}/{modeend}
o When wanting to set a mode without a timer please use setMode command :unselected:
o mode = mode that should be set during the time :unselected:
o until = seconds since 2009 until the timer should run :unselected:
Setting this value to zero stops an existing timer :selected:
-1 activates the mode endlessly :selected:
o modeend = mode that should be set after the timer elapses or timer is stopped :unselected:
· useEmergency/{1/0}
o Only when not defined via logic (connectedInputs) :unselected:
o If activated emergency heat is used for heating :unselected:
· startEmergencyTimer/{secondsSince2009Until}
16.0
Structure File
Page 53 of 174 :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Activate emergency input until the given time
o Stop with 0 :unselected:
o -1 for always on :unselected:
· setMinimumTempCooling/{temp}
· setMaximumTempHeating/{temp}
· setServiceMode/{active}
Since 15.1
Activates or deactivates the service mode
₡ active:
Please check "serviceMode"
States
· mode
o 0 = Off
o 1 = Heating and cooling
o 2 = Heating
o 3 = Cooling
· controls
o List of controls (IRCv2) added to the ClimateController :unselected:
o Possible values: :unselected: uuid = The uuidAction of the represented IRCv2 :selected:
demand = The demand of the represented IRCv2 :selected:
· - 1 = Cooling
· 0 = None
· 1 = Heating
o Example: :unselected:
■ :selected: [
· { o uuid: "", o demand: - 1, 0, 1
● }
■ :selected:
]
· currentStatus
o The current active status/mode :unselected:
o Bitmask: :unselected:
1 Bit = Heating first stage active :selected:
2 Bit = Heating second stage active
3 Bit = Heating emergency active
16.0
Structure File
Page 54 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
4 Bit = Cooling first stage active
5 Bit = Cooling second stage active
6 Bit = Switching mode to cool
7 Bit = Switching mode to heat
8 Bit = Switching delayed by reversing valve
9 Bit = Switching delayed by fan overrun time
10 Bit = Switching delayed by minimum off time
11 Bit = Need heat but too hot outside
12 Bit = Need cool but too cold outside
13 Bit = Cooling demand but too less
14 Bit = Heating demand but too less
· fanTimerUntil
o Seconds since 2009 until the fan timer is running
o -1 if the fan runs until manually put back to auto
o 0 when the fan is on auto
· modeTimerUntil
o Seconds since 2009 until the mode override is running
o -1 if the mode override is active without an end specified.
o 0 when no mode timer is active
· emergencyTimerUntil
o Seconds since 2009 until the emergency heating is used
o -1 for always until deactivated
o 0 when the emergency heat is not used
· fan
o Current status of fan output
· emergencyOverride
o State of emergency input or value set via app when nothing is connected to input
· humidity
o Current humidity in % :unselected:
· actualOutdoorTemp
o The outdoor temperature
o -1000 = No temperature available
· minimumTempCooling
o Minimum temperature to be able to cool
· maximumTempHeating
o Minimum temperature to be able to heat
· protectionTemp
o Optional, only available for HVAC USA (details:ctype: 510)
16.0
Structure File
Page 55 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected:
LOXONE Create Automation
o If given, the value defines the lower bound for minimumTempCooling :unselected:
· threshold
o Minimum demand in percent (0-100) to start heating or cooling :unselected:
· demandCool
o Cooling demand of all room controllers in percent (0-100) :unselected:
· demandHeat
o Heating demand of all room controllers in percent (0-100) :unselected:
· ExcessEnergy
o Since V15.1 :unselected:
o Bitmask State of EH / EC-Input: :unselected:
o Bit 1: EH is Active :unselected:
o Bit 2: EC is Active :unselected:
· stage
o Currently active heating/cooling stage :unselected:
o 0 - Off :unselected:
o 1 - First stage :unselected:
o 2 - Second stage :unselected:
· serviceMode
o What service mode setting is currently active :unselected:
o Possible values: :unselected:
0 = Off :selected:
1 = Standby (Heating & Cooling OFF) :selected:
2 = Heating On :selected:
3 = Cooling On
4 = Fan On :selected:
· outdoorTempInfo
o Information about the used outdoor temperature :unselected:
o Possible values :unselected:
· 0 = Not enough data :selected:
· 1 = Ok :selected:
2 = No data at all :selected:
o Since V15.1 :unselected:
· outdoorTempMode
o Since V15.1 :unselected:
o Info about which outdoor temperature is used :unselected:
o 0 = not used :unselected:
o 1 = average of past 48h :unselected:
o 2 = system variable Expected outdoor temperature 48h :unselected:
o 3 = current outdoor temperature :unselected:
16.0
Structure File
Page 56 of 174 :selected:
LOXONE
Create Automation
16.0
Structure File
Page 57 of 174
LOXONE Create Automation
ColorPickerV2
This control type only appears as subcontrol of the LightControllerV2.
Details
₡ minKelvin
o Lowest color-temperature supported :unselected:
o e.g. 2700[K] :unselected:
o Avaliable since 15.0(default: 2700) :unselected:
₡ maxKelvin
o Highest color-temperature supported :unselected:
o e.g. 8500[K] :unselected:
o Avaliable since 15.0(default: 6500) :unselected:
₡ pickerType
o Color-picker type :unselected:
o Rgb/Lumitech :unselected:
Support color temperature + RGB :selected:
o TunableWhite :unselected:
Support only color temperature :selected:
States
₡ color o The color as a string in the text property of the returned JSON :unselected:
o hsv(0,100,100) for RGB Values :unselected:
o temp(100,4483) for color temperatures (brightness, kelvin) :unselected:
₡ sequence
o an JSON-Object containing the description of a sequence :unselected:
colors :selected: :selected: . An array of colors that will be iterated while being active
· z.B .: [ “hsv(0,100,100)", "hsv(100,100,100)", "hsv(200,100,100)"] :selected:
interval :selected:
. seconds how long it takes until one color changes to the next color. :selected:
· between 60s and 3600
" type
· Available since version 13.0 :selected:
16.0
Structure File
Page 58 of 174 :selected:
LOXONE Create Automation
· type of sequence
· 0: rgb
· 2: daylight
mode
· available for daylight-sequence (since version 13.0)
· 4 = Daylight-sequence for direct lighting
· 5 = Daylight-sequence for indirect lighting
₡ sequenceColorIdx
o -1 if the sequence isn't active
o otherwise it's the index of the active target color in the colors array of the current sequence.
Commands
· setFav/{favColorldx}/{color}
o Sets the favorite color
favColorldx
. The index of the color in the favoriteColors array
. A new entry is created if the value already exists
· Value is updated if it already exists
" color
. Either an HSV or an lumitech color string
o hsv(0,100,100) for RGB Values
o temp(100,4483) for brightness/colortemp
· favorite color is deleted if color is empty
· setFavSequence/{sequenceldx}/{duration_seconds}/{colro_n ... }
o sequenceldx :unselected:
Index of Sequence array in GlobalStates
o duration_seconds :unselected:
duration in seconds
o color_n / Maximal 6 :unselected:
Colors separated by /
· setSequence/{duration_seconds}/{color_n ... }/{startIdx}
o duration_second :unselected:
duration in seconds
o color_n/ Maximal 6 :unselected:
Color separated by /
o startldx
16.0
Structure File
Page 59 of 174 :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :unselected: :selected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected:
LOXONE Create Automation
Start index of the sequence
-1 if the position in the sequence should not be modified.
· setBrightness/{value}
o updates the sequencers brightness. :unselected:
· hsv({hue},{saturation},{value})
o sets a certain color, using HSV representation :unselected:
· temp({brightness},{temperature})
o sets a light with a certain color. (warm white, cold white) :unselected:
· daylight({brightness})
o Available since version 13.0 :unselected:
· daylighttype({type})
o Set direct/indirect daylight type for this channel :unselected:
o Available since version 15.1 :unselected:
o Types: 4(direct), 5(indirect) :unselected:
Details
₡ pickerType
o Rgb for RGB Pickers :unselected:
o Lumitech for Lumitech Pickers :unselected:
o TunableWhite: no RGB support, only light-temperature :unselected:
ColorPicker
This control type only appears as subcontrol of the LightController.
Details
€ pickerType
o Rgb for RGB Pickers :unselected:
o Lumitech for Lumitech Pickers :unselected:
States
₡ color o The color as a string in the text property of the returned JSON :unselected:
o hsv(0,100,100) for RGB Values :unselected:
o lumitech(100,4483) for Lumitech Values :unselected:
₡ favorites
o The favorites colors in the text property of the returned JSON :unselected:
o An array of either hsv or lumitech colors :unselected:
16.0
Structure File
Page 60 of 174 :selected: :selected: :selected:
LOXONE Create Automation
Commands
· on :selected:
o Enables the ColorPicker :unselected:
· off
o Disables the ColorPicker :unselected:
· hsv(hue, sat, val) :selected:
o set a new HSV color :unselected:
· lumitech(brigthness,kelvin) :selected:
o set a new color temperature, only supported for Lumitech Pickers :unselected:
· setfav/{favIndex}/{color} :selected:
o favIndex :unselected:
The index of the favorit (1 - 4) :selected:
o color :unselected:
Either an HSV or an lumitech color string :selected:
· hsv(0,100,100) for RGB Values :selected:
· lumitech(100,4483) for Lumitech Values :selected:
Daytimer
Covered Config Items
₡ Schedule
Details
· analog :selected:
o indicates if the daytimer has an analog or digital output :unselected:
· text (digital only) :selected:
o on :unselected:
the text if the value is 1 :selected:
o off :unselected:
the text if the value is 0 :selected:
· format (analog only) :selected:
o the format of the value :unselected:
States
16.0
Structure File
Page 61 of 174 :selected:
LOXONE Create Automation
· modeList
o All available modes in a proprietary list format :unselected:
o 0:mode=0;name=\"Feiertag\",1:mode=1;name=\"Urlaub\" :unselected:
· mode
o current operating mode of the daytimer :unselected:
· override
o the remaining time of the time :unselected:
· value
o current value, 0 or 1 digital and a value for analog :unselected:
· entriesAndDefaultValue
o daytimer event with entries :unselected:
o a default value (used only for analog) :unselected:
· resetActive
o stays active as long as the reset input of the daytimer is active. :unselected:
o Since Config 9.0 :unselected:
· needsActivation :selected:
o Only available if the control needs to be activated :unselected:
Commands
· pulse
o activates the new value if an entry needs activation :unselected:
· default
o changes the default value in the analog daytimer :unselected:
o e.g .: default/8 :unselected:
· startOverride
o starts the timer with a new value :unselected:
o e.g .: startOverride/{value}/{howLongInSecs} :unselected:
· stopOverride
o stops the timer :unselected:
· set
o change entries of the daytimer, :unselected:
o set/{numberOfEntries}/{entry}/{entry}/ ... , :unselected:
o {entry} = {mode};{fromMin};{toMin};{needsActivation};{valueOfEntry} :unselected:
valueOfEntry will always be "0" in digital daytimers, or left out. Digital :selected:
daytimers outputs are "On" as long as an entry exists.
from and to are to be given as minutes since midnight. :selected:
· modeslist :selected:
16.0
Structure File
Page 62 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
o operating modes list sorted by the priority, on the end all weekdays from 3-9
o 7,1,2,3,4,5,6,7,8,9
Intelligent Room Controller Daytimer v2 available since Miniserver 10.0
The daytimer used in the intelligent Room Controller v2 is an analog daytimer where the entries values identify the target temperature (identified by the temperature id, e.g. Comfort Temperature, Building Protection). This control inherits states and details from the Daytimer.
Manual Temperature Range in Schedule
New Since Config xx.xx.xx
Until now, the daytimer-entry values 0, 1 or 2 did indicate which temperature mode is active.
In order to support custom temperatures, entry values can now carry a custom temperature which also defines in which mode the IRC should operate to reach it (heating and cooling, heating only, cooling only). The following steps go into detail on how to identify those entries & extract the contained information:
UINT32 nVal = (UINT32) dVal
BYTE cMode = nVal & 0xFF - erstes Byte ist der Modus
INT16 nTemp = (int16)(nVal >> 8) - nächste 2 Byte: Target * 10
Example, 22,5Celsius with heating only.
1. Cast the entries value to an uint32 (it is transferred as double)
a. Double: 57.648,0
b. UInt32: 57.648
c. Binary: 0000 0000 0000 0000 1110 0001 0011 0000
2. If the 5th bit is set, the value contains a custom temperature and if heating and or cooling is allowed.
a. Binary: 0011 0000
3. Heating or Cooling Mode must defined with Bit's 6-7
a. Bit 6 set = Heating allowed (Binary: 0011 0000)
b. Bit 7 set = Cooling allowed (Binary: 0101 0000)
c. Both bits set = Heating and Cooling allowed. (Binary: 0111 0000)
4. Omit the first 8 bits and extract the next 16bits.
16.0
Structure File
Page 63 of 174 :unselected: :unselected:
LOXONE Create Automation
a. Binary: 0000 0000 1110 0001
5. These 16 Bits represent the target-temperature in INT16-Format with an Factor 10
a. Binary 0000 0000 1110 0001
b. INT16-Value: 225
c. Target-Temperature: 22.5℃
Commands
· set :selected:
o set the calendar entries :unselected:
· modeslist :selected:
o operating modes list :unselected:
Intelligent Room Controller Daytimer
The daytimers used in the intelligent Room Controller are analog daytimers where the entries values identify the target temperature (identified by the temperature id, e.g. Comfort Temperature, Empty House). This control inherits states and details from the Daytimer.
Commands
· setc :selected:
o change entries of the cooling daytimer, :unselected:
· set :selected:
o for heating daytimer :unselected:
· modeslistc :selected:
o operating modes list for the cooling daytimer, modeslist for heating daytimer :unselected:
Pool Daytimer
A pool daytimer is an analog daytimer where the analog value identifies what cycle (e.g. Backwash, Filter) is used while the entry is active. This control inherits states and details from the Daytimer.
Dimmer
Covered Config Items
¢ Dimmer
States
16.0
Structure File
Page 64 of 174
LOXONE Create Automation
₡ position
o The current position for the dimmer :unselected:
¥ min
o The current min value :unselected:
₡ max
o The current max value :unselected:
₡ step
o The current step value :unselected:
Commands
· on :selected:
○ :unselected: Sets the Dimmer to the last known position
· off :selected:
o Disables the dimmer, sets position to 0 :unselected:
· {pos} :selected:
o The position of the Dimmer :unselected:
o If position if over max, max will be set and if position is over min, min will be :unselected:
set
EnergyManager
Covered Config Items
¢ Energy Manager
Details
· loads :selected:
o Json Array containing an object for each used load :unselected:
o {"id": 0, "name": "Name of load", "minOnTime" : 10} :unselected:
o minOnTime - in seconds (when zero no trigger available) :unselected:
States
· currentPower :selected:
o value of Input Alp :unselected:
· currentBat :selected:
o value of Input Alb :unselected:
· loads :selected:
16.0
Structure File
Page 65 of 174 :selected: :unselected:
LOXONE Create Automation
o Json TextEvent :unselected:
o Array containing an object for each used load :unselected:
o {"id":0,"isActive":false,"isPreparing":false,"activeUntil":0, "isWaitingforActivation": :unselected: false, "isPermanentOn": false}
id - identifier/priority of the load (starting at 0)
isActive - Output for the load is currently active
isPreparing - Output is active due to preparing/activation time
activeUntil - Seconds Since 2009 until when the output is active (0 when active without end)
isWaitingforActivation - Needs manual activation so that it can activate
isPermanentOn - True when load is permanentely activated by input or via app
Commands
· trigger/{id}
o Start an output for minimum time or start activation :unselected:
· turnOn/{id}
o Permanent On for load :unselected:
· turnOff/{id}
o Turn load off :unselected:
o Overrides input until next change on input :unselected:
EnergyManager2
Covered Config Items
¢ Energy Manager Gen. 2
Details
· MinSocLocked
o Boolean :unselected:
o Parameter MinSoc has something connect and thus can not be changed :unselected:
· MaxSpwrLocked
o Boolean :unselected:
o Parameter MaxSpwr has something connect and thus can not be changed :unselected:
· HasSsoc
o Ssoc (Storage state of charge) is connected and valid :unselected:
· HasSpwr
16.0
Structure File
Page 66 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Spwr (Storage power) is connected and valid
States
· Gpwr
o value of Input Gpwr (Grid Power) [kW]
· Spwr
o value of Input Spwr (Storage Power) [kW]
· Ppwr
o value of Input Ppwr (Production Power) [KW]
· Ssoc
o value of Input Ssoc (Storage State of Charge) [%]
· MinSoc o value of Parameter MinSoc(Minimum Storage State of Charge) [%]
· MaxSpwr
o value of Parameter MaxSpwr (Max Storage Power) [kW]
· loads
o Json TextEvent
o Array containing an object for each used load
prio-priority of the load (starting at 0)
id - output index (starting at 0) - used for automatic designer actions
name - The name of the load
uuid - Unique identifier used for webservices
icon - path to icon. Request the icon with [miniserver address]/[icon]
· pwr - current actual used power
. The actual used power, this will equal the provided power if the Manager does not know the state
ppwr - provided power, the power which is provided by the Energy Manager
· Since V15.1
hasActual - Boolean, True when the Manager knows the status of the load (e.g. digital status, or actual power)
· False when status input of load is not connected and manager can only assume that load is active when power is provided
· Since V15.1
active - Output for the load is currently active
activatedManually - activated manually (input or app)
deactivatedManually - deactivated manually (input or app)
16.0
Structure File
Page 67 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
minimumActiveUntil - UTC Seconds Since 2009 until when the load is active at minimum (when load has a minimum runtime)
activeDue ToDailyRuntime - Load was activated to reach minimum daily runtime
■
Commands
· manage
o Do a Check of the inputs right now without waiting for any timeouts :unselected:
· setMinSoc/{value}
o Set MinSoc Parameter :unselected:
o Will only work when input is not locked (see details) :unselected:
· setMaxSpwr/{value}
o Set MaxSpwr Parameter :unselected:
o Will only work when input is not locked (see details) :unselected:
· [uuid load]/activate
o Activate load until midnight :unselected:
· [uuid load]/deactivate
o Deactivate load until midnight :unselected:
· [uuid load]/automatic
o Set load back to automatic :unselected:
o Since V13.1.11.2 :unselected:
· [uuid load]/edit/ ...
o Same commands and returns as in expert mode :unselected:
o [uuid load]/edit/load :unselected:
Get edit json in the same format as expert mode :selected:
excluding priority setting :selected:
o [uuid load]/edit/cancel :unselected:
Cancel editing
o [uuid load]/edit/verify/[setting-id]/[setting-value] :unselected:
o [uuid load]/edit/refresh :unselected:
Refresh edit timeout
o [uuid load]/edit/save :unselected:
Apply changed settings
· order/[uuidPrio1],[uuidPrio2] ....
· getStorageSettingDescriptions
16.0
Structure File
Page 68 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o json-Array containing the title, description and unit of the energy storage :unselected:
settings (MinSoc, MaxSpwr)
EnergyFlowMonitor Covered Config Items
· Energy Flow Monitor
Details
· nodes
o array of nodes to be shown in this monitor, see separate section below for more :unselected:
infos
· actualFormat
o the format to use for displaying actual values, e.g. "%.3f kW" for the currently :unselected: consumed power.
· totalFormat
o used for both total & totalNeg values. :unselected:
o the format to use for displaying accumulated values, e.g. "%.3f kWh" for showing :unselected: the total energy consumption of a day/hour/ ...
· storageFormat
o Unit for Storage :unselected:
· rest
o Show Rest of root node :unselected:
· [restName]
o Rest name of root node :unselected:
· [restIcon]
o Icon path of root node rest :unselected:
o e.g. "IconsFilled/power-grid.svg" :unselected:
Node
A node provides information about a meter or group that is to be shown in the energy flow monitor. Nodes can be built up in a tree hierarchy, where one node could contain other nodes. rest
Only meters supporting the new statistic handling are supported to be used in this block.
· Properties (properties in [brackets] are optional)
o uuid -> identifies this node in the EFM, used in getNodeState-Command :unselected:
16.0
Structure File
Page 69 of 174 :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o node Type -> identifies the type of node, the following values exist Grid . "actual" < 0 = currently exporting to grid o accumulated value available via "totalNeg"
. "actual" > 0 = currently consuming from the grid o accumulated value available via "total" Storage
· "actual" < 0 = storage is being charged/filled o accumulated value available via "totalNeg"
· "actual" > 0 = storage is being used (discharged, relieved)
o accumulated value available via "total" Production
· "actual" < 0 = not producing, but consuming o accumulated value available via "totalNeg"
o e.g. solar panels that heat up to melt snow/ice · "actual" > 0 = producing, delivering o accumulated value available via "total" Load
· "actual" < 0 = not consuming, but providing o e.g. a battery or solar energy is located behind this meter - invalid configuration?
o accumulated value available via "totalNeg" · "actual" > 0 = consuming o accumulated value available via "total" Wallbox
· same as consumption
Group
o title -> name to show on the node.
o icon - the source path of the icon to use for this node
o [actualEfmState]
name of the state-property of the EFM representing the live value for this node.
only available for root-level nodes, child nodes need to use the getNodeState-command with their uuid to request state-values.
o [ctrlUuid]
16.0
Structure File
Page 70 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected:
LOXONE Create Automation
uuid of the control represented by this node. Used to request statistics or the states.
If the ctrlUuid is missing, it means that the statistics for this node need to be collected from its child nodes.
o [nodes] :unselected:
List of child-nodes represented by this node. :selected:
o [rest] :unselected:
Show the rest :selected:
o [restName] :unselected:
Rest name of root node :selected:
o [restIcon] :unselected:
Icon path of root node reset :selected:
States
· Ppwr :selected:
o Current production power (unit as defined in the settings of the block) :unselected:
· Gpwr :selected:
o Current grid power (unit as defined in the settings of the block) :unselected:
· Spwr :selected:
o Current storage power (unit as defined in the settings of the block) :unselected:
· Pre :selected:
o Price export per kWh :unselected:
· Pri :selected:
o Price import per kWh :unselected:
· CO2 :selected:
o CO2 Factor (Kg/kWh) :unselected:
· actualO ... actualN :selected:
o for each node that the EFM itself delivers the live state, there is such a state. :unselected:
o returns the actual value, e.g. the current production power. :unselected:
o the "actualEfmState" property of the nodes provide info on what state-property :unselected: delivers the state for that node.
Commands
· getNodeValue/{viewType}/{nodeUuidList} :selected:
o returns a JSON object, where the nodeUuid is the key and the value is a JSON :unselected:
with the output values for the node requested
" { "node1uuid": { "total": 5.6, "totalNeg": "2.1" }, "node2uuid": {"total": 1,2}} :selected:
16.0
Structure File
Page 71 of 174 :selected: :selected:
LOXONE Create Automation
" {"node1uuid": {"actual": 1}, "node2uuid": {"actual": - 1, storage: 50}}
o this command is used to avoid live status updates of all nodes for all viewTypes, as this would cause unnecessary data transfers.
o {nodeUuidList}
List of nodeUuids separated by a semicolon "," o {viewType} can either be
actual -> e.g. the current consumption power
day -> e.g. the energy consumed on the current day
week -> e.g. the energy consumed in the current week
month -> e.g. the energy consumption of the current month
year -> e.g. the energy production of the current year
lifetime -> total/lifetime
· get/{viewType}
o returns a JSON object, with data values which where calculated by the EFM
Actual request contains Gpwr, Ppwr, Spwr, Cpwr (Power of loads), Rest
Day, week, month, .. contain Export, Import, Production, Charge, Discharge, Consumption and Income
o {viewType} can either be
see getNodeValue
Statistics
For this control, the new statistic handling has been implemented, see StatisticV2
Fronius
Covered Config Items
¢ Energy Monitor
States
· prodCurr
o kW current production power
· prodCurrDay o kWh energy production all over the current day
16.0
Structure File
Page 72 of 174 :unselected: :unselected: :selected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :unselected: :selected:
LOXONE Create Automation
. prodCurrMonth
· prodCurrYear
· prodTotal
o kWh energy production since setting up
. consCurr
o kW current consumption power
. consCurrDay
o kWh energy consumed throughout the current day
· gridCurr
o kW current grid consumption/delivery power
o if negative, power is being delivered to the grid
o available since Miniserver 8.1
· batteryCurr
o kW current battery charging/usage power.
o if negative, the battery is charging
o available since Miniserver 8.1
· stateOfCharge
o 0-100, represents the charging state of the battery. 100 = fully charged.
o available since Miniserver 8.1
· earningsDay
o how much money was earned by either consuming the produced power yourself instead of consuming it from the grid, or by exporting unused produced power to the grid. Depends on priceDelivery and priceConsumption
· earningsMonth
· earningsYear
· earningsTotal
· priceDelivery
o Price per unit when exporting to the grid
· priceConsumption
o Price per Unit while consuming from the grid :unselected:
· co2Factor
o How much co2 does it take to produce one kWh, used to compute CO2 savings :unselected:
· generatorType
o 0 = Fronius :unselected:
data supplied by a Fronius generator
o 1 = Inputs
data supplied by the block inputs
16.0
Structure File
Page 73 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected:
LOXONE Create Automation
o 2 = Kostal
data supplied by a Kostal devices
· mode
o The mode gives info on what data sources are available on this energy monitor. As there are several combinations, a bitmask is used to determine whether or not certain sources are available.
ProducedPower: 0x01
Cons -> GridPower: 0x02
Batt -> BattPower: 0x04
ProdEnergy: 0x08
ConsEnergy: 0x16
DelEnergy: 0x32
o Here are some examples for possible resulting values
3 = Production and Consumption available
1 = Production only
2 = Consumption only
0 = No data available
7 = Production, Consumption and Battery available . ..
· online
o 0 = online
o 1 = offline
Gate
Covered Config Items
₡ Gate
Details
· animation
o 0 = Garage Door
o 1 = Single Gate opening to the left
o 2 = Single Gate opening to the right
o 3 = Gate opening to both sides
o 4 = Folding door opening to the left
o 5 = Folding door opening to the right
16.0
Structure File
Page 74 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
States
₡ position
o the position from 1 = open and 0 = closed :unselected: active
₡
o -1 = close :unselected:
o 0 = not moving :unselected:
o 1 = open :unselected:
₡ preventOpen
o 0 = not preventing opening of door :unselected:
o 1 = preventing opening of door :unselected:
₡ preventClose
o 0 = not preventing closing of door :unselected:
o 1 = preventing closing of door :unselected:
Commands
· open :selected:
○ :unselected: Opens the Gate
· close :selected:
o Closes the Gate :unselected:
· stop :selected:
o Stops a moving gate :unselected:
· forceOpen :selected:
o Stops the gate when it is moving and then opens it :unselected:
o Since V12.1 :unselected:
· forceClose :selected:
o Stops the gate when it is moving and then closes it :unselected:
o Since V12.1 :unselected:
· partiallyOpen :selected:
o Moves the gate to the defined partially open position :unselected:
o Since 14.2 :unselected:
Heatmixer
Covered Config Items
₡ Mixing Valve Controller
16.0
Structure File
Page 75 of 174
LOXONE
Create Automation
States
· tempTarget
o temperature the controller currently aims for :unselected:
· tempActual
o actual temperature reported by the sensor attached to the input :unselected:
Hourcounter
Covered Config Items
Maintenance counter
States
· total
o total number of seconds the counter has been active so far :unselected:
· remaining
o how many seconds left until the next maintenance is required :unselected:
o 0 if required or overdue :unselected:
· lastActivation :selected:
o the timestamp (in seconds) when the counter was activated the last time. :unselected:
o updated on each new activation :unselected:
· overdue :selected:
o 0 if not overdue, otherwise maintenance is required :unselected:
· maintenanceInterval
o seconds until the next maintenance :unselected:
· stateUnit
o desired output unit on the UI (state values remain in seconds!) :unselected:
0 = seconds :selected:
1 = minutes :selected:
2 = hours :selected:
3 = days :selected:
· active
o 0/1, whether or not the counter is currently active :unselected:
· overdueSince
o seconds since the maintainanceInterval was exceeded :unselected:
o 0 maintenance is not required yet. :unselected:
16.0
Structure File
Page 76 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
Commands
· reset :selected:
o will cause a reset of the following values :unselected:
remaining to maintenanceInterval :selected:
overdue to 0 :selected:
overdueSince to 0 :selected: :selected: · resetAll
o like reset, but also sets :unselected:
total to 0 :selected:
lastActivation to 0 :selected:
InfoOnlyAnalog
Covered Config Items
₡ Virtual state
Details
₡ format
○ :unselected: the format of the value
States
₡ value
○ :unselected: the current value of the virtual state
InfoOnlyDigital
Covered Config Items
₡ Virtual state
Details
₡ text
○ :unselected: on
16.0
Structure File
Page 77 of 174
LOXONE Create Automation
on text if the value is 1 :selected:
o off :unselected:
off text if the value is 0 :selected:
₡ image
o on :unselected:
uuid of the "on"-image :selected:
o onColor :unselected:
Color of the image. Fill the On image with this color if possible :selected:
Since 13.1 :selected:
o off :unselected:
uuid of the "off"-image :selected:
o offColor :unselected:
Color of the image. Fill the Off image with this color if possible :selected:
Since 13.1 :selected:
₡ color
○ :unselected: on
text color if the value is 1 :selected:
o off :unselected:
text color if the value is 0 :selected:
States
₡ active :unselected: the current value of the virtual state ○
InfoOnlyText
Covered Config Items
Virtual State
Details
· format (like %f) :selected:
States
· text
o Text Event with the text :unselected:
16.0
Structure File
Page 78 of 174
LOXONE Create Automation
Intelligent Room Controller v2 available since Miniserver 10.0
Covered Config Items
¢ Intelligent room controller V2
Details
· format
o defines Temperature-Format (℃ or ºF)
· timerModes
o Defined the available timers
o A manual mode is also available (id = 3), but not available in this structure
o Structure description
" name = Name of the mode
id = ID of the mode, defined by the Miniserver (is used in states.activeMode)
. connectedInputs
o Bitmap; defines which temperature-settings are locked
o Available Bits
Bit 1
Comfort-Temperature Heating
Bit 2
Comfort-Temperature Cooling
Bit 3
Comfort Temperature Heat+Cooling
Bit 4
Allowed Comfort-Tolerance
■
Bit 5
Lower absent Temperature
Bit 6
Upper absent Temperature
■
Bit 7
Allowe deviation absent
Bit 8
Shading temperature heating
Bit 9
Shading temperature cooling
■
Bit 10
FFrostprotect Temperature
Bit 11
HeatProtect Temperature
Bit 12
Mode input
Bit 13
CO2-Level
Bit 14
Indoor Humidity
· possibleCapabilities
o Bitmask of possible capabilities of the room controller
16.0
Structure File
Page 79 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
1 Bit = Active Heat possible
2 Bit = Active Cooling possible
3 Bit = Shading possible
4 Bit = ventilation available
5 Bit = reserved
6 Bit = Passive Cooling (set with shading capability)
■
o Depending on the status of the HVAC/ClimateControllers the capabilities may be :unselected: different during runtime (see state capabilities)
· SingleComfortTemperature
o Since Miniserver 15.1
o If true, the miniserver uses a fixed comfort temperature instead of separate heating/cooling temperature
· linkedAcControls
o Array of UUIDs of the AC Controls linked to this IRC
o Since Miniserver 15.1
@States
· activeMode
o The active mode
o Available modes:
o 0 = Economy
o 1 = Comfort temperature
o 2 = Building protection
o 3 = Manual
o 4 = Off
o @TODO-combined mode can also be set here
· operatingMode
o Available modes
0 = Automatic, heating and cooling allowed
1 = Automatic, only heating allowed
2 = Automatic, only cooling allowed
3 = Manual, heating and cooling allowed
4 = Manual, only heating allowed
5 = Manual, only cooling allowed
16.0
Structure File
Page 80 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
· - 1 = Off
· overrideEntries
o Shows an array of the current active override or an empty array if no active override exists
o Structure
start = Seconds since 2009
end = 0 if currently active or seconds since 2009
reason
· 0 = None
· 1 = Someone is present -> Comfort mode is active
· 2 = Window open -> Eco+ mode is active
· 3 = Comfort override
· 4 = Eco override
· 5 = Eco+ override
· 6 = Prepare State Heat Up
· 7 = Prepare State Cool Down
· 8 = Overriden by source (source needs demand)
o source
the name of the source, e.g: The name of the connected input of the block or null it not available
o isTimer
Indicates, that the timer has been started via the app
true | false
· prepareState
o Possible Values
= - 1 = Cooling down
0 = No Action
1 = Heating up
· overrideReason
o PossibleValues
Please refer to overrideEntries.reason
· tempActual
o The current Temperature
· tempTarget
o The current target Temperature
. comfortTemperature
16.0
Structure File
Page 81 of 174 :unselected: :selected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Comfort temperature for heating
o When using single comfort temperature: comfort temperature for heating and cooling (since 15.1) :unselected:
. comfortTemperatureCool
o Comfort temperature for cooling
o Available since Miniserver version 12.1
· comfortTolerance
o Since 15.1
o The allowed comfort tolerance (+)
· absentMinOffset
o Literally the minimal temperature offset of the economy mode :unselected:
o When using single comfort temperature: Allowed deviation in Eco-Mode (+) :unselected: (since 15.1)
○ :unselected:
· absentMaxOffset
o Literally the maximal temperature offset of the economy mode
· frostProtectTemperature
o Literally the minimal temperature of the Building protection :unselected:
· heatProtectTemperature
o Literally the maximal temperature of the Building protection
· comfortTemperatureOffset
o The offset of the comfort temperature, if adopted. this is temporary and will :unselected: reset itself once the next scheduled comfort window ends.
· openWindow
o state of open Window :unselected:
· excessEnergyTempOffset
o available since miniserver 11.02
o shows relative offset to planned target temperature forced by heating or cooling source (KlimaController, ... )
· temperatureBoundaryInfo
o Information about the temperature boundaries :unselected:
o Possible values :unselected:
= 0 = Not enough data
· 1 = Ok
2 = No data at all
16.0
Structure File
Page 82 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· actualOutdoorTemp
o The outdoor temperature :unselected:
o -1000 = No temperature available :unselected:
· averageOutdoorTemp
o the calcualted average-temperature :unselected:
o -1000 = no 48h average available yet :unselected:
· currentMode
o shows the currently active Mode :unselected:
o values identical to "operatingMode" :unselected:
o as a difference to the state "operatingMode", this state gives information about the :unselected: actual mode in heating+cooling modes
· capabilities
o State because Climate/HVAC Control may not provide cooling or heating due to current outdoor temperature :unselected:
o Bitmask of current capabilities of the room controller :unselected:
1 Bit = Active Heat possible :selected:
2 Bit = Active Cooling possible :selected:
3 Bit = Shading possible :selected:
4 Bit = ventilation available :selected:
· shadingCoolTemp
o Temperature when shading is started in mode cooling :unselected:
· shadingHeatTemp
o Temperature when shading is started in mode heating :unselected:
· shadingOut
o Status of shading output :unselected:
· co2
o Since 15.1 :unselected:
o Current co2 level :unselected:
· humidityActual
o Since 15.1 :unselected:
o Current indoor humidity :unselected:
Commands
· override/{modeld}/[{until}]/[{temp}]
16.0
Structure File
Page 83 of 174 :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Starts a override timer
o modeld :unselected:
The requested modeld of the timer :selected:
o [until] :unselected:
Seconds since 2009 when the timer should end :selected:
o [temp] :unselected:
Only needed if we specifically want to set the temperature (manual mode) :selected:
· stopOverride
Stops a currently running override timer
· setComfortTemperature/{temp}
o temp :unselected:
The comfort temperature for heating :selected:
o When using a single comfort temperature, this webservice sets the the absolute comfort temperature (since 15.1)
· setComfortTemperatureCool/{temp}
o temp :unselected:
The comfort temperature for cooling
o Available since Miniserver version 12.1 :unselected:
· setshadingtemperaturecool/{temp}
o Temperature when shading is started in mode cooling
o Available since Miniserver version 12.1 :unselected:
· setshadingtemperatureheat/{temp}
o Temperature when shading is started in mode heating
o Available since Miniserver version 12.1 :unselected:
· setComfortTolerance/{tolerance}
o Since 15.1 :unselected:
o tolerance :unselected:
The allowed deviation between current temperature and comfort temperature before switching mode Min = 0.5, max = 3.0 :selected:
· setAbsentMinTemperature/{offset}
o offset :unselected:
The minimum temperature in Eco-Mode :selected:
16.0
Structure File
Page 84 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o When using single comfort temperature: Set Allowed deviation in Eco-Mode (+) :unselected: (since 15.1)
· setAbsentMaxTemperature/{offset}
o offset :unselected:
The maximum temperature in Eco-Mode :selected:
· setManualTemperature/{temp}
o temp :unselected:
The Manual Target-Temperature :selected:
· setOperatingMode/{opMode}
o opMode :unselected:
Please check states.operatingMode :selected:
· setComfortModeTemp/{temp} :unselected: o Temporary change the comfort mode temperature. This adoption wil be reset once the next scheduled comfort-mode ends.
o temp :unselected:
Temperature described as an numerical value :selected:
o How to calculate the temp: :unselected:
states.comfortTemperatur + states.comfortTemperatureOffset + step(0.5) :selected:
states.comfortTemperatur + states.comfortTemperatureOffset - step(0.5) :selected:
· set/
o Set calendar-Entries :unselected:
· modeslist/
o Set new Modeslist for Daytimer :unselected:
· setCoolingBoundary/
o sets the upper temperature-boundary for "only cooling allowed" :unselected:
· setHeatingBoundary/
o sets the lower Temperature-boundary for "only heating allowed :unselected:
· activatePresenceSchedule
o Available since 12.0 :unselected:
Sub-Controls
· A daytimer to define when which temperature is to be used
Intelligent Room Controller
Covered Config Items
16.0
Structure File
Page 85 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE
Create Automation
¢ Intelligent room controller
Info
Please note that the Miniserver will convert the temperatures to the unit specified on the Miniserver. This means that if Fahrenheit is set in Loxone Config, the temperatures are transmitted in Fahrenheit.
Details
· restrictedToMode 0 0
Visualize heating and cooling
0 1
Visualize cooling only
0 2
Visualize heating only
· heatPeriodStart, heatPeriodEnd
o Provided if this room controller is using a custom heating period. Returns the month and day when the heating period of this IRoomController will start and end. Missing if the global heating period is used.
o Modified in Config 8.3
· coolPeriodStart, coolPeriodEnd
o the same as heatPeriodStart/heatPeriodEnd but for the cooling period.
· temperatures
o id's of the temperature modes to identify values from the states, e.g.
o temperature-ids
0 = Economy
1 = Comfort Heating
2 = Comfort Cooling
3 = Empty House
4 = Heat Protection
5 = Increased Heat
6 = Party
7 = Manual
o isAbsolute
is the value depending of comfort heating/cooling or an absolute value
States
16.0
Structure File
Page 86 of 174 :unselected: :selected: :unselected: :selected: :unselected: :selected: :unselected: :selected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :selected:
LOXONE Create Automation
₡ # tempTarget
o the current target temperature :unselected:
₡ tempActual
o the current temperature :unselected:
₡ error :unselected: o could be a big difference between target and actual temperature, the actual temperature is bigger than the heat protection temperature or the actual temperature is lower than the empty house temperature
& mode
o information about the mode of the IRoomController
o modes: :unselected:
0 = Automatic :selected:
· cooling or heating depending on the heating/cooling period
. only returned if neither a heating nor a cooling period is active
= 1 = Automatic (currently heating),
2 = Automatic (currently cooling),
3 = Automatic heating,
4 = Automatic cooling,
5 = manual heating,
6 = manual cooling
₡ serviceMode
o the current service mode index
o serviceModes: :unselected:
. 0 = off
1 = heating and cooling off
2 = heating on cooling of
3 = heating off cooling on
4 = heating and cooling on
₡ currHeatTempIx
o the current heating temperature index of the temperatures :unselected: currCoolTempIx
₡
o the current cooling temperature index of the temperatures :unselected:
₡ override
o the remaining time of the timer :unselected: openWindow
₡
o if the window is currently opened :unselected: override Total
₡
16.0
Structure File
Page 87 of 174 :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o the total time with which the timer was started :unselected:
¢ manualMode
o if the user overrides with manual intervention :unselected:
o modes: :unselected:
. 0 = off :selected:
1 = comfort overriding :selected:
2 = economy overriding :selected:
3 = timer overriding (through app) :selected:
4 = movement/presence :selected:
¢ temperatures
o an array of temperatures, index is the same as the id of the existing :unselected: temperatures in the details object
₡ stop
o While this state is on, all outputs of the room controller will remain off, :unselected: regardless of the temperatures. The rest of the room controller will respond as usual.
o available since Miniserver 9.0 :unselected:
Commands
· mode :selected:
o in which mode the IRoomController should work (0-6) :unselected: :selected: Automatic (currently cooling or heating, nr 1 & 2) are not manually selectable, they are chosen depending on the heating/cooling period. Use 3 & 4 instead.
· service :selected:
o to activate the service mode with an id from 0 - 4 :unselected:
· starttimer :selected:
o starts the timer with a temperature id and remaining seconds :unselected:
· stoptimer :selected:
o stops the timer :unselected:
· settemp :selected:
o changes the value of an temperature with a temperature id and the new value :unselected:
Sub-Controls
# 2 Intelligent room controller daytimer for heating and cooling
16.0
Structure File
Page 88 of 174 :unselected: :unselected:
LOXONE Create Automation
Intercom
Covered Config Items
¢ Door Controller
Details
· deviceType :selected:
o 0 = Custom/Unknown :unselected:
o 1 = Loxone Intercom :unselected:
o 2 = Loxone Intercom XL :unselected:
o available since Miniserver 8.0 :unselected:
· videolnfo
o An empty object for legacy reasons :unselected:
· audiolnfo
o An empty object for legacy reasons :unselected:
. The content of videoInfo and audioInfo have been moved to securedDetails in Version :selected: 8.1
· lastBellEventImages
o true if the Miniserver does store images for the last bell event entries. :unselected:
o false or missing if it does not. :unselected:
o available since Miniserver 8.3 :unselected:
· showBellImage :selected:
o Bool :unselected:
o false by default :unselected:
o true -> the app shows the bell image when ringing :unselected:
o false - the app automatically starts the live stream :unselected:
o Since version 12.4.1.14 :unselected:
Secured Details
· videolnfo :selected:
o alertImage :unselected:
an (optional) path to a still image (jpg), used to save pictures for "lastBellEvents" :selected:
o streamUrl :unselected:
The path to either a mjpg stream or jpg images :selected:
16.0
Structure File
Page 89 of 174 :selected: :selected: :selected: :selected:
LOXONE
Create Automation
the streamUrl might contain "remoteConnect" or "cloudDNS" instead of a hostname or ip address. If so, they need to be replaced as follows:
· cloudDNS
o Replace it with the resolved IP of the Miniserver :unselected:
· remoteConnect
o Replace the host and the port with the current hostname & port of the Miniserver.
o https://{adoptedip}.{snr}.dyndns.loxonecloud.com:{port}
o The Miniserver will forward the video from the camera.
o https is mandatory in this case
○ user
o pass
· audiolnfo
o host
the host portion for making a sip-call
o user
the (optional) user portion for making an sip-call
o pass :unselected:
Loxone Intercoms only - the password to authenticate the call
available since Miniserver 8.0
States
¢ bell
o 0 = not ringing.
o 1 = ringing.
₡ lastBellEvents
o Text containing the timestamps for each bell-activity that wasn't answered.
o YYYYMMDDHHMMSS, separated by a pipe
o e.g .: "20151001074904|20151001075008|20151008171552"
o recorded images (if available) can be accessed via "camimage/{uuidAction}/{timestamp}"
₡ version
o Loxone Intercoms only - text containing the currently installed firmware versions.
₡ lastBellTimestamp
16.0
Structure File
Page 90 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected:
LOXONE Create Automation
o Timestamp during ringing when ringing started :unselected:
o Format: YYYYMMDDHHMMSS, :unselected:
o Same timestamp as bell image :unselected:
o From Version 12.2.9.14 :unselected:
Commands
. answer :selected:
o Will deactivate the bell. :unselected:
Sub-Controls
₡
0-3 Outputs of type Pushbutton
o since 8.0 outputs can handle a 'pulse' command. It was ignored in earlier :unselected:
versions.
IntercomV2
Covered Config Items
¢ Intercom
Details
· deviceType :selected:
o 0 = Custom/Unknown :unselected:
o 1 = Intercom :unselected:
· serialNo: :selected:
o The SerialNumber of the device :unselected:
· deviceUuid :selected:
o UUID of the device object :unselected:
· deviceName :selected:
o Name of the device object :unselected:
· optionsFramerate :selected:
o Array with possible framerate options. Use the id in the setvideosettings webservices :unselected: :unselected: o e.g .: { "id": 5, "name": "5 fps" }
16.0
Structure File
Page 91 of 174 :unselected: :selected:
LOXONE Create Automation
· optionsResolution
o Array with possible resolution options. Use the id in the setvideosettings webservices :unselected:
o e.g .: :unselected:
{ "id": 720, "name": "720p" }
· trustAddress
o Is set when the intercom device originates from a trust member :unselected:
o This address must be used to connect to the intercom instead of the connected Miniserver :unselected:
o Since V12.4 :unselected:
· trustMember
o Is set when the intercom device originates from a trust member :unselected:
o Trust member serial from which the intercom device originates :unselected:
o Since V12.4 :unselected:
States
₡ bell
o 0 = not ringing. :unselected:
o 1 = ringing. :unselected:
₡ address
o The resolved IP address of the device :unselected:
₡ answers
₡
o Array of Answers :unselected: muted o Bool - Represents the Qb output of the Control Block :unselected: deviceState
₡
o StateUnknown = 0 :unselected:
o StateOk = 1 :unselected:
o StateRebooting = 2 :unselected:
o StateInitializing = 3 :unselected:
₡ video Settings Intern
o 4 Byte Value divided into 2 pairs
o Each Pair is a setting :unselected:
o Byte 0 & 1: Framerate :unselected:
o Byte 2 & 3: Resolution :unselected:
16.0
Structure File
Page 92 of 174 :unselected: :selected: :selected: :unselected:
LOXONE Create Automation
₡ videoSettingsExtern
o 4 Byte Value divided into 2 pairs :unselected:
o Each Pair is a setting :unselected:
o Byte 0 & 1: Framerate :unselected:
o Byte 2 & 3: Resolution :unselected:
Commands
· answer
o Will deactivate the bell. :unselected:
· playTts/{idx} o {idx} :unselected:
The index of the answer to play :selected:
· mute/{0/1}
o Mute/unmute the Control Block :unselected:
· setAnswers/{answer0}/{answer1}/ ...
o Sets all answers :unselected:
o No empty answer in the middle allowed! :unselected:
· setallvideosettings/{framerate internal}/{resolution internal}/{framerate external}/{resolution external}
· setvideosettings/{internal 0/1}/{framerate}/{resolution}
· setframerate/{internal 0/1}/{framerate}
· setresolution/{internal 0/1}/{resolution}
· setnumberbellimages/{number bell images}
· getnumberbellimages
Sub-Controls
¢ 0-N Outputs of type Pushbutton
Irrigation
Covered Config Items
¢ Irrigation
Details
States
¢ rainActive
16.0
Structure File
Page 93 of 174 :selected: :selected: :selected:
LOXONE Create Automation
expectedPrecipitation
¿ maxExpectedPrecipitation
o Parameter maximum precipitation :unselected:
o When expected precipitation exceeds this value irrigation will not start when :unselected: triggered by automatic rule or logic
₡ currentZone
o Current active zone :unselected:
0 -1 - Off :unselected:
o 0 - Zone 1 :unselected:
o 1 - Zone 2 :unselected:
o ... :unselected:
o 8 - All active :unselected:
₡ zones
o json array containing an object for each used zone :unselected:
o [{"id":0,"name":"Name of zone","duration":600}] :unselected:
· id - index/id of zone (starting at zero)
name - set name of zone
duration - ON duration in seconds
setByLogic - TRUE when duration is set by logic
& rainTime
Total seconds it was raining in the last 24 hours :unselected:
Commands
· startForce
o Start irrigation ignoring expected rain and past rain amount :unselected:
· start
o Start irrigation only when it has not rained enough and will not rain enough :unselected:
· stop
o Stops running irrigation :unselected:
o Irrigation is running when currentZone is not -1 :unselected:
· setDuration/{zone ID}={Duration in seconds}
o Set duration of one zone :unselected:
· setDurations/{zone ID 1}={Duration}/{zone ID 2}={Duration}/ ...
o Set duration of multiple zones :unselected:
· select/{zone ID}
o Activate a zone manually :unselected:
o Activates the zone until stopped :unselected:
16.0
Structure File
Page 94 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o 0 - Deactivate all :unselected:
o 9 - Activate all
Sub-Controls
₡ Tracker
Jalousie
Covered Config Items
₡ Blinds
Automatic blinds
Automatic blinds integrated
¢ Roof Window Shading
Details
· isAutomatic
o If this is an Jalousie with Automatic (automatic blinds, automatic blinds :unselected: integrated)
· animation
o The animation type of the JalousieControl :unselected:
0 = Blinds :selected:
1 = Shutters
2 = Curtain both sides
3 = Not supported
4 = Curtain Left
5 = Curtain Right
· availableModes
o ConfigMode, not documented. :unselected:
· type
o subtype of control :unselected:
348: AutoJalousie :selected:
502: Roof Shade :selected:
States
· up
o Jalousie is moving up :unselected:
16.0
Structure File
Page 95 of 174 :unselected: :selected: :selected:
LOXONE Create Automation
· down
o Jalousie is moving down :unselected:
· position
o The position of the Jalousie, a number from 0 to 1 :unselected:
· Jalousie upper position = 0 :selected:
Jalousie lower position = 1
· shadePosition
o The orientation of the slats (important only for animation 0 = blinds), a number :unselected: from 0 to 1
horizontal slats = 0 :selected:
vertical slats (fully shaded) = 1
· safetyActive
o Only used by ones with Automatic, this represents the safety shutdown :unselected:
. autoAllowed
o Only used by ones with Automatic :unselected:
· autoActive
o Only used by ones with Automatic :unselected:
· locked
o Only used by ones with Automatic, this represents the output Ql in Loxone :unselected: Config. Overrules the safetyActive, as not even for moving to the safety position is allowed.
· hasEndposition
o ConfigMode, not documented. :unselected:
· mode
o ConfigMode, not documented. :unselected:
· learningStep
o ConfigMode, not documented. :unselected:
· infoText
o informs e.g. on what caused the locked state, or what did cause the safety to become active. :unselected:
o available since Miniserver 9.0 :unselected:
· deviceState
o State of used device (0 - no device, 1 - offline, 2 - online) :unselected:
o available since Miniserver 11.3 :unselected:
· targetPosition
o Blind target position (0 = upper position, 1 = lower position) :unselected:
o available since Miniserver 11.3 :unselected:
16.0
Structure File
Page 96 of 174 :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· targetPositionLamelle
o Slats target position (0 = horizontal, 1 = vertical) :unselected:
o available since Miniserver 11.3 :unselected:
· adjustingEndPos
o Contains the Bitmask when Adjusting the End-Position is active :unselected:
o Bits are described in Command "endPosAdjustment" :unselected:
Commands
· up :selected:
o Sends an Up command to the Jalousie, as long as no UpOff or other direction :unselected: command has been sent the Jalousie will go UP
· UpOff
o Sends an UpOff command to the Jalousie, this will stop the up motion of the :unselected: Jalousie
· down
o Sends an Down command to the Jalousie, as long as no DownOff or other :unselected: direction command has been sent the Jalousie will go Down
· DownOff
o Sends an DownOff command to the Jalousie, this will stop the down motion of :unselected: the Jalousie
· FullUp
o Triggers a full up motion :unselected:
· FullDown
o Triggers a full down motion :unselected:
· shade
o Shades the Jalousie to the perfect position :unselected:
· auto
o Enables the Automatic of the Jalousie (if the control supports it) :unselected:
· NoAuto
o This disables the Automatic mode for the Jalousie (if the control supports it) :unselected:
· manualPosition/{targetPosition}
o when received, the jalousie will move to the targetPosition provided :unselected:
o targetPosition :unselected:
= 0 = fully open :selected:
100 = fully closed :selected:
· manualLamelle/{targetPosition}
o when received, the slats will rotate to this position :unselected:
16.0
Structure File
Page 97 of 174 :selected: :selected: :selected:
LOXONE Create Automation
0 = horizontal
= 100 = vertical (user facing edge higher than window facing edge)
· manualPosBlind/{targetPosition}/{targetLamellePosition}
o combination of manualPosition and manualLamelle
· stop
o immediately stops the position, remains in current position
· setAdjustingEndPos/{0/1}
o Used to activate end position adjustment mode. Activating the mode is required to be able to use "endPosAdjustment"
o Requires expert mode permission.
0 = off, both the up and down output turn off and the shading is back to normal operation
1 = on, the command "endPosAdjustment" can be used to activate the up and down outputs.
o The adjust mode has to be kept alive, by resending it every 5000ms
Otherwise, the adjust mode will be disabled and the outputs will turn off 15000ms (15s) after the last command.
· endPosAdjustment/{outputBitmask}.
o Only allowed, when adjustingEndPos is active, which in turn can be activated using the "setAdjustingEndPos"-Command
o Bits
0x00 - Up & Down off.
0x01 -> Down output on
= 0x02 -> Up output on
o Active output commands must be repeated each 200ms, otherwise, the outputs will turn off after 600ms (safety precaution)
o Examples
endPosAdjust/0 - both outputs are off
endPosAdjust/1 -> up output is on
endPosAdjust/2 - down output is on
endPosAdjust/3 - up and down outputs are on
NFC Code Touch
Covered Config Items
16.0
Structure File
Page 98 of 174 :unselected: :unselected: :selected: :unselected: :selected: :unselected: :unselected: :selected: :selected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· NFC Code Touch
Details
· accessOutputs
¢ object with output names, q1-q6
¢ if output isn't used, it isn't listed
· place: optional field with the installation place string if the object has one ¢ available since 10.2
. twoFactorAuth: optional field, true if device requires two authentifications for access available since 10.3
· keyPadAuthType: authentication type int
available since v 13.2
Deprecated since v 15.0
¢ 0 =2FA ¢ 1 = Code or Nfc ¢ 2 = Nfc
¢ 3 = Code
States
· historyDate
¢ unix timestamp in milliseconds from the latest history entry
¢ is 0 if no history exists
¢ reload the history if this updates
· codeDate
¢ unix timestamp in milliseconds from the latest change of some code ¢ the codes has changed, reload them
· deviceState
¢ available since 10.2
¢ bitmap: determines if the linked device has the capability to learn nfc tags.
bit 0: offline
bit 1: dummy
bit 2: nfc unavailable (battery powered air device)
· nfcLearnResult
available since 10.2
¢ json array with nfc objects:
¢ nfc structure:
name - name of nfc tag, or 0
16.0
Structure File
Page 99 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
id - Tag Id, always given.
In case of error:
· 00 00 00 00 00 00 00 00 E8 read error
· 00 00 00 00 00 00 00 00 EE authentication error
· 00 00 00 00 00 00 00 00 EF init error
· 00 00 00 00 00 00 00 00 init error (for backwards compability)
userUuid- uuid of the user this is linked to, or 0
deviceUuid - uuid of a nfc code touch this is linked to, or 0
· keyPadAuthType
¢ Available since V15.0
¢ 0 = 2FA
¢ 1 = Code or Nfc
¢ 2 = Nfc ¢ 3 = Code ¢ 4= OCPP
Commands
· output/{outputNr}
o outputNr can be 1-6 :unselected:
o sends an impuls to the specific output :unselected:
o Code 423 :unselected:
user is not permitted at the moment :selected:
· history
o returns JSON array with history entry objects :unselected:
o entry structure: :unselected:
" ts :selected:
· unix timestamp in utc
" output
· 1-6 for standard operation
· 1-9999 with additional preselection enabled
. 0 for first factor of two factor authentication o available since 10.3 :unselected:
· - 1 for denied access
type
16.0
Structure File
Page 100 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· int, 0-11
· 0 = code (can be associated with one user)
· 1 = code (with name)
· 2 = code (ambiguous, used from multiple users)
· 3 = nfc (can be associated with one user)
· 4 = nfc (with name)
· 5 = nfc (ambiguous, used from multiple users)
· 6 = via app
· 7 = denied code
· 8 = denied NFC Tag
. 9 = denied app access attempt
· 10 = allowed external (available since V15.1)
· 11 = denied external (available since V15.1)
user
· string, optional, name of user (if type is 0, 3, 6, 10, 11)
· if user of trust member, trust member name
description
· string optional
o if type 1: name of code
o if type 3 or 4: name of tag
extMedium
. Only for types 10 and 11
· optional, e.g. "Card" or "OCPP-ID" or "NFC Tag"
extAuthority
. Only for types 10 and 11
· e.g. "Server", "Charging Station Management"
· codes
o JSON array with Code objects from this keypad
o Code structure:
uuid
· string
name
· string
isActive
· boolean, if code is currently active
isEmpty
· boolean, if code is not yet given
16.0
Structure File
Page 101 of 174 :unselected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
" type
· int, 0-5
· 0 = permanent code
· 1 = one-time code
· 2 = time-dependent code
· 3 = valid until code
o available since v 13.2
· 4 = valid from code
o available since v 13.2
· 5 = deactivated code
o available since v 13.2
outputs
· permitted outputs this code can be used for
· bitmask
o 1 = q1, 2 = q2, 3 = q1+q2, 4 = q3, 5 = q1+q3, 6 = q2+q3 ...
standardOutput
· int, 1-6
timeFrom
. optional, if type is 2
· unix timestamp in utc
timeTo
· optional, if type is 2
· unix timestamp in utc
· code/create/{name}/{code}/{type}/{outputs}/{standardOutput}
o creates a new code with the following properties:
o name
string, URIComponent encoded
o code
" string (numeric)
o type
· int, 0-2 (see Code structure above)
o outputs
bitmask (see Code structure above)
o standardOutput
int, 1-6 · code/create/{name}/{code}/{type}/{outputs}/{standardOutput}/{timeFrom}/{timeTo} o same as previous command but for codes with type 2:
16.0
Structure File
Page 102 of 174 :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
o timeFrom :unselected:
unix timestamp in utc :selected:
o timeTo :unselected:
unix timestamp in utc · code/update/{uuid}/{isActive}/{name}/{code}/{type}/{outputs}/{standardOutput} o updates the code with the following properties: :unselected:
o uuid :unselected:
string, code uuid :selected:
o isActive :unselected:
true/false, if the code should be activated or deactivated :selected:
o code :unselected:
code or -1 if code does not change :selected:
· code/update/{uuid}/{isActive}/{name}/{code}/{type}/{outputs}/{standardOutput}/{timeFr om}/{timeTo}
o updates the code with the following properties: :unselected:
o timeFrom :unselected:
o timeTo :unselected:
· code/activate/{uuid}
o activates code :unselected:
· code/deactivate/{uuid}
o deactivates code :unselected:
· code/delete/{uuid}
o deletes the code :unselected:
· nfc/startlearn
o starts a nfc learn session. Returns 423 if device is not capable of executing the command (device offline, not configured or battery powered). Returns 500 in every other error case (User not admin, user has no visu pwd ... ) :unselected:
o available since 10.2 :unselected:
· nfc/stoplearn
o stops a nfc learn session :unselected:
o available since 10.2 :unselected:
LightController
Covered Config Items
¢ Lighting controller
¢ Hotel lighting controller
16.0
Structure File
Page 103 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
Details
· movementScene
o The scene number that is assigned as the movement scene :unselected:
States
₡ activeScene
o The current active scene number :unselected:
₡ sceneList
o Returns a JSON-Object in the following format :unselected:
o uuid -> the UUID of the LightControl :unselected:
o uuidlcon -> This is only used by the Status Control :unselected:
o text -> This is an array of Scenes separated by a "," :unselected:
= "1=\"Szene1\",2=\"Szene2\",7=\"Szene7\"" :selected:
Commands
· off
o Enables lightscene 0 (All off) :unselected:
· on
o Enables lightscene 9 (All on) :unselected:
· {sceneNumber}
o This will activate the scene :unselected:
· {sceneNumber}/learn/{sceneName}
o This learns the current output values to the given scene and overrides it if the :unselected:
scene already exists. Also used to rename a scene or create a new one.
· {sceneNumber}/delete
o This deletes the given scene :unselected:
· plus
o Changes to the next scene :unselected:
o available since Miniserver 8.1 :unselected:
· minus
o Changes to the previous scene :unselected:
o available since Miniserver 8.1 :unselected:
LightControllerV2
Covered Config Items
16.0
Structure File
Page 104 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
₡ Lighting Controller
Details
· masterValue
o the UUID of the master brightness in the subcontrols
o will always be present. :unselected:
· masterColor
o the UUID of the master rgb in the subcontrols
o will only be present if there are subcontrols of the type colorPicker :unselected:
States
₡ activeMoods
o A list of mood ids that are currently active (JSON-Array) ¢ moodList
o Returns an array with JSON-Objects each containing the attributes listed below. The position in the array reflects the list order.
o name :unselected:
The userfriendly name for this mood. :selected:
o shortName :unselected:
Short name (used e.g. for Touch Flex display)
Since version 12.4.2.24
o id An ID that uniquely identifies this mood (e.g. inside activeMoods) (e.g. ID1, ID2, .. )
o t5
whether or not this mood can be controlled with a t5 input
o static
If a mood is marked as static it cannot be deleted or modified in any way. But it can be moved within and between favorite and additional lists.
o used :selected: Bitmask that tells if the mood is used for a specific purpose in the logic. If it's not used, it can be removed without affecting the logic on the Miniserver.
· 0: not used
. 1: this mood is activated by a movement event
16.0
Structure File
Page 105 of 174 :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected:
LOXONE Create Automation
· 2: assigned to a T5/1-n input and there is logic or a touch connected to
. 4: this mood is activated by the Alarm Clock
. 8: assigned to a P/1-n input and there is logic or a presence/movement detector connected to it
o Supported from config V12.xx on
₡ favoriteMoods
o [ID1,ID12,ID4,ID5]
○
₡ additionalMoods
o [ID2,ID3,ID6,ID7,ID8,ID9,ID10,ID11,ID13]
₡ circuitNames
o Available since version 11.3.1.26
o This is used for dynamic names of the subcontrols
o Name of light circuits in a key value json
₡ daylightConfig
o Available since version 13.0
o from -minutes since midnight
o until -> minutes since midnight
o mode
0 - sunrise / sunset
# 1 - custom time
o type
· Key-value json if daylight-lighting-type per light circuit:
Values: 4: direct, 5: indirect
Contains only circuits which supports daylight controls
¢ presence
o Bitmask that tells if presence is allowed on the control
Bit0: Presence is allowed by control-input DisP
16.0
Structure File
Page 106 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
Bit1: Presence is allowed by App-Setting
Bit2: Presence is used on control (any presence or movement input connected)
o Presence on the control is allowed when both bits are active :unselected:
Commands
· changeTo/{moodld}
o This will activate the mood, other moods will be deactivated :unselected:
o The moodID 0 is always "off" :unselected:
· addMood/{moodld}
o adds the provided mood and mixes it with the other moods that are active :unselected:
· removeMood/{moodId}
o deactivates the provided mood, leaves the other moods untouched. :unselected:
· moveFavoriteMood/{moodld}/{newldx}
o moves a mood within the favorite mood list :unselected:
o moodld :unselected:
Id of the mode :selected:
o newldx :unselected:
New Index of the mood
· moveAdditionalMood/{moodld}/{newldx}
o Moves a mood within the additional mood list :unselected:
o moodld :unselected:
Id of the mode
o newldx :unselected:
New Index of the mood
· moveMood/{moodId}/{newldx}
o Moves a mood within the mood list :unselected:
o moodld :unselected:
id of the mode
o newldx :unselected:
New Index of the mood
· addToFavoriteMood/{moodld}
o Moves a mood to the favorite mood (last index) :unselected:
o moodld :unselected:
16.0
Structure File
Page 107 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
Id of the mode
· removeFromFavoriteMood/{moodld}
o Moves a mood from the favorite mood list to the additional mood list :unselected:
o moodld :unselected:
Id of the mode
· learn/{moodld}/{moodName}
o This learns the current output values to the given mood and overrides it if the mood already exists. Also used to rename a mood or create a new one. :unselected:
o moodlds < 8 are linked to the appropriate T5 Input :unselected:
· delete/{moodld}
o This deletes the given mood. :unselected:
· plus
o Changes to the next mood :unselected:
· minus
o Changes to the previous mood :unselected:
· setmoodid/{currentID}/{newID}
o changes the id of an existing mood :unselected:
o if a mood with newID exists, moodIDs are exchanged :unselected:
· setmoodshortname/{moodld}/{new name}
o Set the short name of a mood :unselected:
o since version 12.4.2.24 :unselected:
· setmoodname/{moodld}/{new name}
o Set the name of a mood :unselected:
· setcircuitnames/{json key/value pairs}
o Available since Miniserver version 11.3.1.26 :unselected:
o Json Key must be subcontrol IDs, for example :unselected:
= {"Al1":"Name circuit 1", "uuidaction/Al2":"Name circuit 2"}
· setdaylightconfig/{json}
o available since Version 13.0 :unselected:
o from -minutes since midnight :unselected:
o until -> minutes since midnight
o mode :unselected:
0 - sunrise / sunset
" 1 - custom time
· presence/{on,off}
16.0
Structure File
Page 108 of 174 :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Enable/disable presence functionality
Subcontrols
. There is one subcontrol per light circuit attached to this block and, if available, Master- Brightness and/or Master-Color.
· Types of subcontrols
o Dimmer: :unselected:
· Master-Brightness, if available
Light-Circuits of any dimmer type and suitable smart actuators
o ColorPickerV2: :unselected:
· Master-Color, if available
Light-Circuits of type RGB, Lumitech and suitable smart actuators
o Switch:
Light-Circuits of type switch
· Commands
o setName/[new Name]
Available since Miniserver version 11.3.1.26
Set name of light-circuit, if setting names of more light circuits please use 'setcircuitnames' command
LightsceneRGB
Covered Config Items
¢ RGB lighting controller
Details
· sceneList
o An array of scene names
States
₡ activeScene
o The current active scene number
color
o A string of the current color
· hsv(0, 100, 100)
Commands
16.0
Structure File
Page 109 of 174 :unselected: :unselected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :unselected: :selected: :unselected: :selected:
LOXONE Create Automation
· off
o Enables lightscene 0 (All off) :unselected:
· on
o All on :unselected:
· {sceneNumber}
o this will activate the scene :unselected:
· {sceneNumber}/learn :selected:
o this will override the selected scene with the new selected color If this scene :unselected: does not exist no new scene will be created!
LoadManager
Available Since 12.1.5.25
Covered Config Items
· Load Manager (Last Manager) :selected:
Details
· loads :selected:
o Json array containing an object for each load: :unselected:
o { "id": Index of the load, "name":"Name of load", "power": Power of the load in :unselected:
kW, "hasStatus": boolean if the status in the status bitmask is valid and should be displayed }
o idx=0 is the first load that will be locked :unselected:
· mode :selected:
o 0 = Overload Manager, 1 = Peak Manager, 2 = Peak Overload Manager :unselected:
o Since V15.3 :unselected:
States
· currentPower :selected:
o Current power in kW (rounded to two decimal places) in 'Overload' mode :unselected:
o Average power in kW (rounded to two decimal places) in 'Peak' mode :unselected:
· peakOverloadPower
o Current power in kW (rounded to two decimal places) in 'Peak Overload' mode :unselected:
o Since V15.3 :unselected:
· maxTp
16.0
Structure File
Page 110 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Maximum technical power in kW (rounded to two decimal places) from the block :unselected: parameter in 'Peak Overload' mode
o Since V15.3 :unselected:
· maxPower :selected:
o Maximum power in kW (rounded to two decimal places) :unselected:
· availablePower
o Remaining free Power in kW (rounded to two decimal places) :unselected:
· maxPowerExceeded
o TRUE when maximum power is reached :unselected:
· lockedLoads :selected:
o Bitmask of each locked load :unselected:
o 1 -> locked :unselected:
o First Bit -> First Load :unselected:
· statusLoads :selected:
o Bitmask of each active load :unselected:
o 1 -> active :unselected:
o First Bit -> First Load :unselected:
o Only display status if 'hasStatus' attribute of load is true :unselected:
MailBox Available Since 10.5.1.14
Covered Config Items
· Paketsafe
Details
· deviceType :selected:
o 0 -> no device :unselected:
o 1 -> Air device :unselected:
o 2 -> Tree device :unselected:
States
· notificationsDisabledInput :selected:
o State of the notifications disabled input :unselected:
· packetReceived
o State if a packet has been received :unselected:
· mailReceived
16.0
Structure File
Page 111 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
o State if mail has been received :unselected:
· disableEndTime :selected:
o UTC timestamp until the notifications are disabled in seconds since 2009 :unselected:
Subcontrol
· Tracker :selected:
Commands :selected: · confirmPacket
o Confirm receive of a packet :unselected:
· confirmMail :selected:
o Confirm receive of mail :unselected:
· disableNotifications/[seconds] :selected:
o Disable the notifications for x seconds :unselected:
o 0 seconds for cancelling timer :unselected:
Meter
Covered Config Items
¢ Utility meter
Details :selected: · actualFormat
o format specifier for the value of "actual" :unselected:
totalFormat
o format specifier for the value of "total" :unselected:
₡ [type]
o available since 13.01 :unselected:
o values: :unselected:
unidirectional = default, if missing :selected:
bidirectional :selected:
storage :selected:
legacy (old meter objects) :selected:
₡ [totalFormatNeg]
o unavailable if unidirectional :unselected:
16.0
Structure File
Page 112 of 174
LOXONE Create Automation
o available since 13.01 :unselected:
# [storageFormat]
o only available for storage types :unselected:
o available since 13.01 :unselected:
₡ [storageMax]
o available since 13.01
o this equals 100% filled storage :unselected:
₡ [displayType]
o available since 13.01
o how the app should represent this meter (a container, a battery, ... )
o "regular" or missing -> regular meter UI. :unselected:
o Values: :unselected:
0: regular :selected:
€ [powerName]
o Display Name of 'Power or flow' :unselected:
States
· actual
· total
· [totalNeg]
o available since 13.01 :unselected:
o only available if type is not unidirectional :unselected:
· [storage]
o available since 13.01 :unselected:
o only available if type is storage :unselected:
· [totalDay]
o Meter reading today :unselected:
· [totalWeek]
o Meter reading this week :unselected:
· [totalMonth]
o Meter reading this month :unselected:
· [totalYear]
o Meter reading this year :unselected:
· [totalNegDay]
o Meter reading today :unselected:
o only available if type is not unidirectional :unselected:
16.0
Structure File
Page 113 of 174 :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· [totalNegWeek] :selected:
o Meter reading this week :unselected:
o only available if type is not unidirectional :unselected:
· [totalNegMonth] :selected:
o Meter reading this month :unselected:
o only available if type is not unidirectional :unselected:
· [totalNegYear] :selected:
o Meter reading this year :unselected:
o only available if type is not unidirectional :unselected:
Commands
· reset :selected:
o Resets all values to 0 :unselected:
Statistics
Since 13.01 instances of this control may support the new StatisticV2 handling. Check the JSON property "statisticV2" vs "statistic" to check what type of statistic is supported.
MsShortcut
Covered Config Items
₡ Miniserver Shortcut
Details
₡ trust
o Boolean - True when its a Trust Link :unselected:
o Get the address in this case from the Trust Link API :unselected:
₡ serialNr
o Serial number of the Miniserver :unselected:
₡ localUrl
o Local address of the Miniserver :unselected:
o Only when not a Trust Miniserver :unselected:
₡ remoteUrl
o Remote address of the Miniserver :unselected:
o Only when not a Trust Miniserver :unselected:
16.0
Structure File
Page 114 of 174 :selected:
LOXONE Create Automation
PoolController
Covered Config Items
€ Pool Controller
Info
The PoolControl only works with an Aquastar Air Valve.
Details
· valveType
o The types of the valve
0 = No Valve :selected:
· 1 = Aquastar
o hasEco
If something is connected to the ECO input
o hasTargetTemp
Is input T connected
o hasActualTemp
Is input Al connected o hasWaterLevel
Is input WI connected o waterLevelUnit
The unit for the waterlevel
o hasCoverPosition
Is input CP connected
o hasCover
Is output Qco and Qcc connected
o swimmingMachineType
Is dependent on output AQsm
· 0 = no swimming machine
· 1 = digital value
· 2 = analog value
o hasCustom1
Is input Al1 connected
o hasCustom2
16.0
Structure File
Page 115 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
Is input Al2 connected
o customName1
Name for value of input Al1
o customName2
Name for value of input Al2
o customUnit1
Unit for value of Al1
o customUnit2
Unit for value of Al2
o filterBounds
Min and max values for mode "Filter" in seconds
o backwashBounds
Min and Max values for mode "backwash" in seconds
o rinseBounds
Min and Max values for mode "rinse" in seconds
o circularBounds
Min and Max values for mode "circulate" in seconds
o drainBounds
Min and Max values for mode "drain" in seconds
o hasHeating
Is output Qh connected
o hasCooling
Is output Qc connected
States
· currentOpMode
o current operating mode
0 = Out of order
· while the reset input is active
1 = Automatic
2 = Servicemode
· currentTempMode
o current temperature mode
. 0 = off
1 = Automatic
2 = Automatic heating
3 = Automatic cooling
16.0
Structure File
Page 116 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
4 = manual heating
5 = manual cooling
· tempActual
o The actual water temperature :unselected:
· tempModeCycleActive
o If the temperature regulation cycle is active :unselected:
· tempTarget
o The target temperature :unselected:
· waterLevel
o The actual water level :unselected:
· custom1
o The value of Al1 :unselected:
· custom2
o The value of Al2 :unselected:
· heatingApproved
o If heating is approved, only used if hasHeating is true :unselected:
· coolingApproved
o If cooling is approved, only used if hasCooling is true :unselected:
· ecoActive
o If the eco mode is active :unselected:
· swimmingMachine
o This value can either be digital or analog, please refer to :unselected: "swimmingMachine Type" in the control details
· coverPosition
o Analog value of the cover position :unselected:
0.0 = open :selected:
1.0 = closed :selected:
· coverOpening
o If the cover is opening right now :unselected:
· coverClosing
o If the cover is closing right now :unselected:
· currentCycle
o Current active cycle :unselected:
0 = No cycle is active :selected:
1 = Filter :selected:
2 = Flushing
3 = Circulate
16.0
Structure File
Page 117 of 174 :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
4 = Drain
· remaining Time
o Remaining time of the active cycle in seconds
· valvePosition
o The current position of the valve
-1 = Valve moves
0 = Filter
1 = Backwash
2 = Clearwash
3 = Circulate
4 = Closed
5 = Drain
· 2-way ball valve is activated
· Valve moves to position drain
· Activates pump for a given time
. After given time valve will be deactivated, but the 2-way ball valve and the valve stays at the drain position
6 = Relieve
. This is used to relieve the valve in the winter
· pump
o If the pump is active
· drainValve
o If the drainvalve is opened. There might be a separate drainValve attached that needs to be opened besides setting the valvePosition to drain.
· delayTime
o The time of the delay
This must be in the range of "delayBounds"
· filterTime
o The time in seconds the mode "Filter" will be active
This must be in the range of "filterBounds"
· backwashTime
o The time in seconds the mode "BackwashTime" will be active
This must be in the range of "backwashBounds"
· rinseTime
o The time in seconds the mode "Rinse" will be active
This must be in the range of "rinseBounds"
· circulateTime
16.0
Structure File
Page 118 of 174 :unselected: :unselected: :unselected: :unselected: :selected: :selected: :unselected:
LOXONE Create Automation
o The time in seconds the mode "Circulate" will be active
This must be in the range of "circulateBounds"
· drainTime
o The time in seconds the mode "Drain" will be active
· error
o Error status of the PoolController (available since Miniserver 8.0)
0 = No error is or was present
1 = An error was present
2 = An error is currently present
3 = Device is offline
· cycleAbortable
o State that shows if the current cycle can be aborted
o available since 10.2
Commands
· coverClose
o Closes the cover if one is connected
· coverOpen
o Opens the cover if one is connected
· operatingMode/{opMode}
o Activates the given operating mode
0 = Out of order
1 = Automatic
2 = Servicemode
· tempMode/{tempMode}
o Activates the given temperature mode
. 0 = off
1 = Automatic
2 = Automatic heating
3 = Automatic cooling
4 = manual heating
5 = manual cooling
· eco/{state}
o state
. 0 = off
1 = on
· targetTemp/{temperatur}
16.0
Structure File
Page 119 of 174 :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :unselected:
LOXONE Create Automation
o Sets the given target temperature
· swimmingMachine/{value}
o value can be either analog (0.0 - 1.0) or digital. Please refer to "swimmingMachineType"
· startCycle/{cycleID}/{seconds1}/{seconds2} o starts the given cycle with the given seconds " cycleID"
· 1 = Filter
· 2 = Flushing
· 3 = Circulate
· 4 = Drain
seconds1 = The duration the cycle will be active
. If no seconds are given the values from the states (filterTime, circle Time, .. ) will be used
· filter
o Short for "startCycle/1" :unselected:
· backwash
o short for "startCycle/2"
· circulate
o short for "startCycle/3"
· drain
o short for "startCycle/4"
· valvePos/{position}
o Sets the valve position
position
· 0 = Filter
· 1 = Backwash
· 2 = Clearwash
· 3 = Circulate
· 4 = Closed
· 5 = Drain
Position 6 = Relieve-position cannot be set using a command
· pump/{state}
o Activates or deactivates the pump
o state
■ 0 = off
· 1 = on
16.0
Structure File
Page 120 of 174 :unselected: :unselected: :selected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· drainValve/{state}
o Opens or closes the drainvalve :unselected:
o state :unselected:
. 0 = open :selected:
1 = close :selected:
· reset
o Pulse for reset :unselected:
o reset/1 will activate "Out of order" :unselected:
· disable/{state}
o Disables or enables childlock :unselected:
o state :unselected:
. 0 = off :selected:
· 1 = on :selected:
· delayTime/{time}
o Sets the delaytime in seconds :unselected:
o NOTE: This value must be between "delayBounds" :unselected:
· filterTime/{time}
o Sets the filtertime in seconds :unselected:
o NOTE: This value must be between "filterBounds" :unselected:
· backwashTime/{time}
o Sets the backwashtime in seconds :unselected:
o NOTE: This value must be between "backwashBounds" :unselected:
· rinseTime/{time}
o Sets the rinseTime in seconds :unselected:
o NOTE: This value must be between "rinseBounds" :unselected:
· circulateTime/{time}
o Sets the circulate Time in seconds :unselected:
o NOTE: This value must be between "circulateBounds" :unselected:
· drainTime/{time}
o Sets the drainTime in seconds :unselected:
o NOTE: This value must be between "drainTime" :unselected:
· approveHeating/{value}
o Approves or disapproves heating :unselected:
o value :unselected:
= 1 = approve :selected:
0 = disapprove :selected:
· approveCooling/{value}
16.0
Structure File
Page 121 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Approves or disapproves heating :unselected:
o value :unselected:
= 1 = approve :selected:
0 = disapprove :selected:
· skipDelay :selected:
o This cancels the delay :unselected:
· ackError :selected:
· Acknowledges the current error (available since Miniserver 8.0) :unselected:
The poolcontroller uses a daytimer as subcontrol for defining the schedule.
Pushbutton
Covered Config Items
¢ Virtual Input (Pushbutton)
¢ Push Button
₡ Scene
Details
· type :selected:
o 511 - scene :unselected:
o 71 - virtual input visualized as push button :unselected:
o Missing = Push button block :unselected:
States
· active :selected:
o the current state of the pushbutton :unselected:
Commands
· pulse :selected:
o if the button was only tapped for a very short time, use pulse instead of on and :unselected:
off
· on :selected:
16.0
Structure File
Page 122 of 174 :selected: :selected:
LOXONE Create Automation
o when the button is hit but not released immediately :unselected:
· off :selected:
o to deactivate the button after an on-command :unselected:
Radio
Covered Config Items
₡ Radio buttons (8x)
€ Radio buttons (16x)
Details
· allOff :selected:
o the name shown when no output is selected :unselected:
o Empty string if selecting neither one of the outputs is not an option :unselected:
₡ outputs
o set of output names with their ID as key. (1 - 16) :unselected:
o there might be missing IDs (e.g .: 1,2,5,8 is a valid set of IDs) :unselected:
States
· activeOutput :selected:
o ID of the currently active output. :unselected:
o 0 means no output is selected, show "All Off" (if available) :unselected:
Commands :selected: · reset
o Deselects the currently selected ID, changes activeOutput to 0 :unselected:
· ID :selected:
o Simply sending an ID will activate the corresponding output & change :unselected: activeOutput
o Send '1' for the first output :unselected:
o 0 cannot be selected directly, only via 'reset' :unselected: :selected: · next
o Select the next output. Respects the 'SkO' parameter :unselected:
o Since 13.3.1.10 :unselected:
· prev :selected:
o Select the previous output. Respects the 'SkO' parameter :unselected:
16.0
Structure File
Page 123 of 174 :selected: :selected:
LOXONE Create Automation
○ :unselected: Since 13.3.1.10
PresenceDetector Details
₡ maxEntries
o Maximum number of Tracker-Entries :unselected: parameterConnections
₡
o bitmask of parameters controlled by logic. :unselected:
o the index is in the order of the parameters of the control :unselected:
o Value 1 (Bit0): parameter "time" is locked :unselected:
States
₡ activeSince
o timestamp in seconds since 2009 :unselected:
₡ active
o presence state :unselected:
₡ locked
o locked state :unselected:
₡ infoText
o reason why the presence detector is locked :unselected:
¥ time
o current overrun time (TH) :unselected:
Commands
· {value} :selected:
o value for the virtual input :unselected:
o between min and max :unselected:
· time/{value] :selected:
o set overrun time :unselected:
Subcontrols
· trackerSubcontrol :selected:
16.0
Structure File
Page 124 of 174 :selected: :selected: :selected:
LOXONE Create Automation
PulseAt
Covered Config Items
¢ Pulse At
Details
· pulseDurationLocked
o true wenn Pulse time input is set by logic :unselected:
States
· isActive
o If the Output on Q is 1 :unselected:
· startTime
o seconds since midnight :unselected:
· oneTimePulseDate
O If a fix date for a one time pulse is set secondsSince2009 :unselected:
· pulseDuration
O Pulse duration in seconds :unselected:
· type
o Current used type :unselected:
Commands
· setTime/{time}
· setOneTimePulse/{date}
o Value=0 sets the pulse recurring every day :unselected:
· setPulseDuration/{duration}
· pulse
· setType/{type-id}
O Set the pulse type to one of the types available in the control details :unselected:
Remote
Covered Config Items
₡ Media controller
Details
16.0
Structure File
Page 125 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· favoritePad :selected:
o 0 = D-Pad should be initially visible :unselected:
o 1 = Number-Pad should be initially visible :unselected:
· modeList :selected:
o Object with the modes (usedButtons are currently not used!) :unselected:
" { "1": { "name": "Mode One", "usedButtons": []} :selected:
States
₡ timeout
o The timeout in milliseconds :unselected:
& mode
o The key for the current mode (The key is for the modeList) :unselected:
o 0 means "no mode selected" = all off. :unselected:
₡ active o Will be true if the Miniserver is sending the commands for switching the modes :unselected: or power on
o since Config 8.0 :unselected:
Commands
· mode/{modelD} :selected:
o This enables the mode with the given ID :unselected:
o cannot select mode "0" - use reset for that. :unselected:
· on
o This enables the AQp (Power) output :unselected:
· off
o This disables the AQp (Power) output :unselected:
· mute
o Represents the Mute Button :unselected:
· play
o Represents the Play Button :unselected:
· pause
o Represents the Pause Button :unselected:
· stop
o Represents the Stop Button :unselected:
· rewind :selected:
o Represents the Rewind Button :unselected:
· previous :selected:
16.0
Structure File
Page 126 of 174 :selected: :selected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
o Represents the Previous Button :unselected:
· next
o Represents the Next Button :unselected:
· forward
o Represents the Forward Button :unselected:
· menu
o Represents the Menu Button :unselected:
· info
o Represents the Info Button :unselected:
· exit
o Represents the Exit Button :unselected:
· guide
o Represents the Guide Button :unselected:
· volplus
o Represents the press of the Volume Plus Button :unselected:
· volminus
o Represents the press of the Volume Minus Button :unselected:
· volplusoff
o Represents the release of the Volume Plus Button :unselected:
· volminusoff
o Represents the release of the Volume Minus Button :unselected:
· prgplus
o Represents the press of the Program Plus Button :unselected:
· prgminus
o Represents the press of the Program Minus Button :unselected:
· prgplusoff
o Represents the release of the Program Plus Button :unselected:
· prgminusoff
o Represents the release of the Program Minus Button :unselected:
· return
o Represents the Return Button :unselected:
· btnred
o Represents the Red Button :unselected:
· btnblue
o Represents the Blue Button :unselected:
· btnyellow
o Represents the Yellow Button :unselected:
16.0
Structure File
Page 127 of 174 :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· btngreen
o Represents the Green Button :unselected:
· dirok
o Represents the D-Pad OK Button :unselected:
· dirup
o Represents the press of the D-Pad Up Button :unselected:
· dirupoff
Represents the release of the D-Pad Up Button :unselected:
· dirdown
Represents the press of the D-Pad Down Button :unselected:
· dirdownoff
o Represents the release of the D-Pad Down Button :unselected:
· dirleft
o Represents the press of the D-Pad Left Button :unselected:
· dirleftoff
o Represents the release of the D-Pad Left Button :unselected:
· dirright
o Represents the press of the D-Pad Right Button :unselected:
· dirrightoff
o Represents the release of the D-Pad Right Button :unselected:
· num{x}
o This sends the number "x", "x" goes from 0-9 :unselected:
Example: "num1" for number 1 :selected:
· number/{x}
o This sends the number "x", "x" can be any positive number :unselected:
· Example: "number/18" for number 18 :selected:
· reset
o turns off all devices of the current mode & changes to mode 0 (= no mode :unselected: active)
o available since Miniserver 8.0 :unselected:
Sauna
Covered Config Items
Sauna controller
16.0
Structure File
Page 128 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
Sauna controller with evaporator
Details
· hasVaporizer
o determines whether or not it is a full featured sauna with vaporizer :unselected:
· hasDoorSensor
o whether or not the value of the door sensor can be visualized (might not be :unselected: attached)
States
· active
o whether or not the sauna is active ( != power!) :unselected:
· power
o is it currently heating up :unselected:
· tempActual
o the actual temperature inside the sauna :unselected:
· tempBench
o the actual temperature provided by the bench sensor :unselected:
· tempTarget
o the current target temperature :unselected:
· fan
o is the fan on :unselected:
o indicates the airing phase (if drying is on too) :unselected:
o the airing phase will stop after the airing-time configured on the block is :unselected: reached.
· drying
o indicates that the "drying phase" is on :unselected:
o if the fan is on too, it's called the "airing phase" :unselected:
o the drying will stop once the targetTemp is reached :unselected:
· doorClosed
o active if door is closed, only to be used if hasDoorSensor is true :unselected:
· presence
o forwards the state of the presence input of the block :unselected:
· error
o digital indicator for a sauna error :unselected:
· saunaError
o indicates what error has occurred and why the sauna has shut down. :unselected:
16.0
Structure File
Page 129 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o 0 = no error :unselected:
o 1 = too hot :unselected:
· timer
o seconds left of the sauna timer :unselected:
· timerTotal
o total number of seconds of the sauna timer :unselected:
· lessWater (evaporator only)
o becomes active if the evaporator runs out of "water" :unselected:
· humidityActual (evaporator only)
o actual humidity inside the sauna :unselected:
· humidityTarget (evaporator only)
o target humidity inside the sauna :unselected:
· mode (evaporator only) :unselected:
o when an evaporator is present, different sauna modes can be used, these are identified by a modeNr
o 0 = Off :unselected:
o 1 = Finnish manual operation :unselected:
o 2 = Humidity manual operation :unselected:
o 3 = Finnish automatic operation (80℃) :unselected:
o 4 = Herbal automatic sauna (45℃, 50%) :unselected:
o 5 = Soft steam bath automatic (50℃, 50%) :unselected:
o 6 = Warm air bath automatic (45℃, 45%) :unselected:
Commands
● on
o turns the sauna on :unselected:
· off
o turns the sauna off right away (no airing/drying phase) :unselected:
· fanoff
o turns the fan off :unselected:
· fanon
o turns the fan on, only works if sauna is active :unselected:
· temp/{target}
o set the target temperature (for manual mode) :unselected:
· humidity/{target}
o set the target humidity (hasVaporizer only) :unselected:
· mode/{modeNr}
16.0
Structure File
Page 130 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o see mode state description for details :unselected:
· pulse :selected:
o changes between the sauna activity-states :unselected:
" off -> on
" on -> drying
drying -> airing
airing -> off
· starttimer :selected:
o starts the sand timer, will count down from timerTotal :unselected:
· ontemp/{target} :selected:
o turns the sauna on and sets target temperature :unselected:
Sequential Covered Config Items
· Sequential Controller :selected:
Details
· sequences :selected:
o JSON array containing sequence objects. :unselected:
· [{ id: 4, name: "Gustav" }, {id: 8, name: "Karl}" ] :unselected:
States
· activeSequence :selected:
o id of the currently active sequence :unselected:
o 0 = no sequence active. :unselected:
Commands
· triggerSequence/{sequenceld} :selected:
o activates the sequence with the provided id. :unselected:
o id 0 means that any currently active sequence will be stopped. :unselected:
Slider
Covered Config Items
16.0
Structure File
Page 131 of 174 :selected: :selected: :selected: :unselected:
LOXONE Create Automation
₡ Virtual Input (Slider)
Details
₡ format
o the format of the value :unselected:
¥ min
o the minimum value :unselected:
₡ max
o the maximum value :unselected:
₡ step
o the step to the next value of the slider when pressing "-" or "+" :unselected:
States
· value :selected:
o the current value of the slider :unselected:
· error :selected:
o indicates an invalid value of the slider :unselected:
Commands
· {number} :selected:
o value for the slider :unselected:
o between min and max :unselected:
SmokeAlarm
Covered Config Items
· Fire/water alarm :selected:
Details :selected: · hasAcousticAlarm
o returns true if the smoke alarm control has an acoustic alarm configured :unselected:
· availableAlarms :selected:
o Bitmask showing what is being monitored (may be a combination) :unselected:
0x01: Smoke :selected:
16.0
Structure File
Page 132 of 174 :selected: :selected:
LOXONE Create Automation
0x02: Water
0x04: Temperature
0x08: Arc Fault (electrical wiring), available since Miniserver 9.3
States
· nextLevel
o the ID of the next alarm level
1 = Silent
2 = Acustic
3 = Optical
4 = Internal
5 = External
6 = Remote
· nextLevelDelay
o The delay of the next level in seconds, this can be specified with the parameters D1 - D6 in Loxone Config. This increments every second ...
. nextLevelDelayTotal
o The total delay of the next level in seconds, this can be specified with the parameters D1 - D6 in Loxone Config.
· level
o The ID of the current alarm level
· 1 = Pre Alarm
2 = Main Alarm
· sensors (DEPRECATED, handled by subcontrol)
o A string of sensors separated by a pipe ("I")
· acousticAlarm
o The state of the acoustic alarm 0 for not active and 1 for active
o This only can be 1 if something is connected to the "Qh" Output in Loxone Config and the main alarm is active
· testAlarm
o 0 if testalarm is not active and 1 if it is active
· alarmCause
o Bitmask for Alarm-Causes (may be a combination)
0x01: Smoke
0x02: Water
0x04: Temperature
0x08: Arc Fault (electrical wiring)
16.0
Structure File
Page 133 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· startTime :selected:
o timestamp when alarm started :unselected:
· timeServiceMode :selected:
o Remaining seconds how long the service mode remains active :unselected:
o 99999 - infinite :unselected:
· areAlarmSignalsOff :selected:
o State if all alarm signals are disabled (by confirming alarm) :unselected:
available since 10.3 :selected:
Commands
· mute :selected:
o mutes the sirene :unselected:
· confirm :selected:
o Acknowledge the alarm :unselected:
· servicemode/{number} :selected:
o 0 = Off :unselected:
o 1 = duration is infinite (timeServiceMode state will be set to 99999) :unselected:
o >1 Time in seconds until the service mode stops :unselected:
· startDrill :selected:
o Start a test alarm (available since 10.3) :unselected:
SolarPumpController
Covered Config Items
¢ Thermal Solar Controller
Details
· buffers :selected:
o List of used buffers :unselected:
o eg. [ :unselected:
■ :selected:
{ :selected:
· "name": "Buffer 1"
" }, :selected: :selected: ■ {
· "name": "Buffer2" :selected:
■ :selected: } :unselected: ○ ]
16.0
Structure File
Page 134 of 174
LOXONE Create Automation
States
· bufferTemp{n}
o n: number from 0 to 4 :unselected:
o Temperature of buffer n :unselected:
· bufferState{n}
o n: number from 0 to 4 :unselected:
o State of buffer n :unselected:
o Possible Values :unselected:
0 = Waiting, Buffer is waiting to be heated :selected:
· 1 = Heating, Buffer is heating :selected:
2 = Cooling, Buffer is cooling :selected:
3 = OK, Buffer is heated to its temperature :selected:
· logicOverride
o If the control is overwritten by logic :unselected:
· collectorTemp
o Temperature of the collector :unselected:
· heatOverload
o If heat overload is reached :unselected:
SpotPriceOptimizer
Covered Config Items
¢ Spot Price Optimizer
Details
· format
o Unit format e.g. "%.3f€/kWh" :unselected:
· demandByLogic
o if true demand parameter is set by logic :unselected:
· periodByLogic
o if true period parameter is set by logic :unselected:
· minRuntimeByLogic
o If true minimum runtime is set by logic :unselected:
· hasAutomation
o If something is connected to output O :unselected:
States
16.0
Structure File
Page 135 of 174 :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· active
o If the output is currently active :unselected:
· current
o Current Price/Value :unselected:
· demand
o Value of Demand Parameter
· period
o Value of period Parameter :unselected:
· minRuntime
o Value of minimum runtime parameter, which defines the minimum continuous duration for which the output must remain active once turned on. :unselected:
· manualMax
o Is the price above this value it's always very high :unselected:
· forecastsTimestamp
o Timestamp of the forecasts - when it changes request the forecasts again with :unselected:
'getforecasts'
· cycleFrom
o Seconds since 2009 UTC :unselected:
o Timestamp from what time the current cycle started (rounded to hours) :unselected:
· cycleUntil
o Seconds since 2009 UTC
o Timestamp until what time the current cycle is running (rounded to hours) :unselected:
Commands
· cancel
o Cancel active cycle :unselected:
· start/[demand in hours]/[period in hours]
o Start a new cycle with the given parameters :unselected:
o When demand is set by logic the parameter can be set to 0 :unselected:
· getForecasts
o Current available forecast objects as json array :unselected:
{ "u": "1a173fef-02bf-223c-ffff504f94a00067", "v": 24.795, "s": 1668510000, "e": 1668513600,
16.0
Structure File
Page 136 of 174 :unselected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected:
LOXONE Create Automation
"planned": true },
o u - unique id
o v - value
o s - Start Unix UTC Timestamp
o e - End Unix UTC Timestamp
o planned optional
true when the block has planned to activate the output at this timestamp
StatusMonitor Covered Config Items
Status Monitor
Details
₡ inputs
¢ An array of objects containing information of the inputs monitored by this block.
¢ The index of the object inside this array will correspond to the position of the state for this object in the inputStates array.
¢ An object contains a name for the input, the uuid of the room it is in and the installation place
uuid the uuid of the control which may be visualized to navigate there
· Fallback when control cannot be serialized
name the name of input
room the uuid of the room
installPlace the installation place of the input
· New since V12.0, previously this was part of the name.
status
¢ An object containing information about states. This object includes sub-objects, each representing a specific state. Subobject naming index starts 0.
States without an status value will not appear in this array.
16.0
Structure File
Page 137 of 174 :unselected: :unselected: :selected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
¢ A state has a name, uuid of the state itself, output index, priority and color for the text
name
text of the status
· prio
priority of the state (starting at 0)
@ id
output index of the state (starting at 0)
color
color of the status text
uuid
the uuid of the state itself
States
· numState0 - numState9, numDef
¢ The number of inputs wiht the corresponding state.
¢ The sum of the values from all these states is equal to the number of inputs monitored including integrated status monitors
¢ Inputs that do not have a matching state value will be counted towards numCdef
¢ The identifier "id" from Details->states corresponds with these states.
" e.g. id = 0 -> numState0
e.g. id = 10 - numDef
· inputStates
¢ Will return a string containing the states of each of the inputs by this block including integrated status monitors.
¢ The individual states are separated by a comma. The position of the state in this string corresponds to the position of the objects inside the inputs array under details.
¢ Each state is a integer value that is used to identify the current status the input belongs to (starting at 0)
" e.g. 1 - status with 'id' = 1
¢ Integrated status monitors have the value of the highest priority status.
¢ Integrated unconfigured status monitors have value 255.
SteakThermo
Covered Config Items
16.0
Structure File
Page 138 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
₡ Touch & Grill
o Sensors are counted from left (yellow) to right (green) :unselected:
Details
· deviceType
o 0: No device connected :unselected:
o 1: Touch & Grill Air with two sensors :unselected:
· isTouchProtectConnected
o 0: No logic connected to the input DisT :unselected:
o 1: Logic connected to the input DisT :unselected:
· isBrightnessConnected
o 0: No logic connected to the parameter B :unselected:
o 1: Logic connected to the parameter B :unselected:
States
· currentTemperatures
o TextEvent can be interpreted as JSON :unselected:
0 [ :unselected:
■ :selected: 82,
■ 148
0 ] :unselected:
o Temperatures of the sensors, from left to right :unselected:
· temperatureYellow
o Temperature of the yellow sensor :unselected:
· temperatureGreen
o Temperature of the green sensor :unselected:
· sensorInfo
o TextEvent text can be interpreted as JSON :unselected:
0 [ :unselected:
■ :selected:
· "name": "Brisket",
· "connected": true,
· "target": 85
■ :selected:
■ :selected: {
· "name": "Meat",
· "connected": true,
· "target": 100
16.0
Structure File
Page 139 of 174 :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
" } :selected:
0 ] :unselected:
o name: :unselected:
Name of the sensor, can be set by the user :selected:
o connected :unselected:
If the sensor is connected :selected:
o target :unselected:
defined target temperature of the sensor :selected:
o Index in the Array :unselected:
0 = Left (yellow) :selected:
· 1 = Right (green) :selected:
· targetYellow
o Target temperature of the yellow sensor :unselected:
· targetGreen
o Target temperature of the green sensor :unselected:
· sensorAlarms
o TextEvent can be interpreted as JSON :unselected:
0 [ :unselected:
■ :selected: {
· "text": "Target temperature reached",
· "time": {secondsSince2009}, :selected:
· "ringing": true " }, :selected:
■
{
· "text": "Target temperature reached",
· "time": {secondsSince2009},
· "ringing": false
■ :selected: } :unselected: ○
]
o text: :unselected:
Description of the sensor alarm :selected:
o time: :unselected:
When the alarm has been triggered in seconds since 2009 :selected:
o ringing: :unselected:
If the device is beeping or not :selected:
· yellowAlarmActive
o If the yellow sensor does have an active alarm :unselected:
· greenAlarmActive
o If the green sensor does have an active alarm :unselected:
. activeAlarm Text
o TextEvent the text property is the text of the currently active Alarm :unselected:
o Text is set if Qa is active and logic is connected Qa :unselected:
o If no alarm is active this value will be an empty string :unselected:
16.0
Structure File
Page 140 of 174 :selected: :selected:
LOXONE Create Automation
· timerRemaining
o Remaining time in seconds of the timer :unselected:
· timerInfo
o TextEvent, can be interpreted as JSON o { :unselected: :unselected: :selected:
"active": true, :selected: "duration": 300 0 } :unselected: ■
o active: :unselected:
If the timer is running :selected:
o duration: :unselected:
Time in Seconds :selected:
· timerAlarm :selected:
o TextEvent, can be interpreted as JSON :unselected: :unselected: ○ { "text": "Timer started", :selected:
= "time": {secondsSince2009}, :selected:
"ringing": true :selected: 0 } :unselected:
o text: :unselected:
Description of the timer alarm :selected:
o time: :unselected:
When the alarm has been triggered in seconds since 2009 :selected:
o ringing: :unselected:
If the device is beeping or not :selected:
· timerAlarmActive :selected:
o If a timer alarm is active :unselected:
· deviceState
o The current state of the device :unselected:
o 0 = Running :unselected:
o 1 = Offline :unselected:
o 2 = Turned off :unselected:
o 3 = Error :unselected:
o 4 = Device Asleep :unselected:
· displayAlwaysOnBat
o If the display should stay on while the device is running on battery :unselected:
· displayAlwaysOnDc
o If the display should stay on while the device is connected to power :unselected:
· availableControls
o TextEvent, can be interpreted as JSON :unselected:
0 [ :unselected:
■ :selected:
· "name": "Kitchen",
16.0
Structure File
Page 141 of 174
LOXONE Create Automation
· "uuid": "Of38633e-0073-1a75-ffffed93b67b4c69"
■ ■ :selected: :selected: }, {
· "name": "Garden",
· "uuid": "0b734138-032f-0284-ffff403fb0c34b9e" :selected:
}
○ :unselected:
■ :selected: ]
o Defines the available controls the device has been assigned to :unselected:
· activeControl
o Index of the availableControls state :unselected:
· isActive
o If this particular control is the active control :unselected:
· touchProtection :selected:
o If the touch protection of the device is active :unselected:
· displayBrightness
o Brightness of the display, available since 10.3 :unselected:
o Values from 0 - 100 (%) :unselected:
· powerMode
o The powerMode of the device, available since 10.2.1.16 :unselected:
o 1 = Device is DC powered :unselected:
o 2 = Device is battery powered :unselected:
· batteryStateOfCharge :selected:
o The battery state of charge in percentage, available since 10.2.1.16 :unselected:
o Values from 0 - 100 (%) :unselected:
Commands
· quitAlarm
o Quits all ongoing alarms :unselected:
· setSensor/{sensorIndex}/{targetTemp}/{sensorName}
o sensorIndex :unselected:
The index of the sensor :selected:
· 0 = Left (yellow)
· 1 = Right (green)
o targetTemp :unselected:
The target temperature of the sensor :selected:
o sensorName :unselected:
The name for the sensor :selected:
· setDisplayAlwaysOnBat/{on}
o on :unselected:
16.0
Structure File
Page 142 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
0 = Disabled :selected:
1 = Enabled :selected:
· setDisplayAlwaysOnDc/{on}
○ :unselected: on
0 = Disabled :selected:
· 1 = Enabled :selected:
· setActive/{index} :selected:
o Sets the given control as active :unselected:
o index :unselected:
The index of the control in the availableControls state :selected:
· setThisActive :selected:
o Sets this control as the active control :unselected:
· setTimerDuration/{duration}
o Sets the timer duration :unselected:
o duration :unselected:
The duration in seconds :selected:
· startTimer :selected:
o Starts the timer with the defined duration :unselected:
· stopTimer
o Stops the timer :unselected:
· setTouchProtection/{on}
o Enables or disables the touch protection :unselected:
o on :unselected:
0 = Disabled :selected:
· 1 = Enabled :selected:
· setDisplayBrightness/{brightness}
o Sets the brightness of the display, available since 10.3 :unselected:
o brightness :unselected:
Values from 0 to 100 (%) :selected:
Switch
Covered Config Items
¢ Virtual Input (Switch)
¢ Push button
16.0
Structure File
Page 143 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
States
· active :selected:
o the current state of the switch :unselected:
· lockedOn :selected:
o active when switch cant be deactivated because of a constant on by logic :unselected:
Commands
● :selected: on
o activates the switch :unselected:
· off :selected:
o deactivates the switch :unselected:
SystemScheme
Available since 11.0
Covered Config Items
¢ SystemScheme
Details :selected: · imagePath
o Path to the image to be shown in this block (Path -> uuid from the control) :unselected: :selected: · imageVersion
o Version that can be used to detect image change :unselected:
· mainControl :selected:
o Uuid of control that should be displayed on overview :unselected:
· schemeSize :selected:
o how much space will the scheme require (the image might be larger/smaller and needs to be scaled) :unselected:
o Object, containing width and height attribute :unselected:
· controlReferences :selected: o which controls are being referenced and how/where to show them :unselected:
o each object within the array contains the following attributes: :unselected:
uuidAction :selected:
16.0
Structure File
Page 144 of 174 :unselected: :unselected:
LOXONE Create Automation :selected: . the uuid of the referenced control. The control might not be available within the structure file due to permission restrictions.
pos
· position which the displayed control needs to be centered to :selected:
· Object, containing the x and y position. :selected:
TextState
Covered Config Items
₡ Status
States
· textAndlcon :selected:
o TextEvent with text and icon :unselected:
· iconAndColor :selected:
o Text state with json containing icon and icon color :unselected:
o { "icon":"Icons/xxxx-xxx-xx.svg", "color": "#45864A" } :unselected:
o Since 13.1 :unselected:
TextInput
Covered Config Items
¢ Virtual text input
States
· text :selected: :unselected:
○ TextEvent with text
Commands
· {text} :selected:
o new value of text input :unselected:
TimedSwitch
Covered Config Items
16.0
Structure File
Page 145 of 174
LOXONE Create Automation
₡ Stairwell light switch ¢ Comfort/Multifunction switch
Details
· isStairwayLs :selected:
o true = "Stairwell light switch" :unselected:
o false = "Comfort/Multifunction switch" :unselected:
States
₡ deactivationDelayTotal
o seconds, how long the output will be active if the timer is used. :unselected:
€ deactivationDelay
o countdown until the output is deactivated. :unselected:
0 = the output is turned off :selected:
-1 = the output is permanently on :selected:
otherwise it will count down from deactivationDelayTotal :selected:
Commands
● :selected: on
o Permanently activates the TimedSwitch, deactivationDelay will change to -1 :unselected:
· off :selected:
o Turns off the TimedSwitch, deactivationDelay will change to 0 :unselected:
· pulse :selected:
o deactivationDelay = 0 :unselected:
Will start the countdown, from deactivationDelayTotal to 0 :selected:
o isStairwayLs = true :unselected:
deactivationDelay = - 1 :selected:
. No effect, will remain permanently on. :selected:
deactivationDelay > 0 :selected:
· Restarts the countdown :selected:
o isStairwayLs = false :unselected:
turns it off. (from countdown or permanent on state) :selected:
Tracker
Covered Config Items
16.0
Structure File
Page 146 of 174 :selected: :selected:
LOXONE Create Automation
₡ Tracker
Details
· maxEntries :selected:
o maximal count of entries returned by the miniserver :unselected:
States
· entries :selected: o entries returned from the miniserver as string :unselected:
o entries are separated by a pipe symbol "l" :unselected:
o Hex character "\x14" indicates a new line :unselected:
UpDownLeftRight digital
Covered Config Items
¢ Virtual Input (Left-right buttons)
Virtual Input (up-down buttons)
Commands
· UpOn :selected:
o activates the up/right output :unselected:
· UpOff :selected:
o deactivates the up/right output :unselected:
· PulseUp :selected:
o since Config 8.0 :unselected:
o impuls on up/right output :unselected:
· DownOn :selected:
o activates the down/left output :unselected:
· DownOff :selected:
o deactivates the down/left output :unselected:
· PulseDown :selected:
o since Config 8.0 :unselected:
o impuls on down/left output :unselected:
16.0
Structure File
Page 147 of 174 :selected: :selected: :selected:
LOXONE Create Automation
UpDownLeftRight analog
Covered Config Items
¢ Virtual Input (Left-Right buttons)
¢ Virtual Input (Up-Down buttons)
Details
₡ format
o the format of the value :unselected:
¢ min
o the minimum value :unselected:
¥ max
o the maximum value :unselected:
₡ step
o the step to the next value of the virtual input when pressing up/down/left/right :unselected:
States
● :selected: value
o the current value of the virtual input :unselected:
· error :selected:
o indicates an invalid value of the virtual input :unselected:
Commands
· {value} :selected:
o value for the virtual input :unselected:
o between min and max :unselected:
ValueSelector
Covered Config Items
¢ Push-button +/-
¢ Push-button +
Details
16.0
Structure File
Page 148 of 174 :selected:
LOXONE Create Automation
¢ increaseOnly
o indicates if the button has only an "+"-input :unselected:
₡ format
o the format of the value :unselected:
States
¥ min ○ :unselected: the minimum value
¥ max
o the maximum value :unselected:
₡ step
o the step to the next value of the virtual input when pressing up/down/left/right :unselected:
₡ value
o the current value of the virtual input :unselected:
Commands
· {value} :selected:
o value for the virtual input :unselected:
o between min and max :unselected:
Ventilation
Covered Config Items
₡ Ventilation
Details
₡ hasPresence
o indicates that the "Mv" input is connected to logic :unselected:
¿ hasAbsenceMin
o indicates that the parameter "V" is connected to logic :unselected: ¿ hasAbsenceMax
o indicates that the parameter "Vi" is connected to logic :unselected:
¿ hasPresenceMin
o indicates that the parameter "VP" is connected to logic :unselected:
₡ hasPresenceMax
o indicates that the parameter "VPi is connected to logic :unselected:
16.0
Structure File
Page 149 of 174 :selected: :selected: :selected:
LOXONE Create Automation
¿ hasHumidityMax
o indicates that the parameter "Hmax" is connected to logic :unselected:
¢ hasIndorHumidity
o indicates that the "Hi" input is connected to logic :unselected: modes
₡
o All available modes :unselected:
○ :unselected:
[
■ :selected: {
· name: "Heat exchanger" :selected:
· id: 2 :selected: " }, :selected:
■ :selected: {
· name: "Exhaust", :selected:
· id: 1 :selected:
■ :selected: },
■ :selected: ...
○ :unselected:
]
¢ timerProfiles
o Available timer profiles :unselected:
○ :unselected: [
■ :selected: {
· name: "Resting", :selected:
· useCase: "", :selected:
· interval: 3600, :selected:
· speed: :selected:
○ :unselected:
{
value: 100, :selected:
enabled: true :selected:
○ :unselected:
}
· modes: :selected:
○ :unselected:
[
■ :selected: 0, 1,
■ :selected:
■ :selected:
4,
0 ], :unselected:
· defaultMode: 1 :selected:
" }, :selected:
16.0
Structure File
Page 150 of 174
LOXONE Create Automation
■ ...
○
]
o [useCase]
Optional description of the timer profile, string will be empty if no useCase is given
o interval
Time in seconds this timer should be running
o speed
Object with following properties
· value
o Speed in % of the ventilator
· enabled
o If the current timer profile is allowed to change the speed
o modes
Array of available mode ids for this particular timer profile
These mode ids refer to the ids of the mode object in the modes detail
o defaultMode
defines the per default selected mode for the timerProfile
o In addition manual timers are available, the manual timer must be handled by the app itself, these are not provided by the Miniserver
¢ type
o Defines the type of the control. This will be used to differentiate between different blocks to be able to use different UI or functions
0: Generic
# 1: Leaf
States
₡ ventReason
o The ID for ventilation reason
0: Basic ventilation
1: Increased ventilation
2: Reserved for future use
3: Reserved for future use
4: Stop
· Block input "St"
5: Window opened
16.0
Structure File
Page 151 of 174 :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· Block input "lw"
6: Turbo
· Block input "Tb"
7: Manual
· Overwritten via timer
8: Exhaust
· Block input "Ex"
9: Fall-asleep-mode
· Block input "SI"
10: Frostprotection
· Intake Temperature < TF
· available since 10.2
₡ temperatureSupport
o Whether or not temperature support is active
0 -1: Cooling
o 0: No Temperature support is active
o 1: Heating
₡ activeTimerProfile
o Index of the current active timer profile
0 -1: Manual
o -2: No timer active
0 -3: Someone is changing settings
₡ stoppedBy
o The name of the connected logic if the stop ("St") input is active
₡ overwriteUntil
o Unixtimestamp
o This state is 0 if no timer is active
₡ controlInfo
o TextEvent can be interpreted as JSON o { level: 1, " title: "Error",
desc: "Ventilation left offline",
" link: "https://www.loxone.com",
action:
● { o name: "Acknowledge",
16.0
Structure File
Page 152 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o cmd: "cancelAlarm"
● }
o level } ○ :unselected:
Defines the state of the control :selected:
0: Ok :selected:
· Won't be displayed in the visualisation
1: Error :selected:
· Red colored
2: Warning :selected:
· Orange colored
3: Info :selected:
· Gray colored :selected:
o [title] :unselected:
Optional: The level will be used instead, if no title is defined :selected:
o desc :unselected:
Describes the current state :selected:
o [link] :unselected:
Optional: link to further information, e.g Online documentation :selected:
o [action] :unselected:
Optional: Defines an action object to resolve the current state :selected:
. name: Name of the action :selected:
. cmd: Command that resolves the current state :selected:
₡ speed
o Value in % representing the speed of the ventilation :unselected: mode
₡
o Defines the id of the current active mode defined in the details modes object :unselected: presenceMin
₡
o Minimal procentual value of the ventilation if someone is present :unselected: presenceMax
₡
o Maximal procentual value of the ventilation if someone is present :unselected: absenceMin
₡
o Minimal procentual value of the ventilation if no one is present :unselected: absenceMax
₡
o Maximal procentual value of the ventilation if no one is present :unselected:
₡ humidityIndoor
o Value of the indoor humidity sensor
16.0
Structure File
Page 153 of 174 :unselected: :selected: :unselected: :unselected:
LOXONE Create Automation
₡ presence
o If presence is active :unselected:
₡ frostTemp
o Temperature when the frost protection gets active :unselected:
₡ humidityMax
o The max humidity set (in %) :unselected: airQualityMax
₡
o The max air quality set (in ppm) :unselected:
₡ airQualityIndoor
o The current air quality in ppm) :unselected:
₡ temperatureIndoor
o The current indoor temperature :unselected:
₡ temperatureOutdoor ○ :unselected: The current outdoor temperature
₡ temperatureTarget
o The current target temperature :unselected:
Commands
· setTimer/{interval}/{speed}/{modeld}/{timerProfileldx}
o interval :unselected:
Time in seconds :selected:
o speed :unselected:
Ventilation speed :selected:
0- 100 :selected:
o modeld :unselected:
Id of the mode defined in the details modes object :selected:
o timerProfileldx :unselected:
Array index of the timer :selected:
-1 for the Manual mode :selected:
· setTimer/0
o stops any currently running timer, returns to automatic mode :unselected:
· setAbsenceMin/{value}
o Sets the minimal ventilation intensity if no one is present :unselected:
· setAbsenceMax/{value}
o Sets the maximal ventilation intensity if no one is present :unselected:
· setPresenceMin/{value}
o Sets the minimal ventilation intensity if someone is present :unselected:
· setPresenceMax/{value}
16.0
Structure File
Page 154 of 174 :selected: :selected: :selected:
LOXONE Create Automation
o Sets the maximal ventilation intensity if someone is present :unselected:
· ackFilterChange :selected:
o Acknowledges the "filter change" message :unselected:
Webpage
Covered Config Items
₡ Webpage
Details
· url :selected: o The defined low resolution URL (This will be the Internal URL if you are :unselected: connected internally and the external URL if you are connected externally)
· urlHd :selected:
o The defined high resolution URL (This will be the Internal URL if you are :unselected: connected internally and the external URL if you are connected from externally)
· image :selected:
· iconColor :selected:
o Color of the Icon :unselected:
o Since 13.1 :unselected:
Window
Covered Config Items
· Window :selected:
States
· position :selected:
o window position :unselected:
o 0 .. fully closed :unselected:
o 1 .. fully opened :unselected:
o 0 < pos < 1 ... something in between :unselected:
· direction :selected:
o -1 .. window is getting closed :unselected:
o 0 .. window is not moving :unselected:
o 1 .. window is getting opened :unselected:
· lockedReason :selected:
16.0
Structure File
Page 155 of 174 :selected: :unselected:
LOXONE Create Automation
o Name of the reason why the control is blocked. It is the name of the input connected with "WP" :unselected:
· targetPosition :selected:
o Window target position (0 = fully closed - 1 = fully opened) :unselected:
o available since Miniserver 11.3 :unselected:
· type :selected:
o Window Type for Visualisation :unselected:
Since V15.3 :selected:
o 0 ... Tilt Top :unselected:
o 1 ... Tilt Bottom :unselected:
o 2 ... Tilt Left :unselected:
o 3 ... Tilt Right :unselected:
o 4 ... Sliding Left :unselected:
o 5 ... Sliding Right :unselected:
Commands
· open/on, open/off :selected:
o start/stop opening window in jog mode :unselected:
· close/on, close/off :selected:
o start/stop closing window in jog mode :unselected:
· fullopen
o open window completely :unselected:
· fullclose
o close window completely :unselected:
· moveToPosition/ ... :selected:
o starts movement to a defined position :unselected:
o 0 = fully closed :unselected:
o 100 = fully open :unselected:
· slightlyOpen/ :selected:
o move to partially open position :unselected:
· stop :selected:
o Stop movement :unselected:
o available since Miniserver 11.3 :unselected:
WindowMonitor Covered Config Items
16.0
Structure File
Page 156 of 174 :selected: :selected:
LOXONE Create Automation
Window- and Door Monitor
Details
₡ windows
¢ An array of objects containing information on a window or a door monitored by this block.
¢ The index of the object inside this array will correspond to the position of the state for this door or window in the windowStates array.
¢ An object contains a name for the door or window, the uuid of the room it is in and the installation place
uuid the uuid of the control which may be visualized to navigate there
· Fallback when control cannot be serialized
name the name of the door/window
room the uuid of the room
installPlace the installation place of the window/door
· New since V12.0, previously this was part of the name.
States
· windowStates
¢ Will return a string containing the states of each of the windows and doors monitored by this block.
¢ The individual states are separated by a comma. The position of the state in this string corresponds to the position of the object this state is for inside the windows-array in the details.
¢ Each state is a integer value that represents a bitmask where the individual bits correspond to the following states:
none -> state unknown / sensor offline
# 1 - closed
2 - tilted
. 4- open
8 - locked
16 - unlocked
· numOpen, numClosed, numTilted, numOffline, numLocked, numUnlocked
16.0
Structure File
Page 157 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
¢ The number of windows & doors in the corresponding states.
¢ The sum of the values from all these states is equal to the number of windows & doors monitored.
¢ The windows/doors with two states will always be counted to the "worst" state.
e.g .: A lockable door is unlocked and closed. It will be counted to numUnlocked and not to numClosed.
PowerUnit
Covered Config Items
¢ Power Supply & Backup
Details
₡ Outputs
¢ An array of the configured names for each Output / fuse ¢ provided name with output number if used.
¢ Empty strings if unused (=no name specified)
¢ output numbers starting with 1, not 0!
¢ e.g. [ "East (1)", "", "West (3)", "North (4)", "South (5)", "Outside (6)", "" ] ¥ supplyMode
¢ configured supply-mode
¢ 0: use as power-supply and backup
¢ 1: use as power-supply only
€ meterRefs
¢ maps between meter-id and subcontrol
¢ id 0: total energy meter
¢ id 1-7: meters for CP1-CP7
States
· outputPower
¢ Total output power of the Device (kW)
· CP1 - CP7 ¢ Power of Outputs 1 - 7 (kW)
· fuse
16.0
Structure File
Page 158 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
¢ Bitmask for all fallen Fuses ¢ Bit 1: Fuse for Output 1, ...
· supplyTimeRemaining
¢ Unix-Timestamp: end of supply time in Backupmode
¢ 0 if unknown / not applicable
· batteryStateOfCharge ¢ SOC of attached Battery 0 - 100 (%)
· devicelnfo
¢ Bitmask for Additional Device-States
¢ Bit 1: Backup-Mode active
¢ Bit 2: Overcurrent detected
¢ Bit 3: Battery Missing
¢ Bit 4: Battery Empty
¢ Bit 5: Battery-Test active
¢ Bit 6: Battery-Test finished
¢ Bit 7: Battery needs service
¢ Bit 8: Battery defective
¢ Bit 9: Battery Test OK
¢ Bit 10: Device is overheating
Subcontrol
· Tracker
Wallbox2
Covered Config Items
₡ Wallbox Gen. 2
Details :selected: . Details for Meter
o see <Meter> :unselected:
· min :selected:
o Minimum allowed charging Power :unselected:
. max
o Maximum allowed charging Power :unselected:
· modes
16.0
Structure File
Page 159 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o configurable charging modes
o JSON-array :unselected:
o fields per object:
" id: connector-id
name: configured name, empty: unconfigured
value: configured charging limit [kWh]
setByLogic: limit is defined by logic
· price: Price per kWh
· Since V14.5
· connectedInputs
o Bitmask of inputs which are used by logic or referenced device :unselected:
Bit 0: Allow charging (Ec)
Bit 1: Vehicle Connected (Vc)
Bit 2: Power connected
Bit 3: Meter Reading connected
Bit 4: Charge Active Information
· manager
o Uuid of Wallbox Energy Manager :unselected:
o Since V14.5 :unselected:
. ocpp
o Optional, object when used with OCPP Plugin, missing if not.
o "name" - Name of the OCPP Plugin :unselected:
o Since 15.1 :unselected:
States
· Details for Meter
o see <Meter>
o using actual + total
· connected
o vehicle is connected
· enabled
o charging is allowed / enabled
· active
o vehicle is currently charging :unselected:
· loadshed
o loadshedding is active :unselected:
· phaseSwitching
o charging is paused as phase switching is active, will resume when done :unselected:
· mode
o limitation mode <int see Modes> :unselected:
o additional manual mode: :unselected:
identified by id 99
cannot be renamed, but the limit can be changed
16.0
Structure File
Page 160 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
· limit
o limitation value [kw]
· modeUnplug
o limitation mode after unplugging vehicle
o cannot be the manual mode.
o 0 = keep previous mode
· session:
o JSON-object for current charging session
o fields
connect: unix timestamp of vehicle connection
disconnect: unix timestamp if vehicle disconnection
start: unix timestamp of last enabling of charge
energy: total energy charged in this session
" power: current charging power
" user: userID for current charging session
price: Price of the session
· since V14.5
· Since 15.1, -1337 = no price identifier extAuthority
· optional, available since 15.2
· e.g. Name of OCPP Plugin that approved a charging session
· priority
o Flag if requested priority (When used in Wallbox Energy Manager)
o since V14.5
· pricePerHour
o Price per connected hour
o since V14.5
o Since 15.1, -1337 = no price identifier
· pricePerkWh
o Current price per kWh
o since V14.5
o Since 15.1, -1337 = no price identifier
· [limitedBy]
o Limitation type, see limitation flags on Wallbox Manager
o Only when managed by Wallbox Manager
o since V14.5
Subcontrol
. Tracker - see <Tracker>
o History of last charging sessions
o Each line is a JSON-Object of a single session
16.0
Structure File
Page 161 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected:
LOXONE Create Automation
o Fields: see <session> state :unselected:
Commands
· allow/off
o disable charging :unselected:
· allow/on
o enable charging :unselected:
· setmode/<value>
o set limitation mode :unselected:
o allowed range: 1-5 + 99 (manual) :unselected:
o When manual mode is selected (99), it will reuse the previous charging limit. :unselected:
· modeUnplug/<value>
o set charging mode when disconnecting vehicle :unselected:
o 0 = keep current Mode :unselected:
o 1-5: switch to charging mode 1-5 :unselected:
o Must not be 99 (Manual Mode) :unselected:
· updateMode/<id>/<limitValue>/<name>/<price>
o <name> -> the name, but uriEncoded :unselected:
o <price> -> optional, since V14.5 :unselected:
· manualLimit/<value>
o sets the manual charging limit :unselected:
o if not already in manual mode, it will activate the manual mode (99) :unselected:
WallboxManager
Covered Config Items
- Wallbox Manager
Details
· nodes
o array of nodes to be shown in this manager, see separate section below for :unselected:
more infos
A node provides information about a wallbox or strand that is to be shown in the Wallbox Manager. Nodes can be built up in a tree hierarchy, where one node could contain other nodes. A node can either be a Wallbox itself, or a collection of nodes.
· Properties (properties in [brackets] are optional)
o uuid - identifies this node in the WBEM, used in getNodeState-Command :unselected:
16.0
Structure File
Page 162 of 174 :selected: :selected: :selected: :selected: :selected: :selected:
LOXONE Create Automation
o node Type -> identifies the type of node, the following values exist
Wallbox
Group
o title -> name to show on the node.
o icon -> the source path of the icon to use for this node
o [fuse]
Fuse power when the node is a group
o [actualState]
name of the state-property of the WBM representing the live value for this node.
only available for root-level nodes, child nodes need to use the getNodeState-command with their uuid to request state-values.
o [limitedByState]
name of the satte property of the WBM representing the limitedby value for this node
only available for root-level nodes, child nodes need to use the getNodeState-command with their uuid to request state-values.
o [ctrlUuid]
uuid of the control represented by this node. Used to request statistics or the states.
If the ctrlUuid is missing, it means that the statistics for this node need to be collected from its child nodes.
o [nodes]
List of child-nodes represented by this node.
States
· priceEco
· pricePrio
· priceHour
· Peco
· Pmax
· Cp
o Current Power, power actually used by the wallboxes
· Ap
o Assigned Power, power assigned to the wallboxes to be used.
· actualO ... actualN
o for each top node of the Manager there is a state of the current assigned power
16.0
Structure File
Page 163 of 174 :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
o returns the assigned power (not the actually used power)
o the "actualState" property of the nodes provide info on what state-property delivers the state for that node.
· limitedByO ... limitedByN
o for each top node of the Manager there is a state of the current limited by bitmask
o returns the limitedByMask (see limitedBy below for bits)
o the "limitedByState" property of the nodes provide info on what state-property delivers the state for that node.
Commands
· getNodeValue/{nodeUuidList}
o {nodeUuidList}
o List of nodeUuids separated by a semicolon "," or comma ','
returns a JSON object, where the nodeUuid is the key and the value is a JSON with the output values for the node requested
· { "node1uuid": { "fuse": 1 }, "node2uuid": {"assigned": 2,"used":1.5}}
· {"node1uuid": {"assignedl": 0, limitedBy: 1}}
· [fuse]
· when node is a strand the maximum fuse power
· assigned
· Assigned power
used
· actual used power
· Combined usedpower when its a strand
" [limitedBy]
. When there is a limit on a wallbox
· Reason why there is a charging limit
· Bitmask
· Bits:
o none = 0x00
No limit
o eco = 0x01
Peco o strand = 0x02
This strand
16.0
Structure File
Page 164 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected:
LOXONE Create Automation
o parentStrand = 0x04 :unselected:
Maximum of a parent (must not be the direct parent)
o Pmax = 0x08 :unselected:
General Pmax :selected:
= [demand]
. Boolean flag that is set when Wallbox has a demand (connected :selected:
and charging activated
= [priority]
· Boolean flag when Wallbox has a priority demand :selected:
ACControl
Covered Config Items
AC Control
Details
· type :selected:
o Configured AC Type :unselected:
· modes
o Json-array with used modes :unselected:
· airflow :selected:
o Json-array with used airflow-modes :unselected:
· fanspeed :selected:
o Json-array with used fanspeeds :unselected:
o since 15.1 :unselected:
Off mode :selected:
· connectedInputs :selected:
o Bitmask of Connected Inputs :unselected:
o Bit1: toggle (1 << 0- 0b0001) :unselected:
o Bit2: On (1 << 1 - 0b0010) :unselected:
o Bit3: Off (1 << 2 - 0b0100) :unselected:
o Bit4: ot :unselected:
o Bit5: 0c :unselected:
(1 << 3 - 0b1000) ( ... )
o Bit6: Mode :unselected:
o Bit7: Fan :unselected:
o Bit8: ADir :unselected:
o Bit9: Door/Window contact :unselected:
16.0
Structure File
Page 165 of 174 :selected: :selected: :selected: :selected:
LOXONE Create Automation
o Bit10: Pause Timer
o Bit11: Load Shedding :unselected:
o Bit12: Reset to Default :unselected:
o Bit13: Silent Mode :unselected:
· connectedParameters
o Bitmask of connected Parameters :unselected:
o Bit2: externallyControlled :unselected:
· externalControlled
o since 15.1 :unselected:
o true if AC is externally controlled by an IRC :unselected:
· UuidExternal
o since 15.1 :unselected:
o Optional; Uuid of connected IRC if configured :unselected:
. UuidClimate
o since 15.1 :unselected:
o Optional; uuid of configured AC Climate Controller if configured :unselected:
States
· status
o Power-Status(On/Off) :unselected:
· mode
o Active mode (1-5) :unselected:
· fan
o Active fanspeed(1-6) :unselected:
· defaultFan
o Fan speed set when control is powered off :unselected:
o since 15.1 :unselected:
· ventMode
o Active airflow mode (1-7) :unselected:
· operatingmodes
o Json-array with configured mode names :unselected:
· fanspeeds
o Json-array with configured fanspeed names
· airflows
o Json-array with configured mode names :unselected:
· pauseUntil
o When unequal 0 a sleep timer is active :unselected:
o Equal -1 when sleep input is constant on :unselected:
o UTC Unix-Seconds Timestamp :unselected:
· pauseReason
o Bitmask reason why the AC is paused (Off) :unselected:
o When equal 0 block is not paused or Off intentionally (only set when :unselected: desired state is active)
o pause_timer = 0x01 :unselected:
16.0
Structure File
Page 166 of 174 :unselected: :unselected: :selected:
LOXONE Create Automation
Paused by timer
o pause_doorWindow = 0x02,
Paused by opened window/door
o pause_loadShed = 0x04
Paused by load shedding
o pause_silentMode = 0x08
Since V15.2
Paused by Silent Mode
o pause_prohibitedHeat = 0x10
Since V15.1
Paused because desired heating mode is prohibited by Ac Climate Controller
o pause_prohibitedCool= 0x20
Since V15.1
Paused because desired cooling mode is prohibited by Ac Climate Controller :selected:
■
o pause_AccDiffmode = 0x40
Since V15.1
The Ac-Climate controller (outdoor unit) is currently in a different heating/cooling mode
o pause_IrcDiffmode = 0x80
Since V15.6
The connected Intelligent Roomcontroller is currently in a different heating/cooling mode
· pauseTime
o Value of pause time parameter (Smt)
o Seconds
· temperature
o Current Room Temperature
¢ targetTemperature
o Target Temperature for the AC
¢ silentMode
o Since V15.1
¢ Override
o JSON -serialized status of override-timer (see command startoverride)
o Available JSON-keys
¢ mode: active mode
¢ fanspeed: active fanspeed
airflow: active airflow (vent-mode)
¢ target: target-temperature
until: unix timestamp of end of this override timer
value 0: no override timer is active
16.0
Structure File
Page 167 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :unselected: :unselected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected: :selected: :selected: :selected:
LOXONE Create Automation
¢ settings
o Since V15.2 :unselected:
o Settings json which can also be retrieved via "/getsettings" :unselected:
· minTemp
o Since V15.3 :unselected:
o minimal possible value for target temperature :unselected:
· maxTemp
o Since V15.3 :unselected:
o maximum possible value for target temperature :unselected:
Commands
· toggle
o Toggle on/off :unselected:
· on
o Set power on :unselected:
· off
o Set power off :unselected:
· setTarget/<temp>
o Set target temperature :unselected:
· setMode/<mode>
o Set operating mode :unselected:
· setFan/<mode>
o Set fan speed mode :unselected:
· setAirDir/<mode>
o Set airflow mode :unselected:
· setmodename/<ID>/<name>
o Update name for operating mode :unselected:
· setairflowname/<ID>/<name>
o Update airflow name :unselected:
· setfanspeedname/<ID>/<name>
o Update fanspeed-name :unselected:
· setoverride/0
o since V15.1 :unselected:
o stops overridetimer :unselected:
· setoverride/<duration>/<mode>/<fanspeed>/<airflow>/<targettemp>
o since V15.1 :unselected:
o starts a override-timer with given parameters :unselected:
o duration: in seconds :unselected:
o other parameters: if 0, current setting is preserved :unselected:
· startpause/<duration>
o Start pause timer with the given duration :unselected:
o duration: in seconds :unselected:
· cancelpause
16.0
Structure File
Page 168 of 174 :selected: :selected: :selected: :selected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
o Stop pause timer
₡ setdefaultairflow/<ID>
o Update default airflow setting
o Since V15.1
o 0: Retain Last Value
o 1: Auto
O ...
₡ setdefaultfanspeed/<ID>
o Update default fanspeed setting
o Since V15.1
o 0: Retain Last value :unselected:
o 1: Auto
o ... :unselected:
· setdefaulttargettemp/<temp>
o Update default target temperature setting
o Since V15.1 :unselected:
o -1: Retain Last Value :unselected:
· setsilentmode/<ID>
o Update silent mode setting
o Since V15.1
o 0: Off
o 1: Auto
o 2: ...
· SetAutomaticModeFanSpeed/<ID>
o Since V15.1
o Fan Speed Options
· GetSettings
¢ Formats the settings in JSON
Since V15.1
€ Example:
{"SilentMode":2,"DefaultAirflow":0,"DefaultFanspeed":0,"DefaultTargetTemp": -1.000}
· SilentMode: Default Value for Silent Mode
o 0 Off
· AutomaticModeFanspeed: Fanspeed which is set when AC is controlled by IRC
o Fan Speed Options :unselected:
. DefaultAirflow: Default Value for Airflow
o -1: Retain Last Value
. DefaultFanspeed: Default Value for Fanspeed
o -1: Retain Last Value
. DefaultTargetTemp: Default Value for target temperature
o -1: Retain Last Value :unselected:
16.0
Structure File
Page 169 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected:
LOXONE
Create Automation
16.0
Structure File
Page 170 of 174
LOXONE Create Automation
Revision History
15.3 · LoadManager
o - added "mode" in details which specifies which working mode is used
o - added "currentpower2" state used in "Peak Overload" mode for current power
· Window - Added types
15.1
· Wallbox2 - added "ocpp" in details when OCPP is used.
o When available, prices with value "-1337.0" will be hidden in App
o Name property is used to specify exter
o Affected: pricePerkWh, pricePerHour, session-price
· AC Control - Added Default Values, Silent Mode, IRC Integration
15.0
· NFC Code Touch - new state "keyPadAuth Type" for authentification parameter
· Audio - bluetooth states and commands
14.7
· MsShortcut Support
· Door and Windowsmonitor - Support for nesting of controls
· Status Monitor Support
14.5
· Wallbox Manager Support
· Wallbox - New states for prices and Manager support
14.4
· Climate Controller - Option to turn it off
. Control History Support
· Statistic Option 15 minute interval
14.2
· Intelligent Room Controller - fixed setpoints support added
. ACControl - Pause Timer, Load Shedding added
. Garage Door - Partially Open Support added
16.0
Structure File
Page 171 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :selected:
LOXONE Create Automation
. Spot PriceOptimizer
14.1
· ACControl
· WallboxV2 support introduced
· Statistics Download via HTTP support added
14.0
· AudioZoneV2 - added volume state
· ClimateController - connected inputs/parameters info added
· AudioZoneV2 - connected parameter info added
· msInfo - now provides info on MS hostname
· EFM - Added custom names/icons for rest (untracked)
· Radiobuttons - prev+next commands
13.2
· NFC Code Touch - KeypadAuthTypes added + additional code types
13.1
· Energy Flow Monitor
· StatisticV2
· Meter
o Adopted properties to represent new meter types
. EnergyManager2
13.0
· Power Supply and Backup
· Alarm
o nextLevelAt with unix-Timestamp, nextLevelDelay deprecated
o armedAt with unix-Timestamp, armedDelay and armedDelayTotal deprecated
· Audio Zone
o volumeModes - provides info on external volume control (none, external absolute value, external up/down pulses)
· Climate Controller
o standbyReason - provides info on why it is not be heating or cooling :unselected:
. ColorPickerV2
o Support for daylight sequences
o Support for tunable white only
16.0
Structure File
Page 172 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :selected: :selected: :unselected:
LOXONE Create Automation
· IRCv2
o New possible capability for passive cooling
· Intercom
o showBellImage detail specifying whether to show the image or stream when bell is rang
· IntercomV2
o Support for trust.
· NFC Code Touch
o Attribute provides insight on whether a code has been provided or not.
· LightControllerV2
o supports short names for lighting moods
o support for human centered lighting.
· Pushbutton
o provides insight on which type of pushbutton it represents (e.g. scene)
12.2
· Sequential Controller support
· Support for new Intercom (IntercomV2)
12.1
· New US HVAC block
. New features in IRCv2
o Additional cooling comfort temperature
o Presence-dependent daytimer entries
· Jalousie - Support for end position adjustment (expert mode only)
· New Load Manager block
· API for Energy Manager
· New Irrigation block
· API for Pulse At
· Geo-Location provided in msInfo (latitude, longitude and altitude)
12.0
· Reset controls to predefined presets
· Locking and Unlocking Controls via API
. Controls may now have other controls linked to them
· LightControlV2 - circuit names may now be modified via API
· IRCv2 - added support for temperature limits
· ClimateController - informs on excess energy available
16.0
Structure File
Page 173 of 174 :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :unselected: :selected: :selected: :unselected: :selected: :selected: :selected: :selected:
LOXONE
Create Automation
16.0
Structure File
Page 174 of 174

---
## Extracted Tables

### Table (17x2)
Table of contents | 2
--- | ---
General Info | 8
autopilot. | 13
mediaServer | 13
Loxone Audioserver | 14
Miniserver Compact | 14
Loxone Music Server | 14
messageCenter | 14
Controls | 15
lastModified | 8
msInfo | 8
globalStates | 10
rooms | 11
cats | 12
weatherServer | 12
times | 13
caller | 13

### Table (28x2)
Mandatory fields | 15
--- | ---
Optional fields | 15
Structure | 20
Commands | 22
Binary Result | 25
Secured Details | 26
Commands | 26
Control History | 26
Commands | 26
Data Structure | 26
Trigger Types | 26
Control Types | 27
Locking and Unlocking Controls | 16
AalEmergency | 27
AalSmartAlarm | 28
Alarm | 29
AlarmChain | 31
AlarmClock | 32
Application | 36
AudioZone | 36
AudioZoneV2 | 41
Which controls can be (un)locked via API? | 17
How to know a control is locked and why ?. | 17
How to (un)lock a control via API? | 17
Statistic | 17
Commands | 19
BinaryFormat | 19
StatisticV2 | 20

### Table (28x2)
CarCharger | 44
--- | ---
BMW Wallbox specific | 46
Pool Daytimer | 64
Dimmer | 64
EnergyManager | 65
EnergyManager2 | 66
EnergyFlowMonitor | 69
Fronius | 72
Gate | 74
Heatmixer | 75
Hourcounter | 76
InfoOnlyAnalog | 77
Central Objects | 46
InfoOnlyDigital | 77
InfoOnlyText | 78
Intelligent Room Controller v2 | 79
Intelligent Room Controller | 85
Intercom | 89
IntercomV2 | 91
Irrigation | 93
Jalousie | 95
ClimateController. | 48
ClimateControllerUS | 52
ColorPickerV2 | 58
ColorPicker | 60
Daytimer | 61
Intelligent Room Controller Daytimer v2 | 63
Intelligent Room Controller Daytimer | 64

### Table (28x2)
NFC Code Touch | 98
--- | ---
LightController | 103
Radio | 123
PresenceDetector | 124
PulseAt | 125
Remote | 125
Sauna | 128
Sequential | 131
Slider | 131
SmokeAlarm | 132
SolarPumpController | 134
SpotPriceOptimizer | 135
LightControllerV2 | 104
StatusMonitor | 137
SteakThermo | 138
Switch | 143
SystemScheme | 144
TextState | 145
TextInput | 145
TimedSwitch | 145
Tracker | 146
LightsceneRGB | 109
LoadManager | 110
MailBox | 111
Meter | 112
MsShortcut | 114
PoolController | 115
Pushbutton | 122

### Table (27x2)
UpDownLeftRight digital | 147
--- | ---
UpDownLeftRight analog | 148
ACControl | 165
Revision History | 171
15.3 [ WIP ]. | 171
15.1 | 171
15.0 | 171
14.7 | 171
14.5 | 171
14.4 | 171
14.2 | 171
14.1 | 172
ValueSelector | 148
14.0 | 172
13.2 | 172
13.1 | 172
13.0 | 172
12.2 | 173
12.1 | 173
12.0 | 173
Ventilation | 149
Webpage | 155
Window | 155
WindowMonitor | 156
PowerUnit | 158
Wallbox2 | 159
WallboxManager | 162

### Table (8x2)
"uid": String, | // unique message id
--- | ---
"ts": Number, | // unix timestamp
"type": 10, | // type, 10 = normal message
"title": String, | // title
"message": String, | // message
"data": { | // additional data
“Lvl": Number, | // level, 1 = Info, 2 = Error, 3 = SystemError
["uuid": String] | // optional uuid (from Control, eg. Alarm)

### Table (3x4)
 | "title": | "Grid | Import",
--- | --- | --- | ---
"format": | "%.3 kWh",
"output": | "total",

### Table (14x3)
:selected: | Bit 1 | Comfort-Temperature Heating
--- | --- | ---
:selected: | Bit 2 | Comfort-Temperature Cooling
 | Bit 11 | HeatProtect Temperature
 | Bit 12 | Mode input
 | Bit 13 | CO2-Level
 | Bit 14 | Indoor Humidity
 | Bit 3 | Comfort Temperature Heat+Cooling
 | Bit 4 | Allowed Comfort-Tolerance
■ | Bit 5 | Lower absent Temperature
 | Bit 6 | Upper absent Temperature
■ | Bit 7 | Allowe deviation absent
 | Bit 8 | Shading temperature heating
 | Bit 9 | Shading temperature cooling
■ | Bit 10 | FFrostprotect Temperature

### Table (5x2)
name
:selected: | text of the status
--- | ---
· prio | priority of the state (starting at 0)
@ id
:selected: | output index of the state (starting at 0)
color
:selected: | color of the status text
uuid
:selected: | the uuid of the state itself
