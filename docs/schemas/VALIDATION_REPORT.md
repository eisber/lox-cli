# Validation Report — All 163 Block Types

| Metric | Value |
|--------|-------|
| Total types | 163 |
| Structural validated (CLI injection) | 163/163 ✅ |
| Behavioral validated | 8 |
| Source | crosswalk.json + validate_blocks.py |

### Logic (4 types, 11 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| And | I1, I2, Q | ✅ | 14/14 PASS |
| Or | I1, I2, Q | ✅ | 14/14 |
| Not | I, Q | ✅ | 14/14 |
| Xor | I1, I2, Q | ✅ | 14/14 |

### Math (10 types, 35 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| Add2 | I1, I2, Q | ✅ | 6/6 PASS |
| Add4 | I1, I2, I3, I4, Q | ✅ |  |
| Subtract | I1, I2, Q | ✅ | 6/6 |
| Multiply | I1, I2, Q | ✅ | 6/6 |
| Divide | I1, I2, Q, Qr | ✅ | 6/6 |
| Modulo | I1, I2, Q, Qr | ✅ |  |
| Formula | I1, Q | ✅ |  |
| Average | I1, Q | ✅ |  |
| MovingAverage | I, Max, Min, PT, Q, R | ✅ |  |
| Integer | I, Q, Qr | ✅ |  |

### Encoding (5 types, 16 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| BinaryDecoder | I, Q | ✅ |  |
| BinaryDecoder2 | I, Q | ✅ |  |
| BinaryEncoder | I, Q | ✅ |  |
| AnalogMultiplexer2 | I1, I2, Q, S | ✅ |  |
| AnalogMultiplexer4 | I1, I2, I3, I4, Q, S | ✅ |  |

### Comparison (8 types, 39 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| Comparator | EQ, GT, I1, I2, LT | ✅ |  |
| ThresholdSwitch | AQ, I, Off, On, Q | ✅ |  |
| DiffThresholdSwitch | I1, I2, Off, On, Q | ✅ |  |
| MinMax | I, Max, Min, Q | ✅ |  |
| MinMaxReset | I, Max, Min, Q, R | ✅ |  |
| AnalogLimiter | I, Max, Min, Q | ✅ |  |
| AnalogWatchdog | AQ, I, Q | ✅ |  |
| ValueValidator | AQ, En, I, Max, Min, Q +2 | ✅ |  |

### Stateful (11 types, 48 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| Memory | AQ, Input, Q | ✅ |  |
| MemoryFlags | I | ✅ |  |
| FlipFlopRS | Q, Qn, R, S, Trg | ✅ |  |
| FlipFlopSR | Q, Qn, R, S, Trg | ✅ |  |
| Counter | I, Q, Qmax, Qmin, R | ✅ |  |
| UpDownCounter | Dn, Q, Qmax, Qmin, R, Up | ✅ |  |
| Stepper | I, Q, Qmax, Qmin, R | ✅ |  |
| ShiftRegister | Clk, I, Q, Qn, R | ✅ |  |
| EdgeDetection | Chg, I, Neg, Pos, Q | ✅ |  |
| Status | AQ, I, Q | ✅ |  |
| StatusMonitor | AQ, En, I, Q, R | ✅ |  |

### Timing (12 types, 46 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| Monoflop | I, PT, Q, R | ✅ |  |
| PulseGenerator | I, PT, Q, R | ✅ |  |
| PulseAt | I, PT, Q, R | ✅ |  |
| PulseBy | I, Q | ✅ |  |
| DelayedPulse | I, PT, Q | ✅ |  |
| SwitchOnDelay | I, PT, Q, R | ✅ |  |
| SwitchOnOffDelay | I, PToff, PTon, Q, R | ✅ |  |
| SavingSwitchOnDelay | I, PT, Q, R | ✅ |  |
| PWM | I, PT, Q | ✅ |  |
| WipingRelay | I, Q, QT, R | ✅ |  |
| RandomGenerator | I, Min, Q, R | ✅ |  |
| RandomController | I, Max, Min, Q, R | ✅ |  |

