# How the light sets are controlled

## Bluetooth layout

| Value | Number | Source |
|---|---|---|
| Service the app scans for | `1000` (`00001000-0000-1000-8000-00805f9b34fb`) | `-[ConnecTool startSearch]` 0x10002f4bc → `scanDeviceByServerUUID:@"1000" timeOut:0 filter:NO`, string 0x100056308 |
| Scan filter | none: every device in range is listed, no name check | same call (filter NO); `-[ZDSearchViewController refreshDeviceArray:]` 0x10003407c uses no strings |
| Command characteristic | service `1000` → char `1001` (`1003` for the other write); service `2000` → `2001`/`2003` | `-[BLEManager sendDataToDeviceWithUUID1:device:UUIDString:]` 0x10001865c; service values `orr w2,wzr,#0x1000` 0x100018564 / `#0x2000` 0x1000185b4 |
| Characteristic used for commands | `1001` | `-[ConnecTool sendData:device:]` 0x10002f650 passes string 0x100056348 "1001" |
| Write type | without response | `-[BLEManager writeValue:characteristicUUID:p:data:responseState:]` 0x100019f60: responseState 0 → `orr w4,wzr,#1` (type 1) at 0x10001a0e4; every caller passes `w6 = 0` (0x100018634) |

## Commands

| Function | Command | Source |
|---|---|---|
| Turn on | `AD030100` | string 0x1000579e8, `lightListViewOnButtonBeClicked` 0x100036308 |
| Turn off | `AD030000` | string 0x100057a08, `lightListViewOffButtonBeClicked` 0x10003631c |
| Combination | `AD0101` | 0x100057a28, `combinationBtnBeClicked` 0x1000364b4 |
| In waves | `AD0102` | 0x100057a48, `inWavesBtnBeClicked` 0x1000364c8 |
| Sequential | `AD0103` | 0x100057a68, `sequentialBtnBeClicked` 0x1000364dc |
| Slow glow | `AD0104` | 0x100057a88, `slogloBtnBeClicked` 0x1000364f0 |
| Chase/Flash | `AD0105` | 0x100057aa8, `chasingFlashBtnBeClicked` 0x100036504 |
| Twinkle/Flash | `AD0107` | Device test on the light set. iPhone `twinkleFlashBtnBeClicked` 0x100036518 / string 0x100057ac8 sends `AD0106` for this label; on the device that command is Slow fade. |
| Slow fade | `AD0106` | Device test on the light set. iPhone `slowFadeBtnBeClicked` 0x10003652c / string 0x100057ae8 sends `AD0107` for this label; on the device that command is Twinkle/Flash. |

## Brightness (Steady/Dimmer)

| Button | Stored app level | Command sent | Source |
|---|---|---|---|
| 100 % | 1 | `AD010803` | `tenPerBtn:` 0x10000f73c (`orr w8,wzr,#1` 0x10000f760), string 0x100055a88 |
| 75 % | 2 | `AD010802` | `sevenFiveBtn:` 0x10000f86c (`#2` 0x10000f890), string 0x100055aa8 |
| 50 % | 3 | `AD010801` | `fivePerBtn:` 0x10000f99c (`#3` 0x10000f9c0), string 0x100055ac8 |
| 25 % | 4 | `AD010800` | `twoFiveBtn:` 0x10000facc (`#4` 0x10000faf0), string 0x100055ae8 |

## Connection

| Rule | Value | Source |
|---|---|---|
| Kept open while the control screen is open | yes; disconnect only when leaving | `disConnectSelectDevice` 0x100036330 → `delayAutoDisConnect:` 0x100036444 |
| Stagger when disconnecting several light sets | 0.01 s × index | constant 0x100043ef0 = 0.01, `fmul` at 0x1000363e4 |
| Reconnect timer | every 0.2 s | `-[ConnecTool init]` 0x10002f26c, constant 0x100043d08 = 0.2 |
| Reconnect attempts | 20 (stops when count ≥ 0x14) | `-[ConnecTool repeatConnectMethod]` 0x10002f348 |
| State read back | none: notify is switched on, but no screen handles incoming data | `-[BLEManager enableNotify:]` 0x100018238; ConnecTool has no receive method |
