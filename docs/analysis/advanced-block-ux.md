# Advanced block configuration and UX coverage

This investigation covers configuration authoring, validation, and feedback,
not upload fidelity. The baseline is Loxone Config **17.2.10.21**, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
Qt **6.10.2** is the revision-13 decoder's format reference, not an independently
verified embedded-library build/version.
Recovered code, resource payloads, and full reports remain local and Git-ignored.
The inventory does not access or modify a Miniserver.

## Reproduce the inventory

After completing the bulk analysis described in
[the reverse-engineering guide](../guides/reverse-engineering.md), build the
current CLI and run:

```powershell
cargo build --release
python .\scripts\inspect-loxone-ux.py --rea-root E:\work\rea
```

The command reads `.reverse-engineering\latest.json`. `--state` selects another
completed analysis; `--output` selects the local report directory; `--lox`
selects another compatible CLI. Omitting `--rea-root` skips PE dialog resources
and records that part as uninspected. No task, watcher, or schedule is created.
Exit 1 is an inventory failure; exit 2 retains a partial inventory with explicit
fixture omissions, unsupported Qt objects, or decoding issues.

New extractions also require reviewed per-object Qt header-layout profiles via
`--qt-layout-profiles FILE`. The profile JSON declares `source_sha256`,
`format: "PE32+/AMD64"`, `metadata_revision: 13`, and an `objects` list. Each
object binds a `pointer_triple_address` to either `qt6-superdata-one-word` or
`qt6-superdata-two-word`, plus an independently reviewed `base_return_address`
and `base_return_sha256`. The witness is an eight-byte AMD64 static
`LEA RAX, [RIP+disp32]; RET` sequence returning the object base. Its bytes and
the header shape are checked against the pinned source; the witness does not
establish a function entry, receiver type or runtime behavior.

Missing or conflicting profiles remain explicit decoding issues without a
guessed object base. Revision alone does not select a header width. Seven
reviewed entries demonstrated that the historical scanner's
`metaobject_address` was eight bytes after the code-returned base; their
absolute strings/data/dispatcher pointers remain correct. At that stage, the
other 638 entries had not received that assessment. Do not blanket-shift the frozen
645-record snapshot or its address-derived class IDs.

Use a new `--output` directory for normalized extractions: the inventory refuses
to overwrite an existing `qt-metadata.json`. Corrected addresses produce new
snapshot identities, not an in-place migration of historical evidence or an
increase in candidate/runtime coverage.

### Reviewed eight-dialog extension

A later, independently reviewed cohort adds **eight header layouts and 72
class-local metadata methods** (58 distinct method names). These classes are
disjoint from the original seven:

| Qt metadata class | Local methods |
|---|---:|
| `CEnergyManagerConfigurationDlg` | 5 |
| `CPresenceDetectorConfigurationDlg` | 16 |
| `CTargetBrightnessConfigurationDlg` | 13 |
| `CNanoMotorControllerConfigurationDlg` | 15 |
| `CLoadManagerConfigurationDlg` | 3 |
| `CTouchFlexConfigurationDlg` | 8 |
| `CWallboxManagerConfigurationDlg` | 3 |
| `CTouchPureDisplayConfigurationDlg` | 9 |

Parent aggregation through the unchanged, source-pinned decoder produces a
**separate 15-layout, 93-class-local-method snapshot**, leaving **630/645 layouts
unvalidated**. Every reviewed class/method record matches its original cohort.
Method names shared across classes are not deduplicated as implementations.
The frozen 645-row corpus and original seven-class snapshot remain unchanged.

The eight-class witness supplement SHA-256 is
`be2fd3c6bd6225fa843c42cdb83a874ddb3c7837448764618b0811a638d7bec8`;
independent admission is
`84670348b24865d7c9e6300ef56441d228bdd122ebab4e7e86f267b4ac513b2e`.
The separate aggregate snapshot SHA-256 is
`906a3ee9d81ce2c9ef9299a03ce632667b39ba0f3c08cdbdb0cba79489865b2c`.
Its profiles are retained at
`.reverse-engineering\element-atlas\qt-reviewed-cohorts-20261107T1540Z\reviewed-fifteen-profiles.json`
(SHA-256 `067a25bdd784268447b6eb0bc80abd590906f48266d647ee9b4bd84f4ad7e491`).
Use that file with the existing `--qt-layout-profiles` option and a new output
directory; never overwrite a historical snapshot.

This admission covers revision-13 metadata tables and code-returned header-base
witnesses only. Five new callback bodies remain absent from the frozen canonical
index; their callable ABI and implementation remain unknown. All 72 new methods'
GUI bindings, implementation behavior and runtime connections remain unverified.
Dialog names do not prove factory/XML/control ownership, and no catalog-identity,
UX-rule or native-parity count is promoted. The initial future-dated milestone
was corrected append-only; its historical event time remains unknown.

For release comparisons, retain separate output directories with `--output`.
Compare control identities, parameter patterns, and Qt class/member signatures,
not native addresses, which can move between builds. A changed control identity
or unsupported metadata revision stays unknown rather than inheriting old
coverage.