### Other (113 types, 1415 connectors)
| Type | Connectors | Structural | Behavioral |
|------|-----------|-----------|-----------|
| ACCentral | API, C, Ec, Eh, H, Mode +5 | ✅ |  |
| ACControl | API, Hys, O, Ptd, Status, maxT +1 | ✅ |  |
| AalSmartAlarm | A, A1-2, API, Dis, Off, Tb +2 | ✅ |  |
| AccessController | Dsel, Eid, Off, P, Pd, Sel | ✅ |  |
| Alarm | A, API, Ad, Adnp, Anp, Aoc +21 | ✅ |  |
| AlarmChain | A, A1-10, AEs, As, Au, Ca +4 | ✅ |  |
| AlarmClock | AQs, Acknowledge, AlarmTime, BrightAct, BrightInact, Deactivate +20 | ✅ |  |
| AudioCentral | AIs, API, Alarm, Bell, Buzzer, Cs +20 | ✅ |  |
| AudioPlayer | 2C, 3C, API, Alarm, BTp, Bell +31 | ✅ |  |
| AudioPlayerFixedGroup | API, Alarm, BTp, Bell, Cs, DisP +12 | ✅ |  |
| AutoPilot | Input, output | ✅ |  |
| CallGenerator | API, Dk0-9, Td, Tr, Tu, Uid +1 | ✅ |  |
| CentralJalousie | API, Blk, Cc, Co, Dir, DisPc +29 | ✅ |  |
| ClimateController | API, Ah, B, C2, Dfc, Doff +23 | ✅ |  |
| CombinedWindowContact | Open, S, Secured, Tilt | ✅ |  |
| ComfortSwitch | API, DisPc, Off, On, Rem, Tg +2 | ✅ |  |
| CommandRecognition | Lv, T | ✅ |  |
| DaylightControl | API, Act, Hys, Lc, Off, Sts | ✅ |  |
| DewpointCalculator | H, O, ϑ, ϑd | ✅ |  |
| Dimmer | +, -, API, D, Di, DisPc +7 | ✅ |  |
| DoorController | API, Bell, O1, O2, O3, Off +1 | ✅ |  |
| DoorWindowMonitor | API, Dwco, Dwcs, Dwct, Hpos, Rem +2 | ✅ |  |
| EIBDimmer | API, Cdv, D, DisPc, Off, On +3 | ✅ |  |
| EIBJalousie | API, DisPc, Lo, Pos, So, Tlc | ✅ |  |
| EIBPushbutton | API, DisPc, O, Off, On, Rem +2 | ✅ |  |
| EmergencyAlarm | A, API, Ca, Cc, Off, Ta +2 | ✅ |  |
| EnergyFlowMonitor | API, Co2d, Off, Pre, Pri, Rest +1 | ✅ |  |
| EnergyManager | API, O, Off, Prio, Ps, Re | ✅ |  |
| EnergyManager2 | API, Gpwr, L1-12, MaxSpwr, MinSoc, Next +6 | ✅ |  |
| EnergyMonitor | API, Abs, Gpwr, Ppwr, Ptot, R +2 | ✅ |  |
| EventDBConnector | API, CI1, CI10, CI11, CI12, CI13 +15 | ✅ |  |
| FireWaterAlarm | API, Ca, Cs, F, Mad, MaxA +9 | ✅ |  |
| FixedValueMeter | API, Mro, R, S | ✅ |  |
| FlowTemperature | API, B, G, I, Ib, Max +8 | ✅ |  |
| GateCentral | API, Cc, Co, DisPc, Nc, No +4 | ✅ |  |
| GateController | API, Cc, Co, DisPc, Ic, Io +14 | ✅ |  |
| HVACController | B, E, Ec, Eh, Emh, Fan +10 | ✅ |  |
| HeatingCurve | Ct, Dis, Iv, O, S, maxFt +1 | ✅ |  |
| HotelLightController | AIr, AQ1-20, AQrm, AQs, Dis, DisMo +23 | ✅ |  |
| IRController | I1-6 | ✅ |  |
| IRoomControllerV2 | ADir, Bp, C, CO2, DisP, Dwc +9 | ✅ |  |
| IntercomV2 | API, Bbl, Bbr, Bell, Mute, O1 +5 | ✅ |  |
| InternormVentilation | API, B, Bva, Bvp, CO2max, Dwc +13 | ✅ |  |
| Irrigation | API, Act, Av, MaxR, MaxRa, Off +6 | ✅ |  |
| Jalousie | API, AQpp, Bld, Bldo, Cc, Cl +35 | ✅ |  |
| JalousieCentral | API, Cc, Co, DisPc, DisSp, Nc +10 | ✅ |  |
| LeafVentilation | API, B, Bva, Bvp, CO2max, Dwc +13 | ✅ |  |
| LightCentral | API, Alarm, Buzzer, DisP, DisPc, Lc1-4 +7 | ✅ |  |
| LightController2 | AQ1, Alarm, AlarmClock, AlarmClockPeriod, AlarmPeriod, Brightness +35 | ✅ |  |
| LightsceneRGB | +, -, AIs, AQa, AQb, AQg +6 | ✅ |  |
| LoadManager | API, Ap, AvgP, Gi, Gpwr, Hys +8 | ✅ |  |
| LongClick | O1, O2, O3, O4, R, Rem +2 | ✅ |  |
| MailBox | API, Cm, Cp, M, P, Txle | ✅ |  |
| MailGenerator | API, Td, Tr, Tu, Uid, V1-8 | ✅ |  |
| MediaController | API, Ch, DisPc, M1-8, Mode, Nst +9 | ✅ |  |
| Meter | API, Mr, Mro, Pf, R | ✅ |  |
| MeterBidirectional | API, Mrc, Mrd, Mroc, Mrod, Pf +1 | ✅ |  |
| MeterStorage | API, Mrc, Mrd, Mroc, Mrod, Pf +1 | ✅ |  |
| MixingValve | API, DisPc, Error, Inv, Ki%, Kp% +6 | ✅ |  |
| MultiClick | 1C, 2C, 3C, 4C, Off, Rem +3 | ✅ |  |
| MusicServerZone | A, AIs, AIv, API, AQr, Be +40 | ✅ |  |
| NFCCodeTouch | API, Ad, As, Au, Bbl, Bbr +11 | ✅ |  |
| PIController | Auto, CO, Max, Min, Mv, Off +5 | ✅ |  |
| PIDController | Auto, CO, Max, Min, Mv, Off +5 | ✅ |  |
| PVForecast | API, Off, Period, Pnd, Pp, Ppwr +2 | ✅ |  |
| Ping | N, Off, Online, Pi, Td | ✅ |  |
| PoolController | API, Bw, Bwd, C, Ci, Cid +27 | ✅ |  |
| PositionController2 | Hys, Inv, O, Off, PV, SP | ✅ |  |
| PositionController3 | O1, O2, Off, PV, SP1, SP2 | ✅ |  |
| PowerSupplyBackup | API, Bm, Ol, P1-7, Pt, Soc | ✅ |  |
| PresenceDetector | ControlInput, DeviceActivate, DeviceExtend, DeviceTrigger, InputActivate, InputExtend +13 | ✅ |  |
| PulseMeter | API, F, Mro, P, Pf, R | ✅ |  |
| PulseMeterBidirectional | API, Fc, Fd, Mroc, Mrod, Pc +3 | ✅ |  |
| PulseMeterStorage | API, Fc, Fd, Mroc, Mrod, Pc +3 | ✅ |  |
| PushSwitch | API, DisPc, Don, O, Off, On +1 | ✅ |  |
| Pushbutton | API, DisPc, Don, O, Off, Rem +1 | ✅ |  |
| RadioButtons | API, DisPc, I1-8, Max, Off, Rem +2 | ✅ |  |
| RadioButtons16 | API, DisPc, I1-16, Max, Off, Rem +2 | ✅ |  |
| RampController | Off, Rem, St, Sts, Sv, V | ✅ |  |
| RuntimeCounter | API, En, Mi, Off, Rmc, Rtm +1 | ✅ |  |
| Sauna | API, Dc, DisPc, Dry, Dryd, Dryϑ +19 | ✅ |  |
| SaunaEvaporator | API, Dc, DisPc, Dryd, Dryϑ, Ev +29 | ✅ |  |
| Scaler | Sv, Sv1, Sv2, V, V1, V2 | ✅ |  |
| Scene | API, Off | ✅ |  |
| Script | I1-13, Rem, T1-3, Txt1-3 | ✅ |  |
| SecurityCentral | A, API, Ad, Adnp, Anp, Ca +5 | ✅ |  |
| SelectionSwitchOnOff | API, DisPc, Off, Rem, Soff, Son | ✅ |  |
| SelectionSwitchPlus | +, API, DisPc, Off, Rem, Rr +2 | ✅ |  |
| SelectionSwitchPlusMinus | +, -, API, DisPc, Off, Rem +3 | ✅ |  |
| Sequencer | DisPc, Dv, O1-8, P, R, Rem +2 | ✅ |  |
| SequentialController | API, Off, Rem, TQ | ✅ |  |
| SessionDBConnector | API, CI1, CI10, CI11, CI12, CI13 +17 | ✅ |  |
| SkyWindow | API, CPos, Cc, Co, Ic, Io +9 | ✅ |  |
| SkyWindowJalousie | API, Cc, Cl, Co, Dir, DisPc +26 | ✅ |  |
| SolarPumpController | API, Co, Hs, M, Maxϑc, Maxϑs +12 | ✅ |  |
| SpotPriceOptimizer | +0 to +23, API, Avg, High, I2, I3 +9 | ✅ |  |
| StairwellLight | API, DisPc, Don, Off, Rem, Tr +1 | ✅ |  |
| Switch | API, DisPc, Off, Rem, Tg | ✅ |  |
| Tablet | API, Blvl, Cac, DBr, Dnd, Ds +3 | ✅ |  |
| TextGenerator | API, Td, Tr, Tu, Txt, Txt1-4 +2 | ✅ |  |
| TimerScheduler | API, Act, Am, Don, Mm, O +4 | ✅ |  |
| ToiletVentilation | API, DisPc, FPet, Fan, Fsd, Off +4 | ✅ |  |
| TouchGrillAir |  | ✅ |  |
| TouchGrillControl | API, Afb, Ag, At, Atx, Ay +6 | ✅ |  |
| TouchNightlight |  | ✅ |  |
| TouchPureFlex | API, B, Bl, Br, BrBP, BrBnP +16 | ✅ |  |
| UtilityMeter | API, Abs, C, Cf, Cp, Mro +4 | ✅ |  |
| Ventilation | API, B, Bva, Bvp, CO2, CO2max +18 | ✅ |  |
| Wallbox | API, Cp, Cpl, E, M, Mr +4 | ✅ |  |
| WallboxBlock | API, Ca, Cac, Cclc, Cfp, Ec +12 | ✅ |  |
| WallboxManager | API, Ap, Cfp, Cp, Off, Peco +3 | ✅ |  |
| WindGauge | Avg, AvgMax, F, G, Wa | ✅ |  |
| WindowCentral | API, Cc, Co, Nc, No, Off +4 | ✅ |  |

