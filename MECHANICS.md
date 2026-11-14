# How the light sets are controlled

Read from the original iPhone app's program (`ZDLights`, arm64, Objective-C,
sha256 820fa718…6397). Only one 4 KiB page is encrypted (0x100028000–0x100029000,
start-up code); the Bluetooth code is readable.

How it was read: every function below was opened with `native_function_pseudocode`.
Ghidra could not decompile on this Mac (its decompiler binary is missing from the
macOS download), so the built-in decompiler answered. That decompiler drops some
immediate values (it shows `wzr` where the instruction loads 1 or 0x1000); those
values were read from the instructions at the addresses given.

## Bluetooth layout

| Value | Number | Source |
|---|---|---|
| Service the app scans for | `1000` (`00001000-0000-1000-8000-00805f9b34fb`) | `-[ConnecTool startSearch]` 0x10002f4bc → `scanDeviceByServerUUID:@"1000" timeOut:0 filter:NO`, string 0x100056308 |
| Scan filter | none: every device in range is listed, no name check | same call (filter NO); `-[ZDSearchViewController refreshDeviceArray:]` 0x10003407c uses no strings |
| Command characteristic | service `1000` → char `1001` (`1003` for the other write); service `2000` → `2001`/`2003` | `-[BLEManager sendDataToDeviceWithUUID1:device:UUIDString:]` 0x10001865c; service values `orr w2,wzr,#0x1000` 0x100018564 / `#0x2000` 0x1000185b4 |
| Characteristic used for commands | `1001` | `-[ConnecTool sendData:device:]` 0x10002f650 passes string 0x100056348 "1001" |
| Write type | without response | `-[BLEManager writeValue:characteristicUUID:p:data:responseState:]` 0x100019f60: responseState 0 → `orr w4,wzr,#1` (type 1) at 0x10001a0e4; every caller passes `w6 = 0` (0x100018634) |

## Frame ("encryption")

- Always on: `-[ConnecTool init]` 0x10002f170 calls `setIsEncryption:` with
  `mov w2,#1` at 0x10002f204.
- `-[BLEManager encryption:]` 0x10001b874:
  1. text = `"54"` + command (format `54%@`, string 0x1000567e8)
  2. bytes = `stringToByte:` (0x10001bc84): uppercase, spaces removed, odd length
     → nothing, pairs of 0-9/A-F → bytes; n = length
  3. k = `rand()` × 255, low byte (`lsl w8,w0,#8; sub w8,w8,w0` at 0x10001b920)
  4. frame = `[0x54, n ^ 0x54, k ^ 0x54] + [b ^ k for b in bytes[1:]]`,
     length n + 2 (0x10001b978)

Implemented in `protocol.py` `build_frame()`; tests in `tests/test_protocol.py`.

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

The control screen's program buttons (`-[ZDControlViewController …BtnBeClicked]`,
e.g. 0x10003ba2c) only forward to the list screen above, which sends the command
to every selected light set (`sendBulbCommandString:` 0x100036624, one
`-[ConnecTool sendData:device:]` per light set).

The control screen's own off button (`offBtnClicked` 0x10003b450) only stores
`MODE_select = 0` and asks for `offBtnClickedNine`, which nothing in the program
implements; turning off works through the list's off button (`AD030000`).

## Brightness (Steady/Dimmer)

| Button | Stored app level | Command sent | Source |
|---|---|---|---|
| 100 % | 1 | `AD010803` | `tenPerBtn:` 0x10000f73c (`orr w8,wzr,#1` 0x10000f760), string 0x100055a88 |
| 75 % | 2 | `AD010802` | `sevenFiveBtn:` 0x10000f86c (`#2` 0x10000f890), string 0x100055aa8 |
| 50 % | 3 | `AD010801` | `fivePerBtn:` 0x10000f99c (`#3` 0x10000f9c0), string 0x100055ac8 |
| 25 % | 4 | `AD010800` | `twoFiveBtn:` 0x10000facc (`#4` 0x10000faf0), string 0x100055ae8 |

Rule: level byte = 4 − app level. Pressing a brightness button sends the steady
command at once (brightness = steady light). The Steady button
(`steadyBtnBeClicked` 0x100036540) resends the last level through a jump table at
0x1000365dc; with no level chosen yet (level 0, never set in `viewDidLoad`
0x100030dbc) it sends nothing.

In Home Assistant: brightness 1–255 goes to the nearest of the four levels
(round(b / 255 × 4) − 1, clamped 0–3); a level shows as 63/127/191/255.
**Deliberate difference:** Home Assistant always has a brightness, so the
"Steady" effect starts at 100 % instead of sending nothing.

## Connection

| Rule | Value | Source |
|---|---|---|
| Kept open while the control screen is open | yes; disconnect only when leaving | `disConnectSelectDevice` 0x100036330 → `delayAutoDisConnect:` 0x100036444 |
| Stagger when disconnecting several light sets | 0.01 s × index | constant 0x100043ef0 = 0.01, `fmul` at 0x1000363e4 |
| Reconnect timer | every 0.2 s | `-[ConnecTool init]` 0x10002f26c, constant 0x100043d08 = 0.2 |
| Reconnect attempts | 20 (stops when count ≥ 0x14) | `-[ConnecTool repeatConnectMethod]` 0x10002f348 |
| State read back | none: notify is switched on, but no screen handles incoming data | `-[BLEManager enableNotify:]` 0x100018238; ConnecTool has no receive method |

In Home Assistant the connection is kept open by default (option "keep
connected", like the app). If the person turns it off (to use the phone app as
well), it is released 30 s after the last command (**estimated**, the app has no
idle timeout). Reconnecting uses 4 attempts of bleak-retry-connector instead of
20 × 0.2 s, because one attempt over a Bluetooth proxy can take longer than the
app's whole 4 s budget (deliberate difference). The light shows an assumed state.

## Left out (not the main function)

- Two on/off timers stored on the light set (`AD%@%@%@%@%@%@`, 0x1000579a8) and
  clock sync on connect (`AD06%@`, 0x100057988,
  `sendConnectRTCCommandStringDevice:` 0x100035490).
- Music mode (driven by the phone's audio).
