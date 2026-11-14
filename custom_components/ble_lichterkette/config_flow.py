"""Setup dialog: pick a light set found over Bluetooth."""
from __future__ import annotations

from typing import Any

import voluptuous as vol

from homeassistant.components.bluetooth import (
    BluetoothServiceInfoBleak,
    async_discovered_service_info,
)
from homeassistant.config_entries import (
    ConfigEntry,
    ConfigFlow,
    ConfigFlowResult,
    OptionsFlow,
)
from homeassistant.const import CONF_ADDRESS, CONF_NAME
from homeassistant.core import callback

from .const import CONF_KEEP_CONNECTED, DEFAULT_KEEP_CONNECTED, DEFAULT_NAME, DOMAIN
from .protocol import ALT_SERVICE_UUID, SERVICE_UUID


def _looks_like_light_set(info: BluetoothServiceInfoBleak) -> bool:
    uuids = {u.lower() for u in info.service_uuids}
    return SERVICE_UUID in uuids or ALT_SERVICE_UUID in uuids


def _title(info: BluetoothServiceInfoBleak) -> str:
    name = info.name if info.name and info.name != info.address else DEFAULT_NAME
    return name


class LichterketteConfigFlow(ConfigFlow, domain=DOMAIN):
    """Config flow for BLE Lichterkette."""

    VERSION = 1

    def __init__(self) -> None:
        self._discovery: BluetoothServiceInfoBleak | None = None
        self._found: dict[str, BluetoothServiceInfoBleak] = {}

    async def async_step_bluetooth(
        self, discovery_info: BluetoothServiceInfoBleak
    ) -> ConfigFlowResult:
        """Found automatically by Home Assistant's Bluetooth scanner."""
        await self.async_set_unique_id(discovery_info.address)
        self._abort_if_unique_id_configured()
        self._discovery = discovery_info
        self.context["title_placeholders"] = {"name": _title(discovery_info)}
        return await self.async_step_bluetooth_confirm()

    async def async_step_bluetooth_confirm(
        self, user_input: dict[str, Any] | None = None
    ) -> ConfigFlowResult:
        assert self._discovery is not None
        if user_input is not None:
            return self._create(self._discovery.address, user_input[CONF_NAME])
        self._set_confirm_only()
        return self.async_show_form(
            step_id="bluetooth_confirm",
            data_schema=vol.Schema(
                {vol.Required(CONF_NAME, default=_title(self._discovery)): str}
            ),
            description_placeholders={
                "name": _title(self._discovery),
                "address": self._discovery.address,
            },
        )

    async def async_step_user(
        self, user_input: dict[str, Any] | None = None
    ) -> ConfigFlowResult:
        """Added by hand: choose from every Bluetooth device in range."""
        if user_input is not None:
            address = user_input[CONF_ADDRESS]
            await self.async_set_unique_id(address, raise_on_progress=False)
            self._abort_if_unique_id_configured()
            return self._create(address, user_input[CONF_NAME])

        configured = self._async_current_ids(include_ignore=False)
        self._found = {
            info.address: info
            for info in async_discovered_service_info(self.hass, connectable=True)
            if info.address not in configured
        }
        if not self._found:
            return self.async_abort(reason="no_devices_found")

        # Light sets that advertise service 1000/2000 first, then the rest by signal.
        ordered = sorted(
            self._found.values(),
            key=lambda i: (not _looks_like_light_set(i), -(i.rssi or -127)),
        )
        choices = {
            i.address: (
                f"{'★ ' if _looks_like_light_set(i) else ''}{i.name or '?'} "
                f"({i.address}, {i.rssi} dBm)"
            )
            for i in ordered
        }
        return self.async_show_form(
            step_id="user",
            data_schema=vol.Schema(
                {
                    vol.Required(CONF_ADDRESS): vol.In(choices),
                    vol.Required(CONF_NAME, default=DEFAULT_NAME): str,
                }
            ),
        )

    def _create(self, address: str, name: str) -> ConfigFlowResult:
        return self.async_create_entry(
            title=name or DEFAULT_NAME,
            data={CONF_ADDRESS: address, CONF_NAME: name or DEFAULT_NAME},
        )

    @staticmethod
    @callback
    def async_get_options_flow(config_entry: ConfigEntry) -> OptionsFlow:
        return LichterketteOptionsFlow()


class LichterketteOptionsFlow(OptionsFlow):
    """Keep the Bluetooth connection open (like the app) or release it."""

    async def async_step_init(
        self, user_input: dict[str, Any] | None = None
    ) -> ConfigFlowResult:
        if user_input is not None:
            return self.async_create_entry(data=user_input)
        current = self.config_entry.options.get(
            CONF_KEEP_CONNECTED, DEFAULT_KEEP_CONNECTED
        )
        return self.async_show_form(
            step_id="init",
            data_schema=vol.Schema(
                {vol.Required(CONF_KEEP_CONNECTED, default=current): bool}
            ),
        )
