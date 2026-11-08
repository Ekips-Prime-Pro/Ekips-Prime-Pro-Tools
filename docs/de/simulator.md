# Simulator – Entwurf

> **Stand:** Entwurf und Gerüst (`crates/sim`). Der Simulator wird gebaut,
> sobald das Core-Repository die virtuelle Maschine (VM) hat, die übersetzte
> Programme ausführt. Diese Seite beschreibt den Plan.

*English version: [../en/simulator.md](../en/simulator.md)*

## Ziele

1. **Testen ohne Roboter.** Schüler starten ihr Programm mit
   `ekips run --sim mission.ekips` und sehen einen 2D-Roboter auf der
   FLL-Matte fahren.
2. **Gleiches Programm, gleiches Verhalten.** Ein Programm, das im Simulator
   funktioniert, soll sich auf dem Hub genauso verhalten (im Rahmen eines
   einfachen Physikmodells).
3. **Läuft überall:** Windows, Linux (x86_64 und aarch64 / Raspberry Pi),
   macOS und **im Browser** über WebAssembly, damit Schulen nichts
   installieren müssen.
4. **Reproduzierbar.** Gleiches Programm und gleiche Startposition ergeben
   immer dasselbe Ergebnis. So kann man den Simulator für Tests nutzen und
   Lösungen vergleichen.

Vorerst nicht geplant: 3D, realistische Zusammenstöße mit Missionsmodellen,
durchrutschende Räder, Simulation der LEGO-Firmware.

## Architektur

```text
 .ekips-Quelltext ──ekips-compiler──▶ Bytecode ──▶ ekips-vm (Core)
                                                      │
                                       Hardware-Trait (ekips-hub-api)
                                     ┌────────────────┴───────────────┐
                                     ▼                                ▼
                         SimHardware (diese Crate)          Firmware (auf dem Hub)
                                     │
                                   World
                   (Matte, Roboter, Geräte, Uhr)
                                     │
                      ┌──────────────┴──────────────┐
                      ▼                             ▼
          natives Frontend (Fenster,       Browser-Frontend
          `ekips run --sim`)               (WASM + Canvas)
```

- Die **VM** greift nie direkt auf Hardware zu. Sie ruft den Trait
  `Hardware` aus `ekips-hub-api` auf. Befehle, die Zeit brauchen
  (`drive_straight`, `motor_run_angle`, …), starten nur eine Bewegung; die VM
  fragt `*_done` ab und lässt in der Zwischenzeit andere Tasks laufen.
- **`SimHardware`** implementiert `Hardware` auf Basis einer `World`.
- **`World`** enthält die Matte, den Roboter (Position, Motoren, Sensoren)
  und eine simulierte Uhr. Sie rechnet in festen Schritten von
  `STEP_MS = 10` ms weiter.
- **Frontends** zeichnen nur die Welt und nehmen Eingaben entgegen
  (Start/Stopp, Roboter platzieren). Sie liegen in eigenen Crates, damit
  `ekips-sim` ohne plattformabhängige Abhängigkeiten für
  `wasm32-unknown-unknown` baut (wird in der CI geprüft).

### Hauptschleife

```text
loop {
    vm.run_until_blocked(&mut sim_hardware);   // Programm-Tasks ausführen
    world.step(STEP_MS);                       // Motoren, Roboter, Sensoren bewegen
    frontend.draw(&world);                     // nativ oder Browser, optional
}
```

Ohne Frontend („headless“) läuft die Schleife so schnell wie möglich. Das
wird für Tests benutzt und könnte später Missionen automatisch prüfen.

## Weltmodell

### Koordinaten

Dieselben Konventionen wie in `ekips-hub-api` (in `crates/sim` umgesetzt):

- Millimeter, Grad, Millisekunden;
- Ursprung in der **linken unteren Ecke** der Matte, `x` nach rechts, `y` vom
  Schüler weg;
- Richtung `0°` zeigt entlang `+y`; **positiv = im Uhrzeigersinn**, wie
  `hub.heading`.

### Matte

