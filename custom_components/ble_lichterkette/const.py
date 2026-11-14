"""Constants for the BLE Lichterkette integration."""
from __future__ import annotations

DOMAIN = "ble_lichterkette"
DEFAULT_NAME = "Lichterkette"

# The app keeps the connection open while its control screen is open and only
# disconnects when you leave it (-[ZDSearchViewController disConnectSelectDevice]
# 0x100036330). Home Assistant is "always on the control screen", so by default
# the connection is kept open the same way.
CONF_KEEP_CONNECTED = "keep_connected"
DEFAULT_KEEP_CONNECTED = True

# Only used when the person turns "keep connected" off, so the phone app can
# connect in between (estimated, the app has no idle timeout).
IDLE_DISCONNECT_DELAY = 30.0

# The app retries a lost connection every 0.2 s, at most 20 times
# (-[ConnecTool init] 0x10002f26c timer 0.2 s, -[ConnecTool repeatConnectMethod]
# 0x10002f348 stops after count >= 0x14).
APP_RECONNECT_INTERVAL = 0.2
APP_RECONNECT_ATTEMPTS = 20
# One attempt over a Bluetooth proxy can take longer than the app's whole 4 s
# budget, so Home Assistant uses bleak-retry-connector's own attempts instead.
CONNECT_ATTEMPTS = 4
