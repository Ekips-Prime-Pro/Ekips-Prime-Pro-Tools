# Language server `ekips-lsp`

`ekips-lsp` makes your editor understand Ekips Prime Pro. The VS Code
extension (separate repository *Ekips-Prime-Pro-VSCode*) starts it
automatically; you normally never run it yourself.

*Deutsche Version: [../de/lsp.md](../de/lsp.md)*

## What you get in the editor

| Feature | What it does |
|---|---|
| **Problems while you type** | After every change the program is checked. Mistakes are underlined, with the same student-friendly message and hint as `ekips check`. |
| **Hover** | Point at a word to see what it means: keywords (`await`, `while`, …), units (`20cm`, `90deg`, `50%`), built-ins (`Motor`, `hub`, `Port.A`, …), methods (`robot.drive`) and your own functions and variables (with the line where they are defined). |
| **Autocompletion** | Keywords, built-in classes and functions, the names you defined, and **snippets** (see below). After a dot it offers the members: `robot.` → `drive`, `turn`, `run`, `stop` when `robot` was created with `new DriveBase(…)`; `Port.` → `A` … `F`. After a number it offers units: `20c` → `cm`. |

Nothing is suggested inside comments or texts (`"…"`).

### Snippets

| Type | You get |
|---|---|
| `program` | A complete program with two motors, a drive base and `main` |
| `main` | `async function main() { … }` |
| `function`, `async function` | A function |
| `if`, `if else`, `while`, `for`, `for of` | Conditions and loops |
| `motor`, `drivebase`, `colorsensor`, `distancesensor`, `forcesensor` | Setting up hardware, with a choice of ports |
| `all` | `await Promise.all([…])` – do things at the same time |
| `race` | Drive until something happens, then stop |
| `waitUntil` | `await waitUntil(() => …)` |

Press <kbd>Tab</kbd> to jump to the next place to fill in.

## Language

Messages and explanations are in English or German. The language is chosen
in this order:

1. `initializationOptions.lang` sent by the editor (`"en"` or `"de"`),
   e.g. from a setting of the VS Code extension
2. the command-line option `--lang en|de`
3. the environment variable `EKIPS_LANG`
4. the display language of the editor (`locale` in the LSP `initialize` request)
5. the system language; English if it is neither English nor German

## For editor integrators

```text
ekips-lsp [--stdio] [--lang en|de]
ekips-lsp --version
ekips-lsp --help
```

- Transport: stdin/stdout (`--stdio` is accepted and ignored).
- Text synchronisation: full documents (`TextDocumentSyncKind.Full`).
- Position encoding: UTF-8 if the client offers it, otherwise UTF-16.
- Capabilities: `textDocument/publishDiagnostics` (with `version`),
  `textDocument/hover`, `textDocument/completion` (trigger character `.`;
  snippets only if the client announces `snippetSupport`).
- Diagnostics have `source: "ekips"`; the hint is appended to the message
  after a line break (`help: …` / `Hilfe: …`).

Example for VS Code (`vscode-languageclient`):

```ts
const serverOptions = { command: "ekips-lsp", args: ["--stdio"] };
const clientOptions = {
  documentSelector: [{ scheme: "file", language: "ekips" }],
  initializationOptions: { lang: vscode.env.language.startsWith("de") ? "de" : "en" },
};
```

The texts for hover and snippets live in `crates/lsp/src/knowledge.rs` and
follow the language reference in the Core repository. Tests make sure every
keyword and unit of the compiler is documented and every snippet parses.
