# Installation und Releases

*English version: [../en/install.md](../en/install.md)*

## Herunterladen (empfohlen)

Fertige Programme gibt es auf der
[Release-Seite](https://github.com/Ekips-Prime-Pro/Ekips-Prime-Pro-Tools/releases).
Wähle die Datei für deinen Computer:

| Computer | Datei |
|---|---|
| Windows (die meisten PCs) | `ekips-tools-<version>-x86_64-pc-windows-msvc.zip` |
| Windows auf ARM | `ekips-tools-<version>-aarch64-pc-windows-msvc.zip` |
| Mac mit Apple-Chip (M1, M2, …) | `ekips-tools-<version>-aarch64-apple-darwin.tar.gz` |
| Mac mit Intel-Chip | `ekips-tools-<version>-x86_64-apple-darwin.tar.gz` |
| Linux (die meisten PCs) | `ekips-tools-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Raspberry Pi (64-Bit Raspberry Pi OS) | `ekips-tools-<version>-aarch64-unknown-linux-musl.tar.gz` |

Jedes Archiv enthält `ekips` (Kommandozeilenprogramm) und `ekips-lsp`
(Language Server). Entpacke es und füge den Ordner zu deinem `PATH` hinzu,
oder rufe die Programme mit vollem Pfad auf. Mit `SHA256SUMS.txt` kannst du
prüfen, ob der Download vollständig ist.

Unter macOS kann der erste Start blockiert werden, weil die Programme nicht
signiert sind. Erlaube sie unter *Systemeinstellungen → Datenschutz &
Sicherheit* oder führe `xattr -d com.apple.quarantine ekips ekips-lsp` aus.

Die Linux-Programme sind statisch gelinkt und laufen auf jeder Distribution.

## Aus dem Quellcode bauen

Du brauchst [Rust](https://rustup.rs) (stable).

```sh
git clone https://github.com/Ekips-Prime-Pro/Ekips-Prime-Pro-Tools
cd Ekips-Prime-Pro-Tools
cargo install --path crates/cli    # installiert `ekips`
cargo install --path crates/lsp    # installiert `ekips-lsp`
```

## Ein Release erstellen (für Maintainer)

1. `version` in der Haupt-`Cargo.toml` (`[workspace.package]`) anpassen und
   committen.
2. Einen Tag, der mit `v` beginnt, erstellen und pushen:
   ```sh
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. Der Workflow `.github/workflows/release.yml` baut alle sechs Zielplattformen,
   packt sie mit `README.md` und `LICENSE`, schreibt `SHA256SUMS.txt` und
   veröffentlicht ein GitHub-Release mit automatisch erzeugten Release Notes.
   Tags mit `-` (z. B. `v0.2.0-beta.1`) werden zu Vorab-Releases.

Ein Release für einen schon vorhandenen Tag kann über *Actions → Release →
Run workflow* neu gebaut werden.

### Abhängigkeit vom Core

`ekips-compiler` kommt als Git-Abhängigkeit aus dem Core-Repository (Haupt-
`Cargo.toml`). Zurzeit ist ein Commit festgelegt (`rev = …`). Sobald der Core
Tags veröffentlicht (`v0.2.0`, …), auf `tag = "v0.2.0"` umstellen und
`cargo update -p ekips-compiler` ausführen.
