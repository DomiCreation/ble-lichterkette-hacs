"""BLE Lichterkette: control Bluetooth light sets from Home Assistant."""
from __future__ import annotations

from homeassistant.components import bluetooth
from homeassistant.config_entries import ConfigEntry
from homeassistant.const import CONF_ADDRESS, CONF_NAME, Platform
from homeassistant.core import HomeAssistant, callback
from homeassistant.exceptions import ConfigEntryNotReady

from .const import CONF_KEEP_CONNECTED, DEFAULT_KEEP_CONNECTED, DEFAULT_NAME
from .device import LichterketteDevice

PLATFORMS: list[Platform] = [Platform.LIGHT]

LichterketteConfigEntry = ConfigEntry[LichterketteDevice]


async def async_setup_entry(hass: HomeAssistant, entry: LichterketteConfigEntry) -> bool:
    address: str = entry.data[CONF_ADDRESS]
    ble_device = bluetooth.async_ble_device_from_address(
        hass, address.upper(), connectable=True
    )
    if ble_device is None:
        raise ConfigEntryNotReady(
            f"Lichterkette {address} ist gerade nicht in Bluetooth-Reichweite"
        )

    device = LichterketteDevice(
        ble_device,
        entry.data.get(CONF_NAME) or DEFAULT_NAME,
        entry.options.get(CONF_KEEP_CONNECTED, DEFAULT_KEEP_CONNECTED),
    )
    entry.async_on_unload(entry.add_update_listener(_options_updated))

    @callback
    def _seen(
        service_info: bluetooth.BluetoothServiceInfoBleak,
        _change: bluetooth.BluetoothChange,
    ) -> None:
        device.set_ble_device(service_info.device)

    entry.async_on_unload(
        bluetooth.async_register_callback(
            hass,
            _seen,
            bluetooth.BluetoothCallbackMatcher(address=address.upper(), connectable=True),
            bluetooth.BluetoothScanningMode.PASSIVE,
        )
    )

    entry.runtime_data = device
    await hass.config_entries.async_forward_entry_setups(entry, PLATFORMS)
    return True


async def _options_updated(hass: HomeAssistant, entry: LichterketteConfigEntry) -> None:
    await hass.config_entries.async_reload(entry.entry_id)


async def async_unload_entry(hass: HomeAssistant, entry: LichterketteConfigEntry) -> bool:
    unloaded = await hass.config_entries.async_unload_platforms(entry, PLATFORMS)
    if unloaded:
        await entry.runtime_data.stop()
    return unloaded
