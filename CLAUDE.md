# CLAUDE.md – Ekips Prime Pro Tools

Guidance for AI assistants (and humans) working in this repository.

## Project context

**Ekips Prime Pro** is a TypeScript-like robot language (no type annotations,
units on numbers such as `20cm`, `90deg`, `2s`, `50%`, `async`/`await` for
parallel actions) for the LEGO® SPIKE™ Prime hub. The users are **students
in grades 7–10** and FIRST® LEGO® League teams.

| Repository | Contents |
|---|---|
| [Ekips-Prime-Pro-Core](https://github.com/Ekips-Prime-Pro/ekips-prime-pro-core) | Language, compiler (`ekips-compiler`), VM, bytecode, hardware API |
| **Ekips-Prime-Pro-Tools** (this one) | `ekips` CLI, language server, simulator |
| Ekips-Prime-Pro-VSCode | VS Code extension; starts the `ekips-lsp` binary from this repo |

The language reference lives in the Core repo: `docs/language.md`. Read it
before changing anything that touches the language (keywords, snippets,
hover texts).

### Workspace layout

| Crate | Path | Output | Purpose |
|---|---|---|---|
| `ekips-cli` | `crates/cli` | binary `ekips` | `check`, later `run --sim`, `upload`, `monitor`, `flash-firmware`, `backup-firmware` |
| `ekips-lsp` | `crates/lsp` | binary `ekips-lsp` | Language server (stdio): diagnostics, hover, completion |
| `ekips-sim` | `crates/sim` | library | 2D robot simulator (skeleton until the Core has a VM) |

`ekips-compiler` is a Git dependency pinned to a commit `rev` in the root
`Cargo.toml`. Switch to tags (`tag = "v0.2.0"`, …) once the Core publishes them.

## Rules

1. **English only** in code, comments, identifiers, commit messages and PR texts.
2. **Document every feature immediately** under `docs/`, **always in both
   languages**: `docs/en/…` and `docs/de/…` with the same file names. Update
   `docs/README.md` when adding a page.
3. **Student-facing output is bilingual** (English and German). The language
   is chosen with `--lang en|de`; the default comes from the system language
   (`EKIPS_LANG` overrides it). Never add a user-visible message in only one
   language. Keep wording simple and friendly (grades 7–10).
4. **Cross-platform:** everything must work on Windows, Linux (x86_64 and
   aarch64 / Raspberry Pi) and macOS. No shell-specific behaviour, use
   `std::path`, avoid native dependencies that do not cross-compile.
5. **Quality gates** (also enforced by CI on all three operating systems):
   ```sh
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
6. Small, focused commits; push regularly.

## Conventions

- Exit codes of `ekips`: `0` OK, `1` problems found in the program,
  `2` usage or file error, `3` command not available yet.
- Messages live next to the code that prints them as `(en, de)` pairs; use
  the `t(lang, en, de)` helper pattern instead of separate files.
- Releases: pushing a tag `v*` builds binaries for all platforms
  (`.github/workflows/release.yml`).
