"""The light set as a Home Assistant light: on/off, 4-step dimmer, 8 programs."""
from __future__ import annotations

from typing import Any

from homeassistant.components.light import (
    ATTR_BRIGHTNESS,
    ATTR_EFFECT,
    ColorMode,
    LightEntity,
    LightEntityFeature,
)
from homeassistant.const import STATE_ON
from homeassistant.core import HomeAssistant
from homeassistant.helpers import device_registry as dr
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity_platform import AddEntitiesCallback
from homeassistant.helpers.restore_state import RestoreEntity

from . import StringLightsConfigEntry
from .const import DOMAIN
from .protocol import (
    CMD_OFF,
    CMD_ON,
    EFFECT_LIST,
    EFFECT_STEADY,
    MODE_COMMANDS,
    brightness_from_level,
    level_from_brightness,
    steady_command,
)


async def async_setup_entry(
    hass: HomeAssistant,
    entry: StringLightsConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([StringLightsLight(entry)])


class StringLightsLight(LightEntity, RestoreEntity):
    """One light set."""

    _attr_has_entity_name = True
    _attr_name = None
    # The light set does not report back, so Home Assistant shows what it last sent.
    _attr_assumed_state = True
    _attr_color_mode = ColorMode.BRIGHTNESS
    _attr_supported_color_modes = {ColorMode.BRIGHTNESS}
    _attr_supported_features = LightEntityFeature.EFFECT
    _attr_effect_list = EFFECT_LIST

    def __init__(self, entry: StringLightsConfigEntry) -> None:
        self._device = entry.runtime_data
        address = self._device.address
        self._attr_unique_id = address
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, address)},
            connections={(dr.CONNECTION_BLUETOOTH, address)},
            name=entry.title,
            model="Bluetooth string lights",
        )
        self._attr_is_on = False
        self._attr_brightness = brightness_from_level(3)
        self._attr_effect = EFFECT_STEADY

    async def async_added_to_hass(self) -> None:
        await super().async_added_to_hass()
        last = await self.async_get_last_state()
        if last is None:
            return
        self._attr_is_on = last.state == STATE_ON
        if (b := last.attributes.get(ATTR_BRIGHTNESS)) is not None:
            self._attr_brightness = brightness_from_level(level_from_brightness(b))
        if (e := last.attributes.get(ATTR_EFFECT)) in EFFECT_LIST:
            self._attr_effect = e

    async def async_turn_on(self, **kwargs: Any) -> None:
        commands: list[str] = []
        if not self._attr_is_on or not (ATTR_EFFECT in kwargs or ATTR_BRIGHTNESS in kwargs):
            commands.append(CMD_ON)

        effect = kwargs.get(ATTR_EFFECT)
        if ATTR_BRIGHTNESS in kwargs:
            # Like the app: choosing a brightness switches to steady light.
            self._attr_brightness = brightness_from_level(
                level_from_brightness(kwargs[ATTR_BRIGHTNESS])
            )
            if effect is None:
                effect = EFFECT_STEADY

        if effect in MODE_COMMANDS:
            commands.append(MODE_COMMANDS[effect])
            self._attr_effect = effect
        elif effect == EFFECT_STEADY:
            commands.append(steady_command(level_from_brightness(self._attr_brightness or 255)))
            self._attr_effect = EFFECT_STEADY

        await self._device.send(*commands)
        self._attr_is_on = True
        self.async_write_ha_state()

    async def async_turn_off(self, **kwargs: Any) -> None:
        await self._device.send(CMD_OFF)
        self._attr_is_on = False
        self.async_write_ha_state()
