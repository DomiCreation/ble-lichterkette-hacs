# BLE String Lights for Home Assistant 🇺🇸

Custom integration that controls Bluetooth LED string lights (the kind otherwise operated with the iPhone app) directly from Home Assistant. **Tested with Lumineo light chain/string lights.**

## Features

- On / Off
- Brightness in four steps (25 / 50 / 75 / 100 %). As in the app, changing brightness switches to Steady.
- 8 light programs as effects: Combination, In waves, Sequential, Slow glow, Chase/Flash, Twinkle/Flash, Slow fade, Steady

## Limitations

The app's timer and music mode are not included. The lights do not report their state; Home Assistant shows the last command sent (assumed state).

## Requirements

- Home Assistant 2024.6 or newer
- A Bluetooth adapter on the Home Assistant host, or an [ESPHome Bluetooth proxy](https://esphome.io/components/bluetooth_proxy/) near the lights

The lights usually accept only one Bluetooth connection at a time. Close the iPhone app before using this integration.

## Installation

### Home Assistant 2026.11 and later (Marketplace)

HACS ships with Home Assistant as **Settings → Marketplace**. To add this repository before it is in the default catalog:

1. Go to **Settings → Marketplace**.
2. In the upper-right corner, select **Add from link**.
3. Enter `https://github.com/DomiCreation/ble-string-lights-hacs` (or `DomiCreation/ble-string-lights-hacs`) and select **Add**. Connect GitHub if prompted.
4. Open the repository in the list and install it, then restart Home Assistant if asked.

### Home Assistant 2024.6–2026.10 (HACS)

1. [Install HACS](https://www.hacs.xyz/docs/use/download/download/) if you do not have it yet.
2. In Home Assistant: **HACS → ⋮ → Custom repositories**, add `https://github.com/DomiCreation/ble-string-lights-hacs`, category **Integration**.
3. Download **BLE String Lights** and restart Home Assistant.

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=DomiCreation&repository=ble-string-lights-hacs&category=integration)

On Home Assistant 2026.11 and later this my-link still works; it opens the repository in the Marketplace.

### Manual

Copy `custom_components/ble_string_lights` to `/config/custom_components/ble_string_lights` and restart Home Assistant.

## Setup

1. Switch the lights on and close the iPhone app.
2. If Home Assistant discovers them, they appear under **Settings → Devices & services** as a discovered device. Otherwise: **Add integration → BLE String Lights** and pick the device from the list (**★** = advertises the expected Bluetooth service).
3. Several strings can be switched together with a Home Assistant [light group](https://www.home-assistant.io/integrations/group/#light-group).

Option **Keep the Bluetooth connection open** under **Configure** (default on), like the app. When off, the connection is released 30 seconds after the last command so the phone app can connect in between.

Example:

```yaml
action: light.turn_on
target:
  entity_id: light.string_lights
data:
  effect: "Twinkle/Flash"
```

## Supported devices

GATT service `00001000-0000-1000-8000-00805f9b34fb` or `00002000-0000-1000-8000-00805f9b34fb` with the matching app's protocol. **Tested with Lumineo light chain/string lights.** Others with the same services and protocol may work; that is unverified.

The protocol is documented in [`MECHANICS.md`](MECHANICS.md).

---

# BLE String Lights für Home Assistant 🇩🇪

Benutzerdefinierte Integration, die Bluetooth-Lichterketten (die sonst mit der iPhone-App bedient werden) direkt aus Home Assistant steuert. **Getestet mit Lumineo Lichterkette.**

## Funktionen

- Ein / Aus
- Helligkeit in vier Stufen (25 / 50 / 75 / 100 %). Wie in der App schaltet das Dimmen auf Steady (Dauerlicht).
- 8 Lichtprogramme als Effekte: Combination, In waves, Sequential, Slow glow, Chase/Flash, Twinkle/Flash, Slow fade, Steady

## Einschränkungen

Timer und Musik-Modus der App sind nicht enthalten. Die Lichterkette meldet ihren Zustand nicht zurück; Home Assistant zeigt den zuletzt gesendeten Befehl (angenommener Zustand).

## Voraussetzungen

- Home Assistant 2024.6 oder neuer
- Ein Bluetooth-Adapter am Home-Assistant-Rechner oder ein [ESPHome-Bluetooth-Proxy](https://esphome.io/components/bluetooth_proxy/) in der Nähe der Lichterkette

Die Lichterkette nimmt meist nur eine Bluetooth-Verbindung gleichzeitig an. Schließe die iPhone-App, bevor du die Integration nutzt.

## Installation

### Home Assistant 2026.11 und neuer (Marketplace)

Ab dieser Version ist HACS als **Einstellungen → Marketplace** fest eingebaut. So fügst du dieses Repository hinzu, solange es noch nicht im Standardkatalog steht:

1. **Einstellungen → Marketplace** öffnen.
2. Oben rechts **Add from link** / **Über Link hinzufügen** wählen.
3. `https://github.com/DomiCreation/ble-string-lights-hacs` (oder `DomiCreation/ble-string-lights-hacs`) eintragen und **Add** wählen. GitHub bei Bedarf verbinden.
4. Das Repository in der Liste öffnen, installieren und Home Assistant bei Bedarf neu starten.

### Home Assistant 2024.6–2026.10 (HACS)

1. [HACS installieren](https://www.hacs.xyz/docs/use/download/download/), falls noch nicht vorhanden.
2. In Home Assistant: **HACS → ⋮ → Benutzerdefinierte Repositories**, `https://github.com/DomiCreation/ble-string-lights-hacs` eintragen, Typ **Integration**.
3. **BLE String Lights** herunterladen und Home Assistant neu starten.

[![Home Assistant öffnen und das Repository im Home Assistant Community Store anzeigen.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=DomiCreation&repository=ble-string-lights-hacs&category=integration)

Unter Home Assistant 2026.11 und neuer funktioniert dieser My-Link weiterhin und öffnet das Repository im Marketplace.

### Manuell

Den Ordner `custom_components/ble_string_lights` nach `/config/custom_components/ble_string_lights` kopieren und Home Assistant neu starten.

## Einrichten

1. Lichterkette einschalten und die iPhone-App schließen.
2. Wird sie automatisch gefunden, erscheint sie unter **Einstellungen → Geräte & Dienste** als entdeckt. Sonst: **Integration hinzufügen → BLE String Lights** und das Gerät aus der Liste wählen (**★** = sendet den erwarteten Bluetooth-Dienst).
3. Mehrere Lichterketten gemeinsam schalten: in Home Assistant eine [Lichtgruppe](https://www.home-assistant.io/integrations/group/#light-group) anlegen.

Option **Bluetooth-Verbindung offen halten** unter **Konfigurieren** (Standard: an), wie in der App. Wenn aus, wird die Verbindung 30 Sekunden nach dem letzten Befehl freigegeben, damit die Handy-App zwischendurch verbinden kann.

Beispiel:

```yaml
action: light.turn_on
target:
  entity_id: light.string_lights
data:
  effect: "Twinkle/Flash"
```

## Unterstützte Geräte

GATT-Dienst `00001000-0000-1000-8000-00805f9b34fb` oder `00002000-0000-1000-8000-00805f9b34fb` mit dem Protokoll der passenden App. **Getestet mit Lumineo Lichterkette.** Andere Geräte mit denselben Diensten und demselben Protokoll können funktionieren; das ist nicht geprüft.

Das Protokoll steht in [`MECHANICS.md`](MECHANICS.md).
