# BLE Lichterkette für Home Assistant

Steuert Bluetooth-Lichterketten, die sonst mit der iPhone-App bedient werden,
direkt aus Home Assistant:

- **Ein / Aus**
- **Helligkeit** in vier Stufen (25 %, 50 %, 75 %, 100 %) – wie in der App schaltet
  das Dimmen auf „Steady“ (Dauerlicht)
- **8 Lichtprogramme** als Effekte: Combination, In waves, Sequential, Slow glow,
  Chase/Flash, Twinkle/Flash, Slow fade, Steady

Timer und Musik-Modus der App sind nicht enthalten.

## Voraussetzungen

- Home Assistant 2024.6 oder neuer
- Bluetooth in Home Assistant: ein Bluetooth-Adapter am Home-Assistant-Rechner
  oder ein **ESPHome-Bluetooth-Proxy** in der Nähe der Lichterkette

## Installation über HACS

1. Diesen Ordner als eigenes GitHub-Repository hochladen (der Inhalt dieses
   Ordners ist das Repository: `custom_components/`, `hacs.json`, diese Datei).
2. In Home Assistant: **HACS → ⋮ → Benutzerdefinierte Repositories**, die
   Repository-Adresse eintragen, Typ **Integration**.
3. „BLE Lichterkette“ in HACS herunterladen und Home Assistant neu starten.

### Ohne HACS

Den Ordner `custom_components/ble_lichterkette` nach
`/config/custom_components/ble_lichterkette` kopieren und Home Assistant neu starten.

## Einrichten

1. Lichterkette einschalten. **Die iPhone-App schließen** – die Lichterkette
   nimmt meist nur eine Bluetooth-Verbindung gleichzeitig an. Wie die App hält
   Home Assistant die Verbindung offen; willst du die App weiter nutzen, schalte
   unter **Konfigurieren → „Bluetooth-Verbindung offen halten“** aus.
2. Wird sie automatisch gefunden, erscheint sie unter **Einstellungen → Geräte &
   Dienste** als „Entdeckt“. Sonst: **Integration hinzufügen → BLE Lichterkette**
   und das Gerät aus der Liste wählen (★ = sendet den passenden Bluetooth-Dienst).
3. Mehrere Lichterketten gemeinsam schalten (wie „Alle“ in der App): in Home
   Assistant eine **Lichtgruppe** anlegen.

Beispiel-Aktion:

```yaml
action: light.turn_on
target:
  entity_id: light.lichterkette
data:
  effect: "Twinkle/Flash"
```

## Am Mac testen (optional)

Ein kleines Fensterprogramm (Rust) schickt dieselben Befehle wie die
Home-Assistant-Erweiterung – praktisch, um die Lichterkette vorher zu prüfen:

```sh
cargo run --release
```

„Lichterketten suchen“ → Gerät anklicken (★ = sendet den Lichterketten-Dienst)
→ Ein/Aus, Programme und Helligkeit ausprobieren. Beim ersten Mal fragt macOS
nach der Bluetooth-Erlaubnis für das Terminal.

Ohne Fenster geht es auch im Terminal mit Python:

```sh
python3 -m pip install bleak
python3 tools/lichterkette_test.py scan
python3 tools/lichterkette_test.py <Adresse> on
python3 tools/lichterkette_test.py <Adresse> mode 3
python3 tools/lichterkette_test.py <Adresse> steady 50
```

Am Mac zeigt `scan` statt einer MAC-Adresse eine lange Kennung – die ist nur
für den Test am Mac; Home Assistant zeigt die echte Adresse.

## Hinweise

- Die Lichterkette meldet ihren Zustand nicht zurück. Home Assistant zeigt an,
  was zuletzt gesendet wurde (wird sie per Fernbedienung oder App umgeschaltet,
  stimmt die Anzeige nicht mehr).
- Wie das Bluetooth-Protokoll aufgebaut ist, steht in `MECHANICS.md`.