| Local output under `.reverse-engineering\ux` | Contents |
|---|---|
| `advanced-blocks.tsv` | Every advanced candidate, its numeric identity, counts, and selection reasons |
| `block-ux-coverage.json` | All catalog records, parameter mappings, storage observations, native leads, and unverified parity dimensions |
| `qt-metadata.json` | Original declared Qt class/member/property names and shared dispatcher addresses |
| `native-dialogs.json` | Decoded Windows dialog templates, controls and string tables, bound to verified REA resource ranges |
| `english-resources.evidence.json` | Original REA PE-resource Evidence |

`lox config surfaces FILE -o json` independently inventories attribute names
and nested storage paths using `ConfigEditor`, including native digit-prefixed
attributes such as `12hTF`. It emits no titles, UUID values, parameter values,
passwords, or program text and does not change the file. Typed child elements
remain distinct, such as `C[Type=LightscenesC]`.

## Baseline findings

The Qt scanner counts below are historical observations, not the independently
reviewed 15-layout/93-method aggregate above.

| Measurement | Observed |
|---|---:|
| Catalog records examined | 522 |
| Explicit XML type identities | 220 |
| Advanced candidates | 124 |
| Advanced candidates with explicit XML types | 120 |
| Catalog parameter entries | 757 |
| Parameter entries associated with existing XML keys | 645 |
| Configuration fixtures read by the native parser | 36, with no omissions |
| Historical revision-13 Qt candidate rows | 645 |
| Historical declared Qt method records | 1,892 |
| Historical declared Qt property records | 283 |
| Qt dialog/editor/configuration name candidates | 214 |
| MSVC RTTI dialog/editor name leads | 539 |
| English Windows dialog templates decoded | 113 |
| Static controls in those templates | 1,494 |

These are different populations, **not UX coverage percentages**. A Qt property
may describe styling rather than block configuration. A template control may be
a label, an obsolete dialog, or one row of a dynamically populated editor.
An observed pattern expansion is not an exhaustive channel limit.

Selection includes types with at least eight parameter entries, at least
24 combined I/O entries, conditional metadata, repeated connector patterns,
observed nested storage, or a known specialized configuration family. This
intentionally also includes some simple repeatable arithmetic blocks as
structural candidates; inclusion alone does not establish a complicated GUI.
`TemplateId` is a documentation reference, not evidence of repeated UI rows.
All 522 records remain in the JSON so these heuristics do not hide other types.

Four advanced records have no explicit `LxType`: Air Analog Actuator (175),
Tree Analog Actuator (185), Central (319), and Zambelli (467). Their numeric
identities are preserved; no serialization type is invented.

## Parameter-heavy types

Counts are catalog parameter entries, excluding additional dialog settings:

| Entries | XML types |
|---:|---|
| 22 | `AutoJalousie` |
| 21 | `LightController2`, `MediaClient` |
| 20 | `HeatIRoomController2` |
| 17 | `LightController` |
| 16 | `ShadeRoof` |
| 15 | `Jalousiemotor`, `HVACController` |
| 14 | `Alarm`, `IRoomcontrol` |
| 13 | `FancoilFreshAir`, `ClimateControllerUS` |
| 12 | `MusicPlayer`, `LightControllerH`, `PresenceController` |
| 11 | `Fan` |
| 10 | `Door`, `Ventilation`, `Heatmixer`, `PID` |
| 9 | `SaunaVapor`, `Sauna`, `AlarmClock`, `Heatmixer2`, `VentInternorm`, `CentralFancoil`, `PI` |
| 8 | `TpfController`, `Solarpumpcontrol`, `Leaf`, `JalousieUpDown2`, `TabletDevice`, `PushDimmer` |

Lower counts do not imply simple configuration:

| Family | Additional surfaces to characterize |
|---|---|
| Lighting | Multi-output mood records, output roles, `HCL`, `PSD`, `LSConfig`, transitions and assignments. `LightController2` has 63 observed storage paths. |
| Energy / charging | `EnergyManager2` load records, ordering and per-load options; `EFM` meter/group topology; `WBEM` groups, charging modes and limits; `LoadShed` and `SpotOpt` configuration. |
| HVAC | Equipment assignments, mode-dependent setpoints, curves, schedules and interlocks across room, central, fan-coil, ventilation, pool and sauna controllers. |
| Shading | Calibration, slat geometry, orientation, automatic modes, locking and safety behavior. |
| Audio | Player/zone assignment, favorites, sources, announcements and mode-dependent volume. |
| Schedules | Weekday/operating-mode entries, exceptions, alarm records and actions. |
| Rules / programming | `StateTexts` conditions and results, sequence programs, expressions and status mappings. |
| Security / access | Zones, authorizations, actions, schedules, device associations and their dependencies. |
| Integrations / devices | Protocol configuration, command templates, assignments, mappings and device-specific editors. |

The complete candidate set, including cross-family types and generic
structural candidates, is in `advanced-blocks.tsv`.

