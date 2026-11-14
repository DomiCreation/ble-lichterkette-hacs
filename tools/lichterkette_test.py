"""Test a light set from a Mac or PC, without Home Assistant.

Setup once:   python3 -m pip install bleak
Find lights:  python3 tools/lichterkette_test.py scan
Control:      python3 tools/lichterkette_test.py <address> on|off|steady 100|75|50|25|mode <1-7>

Modes: 1 Combination, 2 In waves, 3 Sequential, 4 Slow glow,
       5 Chase/Flash, 6 Twinkle/Flash, 7 Slow fade
"""
import asyncio
import importlib.util
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location(
    "protocol", HERE.parent / "custom_components" / "ble_string_lights" / "protocol.py"
)
p = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(p)

PERCENT_TO_LEVEL = {100: 3, 75: 2, 50: 1, 25: 0}
MODES = list(p.MODE_COMMANDS.values())


async def scan() -> None:
    from bleak import BleakScanner

    print("Suche 8 Sekunden nach Bluetooth-Geräten ...")
    found = await BleakScanner.discover(timeout=8.0, return_adv=True)
    rows = []
    for dev, adv in found.values():
        uuids = {u.lower() for u in adv.service_uuids}
        likely = p.SERVICE_UUID in uuids or p.ALT_SERVICE_UUID in uuids
        rows.append((not likely, -adv.rssi, dev.address, dev.name or "?", likely))
    for _, _, address, name, likely in sorted(rows):
        print(f"{'★' if likely else ' '} {address}  {name}")
    print("★ = sendet den Lichterketten-Dienst 1000/2000")


async def control(address: str, words: list[str]) -> None:
    from bleak import BleakClient

    action = words[0].lower()
    if action == "on":
        command = p.CMD_ON
    elif action == "off":
        command = p.CMD_OFF
    elif action == "steady":
        command = p.steady_command(PERCENT_TO_LEVEL[int(words[1])])
    elif action == "mode":
        command = MODES[int(words[1]) - 1]
    else:
        raise SystemExit(__doc__)

    async with BleakClient(address, timeout=15.0) as client:
        char = None
        for uuid in p.WRITE_CHAR_UUIDS:
            char = client.services.get_characteristic(uuid)
            if char:
                break
        if char is None:
            raise SystemExit("Kein Befehls-Kanal 1001/2001 gefunden – ist das die Lichterkette?")
        frame = p.build_frame(command)
        await client.write_gatt_char(char, frame, response=False)
        await asyncio.sleep(0.3)
        print(f"gesendet: {command} ({frame.hex()})")


def main() -> None:
    args = sys.argv[1:]
    if not args:
        raise SystemExit(__doc__)
    if args[0] == "scan":
        asyncio.run(scan())
    elif len(args) >= 2:
        asyncio.run(control(args[0], args[1:]))
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()
