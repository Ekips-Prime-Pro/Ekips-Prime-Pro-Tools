# Language Server `ekips-lsp`

`ekips-lsp` bringt deinem Editor Ekips Prime Pro bei. Die VS-Code-Erweiterung
(eigenes Repository *Ekips-Prime-Pro-VSCode*) startet ihn automatisch; selbst
starten musst du ihn normalerweise nie.

*English version: [../en/lsp.md](../en/lsp.md)*

## Was du im Editor bekommst

| Funktion | Was sie macht |
|---|---|
| **Fehler schon beim Tippen** | Nach jeder Änderung wird das Programm geprüft. Fehler werden unterstrichen, mit derselben schülerfreundlichen Meldung und demselben Tipp wie bei `ekips check`. |
| **Hover (Erklärung beim Darüberfahren)** | Zeige mit der Maus auf ein Wort, um zu sehen, was es bedeutet: Schlüsselwörter (`await`, `while`, …), Einheiten (`20cm`, `90deg`, `50%`), eingebaute Dinge (`Motor`, `hub`, `Port.A`, …), Methoden (`robot.drive`) und deine eigenen Funktionen und Variablen (mit der Zeile, in der sie definiert sind). |
| **Autovervollständigung** | Schlüsselwörter, eingebaute Klassen und Funktionen, deine eigenen Namen und **Snippets** (siehe unten). Nach einem Punkt werden die passenden Mitglieder angeboten: `robot.` → `drive`, `turn`, `run`, `stop`, wenn `robot` mit `new DriveBase(…)` erzeugt wurde; `Port.` → `A` … `F`. Nach einer Zahl kommen Einheiten: `20c` → `cm`. |

In Kommentaren und Texten (`"…"`) wird nichts vorgeschlagen.

### Snippets (Code-Vorlagen)

| Tippe | Du bekommst |
|---|---|
| `program` | Ein vollständiges Programm mit zwei Motoren, Fahrgestell und `main` |
| `main` | `async function main() { … }` |
| `function`, `async function` | Eine Funktion |
| `if`, `if else`, `while`, `for`, `for of` | Bedingungen und Schleifen |
| `motor`, `drivebase`, `colorsensor`, `distancesensor`, `forcesensor` | Hardware einrichten, mit Auswahl des Ports |
| `all` | `await Promise.all([…])` – Dinge gleichzeitig tun |
| `race` | Fahren, bis etwas passiert, dann anhalten |
| `waitUntil` | `await waitUntil(() => …)` |

Mit <kbd>Tab</kbd> springst du zur nächsten Stelle, die du ausfüllen musst.

## Sprache

Meldungen und Erklärungen gibt es auf Englisch oder Deutsch. Die Sprache wird
in dieser Reihenfolge bestimmt:

1. `initializationOptions.lang`, vom Editor geschickt (`"en"` oder `"de"`),
   z. B. aus einer Einstellung der VS-Code-Erweiterung
2. die Kommandozeilenoption `--lang en|de`
3. die Umgebungsvariable `EKIPS_LANG`
4. die Anzeigesprache des Editors (`locale` in der LSP-Anfrage `initialize`)
5. die Systemsprache; Englisch, wenn sie weder Englisch noch Deutsch ist

## Für Editor-Entwickler

```text
ekips-lsp [--stdio] [--lang en|de]
ekips-lsp --version
ekips-lsp --help
```

- Transport: stdin/stdout (`--stdio` wird akzeptiert und ignoriert).
- Text-Synchronisation: ganze Dokumente (`TextDocumentSyncKind.Full`).
- Positions-Kodierung: UTF-8, wenn der Client es anbietet, sonst UTF-16.
- Fähigkeiten: `textDocument/publishDiagnostics` (mit `version`),
  `textDocument/hover`, `textDocument/completion` (Auslösezeichen `.`;
  Snippets nur, wenn der Client `snippetSupport` meldet).
- Diagnosen haben `source: "ekips"`; der Tipp steht nach einem Zeilenumbruch
  hinter der Meldung (`help: …` / `Hilfe: …`).

Beispiel für VS Code (`vscode-languageclient`):

```ts
const serverOptions = { command: "ekips-lsp", args: ["--stdio"] };
const clientOptions = {
  documentSelector: [{ scheme: "file", language: "ekips" }],
  initializationOptions: { lang: vscode.env.language.startsWith("de") ? "de" : "en" },
};
```

Die Texte für Hover und Snippets stehen in `crates/lsp/src/knowledge.rs` und
folgen der Sprachreferenz im Core-Repository. Tests stellen sicher, dass
jedes Schlüsselwort und jede Einheit des Compilers erklärt ist und jedes
Snippet fehlerfrei geparst wird.