- Standardgröße: FLL-Matte, etwa **2362 mm × 1143 mm** (`Mat::FLL`).
- Ein Hintergrundbild (PNG/JPEG der Saison-Matte) wird darauf skaliert.
  Bilder lädt das Frontend und übergibt sie als RGB-Pixel, damit die
  Kern-Crate ohne Bild-Decoder auskommt.
- Außerhalb der Matte zählt der Tischrand als Wand für den Abstandssensor.

### Roboter

- Wird vom Programm selbst eingerichtet: `new DriveBase(left, right, 56mm, 112mm)`
  liefert Raddurchmesser und Spurweite.
- Bewegung: Differentialantrieb (`Pose::advance`, schon umgesetzt): Beide
  Räder rollen pro Schritt eine Strecke, der Roboter folgt dem entstehenden
  Bogen.
- Motoren: Geschwindigkeit in % wird in Grad pro Sekunde umgerechnet
  (SPIKE-Mittelmotor etwa 1000 °/s bei 100 %), mit einer einfachen
  Beschleunigungsrampe. Bewegungen enden genau am Ziel, damit die Ergebnisse
  reproduzierbar bleiben.
- Sensorpositionen relativ zur Robotermitte sind Teil einer
  **Roboterbeschreibung** (Standard: Farbsensor vorne, Abstandssensor vorne,
  Kraftsensor vorne). Später können Schüler sie bearbeiten.

### Sensoren

| Sensor | Simulation |
|---|---|
| Farbsensor | Liest das Mattenbild unter dem Sensor (Mittelwert einer kleinen Fläche) und ordnet den RGB-Wert der nächsten `Color` und einer Reflexion in % zu |
| Abstandssensor | Strahl vom Sensor in Fahrtrichtung gegen den Tischrand und optionale Hindernisse (Rechtecke); `None` über 200 cm |
| Kraftsensor | `pressed`, wenn die Vorderseite des Roboters den Tischrand oder ein Hindernis berührt |
| Gyro (`hub.heading`) | Die exakte simulierte Richtung (später evtl. mit leichtem Rauschen) |
| Akku | Konstant 100 % |

### Ausgaben des Hubs

`hub.show`, `hub.beep` und `console.log` werden im Frontend angezeigt
(Vorschau der Lichtmatrix, Ton oder Hinweis im Protokoll, Konsolenbereich).

## Frontends

| Frontend | Technik (Vorschlag) | Hinweise |
|---|---|---|
| Natives Fenster | `minifb` oder `pixels`/`winit` – klein, reines Rust, läuft auf dem Raspberry Pi | Wird von `ekips run --sim` gestartet. Muss auch ohne GPU funktionieren. |
| Browser | `wasm-bindgen` + HTML-Canvas | Als statische Webseite; Editor und Simulator auf einer Seite. |
| Headless | keins | `ekips run --sim --headless` gibt einen Verlauf aus (Zeit, Position, Protokoll) für Tests. |

## Texte für Schüler

Alle Meldungen (z. B. „Der Roboter hat die Matte verlassen“, „An Port C ist
kein Motor“) gibt es auf Englisch und Deutsch, wie überall in den Tools.
Hardwarefehler kommen als `HardwareError` und werden in der CLI bzw. im
Frontend übersetzt.

## Phasen

1. **Jetzt:** Gerüst – Matte, Position, Fahrkinematik, WASM-Build in der CI.
2. **Sobald die Core-VM existiert:** `World`, `SimHardware`, Headless-Runner,
   `ekips run --sim --headless`.
3. Frontend mit nativem Fenster.
4. Browser-Frontend (WASM) mit Mattenbildern.
5. Editor für die Roboterbeschreibung, Hindernisse, Missionsmodelle.

## Offene Fragen

- Genaues Motormodell (Geschwindigkeitskurve, Beschleunigung) – am echten Hub messen.
- Sollen Mattenbilder mit den Tools ausgeliefert werden (Lizenz der FLL-Matten)?
- Wie werden Hindernisse/Missionsmodelle beschrieben (einfache JSON-Datei?)?