## Additional recovery beyond bulk decompilation

The application is a **Qt/MFC hybrid**, not a pure Qt application.

The static Qt decoder uses the revision-13 layouts from
[Qt 6.10.2](https://github.com/qt/qtbase/tree/v6.10.2/src/corelib/kernel).
It recovers declared names, argument/type references, property flags, and the
shared `qt_static_metacall` dispatcher. Dispatcher addresses are joined to the
existing native export without changing its checksummed files. A dispatcher
can serve multiple declared methods: 1,892 method records do not mean 1,892
distinct implementation addresses have been identified.

Examples of actionable recovered handlers:

| Declared class | Dispatcher | Declared methods |
|---|---|---|
| `CEnergyManagerConfigurationDlg` | `0x140c084d0` | `OnModified`, `OnSortUp`, `OnSortDown`, `OnUpdateButtons` |
| `CEnergyFlowMonitorDlg` | `0x140d2daf0` | `OnCreateGroup`, `OnAddMeter` |
| `CWallboxManagerConfigurationDlg` | `0x140d2dcc0` | Context-menu and group-order handlers |
| `CAlarmClockDlg` | `0x140c086b0` | `OnAdd`, `OnDelete`, `OnDataEdited` |
| `CStateTextDlg` | `0x140c0b710` | Row editing, movement, deletion and reordering |
| `LxSequenceControl::CSequenceControlDlg` | `0x140c24fd0` | `CheckCode`, `OnChanged`, `ToggleSimu`, `StopSimu` |

REA also supports engine-free `inspect-pe-resources`. Its payload ranges and
hashes allow the Windows resource decoder to read the resource DLL without
loading or executing it. The English DLL contains 113 dialog templates,
including lighting templates with 148 and 211 static controls. Their presence
does not prove they are reachable in the current GUI or identify a block
version. The executable itself contains no `RT_DIALOG` payloads.

The same decoder reads the English DLL's 862 string-table resources and
preserves their numeric string IDs and empty slots. This provides additional
labels, help and diagnostic messages to correlate with recovered code, not
proof that a message represents a particular validation rule.

No loose `.qml`, `.ui` or `.js` files were found in the captured installation.
A bounded name probe of four embedded Qt resource registrars found two
distinct resource sets, with named CSS assets but no files named with those
three extensions. This is not a content-level assertion that all declarative
source is absent: resource aliases can omit extensions. Embedded resource names
and styles do not reconstruct the C++ configuration/validation implementation.

REA's Windows Ghidra operations additionally support recovered types, call-site
resolution, references and bounded p-code value tracing. They can help follow
these dispatchers into serializers and validators. Its native accessibility
and live-call observation workflows are Mach-O/macOS-scoped, not Windows GUI
automation. Ordinary Windows UI Automation can inspect some controls, but
custom views can remain inaccessible.

External tools checked:

| Project | Useful capability | Qualification |
|---|---|---|
| [QtREAnalyzer](https://github.com/diommsantos/QtREAnalyzer) | Ghidra annotations for Qt classes, methods and properties | The inspected implementation uses Qt 5's `QByteArrayData` string carrier, not this binary's Qt 6 offsets-and-sizes carrier. It was not applied blindly. |
| [QtRE](https://github.com/OSUSecLab/QtRE) | Research algorithms for Qt metadata and signal/slot connection recovery | Its documented tested Ghidra version is 9.2.2. Compatibility with Ghidra 12.1.4 and Qt 6.10.2 was not established. |
| [GammaRay](https://github.com/KDAB/GammaRay) | Live QObject/model/property/signal inspection | A promising offline runtime oracle, but requires a compatible probe for the target Qt/compiler/architecture. No probe was attached in this investigation. |
| [Inside The Loxone Miniserver](https://github.com/sarnau/Inside-The-Loxone-Miniserver) | Protocol and hardware documentation | Complementary to editor research; not original Loxone Config source. |

Neither metadata recovery nor runtime introspection reconstructs original
comments, all C++ types, or code that exists only in another component.

## What counts as complete UX parity

For a specific release and block, parity requires identified and verified
configuration fields, nested-list CRUD/order/reference behavior, defaults,
units/ranges/enums, sentinel values, conditional settings, cross-field rules,
and diagnostic/feedback behavior. GUI save/reopen round trips must preserve
intent and unrelated data; regenerated UUIDs and formatting require semantic
comparison rather than byte equality.

Resource metadata, native literals, dialog names and fixture shapes are
navigation evidence, not proof of those behaviors. The report therefore leaves
parameter-domain, cross-field validation, feedback and GUI-roundtrip parity
**unverified**, with an unknown total GUI-field denominator. It does not claim
100% coverage.

An official lighting sample was opened as a disposable offline copy and its
UI inspected. This was not a parameter-by-parameter roundtrip. No live server
configuration, scheduled alarm, or alarm dependency was changed. Production
uploads are unnecessary for this inventory and should not be used as a
substitute for offline configuration tests.
