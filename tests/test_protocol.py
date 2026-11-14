"""Checks the frame builder against the rules read from the original app.

Run: python3 -I tests/test_protocol.py  (from the app folder)
"""
import importlib.util
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "protocol", HERE.parent / "custom_components" / "ble_lichterkette" / "protocol.py"
)
p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p)


def check(cond, msg):
    if not cond:
        print("FAIL:", msg)
        sys.exit(1)


# ON with key 0x00: bytes 54 AD 03 01 00 (5) -> 54, 05^54, 00^54, AD 03 01 00
check(p.build_frame("AD030100", key=0).hex() == "545154ad030100", "ON frame, key 0")
# key 0x5A: 54, 51, 5A^54=0E, AD^5A=F7, 03^5A=59, 01^5A=5B, 00^5A=5A
check(p.build_frame("AD030100", key=0x5A).hex() == "54510ef7595b5a", "ON frame, key 5A")
# mode command is one byte shorter: n=4 -> 4^54=50
check(p.build_frame("AD0101", key=0).hex() == "545054ad0101", "mode frame")
for cmd in [p.CMD_ON, p.CMD_OFF, *p.MODE_COMMANDS.values(), p.steady_command(2)]:
    for key in (0, 1, 0x7F, 0xFF):
        check(p.parse_frame(p.build_frame(cmd, key)) == cmd, f"round trip {cmd}/{key}")
check(p.steady_command(3) == "AD010803" and p.steady_command(0) == "AD010800", "steady")
check([p.level_from_brightness(b) for b in (255, 191, 127, 63, 1)] == [3, 2, 1, 0, 0], "levels")
check([p.brightness_from_level(l) for l in (3, 2, 1, 0)] == [255, 191, 127, 63], "brightness")
check(len(p.EFFECT_LIST) == 8, "8 programs")
check(p.MODE_COMMANDS[p.EFFECT_TWINKLE_FLASH] == "AD0107", "twinkle/flash is AD0107")
check(p.MODE_COMMANDS[p.EFFECT_SLOW_FADE] == "AD0106", "slow fade is AD0106")
# Button -> command, as read from the four brightness buttons and the Steady jump table
for app_level, cmd in {1: "AD010803", 2: "AD010802", 3: "AD010801", 4: "AD010800"}.items():
    check(p.steady_command(p.APP_LEVEL_TO_BYTE[app_level]) == cmd, f"app level {app_level}")
# stringToByte rejects odd-length hex; ours must refuse too
try:
    p.build_frame("AD030", key=0)
    check(False, "odd hex accepted")
except ValueError:
    pass
print("all protocol checks passed")
