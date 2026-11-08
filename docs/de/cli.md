# Das Kommandozeilenprogramm `ekips`

`ekips` prüft deine Ekips-Prime-Pro-Programme auf Fehler. Später wird es
sie auch im Simulator ausführen und auf den Hub übertragen.

*English version: [../en/cli.md](../en/cli.md)*

## Programme prüfen: `ekips check`

```sh
ekips check mission.ekips
ekips check mission1.ekips mission2.ekips
ekips check .              # alle .ekips-Dateien in diesem Ordner und seinen Unterordnern
```

Wenn alles stimmt, siehst du:

```text
mission.ekips: OK, keine Fehler gefunden.
```

Wenn etwas falsch ist, zeigt `ekips`, wo der Fehler ist und wie du ihn
behebst:

```text
Fehler: Das Wort `cosnt` verstehe ich hier nicht.
 --> mission.ekips:1:1
  |
1 | cosnt arm = new Motor(Port.C);
  | ^^^^^
  = Hilfe: Meintest du `const`?

mission.ekips: 1 Fehler
```

- Die Zeile `--> mission.ekips:1:1` bedeutet: Datei `mission.ekips`, Zeile 1, Spalte 1.
- Die `^^^^^` markieren die genaue Stelle.
- Die Zeile `Hilfe:` schlägt eine Lösung vor.

Wenn du mehrere Dateien prüfst, steht am Ende eine Zusammenfassung.

### Optionen von `check`

| Option | Bedeutung |
|---|---|
| `--ast` | Zusätzlich den Syntaxbaum ausgeben, also wie der Compiler das Programm verstanden hat (für Profis) |

## Optionen für alle Befehle

| Option | Bedeutung |
|---|---|
| `--lang en` / `--lang de` | Sprache aller Meldungen. Ohne diese Option wird die Sprache deines Computers benutzt (Deutsch, wenn er auf Deutsch eingestellt ist, sonst Englisch). |
| `--color auto\|always\|never` | Farben in der Ausgabe. `auto` (Standard) benutzt Farben nur im Terminal. Die Umgebungsvariable `NO_COLOR` schaltet Farben ebenfalls ab. |
| `-h`, `--help` | Hilfe anzeigen (in der gewählten Sprache) |
| `-V`, `--version` | Version anzeigen |

Die Umgebungsvariable `EKIPS_LANG` (`en` oder `de`) stellt die Sprache
ebenfalls ein; `--lang` hat Vorrang.

## Geplante Befehle

Diese Befehle gibt es schon, sie zeigen aber nur eine Meldung, dass sie noch
nicht verfügbar sind. An deinem Computer oder Hub wird nichts verändert.

| Befehl | Wird später |
|---|---|
| `ekips run --sim <datei>` | das Programm im Simulator ausführen |
| `ekips run <datei>` | das Programm auf dem Hub ausführen |
| `ekips upload <datei>` | das Programm auf den Hub übertragen |
| `ekips monitor` | anzeigen, was der Hub ausgibt (`console.log`) |
| `ekips flash-firmware` | die Ekips-Prime-Pro-Firmware auf den Hub spielen |
| `ekips backup-firmware [datei]` | die aktuelle Firmware des Hubs sichern, damit man sie wiederherstellen kann |

## Exit-Codes

Nützlich für Skripte und Editoren:

| Code | Bedeutung |
|---|---|
| `0` | Alles in Ordnung |
| `1` | Mindestens ein Programm hat einen Fehler |
| `2` | Falscher Aufruf, oder eine Datei konnte nicht gelesen werden |
| `3` | Der Befehl ist noch nicht verfügbar |

Die Hilfetexte von `ekips` sind übersetzt. Meldungen über einen *falschen
Aufruf* (zum Beispiel eine unbekannte Option) kommen von der Bibliothek
`clap` und sind nur auf Englisch; im deutschen Modus folgt ein deutscher
Hinweis.
