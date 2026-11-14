# How the light sets are controlled

## Bluetooth layout

| Value | Number |
|---|---|
| Service the app scans for | `1000` (`00001000-0000-1000-8000-00805f9b34fb`) |
| Scan filter | none: every device in range is listed, no name check |
| Command characteristic | service `1000` → char `1001` (`1003` for the other write); service `2000` → `2001`/`2003` |
| Characteristic used for commands | `1001` |
| Write type | without response |

## Commands

| Function | Command |
|---|---|
| Turn on | `AD030100` |
| Turn off | `AD030000` |
| Combination | `AD0101` |
| In waves | `AD0102` |
| Sequential | `AD0103` |
| Slow glow | `AD0104` |
| Chase/Flash | `AD0105` |
| Twinkle/Flash | `AD0107` |
| Slow fade | `AD0106` |

## Brightness (Steady/Dimmer)

| Button | Stored app level | Command sent |
|---|---|---|
| 100 % | 1 | `AD010803` |
| 75 % | 2 | `AD010802` |
| 50 % | 3 | `AD010801` |
| 25 % | 4 | `AD010800` |

## Connection

| Rule | Value |
|---|---|
| Kept open while the control screen is open | yes; disconnect only when leaving |
| Stagger when disconnecting several light sets | 0.01 s × index |
| Reconnect timer | every 0.2 s |
| Reconnect attempts | 20 |
| State read back | none: notify is switched on, but no screen handles incoming data |
