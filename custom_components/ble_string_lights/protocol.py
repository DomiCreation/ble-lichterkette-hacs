"""Bluetooth protocol of the light sets (pure Python, no Home Assistant imports)."""
from __future__ import annotations

import secrets

# GATT layout. The app scans for service "1000" and writes commands to
# characteristic "1001" (a second layout 2000/2001 is handled the same way).
SERVICE_UUID = "00001000-0000-1000-8000-00805f9b34fb"
WRITE_CHAR_UUID = "00001001-0000-1000-8000-00805f9b34fb"
ALT_SERVICE_UUID = "00002000-0000-1000-8000-00805f9b34fb"
ALT_WRITE_CHAR_UUID = "00002001-0000-1000-8000-00805f9b34fb"
WRITE_CHAR_UUIDS = (WRITE_CHAR_UUID, ALT_WRITE_CHAR_UUID)

# Plain commands (hex text before the frame is built).
CMD_ON = "AD030100"
CMD_OFF = "AD030000"

EFFECT_COMBINATION = "Combination"
EFFECT_IN_WAVES = "In waves"
EFFECT_SEQUENTIAL = "Sequential"
EFFECT_SLOW_GLOW = "Slow glow"
EFFECT_CHASE_FLASH = "Chase/Flash"
EFFECT_TWINKLE_FLASH = "Twinkle/Flash"
EFFECT_SLOW_FADE = "Slow fade"
EFFECT_STEADY = "Steady"

MODE_COMMANDS: dict[str, str] = {
    EFFECT_COMBINATION: "AD0101",
    EFFECT_IN_WAVES: "AD0102",
    EFFECT_SEQUENTIAL: "AD0103",
    EFFECT_SLOW_GLOW: "AD0104",
    EFFECT_CHASE_FLASH: "AD0105",
    EFFECT_TWINKLE_FLASH: "AD0107",
    EFFECT_SLOW_FADE: "AD0106",
}

EFFECT_LIST: list[str] = [*MODE_COMMANDS, EFFECT_STEADY]

# Steady/Dimmer: "AD0108" + level byte. 03 = 100 %, 02 = 75 %, 01 = 50 %, 00 = 25 %.
# Buttons store a level 1..4 (100 % = 1 ... 25 % = 4); byte = 4 - app level.
STEADY_PREFIX = "AD0108"
LEVEL_PERCENT = {3: 100, 2: 75, 1: 50, 0: 25}
APP_LEVEL_TO_BYTE = {1: 3, 2: 2, 3: 1, 4: 0}

FRAME_MARK = 0x54


def steady_command(level: int) -> str:
    """Steady light at one of the four dimmer levels (0..3)."""
    level = max(0, min(3, int(level)))
    return f"{STEADY_PREFIX}{level:02X}"


def level_from_brightness(brightness: int) -> int:
    """Home Assistant brightness (1..255) to the nearest dimmer level (0..3)."""
    quarters = round(max(0, min(255, int(brightness))) / 255 * 4)
    return max(0, min(3, quarters - 1))


def brightness_from_level(level: int) -> int:
    """Dimmer level (0..3) to Home Assistant brightness (63, 127, 191, 255)."""
    return (max(0, min(3, int(level))) + 1) * 64 - 1


def build_frame(command_hex: str, key: int | None = None) -> bytes:
    """Wrap a command the way the app does before writing it.

    bytes = hex("54" + command); n = len(bytes); k = random byte
    frame = [0x54, n ^ 0x54, k ^ 0x54] + [b ^ k for b in bytes[1:]]
    """
    raw = bytes.fromhex("54" + command_hex.replace(" ", ""))
    if key is None:
        key = secrets.randbelow(256)
    key &= 0xFF
    return bytes(
        [FRAME_MARK, (len(raw) ^ FRAME_MARK) & 0xFF, key ^ FRAME_MARK]
        + [b ^ key for b in raw[1:]]
    )


def parse_frame(frame: bytes) -> str:
    """Undo build_frame (used by the tests and the test tool)."""
    if len(frame) < 3 or frame[0] != FRAME_MARK:
        raise ValueError("not a light-set frame")
    length = frame[1] ^ FRAME_MARK
    key = frame[2] ^ FRAME_MARK
    body = bytes(b ^ key for b in frame[3:])
    if len(body) != length - 1:
        raise ValueError("length byte does not match")
    return body.hex().upper()
