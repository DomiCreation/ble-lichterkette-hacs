"""Connection to one light set."""
from __future__ import annotations

import asyncio
import logging

from bleak.backends.device import BLEDevice
from bleak.exc import BleakError
from bleak_retry_connector import (
    BleakClientWithServiceCache,
    establish_connection,
)

from .const import CONNECT_ATTEMPTS, IDLE_DISCONNECT_DELAY
from .protocol import WRITE_CHAR_UUIDS, build_frame

_LOGGER = logging.getLogger(__name__)


class StringLightsDevice:
    """Sends commands to one light set over Bluetooth LE."""

    def __init__(self, ble_device: BLEDevice, name: str, keep_connected: bool) -> None:
        self._ble_device = ble_device
        self.name = name
        self.keep_connected = keep_connected
        self._client: BleakClientWithServiceCache | None = None
        self._write_char = None
        self._lock = asyncio.Lock()
        self._disconnect_timer: asyncio.TimerHandle | None = None

    @property
    def address(self) -> str:
        return self._ble_device.address

    def set_ble_device(self, ble_device: BLEDevice) -> None:
        """Newer advertisement (or another Bluetooth proxy) for the same device."""
        self._ble_device = ble_device

    async def send(self, *commands: str) -> None:
        """Write one or more commands, in order."""
        async with self._lock:
            self._cancel_disconnect_timer()
            for attempt in range(2):
                try:
                    await self._ensure_connected()
                    assert self._client is not None
                    for command in commands:
                        frame = build_frame(command)
                        _LOGGER.debug("%s: %s -> %s", self.name, command, frame.hex())
                        # Write without response.
                        await self._client.write_gatt_char(
                            self._write_char, frame, response=False
                        )
                        await asyncio.sleep(0.05)
                    break
                except (BleakError, asyncio.TimeoutError) as err:
                    _LOGGER.debug("%s: write failed (%s), retrying", self.name, err)
                    await self._disconnect()
                    if attempt:
                        raise
            if not self.keep_connected:
                self._schedule_disconnect()

    async def _ensure_connected(self) -> None:
        if self._client is not None and self._client.is_connected:
            return
        client = await establish_connection(
            BleakClientWithServiceCache,
            self._ble_device,
            self.name,
            disconnected_callback=self._on_disconnected,
            max_attempts=CONNECT_ATTEMPTS,
        )
        char = None
        for uuid in WRITE_CHAR_UUIDS:
            char = client.services.get_characteristic(uuid)
            if char is not None:
                break
        if char is None:
            await client.disconnect()
            raise BleakError(
                f"{self.name}: no command characteristic 1001/2001 found"
            )
        self._client = client
        self._write_char = char

    def _on_disconnected(self, _client) -> None:
        self._client = None
        self._write_char = None

    def _cancel_disconnect_timer(self) -> None:
        if self._disconnect_timer is not None:
            self._disconnect_timer.cancel()
            self._disconnect_timer = None

    def _schedule_disconnect(self) -> None:
        self._cancel_disconnect_timer()
        loop = asyncio.get_running_loop()
        self._disconnect_timer = loop.call_later(
            IDLE_DISCONNECT_DELAY,
            lambda: asyncio.ensure_future(self._timed_disconnect()),
        )

    async def _timed_disconnect(self) -> None:
        async with self._lock:
            await self._disconnect()

    async def _disconnect(self) -> None:
        client, self._client, self._write_char = self._client, None, None
        if client is not None:
            try:
                await client.disconnect()
            except BleakError:
                pass

    async def stop(self) -> None:
        self._cancel_disconnect_timer()
        await self._disconnect()
